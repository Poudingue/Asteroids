# Task 7: Asteroids — Spawning, Rendering, Fragmentation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Port the OCaml asteroid system faithfully — asteroids spawn in waves per stage, render as random jagged polygons, wrap around, and fragment when destroyed.

**Architecture:** Add `spawn_random_asteroid` and `frag_asteroid` to `objects.rs`. Add entity movement, OOS transfer, despawn, stage/spawn logic, color interpolation, and asteroid rendering to `game.rs`. Wire into main loop in `main.rs`. All logic mirrors OCaml `etat_suivant` faithfully.

**Tech Stack:** Rust, existing Renderer2D, existing math_utils/color/parameters modules.

---

### Task 1: Add `spawn_random_asteroid` and `observer_proper_time`

**Files:**
- Modify: `rs/src/objects.rs` — add `spawn_random_asteroid`
- Modify: `rs/src/parameters.rs` — add `observer_proper_time` field to Globals

**Step 1: Add `observer_proper_time` to Globals**

In `rs/src/parameters.rs`, add field to Globals struct and initialize it:

```rust
// In Globals struct, near other game state fields:
pub observer_proper_time: f64,

// In Globals::default() / new(), initialize:
observer_proper_time: 1.0,
```

**Step 2: Add `spawn_random_asteroid` to objects.rs**

Matches OCaml logic — combines `random_out_of_screen` + velocity generation + `spawn_asteroid`:

```rust
/// Spawn a random asteroid for the given stage, positioned off-screen
pub fn spawn_random_asteroid(stage: i32, phys_w: f64, phys_h: f64, rng: &mut impl Rng) -> Entity {
    let radius = randfloat(ASTEROID_MIN_SPAWN_RADIUS, ASTEROID_MAX_SPAWN_RADIUS, rng);
    let pos = random_out_of_screen(radius, phys_w, phys_h, rng);
    let vel_angle = rng.gen::<f64>() * 2.0 * PI;
    let vel_magnitude = randfloat(
        ASTEROID_MIN_VELOCITY,
        ASTEROID_MAX_VELOCITY + ASTEROID_STAGE_VELOCITY * stage as f64,
        rng,
    );
    let vel = polar_to_affine(vel_angle, vel_magnitude);
    spawn_asteroid(pos, vel, radius, rng)
}
```

Needs `use crate::math_utils::polar_to_affine;` if not already imported.

**Step 3: Verify**

Run: `cargo check` from `rs/`
Expected: compiles with only pre-existing dead-code warnings.

**Step 4: Commit**

```bash
git add rs/src/objects.rs rs/src/parameters.rs
git commit -m "feat(rs): add spawn_random_asteroid and observer_proper_time"
```

---

### Task 2: Entity Movement — Inertia, Rotation, Recenter

**Files:**
- Modify: `rs/src/game.rs` — add movement functions

**Step 1: Add `inertie_objet` (velocity-based position update)**

Matches OCaml `deplac_objet` + `inertie_objet`. Uses `observer_proper_time / entity.proper_time` for time dilation:

```rust
/// Apply inertial movement: position += velocity * dt (with time dilation)
fn inertie_objet(entity: &mut Entity, globals: &Globals) {
    let dt = (globals.time_current_frame - globals.time_last_frame)
        * globals.game_speed
        * globals.observer_proper_time / entity.proper_time;
    entity.position = addtuple(entity.position, multuple(entity.velocity, dt));
}

/// Apply inertia to all entities in a list
fn inertie_objets(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        inertie_objet(e, globals);
    }
}
```

**Step 2: Add `moment_objet` (rotation update)**

Matches OCaml `moment_objet`:

```rust
/// Apply angular momentum: orientation += moment * dt
fn moment_objet(entity: &mut Entity, globals: &Globals) {
    let dt = (globals.time_current_frame - globals.time_last_frame)
        * globals.game_speed
        * globals.observer_proper_time / entity.proper_time;
    entity.orientation += entity.moment * dt;
}

fn moment_objets(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        moment_objet(e, globals);
    }
}
```

**Step 3: Add `recenter_objet` (wrapping)**

