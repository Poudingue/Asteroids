# Asteroids V2 Refactor — Design Specification

**Date**: 2026-03-28
**Status**: Approved
**Scope**: Full architectural overhaul of the Rust/wgpu Asteroids game

---

## Core Principle

ALL behavior is controlled by constants/config — no magic numbers buried in logic.
Every behavioral choice is a tunable parameter. If it affects gameplay, it lives in `config.rs`.

---

## 1. Rendering Pipeline: Forward → Post-Process

### 1.1 Current State

| Property | V1 |
|---|---|
| Render passes | 1 |
| Draw calls | 1 (per frame) |
| Color pipeline | `rgb_of_hdr()` per-vertex on CPU, bakes u8 before upload |
| Offscreen texture | None |
| Post-processing | None |
| Surface format | `Bgra8Unorm` (non-sRGB, no gamma) |
| Shader | Trivial passthrough: pixel → NDC, flat color |

### 1.2 V2 Design: Four Render Passes

#### Pass 1 — World Geometry → `Rgba16Float` offscreen

- Polygon entities: ship, asteroids, projectiles
- Per-entity `hdr_exposure` baked CPU-side (multiplied into vertex color before upload)
- SDF instanced circles for explosions and glow effects
- Preserves flexible CPU-side per-entity brightness (e.g. streak length → brightness mapping)

#### Pass 2 — GPU Compute Particles → same `Rgba16Float` offscreen

- All short-lived visual-only particles: chunks, smoke, fire, sparks, muzzle flash
- CPU sends spawn events only: `(position, velocity, color, lifetime, type)`
- Compute shader handles update + render autonomously, each frame
- Compact particle struct (~48 bytes) replaces `Entity` (~600+ bytes)
- Storage buffer for particle pool; indirect draw for live count

#### Pass 3 — Post-Process Fullscreen Quad

- Global exposure (`game_exposure`) applied in shader
- `add_color` flash and `mul_color` tint applied in shader
- `redirect_spectre_wide` channel-bleed ported to WGSL
- HDR output path (if display supports it) or SDR clamp
- Controlled by `ZOOM_AFFECTS_HUD: bool` constant

#### Pass 4 — HUD Overlay (conditional)

- Only active when `ZOOM_AFFECTS_HUD = false`
- HUD renders in a separate pass at fixed screen space coordinates
- When `ZOOM_AFFECTS_HUD = true`, HUD is part of Pass 1 and zooms with the world

### 1.3 SDF Circle Infrastructure

Instanced quads rendered with SDF fragment shader:

```wgsl
let d = length(uv - center) - radius;
```

- One draw call for all circles
- Per-instance data: `(center: vec2<f32>, radius: f32, color: vec4<f32>)`
- Reusable for future effects: soft edges, glow rings, outlines, distance field queries

---

## 2. HDR Output and Calibration

### 2.1 Internal Pipeline

- Always HDR-ready internally: render to `Rgba16Float`
- HDR display output is OPTIONAL — toggled in settings
- No branching in render logic based on HDR mode; the output path changes, not the pipeline

### 2.2 Calibration Menu

Visible only when HDR is active. Three adjustable values:

| Parameter | Meaning |
|---|---|
| `MAX_BRIGHTNESS` | Peak nits (HDR headroom above SDR white) |
| `PAPER_BRIGHTNESS` | Current "255" equivalent — the SDR white point |
| `INTERFACE_BRIGHTNESS` | HUD and text brightness (currently anchored to "255") |

### 2.3 Tonemapping

- `redirect_spectre_wide` channel-bleed style ported to `postprocess.wgsl`
- Architecture supports swapping in Reinhard or ACES later without structural changes
- Per-entity exposure remains CPU-side (multiplied before vertex upload)

### 2.4 Constants

```rust
HDR_ENABLED: bool
HDR_MAX_BRIGHTNESS: f64
HDR_PAPER_BRIGHTNESS: f64
HDR_INTERFACE_BRIGHTNESS: f64
```

---

## 3. Code Restructure

