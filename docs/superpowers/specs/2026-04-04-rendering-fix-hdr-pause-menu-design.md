---
title: Rendering Fix, HDR Pipeline & Pause Menu Redesign
date: 2026-04-04
status: draft
---

# Rendering Fix, HDR Pipeline & Pause Menu Redesign

## Context

The Asteroids game (Rust + wgpu, ported from OCaml) has a broken rendering pipeline. Everything renders in black and white with XOR-like artifacts. Root cause: `to_hdr_rgba` writes alpha as 255.0 instead of 1.0 into an `Rgba16Float` offscreen texture using `BlendState::ALPHA_BLENDING`. The GPU blend equation computes `src.rgb * 255 - dst.rgb * 254`, producing negative luminance and visual corruption.

Additionally, the tonemapper uses a hard spectral redirect (`redirect_spectre_wide`) that crushes bright HDR colors to white, and the game has no HDR display support despite using an HDR-capable internal pipeline.

This spec addresses all three concerns in a single coherent change.

## Part 1: Rendering Fixes

### 1a. Alpha Normalization (Critical)

**File**: `src/game.rs`

Change `to_hdr_rgba` alpha from `255.0` to `1.0`:

```rust
pub(crate) fn to_hdr_rgba(color: HdrColor) -> [f32; 4] {
    [color.r as f32, color.g as f32, color.b as f32, 1.0]
}
```

**File**: `src/shaders/postprocess.wgsl`

Normalize alpha output in the tonemap return: output `1.0` for alpha instead of passing through the raw value.

**Effect**: Fixes B&W/XOR artifacts, restores SDF anti-aliasing (smoothstep alpha now produces correct 0-1 blend factors).

### 1b. Postprocess Alpha Passthrough

The postprocess shader currently passes `hdr_color.a` through without normalization. With the alpha fix, offscreen texture alpha will be 1.0 (or SDF smoothstep values in 0-1), so the postprocess output should emit `1.0` for opaque surface output.

### 1c. Dead Code Cleanup

- Remove unused `zoom_factor` uniform from `world.wgsl` and its buffer/bind-group-entry in `rendering/mod.rs`.

## Part 2: HDR Pipeline & Tonemapping

### Surface Format Selection

Two modes based on HDR toggle:

| Mode | Surface Format | Color Space | Gamma |
|------|---------------|-------------|-------|
| SDR (HDR off) | `Bgra8UnormSrgb` | sRGB | Hardware sRGB (automatic) |
| HDR (HDR on) | `Rgba16Float` | scRGB (WCG) | Linear (no conversion) |

Switching HDR on/off requires reconfiguring the surface and recreating affected pipelines that reference the surface format (postprocess, HUD).

### Tonemapping Strategy

**Core principle**: Only tonemap what exceeds max brightness. Everything below passes through untouched.

**Three user-configurable values** (stored in `ExposureConfig` or a new `HdrConfig`):

| Parameter | Default | Unit | Purpose |
|-----------|---------|------|---------|
| `hud_nits` | 155 | nits | HUD element brightness |
| `paper_white` | 200 | nits | Maps "1.0" intensity (the reference white level) |
| `max_brightness` | 1000 | nits | Display peak brightness; tonemap ceiling |

**HDR mode tonemap flow:**

1. Input: HDR color in 0-2000+ range from offscreen texture
2. Apply `add_color` and `mul_color` (stage tint/flash effects)
3. Scale: `color_nits = color * (paper_white / 255.0)` — maps the internal 0-255 range to nits
4. **Passthrough**: if all channels ≤ `max_brightness`, output as-is (preserves full saturation and WCG)
5. **Soft redirect**: only for channels exceeding `max_brightness`, use smoothstep-gated spectral bleed to redistribute excess energy into neighboring channels
6. Final clamp to `max_brightness`, then normalize: `output = color_nits / max_brightness`

**SDR mode tonemap flow:**

1. Same as HDR steps 1-2
2. Soft redirect at 255 threshold (the existing `tonemap_spectral_bleed` logic, variant 2)
3. Clamp to [0, 255], divide by 255
4. Output to sRGB surface (hardware applies gamma)

### WCG (Wide Color Gamut)

When HDR is enabled, the `Rgba16Float` surface format supports scRGB, which can represent colors outside the sRGB gamut. The tonemap preserves these vivid colors:

- No spectral redistribution for in-gamut colors
- Only redistribute energy that exceeds `max_brightness` per-channel
- The display's native gamut (typically P3 or Rec.2020 on HDR-capable monitors) determines what is physically visible

### HUD Brightness

The HUD renders directly to the swapchain (bypasses postprocess). In HDR mode:

- HUD colors are scaled by `hud_nits / max_brightness` before output
- The HUD shader's `/255.0` normalization maps to sRGB white; multiply by `hud_nits / paper_white` for correct HDR brightness

