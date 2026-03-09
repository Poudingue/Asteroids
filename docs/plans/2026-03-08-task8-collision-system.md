# Task 8: Collision System — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Port the OCaml spatial-grid collision system, giving asteroids and the ship elastic physics and health-based damage.

**Architecture:** Index-based collision grid (15×9) stores `GridEntry` enum values (which list + which index). Detection runs read-only to collect pairs; consequences are applied sequentially using clones to satisfy the borrow checker. Fragment-vs-fragment repulsion uses a separate pass with promotion of settled fragments to `state.objects`. No explosions or projectiles yet — those come in Task 9.

**Tech Stack:** Rust, existing `math_utils` (moytuple, affine_to_polar, hypothenuse, carre), existing `parameters` constants.

---

### Task 1: Add `damage` and `phys_damage` helpers

**Files:**
- Modify: `rs/src/game.rs`

**Step 1: Add the two damage functions** (before `update_game`)

Matches OCaml `damage` and `phys_damage`:

```rust
/// Apply explosion/direct damage to an entity.
fn damage(entity: &mut Entity, amount: f64, globals: &mut Globals) {
    let actual = (entity.dam_ratio * amount - entity.dam_res).max(0.0);
    entity.health -= actual;
    globals.game_screenshake += actual * SCREENSHAKE_DAM_RATIO;
    if globals.variable_exposure {
        globals.game_exposure *= EXPOSURE_RATIO_DAMAGE;
    }
}

/// Apply physical-collision damage to an entity.
fn phys_damage(entity: &mut Entity, amount: f64, globals: &mut Globals) {
    let actual = (entity.phys_ratio * amount - entity.phys_res).max(0.0);
    entity.health -= actual;
    globals.game_screenshake +=
        actual * SCREENSHAKE_PHYS_RATIO * entity.mass / SCREENSHAKE_PHYS_MASS;
}
```

**Step 2: Verify**

Run: `cargo check` from `rs/`
Expected: compiles (dead-code warnings only).

**Step 3: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add damage and phys_damage helpers"
```

---

### Task 2: Add collision detection functions

**Files:**
- Modify: `rs/src/game.rs`

**Step 1: Add detection primitives** (after `phys_damage`, before `update_game`)

Matches OCaml `collision_circles`, `collision_point`, `collisions_points`, `collision_poly`, `collision`:

```rust
fn collision_circles(pos0: Vec2, r0: f64, pos1: Vec2, r1: f64) -> bool {
    distancecarre(pos0, pos1) < carre(r0 + r1)
}

fn collision_point(pos_point: Vec2, pos_circle: Vec2, radius: f64) -> bool {
    distancecarre(pos_point, pos_circle) < carre(radius)
}

fn collisions_points(points: &[Vec2], pos_circle: Vec2, radius: f64) -> bool {
    points.iter().any(|&p| collision_point(p, pos_circle, radius))
}

fn collision_poly(pos: Vec2, poly: &[Vec2], rotat: f64, circle_pos: Vec2, radius: f64) -> bool {
    let pts = depl_affine_poly(&poly_to_affine(poly, rotat, 1.0), pos);
    collisions_points(&pts, circle_pos, radius)
}

/// Test collision between two entities.
/// `precis`: true = polygon check after circle broadphase; false = circle only.
/// Matches OCaml: skips when both entities are identical (by pointer-like index check at call site).
fn collision_entities(obj1: &Entity, obj2: &Entity, precis: bool, advanced_hitbox: bool) -> bool {
    let (pos1, pos2) = (obj1.position, obj2.position);
    let (h1, h2) = (&obj1.hitbox, &obj2.hitbox);
    if !advanced_hitbox && !precis {
        collision_circles(pos1, h1.int_radius, pos2, h2.int_radius)
    } else if collision_circles(pos1, h1.int_radius, pos2, h2.int_radius) {
        true
    } else {
        collision_poly(pos1, &h1.points, obj1.orientation, pos2, h2.int_radius)
            || collision_poly(pos2, &h2.points, obj2.orientation, pos1, h1.int_radius)
    }
}
```

Note: `depl_affine_poly`, `poly_to_affine` already exist in `game.rs`; `distancecarre`, `carre` are in `math_utils.rs` (wildcard-imported).

**Step 2: Verify**

Run: `cargo check`

**Step 3: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add collision detection — circles, poly, entity"
```