### 3.1 File Organization

```
src/
├─ main.rs              ← SDL2/wgpu init, event loop
├─ game.rs              ← GameState, update_game (orchestration only, no rendering logic)
├─ rendering/
│   ├─ mod.rs           ← Renderer2D, pipeline setup, render pass orchestration
│   ├─ world.rs         ← Entity rendering (polygons, SDF circles)
│   ├─ particles.rs     ← GPU compute particle system
│   ├─ postprocess.rs   ← Fullscreen HDR/exposure/tonemapping pass
│   └─ hud.rs           ← HUD, bars, text, hearts
├─ physics/
│   ├─ mod.rs           ← Physics step orchestration
│   ├─ collision.rs     ← parry2d integration, broadphase/narrowphase
│   └─ response.rs      ← Elastic bounce, damage, repulsion (game-owned)
├─ entities/
│   ├─ mod.rs           ← Entity trait, common types
│   ├─ ship.rs          ← Ship-specific logic
│   ├─ asteroid.rs      ← Asteroid spawning, splitting
│   ├─ projectile.rs    ← Bullet types, weapon configs
│   └─ explosion.rs     ← Blast entities
├─ camera.rs            ← Zoom, pan, ship zone, screenshake
├─ weapons.rs           ← WeaponType enum, switching, scroll wheel logic
├─ input.rs             ← Input mapping (scancodes, keycodes, scroll)
├─ color.rs             ← HdrColor used everywhere (replaces f64 tuples)
├─ math.rs              ← Vec2 struct with std::ops (Add, Sub, Mul, Neg, etc.)
├─ config.rs            ← Split Globals into focused structs
├─ pause_menu.rs        ← Pause UI, HDR calibration menu
└─ shaders/
    ├─ world.wgsl       ← Polygon + SDF vertex/fragment
    ├─ particle.wgsl    ← Compute + render for particles
    └─ postprocess.wgsl ← Tonemapping, exposure, channel bleed
```

### 3.2 Rust Patterns to Apply

| Pattern | Purpose |
|---|---|
| Traits: `Renderable`, `Collidable`, `Spawnable` | Type-based dispatch instead of Vec-membership |
| Enums with associated data | `WeaponType` carries config; `EntityKind` used for actual match dispatch |
| `Vec2` struct with `std::ops` | Replaces all French free functions (`addtuple`, `soustuple`, `multuple`, `moytuple`, etc.) |
| Builder pattern | Entity construction: `AsteroidBuilder::new().radius(400).at(pos).build()` |
| No raw pointers | Extract `rng` from `GameState`, pass as separate `&mut` parameter |
| `GameError` enum | Typed errors for initialization failures |
| `#[derive]` everywhere | `Clone`, `Copy`, `Debug`, `Default` where applicable |
| Compact particle struct | ~48 bytes for GPU particles, separate from `Entity` |
| All English identifiers | Full rename from French |

### 3.3 French → English Rename Map

| French | English |
|---|---|
| `carre` | `squared` |
| `moytuple` | `midpoint` |
| `soustuple` | `sub` |
| `addtuple` | `add` |
| `multuple` | `mul` |
| `deplac_objet` | `move_entity` |
| `affiche_barre` | `render_bar` |
| `affiche_coeur` | `render_heart` |
| All others | English equivalent |

### 3.4 Specific Bug Fixes and Cleanups

- **Raw pointer hacks**: `render_pause_title`, `render_frame`, `render_hud` use raw pointers to split borrows on `state` vs `state.rng`. Fix by extracting `rng` from `GameState` and passing as separate `&mut` parameter.
- **Globals split**: `Globals` has 76 fields. Split into focused structs: `WeaponConfig`, `CameraState`, `RenderState`, `ExposureState`, `SpawnConfig`.
- **Color types**: Replace all `(f64, f64, f64)` color tuples with `HdrColor` everywhere.
- **Constant deduplication**: Constants duplicated between `objects.rs` and `parameters.rs` — consolidate to single source of truth in `config.rs`.
- **`drain_filter_stable`**: Replace with `Vec::extract_if` (stable Rust since 1.87).
- **Dead code**: Remove `diff` helper, dead `EntityKind` variants (e.g. `Spark`).
- **Retro/scanline rendering**: Removed entirely.
- **`EntityKind` semantic lies**: Chunks use `EntityKind::Asteroid` — this is false. Every `EntityKind` variant must correspond to actual dispatch logic or be removed.

