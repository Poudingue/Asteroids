# Asteroids Rust/wgpu Port - Design Document

## Goal

Port the OCaml Asteroids game (~2,200 LOC) to Rust with wgpu (Vulkan backend) + SDL2. The port must be **visually indistinguishable** from the original — same physics, same visuals, same gameplay feel.

## Tech Stack

- **Language**: Rust (2021 edition)
- **GPU**: wgpu (uses Vulkan backend on Windows/Linux)
- **Windowing/Input**: SDL2 via `sdl2` crate
- **Math**: Custom Vec2/f64 (no external math crate needed — the math is simple)

## Module Mapping

| OCaml Module | Rust Module | Description |
|---|---|---|
| `parameters.ml` | `parameters.rs` | Constants + `Globals` struct for mutable state |
| `functions.ml` | `math_utils.rs` | Vec2 ops, polar/affine, modulo, dithering |
| `colors.ml` | `color.rs` | `HdrColor` struct, spectral redistribution, exposure |
| `objects.ml` | `objects.rs` | `Entity`, `Star`, `Hitbox`, `Visuals`, spawn functions |
| `buttons.ml` | `buttons.rs` | `ButtonBoolean`, `SliderFloat`, pause menu |
| `hud.ml` | (empty, inlined) | — |
| `asteroids.ml` | `game.rs` + `main.rs` | `GameState`, physics, collisions, rendering, loop |
| (new) | `renderer.rs` | `Renderer2D` — wgpu batched 2D drawing |
| (new) | `shaders/shape.wgsl` | Vertex/fragment shader for filled shapes |

## Architecture

### 1. Renderer2D

A struct owning all wgpu state. Provides an immediate-mode API that matches the OCaml `Graphics` calls:

```rust
impl Renderer2D {
    fn fill_circle(&mut self, center: (f64, f64), radius: f64, color: [u8; 4]);
    fn fill_poly(&mut self, points: &[(i32, i32)], color: [u8; 4]);
    fn draw_poly(&mut self, points: &[(i32, i32)], color: [u8; 4], line_width: u32);
    fn draw_line(&mut self, p1: (i32, i32), p2: (i32, i32), color: [u8; 4], width: u32);
    fn plot(&mut self, pos: (i32, i32), color: [u8; 4]);
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]);
    fn fill_ellipse(&mut self, center: (i32, i32), rx: i32, ry: i32, color: [u8; 4]);
    fn draw_string(&mut self, text: &str, pos: (i32, i32), color: [u8; 4]);
    fn present(&mut self);  // flush batch, submit to GPU, present
}
```

Internally, all shapes are tessellated into triangles and batched into a single vertex buffer (position + color per vertex). One draw call per frame. Circles use triangle fans (~32 segments). Polygons use ear-clipping or fan triangulation. Lines with width are expanded into quads.

The shader is trivial: pass-through vertex positions (in screen-space pixels, converted to NDC in shader), flat color interpolation.

### 2. Globals

All OCaml mutable `ref` values become fields on a `Globals` struct:

```rust
struct Globals {
    // Time
    game_speed: f64,
    game_speed_target: f64,
    observer_proper_time: f64,
    time_last_frame: f64,
    time_current_frame: f64,
    pause: bool,
    restart: bool,
    quit: bool,

    // Visual effects
    game_exposure: f64,
    game_exposure_target: f64,
    game_screenshake: f64,
    game_screenshake_pos: Vec2,
    game_screenshake_previous_pos: Vec2,
    add_color: HdrColor,
    mul_color: HdrColor,
    mul_base: HdrColor,
    space_color: HdrColor,
    // ... etc

    // Settings (toggleable in pause menu)
    screenshake_enabled: bool,
    smoke_enabled: bool,
    chunks_enabled: bool,
    flashes_enabled: bool,
    scanlines_enabled: bool,
    retro: bool,
    motion_blur: bool,
    advanced_hitbox: bool,
    dyn_color: bool,
    variable_exposure: bool,

    // Dithering
    current_jitter: Vec2,
    current_jitter_coll_table: Vec2,

    // Rendering
    ratio_rendu: f64,
    phys_width: f64,
    phys_height: f64,
}
```

### 3. Entity (maps `objet_physique`)

```rust
#[derive(Clone)]
enum EntityKind {
    Asteroid, Projectile, Ship, Explosion, Smoke, Spark,
    Shotgun, Sniper, Machinegun,
}

#[derive(Clone)]
struct Polygon(Vec<(f64, f64)>);  // (angle, distance) pairs

#[derive(Clone)]
struct Hitbox {
    ext_radius: f64,
    int_radius: f64,
    points: Polygon,
}

#[derive(Clone)]
struct Visuals {
    color: HdrColor,
    radius: f64,
    shapes: Vec<(HdrColor, Polygon)>,
}

#[derive(Clone)]
struct Entity {
    kind: EntityKind,
    hitbox: Hitbox,
    visuals: Visuals,
    mass: f64,
    health: f64,
    max_health: f64,
    dam_ratio: f64,
    dam_res: f64,
    phys_ratio: f64,
    phys_res: f64,
    position: Vec2,
    velocity: Vec2,
    orientation: f64,
    moment: f64,
    proper_time: f64,
    hdr_exposure: f64,
}
```

### 4. GameState (maps `etat`)