---

### Task 3: Grid data structures + `insert_into_grid`

**Files:**
- Modify: `rs/src/game.rs`

**Step 1: Add `GridEntry` enum and `Grid` type alias** (at top of `game.rs`, after imports)

```rust
/// Identifies an entity by which list it lives in and its index.
/// Matches OCaml's ref-based grid entries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GridEntry {
    Object(usize),
    ObjectOos(usize),
    TooSmall(usize),
    TooSmallOos(usize),
    Fragment(usize),
    Ship,
}

type CollisionGrid = Vec<Vec<GridEntry>>;

fn make_grid() -> CollisionGrid {
    vec![Vec::new(); (WIDTH_COLLISION_TABLE * HEIGHT_COLLISION_TABLE) as usize]
}

fn clear_grid(grid: &mut CollisionGrid) {
    for cell in grid.iter_mut() {
        cell.clear();
    }
}
```

**Step 2: Add `insert_into_grid`**

Matches OCaml `rev_filtertable`. Inserts each entity into exactly one grid cell based on its center position, with jitter offset `globals.current_jitter_coll_table`:

```rust
/// Insert a slice of (entry, position) pairs into the collision grid.
/// Matches OCaml rev_filtertable: each entity goes into one cell (its center).
fn insert_into_grid(
    entries: &[(GridEntry, Vec2)],
    grid: &mut CollisionGrid,
    globals: &Globals,
) {
    let gw = WIDTH_COLLISION_TABLE as f64;
    let gh = HEIGHT_COLLISION_TABLE as f64;
    let (jx, jy) = globals.current_jitter_coll_table;
    for &(entry, (x, y)) in entries {
        let x2 = jx + gw * (x + globals.phys_width) / (3.0 * globals.phys_width);
        let y2 = jy + gh * (y + globals.phys_height) / (3.0 * globals.phys_height);
        if x2 < 0.0 || y2 < 0.0 || x2 >= gw || y2 >= gh {
            continue;
        }
        let xi = x2 as usize;
        let yi = y2 as usize;
        let idx = xi * HEIGHT_COLLISION_TABLE as usize + yi;
        grid[idx].push(entry);
    }
}
```

**Step 3: Add `get_entity` / `get_entity_mut` lookup helpers**

```rust
fn get_entity<'a>(state: &'a GameState, entry: GridEntry) -> &'a Entity {
    match entry {
        GridEntry::Object(i)    => &state.objects[i],
        GridEntry::ObjectOos(i) => &state.objects_oos[i],
        GridEntry::TooSmall(i)  => &state.toosmall[i],
        GridEntry::TooSmallOos(i) => &state.toosmall_oos[i],
        GridEntry::Fragment(i)  => &state.fragments[i],
        GridEntry::Ship         => &state.ship,
    }
}

fn get_entity_mut<'a>(state: &'a mut GameState, entry: GridEntry) -> &'a mut Entity {
    match entry {
        GridEntry::Object(i)    => &mut state.objects[i],
        GridEntry::ObjectOos(i) => &mut state.objects_oos[i],
        GridEntry::TooSmall(i)  => &mut state.toosmall[i],
        GridEntry::TooSmallOos(i) => &mut state.toosmall_oos[i],
        GridEntry::Fragment(i)  => &mut state.fragments[i],
        GridEntry::Ship         => &mut state.ship,
    }
}
```

**Step 4: Verify**

Run: `cargo check`