In SDR mode, HUD rendering is unchanged (divide by 255, sRGB surface handles gamma).

### Shader Changes

**`postprocess.wgsl`:**

- Switch active variant to soft redirect (variant 2 base, modified)
- Add uniforms: `paper_white`, `max_brightness`, `hdr_enabled`
- When `hdr_enabled`:
  - Scale input by `paper_white / 255.0`
  - Passthrough below `max_brightness`
  - Soft smoothstep bleed only above `max_brightness`
  - Output = `result / max_brightness`
- When not `hdr_enabled`:
  - Existing soft redirect at 255 threshold
  - Output = `result / 255.0`

**`hud.wgsl`:**

- Add uniform: `hud_brightness_scale` (= `hud_nits / max_brightness` in HDR, `1.0` in SDR)
- Multiply output RGB by `hud_brightness_scale`

## Part 3: Pause Menu Redesign

### Layout

Replace the current grid layout with a single vertical scrollable list. All options stack top-to-bottom in a single column within the 16:9 safe zone.

### Option Types

Three types of menu entries:

1. **Toggle**: On/off boolean. Click to flip. Displays green (on) or red (off) background.
2. **Cycle**: Click to advance through discrete values. Displays current value.
3. **Slider**: Horizontal bar. Mouse drag to adjust continuous value. Displays numeric value.

### Options List (top to bottom)

```
[Action buttons]
  Resume
  New Game
  Quit
--- separator ---
[Gameplay toggles]
  Advanced Hitbox        (toggle)
  Smoke Particles        (toggle)
  Screenshake            (toggle)
  Light Flashes          (toggle)
  Chunk Particles        (toggle)
  Color Effects          (toggle)
--- separator ---
[Rendering options]
  MSAA                   (cycle: Off / x2 / x4)
  SMAA                   (toggle)
  HDR                    (toggle)
--- separator (visible only when HDR is on) ---
[HDR Configuration]
  HUD Nits               (slider: 50-500, default 155)
  Paper White            (slider: 80-500, default 200)
  Max Brightness         (slider: 400-2000, default 1000)
```

HDR config options are only visible/interactable when HDR toggle is on.

### Navigation

| Input | Action |
|-------|--------|
| Mouse wheel up/down | Scroll list |
| W / Arrow Up | Move selection up |
| S / Arrow Down | Move selection down |
| Mouse hover | Highlight entry |
| Left click | Toggle/cycle, or start slider drag |
| Mouse drag (on slider) | Adjust slider value |
| A / Arrow Left | Decrease slider value (step) |
| D / Arrow Right | Increase slider value (step) |
| Escape | Resume game |

**Hard stop** at list edges (no wrap-around).

### Rendering

Each entry is a horizontal bar spanning the menu width:
- Fixed height per entry, calculated from safe zone
- Selected/hovered entry has a highlight border or brighter background
- Toggles show green/red fill
- Cycles show the current value text
- Sliders show a horizontal track with a draggable thumb and numeric value

The menu title "ASTEROIDS" remains at the top. The list begins below it and scrolls if it exceeds the visible area.

### Scroll Mechanics

- Visible area fits N entries (calculated from safe zone height and entry height)
- Scroll offset is an integer (number of entries scrolled past the top)
- Keyboard selection auto-scrolls to keep the selected entry visible
- Mouse wheel scrolls by 1 entry per tick

## Audit Findings (address opportunistically)

These were found during the rendering audit and should be fixed when touching the relevant code:

| Priority | Issue | Location | Fix |
|----------|-------|----------|-----|
| HIGH | SDF pipelines MSAA=1 vs world MSAA=4 | `rendering/mod.rs:489,523` | Make MSAA configurable (ties into pause menu MSAA option) |
| MEDIUM | Dead `zoom_factor` uniform | `world.wgsl:12`, `rendering/mod.rs:188` | Remove |
| LOW | 8-byte uniform buffer below alignment recommendation | `rendering/mod.rs:182` | Pad to 16 bytes |

## Implementation Order

1. **Alpha fix** → immediate visual restoration (test with `scenarios/test_visual.ron`)
2. **Switch to soft redirect** → better color handling
3. **Pause menu vertical list** → UI foundation for settings
4. **HDR config values** → paper_white, max_brightness, hud_nits as uniforms
5. **Surface format switching** → SDR (sRGB) vs HDR (Rgba16Float) based on toggle
6. **HDR tonemap path** → passthrough below max_brightness, soft redirect above
7. **HUD brightness scaling** → hud_nits support
8. **MSAA configurability** → cycle option in pause menu
9. **Cleanup** → dead zoom_factor, alignment padding

## Dependencies

- Step 1 is independent and can be tested immediately
- Steps 2-3 are independent of each other
- Steps 4-7 form the HDR pipeline (sequential)
- Step 8 depends on step 3 (pause menu must exist)
- Step 9 is independent
