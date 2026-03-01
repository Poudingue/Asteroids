# Asteroids Rust/wgpu Port - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Port the OCaml Asteroids game to Rust with wgpu (Vulkan backend) + SDL2, visually indistinguishable from the original.

**Architecture:** Custom 2D renderer on wgpu batching all shapes (circles, polygons, lines) into a single vertex buffer per frame. Game state is an imperative `GameState` struct with `Entity` objects in categorized `Vec`s. SDL2 handles windowing and input.

**Tech Stack:** Rust 2021, wgpu, SDL2, bytemuck, rand, pollster

**Design doc:** `docs/plans/2026-03-01-rust-vulkan-port-design.md`

**Original source:** `ml/` directory (OCaml)

---

## Task 1: Project Scaffold — SDL2 Window + wgpu Clear-to-Black

**Files:**
- Create: `rs/Cargo.toml`
- Create: `rs/src/main.rs`
- Create: `rs/build.rs` (optional, for SDL2)

**Goal:** Open a 1920x1080 window, initialize wgpu with Vulkan backend, clear to black each frame.

**Step 1: Create `rs/Cargo.toml`**

```toml
[package]
name = "asteroids"
version = "0.1.0"
edition = "2021"

[dependencies]
wgpu = "24"
sdl2 = { version = "0.37", features = ["raw-window-handle"] }
raw-window-handle = "0.6"
pollster = "0.4"
bytemuck = { version = "1", features = ["derive"] }
rand = "0.8"
```

**Step 2: Create `rs/src/main.rs`**

Minimal SDL2 + wgpu setup:
1. `sdl2::init()` with video subsystem
2. Create 1920x1080 window
3. Get raw window handle via `raw-window-handle` traits
4. Create wgpu `Instance` (Vulkan backend), `Surface`, `Adapter`, `Device`, `Queue`
5. Configure surface with `Bgra8UnormSrgb` format
6. Main loop: poll SDL2 events, get surface texture, create render pass clearing to black, submit, present
7. Quit on window close or ESC

**Step 3: Build and run**

```bash
cd rs && cargo run
```

Expected: A black 1920x1080 window appears. Pressing ESC or closing the window exits cleanly.

**Step 4: Commit**

```bash
git add rs/
git commit -m "feat(rs): scaffold SDL2 window + wgpu clear-to-black"
```

---

## Task 2: Renderer2D — Batched 2D Shape Drawing

**Files:**
- Create: `rs/src/renderer.rs`
- Create: `rs/src/shaders/shape.wgsl`
- Modify: `rs/src/main.rs` — use Renderer2D

**Goal:** A `Renderer2D` struct that accepts immediate-mode draw calls and flushes them as a single batched draw call per frame.

**Step 1: Create the WGSL shader `rs/src/shaders/shape.wgsl`**

```wgsl
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Convert pixel coords to NDC: x: [0, width] -> [-1, 1], y: [0, height] -> [-1, 1]
    out.position = vec4<f32>(
        (in.position.x / screen_size.x) * 2.0 - 1.0,
        (in.position.y / screen_size.y) * 2.0 - 1.0,
        0.0,
        1.0
    );
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
```

**Step 2: Create `rs/src/renderer.rs`**

Core structure:
```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

pub struct Renderer2D {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    pipeline: wgpu::RenderPipeline,
    screen_size_buffer: wgpu::Buffer,
    screen_size_bind_group: wgpu::BindGroup,
    vertices: Vec<Vertex>,  // CPU-side batch
    width: u32,
    height: u32,
}
```

Methods to implement:
- `new(device, queue, surface, config, width, height)` — create pipeline, bind group
- `fill_rect(x, y, w, h, color)` — push 6 vertices (2 triangles)
- `fill_circle(cx, cy, radius, color)` — push triangle fan (~32 segments)
- `fill_poly(points, color)` — fan triangulation from first vertex (works for convex polys; the OCaml code uses convex polygons)
- `draw_poly(points, color, line_width)` — expand each edge into a quad
- `draw_line(x1, y1, x2, y2, color, width)` — perpendicular extrusion into quad
- `plot(x, y, color)` — 1px rect
- `fill_ellipse(cx, cy, rx, ry, color)` — scaled circle tessellation
- `begin_frame()` — clear vertex batch
- `end_frame(surface_texture)` — create vertex buffer from batch, render pass, draw, present

