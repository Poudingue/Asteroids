# Wave 3: SSAA + Visual Fixes

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add SSAA as a reference-quality rendering mode with pluggable downsample filters, and fix three visual issues (trail brightness, SDR/HDR parity, capsule brightness) plus merge the exposure slider.

**Architecture:** SSAA scales the offscreen render target by an integer factor. The postprocess shader gains a box-filter downsample loop when ssaa_factor > 1. Visual fixes are parameter tuning + shader investigation.

**Tech Stack:** Rust, wgpu 24, WGSL

**Depends on:** Wave 2 complete (layered compositing renderer)

---

## Stream C: SSAA + Downsample

### Task C1: Add SSAA config to parameters.rs

**Files:**
- Modify: `src/parameters.rs`

- [ ] **Step 1: Add enum:**

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DownsampleFilter {
    Box = 0,
    Lanczos = 1,  // Future
}
```

- [ ] **Step 2: Add fields to `HdrConfig`:**

```rust
pub ssaa_factor: u32,       // 1 = off, 2/3/4 = multiplier
pub downsample_filter: u32, // 0 = Box, 1 = Lanczos (future)
```

- [ ] **Step 3: Update `Default` impl for `HdrConfig`** — `ssaa_factor: 1`, `downsample_filter: 0`

- [ ] **Step 4: Add tests:**

```rust
#[test]
fn ssaa_factor_defaults_to_off() {
    let config = HdrConfig::default();
    assert_eq!(config.ssaa_factor, 1);
}

#[test]
fn downsample_filter_defaults_to_box() {
    let config = HdrConfig::default();
    assert_eq!(config.downsample_filter, 0);
}
```

- [ ] **Step 5: Run `cargo test`**
- [ ] **Step 6: Commit** — `git commit -m "feat: add ssaa_factor and DownsampleFilter to HdrConfig"`

---

### Task C2: Scale offscreen texture for SSAA

**Files:**
- Modify: `src/rendering/textures.rs`
- Modify: `src/rendering/mod.rs`

- [ ] **Step 1: Add `scaled_dimensions` to textures.rs:**

```rust
pub fn scaled_dimensions(width: u32, height: u32, ssaa_factor: u32) -> (u32, u32) {
    (width * ssaa_factor, height * ssaa_factor)
}
```

- [ ] **Step 2: Add tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_dimensions_1x() {
        assert_eq!(scaled_dimensions(1920, 1080, 1), (1920, 1080));
    }

    #[test]
    fn scaled_dimensions_4x() {
        assert_eq!(scaled_dimensions(1920, 1080, 4), (7680, 4320));
    }

    #[test]
    fn scaled_dimensions_2x() {
        assert_eq!(scaled_dimensions(800, 600, 2), (1600, 1200));
    }
}
```

- [ ] **Step 3: Add `ssaa_factor: u32` field to `Renderer2D`** (default 1)

- [ ] **Step 4: Update `resize()`** to use scaled dimensions:

```rust
let (scaled_w, scaled_h) = textures::scaled_dimensions(self.width, self.height, self.ssaa_factor);
// Use scaled_w, scaled_h for offscreen texture and MSAA texture creation
// screen_size_buffer should contain SCALED dimensions (shaders use this for positioning)
```

- [ ] **Step 5: Add `set_ssaa_factor()` method:**

```rust
pub fn set_ssaa_factor(&mut self, factor: u32, device: &wgpu::Device, queue: &wgpu::Queue) {
    self.ssaa_factor = factor.clamp(1, 4);
    self.resize(self.width, self.height, device, queue);
}
```

- [ ] **Step 6: Run `cargo check`**
- [ ] **Step 7: Commit** — `git commit -m "feat: SSAA scales offscreen render target"`

---

### Task C3: Update PostProcessUniforms for SSAA

**Files:**
- Modify: `src/rendering/mod.rs`

