# Asteroids

A from-scratch Rust port of the classic arcade game, originally written in OCaml. Collision physics, advanced visual effects, and a solid gameplay loop.

**V1** is tagged. **V2 refactor** is in progress — post-process rendering pipeline, GPU particles, polygon physics, and more.

## Stack

- **Rendering**: [wgpu](https://wgpu.rs/) (Vulkan backend) — immediate-mode 2D renderer
- **Windowing / input**: SDL2
- **Other**: `rand`, custom HDR-ready linear color pipeline

## Features

- Fullscreen with 16:9 safe zone (F11 or Alt+Enter to toggle)
- Screenshake on damage/explosions
- Chunk explosion trails
- Dynamic color effects (shield flash, damage pulse, engine fire)
- Scancode-based input — layout-independent (AZERTY/QWERTY both work)
- Pause menu with toggle buttons
- Death phase: controllable burning wreck before respawn
- Teleport (Space)
- HUD bars (health, boost, etc.)
- FPS counter

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
├─ objects.rs       — Entity, spawning, predicates
├─ parameters.rs    — Globals, game constants
├─ input.rs         — Player input handling
├─ camera.rs        — Camera system, zoom
├─ pause_menu.rs    — Pause UI, button system
├─ rendering/       — Render pipeline (in progress)
│   ├─ mod.rs       — Renderer2D
│   ├─ world.rs     — Entity rendering
│   └─ hud.rs       — HUD, text, bars
├─ physics/         — Collision system
│   ├─ mod.rs       — Grid infrastructure
│   ├─ collision.rs — Detection primitives
│   └─ response.rs  — Damage, elastic bounce
└─ shaders/
    └─ shader.wgsl  — Vertex/fragment shader
```

## Testing

```bash
cargo test                          # All tests
cargo test --test math_properties   # Math function tests
cargo test --test color_properties  # Color pipeline tests
```

## Branches & Tags

| Ref | Description |
|-----|-------------|
| `master` | Active development (Rust) |
| `ocaml` | Historical OCaml version |
| `v1` | V1 release tag |

## History

This started as an OCaml assignment. The original OCaml version is preserved on the [`ocaml`](../../tree/ocaml) branch. This branch (`master`) is the Rust rewrite targeting modern GPU rendering.