**Step 3: Wire into `main.rs`**

Replace the manual render pass with `Renderer2D`. Draw a test: white rectangle, a red circle, a green polygon. Verify visually.

**Step 4: Build and run**

```bash
cd rs && cargo run
```

Expected: Test shapes visible on black background.

**Step 5: Commit**

```bash
git commit -m "feat(rs): Renderer2D with batched 2D shape drawing"
```

---

## Task 3: Math Utilities + HDR Color System

**Files:**
- Create: `rs/src/math_utils.rs`
- Create: `rs/src/color.rs`
- Modify: `rs/src/main.rs` — add `mod` declarations

**Goal:** Port `functions.ml` and `colors.ml` verbatim.

**Step 1: Create `rs/src/math_utils.rs`**

Port every function from `ml/functions.ml`:

```rust
pub type Vec2 = (f64, f64);

pub const PI: f64 = std::f64::consts::PI;

pub fn randfloat(min: f64, max: f64) -> f64;
pub fn carre(v: f64) -> f64;
pub fn exp_decay(n: f64, half_life: f64, proper_time: f64, game_speed: f64, observer_proper_time: f64, dt: f64) -> f64;
pub fn abso_exp_decay(n: f64, half_life: f64, dt: f64) -> f64;
pub fn hypothenuse(v: Vec2) -> f64;
pub fn addtuple(a: Vec2, b: Vec2) -> Vec2;
pub fn soustuple(a: Vec2, b: Vec2) -> Vec2;
pub fn multuple(v: Vec2, ratio: f64) -> Vec2;
pub fn moyfloat(val1: f64, val2: f64, ratio: f64) -> f64;
pub fn moytuple(a: Vec2, b: Vec2, ratio: f64) -> Vec2;
pub fn multuple_parallel(a: Vec2, b: Vec2) -> Vec2;
pub fn entretuple(p: Vec2, p1: Vec2, p2: Vec2) -> bool;
pub fn inttuple(v: Vec2) -> (i32, i32);
pub fn floattuple(v: (i32, i32)) -> Vec2;
pub fn polar_to_affine(angle: f64, valeur: f64) -> Vec2;
pub fn affine_to_polar(v: Vec2) -> (f64, f64);
pub fn distancecarre(a: Vec2, b: Vec2) -> f64;
pub fn modulo_float(value: f64, modulo: f64) -> f64;
pub fn modulo_reso(v: Vec2, phys_w: f64, phys_h: f64) -> Vec2;
pub fn modulo_3reso(v: Vec2, phys_w: f64, phys_h: f64) -> Vec2;
```

Note: `exp_decay` and `abso_exp_decay` in OCaml reference global refs (`observer_proper_time`, `game_speed`, `time_last_frame`, `time_current_frame`). In Rust, pass these as parameters.

**Step 2: Create `rs/src/color.rs`**

Port `ml/colors.ml`:

```rust
#[derive(Clone, Copy)]
pub struct HdrColor {
    pub r: f64,
    pub v: f64,  // 'v' not 'g' — match original naming
    pub b: f64,
}

impl HdrColor {
    pub fn new(r: f64, v: f64, b: f64) -> Self;
    pub fn add(self, other: Self) -> Self;
    pub fn sub(self, other: Self) -> Self;
    pub fn mul(self, other: Self) -> Self;
    pub fn intensify(self, i: f64) -> Self;
    pub fn saturate(self, i: f64) -> Self;
    pub fn redirect_spectre_wide(self) -> Self;
    pub fn to_rgb(self, add_color: &Self, mul_color: &Self, game_exposure: f64) -> [u8; 4];
    pub fn half_color(col1: Self, col2: Self, half_life: f64, dt: f64) -> Self;
}
```

`to_rgb` replaces `rgb_of_hdr` — applies add_color, mul_color, game_exposure, spectral redistribution, clamps to 0-255.

**Step 3: Dithering functions**

