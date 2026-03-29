# Phase 1: Rendering Pipeline Overhaul — Design Spec

**Date**: 2026-03-28
**Branch**: v2-phase1-rendering (to be cut from v2-phase0-foundation after Phase 0 merges)
**Status**: Approved

---

## Overview

Replace the single-pass, CPU-driven rendering pipeline with a multi-pass GPU pipeline featuring:
- Offscreen HDR rendering into `Rgba16Float`
- SDF-based entity rendering (circles and capsules) via instanced quads
- Post-process tonemapping pass (multiple variants, compile-time selected)
- Proper anti-aliasing (MSAA for polygons, SDF smoothstep for circles/capsules)
- Separate HUD pass unaffected by tonemapping/exposure

Each migration step produces a playable, visually correct game — no big-bang rewrites.

---

## Multi-Pass Pipeline

| Pass | Target | Contents | AA |
|------|--------|----------|----|
| Pass 1: World polygons | `Rgba16Float` offscreen (multisampled) | Ship, asteroids — CPU-triangulated polygons | MSAA 2×/4× (toggleable) |
| Pass 2: SDF entities | Same offscreen (resolved) | Circles (smoke, explosions, chunks, fire, muzzle) + capsules (projectile trails, star trails) — instanced quads with SDF fragment | SDF AA (smoothstep) |
| Pass 3: Post-process | Swapchain surface | Fullscreen quad: tonemapping, `game_exposure`, `add_color`, `mul_color` | None |
| Pass 4: HUD | Swapchain surface | Score, stage, health, pause menu — fixed screen-space coords | MSAA |

**Key constraint**: HUD (Pass 4) is NOT affected by post-process tonemapping. UI stays readable regardless of exposure changes.

---

## Shader Architecture

New shader files replacing single `shape.wgsl`:

| Shader | Purpose |
|--------|---------|
| `world.wgsl` | Polygon vertex/fragment with `zoom_factor` uniform (hardcoded `1.0` for Phase 1), per-vertex HDR color |
| `sdf.wgsl` | Instanced quads — vertex shader positions quad from instance data, fragment evaluates circle or capsule SDF with smoothstep AA |
| `postprocess.wgsl` | Fullscreen quad, samples offscreen `Rgba16Float` texture, applies selected tonemapping variant via compile-time const |
| `hud.wgsl` | Screen-space polygon vertex/fragment (no zoom, no exposure) — functionally equivalent to current `shape.wgsl` |

### Tonemapping Variants (`postprocess.wgsl`)

Selected by compile-time const `TONEMAP_VARIANT` — zero runtime overhead for unused variants:

| Const value | Name | Description |
|-------------|------|-------------|
| `0` | `TONEMAP_FAITHFUL` | 1:1 port of CPU `redirect_spectre_wide`. Validation baseline — output must match V1 visually. |
| `1` | `TONEMAP_SPECTRAL_BLEED` | Smooth nearest-wavelength redistribution following spectral order (R→orange→yellow→white). Uses `smoothstep`-based redistribution where excess in one channel bleeds to its spectral neighbor. |
| `2` | `TONEMAP_ACES` | ACES filmic tonemapping curve. Industry standard for HDR games/film. |
| `3` | `TONEMAP_REINHARD` | Luminance-based Reinhard mapping. |

All variants operate on the `Rgba16Float` buffer — inherently HDR-ready. `TONEMAP_FAITHFUL` must be validated first (visually identical to V1) before other variants are added.

### SDF Shapes (`sdf.wgsl`)

```wgsl
// Circle SDF
fn sdf_circle(uv: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return length(uv - center) - radius;
}

// Capsule SDF (distance to line segment gives mathematically perfect capsule)
fn sdf_capsule(uv: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, radius: f32) -> f32 {
    return distance_to_segment(uv, p0, p1) - radius;
}
```

### SDF AA (compile-time const `SDF_AA_ENABLED`):

| Value | Behavior |
|-------|----------|
| `true` | `smoothstep(0.5, -0.5, dist)` — maps ±0.5 pixel distance to 100%–0% opacity. Essentially free AA. |
| `false` | `step(0.0, -dist)` — hard edges. |

### Trail Rendering — Two Const-Selectable Implementations

Both selectable at compile time for A/B visual and performance comparison:

1. **Capsule SDF** (`TRAIL_IMPL_CAPSULE`): single instanced quad per trail, segment-distance SDF in fragment shader. Supports intensity falloff via `exp(-dist * falloff)` to replace the current 4 concentric `render_light_trail` draws.
2. **Composite** (`TRAIL_IMPL_COMPOSITE`): 2 circle SDF instances + 1 rectangle quad per trail segment. More instances, simpler per-instance shader.

---

## Buffer Management

Pre-allocated GPU buffers created once at startup, written each frame via `queue.write_buffer`:

| Buffer | Type | Max capacity | Contents |
|--------|------|-------------|---------|
| `polygon_vertices` | `Vec<Vertex>` | 64K vertices | Ship, asteroid polygons, background |
| `sdf_circle_instances` | `Vec<CircleInstance>` | 4K instances | Smoke, fire, muzzle, explosions, chunks |
| `sdf_capsule_instances` | `Vec<CapsuleInstance>` | 2K instances | Projectile trails, star trails |
| `hud_vertices` | `Vec<Vertex>` | 16K vertices | Score, stage, health text, pause menu |
| `postprocess_quad` | Static | 6 vertices | Written once at startup, never updated |

### Instance Structs

```rust
// 28 bytes — aligned
struct CircleInstance {
    center: [f32; 2],
    radius: f32,
    color: [f32; 4],
}

// 36 bytes — aligned
struct CapsuleInstance {
    p0: [f32; 2],
    p1: [f32; 2],
    radius: f32,
    color: [f32; 4],
}
```

WGSL equivalents:
```wgsl
struct CircleInstance  { center: vec2<f32>, radius: f32, color: vec4<f32> }  // 28 bytes
struct CapsuleInstance { p0: vec2<f32>, p1: vec2<f32>, radius: f32, color: vec4<f32> }  // 36 bytes
```

### Particle Budget Constants (`parameters.rs`)

| Entity | Cap | Despawn Strategy |
|--------|-----|-----------------|
| Smoke | 2048 | Oldest first |
| Fire/muzzle | 512 | Oldest first |
| Chunks | 512 | Lowest opacity first |
| Explosions | 256 | Oldest first |
| Projectiles | 256 | Never culled (gameplay-critical) |

**Graceful degradation**: At 90% capacity, accelerate fade of lowest-priority particles — smooth degradation, no hard pop.

---

## SDF Entity Mapping

| Entity | V1 Rendering | Phase 1 Rendering |
|--------|-------------|------------------|
| Smoke | `fill_circle` (CPU fan triangulation) | SDF circle instance |
| Fire, muzzle flash | `fill_circle` | SDF circle instance |
| Explosions | `fill_circle` | SDF circle instance |
| Chunks | `fill_circle` via `render_chunk` | SDF circle instance |
| Asteroid base circle | `fill_circle` + polygon overlay | **Removed** — polygon shape is the sole visual |
| Ship base circle | `fill_circle` + polygon layers | **Converted to polygon** geometry |
| Projectile trails | 4× concentric `render_light_trail` | SDF capsule instance (or composite, const-selected) |
| Star trails | `render_star_trail` line/cross | SDF capsule instance (or composite, const-selected) |
| Ship/asteroid polygons | CPU `fill_poly` scanline fill | **Unchanged** — stays CPU-triangulated, uploaded to `polygon_vertices` |

---

## Color Pipeline Changes

### V1 (Before):

```
entity.color
  → hdr()
  → intensify(exposure * hdr_exposure)
  → to_rgba(add_color, mul_color, game_exposure)   ← tonemapping on CPU per-entity
  → [u8; 4] vertex color
  → passthrough shader
  → swapchain
```

### Phase 1 (After):

```
entity.color
  → hdr()
  → intensify(hdr_exposure)          ← per-entity exposure stays CPU-side
  → [f32; 4] vertex color            ← HDR f32, not u8 clamped
  → world/sdf shader
  → Rgba16Float offscreen
        ↓
  postprocess shader(game_exposure, add_color, mul_color, tonemapping variant)
        ↓
  swapchain
```

**Key changes**:
- Per-entity `hdr_exposure` stays CPU-side (multiplied before vertex upload). No change to per-entity logic.
- `game_exposure`, `add_color`, `mul_color` move to GPU post-process uniform buffer.
- `redirect_spectre_wide` (and tonemapping alternatives) runs once per pixel in post-process, not per-vertex on CPU.
- Vertex colors are now `f32` HDR values — no `[u8; 4]` clamping loss.

---

## Anti-Aliasing Strategy

Three independent, toggleable mechanisms:

| Mechanism | Applies to | Cost | Toggle |
|-----------|-----------|------|--------|
| **SDF AA** | SDF entities (circles, capsules) | ~Free (smoothstep in existing fragment pass) | Compile-time const `SDF_AA_ENABLED` |
| **MSAA 2×/4×** | Polygon geometry (Pass 1 world, Pass 4 HUD) | GPU multisampling overhead | Runtime: off / 2× / 4× |
| **Combined** | Both | Sum of above | SDF AA on circles + MSAA on polygons |

MSAA does **not** apply to SDF entities — SDF AA handles those. The two mechanisms are complementary.

---

## Code Changes

### New Files

| File | Description |
|------|-------------|
| `src/shaders/world.wgsl` | World polygon vertex/fragment shader |
| `src/shaders/sdf.wgsl` | SDF instanced circle + capsule shader |
| `src/shaders/postprocess.wgsl` | Post-process tonemapping fullscreen quad |
| `src/shaders/hud.wgsl` | HUD screen-space vertex/fragment shader |
| `src/renderer.rs` | Multi-pass pipeline orchestration (extracted from `main.rs`) |