---

## 4. GPU Compute Particles

### 4.1 What Moves to GPU

| Particle type | V1 location | V2 |
|---|---|---|
| Chunks (debris) | `EntityKind::Asteroid` with zero hitbox | GPU particle |
| Engine fire / smoke | `Entity` with custom update | GPU particle |
| Muzzle flash | `Entity` | GPU particle |
| Explosion residue smoke | `Entity` | GPU particle |
| Sparks | `EntityKind::Spark` (dead) | GPU particle |
| Death explosion chunks | `chunks_explo` entities | GPU particle |

### 4.2 Architecture

- **CPU sends**: spawn events `(position, velocity, color, lifetime, decay_rate, type)`
- **GPU compute**: each frame — `position += velocity * dt`, `lifetime -= dt`, radius decay, color fade
- **GPU renders**: instanced SDF circles or point sprites (one draw call)
- **Recycling**: dead particles (`lifetime <= 0`) are recycled in-buffer on GPU
- **Buffer**: storage buffer for particle pool, indirect draw for live count

### 4.3 What Stays CPU-Side

| Entity | Reason |
|---|---|
| Explosions | Deal damage; need physics interaction |
| Projectiles | Collision with asteroids |
| Ship | Player-controlled |
| Asteroids | Physics, splitting, collision |

### 4.4 Constants

```rust
PARTICLE_POOL_SIZE: u32           // max particles in GPU buffer
SMOKE_DECAY_HALF_LIFE: f64
SMOKE_COLOR_DECAY_HALF_LIFE: f64
CHUNK_DECAY_RATE: f64
FIRE_BASE_KICK_SPEED: f64
FIRE_RANDOM_JITTER: f64
MUZZLE_RADIUS_RATIO: f64
MUZZLE_SPEED_RATIO: f64
```

---

## 5. Physics Overhaul

### 5.1 Library

`parry2d` from the Rapier ecosystem.

### 5.2 Architecture

| Layer | Mechanism |
|---|---|
| Broadphase | Bounding circle (AABB) for fast rejection — use existing `ext_radius` as bounding sphere, let parry2d convert to AABB |
| Narrowphase | Full polygon-polygon SAT via parry2d — no more circle cores |
| Spatial indexing | parry2d built-in broadphase replaces the 15x9 fixed grid |
| Collision response | Game-owned (not parry2d) — elastic bounce, damage, repulsion. Keeps game feel. |

### 5.3 Changes from V1

- Remove circle-only collision (`collision_circles`)
- Remove vertex-in-circle narrowphase (`collision_poly`)
- Remove fixed grid (`WIDTH_COLLISION_TABLE`, `HEIGHT_COLLISION_TABLE`, grid jitter)
- Asteroids become true polygons — no more `int_radius` circle core
- Fragment collision uses same system (no more O(n²) brute force)
- Explosion damage: keep circle-based area damage (radius check), but via parry2d queries
- Projectile collision: polygon or small circle (configurable per weapon)

### 5.4 Constants

```rust
COLLISION_BROADPHASE_MARGIN: f64   // extra AABB padding
ELASTIC_BOUNCE_RESTITUTION: f64    // replaces MIN_BOUNCE
REPULSION_FORCE: f64               // replaces MIN_REPULSION
DAMAGE_VELOCITY_RATIO: f64         // replaces ratio_phys_deg
PHYSICS_SUBSTEPS: u32              // for stability at high speeds
```

### 5.5 Tests

- **Unit tests**: polygon-polygon collision correctness
- **Regression tests**: known collision scenarios must produce expected results
- **Benchmarks**: V1 grid vs V2 parry2d at various entity counts (10, 50, 200, 500 entities)