**Step 5: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add collision grid — GridEntry, insert_into_grid, entity lookup"
```

---

### Task 4: `consequences_collision` — elastic bounce + damage

**Files:**
- Modify: `rs/src/game.rs`

**Step 1: Add `consequences_collision`**

Matches OCaml `consequences_collision` for the physical collision branch (Task 8 has no explosions/projectiles yet). Takes two owned entities, returns updated pair — avoids simultaneous mut borrows:

```rust
/// Apply physical collision consequences to two entities.
/// Returns updated (e1, e2). Matches OCaml consequences_collision physical branch.
fn consequences_collision(
    mut e1: Entity,
    mut e2: Entity,
    globals: &mut Globals,
) -> (Entity, Entity) {
    let total_mass = e1.mass + e2.mass;
    // Mass-weighted average velocity (accounts for proper time)
    let moy_vel = moytuple(
        multuple(e1.velocity, 1.0 / e1.proper_time),
        multuple(e2.velocity, 1.0 / e2.proper_time),
        e1.mass / total_mass,
    );
    let (angle1, _) = affine_to_polar(soustuple(e1.position, e2.position));
    let (angle2, _) = affine_to_polar(soustuple(e2.position, e1.position));

    let old_vel1 = e1.velocity;
    let old_vel2 = e2.velocity;

    // New velocities — elastic bounce scaled by proper time
    e1.velocity = multuple(
        addtuple(moy_vel, polar_to_affine(angle1, total_mass / e1.mass)),
        e1.proper_time,
    );
    e2.velocity = multuple(
        addtuple(moy_vel, polar_to_affine(angle2, total_mass / (e2.mass * e2.proper_time))),
        e2.proper_time,
    );

    if !globals.pause {
        let dt = (globals.time_current_frame - globals.time_last_frame) * globals.game_speed;
        // Positional repulsion to separate overlapping entities
        e1.position = addtuple(e1.position, polar_to_affine(angle1, MIN_REPULSION * dt));
        e2.position = addtuple(e2.position, polar_to_affine(angle2, MIN_REPULSION * dt));
        // Velocity bounce impulse
        e1.velocity = addtuple(e1.velocity, polar_to_affine(angle1, MIN_BOUNCE * dt));
        e2.velocity = addtuple(e2.velocity, polar_to_affine(angle2, MIN_BOUNCE * dt));
        // Physical damage proportional to velocity change²
        let g1 = hypothenuse(soustuple(old_vel1, e1.velocity));
        let g2 = hypothenuse(soustuple(old_vel2, e2.velocity));
        phys_damage(&mut e1, globals.ratio_phys_deg * carre(g1), globals);
        phys_damage(&mut e2, globals.ratio_phys_deg * carre(g2), globals);
    }
    (e1, e2)
}
```

**Step 2: Add `consequences_collision_frags`** (fragment-only repulsion, no damage)

Matches OCaml `consequences_collision_frags`:

```rust
/// Apply fragment-vs-fragment repulsion (no damage).
fn consequences_collision_frags(mut f1: Entity, mut f2: Entity, globals: &Globals) -> (Entity, Entity) {
    let (angle1, _) = affine_to_polar(soustuple(f1.position, f2.position));
    let (angle2, _) = affine_to_polar(soustuple(f2.position, f1.position));
    let dt = (globals.time_current_frame - globals.time_last_frame) * globals.game_speed;
    f1.position = addtuple(f1.position, polar_to_affine(angle1, dt * FRAGMENT_MIN_REPULSION));
    f2.position = addtuple(f2.position, polar_to_affine(angle2, dt * FRAGMENT_MIN_REPULSION));
    f1.velocity = addtuple(f1.velocity, polar_to_affine(angle1, dt * FRAGMENT_MIN_BOUNCE));
    f2.velocity = addtuple(f2.velocity, polar_to_affine(angle2, dt * FRAGMENT_MIN_BOUNCE));
    (f1, f2)
}
```

**Step 3: Verify**

Add imports needed in `game.rs`:
- `use crate::math_utils::{moytuple, affine_to_polar, hypothenuse};` — check if these are already imported via wildcard; if not, add them.

Run: `cargo check`

**Step 4: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add consequences_collision — elastic bounce, phys_damage, frag repulsion"
```

---

### Task 5: `calculate_collision_tables` + pair application

**Files:**
- Modify: `rs/src/game.rs`

**Step 1: Add `collect_pairs_for_cell`**

Read-only scan of two grid cell lists, collecting all colliding pairs. Matches OCaml's `calculate_collisions_listes_objets` (detection only, no mutation yet):

