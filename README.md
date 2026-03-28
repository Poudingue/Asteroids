# Asteroids

A from-scratch Rust port of the classic arcade game, originally written in OCaml. Collision physics, advanced visual effects, and a solid gameplay loop.

**V1** is tagged and stable on `master`. **V2 refactor** is actively in progress — see roadmap below.

## Stack

- **Rendering**: [wgpu](https://wgpu.rs/) (Vulkan backend) — immediate-mode 2D renderer
- **Windowing / input**: SDL2 (uses `create_surface_unsafe` — SDL2 `Window` carries an `Rc`, not `Sync`)
- **Language**: Rust, custom HDR-ready linear color pipeline

## Screenshot

<!-- TODO: add screenshot -->

## Features

- Fullscreen with 16:9 safe zone (F11 or Alt+Enter to toggle)
- Screenshake on damage and explosions
- Explosion chunk trails
- Dynamic color effects (shield flash, damage pulse, engine fire)
- Scancode-based input — layout-independent (AZERTY/QWERTY both work)
- Pause menu with toggleable visual options
- Death phase: controllable burning wreck before respawn
- Teleport (Space) with cooldown
- HUD bars (health, boost, etc.) and FPS counter
- Wave system: asteroid count and speed scale per stage

## Build & Run

Requires Rust + Cargo. SDL2 is bundled on Windows (`SDL2.dll` included).

```bash
cargo build --release
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| W/Z or S | Thrust / brake |
| A/Q or D | Rotate |
| Mouse | Aim |
| Left click | Fire |
| Space | Teleport |
| P | Pause |
| F11 / Alt+Enter | Toggle fullscreen |
| K | Quit |

Keys are scancode-based — physical position matters, not label. ZQSD (AZERTY) and WASD (QWERTY) both work out of the box.

## Module Structure

```
src/
├─ main.rs          — SDL2/wgpu init, game loop, event handling
├─ lib.rs           — Module declarations
├─ game.rs          — GameState, update_game, render_frame orchestration
├─ math.rs          — Vec2 struct with std::ops
├─ math_utils.rs    — Math helper functions
├─ color.rs         — HdrColor, color pipeline
├─ objects.rs       — Entity types and spawn functions
├─ parameters.rs    — Globals, game constants (split into sub-structs)
├─ input.rs         — Player input handling
├─ camera.rs        — Predictive camera system
├─ pause_menu.rs    — Pause UI and button system
├─ rendering/       — Render pipeline
│   ├─ mod.rs       — Renderer2D
│   ├─ world.rs     — Entity rendering
│   └─ hud.rs       — HUD, text, bars
├─ physics/         — Collision system
│   ├─ mod.rs       — Spatial grid infrastructure
│   ├─ collision.rs — Detection primitives
│   └─ response.rs  — Damage, elastic bounce
└─ shaders/
    └─ shader.wgsl  — Vertex/fragment shader
```

## Testing

```bash
cargo test                               # All tests
cargo test --test math_properties        # Math function tests
cargo test --test color_properties       # Color pipeline tests
cargo test --test conservation_properties  # Physics conservation tests
```

## V2 Roadmap

V2 is a full architectural overhaul with no behavior changes in Phase 0. All behavior remains constant-driven — every tunable value lives in `parameters.rs`.

| Phase | Name | Status | Summary |
|-------|------|--------|---------|
| 0 | Foundation | ~80% | Module extraction, Vec2, HdrColor, French→English rename, 290+ tests |
| 1 | Rendering | Planned | Multi-pass pipeline (offscreen HDR), SDF circles, post-process quad |
| 2 | Physics | Planned | parry2d polygon collisions, game owns collision response |
| 3 | Camera & Zoom | Planned | Ship zone tracking, dynamic zoom uniform in vertex shader |
| 4 | GPU Particles | Planned | Compute shader particle pool (chunks, smoke, fire, sparks) |
| 5 | Weapons | Planned | 3 weapon types, scroll-wheel switch, HUD indicators |
| 6 | HDR Output | Planned | Optional HDR display path, calibration menu (150 nits default) |

### Phase 0 progress

- [x] Dual lib+bin crate (`src/lib.rs`)
- [x] Exhaustive safety-net tests (290+)
- [x] `Vec2` struct replacing `(f64, f64)` tuples throughout
- [x] `HdrColor` field renamed `.g` (was `.v`), operator impls added
- [x] French → English identifier rename (71 identifiers)
- [x] Extract `input.rs`, `camera.rs`, `pause_menu.rs` from `game.rs`
- [x] Extract `rendering/` modules from `game.rs`
- [x] Extract `physics/` modules from `game.rs`
- [x] `MAX_DT = 50ms` cap to prevent physics explosions on frame stalls
- [x] Physics conservation tests (mass, momentum, energy — violations documented)
- [ ] `Globals` config restructure (split into typed sub-structs — in progress)
- [ ] Bug fixes (raw pointers, dead code, `EntityKind` dedup)

## Branches & Tags

| Ref | Description |
|-----|-------------|
| `master` | Stable (V1 complete, V2 merges in) |
| `v2-phase0-foundation` | Active V2 work |
| `ocaml` | Historical OCaml version |
| `v1` | V1 release tag |

## History

This started as an OCaml assignment. The original OCaml version is preserved on the [`ocaml`](../../tree/ocaml) branch. The Rust rewrite (`master`) targets modern GPU rendering via wgpu.
