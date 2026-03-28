# Asteroids

A from-scratch Rust port of the classic arcade game, originally written in OCaml. Collision physics, advanced visual effects, and a solid gameplay loop.

## Stack

- **Rendering**: [wgpu](https://wgpu.rs/) (Vulkan backend) — immediate-mode 2D renderer with scanline polygon fill
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
- Teleport (F key)
- HUD bars (health, boost, etc.)
- FPS counter

## Build & Run

Requires Rust + Cargo. SDL2 is bundled on Windows (`SDL2.dll` included).

```bash
cargo build
cargo run
```

## Controls

| Key | Action |
|-----|--------|
| W/Z or S | Thrust / brake |
| A/Q or D | Rotate |
| Space | Fire |
| F | Teleport |
| Esc / P | Pause |
| F11 / Alt+Enter | Toggle fullscreen |
| K | Quit |

Keys are scancode-based — physical position matters, not label. ZQSD (AZERTY) and WASD (QWERTY) both work out of the box.

## History

This started as an OCaml assignment. The original OCaml version is preserved on the [`ocaml`](../../tree/ocaml) branch. This branch (`master`) is the Rust rewrite targeting modern GPU rendering.