```rust
/// Collect all colliding (e1, e2) pairs from two cell lists.
/// Matches OCaml: iterates list1 × list2, both directions for same list (extend=true).
fn collect_pairs_for_cell(
    cell1: &[GridEntry],
    cell2: &[GridEntry],
    state: &GameState,
    precis: bool,
    globals: &Globals,
    pairs: &mut Vec<(GridEntry, GridEntry)>,
) {
    for &e1 in cell1 {
        for &e2 in cell2 {
            if e1 == e2 {
                continue; // same entity — matches OCaml's objet1 = objet2 check
            }
            let ent1 = get_entity(state, e1);
            let ent2 = get_entity(state, e2);
            if collision_entities(ent1, ent2, precis, globals.advanced_hitbox) {
                pairs.push((e1, e2));
            }
        }
    }
}
```

**Step 2: Add `apply_collision_pairs`**

Applies `consequences_collision` to all collected pairs. Uses clone to avoid simultaneous mut borrows:

```rust
/// Apply physical collision consequences to all collected pairs.
fn apply_collision_pairs(
    pairs: &[(GridEntry, GridEntry)],
    state: &mut GameState,
    globals: &mut Globals,
) {
    for &(e1_ref, e2_ref) in pairs {
        let e1 = get_entity(state, e1_ref).clone();
        let e2 = get_entity(state, e2_ref).clone();
        let (new_e1, new_e2) = consequences_collision(e1, e2, globals);
        *get_entity_mut(state, e1_ref) = new_e1;
        *get_entity_mut(state, e2_ref) = new_e2;
    }
}
```

**Step 3: Add `calculate_collision_tables`**

Matches OCaml: same-cell pairs + (when `extend=true`) adjacent-cell pairs:

```rust
/// Run collision detection between two grids, applying consequences.
/// `extend=true`: also check adjacent cells (right, down, diagonal).
/// `precis=true`: use polygon hitbox after circle broadphase.
/// Matches OCaml calculate_collision_tables.
fn calculate_collision_tables(
    grid1: &CollisionGrid,
    grid2: &CollisionGrid,
    extend: bool,
    state: &mut GameState,
    globals: &mut Globals,
) {
    let w = WIDTH_COLLISION_TABLE as usize;
    let h = HEIGHT_COLLISION_TABLE as usize;
    let mut pairs: Vec<(GridEntry, GridEntry)> = Vec::new();

    // Same-cell pairs (always)
    for x in 0..w {
        for y in 0..h {
            let idx = x * h + y;
            collect_pairs_for_cell(&grid1[idx], &grid2[idx], state, true, globals, &mut pairs);
        }
    }

    // Adjacent-cell pairs (only when extend=true)
    if extend {
        for x in 0..w - 1 {
            for y in 0..h - 1 {
                let base = x * h + y;
                let right = base + h;      // x+1, y
                let down  = base + 1;      // x, y+1
                let diag  = base + h + 1;  // x+1, y+1
                collect_pairs_for_cell(&grid1[base], &grid2[down],  state, false, globals, &mut pairs);
                collect_pairs_for_cell(&grid1[base], &grid2[right], state, false, globals, &mut pairs);
                collect_pairs_for_cell(&grid1[base], &grid2[diag],  state, false, globals, &mut pairs);
            }
        }
    }

    apply_collision_pairs(&pairs, state, globals);
}
```

**Step 4: Verify**

Run: `cargo check`

