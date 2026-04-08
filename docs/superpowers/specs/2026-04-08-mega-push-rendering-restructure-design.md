# Mega-Push: Layered Compositing, Code Restructure, AA Pipeline, Capture Tooling

**Date**: 2026-04-08  
**Status**: Design  
**Scope**: Rendering restructure, code cleanup, AA pipeline, capture tooling, groundwork for distortion fields and i18n

---

## 1. Layered Compositing Renderer

### Problem

The current 2-pass renderer (polygon pass → SDF pass) forces ALL SDF shapes on top of ALL polygons. Stars render over asteroids, smoke renders over ship. This is architectural — not fixable without changing the pass structure.

### Design

Replace the 2-pass system with an **ordered layer system** inspired by compositing software. Each layer has a defined draw order. Within a layer, objects are drawn with appropriate AA.

**Layer stack (back to front):**

| Layer | Content | Type | AA |
|-------|---------|------|----|
| 0 | Background | Clear color | None |
| 1 | Star trails | SDF capsules | Analytic (smoothstep) + ssaa_global |
| 2 | Bullet trails | SDF capsules | Analytic + ssaa_global |
| 3 | Smoke | SDF circles | Analytic + ssaa_global |
| 4 | Polygons: asteroids, fragments, ship | Polygon batch | ssaa_polygon × ssaa_global |
| 5 | Effects: explosion circles, sparkles, future FX | SDF circles | Analytic + ssaa_global |
| 6 | HUD | Polygon + SDF glyphs | ssaa_hud |

**Postprocess (tonemap)** happens after layer 5, before HUD (layer 6). This matches current behavior — HUD applies its own tonemap via HudUniforms.

### Render target layout

- **offscreen**: `Rgba16Float`, single-sample — the compositing surface. Dimensions: `native × ssaa_global`.
- **polygon_texture**: `Rgba16Float`, single-sample — polygon-only SSAA target. Dimensions: `offscreen × ssaa_polygon`. Only allocated when `ssaa_polygon > 1`.
- **hud_texture**: surface format, single-sample — HUD-only SSAA target. Dimensions: `native × ssaa_hud`. Only allocated when `ssaa_hud > 1`.
- **swapchain**: final output after postprocess

**Per-frame sequence:**

```
1. Clear offscreen to background color (at native × ssaa_global resolution)
2. Layer 1: SDF capsule pass — star trails → offscreen (Load, additive blend)
3. Layer 2: SDF capsule pass — bullet trails → offscreen (Load, additive blend)
4. Layer 3: SDF circle pass — smoke → offscreen (Load, alpha blend)
5. Layer 4: If ssaa_polygon > 1:
     a. Clear polygon_texture to transparent
     b. Draw polygon entities → polygon_texture (at higher resolution)
     c. Downsample + alpha-blend polygon_texture → offscreen (composite pass)
   If ssaa_polygon == 1:
     Draw polygon entities directly → offscreen (Load, alpha blend)
6. Layer 5: SDF circle pass — explosions, sparkles → offscreen (Load, additive blend)
7. Postprocess pass: tonemap + downsample (if ssaa_global > 1) offscreen → swapchain
8. Layer 6: If ssaa_hud > 1:
     a. Clear hud_texture to transparent
     b. Draw HUD → hud_texture (at higher resolution)
     c. Downsample + alpha-blend hud_texture → swapchain
   If ssaa_hud == 1:
     Draw HUD directly → swapchain (Load, alpha blend)
```

Note: Layers 1-3 and layer 5 can potentially be batched into fewer draw calls if they share the same pipeline (SDF circles vs capsules). The layer ordering is logical — the implementation may merge consecutive same-type layers into one draw call with ordered instance data.

### Additive blending

Capsule layers (1, 2) and explosion circles (layer 5) should support **additive blend mode** (`src_factor: One, dst_factor: One`). This is configured per-layer, not per-instance. The SDF pipeline already produces premultiplied alpha output — additive blending just changes the blend state.

A second SDF pipeline variant with additive blend state is created at init time. Layers select which pipeline to bind.

### AA: Unified SSAA (replaces MSAA)

MSAA is removed entirely. All anti-aliasing uses a unified mechanism: **render to a larger texture, downsample with a pluggable filter, composite**.

Three independent SSAA parameters, all defaulting to 1 (off):

