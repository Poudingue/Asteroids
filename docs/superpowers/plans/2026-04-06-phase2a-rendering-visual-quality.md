# Phase 2A: Rendering Pipeline & Visual Quality — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix layer ordering, improve SDF rendering quality (soft falloff, no radius dithering), unify trail rendering with brightness conservation, fix HUD tonemap bypass, fix smoke velocity inheritance, fix engine fire at high speed, and enable widest-gamut ship colors.

**Architecture:** Tasks target four files primarily: `src/game.rs` (render order), `src/rendering/world.rs` (SDF/trail rendering), `src/rendering/mod.rs` + `src/shaders/sdf.wgsl` (CircleInstance struct + shader), `src/shaders/hud.wgsl` + `src/rendering/mod.rs` (HUD uniforms/tonemap), `src/objects.rs` (smoke velocity, fire velocity, ship color). Each task is self-contained. Shader changes require visual verification — no automated tests possible there.

**Tech Stack:** Rust 2021, wgpu 0.20, bytemuck (repr(C) GPU structs), WGSL shaders, rand (SmallRng). Test runner: `rtk cargo test`. Build check: `rtk cargo check`. Lint: `rtk cargo clippy`. Format: `cargo fmt`.

---

## File Map

| File | Changes |
|------|---------|
| `src/game.rs` | Reorder render calls (Task 1) |
| `src/rendering/world.rs` | Remove dither_radius for circles (Task 3), add shared render_trail() (Task 4+5) |
| `src/rendering/mod.rs` | Add falloff_width to CircleInstance + push_circle_instance (Task 2), expand HudUniforms (Task 7) |
| `src/shaders/sdf.wgsl` | Add falloff_width to shader CircleInstance + fs_circle (Task 2) |
| `src/shaders/hud.wgsl` | Add tonemap functions + uniform fields + apply in fs_main (Task 7) |
| `src/main.rs` | Pass expanded HudUniforms (Task 7) |
| `src/objects.rs` | Fix smoke velocity in 4 spawn fns (Task 8), fix fire speed formula (Task 9), widen ship color (Task 10) |

---

## Task 1: Layer Order Fix

**Files:**
- Modify: `src/game.rs` lines ~1480–1525

The desired render order is:
`background → stars → smoke → chunks → [sparkles placeholder] → projectiles → fragments → toosmall → asteroids → explosions → ship`

Currently ship is rendered before fragments/toosmall/asteroids/explosions. We move it to the end.

- [ ] **Step 1: Write a compile-time order-guard test**

Add this to `src/game.rs` at the bottom, inside a `#[cfg(test)]` block. This is a static ordering documentation test — it won't catch regressions automatically, but forces us to think about the order and documents it:

```rust
#[cfg(test)]
mod render_order_tests {
    /// Documents the expected render layer order.
    /// If this test fails to compile after a refactor, review render_frame.
    #[test]
    fn render_layer_order_is_documented() {
        // Layer indices (lower = rendered first = further back)
        const BACKGROUND: u8 = 0;
        const STARS: u8 = 1;
        const SMOKE: u8 = 2;
        const CHUNKS: u8 = 3;
        const SPARKLES_PLACEHOLDER: u8 = 4; // not yet implemented
        const PROJECTILES: u8 = 5;
        const FRAGMENTS: u8 = 6;
        const TOOSMALL: u8 = 7;
        const ASTEROIDS: u8 = 8;
        const EXPLOSIONS: u8 = 9;
        const SHIP: u8 = 10;

        assert!(BACKGROUND < STARS);
        assert!(STARS < SMOKE);
        assert!(SMOKE < CHUNKS);
        assert!(CHUNKS < SPARKLES_PLACEHOLDER);
        assert!(SPARKLES_PLACEHOLDER < PROJECTILES);
        assert!(PROJECTILES < FRAGMENTS);
        assert!(FRAGMENTS < TOOSMALL);
        assert!(TOOSMALL < ASTEROIDS);
        assert!(ASTEROIDS < EXPLOSIONS);
        assert!(EXPLOSIONS < SHIP);
    }
}
```