Add to `math_utils.rs` (or a separate section):
```rust
pub fn dither(fl: f64, dither_aa: bool, dither_power: f64) -> i32;
pub fn dither_radius(fl: f64, dither_aa: bool, dither_power_radius: f64) -> i32;
pub fn dither_tuple(v: Vec2, dither_aa: bool, jitter: Vec2) -> (i32, i32);
```

**Step 4: Commit**

```bash
git commit -m "feat(rs): port math utilities and HDR color system"
```

---

## Task 4: Parameters + Globals

**Files:**
- Create: `rs/src/parameters.rs`
- Modify: `rs/src/main.rs` — add `mod`

**Goal:** Port all constants from `ml/parameters.ml` and create the `Globals` struct for mutable state.

**Step 1: Port constants**

Every `let` binding in `parameters.ml` that is NOT a `ref` becomes a `pub const` or `pub static` in Rust. Group them exactly as in the original (display, time, asteroids, ship, projectile, weapons, explosions, camera, screenshake, etc.).

**Step 2: Create `Globals` struct**

Every `let ... = ref ...` in `parameters.ml` becomes a field on `Globals`:

```rust
pub struct Globals {
    pub pause: bool,
    pub restart: bool,
    pub quit: bool,
    pub game_speed: f64,
    pub game_speed_target: f64,
    pub game_exposure: f64,
    pub game_exposure_target: f64,
    pub observer_proper_time: f64,
    pub time_last_frame: f64,
    pub time_current_frame: f64,
    // ... all other ref values
}

impl Globals {
    pub fn new() -> Self { /* defaults matching OCaml */ }
    pub fn dt(&self) -> f64 { self.time_current_frame - self.time_last_frame }
}
```

**Step 3: Commit**

```bash
git commit -m "feat(rs): port all parameters and globals"
```

---

## Task 5: Entity Types + Spawn Functions

**Files:**
- Create: `rs/src/objects.rs`
- Modify: `rs/src/main.rs` — add `mod`

**Goal:** Port `ml/objects.ml` — all entity types and spawn functions.

**Step 1: Define types**

```rust
pub enum EntityKind { Asteroid, Projectile, Ship, Explosion, Smoke, Spark, Shotgun, Sniper, Machinegun }
pub struct Polygon(pub Vec<(f64, f64)>);
pub struct Hitbox { pub ext_radius: f64, pub int_radius: f64, pub points: Polygon }
pub struct Visuals { pub color: HdrColor, pub radius: f64, pub shapes: Vec<(HdrColor, Polygon)> }
pub struct Entity { /* all fields from design doc */ }
pub struct Star { pub last_pos: Vec2, pub pos: Vec2, pub proximity: f64, pub lum: f64 }
```

**Step 2: Port spawn functions**

All spawn functions from `objects.ml`:
- `spawn_ship()` — with hardcoded polygon shapes matching `visuals_ship` and `hitbox_ship`
- `spawn_projectile(pos, vel, proper_time)`
- `spawn_n_projectiles(ship, n, globals)` — returns `Vec<Entity>`
- `spawn_explosion(projectile)`, `spawn_explosion_object(obj)`, `spawn_explosion_death(ship, elapsed)`
- `spawn_explosion_chunk(obj)`
- `spawn_muzzle(projectile)`, `spawn_fire(ship)`
- `spawn_chunk_explo(pos, vel, color, proper_time)`, `spawn_n_chunks(ship, n, color)`
- `polygon_asteroid(radius, n)` — recursive polygon generation
- `spawn_asteroid(pos, vel, radius)`
- `random_out_of_screen(radius, phys_w, phys_h)`
- `spawn_random_star(phys_w, phys_h)`, `n_stars(n, phys_w, phys_h)`

Note: OCaml spawn functions reference global refs (`flashes`, `add_color`, `game_exposure`, etc.). In Rust, pass `&mut Globals` to those that need it.

**Step 3: Commit**

```bash
git commit -m "feat(rs): port entity types and all spawn functions"
```

---

## Task 6: Minimal Game Loop — Ship on Screen

**Files:**
- Create: `rs/src/game.rs`
- Modify: `rs/src/main.rs` — full game loop

**Goal:** Render the ship, aim with mouse, move with keyboard. Stars in background.