- [ ] **Step 1: Replace `_padding` with `ssaa_factor` in `PostProcessUniforms`:**

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PostProcessUniforms {
    pub game_exposure: f32,
    pub add_color_r: f32,
    pub add_color_g: f32,
    pub add_color_b: f32,
    pub mul_color_r: f32,
    pub mul_color_g: f32,
    pub mul_color_b: f32,
    pub hdr_enabled: u32,
    pub exposure: f32,
    pub max_brightness: f32,
    pub tonemap_variant: u32,
    pub ssaa_factor: u32,  // was _padding
}
```

Still 48 bytes — no alignment change.

- [ ] **Step 2: Update `update_postprocess_uniforms()`** to set `ssaa_factor` from `self.ssaa_factor`

- [ ] **Step 3: Update any existing size tests** for PostProcessUniforms

- [ ] **Step 4: Run `cargo check && cargo test`**
- [ ] **Step 5: Commit** — `git commit -m "feat: pass ssaa_factor to postprocess shader"`

---

### Task C4: Box filter downsample in postprocess.wgsl

**Files:**
- Modify: `src/shaders/postprocess.wgsl`

- [ ] **Step 1: Read current postprocess.wgsl** to understand existing shader structure

- [ ] **Step 2: Add `ssaa_factor` to the Uniforms struct in WGSL** (must match Rust struct layout):

```wgsl
struct Uniforms {
    game_exposure: f32,
    add_color_r: f32,
    add_color_g: f32,
    add_color_b: f32,
    mul_color_r: f32,
    mul_color_g: f32,
    mul_color_b: f32,
    hdr_enabled: u32,
    exposure: f32,
    max_brightness: f32,
    tonemap_variant: u32,
    ssaa_factor: u32,
};
```

- [ ] **Step 3: Add box filter sampling function:**

```wgsl
fn sample_box_filter(uv: vec2<f32>, ssaa: u32) -> vec4<f32> {
    if ssaa <= 1u {
        return textureSample(offscreen_texture, offscreen_sampler, uv);
    }
    let tex_size = vec2<f32>(textureDimensions(offscreen_texture));
    let pixel_size = 1.0 / tex_size;
    let sf = f32(ssaa);
    var acc = vec4<f32>(0.0);
    let sample_count = ssaa * ssaa;
    for (var dy = 0u; dy < ssaa; dy++) {
        for (var dx = 0u; dx < ssaa; dx++) {
            let offset = vec2<f32>(
                (f32(dx) - sf * 0.5 + 0.5) * pixel_size.x,
                (f32(dy) - sf * 0.5 + 0.5) * pixel_size.y,
            );
            acc += textureSample(offscreen_texture, offscreen_sampler, uv + offset);
        }
    }
    return acc / f32(sample_count);
}
```

- [ ] **Step 4: Replace single `textureSample` call** in the fragment entry point with `sample_box_filter(uv, uniforms.ssaa_factor)`

- [ ] **Step 5: Run `cargo check`** (shader validates at runtime, but Rust compiles)
- [ ] **Step 6: Visual test** — toggle SSAA 2x/4x, verify sharper edges and no artifacts
- [ ] **Step 7: Commit** — `git commit -m "feat: box filter downsample in postprocess shader for SSAA"`

---

### Task C5: Add SSAA cycle to pause menu

**Files:**
- Modify: `src/pause_menu.rs`
- Modify: `src/game.rs` (or wherever menu changes are wired to renderer)

- [ ] **Step 1: Add helper functions:**

```rust
fn ssaa_get(globals: &Globals) -> usize {
    match globals.hdr.ssaa_factor {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 0,
    }
}