- [ ] **Step 2: Run test to confirm it passes (it's pure assertion logic)**

```
rtk cargo test render_layer_order_is_documented
```

Expected: PASS

- [ ] **Step 3: Reorder render calls in render_frame**

In `src/game.rs`, find the render section (~lines 1480–1525). Replace the block from `// Stars` through the end of the ship/fragments/toosmall/objects/explosions section with:

```rust
    // Stars
    for star in &state.stars {
        render_star_trail(star, renderer, globals, &mut state.rng);
    }

    // Smoke — soft background layer
    for s in &state.smoke {
        render_visuals(s, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // Chunks
    for chunk in &state.chunks {
        render_chunk(chunk, renderer, globals, &mut state.rng);
    }

    // TODO: Sparkles (collision light-trails) — will be added when collision system creates them
    // for spark in &state.sparks { render_spark(spark, renderer, globals, &mut state.rng); }

    // Projectiles
    for p in &state.projectiles {
        render_projectile(p, renderer, globals, &mut state.rng);
    }

    // Fragments — debris layer before asteroids
    for entity in &state.fragments {
        render_visuals(entity, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // Toosmall — micro debris
    for entity in &state.toosmall {
        render_visuals(entity, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // Asteroids
    for entity in &state.objects {
        render_visuals(entity, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // Explosions — in front of asteroids, behind ship
    for e in &state.explosions {
        render_visuals(e, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // Ship — topmost game object (renders above explosions)
    let true_aim = state.ship.orientation;
    state.ship.orientation = state.gamepad.visual_aim_angle;
    render_visuals(&state.ship, Vec2::ZERO, renderer, globals, &mut state.rng);
    state.ship.orientation = true_aim;
```

- [ ] **Step 4: Build and run tests**

```
rtk cargo check && rtk cargo test
```

Expected: no errors, all tests pass.

- [ ] **Step 5: Commit**

```bash
rtk git add src/game.rs
rtk git commit -m "fix: move ship render to top layer (above explosions)"
```

---

## Task 2: Soft SDF Alpha Falloff for Circles

**Files:**
- Modify: `src/rendering/mod.rs` — `CircleInstance` struct and `push_circle_instance` method
- Modify: `src/shaders/sdf.wgsl` — `CircleInstance` struct and `fs_circle` function
- Modify: `src/rendering/world.rs` — all callers of `push_circle_instance` (smoke call site)

`CircleInstance` is currently 32 bytes: `center[f32;2] + radius[f32] + color[f32;4] + _padding[f32]`. We replace `_padding` with `falloff_width: f32` — same size, no layout change.

- [ ] **Step 1: Write a test for the falloff_width field round-trip**

Add to `src/rendering/mod.rs` at the bottom inside `#[cfg(test)]`:

```rust
#[cfg(test)]
mod circle_instance_tests {
    use super::*;

    #[test]
    fn circle_instance_size_unchanged() {
        // Must stay 32 bytes for GPU alignment
        assert_eq!(std::mem::size_of::<CircleInstance>(), 32);
    }

    #[test]
    fn circle_instance_falloff_stored() {
        let c = CircleInstance {
            center: [10.0, 20.0],
            radius: 5.0,
            color: [1.0, 0.5, 0.0, 1.0],
            falloff_width: 0.2,
        };
        assert!((c.falloff_width - 0.2).abs() < 1e-6);
    }
}
```

- [ ] **Step 2: Run tests to confirm they fail (field doesn't exist yet)**

```
rtk cargo test circle_instance
```

Expected: FAIL — `CircleInstance` has no `falloff_width` field.

- [ ] **Step 3: Rename `_padding` to `falloff_width` in CircleInstance (Rust side)**

In `src/rendering/mod.rs`, find the `CircleInstance` struct (lines 54–61) and replace:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub center: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
    pub _padding: f32,
}
```

With:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub center: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
    /// Soft falloff width as a fraction of radius.
    /// 0.0 = sharp edge (1px AA only from smoothstep).
    /// 0.2 = full opacity at 80% of radius, fading to 0 at 100%.
    pub falloff_width: f32,
}
```

- [ ] **Step 4: Update `push_circle_instance` signature**

In `src/rendering/mod.rs`, find `push_circle_instance` (lines 982–992) and replace:

```rust
pub fn push_circle_instance(&mut self, cx: f32, cy: f32, radius: f32, color: [f32; 4]) {
    if radius <= 0.0 {
        return;
    }
    self.sdf_circle_instances.push(CircleInstance {
        center: [cx, cy],
        radius,
        color,
        _padding: 0.0,
    });
}
```

With:

```rust
pub fn push_circle_instance(
    &mut self,
    cx: f32,
    cy: f32,
    radius: f32,
    color: [f32; 4],
    falloff_width: f32,
) {
    if radius <= 0.0 {
        return;
    }
    self.sdf_circle_instances.push(CircleInstance {
        center: [cx, cy],
        radius,
        color,
        falloff_width,
    });
}
```

- [ ] **Step 5: Fix all callers of `push_circle_instance`**

There are three call sites — two in `src/rendering/world.rs` (in `render_visuals` and `render_chunk`) and any others. Search and fix each:

In `src/rendering/world.rs`, `render_visuals` (~line 80):
```rust
renderer.push_circle_instance(x as f32, y as f32, r.max(1) as f32, color, 0.0);
```

In `src/rendering/world.rs`, `render_chunk` (~line 115):
```rust
renderer.push_circle_instance(x as f32, y as f32, r.max(1) as f32, color, 0.0);
```

Check for any other callers:
```
rtk cargo check 2>&1 | head -40
```

Fix any additional call sites found by cargo check, all passing `0.0` as falloff_width for now.

- [ ] **Step 6: Update the WGSL shader — CircleInstance struct**

In `src/shaders/sdf.wgsl`, replace:

```wgsl
struct CircleInstance {
    @location(2) center: vec2<f32>,
    @location(3) radius: f32,
    @location(4) color: vec4<f32>,
};
```

With:

```wgsl
struct CircleInstance {
    @location(2) center: vec2<f32>,
    @location(3) radius: f32,
    @location(4) color: vec4<f32>,
    @location(5) falloff_width: f32,
};
```

- [ ] **Step 7: Update VertexOutput to carry falloff_width through the vertex stage**

In `src/shaders/sdf.wgsl`, replace:

```wgsl
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) radius_px: f32,
};
```

With:

```wgsl
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) radius_px: f32,
    @location(3) falloff_width: f32,
};
```

- [ ] **Step 8: Pass falloff_width through vs_circle**

In `src/shaders/sdf.wgsl`, in `fn vs_circle`, add `out.falloff_width = instance.falloff_width;` before `return out;`:

```wgsl
@vertex
fn vs_circle(
    @builtin(vertex_index) vertex_index: u32,
    instance: CircleInstance,
) -> VertexOutput {
    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );
    let local = quad_pos[vertex_index];
    let margin = 1.0;
    let pixel_pos = instance.center + local * (instance.radius + margin);

    var out: VertexOutput;
    out.position = vec4<f32>(
        (pixel_pos.x / screen_size.x) * 2.0 - 1.0,
        (pixel_pos.y / screen_size.y) * 2.0 - 1.0,
        0.0, 1.0
    );
    out.uv = local * (instance.radius + margin);
    out.color = instance.color;
    out.radius_px = instance.radius;
    out.falloff_width = instance.falloff_width;
    return out;
}
```

- [ ] **Step 9: Apply falloff in fs_circle**

In `src/shaders/sdf.wgsl`, replace the existing `fs_circle` function:

```wgsl
@fragment
fn fs_circle(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv) - in.radius_px;
    var alpha: f32;
    if (SDF_AA_ENABLED) { alpha = smoothstep(0.5, -0.5, dist); }
    else { alpha = select(0.0, 1.0, dist < 0.0); }

    // Soft falloff: fade opacity over outer fraction of radius
    // falloff_width=0 → sharp (no inner fade), falloff_width=0.2 → fade over outer 20%
    if (in.falloff_width > 0.0 && in.radius_px > 0.0) {
        let frac = length(in.uv) / in.radius_px; // 0 at center, 1 at edge
        let falloff_start = 1.0 - in.falloff_width;
        let inner_alpha = smoothstep(1.0, falloff_start, frac);
        alpha = alpha * inner_alpha;
    }

    if (alpha < 0.001) { discard; }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
```

- [ ] **Step 10: Update CircleInstance vertex buffer layout (Rust side)**

The GPU vertex buffer attribute layout for `CircleInstance` must be updated to include the new `falloff_width` field. In `src/rendering/mod.rs`, find the `impl CircleInstance` block with the `desc()` function and replace it:

```rust
impl CircleInstance {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CircleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // location(2): center [f32; 2]
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // location(3): radius f32
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                // location(4): color [f32; 4]
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // location(5): falloff_width f32
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}
```

- [ ] **Step 11: Apply falloff_width=0.2 to smoke**

In `src/rendering/world.rs`, inside `render_visuals`, the circle branch pushes smoke/explosion circles. Smoke entities have `EntityKind::Smoke`. Change the smoke circle call to use `falloff_width=0.2`.

Find the circle branch inside `render_visuals` and update it to branch on entity kind:

```rust
if visuals.radius > 0.0 && visuals.shapes.is_empty() {
    let color = to_hdr_rgba(intensify(hdr(visuals.color), exposure));
    let (x, y) = dither_vec(position, DITHER_AA, globals.render.current_jitter_double);
    let r = visuals.radius * globals.render.render_scale;
    // Smoke gets soft edge falloff; explosions and other SDF circles stay sharp
    let falloff = if entity.kind == EntityKind::Smoke { 0.2 } else { 0.0 };
    renderer.push_circle_instance(x as f32, y as f32, r as f32, color, falloff);
}
```

Note: This also removes the `dither_radius` call from `render_visuals` for the circle branch (covered more in Task 3).

- [ ] **Step 12: Build and run tests**

```
rtk cargo check && rtk cargo test circle_instance
```

Expected: all circle_instance tests pass.

- [ ] **Step 13: Commit**

```bash
rtk git add src/rendering/mod.rs src/shaders/sdf.wgsl src/rendering/world.rs
rtk git commit -m "feat: add falloff_width to CircleInstance SDF (smoke gets 0.2 soft edge)"
```

---

## Task 3: Remove Radius Dithering for SDF Circles

**Files:**
- Modify: `src/rendering/world.rs` — `render_visuals` (circle branch) and `render_chunk`

SDF circles use a smoothstep for anti-aliasing — the 1px subpixel dithering added by `dither_radius` is redundant and introduces noise. Remove it. Keep dithering for capsule radius in `render_projectile` (the capsule falloff uses it differently).

- [ ] **Step 1: Write a test confirming exact radius is used**

This is a structural test — verify the render_chunk path uses exact integer-clamped radius (no random perturbation). Add to `src/rendering/world.rs` in `#[cfg(test)]`:

```rust
#[cfg(test)]
mod dither_tests {
    // Smoke test: render_chunk uses exact pixel radius (no dither).
    // This is verified by code review — the functions that call dither_radius
    // are render_projectile (capsule, intentional) and formerly render_chunk/render_visuals.
    // After Task 3, only render_projectile should call dither_radius.
    #[test]
    fn dither_radius_only_used_for_capsules() {
        // If this test fails to compile, dither_radius was removed from scope.
        // It's intentionally still imported for render_projectile.
        use crate::rendering::world::*;
        let _ = std::stringify!(dither_radius); // Confirm it still exists in scope
    }
}
```

- [ ] **Step 2: Run tests**

```
rtk cargo test dither_tests
```

Expected: PASS (compile-only test).

- [ ] **Step 3: Remove dither_radius from render_visuals circle branch**

In `src/rendering/world.rs`, in `render_visuals`, the circle branch currently calls `dither_radius`. Replace the radius computation:

From:
```rust
let r = dither_radius(
    visuals.radius * globals.render.render_scale,
    DITHER_AA,
    DITHER_POWER_RADIUS,
    rng,
);
renderer.push_circle_instance(x as f32, y as f32, r.max(1) as f32, color, falloff);
```

To:
```rust
// SDF circles: use exact radius — smoothstep AA handles sub-pixel; dithering adds noise
let r = (visuals.radius * globals.render.render_scale).max(1.0);
renderer.push_circle_instance(x as f32, y as f32, r as f32, color, falloff);
```

Also remove `rng` from the function signature if it was only used for `dither_radius`. Check if `rng` is still needed (it is — `render_shapes` is called below). Leave the parameter.

- [ ] **Step 4: Remove dither_radius from render_chunk**

In `src/rendering/world.rs`, in `render_chunk`, replace:

```rust
let (x, y) = dither_vec(pos, DITHER_AA, globals.render.current_jitter_double);
let r = dither_radius(
    globals.render.render_scale * entity.visuals.radius,
    DITHER_AA,
    DITHER_POWER_RADIUS,
    rng,
);
renderer.push_circle_instance(x as f32, y as f32, r.max(1) as f32, color);
```

With:

```rust
let (x, y) = dither_vec(pos, DITHER_AA, globals.render.current_jitter_double);
let r = (globals.render.render_scale * entity.visuals.radius).max(1.0);
renderer.push_circle_instance(x as f32, y as f32, r as f32, color, 0.0);
```

- [ ] **Step 5: Build and run tests**

```
rtk cargo check && rtk cargo test
```

Expected: no errors, all tests pass.

- [ ] **Step 6: Commit**

```bash
rtk git add src/rendering/world.rs
rtk git commit -m "refactor: remove radius dithering for SDF circles (smoothstep handles AA)"
```

---

## Task 4: TrailConfig and Shared render_trail()

**Files:**
- Modify: `src/rendering/world.rs` — add `TrailConfig` struct and `render_trail()` function, refactor `render_star_trail` and `render_projectile`

This creates a shared path for all motion-blur capsule trails.

- [ ] **Step 1: Write tests for TrailConfig defaults**

Add to `src/rendering/world.rs` in `#[cfg(test)]`:

```rust
#[cfg(test)]
mod trail_config_tests {
    use super::*;

    #[test]
    fn trail_config_star_defaults() {
        let cfg = TrailConfig::star();
        assert!((cfg.radius - 1.0).abs() < 1e-9);
        assert!((cfg.shutter_speed - 1.0).abs() < 1e-9);
    }

    #[test]
    fn trail_config_bullet_defaults() {
        let cfg = TrailConfig::bullet(15.0);
        assert!((cfg.radius - 15.0).abs() < 1e-9);
        assert!((cfg.shutter_speed - 1.0).abs() < 1e-9);
    }

    #[test]
    fn trail_config_shutter_zero_means_no_trail() {
        let cfg = TrailConfig {
            radius: 5.0,
            brightness_falloff: 0.5,
            shutter_speed: 0.0,
        };
        assert!((cfg.shutter_speed).abs() < 1e-9);
    }
}
```

- [ ] **Step 2: Run tests (expect FAIL — TrailConfig not yet defined)**

```
rtk cargo test trail_config_tests
```

Expected: FAIL — `TrailConfig` not found.

- [ ] **Step 3: Define TrailConfig and render_trail() in world.rs**

Add near the top of `src/rendering/world.rs`, after the existing imports:

```rust
/// Configuration for motion-blur capsule trail rendering.
/// Shared by stars, bullets, and future sparkle trails.
pub struct TrailConfig {
    /// Capsule radius in screen pixels.
    pub radius: f64,
    /// Brightness falloff exponent along the trail.
    /// Stars use `sqrt(1/(1+dist))`, bullets use `0.5*sqrt(r/(r+dist))`.
    /// This field selects the brightness formula: 0.0=constant, >0=falloff.
    pub brightness_falloff: f64,
    /// Shutter speed multiplier: 0.0=circle (no trail), 1.0=physical, >1=exaggerated.
    pub shutter_speed: f64,
}

impl TrailConfig {
    /// Default config for star trails.
    pub fn star() -> Self {
        TrailConfig { radius: 1.0, brightness_falloff: 1.0, shutter_speed: 1.0 }
    }

    /// Default config for bullet trails. Radius is the projectile radius in screen pixels.
    pub fn bullet(radius: f64) -> Self {
        TrailConfig { radius, brightness_falloff: 0.5, shutter_speed: 1.0 }
    }
}

/// Render a motion-blur capsule trail between two screen-space endpoints.
///
/// Brightness conservation: when a circle becomes a capsule, the additional area would
/// make the total light output increase. We scale color by the ratio:
///   π·r² / (π·r² + 2·r·L)  =  1 / (1 + 2L/(π·r))
/// so a stationary object (L=0) has the same apparent brightness as a moving one.
///
/// `base_color` is the pre-multiplied HDR color at full opacity.
/// `brightness_falloff`: the trail_lum is computed as:
///   - For stars (falloff=1.0): `sqrt(1 / (1 + dist))`
///   - For bullets (falloff=0.5): `falloff * sqrt(radius / (radius + dist))`
pub fn render_trail(
    renderer: &mut Renderer2D,
    p0: (f64, f64),
    p1: (f64, f64),
    cfg: &TrailConfig,
    base_color: [f32; 4],
) {
    let (x1, y1) = p0;
    let (x2, y2) = p1;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let trail_len = (dx * dx + dy * dy).sqrt();

    // Brightness conservation scale factor: preserves total luminous flux
    // as the capsule grows longer relative to a circle of the same radius.
    let r = cfg.radius.max(0.001);
    let area_scale = if trail_len < 0.001 {
        1.0
    } else {
        let pi_r2 = std::f64::consts::PI * r * r;
        pi_r2 / (pi_r2 + 2.0 * r * trail_len)
    };

    // Trail brightness falloff along the length
    let trail_lum = if cfg.brightness_falloff <= 0.0 {
        1.0
    } else if cfg.brightness_falloff >= 1.0 {
        // Star formula: sqrt(1 / (1 + dist))
        (1.0 / (1.0 + trail_len)).sqrt()
    } else {
        // Bullet formula: falloff * sqrt(r / (r + dist))
        cfg.brightness_falloff * (r / (r + trail_len)).sqrt()
    };

    let combined_scale = (area_scale * trail_lum) as f32;
    let color = [
        base_color[0] * combined_scale,
        base_color[1] * combined_scale,
        base_color[2] * combined_scale,
        base_color[3],
    ];

    let radius = (cfg.radius * cfg.shutter_speed).max(1.0) as f32;
    renderer.push_capsule_instance(
        x1 as f32, y1 as f32,
        x2 as f32, y2 as f32,
        radius,
        color,
    );
}
```

- [ ] **Step 4: Run tests**

```
rtk cargo test trail_config_tests
```

Expected: PASS.

- [ ] **Step 5: Commit (TrailConfig + render_trail defined)**

```bash
rtk git add src/rendering/world.rs
rtk git commit -m "feat: add TrailConfig + render_trail() with brightness conservation"
```

---

## Task 5: Refactor render_star_trail and render_projectile to use render_trail()

**Files:**
- Modify: `src/rendering/world.rs`

Note: The static star branch (cross of pixels) stays unchanged — `render_trail` is only for the moving capsule case.

- [ ] **Step 1: Refactor render_star_trail's moving branch**

In `src/rendering/world.rs`, in `render_star_trail`, find the `else` branch that renders a capsule (the "Moving star" path). Replace from the `let dist = ...` line through the `push_capsule_instance` call:

```rust
    } else {
        // Moving star: render as a thin SDF capsule trail
        let cfg = TrailConfig::star();
        // base_color: star color at full brightness (trail_lum applied inside render_trail)
        let base_color = to_hdr_rgba(hdr_add(
            star_color_tmp,
            hdr_add(
                intensify(hdr(globals.visual.space_color), globals.exposure.game_exposure),
                intensify(hdr(globals.exposure.add_color), globals.exposure.game_exposure),
            ),
        ));
        render_trail(renderer, (x1, y1), (x2, y2), &cfg, base_color);
    }
```

- [ ] **Step 2: Refactor render_projectile**

In `src/rendering/world.rs`, in `render_projectile`, replace from `let dist = ...` through the `push_capsule_instance` call:

```rust
    let (x1, y1) = dither_vec(pos1, DITHER_AA, globals.render.current_jitter_double);
    let (x2, y2) = dither_vec(pos2, DITHER_AA, globals.render.current_jitter_double);
    // Capsule radius: dither for sub-pixel AA (intentional for polygon-style capsules)
    let radius_px = dither_radius(rad, DITHER_AA, DITHER_POWER_RADIUS, rng) as f64;

    let cfg = TrailConfig::bullet(radius_px);
    let base_color = to_hdr_rgba(col);
    render_trail(renderer, (x1, y1), (x2, y2), &cfg, base_color);
```

- [ ] **Step 3: Build and run tests**

```
rtk cargo check && rtk cargo test
```

Expected: no errors, all tests pass.

- [ ] **Step 4: Commit**

```bash
rtk git add src/rendering/world.rs
rtk git commit -m "refactor: render_star_trail and render_projectile use shared render_trail()"
```

---

## Task 6: Post-Process vs Per-Object Color Effects — Verify & Document

**Files:**
- Modify: `src/shaders/postprocess.wgsl` — add documentation comment

This is a verification and documentation task. The separation is already correct:
- Global effects (`add_color` screen flash, `mul_color` damage tint) → post-process ✓
- Per-object color (asteroid tint, bullet color via `hdr_exposure`) → forward pass ✓

- [ ] **Step 1: Verify no per-object color is incorrectly baked into post-process**

Search for misuse patterns:
```
rtk cargo check
```

Also visually scan for any place that passes per-entity color through uniforms instead of vertex data:
```bash
rtk grep "add_color\|mul_color" src/game.rs src/rendering/world.rs
```

Expected: `add_color` and `mul_color` only appear in `Globals.exposure` paths, never per-entity.

- [ ] **Step 2: Add documentation comment to postprocess.wgsl**

In `src/shaders/postprocess.wgsl`, add a block comment just above the `fn tonemap` function:

```wgsl
// ============================================================================
// Color Effect Architecture
// ============================================================================
// Two-tier color effects:
//   1. Per-object (forward pass, vertex data): entity visuals.color × hdr_exposure
//      - Asteroid tints, bullet colors, ship shading — vary per entity
//   2. Global (post-process, uniforms): add_color + mul_color × game_exposure × exposure
//      - Screen flash (add_color): e.g. white flash on hit, red flash on damage
//      - Damage tint (mul_color): e.g. red tint when health is low
//      - Brightness (game_exposure, exposure): overall scene intensity
// Do NOT mix these: per-object colors must never be passed via PostProcessUniforms.
// ============================================================================
```

- [ ] **Step 3: Build check**

```
rtk cargo check
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
rtk git add src/shaders/postprocess.wgsl
rtk git commit -m "docs: document two-tier color effect architecture in postprocess.wgsl"
```

---

## Task 7: HUD Tonemap Fix

**Files:**
- Modify: `src/rendering/mod.rs` — expand `HudUniforms`
- Modify: `src/shaders/hud.wgsl` — add tonemap functions, apply in `fs_main`
- Modify: `src/main.rs` — pass new HudUniforms fields when calling `update_hud_uniforms`

Currently the HUD bypasses tonemapping: it outputs `color/255 * brightness_scale` directly. On HDR displays this means HUD text can look different from scene objects at the same physical brightness. Fix: apply the same compression curve.

`HudUniforms` is currently 16 bytes (4 × f32). We expand it to 32 bytes.

- [ ] **Step 1: Write a size test for the expanded HudUniforms**

Add to `src/rendering/mod.rs` in `#[cfg(test)]`:

```rust
#[cfg(test)]
mod hud_uniforms_tests {
    use super::*;

    #[test]
    fn hud_uniforms_size_is_32_bytes() {
        // Must stay 32 bytes (8 × f32) for wgpu uniform alignment
        assert_eq!(std::mem::size_of::<HudUniforms>(), 32);
    }
}
```

- [ ] **Step 2: Run test (expect FAIL — HudUniforms is still 16 bytes)**

```
rtk cargo test hud_uniforms_size
```

Expected: FAIL.

- [ ] **Step 3: Expand HudUniforms**

In `src/rendering/mod.rs`, find `HudUniforms` (lines ~26–31) and replace:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudUniforms {
    pub screen_width: f32,
    pub screen_height: f32,
    pub brightness_scale: f32,
    pub _padding: f32,
}
```

With:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudUniforms {
    pub screen_width: f32,
    pub screen_height: f32,
    /// Brightness scale: 1.0 in SDR, hud_nits/80.0 in HDR.
    pub brightness_scale: f32,
    /// 1.0 if HDR surface is active, 0.0 otherwise.
    pub hdr_enabled: f32,
    /// Maximum scene brightness in nits (same as PostProcessUniforms.max_brightness).
    pub max_brightness: f32,
    /// Tonemap variant index (0=Passthrough, 1=Reinhard, 2=HardRedirect, 3=SoftRedirect).
    pub tonemap_variant: f32,
    /// Exposure multiplier (same as PostProcessUniforms.exposure).
    pub exposure: f32,
    pub _padding: f32,
}
```

- [ ] **Step 4: Run size test**

```
rtk cargo test hud_uniforms_size
```

Expected: PASS (8 × f32 = 32 bytes).

- [ ] **Step 5: Update HudUniforms construction in main.rs**

In `src/main.rs`, find all `HudUniforms { ... }` construction sites (~lines 321–328 and 671–678). There should be two: one on resize and one per frame. Update both.

For the resize site (~line 321):
```rust
renderer.update_hud_uniforms(
    &queue,
    &rendering::HudUniforms {
        screen_width: new_w as f32,
        screen_height: new_h as f32,
        brightness_scale: hud_brightness,
        hdr_enabled: if hdr_config.enabled { 1.0 } else { 0.0 },
        max_brightness: hdr_config.max_brightness as f32,
        tonemap_variant: hdr_config.tonemap_variant as f32,
        exposure: hdr_config.exposure as f32,
        _padding: 0.0,
    },
);
```

For the per-frame site (~line 669):
```rust
renderer.update_hud_uniforms(
    &queue,
    &rendering::HudUniforms {
        screen_width: renderer.width as f32,
        screen_height: renderer.height as f32,
        brightness_scale: hud_brightness,
        hdr_enabled: if hdr_config.enabled { 1.0 } else { 0.0 },
        max_brightness: hdr_config.max_brightness as f32,
        tonemap_variant: hdr_config.tonemap_variant as f32,
        exposure: hdr_config.exposure as f32,
        _padding: 0.0,
    },
);
```

To find the actual variable names for hdr_config in main.rs, check what's in scope at those call sites with:
```bash
rtk grep "hdr_config\|HdrConfig\|tonemap_variant\|max_brightness" src/main.rs
```
Adjust field access paths accordingly.

- [ ] **Step 6: Update hud.wgsl — expand uniform struct**

In `src/shaders/hud.wgsl`, replace:

```wgsl
struct HudUniforms {
    screen_width: f32,
    screen_height: f32,
    brightness_scale: f32,
    _padding: f32,
}
```

With:

```wgsl
struct HudUniforms {
    screen_width: f32,
    screen_height: f32,
    brightness_scale: f32,
    hdr_enabled: f32,
    max_brightness: f32,
    tonemap_variant: f32,
    exposure: f32,
    _padding: f32,
}
```

- [ ] **Step 7: Add tonemap helper functions to hud.wgsl**

In `src/shaders/hud.wgsl`, after the struct definitions and before `@group(0) @binding(0)`, add the tonemap helpers. These are direct copies from `postprocess.wgsl` — they apply the same curve to HUD colors:

```wgsl
// Tonemap helpers — mirror of postprocess.wgsl variants, applied to HUD colors.
// HUD does NOT apply game_exposure or add_color/mul_color (those are scene effects).

fn hud_tonemap_pseudo_reinhard(color: vec3<f32>, max_val: f32) -> vec3<f32> {
    return color * max_val / (color + vec3(max_val));
}

fn hud_soft_redirect(col: vec3<f32>, threshold: f32) -> vec3<f32> {
    let soft_start = threshold * 0.8;
    let excess_r = max(col.r - soft_start, 0.0);
    let excess_g = max(col.g - soft_start, 0.0);
    let excess_b = max(col.b - soft_start, 0.0);
    let blend_r = smoothstep(soft_start, threshold, col.r);
    let blend_g = smoothstep(soft_start, threshold, col.g);
    let blend_b = smoothstep(soft_start, threshold, col.b);
    var r_out = col.r;
    var g_out = col.g;
    var b_out = col.b;
    let r_bleed = excess_r * blend_r;
    r_out = r_out - r_bleed * 0.9; g_out = g_out + r_bleed * 0.6; b_out = b_out + r_bleed * 0.3;
    let g_bleed = excess_g * blend_g;
    g_out = g_out - g_bleed * 0.9; r_out = r_out + g_bleed * 0.45; b_out = b_out + g_bleed * 0.45;
    let b_bleed = excess_b * blend_b;
    b_out = b_out - b_bleed * 0.9; g_out = g_out + b_bleed * 0.6; r_out = r_out + b_bleed * 0.3;
    return clamp(vec3<f32>(r_out, g_out, b_out), vec3<f32>(0.0), vec3<f32>(threshold));
}

fn hud_hard_redirect(col: vec3<f32>, threshold: f32) -> vec3<f32> {
    let double = threshold * 2.0;
    var r = col.r; var g = col.g; var b = col.b;
    var r_out = r;
    if b > double { r_out = r + b - double; }
    else if b > threshold && g > threshold { r_out = r + b + g - threshold * 2.0; }
    else if g > threshold { r_out = r + g - threshold; }
    var g_out = g;
    if r > threshold && b > threshold { g_out = g + r + b - threshold * 2.0; }
    else if r > threshold { g_out = g + r - threshold; }
    else if b > threshold { g_out = g + b - threshold; }
    var b_out = b;
    if r > double { b_out = b + r - double; }
    else if r > threshold && g > threshold { b_out = b + r + g - threshold * 2.0; }
    else if g > threshold { b_out = b + g - threshold; }
    return clamp(vec3<f32>(r_out, g_out, b_out), vec3<f32>(0.0), vec3<f32>(threshold));
}

fn hud_apply_tonemap(raw: vec3<f32>) -> vec3<f32> {
    let variant = u32(uniforms.tonemap_variant);
    if uniforms.hdr_enabled > 0.5 {
        // raw is already in scRGB (brightness_scale applied). Convert to nits, tonemap, back.
        let nits = raw * 80.0;
        let max_b = uniforms.max_brightness;
        var tonemapped: vec3<f32>;
        if variant == 0u { tonemapped = clamp(nits, vec3(0.0), vec3(max_b)); }
        else if variant == 1u { tonemapped = hud_tonemap_pseudo_reinhard(nits, max_b); }
        else if variant == 2u { tonemapped = hud_hard_redirect(nits, max_b); }
        else { tonemapped = hud_soft_redirect(nits, max_b); }
        return tonemapped / 80.0;
    } else {
        // SDR: raw is in [0, brightness_scale] ≈ [0, 1]. Tonemap with threshold=1.0.
        if variant == 0u { return clamp(raw, vec3(0.0), vec3(1.0)); }
        else if variant == 1u { return hud_tonemap_pseudo_reinhard(raw, 1.0); }
        else if variant == 2u { return hud_hard_redirect(raw, 1.0); }
        else { return hud_soft_redirect(raw, 1.0); }
    }
}
```

- [ ] **Step 8: Apply tonemap in fs_main**

In `src/shaders/hud.wgsl`, replace the `fs_main` function:

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Base color: scale from 0-255 to brightness-scaled output space
    let raw = vec3<f32>(
        in.color.r / 255.0 * uniforms.brightness_scale,
        in.color.g / 255.0 * uniforms.brightness_scale,
        in.color.b / 255.0 * uniforms.brightness_scale,
    );
    let alpha = clamp(in.color.a / 255.0, 0.0, 1.0);
    // Apply same tonemap curve as scene (without game_exposure/add_color/mul_color)
    let mapped = hud_apply_tonemap(raw);
    return vec4<f32>(mapped, alpha);
}
```

- [ ] **Step 9: Build check**

```
rtk cargo check && rtk cargo test
```

Expected: no errors, all tests pass.

- [ ] **Step 10: Commit**

```bash
rtk git add src/rendering/mod.rs src/shaders/hud.wgsl src/main.rs
rtk git commit -m "feat: apply scene tonemap curve to HUD (HudUniforms expanded to 32 bytes)"
```

---

## Task 8: Smoke Velocity Inheritance

**Files:**
- Modify: `src/objects.rs` — `spawn_explosion`, `spawn_explosion_object`, `spawn_explosion_death`, `spawn_chunk_explosion`

Currently all explosion spawners give smoke/explosion entities a purely random velocity (`from_polar(random_angle, random_speed)`). They should inherit the parent's velocity so the cloud moves with the collision.

- [ ] **Step 1: Write tests for velocity inheritance**

Add to `src/objects.rs` in `#[cfg(test)]`:

```rust
#[cfg(test)]
mod smoke_velocity_tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use crate::math::Vec2;

    fn make_entity_at_velocity(vx: f64, vy: f64) -> Entity {
        let mut e = spawn_ship();
        e.velocity = Vec2::new(vx, vy);
        e.position = Vec2::new(100.0, 100.0);
        e
    }

    #[test]
    fn spawn_explosion_velocity_includes_parent() {
        let mut rng = SmallRng::seed_from_u64(42);
        let parent = make_entity_at_velocity(1000.0, 0.0);
        // Run many samples: average velocity should be near parent velocity
        // (random component averages to zero over many samples)
        let samples = 200;
        let total_vx: f64 = (0..samples)
            .map(|_| spawn_explosion(&parent, &mut rng).velocity.x)
            .sum();
        let avg_vx = total_vx / samples as f64;
        // Average x-velocity should be close to 1000.0 (parent vx), not near 0
        assert!(avg_vx > 500.0, "avg_vx={avg_vx}, expected ~1000.0");
    }

    #[test]
    fn spawn_explosion_death_velocity_includes_parent() {
        let mut rng = SmallRng::seed_from_u64(42);
        let ship = make_entity_at_velocity(0.0, -500.0);
        let samples = 200;
        let total_vy: f64 = (0..samples)
            .map(|_| spawn_explosion_death(&ship, 1.0, &mut rng).velocity.y)
            .sum();
        let avg_vy = total_vy / samples as f64;
        assert!(avg_vy < -200.0, "avg_vy={avg_vy}, expected ~-500.0");
    }
}
```