**Step 1: Create `GameState`**

```rust
pub struct GameState {
    pub score: i32,
    pub lives: i32,
    pub stage: i32,
    pub cooldown: f64,
    pub cooldown_tp: f64,
    pub last_health: f64,
    pub ship: Entity,
    pub objects: Vec<Entity>,
    pub objects_oos: Vec<Entity>,
    // ... all entity lists
    pub stars: Vec<Star>,
}

impl GameState {
    pub fn new(globals: &Globals) -> Self;
}
```

**Step 2: Implement basic game loop in `main.rs`**

```
loop {
    poll SDL2 events -> update key states, mouse pos
    update time (globals.time_last_frame, time_current_frame)
    aim ship at mouse (orientation = atan2)
    handle keys (Z=accel, Q/D=rotate, A/E=strafe)
    apply inertia to ship
    apply moment to ship
    render: clear background, render stars, render ship
    present
}
```

**Step 3: Port rendering functions**

From `asteroids.ml`:
- `render_visuals(entity, offset, renderer, globals)` — fill_circle + render_shapes
- `render_shapes(shapes, pos, rotat, expos, renderer, globals)` — iterate polygon shapes
- `render_poly(poly, pos, rotat, color, renderer, globals)` — polygon rotation + translation + dithering
- `poly_to_affine(poly, rotat, scale)` — convert polar polygon to screen coords
- `render_star_trail(star, renderer, globals)` — star rendering with parallax

**Step 4: Port movement functions**

From `asteroids.ml`:
- `inertie_objet(entity, globals)` — position += velocity * dt * game_speed
- `moment_objet(entity, globals)` — orientation += moment * dt * game_speed
- `deplac_objet(entity, velocity, globals)` — position displacement
- `accel_objet(entity, accel, globals)` — velocity += accel * dt
- `boost_objet(entity, boost)` — instant velocity change
- `rotat_objet(entity, rotation, globals)` — timed rotation
- `couple_objet(entity, momentum, globals)` — angular acceleration
- Mouse control: `controle_souris` equivalent — aim ship at mouse position

**Step 5: Build and run**

Expected: Ship visible on starry background. Mouse aims ship. AZQD moves ship. Ship has inertia.

**Step 6: Commit**

```bash
git commit -m "feat(rs): minimal game loop with ship movement and star rendering"
```

---

## Task 7: Asteroids — Spawning, Rendering, Fragmentation

**Files:**
- Modify: `rs/src/game.rs`
- Modify: `rs/src/objects.rs` (if needed)

**Goal:** Asteroids spawn in waves, render as random polygons, fragment when destroyed.

**Step 1: Asteroid spawning**

Port `spawn_random_asteroid(stage)` and the wave system from `etat_suivant`:
- `time_since_last_spawn` timer
- `current_stage_asteroids` counter
- Stage progression with color changes
- Spawn out-of-screen, transfer to on-screen when entering view

**Step 2: Asteroid rendering**

Already handled by `render_visuals` (fill_circle base + polygon shapes on top).

**Step 3: Fragmentation**

Port `frag_asteroid(ref_asteroid)`:
- Create fragment with smaller radius, random shape, inherited color
- Reposition, new velocity with random scatter
- `spawn_n_frags(source_list, dest_list, n)` — spawn fragments from dead asteroids

**Step 4: Object culling / screen management**

Port the OOS (out-of-screen) transfer logic:
- `checkspawn_objet` / `checknotspawn_objet` — boundary checks
- Transfer between `objects` <-> `objects_oos`, `toosmall` <-> `toosmall_oos`
- `modulo_3reso` wrapping for objects
- `despawn` — remove dead, negative-radius, too-far objects

**Step 5: Build and run**

Expected: Asteroids appear from off-screen, drift across, wrap around. They have random polygon shapes. (No collisions yet — they pass through everything.)

**Step 6: Commit**

```bash
git commit -m "feat(rs): asteroid spawning, rendering, fragmentation, and culling"
```

---

## Task 8: Collision System

**Files:**
- Modify: `rs/src/game.rs`

**Goal:** Port the spatial grid collision system from `asteroids.ml`.