Matches OCaml `recenter_objet` — applies `modulo_3reso`:

```rust
/// Wrap entity position using 3x-resolution modulo (toroidal world)
fn recenter_objet(entity: &mut Entity, globals: &Globals) {
    entity.position = modulo_3reso(entity.position, globals.phys_width, globals.phys_height);
}

fn recenter_objets(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        recenter_objet(e, globals);
    }
}
```

**Step 4: Add `deplac_objet_abso` (instant displacement, for camera)**

```rust
/// Instant absolute displacement (position += offset, no dt scaling)
pub fn deplac_objet_abso(entity: &mut Entity, offset: Vec2) {
    entity.position = addtuple(entity.position, offset);
}
```

**Step 5: Verify**

Run: `cargo check`

**Step 6: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add entity movement — inertia, rotation, recenter"
```

---

### Task 3: OOS Transfer + Entity Predicates

**Files:**
- Modify: `rs/src/game.rs` — add OOS transfer logic and predicates

**Step 1: Add entity predicates**

Matches OCaml `is_alive`, `is_dead`, `ischunk`, `notchunk`, `big_enough`, `too_small`, `positive_radius`:

```rust
fn is_alive(entity: &Entity) -> bool {
    entity.health > 0.0
}

fn is_dead(entity: &Entity) -> bool {
    entity.health <= 0.0
}

fn ischunk(entity: &Entity) -> bool {
    entity.hitbox.int_radius < CHUNK_MAX_SIZE
}

fn notchunk(entity: &Entity) -> bool {
    !ischunk(entity)
}

fn big_enough(entity: &Entity) -> bool {
    entity.hitbox.int_radius >= ASTEROID_MIN_SIZE
}

fn too_small(entity: &Entity) -> bool {
    !big_enough(entity)
}

fn positive_radius(entity: &Entity) -> bool {
    entity.visuals.radius > 0.0
}
```

**Step 2: Add screen-boundary check functions**

Matches OCaml `checkspawn_objet` / `checknotspawn_objet`:

```rust
/// Check if entity is within visible screen area (with radius margin)
fn checkspawn_objet(entity: &Entity, globals: &Globals) -> bool {
    let (x, y) = entity.position;
    let rad = entity.hitbox.ext_radius;
    (x - rad < globals.phys_width) && (x + rad > 0.0)
        && (y - rad < globals.phys_height) && (y + rad > 0.0)
}
```

**Step 3: Add OOS transfer function**

Matches OCaml transfer pattern. Takes two lists (on-screen and off-screen), returns updated pair:

```rust
/// Transfer entities between on-screen and off-screen lists.
/// Entities entering screen move from oos → onscreen.
/// Entities leaving screen move from onscreen → oos.
fn transfer_oos(
    onscreen: &mut Vec<Entity>,
    oos: &mut Vec<Entity>,
    globals: &Globals,
) {
    // Collect entities leaving screen
    let mut going_out: Vec<Entity> = Vec::new();
    let mut staying_in: Vec<Entity> = Vec::new();
    for e in onscreen.drain(..) {
        if checkspawn_objet(&e, globals) {
            staying_in.push(e);
        } else {
            going_out.push(e);
        }
    }

    // Check OOS entities entering screen
    let mut coming_in: Vec<Entity> = Vec::new();
    let mut staying_out: Vec<Entity> = Vec::new();
    for e in oos.drain(..) {
        if checkspawn_objet(&e, globals) {
            coming_in.push(e);
        } else {
            staying_out.push(e);
        }
    }

    // Rebuild lists
    *onscreen = staying_in;
    onscreen.extend(coming_in);
    *oos = staying_out;
    oos.extend(going_out);
}
```

**Step 4: Verify**

Run: `cargo check`

**Step 5: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add OOS transfer logic and entity predicates"
```

---

### Task 4: Fragmentation — `frag_asteroid` + `spawn_n_frags`

**Files:**
- Modify: `rs/src/objects.rs` — add fragmentation functions

**Step 1: Add `frag_asteroid`**

Matches OCaml faithfully — creates a fragment from a parent asteroid:

