# Asteroids

A from-scratch Rust port of the classic arcade game, originally written in OCaml. Custom GPU renderer, advanced visual effects, and a solid gameplay loop.

**V1** is tagged and stable. **V2 refactor** is actively in progress — see roadmap below.

## Stack

- **Rendering**: [wgpu](https://wgpu.rs/) (Vulkan backend) — layered compositing 2D renderer
- **Windowing / input**: SDL2 (uses `create_surface_unsafe` — SDL2 `Window` carries an `Rc`, not `Sync`)
- **Language**: Rust, custom HDR-ready linear color pipeline

## Screenshot

<!-- TODO: add screenshot -->

## Features

### Rendering
- **Layered compositing pipeline** — 7-layer render graph: background → star trails → smoke → polygons → effect explosions → tonemap → HUD
- **HDR rendering** with configurable tonemapping variants and exposure slider
- **SDF rendering** for circles and capsules (smoke, trails, explosions) with soft falloff
- **Additive blending** for trails and explosion effects
- **Polygon MSAA** — ship and asteroids render into an MSAA buffer, resolved and composited onto the offscreen target
- **Glyph-based text rendering** — HUD text drawn as polygon outlines; no font files required

### Gameplay
- Fullscreen with 16:9 safe zone (F11 or Alt+Enter to toggle)
- Screenshake on damage and explosions
- **Explosion shockwave push** — nearby entities receive an outward impulse on asteroid destruction
- Explosion chunk trails
- Dynamic color effects (shield flash, damage pulse, engine fire)
- Death phase: controllable burning wreck before respawn
- Teleport (Space) with cooldown
- Wave system: asteroid count and speed scale per stage

### Input & Controls
- **Gamepad support** — world-space controls (aim with right stick, all actions mapped)
- Scancode-based keyboard input — layout-independent (AZERTY/QWERTY both work)

### HUD & Menus
- **Pause menu** with toggle, cycle, and slider entry types — configure tonemapping, exposure, visual options
- HUD bars (health, boost, etc.) and FPS counter
- HUD shares the same HDR tonemap curve as the scene

### Engine
- **Fixed-dt deterministic simulation** — physics runs at a fixed timestep, decoupled from render rate
- **i18n groundwork** — locale system with `.ron` locale files (English wired in)
- **Video capture support** — snapshot and PNG export for testing/recording

## Build & Run

Requires Rust + Cargo. SDL2 is bundled on Windows (`SDL2.dll` included).

```bash
cargo build --release
cargo run --release
```

## Controls

### Keyboard

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

### Gamepad

| Input | Action |
|-------|--------|
| Left stick | Thrust / rotate |
| Right stick | Aim (world-space) |
| Right trigger | Fire |
| Face button (South) | Teleport |
| Start | Pause |

## Module Structure

```
src/
├─ main.rs          — SDL2/wgpu init, game loop, event handling
├─ lib.rs           — Dual lib+bin crate, module declarations
├─ game.rs          — GameState and render_frame orchestration
├─ update.rs        — Simulation update logic
├─ spawning.rs      — Entity spawning functions
├─ input.rs         — Keyboard, mouse, and gamepad input
├─ camera.rs        — Predictive camera system
├─ pause_menu.rs    — Pause UI (toggle/cycle/slider items)
├─ capture.rs       — Video capture, PNG export, snapshot testing
├─ field.rs         — Distortion field system (groundwork)
├─ locale.rs        — i18n skeleton and locale loading
├─ locales/
│   └─ en.ron       — English locale strings
├─ math.rs          — Vec2, HdrColor, matrix utilities
├─ rendering/
│   ├─ pipeline.rs  — GPU pipeline setup, pass orchestration
│   ├─ textures.rs  — Texture resources, SDF atlases, offscreen targets
│   ├─ glyphs.rs    — Glyph rendering (polygon glyphs, character lookup)
│   └─ postprocess.wgsl — Tonemap, color effects, SSAA compositing
├─ physics/
│   ├─ mod.rs       — Spatial grid infrastructure
│   ├─ collision.rs — Detection primitives
│   └─ response.rs  — Damage, elastic bounce, shockwave push
└─ util/            — Configuration, RON scenarios, test helpers
```

## Testing

```bash
cargo test                               # All tests
cargo test --test math_properties        # Math function tests
cargo test --test color_properties       # Color pipeline tests
cargo test --test conservation_properties  # Physics conservation tests
```

## V2 Roadmap

V2 is a full architectural overhaul targeting a modern GPU rendering pipeline and richer gameplay. All tunable values live in `parameters.rs`.

| Phase | Name | Status | Summary |
|-------|------|--------|---------|
| 0 | Foundation | ✅ Complete | Module extraction, Vec2, HdrColor, 290+ tests |
| 1 | Rendering Foundation | ✅ Complete | Multi-pass HDR pipeline, SDF circles, post-process quad |
| 2A | Rendering Visual Quality | ✅ Complete | Layered compositing, additive blend, trail system, HUD tonemap |
| 2B | AA & Tooling | ✅ Complete | Polygon MSAA, capture tooling, code restructure |
| 3 | Camera & Zoom | Planned | Ship zone tracking, dynamic zoom |
| 4 | Physics | Planned | parry2d polygon collisions, distortion fields |
| 5 | GPU Particles | Planned | Compute shader particle pool |
| 6 | Weapons | Planned | 3 weapon types, scroll-wheel switch, HUD indicators |
| 7 | i18n & Glyphs | Planned | Extended character sets, locale files |
| 8 | Haptics | Planned | Gamepad vibration |
| 9 | Audio | Planned | Synthesized engine sounds |

## Branches & Tags

| Ref | Description |
|-----|-------------|
| `master` | Main branch (V1 tagged, V2 merges in) |
| `ocaml` | Historical OCaml version |
| `v1` | V1 release tag |

## History

This started as an OCaml assignment. The original OCaml version is preserved on the [`ocaml`](../../tree/ocaml) branch. The Rust rewrite (`master`) targets modern GPU rendering via wgpu.