**Step 1: Collision detection functions**

```rust
fn collision_circles(pos0: Vec2, r0: f64, pos1: Vec2, r1: f64) -> bool;
fn collision_point(pos_point: Vec2, pos_circle: Vec2, radius: f64) -> bool;
fn collisions_points(points: &[Vec2], pos_circle: Vec2, radius: f64) -> bool;
fn collision_poly(pos: Vec2, poly: &Polygon, rotat: f64, circle_pos: Vec2, radius: f64) -> bool;
fn collision(obj1: &Entity, obj2: &Entity, precise: bool, advanced_hitbox: bool) -> bool;
```

**Step 2: Spatial grid**

Port `rev_filtertable` — insert entities into grid cells based on position:
```rust
const GRID_W: usize = 15;
const GRID_H: usize = 9;
type CollisionGrid = Vec<Vec<usize>>;  // GRID_W * GRID_H cells

fn clear_grid(grid: &mut CollisionGrid);
fn insert_into_grid(entities: &[Entity], grid: &mut CollisionGrid, globals: &Globals);
fn calculate_collision_tables(grid1: &CollisionGrid, grid2: &CollisionGrid, entities: &mut Vec<Entity>, extend: bool, ...);
```

**Step 3: Collision consequences**

Port `consequences_collision(obj1, obj2)`:
- Explosion vs object: apply damage
- Projectile vs anything: destroy projectile
- Object vs object: elastic bounce (mass-weighted velocity averaging, repulsion, physical damage)

Port `consequences_collision_frags(frag1, frag2)` for fragment-vs-fragment repulsion.

**Step 4: Wire into game loop**