```rust
/// Create a single fragment from a parent asteroid
pub fn frag_asteroid(parent: &Entity, rng: &mut impl Rng) -> Entity {
    // Start with a fresh asteroid at parent's position/velocity/radius
    let mut fragment = spawn_asteroid(
        parent.position,
        parent.velocity,
        parent.hitbox.int_radius,
        rng,
    );

    let orientation = rng.gen::<f64>() * 2.0 * PI;
    let new_radius = randfloat(FRAGMENT_MIN_SIZE, FRAGMENT_MAX_SIZE, rng)
        * fragment.hitbox.int_radius;

    // Regenerate polygon for new size
    let new_shape = polygon_asteroid(new_radius, rng);

    // Offset position from parent center
    fragment.position = addtuple(
        fragment.position,
        polar_to_affine(orientation, fragment.hitbox.int_radius - new_radius),
    );

    // Update visuals with parent color
    fragment.visuals.radius = new_radius;
    fragment.visuals.color = parent.visuals.color;
    fragment.visuals.shapes = vec![(parent.visuals.color, new_shape.clone())];

    // Update hitbox
    fragment.hitbox.int_radius = new_radius;
    fragment.hitbox.ext_radius = new_radius * ASTEROID_POLYGON_MAX;
    fragment.hitbox.points = new_shape;

    // Recalculate mass and health for new size
    fragment.mass = PI * carre(new_radius) * ASTEROID_DENSITY;
    fragment.health = ASTEROID_MASS_HEALTH * fragment.mass + ASTEROID_MIN_HEALTH;
    fragment.max_health = fragment.health;

    // Add random velocity scatter
    fragment.velocity = addtuple(
        fragment.velocity,
        polar_to_affine(
            orientation,
            randfloat(FRAGMENT_MIN_VELOCITY, FRAGMENT_MAX_VELOCITY, rng),
        ),
    );

    // Adjust HDR exposure randomly
    fragment.hdr_exposure *= randfloat(FRAGMENT_MIN_EXPOSURE, FRAGMENT_MAX_EXPOSURE, rng);

    fragment
}
```

Needs imports: `polar_to_affine`, `addtuple`, `carre` from math_utils.

**Step 2: Add `spawn_n_frags`**

Matches OCaml — spawns N copies of fragments for each dead entity in source:

```rust
/// Spawn fragment_number fragments for each dead entity in source, appending to dest.
/// Returns the updated dest list.
pub fn spawn_n_frags(
    source: &[Entity],
    dest: &mut Vec<Entity>,
    fragment_number: i32,
    rng: &mut impl Rng,
) {
    let dead: Vec<&Entity> = source.iter().filter(|e| e.health <= 0.0).collect();
    for _ in 0..fragment_number {
        for parent in &dead {
            dest.push(frag_asteroid(parent, rng));
        }
    }
}
```

**Step 3: Verify**

Run: `cargo check`

**Step 4: Commit**

```bash
git add rs/src/objects.rs
git commit -m "feat(rs): add asteroid fragmentation (frag_asteroid, spawn_n_frags)"
```

---

### Task 5: Color Interpolation in `update_frame`

**Files:**
- Modify: `rs/src/game.rs` — add color interpolation to `update_frame`

**Step 1: Add color interpolation**

`half_color` already exists in `color.rs`. Add imports and calls in `update_frame`, matching OCaml:

```rust
// In update_frame, inside the `if !globals.pause` block, after screenshake:
if globals.dyn_color {
    let dt = globals.time_current_frame - globals.time_last_frame;
    globals.mul_color = {
        let c = half_color(hdr(globals.mul_color), hdr(globals.mul_base), FILTER_HALF_LIFE, dt);
        (c.r, c.v, c.b)
    };
    globals.space_color = {
        let c = half_color(hdr(globals.space_color), hdr(globals.space_color_goal), SPACE_HALF_LIFE, dt);
        (c.r, c.v, c.b)
    };
    globals.star_color = {
        let c = half_color(hdr(globals.star_color), hdr(globals.star_color_goal), SPACE_HALF_LIFE, dt);
        (c.r, c.v, c.b)
    };
}
```