| Parameter | Scope | Render target | Downsample onto |
|-----------|-------|---------------|-----------------|
| `ssaa_global` | All layers (0-5) | offscreen at `native × factor` | swapchain (in postprocess shader) |
| `ssaa_polygon` | Layer 4 only | polygon texture at `offscreen × factor` | offscreen (alpha-blend pass) |
| `ssaa_hud` | Layer 6 only | HUD texture at `native × factor` | swapchain (alpha-blend pass) |

Parameters stack multiplicatively: global 2× + polygon 2× = polygons effectively at 4×.

**Downsample filter architecture:**

```rust
enum DownsampleFilter {
    Box,      // Simple average — initial implementation
    Lanczos,  // Sharper, slight ringing — future
}
```

One filter shared across all three SSAA paths. The postprocess shader and composite passes use the same `sample_box_filter` (or future Lanczos) function.

**Polygon-only SSAA compositing (layer 4):**
1. Clear polygon texture to transparent (at `offscreen × ssaa_polygon` resolution)
2. Render polygons at higher resolution
3. Downsample to offscreen resolution
4. Alpha-blend onto offscreen — edge pixels have fractional alpha from coverage, compositing correctly against SDF background

No MSAA texture, no skybox blit, no resolve quirks. Same mechanism for all three SSAA knobs.

**Pause menu entries:**
- "GLOBAL AA" cycle: OFF / 2X / 3X / 4X
- "POLYGON AA" cycle: OFF / 2X / 3X / 4X
- "HUD AA" cycle: OFF / 2X / 3X / 4X

---

## 2. Code Restructure

### game.rs (1572 lines) → split

| New file | Extracted from | Content |
|----------|---------------|---------|
| `src/update.rs` | `game.rs` | `update_game`, `apply_inertia_all`, `wrap_entities`, `decay_smoke`, `enforce_particle_budgets`, `update_visual_aim` — all per-frame entity update logic |
| `src/spawning.rs` | `objects.rs` | `spawn_ship`, `spawn_projectile`, `spawn_asteroid`, `spawn_explosion`, `spawn_fire`, `fragment_asteroid`, `spawn_stars` — all spawn/factory functions (objects.rs also 910 lines) |
| `src/game.rs` (residual) | — | `GameState` struct, `GamepadState`, `render_frame`, high-level orchestration only (~200-300 lines) |

`input.rs` already exists (292 lines) — no change needed.

### rendering/mod.rs (1464 lines) → split

| New file | Content |
|----------|---------|
| `src/rendering/pipeline.rs` | Pipeline creation (polygon, SDF circle, SDF capsule, postprocess, HUD), bind group layouts, shader loading |
| `src/rendering/textures.rs` | `create_offscreen_texture`, SSAA texture creation, scaling logic, texture management |
| `src/rendering/mod.rs` (residual) | `Renderer2D` struct, `new`, `resize`, `render` (layer orchestration), surface management (~400-500 lines) |

### rendering/hud.rs (1093 lines) → split

| New file | Content |
|----------|---------|
| `src/glyphs.rs` | `shape_char`, all glyph polygon data, `displacement`, `displace_shape` — the font system (also groundwork for i18n glyph extraction) |
| `src/rendering/hud.rs` (residual) | `render_hud`, `render_string`, `render_bar`, `draw_heart`, UI layout logic (~400-500 lines) |

### Deduplication targets

- Bind group layout duplication in `Renderer2D::resize()` — extract to shared helper
- Any repeated buffer-building patterns in world.rs

---

## 3. AA Pipeline — Unified SSAA

MSAA is removed entirely. All anti-aliasing uses one unified mechanism: **render to a larger texture, downsample with a pluggable filter, composite.**

### Three independent SSAA parameters

| Parameter | Config field | Scope | Default |
|-----------|-------------|-------|---------|
| Global AA | `ssaa_global: u32` | All layers (0-5) — scales offscreen target | 1 (off) |
| Polygon AA | `ssaa_polygon: u32` | Layer 4 only — polygon edges | 1 (off) |
| HUD AA | `ssaa_hud: u32` | Layer 6 only — text/UI crispness | 1 (off) |

Values: 1 (off), 2, 3, 4. Parameters stack multiplicatively.