**Step 5: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add calculate_collision_tables — grid-based pair detection and consequences"
```

---

### Task 6: Fragment-vs-fragment collision + promotion

**Files:**
- Modify: `rs/src/game.rs`

**Step 1: Add `run_fragment_collisions`**

Matches OCaml `calculate_collisions_frags`:
- Finds colliding fragment pairs; applies repulsion
- Non-colliding fragments are promoted to `state.objects`

```rust
/// Repulse colliding fragment pairs.
/// Fragments NOT involved in any collision this frame are promoted to state.objects.
/// Matches OCaml: calculate_collisions_frags + promotion logic.
fn run_fragment_collisions(state: &mut GameState, globals: &Globals) {
    let n = state.fragments.len();
    let mut involved = vec![false; n];

    // Collect colliding pairs (by index)
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if collision_entities(&state.fragments[i], &state.fragments[j], false, globals.advanced_hitbox) {
                pairs.push((i, j));
                involved[i] = true;
                involved[j] = true;
            }
        }
    }

    // Apply repulsion to each pair
    for (i, j) in pairs {
        let f1 = state.fragments[i].clone();
        let f2 = state.fragments[j].clone();
        let (new_f1, new_f2) = consequences_collision_frags(f1, f2, globals);
        state.fragments[i] = new_f1;
        state.fragments[j] = new_f2;
    }

    // Promote non-colliding fragments to objects (they've "settled")
    let mut settled: Vec<Entity> = Vec::new();
    let mut still_colliding: Vec<Entity> = Vec::new();
    for (i, frag) in state.fragments.drain(..).enumerate() {
        if involved[i] {
            still_colliding.push(frag);
        } else {
            settled.push(frag);
        }
    }
    state.fragments = still_colliding;
    state.objects.extend(settled);
}
```

**Step 2: Verify**

Run: `cargo check`

**Step 3: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): add fragment collision — repulsion and promotion to objects"
```

---

### Task 7: Wire collisions into `update_game`

**Files:**
- Modify: `rs/src/game.rs`

**Step 1: Add grid state to `GameState` (or use local `update_game` variables)**

The OCaml uses module-level mutable arrays (effectively globals). In Rust, create them as locals in `update_game` each frame — `make_grid()` is cheap (15×9 = 135 Vecs):

```rust
// At the start of update_game, after inertia/rotation:
let mut grid_objects  = make_grid();
let mut grid_toosmall = make_grid();
let mut grid_other    = make_grid();
let mut grid_frag     = make_grid();
```

**Step 2: Build the four grids** (after inertia/recenter, before spawning)

```rust
// Build entry lists with positions — matches OCaml's ref concatenation
let mut entries_obj: Vec<(GridEntry, Vec2)> = state.objects
    .iter().enumerate().map(|(i, e)| (GridEntry::Object(i), e.position)).collect();
entries_obj.extend(state.objects_oos
    .iter().enumerate().map(|(i, e)| (GridEntry::ObjectOos(i), e.position)));

let mut entries_small: Vec<(GridEntry, Vec2)> = state.toosmall
    .iter().enumerate().map(|(i, e)| (GridEntry::TooSmall(i), e.position)).collect();
entries_small.extend(state.toosmall_oos
    .iter().enumerate().map(|(i, e)| (GridEntry::TooSmallOos(i), e.position)));

let entries_other: Vec<(GridEntry, Vec2)> = vec![
    (GridEntry::Ship, state.ship.position)
    // Explosions and projectiles added in Task 9
];

let entries_frag: Vec<(GridEntry, Vec2)> = state.fragments
    .iter().enumerate().map(|(i, e)| (GridEntry::Fragment(i), e.position)).collect();

insert_into_grid(&entries_obj,   &mut grid_objects,  globals);
insert_into_grid(&entries_small, &mut grid_toosmall, globals);
insert_into_grid(&entries_other, &mut grid_other,    globals);
insert_into_grid(&entries_frag,  &mut grid_frag,     globals);
```