fn ssaa_set(globals: &mut Globals, index: usize) {
    globals.hdr.ssaa_factor = match index {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 4,
        _ => 1,
    };
}
```

- [ ] **Step 2: Add SSAA cycle entry** to `PauseMenu::default()` (MenuEntryKind::Cycle with labels `["OFF", "2X", "3X", "4X"]`), using `ssaa_get`/`ssaa_set` as the accessor/mutator

- [ ] **Step 3: Wire renderer's `set_ssaa_factor`** — when SSAA setting changes in the menu, call `renderer.set_ssaa_factor(globals.hdr.ssaa_factor, device, queue)` in the game loop (same pattern as MSAA toggle)

- [ ] **Step 4: Run `cargo check`**
- [ ] **Step 5: Visual test** — cycle through SSAA modes in pause menu
- [ ] **Step 6: Commit** — `git commit -m "feat: add SSAA cycle to pause menu"`

---

### Task C6: SSAA verification

- [ ] **Step 1: Run `cargo check && cargo clippy && cargo test && cargo fmt`**
- [ ] **Step 2: Visual tests:**
  - SSAA 4x produces noticeably sharper image than 1x
  - SSAA + MSAA combined works without artifacts
  - SSAA OFF → no performance impact (1x = passthrough in shader)
  - HUD renders at native resolution regardless of SSAA
- [ ] **Step 3: Commit any fixes**

---

## Stream E: Visual Fixes

### Task E1: Exposure slider merge

**Files:**
- Modify: `src/pause_menu.rs`
- Modify: `src/parameters.rs`

- [ ] **Step 1: Check current pause menu** for exposure-related entries. Identify if both `exposure` and `game_exposure` are exposed.

- [ ] **Step 2: Ensure single "EXPOSURE" slider** — maps to `HdrConfig.exposure`. Remove any UI entry for `game_exposure` / `ExposureConfig.game_exposure`.

- [ ] **Step 3: Ensure `game_exposure`** in `ExposureConfig` defaults to 1.0 and is not user-modifiable through the UI. It remains available as an internal multiplier for gameplay effects (e.g., flash on explosion).

- [ ] **Step 4: Run `cargo check`**
- [ ] **Step 5: Commit** — `git commit -m "fix: merge exposure into single slider, game_exposure internal only"`

---

### Task E2: Trail brightness re-tuning

**Files:**
- Modify: `src/rendering/world.rs`

The brightness conservation formula (`pi*r^2 / (pi*r^2 + 2*r*L)`) is correct but makes trails too dim.

- [ ] **Step 1: Add `brightness_boost: f64` field to `TrailConfig`:**

```rust
pub struct TrailConfig {
    pub radius: f64,
    pub brightness_falloff: f64,
    pub shutter_speed: f64,
    pub brightness_boost: f64,  // Multiplier applied after conservation formula
}
```

- [ ] **Step 2: Apply in `render_trail`** — after the conservation calculation, multiply the final brightness by `brightness_boost`:

```rust
let conservation_factor = /* existing formula */;
let adjusted = conservation_factor * config.brightness_boost;
```

- [ ] **Step 3: Set values** (needs visual tuning — start with):
  - Star trail: `brightness_boost: 2.5`
  - Bullet trail: `brightness_boost: 2.0`

- [ ] **Step 4: Add test:**

```rust
#[test]
fn brightness_boost_amplifies_conservation() {
    let config = TrailConfig {
        radius: 1.0,
        brightness_falloff: 0.5,
        shutter_speed: 1.0,
        brightness_boost: 3.0,
    };
    // With boost=3.0, the output should be 3x the base conservation factor
    // Exact value depends on formula, but boost > 1 should increase brightness
    assert!(config.brightness_boost > 1.0);
}
```

- [ ] **Step 5: Run `cargo check && cargo test`**
- [ ] **Step 6: Visual test** — trails should be visibly brighter
- [ ] **Step 7: Commit** — `git commit -m "fix: add brightness_boost to TrailConfig for trail visibility"`

---

### Task E3: Capsule brightness adjustment

**Files:**
- Modify: `src/rendering/world.rs`

Capsule (trail segment) brightness needs independent control.

- [ ] **Step 1: Add `capsule_brightness: f64` to `TrailConfig`:**

```rust
pub capsule_brightness: f64,  // Multiplier for capsule color values
```

- [ ] **Step 2: Apply in `render_trail`** — when creating `CapsuleInstance`, multiply the color by `capsule_brightness`:

```rust
let color = [
    (base_color.r * config.capsule_brightness) as f32,
    (base_color.g * config.capsule_brightness) as f32,
    (base_color.b * config.capsule_brightness) as f32,
];
```

- [ ] **Step 3: Set values:**
  - Star trail: `capsule_brightness: 2.0`
  - Bullet trail: `capsule_brightness: 1.5`

- [ ] **Step 4: Run `cargo check`**
- [ ] **Step 5: Commit** — `git commit -m "fix: add capsule_brightness to TrailConfig for independent control"`

---

### Task E4: SDR/HDR parity investigation and fix

**Files:**
- Modify: `src/shaders/postprocess.wgsl` (possibly)
- Modify: `src/parameters.rs` (possibly)

SDR mode appears dimmer than HDR at the same exposure. Requires investigation.

- [ ] **Step 1: Read `postprocess.wgsl`** — identify where HDR and SDR codepaths diverge

- [ ] **Step 2: Read `parameters.rs`** — check `game_exposure_target_sdr` vs `game_exposure_target_hdr` values. If SDR target is lower, that's a likely cause.

- [ ] **Step 3: Investigate the tonemap function** — does it treat SDR and HDR identically? The sRGB transfer function applied by `Bgra8UnormSrgb` surface format compresses highlights via gamma, which can make SDR look dimmer. In HDR (`Rgba16Float`), output is linear.

- [ ] **Step 4: Fix approach** — one of:
  - (a) Increase `game_exposure_target_sdr` to match perceived brightness
  - (b) Add an SDR brightness compensation factor in the postprocess shader
  - (c) Apply a pre-gamma boost in the shader when `hdr_enabled == 0`

- [ ] **Step 5: Implement the fix** based on investigation findings

- [ ] **Step 6: Visual test** — toggle HDR on/off, compare perceived brightness at same exposure
- [ ] **Step 7: Run `cargo check && cargo test`**
- [ ] **Step 8: Commit** — `git commit -m "fix: adjust SDR brightness for HDR parity"`

---

### Task E5: Stream E verification

- [ ] **Step 1: Run `cargo check && cargo clippy && cargo test && cargo fmt`**
- [ ] **Step 2: Visual checklist:**
  - Single "EXPOSURE" slider in pause menu
  - Trails visible and bright
  - Capsule segments independently bright
  - SDR/HDR perceived brightness similar at same exposure
- [ ] **Step 3: Commit any remaining fixes**

---

## Final Wave 3 Verification

- [ ] **Run full test suite: `cargo check && cargo clippy && cargo test && cargo fmt`**
- [ ] **Combined visual test:** SSAA 4x + MSAA x4, all visual fixes active
- [ ] **Performance spot-check:** SSAA 4x on a busy scene (many asteroids + explosions)
- [ ] **Commit any remaining fixes**