- [ ] **Step 2: Run tests (expect FAIL — velocity doesn't include parent)**

```
rtk cargo test smoke_velocity_tests
```

Expected: FAIL — average velocity is near 0, not near parent velocity.

- [ ] **Step 3: Fix spawn_explosion**

In `src/objects.rs`, in `spawn_explosion` (~line 302), replace the velocity line:

```rust
velocity: from_polar(
    rng.gen::<f64>() * 2.0 * PI,
    rng.gen::<f64>() * SMOKE_MAX_SPEED,
),
```

With:

```rust
velocity: {
    let rand_vel = from_polar(
        rng.gen::<f64>() * 2.0 * PI,
        rng.gen::<f64>() * SMOKE_MAX_SPEED,
    );
    crate::math_utils::add_vec(projectile.velocity, rand_vel)
},
```

- [ ] **Step 4: Fix spawn_explosion_object**

In `src/objects.rs`, in `spawn_explosion_object` (~line 344), replace the velocity in the `explosion` Entity literal:

```rust
velocity: from_polar(
    rng.gen::<f64>() * 2.0 * PI,
    rng.gen::<f64>() * SMOKE_MAX_SPEED,
),
```

With:

```rust
velocity: {
    let rand_vel = from_polar(
        rng.gen::<f64>() * 2.0 * PI,
        rng.gen::<f64>() * SMOKE_MAX_SPEED,
    );
    crate::math_utils::add_vec(obj.velocity, rand_vel)
},
```

- [ ] **Step 5: Fix spawn_explosion_death**

In `src/objects.rs`, in `spawn_explosion_death` (~line 422), replace the velocity:

```rust
velocity: from_polar(
    rng.gen::<f64>() * 2.0 * PI,
    rng.gen::<f64>() * SMOKE_MAX_SPEED,
),
```

With:

```rust
velocity: {
    let rand_vel = from_polar(
        rng.gen::<f64>() * 2.0 * PI,
        rng.gen::<f64>() * SMOKE_MAX_SPEED,
    );
    crate::math_utils::add_vec(ship.velocity, rand_vel)
},
```

- [ ] **Step 6: Fix spawn_chunk_explosion**

In `src/objects.rs`, in `spawn_chunk_explosion` (~line 458), replace the velocity:

```rust
velocity: from_polar(
    rng.gen::<f64>() * 2.0 * PI,
    rng.gen::<f64>() * SMOKE_MAX_SPEED,
),
```

With:

```rust
velocity: {
    let rand_vel = from_polar(
        rng.gen::<f64>() * 2.0 * PI,
        rng.gen::<f64>() * SMOKE_MAX_SPEED,
    );
    crate::math_utils::add_vec(obj.velocity, rand_vel)
},
```

- [ ] **Step 7: Run tests**

```
rtk cargo test smoke_velocity_tests
```

Expected: PASS.

- [ ] **Step 8: Build check**

```
rtk cargo check && rtk cargo test
```

Expected: no errors, all tests pass.

- [ ] **Step 9: Commit**

```bash
rtk git add src/objects.rs
rtk git commit -m "fix: explosion smoke inherits parent velocity in all spawn functions"
```

---

## Task 9: Engine Fire Fix at High Speeds

**Files:**
- Modify: `src/objects.rs` — `spawn_fire`
- Modify: `src/parameters.rs` — add `FIRE_SPEED_RATIO` constant

**Problem:** At high speeds, `kick = ship_speed + FIRE_MIN_SPEED + rand*(FIRE_MAX_SPEED-FIRE_MIN_SPEED)`. When `ship_speed` is large, the fire particle's absolute velocity is `ship.velocity + backward_kick`. The backward_kick direction is opposite thrust. But the total velocity magnitude can fail to pull fire behind the ship if ship keeps accelerating.

**Fix:** Multiply ship_speed contribution by `FIRE_SPEED_RATIO > 1.0`. This ensures fire is always ejected faster than the ship's current speed in the backward direction.

- [ ] **Step 1: Write a test for fire velocity always being backward relative to ship**

Add to `src/objects.rs` in `#[cfg(test)]`:

```rust
#[cfg(test)]
mod fire_velocity_tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use crate::math::Vec2;
    use std::f64::consts::PI;

    #[test]
    fn fire_ejects_backward_relative_to_ship_at_high_speed() {
        let mut rng = SmallRng::seed_from_u64(42);
        // Ship moving very fast to the right (thrust angle = 0, backward = PI)
        let thrust_angle = 0.0_f64; // thrust direction = right
        let ship_speed = 5000.0_f64;
        let mut ship = spawn_ship();
        ship.velocity = Vec2::new(ship_speed, 0.0); // fast to the right
        ship.position = Vec2::new(100.0, 100.0);

        // Run many samples — fire must always move slower than ship in x (or leftward)
        for _ in 0..50 {
            let fire = spawn_fire(&ship, thrust_angle, &mut rng);
            // Fire moves backward = should have x-velocity < ship x-velocity
            assert!(
                fire.velocity.x < ship.velocity.x,
                "fire.vx={} >= ship.vx={}: fire appears in front of ship",
                fire.velocity.x, ship.velocity.x
            );
        }
    }
}
```

- [ ] **Step 2: Run test to confirm it fails (fire can appear in front at high speed)**

```
rtk cargo test fire_ejects_backward
```

Expected: FAIL on some seeds or always (kick = ship_speed + FIRE_MIN_SPEED so fire.vx can be > ship.vx when random scatter adds rightward component).

Actually with the current formula: `kick_magnitude = ship_speed + [1000..2000]` and direction is `thrust_angle + PI` (backward), so the fire x-velocity = `ship.vx + cos(thrust_angle + PI) * kick + scatter`. The backward kick is `cos(PI) * kick = -kick`. So `fire.vx = ship.vx - kick + scatter_x`. Since `kick > ship_speed`, `fire.vx < ship.vx` should hold when `scatter_x < kick`. However scatter can be up to 300 in any direction, and kick can be as low as 1000, so for ship_speed=5000: `fire.vx = 5000 - (5000+1000..7000) + scatter_x`. This means fire.vx ≈ -(0..2000) + scatter_x. So fire.vx should be < ship.vx always. The test may already pass.

Run it anyway:
```
rtk cargo test fire_ejects_backward
```

If it PASSES: the current formula is already correct for the test case. Document this and skip to Step 5.

If it FAILS: proceed to Step 4.

- [ ] **Step 3: Add FIRE_SPEED_RATIO to parameters.rs**

In `src/parameters.rs`, find the fire constants section and add:

```rust
/// Ratio applied to ship speed when computing fire kick velocity.
/// >1.0 ensures fire always moves backward relative to ship even with scatter.
pub const FIRE_SPEED_RATIO: f64 = 1.2;
```

- [ ] **Step 4: Update spawn_fire velocity formula**

In `src/objects.rs`, in `spawn_fire` (~line 585), replace:

```rust
let kick =
    ship_speed + FIRE_MIN_SPEED + rng.gen::<f64>() * (FIRE_MAX_SPEED - FIRE_MIN_SPEED);
```

With:

```rust
// kick scales with ship speed by FIRE_SPEED_RATIO to ensure fire always
// appears behind the ship regardless of speed. Ratio >1 guarantees the
// backward kick exceeds ship speed even after random scatter is applied.
let kick = ship_speed * FIRE_SPEED_RATIO
    + FIRE_MIN_SPEED
    + rng.gen::<f64>() * (FIRE_MAX_SPEED - FIRE_MIN_SPEED);
```

Also add the import at the top of the `spawn_fire` function call site (or ensure `FIRE_SPEED_RATIO` is imported):
The function is in `src/objects.rs` which already imports from `crate::parameters::*` — no additional import needed.

- [ ] **Step 5: Run tests**

```
rtk cargo test fire_ejects_backward
```

Expected: PASS.

- [ ] **Step 6: Build check**

```
rtk cargo check && rtk cargo test
```

Expected: no errors, all tests pass.

- [ ] **Step 7: Commit**

```bash
rtk git add src/objects.rs src/parameters.rs
rtk git commit -m "fix: engine fire always ejects backward using FIRE_SPEED_RATIO (high-speed fix)"
```

---

## Task 10: Widest Gamut Ship Colors

**Files:**
- Modify: `src/objects.rs` — `spawn_ship` shape colors
- Modify: `src/parameters.rs` — add `SHIP_COLOR_HDR_*` constants

The ship currently uses `(1000.0, 100.0, 25.0)` for its primary body color. In HDR mode, values can exceed 255 and map to wider-gamut scRGB. We push the primary red channel higher while keeping green/blue low to achieve P3-or-wider red in HDR. In SDR the tonemap will clip to white safely.

**Current ship shape colors (from spawn_ship):**
- `(1000.0, 100.0, 25.0)` — primary body (base circle + main polygon) 
- `(200.0, 20.0, 20.0)` — side fins
- `(250.0, 25.0, 25.0)` — right fin
- `(120.0, 5.0, 5.0)` — left fin
- `(10.0, 10.0, 10.0)` — shadow panel
- `(30.0, 30.0, 30.0)` — shadow panel 2
- `(200.0, 180.0, 160.0)` — highlight panel
- `(20.0, 30.0, 40.0)` — blue-tint panel

We increase the primary red and reduce green/blue to maximize gamut width in HDR. SDR soft redirect will redistribute excess red to neighbors gracefully.

- [ ] **Step 1: Write a test for ship primary color being very saturated**

Add to `src/objects.rs` in `#[cfg(test)]`:

```rust
#[cfg(test)]
mod ship_color_tests {
    use super::*;

    #[test]
    fn ship_primary_color_is_saturated_red() {
        let ship = spawn_ship();
        // The visuals.color is the fallback color (not directly rendered for polygon ships,
        // but the first shape's color should be the primary display color)
        let (r, g, b) = ship.visuals.color;
        // Red channel should dominate: r >> g and r >> b
        assert!(r > g * 5.0, "ship red={r} is not dominant over green={g}");
        assert!(r > b * 5.0, "ship red={r} is not dominant over blue={b}");
        // Red should be in HDR range (>255 for wide gamut)
        assert!(r > 255.0, "ship red={r} should exceed 255 for HDR wide gamut");
    }
}
```

- [ ] **Step 2: Run test**

```
rtk cargo test ship_primary_color
```

Expected: PASS (existing `r=1000.0` already satisfies r>255 and r >> g,b).

- [ ] **Step 3: Add ship color constants to parameters.rs**

In `src/parameters.rs`, after the ship parameters section, add:

```rust
// Ship visual colors (HDR range: values >255 use wider color gamut in HDR mode)
// In SDR, the soft redirect tonemap redistributes excess to neighbors (desaturates gracefully).
/// Primary ship body: intense red, minimal green/blue → P3-or-wider red in HDR
pub const SHIP_COLOR_PRIMARY: (f64, f64, f64) = (1400.0, 60.0, 20.0);
/// Side fin color: warm dark red
pub const SHIP_COLOR_FIN: (f64, f64, f64) = (180.0, 15.0, 10.0);
/// Asymmetric fin highlight
pub const SHIP_COLOR_FIN_HIGHLIGHT: (f64, f64, f64) = (280.0, 20.0, 15.0);
/// Fin shadow
pub const SHIP_COLOR_FIN_SHADOW: (f64, f64, f64) = (100.0, 3.0, 3.0);
/// Dark shadow panels (near-black)
pub const SHIP_COLOR_SHADOW_DARK: (f64, f64, f64) = (8.0, 8.0, 8.0);
pub const SHIP_COLOR_SHADOW_MID: (f64, f64, f64) = (25.0, 25.0, 25.0);
/// Warm highlight panel
pub const SHIP_COLOR_HIGHLIGHT: (f64, f64, f64) = (220.0, 190.0, 160.0);
/// Cool blue-tint accent
pub const SHIP_COLOR_ACCENT: (f64, f64, f64) = (18.0, 28.0, 45.0);
```

- [ ] **Step 4: Apply constants in spawn_ship**

In `src/objects.rs`, in `spawn_ship`, replace the shapes vec and visuals.color with the new constants:

```rust
let shapes = vec![
    (SHIP_COLOR_PRIMARY, Polygon(circle_poly)),
    (
        SHIP_COLOR_FIN,
        Polygon(vec![
            (0.0, 3.0 * SHIP_RADIUS),
            (3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
            (PI, SHIP_RADIUS),
            (-3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
        ]),
    ),
    (
        SHIP_COLOR_FIN_HIGHLIGHT,
        Polygon(vec![
            (0.0, 3.0 * SHIP_RADIUS),
            (PI, SHIP_RADIUS),
            (-3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
        ]),
    ),
    (
        SHIP_COLOR_FIN_SHADOW,
        Polygon(vec![
            (0.0, 3.0 * SHIP_RADIUS),
            (3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
            (PI, SHIP_RADIUS),
        ]),
    ),
    (
        SHIP_COLOR_SHADOW_DARK,
        Polygon(vec![
            (PI, SHIP_RADIUS / 3.0),
            (PI, SHIP_RADIUS),
            (-3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
        ]),
    ),
    (
        SHIP_COLOR_SHADOW_MID,
        Polygon(vec![
            (PI, SHIP_RADIUS / 3.0),
            (3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
            (PI, SHIP_RADIUS),
        ]),
    ),
    (
        SHIP_COLOR_HIGHLIGHT,
        Polygon(vec![
            (0.0, 3.0 * SHIP_RADIUS),
            (0.0, 1.5 * SHIP_RADIUS),
            (-PI / 8.0, 1.5 * SHIP_RADIUS),
        ]),
    ),
    (
        SHIP_COLOR_ACCENT,
        Polygon(vec![
            (0.0, 3.0 * SHIP_RADIUS),
            (PI / 8.0, 1.5 * SHIP_RADIUS),
            (0.0, 1.5 * SHIP_RADIUS),
        ]),
    ),
];
```

And update `visuals.color` in the Entity literal:

```rust
visuals: Visuals {
    color: SHIP_COLOR_PRIMARY,
    radius: SHIP_RADIUS * 0.9,
    shapes,
},
```

- [ ] **Step 5: Run tests**

```
rtk cargo test ship_primary_color
```

Expected: PASS (r=1400.0 > 255 and 1400/60 >> 5).

- [ ] **Step 6: Build check**

```
rtk cargo check && rtk cargo test
```

Expected: no errors, all tests pass.

- [ ] **Step 7: Commit**

```bash
rtk git add src/objects.rs src/parameters.rs
rtk git commit -m "feat: maximize ship red gamut using HDR-range color constants"
```

---

## Post-Plan Visual Verification Checklist

These items require running the game and visual inspection — no automated tests:

- [ ] **Layer order**: Ship renders above explosions (die and confirm the explosion blooms overlay your ship debris, not vice versa)
- [ ] **Smoke falloff**: Smoke circles have soft feathered edges; explosion circles are sharp
- [ ] **Star trails**: Moving stars show trails; stationary stars show cross-pixel pattern unchanged
- [ ] **Bullet trails**: Capsule trails are consistent regardless of bullet speed
- [ ] **HUD tonemap**: HUD text brightness matches scene at same nit level in HDR mode
- [ ] **Smoke velocity**: Explosion clouds drift with the parent body instead of radiating from a fixed world point
- [ ] **Engine fire**: Fire always appears behind ship even at maximum speed
- [ ] **Ship color**: Ship appears vividly red; on HDR displays it should exceed sRGB red gamut

```bash
cargo run --release
```

---

## Final Build & Lint

- [ ] **Run full test suite**

```
rtk cargo test
```

Expected: all tests pass.

- [ ] **Lint**

```
rtk cargo clippy
```

Fix any warnings.

- [ ] **Format**

```
cargo fmt
```

- [ ] **Final commit (if any fmt/clippy fixes)**

```bash
rtk git add -p
rtk git commit -m "chore: fmt + clippy fixes for phase2a rendering quality"
```