**Step 3: Run collision tables** (matches OCaml's 6 `calculate_collision_tables` calls)

```rust
// Asteroid vs asteroid (extend=true — same and adjacent cells)
calculate_collision_tables(&grid_objects.clone(), &grid_objects.clone(), true, state, globals);
// Asteroid vs toosmall (extend=false)
calculate_collision_tables(&grid_objects.clone(), &grid_toosmall.clone(), false, state, globals);
// Ship/other vs asteroid (extend=true)
calculate_collision_tables(&grid_other.clone(), &grid_objects.clone(), true, state, globals);
// Ship/other vs toosmall (extend=true)
calculate_collision_tables(&grid_other.clone(), &grid_toosmall.clone(), true, state, globals);
// Ship/other vs fragment (extend=true)
calculate_collision_tables(&grid_other.clone(), &grid_frag.clone(), true, state, globals);
```

Note: `grid_objects vs grid_frag` and `grid_objects vs grid_objects_frag` are commented out in OCaml — skip here too.

**Step 4: Add score + game speed slowdown on asteroid death**

Matches OCaml: count dead asteroids, add to score, slow game:

```rust
// Count destroyed asteroids this frame
let nb_destroyed = state.objects.iter().filter(|e| is_dead(e)).count()
    + state.objects_oos.iter().filter(|e| is_dead(e)).count()
    + state.toosmall.iter().filter(|e| is_dead(e)).count()
    + state.toosmall_oos.iter().filter(|e| is_dead(e)).count()
    + state.fragments.iter().filter(|e| is_dead(e)).count();

globals.game_speed *= RATIO_TIME_DESTR_ASTEROID.powi(nb_destroyed as i32);
state.score += nb_destroyed as i32;
```

**Step 5: Run fragment collisions** (after all other collision tables, before fragmentation spawning)

```rust
run_fragment_collisions(state, globals);
```

**Step 6: Move the existing `spawn_n_frags` calls AFTER collisions** (they already exist in `update_game` — just ensure ordering is: collisions → count dead → frag spawn → despawn)

The existing call order in `update_game` should be:
1. Inertia, rotation
2. OOS transfer
3. **[NEW] Build grids + run collision tables**
4. **[NEW] Count dead + score + game_speed**
5. **[NEW] run_fragment_collisions**
6. Fragmentation spawn (existing `spawn_n_frags` calls)
7. Chunk promotion (existing)
8. Recenter
9. Spawning
10. Despawn

**Step 7: Verify**

Run: `cargo check`

**Step 8: Commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): wire collision system into update_game"
```

---

### Task 8: Test visually

**Step 1: Build and run**

```bash
cd rs && cargo build --release && ./target/release/asteroids
```

**Expected behavior:**
- Asteroids bounce off each other with elastic physics
- Ship bounces off asteroids (loses health on hard impacts)
- Fragments spread apart after spawning, then become regular asteroids
- Score increments when asteroids are destroyed (by colliding with each other or the ship)
- Game slightly slows down on each asteroid destroyed
- No freezes or crashes with ~10-20 asteroids on screen

**Step 2: Run `cargo clippy`** and fix any new warnings (collision functions may trigger unused-variable warnings):

```bash
cd rs && cargo clippy
```

**Step 3: Final commit**

```bash
git add rs/src/game.rs
git commit -m "feat(rs): complete Task 8 — spatial grid collision system"
```

---

## Reference: OCaml → Rust Mapping

| OCaml | Rust | Location |
|-------|------|----------|
| `collision_circles` | `collision_circles` | game.rs |
| `collision_poly` | `collision_poly` | game.rs |
| `collision` | `collision_entities` | game.rs |
| `damage` | `damage` | game.rs |
| `phys_damage` | `phys_damage` | game.rs |
| `consequences_collision` | `consequences_collision` | game.rs |
| `consequences_collision_frags` | `consequences_collision_frags` | game.rs |
| `rev_filtertable` | `insert_into_grid` | game.rs |
| `calculate_collision_tables` | `calculate_collision_tables` | game.rs |
| `calculate_collisions_frags` | `run_fragment_collisions` | game.rs |

## Key OCaml Behaviors to Preserve

1. **Double application for same-list pairs**: OCaml processes `(a,b)` AND `(b,a)` for same-table collisions. The `collect_pairs_for_cell` function called with `grid1 == grid2` (same ptr) emits both orders. This is intentional — it doubles the bounce effect for same-list collisions.
2. **Jitter on grid insertion**: `globals.current_jitter_coll_table` offsets cell assignment slightly each frame, distributing border entities across cells.
3. **Fragment promotion**: Fragments not involved in any collision this frame are moved to `state.objects` (they've "settled"). This is what makes fragments eventually become regular asteroids.
4. **Score counting**: Count dead entities BEFORE calling `spawn_n_frags` and `despawn` — otherwise newly-dead entities would be removed before counting.
5. **`precis=false` for adjacent cells**: The `extend` pass uses `precis=false` (circle only) — polygon check only for same-cell pairs.