### Modified Files

| File | Changes |
|------|---------|
| `src/main.rs` | Pipeline setup: 4 render passes, offscreen `Rgba16Float` texture, MSAA resolve, buffer pre-allocation |
| `src/rendering/world.rs` | Emit `CircleInstance`/`CapsuleInstance` for SDF entities instead of `fill_circle`. Remove asteroid base circle. Convert ship base circle to polygon. |
| `src/rendering/hud.rs` | Write to separate `hud_vertices` buffer for Pass 4 |
| `src/color.rs` | `to_rgba` no longer applies `game_exposure`/`add_color`/`mul_color` (moved to GPU). `redirect_spectre_wide` stays as CPU reference implementation. |
| `src/parameters.rs` | Add: particle budget constants, AA mode constants, tonemapping variant constant |
| `src/objects.rs` | Remove `fill_circle`-only rendering path |

### Deleted

| Item | Replacement |
|------|-------------|
| `src/shaders/shape.wgsl` | Replaced by 4 specialized shaders |
| `globals.visual.retro`, `globals.visual.scanlines` | Removed entirely |
| `render_scanlines()` and all retro branches | Removed |
| `fill_circle`, `fill_ellipse` | Replaced by SDF instances |
| `dither_radius`, `dither_vec` | Removed if only used by deleted fill paths |

### Kept As-Is

- `fill_poly`, `draw_poly`, `draw_line` — polygon CPU triangulation stays
- `fill_rect` — background quad (2 triangles, trivial)

---

## Migration Strategy

**Invariant**: each step produces a playable, visually correct game. No step may regress gameplay or visuals without an explicit rollback plan.

| Step | Description | Validation |
|------|-------------|-----------|
| 1 | **Offscreen texture + post-process pass** — render existing pipeline into `Rgba16Float` offscreen, post-process samples and outputs unchanged | Pixel-identical to V1 (or within float rounding) |
| 2 | **Move exposure/color to GPU** — port `redirect_spectre_wide` (faithful) to `postprocess.wgsl`. Stop applying `game_exposure`/`add_color`/`mul_color` on CPU | Visually identical to V1 (validation baseline for `TONEMAP_FAITHFUL`) |
| 3 | **Add tonemapping variants** — spectral bleed, ACES, Reinhard alongside faithful. Const-switchable | Each variant compiles and renders without artifacts |
| 4 | **Separate HUD pass** — split HUD vertices into own buffer and Pass 4. HUD unaffected by tonemapping | HUD legibility unchanged at all exposure levels |
| 5 | **SDF circles** — replace smoke/fire/muzzle/explosion/chunk `fill_circle` with instanced SDF. Remove asteroid base circle. Convert ship base circle to polygon | Visual parity: no popping, no missing entities |
| 6 | **SDF capsules** — replace projectile and star trails with capsule SDF instances. Both capsule SDF and composite variants | Trail quality ≥ V1; falloff replaces 4 concentric draws |
| 7 | **MSAA** — multisampled offscreen texture + resolve for polygon passes. Toggleable off/2×/4× | No geometry corruption at any MSAA level |
| 8 | **SDF AA toggle** — smoothstep AA in SDF shader, const-toggleable | SDF entities have smooth edges when enabled |
| 9 | **Delete retro/scanline** — remove all dead code paths | `cargo clippy` clean; no dead code warnings |
| 10 | **Particle budgets** — caps and graceful despawn logic | No crash or stutter at cap; degradation is smooth |

---

## Dependencies on Later Phases

| Later Phase | Dependency |
|-------------|-----------|
| Phase 3 (Camera & Zoom) | `zoom_factor` uniform exists in `world.wgsl` but hardcoded to `1.0` in Phase 1. Phase 3 wires it to the camera system. |
| Phase 4 (GPU Particles) | Offscreen texture and SDF infrastructure are ready. Compute shader dispatch is Phase 4's work. |
| Phase 6 (HDR Output) | `Rgba16Float` pipeline and post-process tonemapping are ready. Phase 6 swaps the swapchain surface format to HDR and adds calibration menu. |

---

## Open Questions / Decisions Deferred to Implementation

- **Capsule SDF vs. Composite default**: which variant ships as default? Measure GPU cost at 2K capsule instances. Recommendation: capsule SDF (single quad, fewer draw calls).
- **MSAA sample count at startup**: default 2× or 4×? Depends on measured GPU cost on target hardware. Expose as parameter in `parameters.rs`.
- **`postprocess_quad` vertex format**: can reuse `Vertex` struct or use a dedicated minimal struct (`vec2<f32>` only). Keep uniform with `hud.wgsl` for simplicity.
- **`CircleInstance` padding**: 28 bytes is not 16-byte aligned. Verify wgpu alignment requirements; add padding if needed to reach 32 bytes.
