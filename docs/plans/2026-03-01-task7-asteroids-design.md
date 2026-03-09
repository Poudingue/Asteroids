# Task 7: Asteroids — Spawning, Rendering, Fragmentation

## Goal

Port the OCaml asteroid system faithfully to Rust. Asteroids spawn in waves per stage, render as random jagged polygons with HDR colors, wrap around the toroidal world, and fragment when destroyed.

## Files

- **Modify:** `rs/src/game.rs` — stage system, spawning, movement, wrapping, rendering, fragmentation, color interpolation
- **Modify:** `rs/src/objects.rs` — minor tweaks if needed (spawn functions already exist)

## Design

### 1. Stage & Spawn System

Matches OCaml `etat_suivant` logic:

- `GameState` already has `stage` field. Add `time_since_last_spawn: f64` and `current_stage_asteroids: i32`.
- Each frame (when not paused): increment `time_since_last_spawn` by `dt * game_speed`.
- When timer exceeds spawn interval (~2s): spawn asteroid via `spawn_random_asteroid(stage)` → push to `objects_oos`.
- Each stage spawns `initial_asteroids + stage` asteroids total.
- Stage increments when all asteroids/fragments/chunks are gone.
- On stage change: set `star_color_goal`, `space_color_goal`, `mul_base` to random per-stage colors (OCaml pattern: `intensify(random_color, 100)` for stars, `intensify(random_color, 10)` for space, `saturate(random_color, filter_saturation)` for mul_base).

### 2. Asteroid Movement

Each frame, for all entity lists (`objects`, `objects_oos`, `toosmall`, `toosmall_oos`, `fragments`, `chunks`, `chunks_oos`):

- `position += velocity * elapsed_time` (where `elapsed_time = dt * game_speed`)
- `orientation += moment * elapsed_time`
- Wrap position with `modulo_3reso()` (3x-resolution wrapping allows off-screen existence)

Matches OCaml `deplac_objet` / `deplac_objet_inertie`.

### 3. OOS (Out-of-Screen) Transfer

Matches OCaml `checkspawn_objet` / `checknotspawn_objet`:

- Each frame, check each entity's position against screen bounds (with margin = entity radius).
- `objects_oos` → `objects`: when entity enters visible area.
- `objects` → `objects_oos`: when entity leaves visible area.
- Same pattern for `toosmall` ↔ `toosmall_oos`, `chunks` ↔ `chunks_oos`.
- Despawn entities that are dead (health ≤ 0), negative radius, or position too far from play area.

### 4. Rendering

`render_visuals()` already handles Entity rendering (base circle + polygon shapes). In `render_frame()`:

- Iterate and render: `objects`, `toosmall`, `fragments`, `chunks` (only on-screen lists).
- Chunks render slightly differently in OCaml (`render_chunk` — simpler, just a circle at 0.25x size, dimmer). Port this.

### 5. Fragmentation

Port OCaml `frag_asteroid()` faithfully:

- **Trigger:** When asteroid health ≤ 0 (wired up in Task 8 collisions, but code ready now).
- **Count:** `FRAGMENT_NUMBER` (5) fragments per asteroid.
- **Fragment radius:** `randfloat(0.4, 0.7) * parent_radius`.
- **Fragment position:** Parent position + offset at random angle by `(parent_radius - new_radius)`.
- **Fragment velocity:** Parent velocity + random direction at `randfloat(FRAG_MIN_SPEED, FRAG_MAX_SPEED)`.
- **Fragment color:** Parent color, `hdr_exposure` adjusted by `randfloat(0.666, 1.5)`.
- **Fragment polygon:** Regenerated via `polygon_asteroid(new_radius)`.
- **Chunk threshold:** Fragments with radius < `MIN_ASTEROID_RADIUS` (~50px) go to `chunks` instead of `fragments`/`objects`.

Also port `spawn_n_frags()` helper that processes a list of dead entities and spawns their fragments.

### 6. Color Interpolation

Port OCaml `half_color` for smooth per-frame transitions:

- `star_color` → `star_color_goal` with `SPACE_HALF_LIFE` decay
- `space_color` → `space_color_goal` with `SPACE_HALF_LIFE` decay
- `mul_color` → `mul_base` with appropriate half-life

These run each frame in `update_frame()`.

### 7. Recenter (Wrapping)

Port OCaml `recenter_objet`:

- Apply `modulo_3reso()` to entity positions each frame.
- Stars use `modulo_reso()` (1x wrapping, already implemented).
- Entities use 3x wrapping to allow off-screen existence.

## Parameters (from OCaml)

Key constants already in `rs/src/parameters.rs`:
- `FRAGMENT_NUMBER = 5`
- `FRAG_MIN_SPEED`, `FRAG_MAX_SPEED`
- `MIN_ASTEROID_RADIUS`
- `SPAWN_INTERVAL`, `INITIAL_ASTEROIDS`
- Stage color parameters: `RAND_MIN_LUM`, `RAND_MAX_LUM`, `SPACE_SATURATION`, `STAR_SATURATION`, `FILTER_SATURATION`

## Testing

Run the game: asteroids should appear from off-screen, drift across with rotation, wrap around edges. They have random jagged polygon shapes with per-stage colors. No collisions yet — they pass through everything. Stage advances when manually testing isn't possible without collisions, so initial stage just keeps spawning.