Add `use crate::color::half_color;` to game.rs imports if not present. Also add `FILTER_HALF_LIFE`, `SPACE_HALF_LIFE` to the `use crate::parameters::*` (already wildcard-imported).

**Step 2: Verify**

Run: `cargo check`

**Step 3: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add color interpolation for star/space/mul colors"
```

---

### Task 6: Spawning, Stage System, Despawn — `update_game`

**Files:**
- Modify: `rs/src/game.rs` — add main `update_game` function

**Step 1: Add `update_game` function**

This is the core game logic function called each frame when not paused. Matches OCaml `etat_suivant` structure:

```rust
/// Main game update: movement, transfers, spawning, despawn.
/// Called each frame when not paused.
pub fn update_game(state: &mut GameState, globals: &mut Globals) {
    // Update observer proper time (for time dilation)
    globals.observer_proper_time = state.ship.proper_time;

    // --- Inertia (position update) ---
    inertie_objet(&mut state.ship, globals);
    inertie_objets(&mut state.objects, globals);
    inertie_objets(&mut state.objects_oos, globals);
    inertie_objets(&mut state.toosmall, globals);
    inertie_objets(&mut state.toosmall_oos, globals);
    inertie_objets(&mut state.fragments, globals);
    inertie_objets(&mut state.chunks, globals);
    inertie_objets(&mut state.chunks_oos, globals);

    // --- Rotation (moment update) ---
    moment_objet(&mut state.ship, globals);
    moment_objets(&mut state.objects, globals);
    moment_objets(&mut state.objects_oos, globals);
    moment_objets(&mut state.toosmall, globals);
    moment_objets(&mut state.toosmall_oos, globals);
    moment_objets(&mut state.fragments, globals);

    // --- Size classification: move too-small asteroids ---
    let small_objs: Vec<Entity> = state.objects.drain_filter_stable(|e| too_small(e));
    state.toosmall.extend(small_objs);
    let small_frags: Vec<Entity> = state.fragments.drain_filter_stable(|e| too_small(e));
    state.toosmall.extend(small_frags);

    // --- OOS transfers ---
    transfer_oos(&mut state.objects, &mut state.objects_oos, globals);
    transfer_oos(&mut state.toosmall, &mut state.toosmall_oos, globals);
    transfer_oos(&mut state.chunks, &mut state.chunks_oos, globals);

    // --- Fragmentation (spawn fragments from dead entities) ---
    // (No collisions yet — entities won't die until Task 8)
    spawn_n_frags(&state.objects.clone(), &mut state.fragments, FRAGMENT_NUMBER, &mut state.rng);
    spawn_n_frags(&state.toosmall.clone(), &mut state.fragments, FRAGMENT_NUMBER, &mut state.rng);
    spawn_n_frags(&state.fragments.clone(), &mut state.fragments, FRAGMENT_NUMBER, &mut state.rng);

    // --- Move chunks out of fragments ---
    let new_chunks: Vec<Entity> = state.fragments.drain_filter_stable(|e| ischunk(e));
    state.chunks.extend(new_chunks);

    // --- Recenter (wrap positions) ---
    recenter_objets(&mut state.objects, globals);
    recenter_objets(&mut state.toosmall, globals);
    recenter_objets(&mut state.objects_oos, globals);
    recenter_objets(&mut state.toosmall_oos, globals);
    recenter_objets(&mut state.fragments, globals);

    // --- Spawning ---
    if globals.time_since_last_spawn > TIME_SPAWN_ASTEROID {
        globals.time_since_last_spawn = 0.0;

        let nb_asteroids_stage = ASTEROID_MIN_NB + ASTEROID_STAGE_NB * state.stage;
        if globals.current_stage_asteroids >= nb_asteroids_stage {
            // Advance to next stage
            state.stage += 1;
            globals.current_stage_asteroids = 0;

            // Pick new random stage colors (matches OCaml)
            let new_col = (
                randfloat(RAND_MIN_LUM, RAND_MAX_LUM, &mut state.rng),
                randfloat(RAND_MIN_LUM, RAND_MAX_LUM, &mut state.rng),
                randfloat(RAND_MIN_LUM, RAND_MAX_LUM, &mut state.rng),
            );
            let new_hdr = hdr(new_col);
            globals.mul_base = {
                let c = saturate(intensify(new_hdr, 1.0), FILTER_SATURATION);
                (c.r, c.v, c.b)
            };
            globals.space_color_goal = {
                let c = saturate(intensify(new_hdr, 10.0), SPACE_SATURATION);
                (c.r, c.v, c.b)
            };
            globals.star_color_goal = {
                let c = saturate(intensify(new_hdr, 100.0), STAR_SATURATION);
                (c.r, c.v, c.b)
            };
        }

        // Spawn one asteroid
        state.objects_oos.push(spawn_random_asteroid(
            state.stage,
            globals.phys_width,
            globals.phys_height,
            &mut state.rng,
        ));
        globals.current_stage_asteroids += 1;
    }

    let elapsed = (globals.time_current_frame - globals.time_last_frame) * globals.game_speed;
    globals.time_since_last_spawn += elapsed;

    // --- Despawn ---
    despawn(state);
}
```

**Step 2: Add the `despawn` function**

Matches OCaml `despawn`:

```rust
fn despawn(state: &mut GameState) {
    state.objects.retain(|e| is_alive(e) && notchunk(e));
    state.objects_oos.retain(|e| is_alive(e) && notchunk(e));
    state.toosmall.retain(|e| is_alive(e) && notchunk(e));
    state.toosmall_oos.retain(|e| is_alive(e) && notchunk(e));
    state.fragments.retain(|e| is_alive(e) && notchunk(e));
    state.chunks.retain(|e| positive_radius(e));
    state.chunks_oos.retain(|e| positive_radius(e));
    state.chunks_explo.retain(|e| positive_radius(e));
}
```

**Step 3: Add `drain_filter_stable` helper**

Since Rust stable doesn't have `drain_filter`, add a simple helper trait or function:

```rust
/// Drain elements matching predicate, keeping order stable.
/// Returns removed elements; modifies vec in-place to keep non-matching.
fn drain_filter_stable<T>(vec: &mut Vec<T>, pred: impl Fn(&T) -> bool) -> Vec<T> {
    let mut removed = Vec::new();
    let mut i = 0;
    while i < vec.len() {
        if pred(&vec[i]) {
            removed.push(vec.remove(i));
        } else {
            i += 1;
        }
    }
    removed
}
```

Note: use this standalone function instead of the method call syntax shown above. Update calls in `update_game` accordingly: `let small_objs = drain_filter_stable(&mut state.objects, |e| too_small(e));`

**Step 4: Verify**

Run: `cargo check`

**Step 5: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add update_game — spawning, stages, movement, despawn"
```