### Downsample filter architecture

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DownsampleFilter {
    Box,      // Simple average — initial implementation
    Lanczos,  // Sharper, slight ringing — future
}
```

One filter shared across all three SSAA paths. The postprocess shader and composite passes use the same `sample_box_filter` function (or future Lanczos kernel).

- **Box filter**: Average `factor × factor` texels per output pixel. Loop in shader.
- **Lanczos filter**: Future. Larger kernel (Lanczos-3 = 6×6 tap), passed via uniform buffer.

### Pause menu entries

- "GLOBAL AA" cycle: OFF / 2X / 3X / 4X
- "POLYGON AA" cycle: OFF / 2X / 3X / 4X
- "HUD AA" cycle: OFF / 2X / 3X / 4X

### Typical usage

- Gameplay: Polygon AA 2× (cheap, smooths ship/asteroid edges)
- Quality: Global AA 2× + Polygon AA 2× (4× effective on polygons, 2× on SDF)
- Screenshots: Global AA 4× (maximum quality, all layers)
- HUD readability: HUD AA 2× (crisper text independently of game AA)

---

## 4. Capture Tooling

### Screenshot (F12)

- Reads back swapchain surface after postprocess (native resolution, 8-bit)
- Saves as PNG via `image` crate
- Path: `screenshots/asteroids_YYYYMMDD_HHMMSS.png`
- CLI: `--screenshot-at-frame N` for automated captures

### Video Capture (F10 toggle)

- Frame-sequence PNGs in `captures/session_YYYYMMDD_HHMMSS/frame_NNNNN.png`
- Toggle on/off with F10
- `VideoCapture` state machine: Idle → Recording → Idle
- Same readback as screenshot, every frame

### Scenario Recording (pause menu)

- "Record Scenario" toggle in pause menu
- Forces fixed-dt mode when enabled
- Records inputs via existing `InputRecorder`
- New `GameStateSnapshot` struct for optional per-frame state dumps
- Replayable via `--scenario` CLI flag

### New module: `src/capture.rs`

- `screenshot_path()`, `capture_session_dir()`, `frame_path()`
- `readback_texture_rgba8()` — reads swapchain surface (always 8-bit post-tonemap)
- `save_png()` — async-friendly PNG write
- `VideoCapture` struct with start/stop/capture_frame

### Dependencies

- `image = "0.25"` (PNG feature only)
- `chrono = "0.4"` (timestamp formatting)

---

## 5. Visual Fixes

### Trail brightness re-tuning

Trails are too dim after the brightness conservation formula (`π·r²/(π·r²+2·r·L)`). The formula is correct but base brightness values need adjustment. This is a parameter tuning pass on `TrailConfig` values for star trails and bullet trails — not an architectural change.

### SDR / HDR parity

SDR mode appears dimmer than HDR at the same exposure setting. This is a separate issue from capsule brightness. Investigate whether the tonemap path differs between SDR and HDR modes, and adjust the SDR exposure curve to match perceived brightness.

### Capsule brightness

Capsule (trail) rendering brightness needs independent adjustment. Currently capsules share color values with their source entities — they may need a brightness multiplier or separate color configuration in `TrailConfig`.

### Exposure slider merge

Merge `exposure` and `game_exposure` into a single user-facing "Exposure" slider. `game_exposure` becomes an internal-only multiplier (default 1.0, not exposed in pause menu). The postprocess shader continues to receive both values — only the UI changes.

---

## 6. Groundwork: Distortion Fields

**Goal**: Lay infrastructure, don't implement behavior.

### New file: `src/field.rs`

```rust
pub enum FieldSourceKind {
    ShockwaveRing { speed: f64, width: f64, pressure: f64 },
    GravityWell { strength: f64, radius: f64 },
    Vortex { angular_speed: f64, radius: f64 },
    WindZone { direction: Vec2, strength: f64, radius: f64 },
}

pub struct FieldSource {
    pub kind: FieldSourceKind,
    pub position: Vec2,
    pub age: f64,
    pub lifetime: f64,
}

pub struct FieldSample {
    pub wind: Vec2,
    pub gravity: Vec2,
    pub time_dilation: f64,  // 1.0 = normal
}