---

## 6. Weapon Switching

### 6.1 Weapon Types

Three weapons defined by config data carried in the `WeaponType` enum:

| Weapon | Pellets | Cooldown | Deviation | Recoil |
|---|---|---|---|---|
| Shotgun | 50 | 0.3 s | 0.3 rad | 1 000 |
| Sniper | 1 | 1.0 s | 0.0 rad (perfect) | 10 000 |
| Machine Gun | 1 | 0.01 s | 0.2 rad | 10 |

### 6.2 Implementation

- `WeaponType` enum with associated config data — replaces loose constants
- `current_weapon: WeaponType` field in `GameState`
- Scroll wheel cycles through weapons (wrapping at both ends)
- `set_weapon()` writes weapon config into active parameters
- Each `WeaponType` uses its own `EntityKind` variant for bullets (visual differentiation)
- HUD indicator shows current weapon selection

### 6.3 Constants (per weapon)

```rust
WEAPON_RECOIL: f64
WEAPON_COOLDOWN: f64
WEAPON_MIN_SPEED: f64
WEAPON_MAX_SPEED: f64
WEAPON_DEVIATION: f64
WEAPON_BULLET_RADIUS: f64
WEAPON_HITBOX_RADIUS: f64
WEAPON_PELLET_COUNT: u32
WEAPON_SCREENSHAKE: f64
WEAPON_EXPOSURE_KICK: f64
WEAPON_FLASH_INTENSITY: f64
```

---

## 7. Dynamic Camera Zoom

### 7.1 Ship Zone

- A rectangle slightly smaller than the 16:9 safe zone inscribed rect
- Defined by `SHIP_ZONE_RATIO: f64` (e.g. `0.8` = 80% of safe zone dimensions)
- When the ship is inside this zone: `zoom = 1.0`
- When the ship crosses the zone boundary: zoom smoothly decreases

### 7.2 Zoom Behavior

- No maximum zoom-out cap — can zoom as far as needed to keep ship on screen
- Zoom uses exponential decay for inertia (same approach as `game_exposure`)
- `ZOOM_DECAY_HALF_LIFE: f64` — controls weight/inertia of zoom transitions
- `ZOOM_OUT_SPEED: f64` — controls how fast zoom decreases when ship exits zone
- `ZOOM_IN_SPEED: f64` — controls how fast zoom recovers when ship returns (may differ)

### 7.3 HUD and Zoom

- `ZOOM_AFFECTS_HUD: bool` — if true, HUD zooms with world (Pass 1); if false, HUD is fixed (Pass 4)
- Default: `true`

### 7.4 Stars and Zoom

- `STARS_AFFECTED_BY_ZOOM: bool` — toggle whether zoom affects star layer at all
- `STARS_ZOOM_INTENSITY: f64` — `0.0` = stars immune to zoom, `1.0` = full zoom effect
- `STARS_PARALLAX_INTENSITY: f64` — scales existing proximity-based parallax
- When zoom affects stars: effective star zoom = `zoom * STARS_ZOOM_INTENSITY * star.proximity`
- When disabled: stars render at fixed screen positions regardless of camera zoom

### 7.5 Constants

```rust
SHIP_ZONE_RATIO: f64
ZOOM_DECAY_HALF_LIFE: f64
ZOOM_OUT_SPEED: f64
ZOOM_IN_SPEED: f64
ZOOM_AFFECTS_HUD: bool
STARS_AFFECTED_BY_ZOOM: bool
STARS_ZOOM_INTENSITY: f64
STARS_PARALLAX_INTENSITY: f64
```

---

## 8. Removals

The following V1 features and patterns are fully removed in V2:

| Removed | Replacement |
|---|---|
| Scanlines rendering | — |
| Retro rendering mode | — |
| Circle collision cores (`int_radius`) | Polygon physics via parry2d |
| Fixed collision grid (15x9) | parry2d spatial indexing |
| French function names | English equivalents |
| Raw pointer hacks in borrow splits | `rng` extracted from `GameState` |
| `drain_filter_stable` | `Vec::extract_if` (stable Rust) |
| `diff` helper (unused) | — |
| Dead `EntityKind` variants (e.g. `Spark`) | — |
| `(f64, f64, f64)` color tuples | `HdrColor` everywhere |
| Semantically incorrect `EntityKind` usage | Enum variants match actual dispatch |

---

## 9. Testing Strategy

### 9.1 Physics

- Unit tests for polygon-polygon collision correctness (known geometry, expected result)
- Regression tests: specific collision scenarios recorded from V1 must produce V2-equivalent results
- Benchmarks: grid vs parry2d at entity counts: 10 / 50 / 200 / 500

### 9.2 Rendering

- Snapshot comparison tests: render known scenes, compare pixel output
- Tests run in headless mode (offscreen texture, no display required)

### 9.3 Weapons

- Unit tests for weapon switching (state transitions, wrapping)
- Unit tests for config application (`set_weapon()` writes correct values)
- Unit tests for cooldown behavior (correct blocking of fire during cooldown)

### 9.4 Camera

- Unit tests for zoom calculation (ship inside/outside zone → correct zoom value)
- Unit tests for ship zone detection (zone boundary conditions)
- Unit tests for exponential decay curves (zoom and exposure convergence)

### 9.5 Integration

- Full frame tests: spawn entities, step physics, render, verify no panics
- Smoke test: run N frames with all entity types active, assert stable state

---

## 10. Dependencies

| Crate | Purpose | Status |
|---|---|---|
| `parry2d` | Polygon collision detection + spatial indexing | New |
| `wgpu` | Multi-pass rendering, compute shaders | Already present |
| `sdl2` | Window, input, event loop | Already present, unchanged |

No other new dependencies are planned. All V2 features are achievable with these three.

---

## Appendix: Constant Inventory by Module

All constants below belong in `config.rs` under focused sub-structs.

### RenderConfig
```rust
HDR_ENABLED: bool
HDR_MAX_BRIGHTNESS: f64
HDR_PAPER_BRIGHTNESS: f64
HDR_INTERFACE_BRIGHTNESS: f64
ZOOM_AFFECTS_HUD: bool
```

### CameraConfig
```rust
SHIP_ZONE_RATIO: f64
ZOOM_DECAY_HALF_LIFE: f64
ZOOM_OUT_SPEED: f64
ZOOM_IN_SPEED: f64
STARS_AFFECTED_BY_ZOOM: bool
STARS_ZOOM_INTENSITY: f64
STARS_PARALLAX_INTENSITY: f64
```

### ParticleConfig
```rust
PARTICLE_POOL_SIZE: u32
SMOKE_DECAY_HALF_LIFE: f64
SMOKE_COLOR_DECAY_HALF_LIFE: f64
CHUNK_DECAY_RATE: f64
FIRE_BASE_KICK_SPEED: f64
FIRE_RANDOM_JITTER: f64
MUZZLE_RADIUS_RATIO: f64
MUZZLE_SPEED_RATIO: f64
```

### PhysicsConfig
```rust
COLLISION_BROADPHASE_MARGIN: f64
ELASTIC_BOUNCE_RESTITUTION: f64
REPULSION_FORCE: f64
DAMAGE_VELOCITY_RATIO: f64
PHYSICS_SUBSTEPS: u32
```

### WeaponConfig (per weapon variant)
```rust
WEAPON_RECOIL: f64
WEAPON_COOLDOWN: f64
WEAPON_MIN_SPEED: f64
WEAPON_MAX_SPEED: f64
WEAPON_DEVIATION: f64
WEAPON_BULLET_RADIUS: f64
WEAPON_HITBOX_RADIUS: f64
WEAPON_PELLET_COUNT: u32
WEAPON_SCREENSHAKE: f64
WEAPON_EXPOSURE_KICK: f64
WEAPON_FLASH_INTENSITY: f64
```