---

### Task 7: Render Asteroids and Chunks

**Files:**
- Modify: `rs/src/game.rs` — update `render_frame` and add `render_chunk`

**Step 1: Add `render_chunk` function**

Matches OCaml `render_chunk` — simpler rendering, just a filled circle:

```rust
/// Render a chunk (small debris) — simpler than full entity rendering
fn render_chunk(
    entity: &Entity,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let pos = multuple(
        addtuple(entity.position, globals.game_screenshake_pos),
        globals.ratio_rendu,
    );
    if globals.retro {
        let (x, y) = dither_tuple(pos, DITHER_AA, globals.current_jitter_double);
        renderer.fill_circle(
            x as f64, y as f64,
            (0.25 * globals.ratio_rendu * entity.visuals.radius).max(1.0),
            [128, 128, 128, 255],
        );
    } else {
        let intensity_chunk = 1.0;
        let color = to_rgba(
            intensify(hdr(entity.visuals.color), intensity_chunk * globals.game_exposure * entity.hdr_exposure),
            globals,
        );
        let (x, y) = dither_tuple(pos, DITHER_AA, globals.current_jitter_double);
        let r = dither_radius(
            globals.ratio_rendu * entity.visuals.radius,
            DITHER_AA, DITHER_POWER_RADIUS, rng,
        );
        renderer.fill_circle(x as f64, y as f64, r.max(1) as f64, color);
    }
}
```