pub fn evaluate_field(position: Vec2, sources: &[FieldSource]) -> FieldSample {
    // Stub — returns zero wind, zero gravity, 1.0 time_dilation
    todo!()
}
```

- Types only, `evaluate_field` returns neutral values
- `FieldSource` storage added to `GameState` as `Vec<FieldSource>`
- No integration into update loop yet — just the data structures

---

## 7. Groundwork: i18n & Glyph System

**Goal**: Extract glyph system, create locale module skeleton.

### Glyph extraction

- Move `shape_char` + all glyph polygon data from `hud.rs` → `src/glyphs.rs`
- Add `pub fn glyph(c: char) -> Option<Vec<Polygon>>` as the new entry point
- Three-tier lookup: override table → accent composition → shape_char → filled-square fallback
- `render_string` in `hud.rs` calls `glyphs::glyph(c)` instead of `shape_char`
- Remove `to_ascii_uppercase()` coercion in `render_string`

### Locale skeleton

- New file: `src/locale.rs`
- `Locale` struct with `HashMap<String, String>`
- `Locale::load(path)` — reads RON locale file
- `Locale::get(key)` — returns translated string with English fallback
- `detect_system_locale()` — stub, returns "en"
- `locales/en.ron` — empty/minimal English locale file
- No actual string extraction from code yet — just the loading infrastructure

### New dependency

- `sys-locale = "0.3"` (for future system locale detection)

---

## 8. File Change Summary

| File | Action |
|------|--------|
| `Cargo.toml` | Add `image`, `chrono`, `sys-locale` |
| `src/lib.rs` | Add modules: `capture`, `field`, `locale`, `glyphs`, `update`, `spawning` |
| `src/game.rs` | Slim down — extract update logic to `update.rs`, keep orchestration |
| `src/update.rs` | **New** — extracted from `game.rs` |
| `src/spawning.rs` | **New** — extracted from `objects.rs` |
| `src/objects.rs` | Slim down — keep types/structs, move spawn functions out |
| `src/field.rs` | **New** — distortion field types + stub |
| `src/locale.rs` | **New** — i18n skeleton |
| `src/glyphs.rs` | **New** — extracted from `hud.rs` |
| `src/capture.rs` | **New** — screenshot + video capture |
| `src/parameters.rs` | Add `ssaa_factor`, `DownsampleFilter` to config; merge exposure UI |
| `src/pause_menu.rs` | Add SSAA cycle, Record Scenario toggle; exposure slider merge |
| `src/recording.rs` | Add `GameStateSnapshot`, `ObjectSnapshot` |
| `src/rendering/mod.rs` | Restructure around layer system; slim down |
| `src/rendering/pipeline.rs` | **New** — extracted pipeline creation |
| `src/rendering/textures.rs` | **New** — extracted texture management |
| `src/rendering/world.rs` | Update draw calls for layer-based rendering |
| `src/rendering/hud.rs` | Slim down — move glyphs to `glyphs.rs` |
| `src/shaders/sdf.wgsl` | Add additive blend variant documentation |
| `src/shaders/postprocess.wgsl` | Add SSAA downsample (box filter), accept `ssaa_factor` uniform |
| `locales/en.ron` | **New** — minimal English locale |

---

## 9. Parallelization Strategy

These workstreams are independent and can be executed in parallel:

| Stream | Files touched | Dependencies |
|--------|--------------|--------------|
| **A: Code restructure** | game.rs, objects.rs, rendering/mod.rs, hud.rs → new split files | None — pure extraction, no behavior change |
| **B: Layer renderer** | rendering/mod.rs, rendering/world.rs, sdf.wgsl | Depends on A (works on restructured files) |
| **C: SSAA + downsample** | rendering/mod.rs, postprocess.wgsl, parameters.rs, pause_menu.rs | Depends on B (layer system) |
| **D: Capture tooling** | capture.rs, game.rs, recording.rs, Cargo.toml | Independent (new module) |
| **E: Visual fixes** | parameters.rs, rendering/world.rs, postprocess.wgsl | Depends on B (layer system context) |
| **F: Field groundwork** | field.rs, game.rs (add Vec<FieldSource>) | Independent (new module + one struct field) |
| **G: i18n groundwork** | glyphs.rs, locale.rs, hud.rs, locales/en.ron, Cargo.toml | Depends on A (glyph extraction is part of hud.rs split) |

**Execution order:**
1. **A + D + F** in parallel (code restructure, capture, fields)
2. **B + G** in parallel (layer renderer, i18n groundwork — after A)
3. **C + E** in parallel (SSAA, visual fixes — after B)

---

## 10. Non-Goals (this push)

- Gameplay features (teleport+explosion, continuous fire, gamepad pause nav) — deferred, parallel workers if trivial
- Full distortion field behavior — groundwork only
- Full i18n string extraction — groundwork only
- SMAA post-process AA — future phase
- Lanczos downsample filter — architecture prepared, implementation future
- GPU particle compute shaders — Phase 5