```rust
struct GameState {
    // Pause menu
    buttons: Vec<ButtonBoolean>,

    // Score/progression
    score: i32,
    lives: i32,
    stage: i32,
    cooldown: f64,
    cooldown_tp: f64,
    last_health: f64,

    // Ship
    ship: Entity,

    // Entity lists (same categorization as OCaml for culling)
    objects: Vec<Entity>,          // on-screen asteroids
    objects_oos: Vec<Entity>,      // off-screen asteroids
    toosmall: Vec<Entity>,         // small asteroids on-screen
    toosmall_oos: Vec<Entity>,     // small asteroids off-screen
    fragments: Vec<Entity>,        // freshly fragmented, need inter-collision
    chunks: Vec<Entity>,           // tiny debris (visual only)
    chunks_oos: Vec<Entity>,
    chunks_explo: Vec<Entity>,     // teleport explosion chunks
    projectiles: Vec<Entity>,
    explosions: Vec<Entity>,
    smoke: Vec<Entity>,
    smoke_oos: Vec<Entity>,
    sparks: Vec<Entity>,
    stars: Vec<Star>,

    // Collision grid
    collision_table: Vec<Vec<usize>>,  // 15*9 cells, indices into combined entity vec
    collision_table_toosmall: Vec<Vec<usize>>,
    collision_table_other: Vec<Vec<usize>>,
    collision_table_frag: Vec<Vec<usize>>,
}
```

### 5. Game Loop

The OCaml recursive loop (`boucle_interaction`) becomes an imperative `while` loop:

```rust
fn main() {
    // SDL2 + wgpu init
    // GameState init

    'game_loop: loop {
        // 1. Handle SDL2 events (keyboard, mouse, window)
        // 2. If quit -> break
        // 3. If restart -> reinit state
        // 4. Update time
        // 5. Process input (mouse aim, key actions)
        // 6. If not paused: physics step (inertie, moment, collisions, spawning, despawning)
        // 7. Camera update
        // 8. Render frame (clear, stars, smoke, chunks, projectiles, ship, objects, explosions, HUD)
        // 9. Present
    }
}
```

### 6. Input Mapping

The original uses OCaml's `key_pressed()` + `wait_next_event` which is blocking. In Rust/SDL2, we use the event pump (non-blocking). To match the original feel:
- Track key states (pressed this frame) via SDL2 event polling
- Mouse position from SDL2
- Same key bindings: Z=forward, Q=left, D=right, A=strafe-left, E=strafe-right, Space=fire, F=teleport, P=pause, R=restart, K=quit
- Left mouse button = accelerate (same as original `controle_souris`)

### 7. Physics Fidelity

All physics formulas are ported verbatim:
- `exp_decay(n, half_life, proper_time)` — same formula
- `abso_exp_decay(n, half_life)` — same formula
- Collision: same circle-circle + polygon-point checks
- Same spatial grid (15x9), same `rev_filtertable` logic
- Same elastic collision with mass-weighted velocity averaging
- Same damage model (dam_ratio, dam_res, phys_ratio, phys_res)
- Same modulo wrapping (3x screen size)

### 8. Visual Fidelity

- **HDR colors**: Same `HdrColor` struct with r/v/b as f64, same spectral redistribution (`redirect_spectre_wide`)
- **Exposure**: Same `game_exposure`, `variable_exposure`, `exposure_half_life`
- **Flashes**: Same additive color system (`add_color`)
- **Screenshake**: Same smooth screenshake with half-life decay
- **Dithering**: Same jitter AA with `dither_power` and `current_jitter_double`
- **Motion blur**: Same light trail rendering (multiple concentric lines with decreasing intensity)
- **Scanlines**: Same horizontal black lines at `scanlines_period` interval
- **Stars**: Same parallax (proximity), same twinkle (random lum), same trail rendering
- **Vector font**: Same `shape_char` polygon definitions for score/HUD text
- **Hearts**: Same `draw_heart` with ellipses + triangle fill
- **Retro mode**: Same white-on-black wireframe toggle

## Implementation Order

1. **Scaffold**: Cargo.toml, SDL2 window, wgpu init, basic clear-to-black
2. **Renderer2D**: fill_circle, fill_poly, fill_rect, draw_line — enough to see something
3. **Math + Color**: Vec2, HdrColor, all utility functions
4. **Parameters + Globals**: Port all constants and state
5. **Objects**: Entity types, spawn functions
6. **Game loop (minimal)**: Ship rendering + mouse aim + movement
7. **Asteroids**: Spawning, polygon rendering, fragmentation
8. **Collisions**: Grid partitioning, collision detection, consequences
9. **Projectiles + Explosions**: Firing, impact, explosion chains
10. **VFX**: Smoke, chunks, light trails, screenshake, exposure, flashes, dithering
11. **HUD**: Health bar, cooldowns, score, hearts, framerate, debug stats
12. **Pause menu**: Buttons, title screen
13. **Polish**: Scanlines, retro mode, star parallax, camera prediction, stage colors
14. **Tuning**: Match all parameter values, verify visual fidelity

## Dependencies (Cargo.toml)

```toml
[dependencies]
wgpu = "24"
sdl2 = { version = "0.37", features = ["raw-window-handle"] }
raw-window-handle = "0.6"
pollster = "0.4"       # async runtime for wgpu
bytemuck = { version = "1", features = ["derive"] }  # vertex data casting
rand = "0.8"
```