In the physics step:
1. Clear all 4 grids
2. Insert entities into appropriate grids
3. Run `calculate_collision_tables` for all grid pairs (matching OCaml's 6 calls)
4. Apply fragment collisions separately

**Step 5: Build and run**

Expected: Ship bounces off asteroids. Asteroids bounce off each other. Physical damage applied.

**Step 6: Commit**

```bash
git commit -m "feat(rs): spatial grid collision system with elastic bounces"
```

---

## Task 9: Projectiles + Explosions

**Files:**
- Modify: `rs/src/game.rs`

**Goal:** Shooting, projectile rendering with light trails, explosions on impact.

**Step 1: Firing system**

Port `tir(ref_etat)`:
- Cooldown system
- `spawn_n_projectiles` on fire
- Recoil on ship
- Screenshake, flash, exposure effects on fire

**Step 2: Projectile rendering**

Port `render_projectile`:
- Multiple concentric light trails with decreasing intensity
- `render_light_trail(radius, pos, velocity, color, proper_time, renderer, globals)`

**Step 3: Explosions**

- `spawn_explosion` on projectile death
- `spawn_explosion_object` on asteroid death
- `spawn_explosion_death` on ship death (continuous while dead)
- Explosions act as damage sources in collision system

**Step 4: Weapon switching**

Port weapon parameters (shotgun, sniper, machinegun) — the globals for projectile stats can be swapped. (The OCaml code has these as separate constants but the switching mechanism is in the parameters.)

**Step 5: Build and run**

Expected: Space fires projectiles. Projectiles have light trails. Hitting asteroids causes explosions and fragmentation. Asteroids break into pieces.

**Step 6: Commit**

```bash
git commit -m "feat(rs): projectiles with light trails, explosions, and impact effects"
```

---

## Task 10: Visual Effects

**Files:**
- Modify: `rs/src/game.rs`
- Modify: `rs/src/renderer.rs` (if needed for line width)

**Goal:** Port all VFX systems to match the original look.

**Step 1: Smoke particles**

Port `decay_smoke`, `decay_chunk`, `decay_chunk_explo`:
- Exponential radius decay
- Exposure decay
- Smoke inherits from explosions
- Muzzle flash on fire, engine fire on thrust

**Step 2: Chunks**

Port chunk rendering (`render_chunk`):
- Small circle with entity color
- Chunk creation from dead small fragments

**Step 3: Screenshake**

Port screenshake system:
- `game_screenshake` half-life decay
- Random position offset each frame
- Optional smooth mode (averaging with previous)
- Screenshake on fire, damage, explosions, death

**Step 4: Exposure + Flashes**

Port variable exposure:
- `game_exposure` with half-life toward target
- `add_color` additive flash with fast decay
- Flash on fire, explosion, damage, teleport, death

**Step 5: Dithering/Jitter AA**

Port dithering:
- `current_jitter_double` random offset applied to all positions
- `dither()`, `dither_radius()`, `dither_tuple()` functions
- Applied in `render_visuals`, `render_poly`, etc.

**Step 6: Build and run**

Expected: Smoke trails behind explosions. Screen shakes on impacts. Light flashes on events. Stars twinkle. Subtle dithering visible on edges.

**Step 7: Commit**

```bash
git commit -m "feat(rs): smoke, screenshake, exposure, flashes, dithering"
```

---

## Task 11: HUD — Health, Score, Cooldowns

**Files:**
- Modify: `rs/src/game.rs`

**Goal:** Port the full HUD from `affiche_hud` in `asteroids.ml`.

**Step 1: Vector font**

Port `shape_char` — the complete character->polygon map for 0-9, A-Z, space:
```rust
fn shape_char(c: char) -> Vec<(f64, f64)>;
fn render_char(encadrement: [(f64,f64); 4], c: char, renderer: &mut Renderer2D, globals: &Globals);
fn render_string(s: &str, pos: Vec2, l_char: f64, h_char: f64, l_space: f64, shake: f64, renderer: &mut Renderer2D, globals: &Globals);
```

Port `displacement` and `displace_shape` for mapping characters into bounding quads.

**Step 2: Health bar**

Port `affiche_barre(ratio, quad_points, color)`:
- Quadrilateral bar that fills based on ratio
- Health bar with red/orange/delayed layers
- Teleport cooldown bar (blue)
- Weapon cooldown bar (yellow)

**Step 3: Hearts**

Port `draw_heart` and `draw_n_hearts`:
- Two ellipses + triangle for heart shape
- Rendered for each remaining life

**Step 4: Score and stage display**

- Score with shake effect (`shake_score`)
- Stage number
- Death countdown timer

**Step 5: Debug stats**

- Object counts, collision stats, framerate
- `draw_string` for debug text (use SDL2 text or simple bitmap — the OCaml version uses `Graphics.draw_string`)

**Step 6: Build and run**

Expected: Full HUD visible — health bar, hearts, score, stage, cooldowns, framerate.

**Step 7: Commit**

```bash
git commit -m "feat(rs): complete HUD with vector font, health bars, score"
```

---

## Task 12: Pause Menu + Buttons

**Files:**
- Create: `rs/src/buttons.rs`
- Modify: `rs/src/game.rs`

**Goal:** Port the pause menu with toggle buttons.

**Step 1: Button types**

Port `ButtonBoolean` from `buttons.ml`:
```rust
pub struct ButtonBoolean {
    pub pos1: Vec2,
    pub pos2: Vec2,
    pub text: String,
    pub text_over: String,
    pub value: bool,
    pub last_mouse_state: bool,
}
```

**Step 2: Button rendering + interaction**

Port `applique_button`:
- Draw filled rect (green=true, red=false) or white/black in retro mode
- Frame border
- Text centered in button
- Tooltip on hover
- Toggle on click (with edge detection via `last_mouse_state`)

**Step 3: Create all buttons**

Port all 11 buttons from `buttons.ml`:
- New Game, Resume, Quit
- Retro Visuals, Scanlines
- Advanced Hitbox, Smoke Particles, Screenshake
- Light Flashes, Chunk Particles, Color Effects

**Step 4: Pause screen**

Port from `affiche_hud`:
- "ASTEROIDS" title (vector font, black shadow + white)
- Display buttons
- P key toggles pause

**Step 5: Wire button values to Globals**

Each button toggles a field in `Globals` (smoke_enabled, screenshake_enabled, etc.).

**Step 6: Build and run**

Expected: P pauses game. Pause screen shows title + all option buttons. Clicking buttons toggles options. Resume/New Game/Quit work.

**Step 7: Commit**

```bash
git commit -m "feat(rs): pause menu with toggle buttons and options"
```

---

## Task 13: Polish — Camera, Stars, Scanlines, Retro, Teleport

**Files:**
- Modify: `rs/src/game.rs`

**Goal:** All remaining features for full parity.

**Step 1: Predictive camera**

Port camera system from `affiche_etat`:
- Camera target = ship position + velocity prediction + center-of-attention of objects + look-ahead direction
- Exponential smoothing toward target (`camera_half_depl`)
- Boundary clamping (`camera_start_bound`, `camera_max_force`)
- Move all objects by camera delta each frame

**Step 2: Star parallax**

Port `deplac_star` and `deplac_stars`:
- Stars move with camera multiplied by `proximity` factor
- Modulo wrapping at screen edges
- Reset `last_pos` on teleport to avoid false trails

**Step 3: Teleportation**

Port `teleport(ref_etat)`:
- F key teleports ship to mouse position
- Cooldown system (`cooldown_tp`)
- Chunk explosion effect at destination
- Flash + exposure effects

**Step 4: Death + respawn**

Port `mort(ref_etat)` and death handling in `boucle_interaction`:
- Continuous explosions while dead
- Countdown timer
- Respawn after timeout or health threshold
- Life loss, game over -> restart with pause
- Slow-mo on death

**Step 5: Scanlines**

Port `render_scanlines`:
- Horizontal black lines every `scanlines_period` pixels
- Optional animation (alternating offset)

**Step 6: Retro mode**

When `retro=true`:
- Background is pure black
- All objects rendered as white wireframes (`draw_poly` instead of `fill_poly`)
- No fill_circle for objects
- Simplified rendering

**Step 7: Stage color system**

Port dynamic color per stage:
- Random `mul_base` color per stage
- `space_color_goal` and `star_color_goal` change
- Smooth interpolation via `half_color`

**Step 8: Build and run**

Expected: Camera smoothly follows ship. Stars have parallax depth. Teleport works with cooldown. Death/respawn cycle works. Scanlines toggle works. Retro mode works.

**Step 9: Commit**

```bash
git commit -m "feat(rs): camera, teleport, death, scanlines, retro mode, stage colors"
```

---

## Task 14: Final Tuning + Verification

**Files:**
- All `rs/src/*.rs`

**Goal:** Match every parameter value, verify visual + gameplay fidelity against the OCaml original.

**Step 1: Parameter audit**

Go through `ml/parameters.ml` line by line. Verify every constant in `rs/src/parameters.rs` matches exactly.

**Step 2: Physics verification**

- Ship acceleration, rotation, boost values
- Asteroid spawn rates, velocities, sizes
- Collision behavior, damage values
- Cooldowns, recoil values

**Step 3: Visual verification**

Run both versions side-by-side:
- Ship polygon shape matches
- Asteroid polygon generation looks equivalent (random, but same distribution)
- Explosion colors and sizes match
- Smoke decay rate matches
- Star density and brightness match
- HUD layout and sizes match
- Score font matches

**Step 4: Input feel**

- Mouse aim responsiveness
- Key repeat behavior (OCaml's key_pressed is per-event; SDL2 may differ)
- Fire rate matches cooldown values
- Teleport feel matches

**Step 5: Performance**

- Verify 60+ FPS with many asteroids
- Profile if needed, optimize renderer batch size

**Step 6: Final commit**

```bash
git commit -m "feat(rs): final tuning and visual fidelity verification"
```

---

## Summary

| Task | Description | Key Output |
|------|-------------|------------|
| 1 | Scaffold | Black window appears |
| 2 | Renderer2D | Shapes draw on screen |
| 3 | Math + Color | Utility functions ready |
| 4 | Parameters | All constants + Globals |
| 5 | Objects | Entity types + spawners |
| 6 | Game Loop | Ship moves on starfield |
| 7 | Asteroids | Asteroids spawn + fragment |
| 8 | Collisions | Physics collisions work |
| 9 | Projectiles | Shooting + explosions |
| 10 | VFX | Smoke, shake, flashes |
| 11 | HUD | Full interface |
| 12 | Pause Menu | Options + buttons |
| 13 | Polish | Camera, teleport, retro |
| 14 | Tuning | Visual parity verified |