**Step 2: Update `render_frame` to include asteroids**

Add asteroid rendering between stars and ship, matching OCaml render order:

```rust
// After stars, before ship:

// Chunks
for chunk in &state.chunks {
    render_chunk(chunk, renderer, globals, &mut state.rng);
}

// Asteroids (objects + toosmall + fragments)
for entity in &state.objects {
    render_visuals(entity, (0.0, 0.0), renderer, globals, &mut state.rng);
}
for entity in &state.toosmall {
    render_visuals(entity, (0.0, 0.0), renderer, globals, &mut state.rng);
}
for entity in &state.fragments {
    render_visuals(entity, (0.0, 0.0), renderer, globals, &mut state.rng);
}
```

**Step 3: Verify**

Run: `cargo check`

**Step 4: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): render asteroids, toosmall, fragments, and chunks"
```

---

### Task 8: Wire Into Main Loop + Test

**Files:**
- Modify: `rs/src/main.rs` — call `update_game` in game loop

**Step 1: Call `update_game` in the main loop**

In `main.rs`, find where per-frame updates happen (after input handling, before rendering). Add the call:

```rust
// After input handling, before render:
if !globals.pause {
    game::update_game(&mut state, &mut globals);
}
```

Remove the existing `modulo_reso` wrapping on ship position (line ~175) since `update_game` now handles all movement via `inertie_objet` + `recenter`. But keep ship-specific movement (acceleration, strafe) in main.rs input handling.

Note: Check that ship movement (acceleration/strafe applied in main.rs) still works correctly alongside `inertie_objet` in `update_game`. The OCaml applies velocity changes (acceleration) to ship velocity, then `inertie_objet` moves the ship by that velocity. Same pattern should work in Rust.

**Step 2: Verify build**

Run: `cargo check`

**Step 3: Test**

Run: `cargo run --release`

Expected behavior:
- Asteroids appear from off-screen edges after ~2 seconds
- They drift across the screen with rotation
- Random jagged polygon shapes, colored per-stage
- They wrap around (toroidal world)
- Stage colors change (star/space colors transition smoothly)
- No collisions — asteroids pass through ship and each other
- Ship still works: mouse aim, click accelerate, WASD movement

**Step 4: Commit**

```bash
git add rs/src/main.rs rs/src/game.rs
git commit -m "feat(rs): asteroid spawning, rendering, fragmentation, and culling"
```

---

## Reference: OCaml → Rust Function Mapping

| OCaml | Rust | Location |
|-------|------|----------|
| `spawn_random_asteroid` | `spawn_random_asteroid` | objects.rs |
| `frag_asteroid` | `frag_asteroid` | objects.rs |
| `spawn_n_frags` | `spawn_n_frags` | objects.rs |
| `inertie_objet` | `inertie_objet` | game.rs |
| `moment_objet` | `moment_objet` | game.rs |
| `recenter_objet` | `recenter_objet` | game.rs |
| `checkspawn_objet` | `checkspawn_objet` | game.rs |
| `deplac_objet_abso` | `deplac_objet_abso` | game.rs |
| `despawn` | `despawn` | game.rs |
| `render_chunk` | `render_chunk` | game.rs |
| `half_color` | `half_color` | color.rs (exists) |
| `saturate` | `saturate` | color.rs (exists) |

## Key OCaml Behaviors to Preserve

1. **Spawn timing**: `time_since_last_spawn` starts at 9.5, `current_stage_asteroids` at 3. Since `ASTEROID_MIN_NB + ASTEROID_STAGE_NB * 0 = 2`, and 3 ≥ 2, the first spawn immediately triggers stage advance to 1 with new colors.
2. **3x wrapping**: `modulo_3reso` allows objects to exist 1 screen-width off-screen in any direction.
3. **OOS pattern**: Objects live in `_oos` lists when off-screen, transfer to main list on entering screen.
4. **Fragment spawning**: 5 fragments per dead entity, inheriting parent color.
5. **Color transitions**: `half_color` exponential decay toward goal, `saturate` for color intensity.
