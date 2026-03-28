use rand::prelude::*;

use crate::color::*;
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;
pub use crate::physics::{collision_circles, collision_point};
use crate::physics::{
    collision_entities,
    consequences_collision, consequences_collision_frags,
    damage,
    CollisionGrid, GridEntry,
    make_grid, insert_into_grid,
};
use crate::renderer::Renderer2D;

// ============================================================================
// GameState
// ============================================================================

pub struct GameState {
    pub score: i32,
    pub lives: i32,
    pub stage: i32,
    pub cooldown: f64,
    pub cooldown_tp: f64,
    pub last_health: f64,
    // Death / respawn state
    pub is_dead: bool,         // true while in mort() loop
    pub time_of_death: f64,   // wall-clock time when ship died
    pub ship: Entity,
    pub objects: Vec<Entity>,
    pub objects_oos: Vec<Entity>,
    pub toosmall: Vec<Entity>,
    pub toosmall_oos: Vec<Entity>,
    pub fragments: Vec<Entity>,
    pub chunks: Vec<Entity>,
    pub chunks_oos: Vec<Entity>,
    pub chunks_explo: Vec<Entity>,
    pub projectiles: Vec<Entity>,
    pub explosions: Vec<Entity>,
    pub smoke: Vec<Entity>,
    pub smoke_oos: Vec<Entity>,
    pub sparks: Vec<Entity>,
    pub stars: Vec<Star>,
    pub rng: ThreadRng,
    /// Pause menu interactive buttons.
    pub buttons: Vec<ButtonBoolean>,
    /// Left mouse button state — used for rising-edge click detection.
    pub mouse_button_down: bool,
}

use crate::pause_menu::ButtonBoolean;

impl GameState {
    pub fn new(globals: &Globals) -> Self {
        let mut rng = thread_rng();
        let mut ship = spawn_ship();
        ship.position = Vec2::new(globals.phys_width / 2.0, globals.phys_height / 2.0);

        Self {
            score: 0,
            lives: SHIP_MAX_LIVES,
            stage: 0,
            cooldown: 0.0,
            cooldown_tp: 0.0,
            last_health: SHIP_MAX_HEALTH,
            is_dead: false,
            time_of_death: 0.0,
            ship,
            objects: Vec::new(),
            objects_oos: Vec::new(),
            toosmall: Vec::new(),
            toosmall_oos: Vec::new(),
            fragments: Vec::new(),
            chunks: Vec::new(),
            chunks_oos: Vec::new(),
            chunks_explo: Vec::new(),
            projectiles: Vec::new(),
            explosions: Vec::new(),
            smoke: Vec::new(),
            smoke_oos: Vec::new(),
            sparks: Vec::new(),
            stars: spawn_stars(
                globals.stars_nb,
                globals.phys_width,
                globals.phys_height,
                &mut rng,
            ),
            rng,
            buttons: crate::pause_menu::make_buttons(globals),
            mouse_button_down: false,
        }
    }
}

// ============================================================================
// Color helpers
// ============================================================================

/// Convert a (r,v,b) color tuple to HdrColor
fn hdr(color: (f64, f64, f64)) -> HdrColor {
    HdrColor::new(color.0, color.1, color.2)
}

/// Convert an HDR color (already intensified) to renderer [u8; 4] RGBA
fn to_rgba(color: HdrColor, globals: &Globals) -> [u8; 4] {
    rgb_of_hdr(
        color,
        &hdr(globals.add_color),
        &hdr(globals.mul_color),
        globals.game_exposure,
    )
}

// ============================================================================
// Movement functions — ported from ml/asteroids.ml
// ============================================================================

/// Displace an object by a velocity vector, scaled by dt * game_speed * observer/proper time
pub fn move_entity(entity: &mut Entity, vel: Vec2, globals: &Globals) {
    let time_factor = globals.dt() * globals.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.position = proj(entity.position, vel, time_factor);
}

/// Apply an object's velocity as displacement (inertia)
pub fn apply_inertia(entity: &mut Entity, globals: &Globals) {
    let vel = entity.velocity;
    move_entity(entity, vel, globals);
}

/// Accelerate an object (velocity += accel * dt * ...)
pub fn accelerate_entity(entity: &mut Entity, accel: Vec2, globals: &Globals) {
    let time_factor = globals.dt() * globals.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.velocity = proj(entity.velocity, accel, time_factor);
}

/// Instant velocity change (no time scaling)
pub fn boost_entity(entity: &mut Entity, boost: Vec2) {
    entity.velocity = proj(entity.velocity, boost, 1.0);
}

/// Timed rotation (orientation += rotation * dt * ...)
pub fn rotate_entity(entity: &mut Entity, rotation: f64, globals: &Globals) {
    let time_factor = globals.dt() * globals.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.orientation += rotation * time_factor;
}

/// Instant rotation (no time scaling)
pub fn turn_entity(entity: &mut Entity, rotation: f64) {
    entity.orientation += rotation;
}

/// Angular acceleration (moment += momentum * dt * ...)
pub fn apply_torque(entity: &mut Entity, momentum: f64, globals: &Globals) {
    let time_factor = globals.dt() * globals.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.moment += momentum * time_factor;
}

/// Instant angular momentum change
pub fn boost_torque(entity: &mut Entity, momentum: f64) {
    entity.moment += momentum;
}

/// Apply moment as rotation (rotational inertia)
pub fn apply_angular_momentum(entity: &mut Entity, globals: &Globals) {
    let moment = entity.moment;
    rotate_entity(entity, moment, globals);
}

/// Instant absolute displacement (for camera movement)
pub fn translate_entity(entity: &mut Entity, velocity: Vec2) {
    entity.position = proj(entity.position, velocity, 1.0);
}

/// Apply inertia to all entities in a list
pub fn apply_inertia_all(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        apply_inertia(e, globals);
    }
}

/// Apply angular momentum to all entities in a list
pub fn apply_angular_momentum_all(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        apply_angular_momentum(e, globals);
    }
}

/// Wrap entity position using 3x-resolution modulo (toroidal world)
pub fn wrap_entity(entity: &mut Entity, globals: &Globals) {
    entity.position = wrap_toroidal(entity.position, globals.phys_width, globals.phys_height);
}

/// Wrap all entities' positions (toroidal world)
pub fn wrap_entities(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        wrap_entity(e, globals);
    }
}


// --- Entity predicates ---

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

/// Check if entity is within visible screen area (with radius margin)
fn checkspawn_objet(entity: &Entity, globals: &Globals) -> bool {
    let x = entity.position.x;
    let y = entity.position.y;
    let rad = entity.hitbox.ext_radius;
    (x - rad < globals.phys_width) && (x + rad > 0.0)
        && (y - rad < globals.phys_height) && (y + rad > 0.0)
}

/// Transfer entities between on-screen and off-screen lists.
fn transfer_oos(
    onscreen: &mut Vec<Entity>,
    oos: &mut Vec<Entity>,
    globals: &Globals,
) {
    let mut going_out: Vec<Entity> = Vec::new();
    let mut staying_in: Vec<Entity> = Vec::new();
    for e in onscreen.drain(..) {
        if checkspawn_objet(&e, globals) {
            staying_in.push(e);
        } else {
            going_out.push(e);
        }
    }

    let mut coming_in: Vec<Entity> = Vec::new();
    let mut staying_out: Vec<Entity> = Vec::new();
    for e in oos.drain(..) {
        if checkspawn_objet(&e, globals) {
            coming_in.push(e);
        } else {
            staying_out.push(e);
        }
    }

    *onscreen = staying_in;
    onscreen.extend(coming_in);
    *oos = staying_out;
    oos.extend(going_out);
}

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

/// Remove dead entities, transfer chunk-sized asteroids to chunks list, and remove zero-radius debris.
/// Matches OCaml despawn: collects ischunk from all asteroid lists before filtering notchunk.
fn despawn(state: &mut GameState, globals: &Globals) {
    if globals.chunks_enabled {
        // Collect chunk-sized asteroids from all asteroid lists (OCaml: ischunk filter then append to ref_chunks)
        let new_from_objects      = drain_filter_stable(&mut state.objects,      ischunk);
        let new_from_objects_oos  = drain_filter_stable(&mut state.objects_oos,  ischunk);
        let new_from_toosmall     = drain_filter_stable(&mut state.toosmall,     ischunk);
        let new_from_toosmall_oos = drain_filter_stable(&mut state.toosmall_oos, ischunk);
        let new_from_fragments    = drain_filter_stable(&mut state.fragments,    ischunk);

        state.chunks.extend(new_from_objects);
        state.chunks.extend(new_from_objects_oos);
        state.chunks.extend(new_from_toosmall);
        state.chunks.extend(new_from_toosmall_oos);
        state.chunks.extend(new_from_fragments);
    } else {
        // Chunks disabled: discard any chunk-sized entities from asteroid lists
        state.objects.retain(|e| !ischunk(e));
        state.objects_oos.retain(|e| !ischunk(e));
        state.toosmall.retain(|e| !ischunk(e));
        state.toosmall_oos.retain(|e| !ischunk(e));
        state.fragments.retain(|e| !ischunk(e));
        // Also clear existing chunks lists
        state.chunks.clear();
        state.chunks_oos.clear();
    }

    // Now filter dead entities from asteroid lists (notchunk already removed above)
    state.objects.retain(is_alive);
    state.objects_oos.retain(is_alive);
    state.toosmall.retain(is_alive);
    state.toosmall_oos.retain(is_alive);
    state.fragments.retain(is_alive);

    // Remove zero/negative-radius debris
    state.chunks.retain(positive_radius);
    state.chunks_oos.retain(positive_radius);
    state.chunks_explo.retain(positive_radius);
}

/// Move a star by velocity scaled by its proximity (parallax)
pub fn move_star(star: &mut Star, velocity: Vec2, globals: &Globals) {
    star.last_pos = star.pos;
    let next = add_vec(star.pos, scale_vec(velocity, star.proximity));
    star.pos = wrap_single(next, globals.phys_width, globals.phys_height);
    // Avoid incorrect motion blur from screen-edge teleport
    if next.x > globals.phys_width || next.x < 0.0 || next.y > globals.phys_height || next.y < 0.0 {
        star.last_pos = star.pos;
    }
}

// ============================================================================
// Rendering functions — ported from ml/asteroids.ml
// ============================================================================

// polygon_to_cartesian and translate_polygon moved to crate::math_utils

/// Render a single polar polygon at a position with rotation and color
fn render_poly(
    poly: &[(f64, f64)],
    pos: Vec2,
    rotat: f64,
    color: [u8; 4],
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    let affine = polygon_to_cartesian(poly, rotat, globals.render_scale);
    let displaced = translate_polygon(&affine, pos);
    let screen_points: Vec<(i32, i32)> = displaced
        .iter()
        .map(|&p| dither_vec(p, DITHER_AA, globals.current_jitter_double))
        .collect();
    if globals.retro {
        renderer.draw_poly(&screen_points, [255, 255, 255, 255], 1.0);
    } else {
        renderer.fill_poly(&screen_points, color);
    }
}

/// Render all shape polygons of an entity's visuals
fn render_shapes(
    shapes: &[((f64, f64, f64), Polygon)],
    pos: Vec2,
    rotat: f64,
    exposure: f64,
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    for (col, Polygon(poly)) in shapes {
        let color = to_rgba(intensify(hdr(*col), exposure), globals);
        render_poly(poly, pos, rotat, color, renderer, globals);
    }
}

/// Render an entity: base circle (if not retro) + polygon shapes
pub fn render_visuals(
    entity: &Entity,
    offset: Vec2,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let visuals = &entity.visuals;
    let position = scale_vec(
        add_vec(
            add_vec(entity.position, globals.game_screenshake_pos),
            offset,
        ),
        globals.render_scale,
    );
    let exposure = globals.game_exposure * entity.hdr_exposure;

    // Base circle (not in retro mode)
    if visuals.radius > 0.0 && !globals.retro {
        let color = to_rgba(intensify(hdr(visuals.color), exposure), globals);
        let (x, y) = dither_vec(position, DITHER_AA, globals.current_jitter_double);
        let r = dither_radius(
            visuals.radius * globals.render_scale,
            DITHER_AA,
            DITHER_POWER_RADIUS,
            rng,
        );
        renderer.fill_circle(x as f64, y as f64, r.max(1) as f64, color);
    }

    // Polygon shapes on top
    render_shapes(
        &visuals.shapes,
        position,
        entity.orientation,
        exposure,
        renderer,
        globals,
    );
}

/// Render a chunk (small debris) — simpler than full entity rendering
fn render_chunk(
    entity: &Entity,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let pos = scale_vec(
        add_vec(entity.position, globals.game_screenshake_pos),
        globals.render_scale,
    );
    if globals.retro {
        let (x, y) = dither_vec(pos, DITHER_AA, globals.current_jitter_double);
        renderer.fill_circle(
            x as f64, y as f64,
            (0.25 * globals.render_scale * entity.visuals.radius).max(1.0),
            [128, 128, 128, 255],
        );
    } else {
        let intensity_chunk = 1.0;
        let color = to_rgba(
            intensify(hdr(entity.visuals.color), intensity_chunk * globals.game_exposure * entity.hdr_exposure),
            globals,
        );
        let (x, y) = dither_vec(pos, DITHER_AA, globals.current_jitter_double);
        let r = dither_radius(
            globals.render_scale * entity.visuals.radius,
            DITHER_AA, DITHER_POWER_RADIUS, rng,
        );
        renderer.fill_circle(x as f64, y as f64, r.max(1) as f64, color);
    }
}

/// Render a star with motion trail
pub fn render_star_trail(
    star: &Star,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let pos1 = scale_vec(
        add_vec(star.pos, globals.game_screenshake_pos),
        globals.render_scale,
    );
    let last_position = scale_vec(
        add_vec(star.last_pos, globals.game_screenshake_previous_pos),
        globals.render_scale,
    );
    let pos2 = lerp_vec(last_position, pos1, SHUTTER_SPEED);
    let (x1, y1) = dither_vec(pos1, DITHER_AA, globals.current_jitter_double);
    let (x2, y2) = dither_vec(pos2, DITHER_AA, globals.current_jitter_double);

    let lum = if globals.pause {
        star.lum + 0.5 * STAR_RAND_LUM
    } else {
        star.lum + rng.gen::<f64>() * STAR_RAND_LUM
    };

    let star_color_tmp = intensify(hdr(globals.star_color), lum * globals.game_exposure);

    if x1 == x2 && y1 == y2 {
        // Static star: render as a cross of pixels
        let center_color = to_rgba(
            intensify(
                hdr_add(
                    star_color_tmp,
                    hdr(globals.space_color),
                ),
                globals.game_exposure,
            ),
            globals,
        );
        renderer.plot(x1, y1, center_color);

        let arm_color = to_rgba(intensify(star_color_tmp, 0.25), globals);
        renderer.plot(x1 + 1, y1, arm_color);
        renderer.plot(x1 - 1, y1, arm_color);
        renderer.plot(x1, y1 + 1, arm_color);
        renderer.plot(x1, y1 - 1, arm_color);

        let diag_color = to_rgba(intensify(star_color_tmp, 0.125), globals);
        renderer.plot(x1 + 1, y1 + 1, diag_color);
        renderer.plot(x1 + 1, y1 - 1, diag_color);
        renderer.plot(x1 - 1, y1 + 1, diag_color);
        renderer.plot(x1 - 1, y1 - 1, diag_color);
    } else {
        // Moving star: render as a line trail
        let dist = magnitude(sub_vec(pos1, pos2));
        let trail_lum = (1.0 / (1.0 + dist)).sqrt();
        let trail_color = hdr_add(
            intensify(star_color_tmp, trail_lum),
            hdr_add(
                intensify(hdr(globals.space_color), globals.game_exposure),
                intensify(hdr(globals.add_color), globals.game_exposure),
            ),
        );
        let color = to_rgba(trail_color, globals);
        renderer.draw_line(x1, y1, x2, y2, color, 2.0);
    }
}

// ============================================================================
// Per-frame update
// ============================================================================

/// Update per-frame globals: jitter, game speed interpolation, exposure
pub fn update_frame(globals: &mut Globals, rng: &mut impl Rng) {
    // Jitter for dithering
    globals.current_jitter_double = Vec2::new(
        rng.gen::<f64>() * DITHER_POWER,
        rng.gen::<f64>() * DITHER_POWER,
    );

    if !globals.pause {
        let t0 = globals.time_last_frame;
        let t1 = globals.time_current_frame;

        // Game speed interpolation (real-time based, not game-time)
        globals.game_speed = globals.game_speed_target
            + abso_exp_decay(
                globals.game_speed - globals.game_speed_target,
                HALF_SPEED_CHANGE,
                t0,
                t1,
            );

        // Exposure interpolation
        globals.game_exposure = globals.game_exposure_target
            + abso_exp_decay(
                globals.game_exposure - globals.game_exposure_target,
                EXPOSURE_HALF_LIFE,
                t0,
                t1,
            );

        // Flash decay
        let flash_decay = abso_exp_decay(1.0, FLASHES_HALF_LIFE, t0, t1);
        globals.add_color = (
            globals.add_color.0 * flash_decay,
            globals.add_color.1 * flash_decay,
            globals.add_color.2 * flash_decay,
        );

        // Screenshake decay
        globals.game_screenshake =
            abso_exp_decay(globals.game_screenshake, SCREENSHAKE_HALF_LIFE, t0, t1);

        // Score shake decay
        globals.shake_score =
            abso_exp_decay(globals.shake_score, SHAKE_SCORE_HALF_LIFE, t0, t1);
        globals.game_screenshake_previous_pos = globals.game_screenshake_pos;
        if globals.screenshake_enabled {
            globals.game_screenshake_pos = scale_vec(
                Vec2::new(rng.gen::<f64>() * 2.0 - 1.0, rng.gen::<f64>() * 2.0 - 1.0),
                globals.game_screenshake,
            );
            // Smooth screenshake: blend toward previous position for a low-pass effect.
            // Matches OCaml: game_screenshake_pos := lerp_vec !game_screenshake_previous_pos !game_screenshake_pos screenshake_smoothness
            if SCREENSHAKE_SMOOTH {
                globals.game_screenshake_pos = lerp_vec(
                    globals.game_screenshake_previous_pos,
                    globals.game_screenshake_pos,
                    SCREENSHAKE_SMOOTHNESS,
                );
            }
        }

        // Color interpolation (dynamic color mode)
        if globals.dyn_color {
            let dt = t1 - t0;
            globals.mul_color = {
                let c = half_color(hdr(globals.mul_color), hdr(globals.mul_base), FILTER_HALF_LIFE, dt);
                (c.r, c.g, c.b)
            };
            globals.space_color = {
                let c = half_color(hdr(globals.space_color), hdr(globals.space_color_goal), SPACE_HALF_LIFE, dt);
                (c.r, c.g, c.b)
            };
            globals.star_color = {
                let c = half_color(hdr(globals.star_color), hdr(globals.star_color_goal), SPACE_HALF_LIFE, dt);
                (c.r, c.g, c.b)
            };
        }
    }

    // --- FPS counter (matches OCaml end-of-frame block) ---
    globals.time_current_count = globals.time_current_frame;
    globals.current_count += 1;
    if globals.time_current_count - globals.time_last_count > 1.0 {
        globals.last_count = globals.current_count;
        globals.current_count = 0;
        globals.time_last_count = globals.time_current_count;
    }
}

// damage, phys_damage, collision_* and consequences_collision* moved to crate::physics

fn get_entity(state: &GameState, entry: GridEntry) -> &Entity {
    match entry {
        GridEntry::Object(i)      => &state.objects[i],
        GridEntry::ObjectOos(i)   => &state.objects_oos[i],
        GridEntry::TooSmall(i)    => &state.toosmall[i],
        GridEntry::TooSmallOos(i) => &state.toosmall_oos[i],
        GridEntry::Fragment(i)    => &state.fragments[i],
        GridEntry::Ship           => &state.ship,
    }
}

fn get_entity_mut(state: &mut GameState, entry: GridEntry) -> &mut Entity {
    match entry {
        GridEntry::Object(i)      => &mut state.objects[i],
        GridEntry::ObjectOos(i)   => &mut state.objects_oos[i],
        GridEntry::TooSmall(i)    => &mut state.toosmall[i],
        GridEntry::TooSmallOos(i) => &mut state.toosmall_oos[i],
        GridEntry::Fragment(i)    => &mut state.fragments[i],
        GridEntry::Ship           => &mut state.ship,
    }
}

// consequences_collision and consequences_collision_frags moved to crate::physics::response

/// Collect all colliding (e1, e2) pairs from two grid cell lists.
/// Matches OCaml: iterates list1 × list2; for same list (tab1==tab2) this gives both directions.
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

/// Run collision detection between two grids, applying consequences.
/// `extend=true`: also check adjacent cells (right, down, diagonal).
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



/// Main game update: movement, transfers, collisions, spawning, despawn.
/// Called each frame when not paused.
pub fn update_game(state: &mut GameState, globals: &mut Globals) {
    // Update observer proper time (for time dilation)
    globals.observer_proper_time = state.ship.proper_time;

    // --- Smoke & chunk decay ---
    for s in state.smoke.iter_mut() { decay_smoke(s, globals); }
    for s in state.smoke_oos.iter_mut() { decay_smoke(s, globals); }

    // --- Decay chunks (radius shrink) ---
    // OCaml formula: radius -= observer_proper_time * game_speed * decay_rate * dt / chunk.proper_time
    {
        let dt = globals.dt();
        let gs = globals.game_speed;
        let opt = globals.observer_proper_time;
        for c in state.chunks.iter_mut() {
            c.visuals.radius -= opt * gs * CHUNK_RADIUS_DECAY * dt / c.proper_time;
        }
        for c in state.chunks_oos.iter_mut() {
            c.visuals.radius -= opt * gs * CHUNK_RADIUS_DECAY * dt / c.proper_time;
        }
        for c in state.chunks_explo.iter_mut() {
            c.visuals.radius -= opt * gs * CHUNK_EXPLO_RADIUS_DECAY * dt / c.proper_time;
        }
    }

    // Remove dead/negative-radius smoke
    state.smoke.retain(|s| s.visuals.radius > 0.0 && s.hdr_exposure > 0.001);
    state.smoke_oos.retain(|s| s.visuals.radius > 0.0 && s.hdr_exposure > 0.001);

    // --- Spawn explosions from dead projectiles ---
    // Previous explosions → smoke (before overwriting)
    if globals.smoke_enabled {
        state.smoke.append(&mut state.explosions);
    } else {
        state.explosions.clear();
    }

    // Spawn new explosions from dead projectiles (health < 0)
    let dead_projectile_explosions: Vec<Entity> = state.projectiles.iter()
        .filter(|p| p.health < 0.0)
        .map(|p| spawn_explosion(p, &mut state.rng))
        .collect();
    state.explosions.extend(dead_projectile_explosions);

    // Spawn explosions from dead asteroids/toosmall/fragments → add to smoke list
    {
        let dead_objects: Vec<Entity> = state.objects.iter().chain(state.objects_oos.iter())
            .chain(state.toosmall.iter()).chain(state.toosmall_oos.iter())
            .chain(state.fragments.iter())
            .filter(|e| is_dead(e))
            .cloned()
            .collect();
        for obj in &dead_objects {
            let (explo, side_effects) = spawn_explosion_object(
                obj,
                globals.flashes_enabled,
                globals.variable_exposure,
                FLASHES_SATURATE,
                FLASHES_EXPLOSION,
                FLASHES_NORMAL_MASS,
                &mut state.rng,
            );
            if let Some(ac) = side_effects.add_color {
                globals.add_color = (
                    globals.add_color.0 + ac.0,
                    globals.add_color.1 + ac.1,
                    globals.add_color.2 + ac.2,
                );
            }
            if let Some(em) = side_effects.exposure_multiplier {
                globals.game_exposure *= em;
            }
            state.smoke.push(explo);
        }
    }

    // Chunk explosions (chunks_explo → explosions)
    if !globals.pause {
        let explo_chunks: Vec<Entity> = state.chunks_explo.iter()
            .map(|c| {
                let (explo, se) = spawn_chunk_explosion(
                    c,
                    globals.flashes_enabled,
                    FLASHES_SATURATE,
                    FLASHES_EXPLOSION,
                    FLASHES_NORMAL_MASS,
                    &mut state.rng,
                );
                if let Some(ac) = se.add_color {
                    globals.add_color = (
                        globals.add_color.0 + ac.0,
                        globals.add_color.1 + ac.1,
                        globals.add_color.2 + ac.2,
                    );
                }
                if let Some(em) = se.exposure_multiplier {
                    globals.game_exposure *= em;
                }
                explo
            })
            .collect();
        state.explosions.extend(explo_chunks);
    }

    // game_speed slowdown per explosion
    let nb_explo = state.explosions.len();
    globals.game_speed *= RATIO_TIME_EXPLOSION.powi(nb_explo as i32);

    // --- Projectile inertia ---
    for p in state.projectiles.iter_mut() {
        apply_inertia(p, globals);
    }

    // --- Explosion inertia (one frame entities) ---
    for e in state.explosions.iter_mut() {
        apply_inertia(e, globals);
    }

    // --- Filter dead or OOS projectiles (projectiles don't wrap, they despawn) ---
    state.projectiles.retain(|p| {
        p.health >= 0.0 && {
            let x = p.position.x;
            let y = p.position.y;
            x >= -globals.phys_width && x <= 2.0 * globals.phys_width
                && y >= -globals.phys_height && y <= 2.0 * globals.phys_height
        }
    });

    // --- Cooldown tick ---
    if state.cooldown > 0.0 {
        state.cooldown -= globals.game_speed * globals.dt();
    }
    if state.cooldown_tp > 0.0 {
        state.cooldown_tp -= globals.game_speed * globals.dt();
    }

    // --- Inertia (position update) ---
    apply_inertia(&mut state.ship, globals);
    apply_inertia_all(&mut state.objects, globals);
    apply_inertia_all(&mut state.objects_oos, globals);
    apply_inertia_all(&mut state.toosmall, globals);
    apply_inertia_all(&mut state.toosmall_oos, globals);
    apply_inertia_all(&mut state.fragments, globals);
    apply_inertia_all(&mut state.chunks, globals);
    apply_inertia_all(&mut state.chunks_oos, globals);
    apply_inertia_all(&mut state.chunks_explo, globals);

    // --- Rotation (moment update) ---
    apply_angular_momentum(&mut state.ship, globals);
    apply_angular_momentum_all(&mut state.objects, globals);
    apply_angular_momentum_all(&mut state.objects_oos, globals);
    apply_angular_momentum_all(&mut state.toosmall, globals);
    apply_angular_momentum_all(&mut state.toosmall_oos, globals);
    apply_angular_momentum_all(&mut state.fragments, globals);

    // --- Size classification: move too-small asteroids ---
    let small_objs = drain_filter_stable(&mut state.objects, too_small);
    state.toosmall.extend(small_objs);
    let small_frags = drain_filter_stable(&mut state.fragments, too_small);
    state.toosmall.extend(small_frags);

    // --- OOS transfers ---
    transfer_oos(&mut state.objects, &mut state.objects_oos, globals);
    transfer_oos(&mut state.toosmall, &mut state.toosmall_oos, globals);
    transfer_oos(&mut state.chunks, &mut state.chunks_oos, globals);

    // === Collision grids ===
    let mut grid_objects  = make_grid();
    let mut grid_toosmall = make_grid();
    let mut grid_other    = make_grid();
    let mut grid_frag     = make_grid();

    let mut entries_obj: Vec<(GridEntry, Vec2)> = state.objects
        .iter().enumerate().map(|(i, e)| (GridEntry::Object(i), e.position)).collect();
    entries_obj.extend(state.objects_oos
        .iter().enumerate().map(|(i, e)| (GridEntry::ObjectOos(i), e.position)));

    let mut entries_small: Vec<(GridEntry, Vec2)> = state.toosmall
        .iter().enumerate().map(|(i, e)| (GridEntry::TooSmall(i), e.position)).collect();
    entries_small.extend(state.toosmall_oos
        .iter().enumerate().map(|(i, e)| (GridEntry::TooSmallOos(i), e.position)));

    // Note: explosions live exactly one frame; we include them in grid_other for collision
    // but they're not tracked by GridEntry index (they can't be mutated via get_entity_mut).
    // OCaml: other_ref = ship :: explosions @ projectiles — explosions damage via mass.
    // We handle explosion→asteroid damage separately below after grid collision.
    let entries_other: Vec<(GridEntry, Vec2)> = vec![
        (GridEntry::Ship, state.ship.position),
    ];

    let entries_frag: Vec<(GridEntry, Vec2)> = state.fragments
        .iter().enumerate().map(|(i, e)| (GridEntry::Fragment(i), e.position)).collect();

    insert_into_grid(&entries_obj,   &mut grid_objects,  globals);
    insert_into_grid(&entries_small, &mut grid_toosmall, globals);
    insert_into_grid(&entries_other, &mut grid_other,    globals);
    insert_into_grid(&entries_frag,  &mut grid_frag,     globals);

    // === Collision detection ===
    // Asteroid vs asteroid (extend=true)
    calculate_collision_tables(&grid_objects, &grid_objects, true, state, globals);
    // Asteroid vs toosmall (extend=false)
    calculate_collision_tables(&grid_objects.clone(), &grid_toosmall.clone(), false, state, globals);
    // Ship/other vs asteroid (extend=true)
    calculate_collision_tables(&grid_other.clone(), &grid_objects.clone(), true, state, globals);
    // Ship/other vs toosmall (extend=true)
    calculate_collision_tables(&grid_other.clone(), &grid_toosmall.clone(), true, state, globals);
    // Ship/other vs fragment (extend=true)
    calculate_collision_tables(&grid_other.clone(), &grid_frag.clone(), true, state, globals);

    // === Explosion damage to asteroids ===
    // Explosions are one-frame entities; we do a simple O(n*m) check here.
    for explo in &state.explosions {
        let explo_pos = explo.position;
        let explo_rad = explo.hitbox.ext_radius;
        let explo_mass = explo.mass;
        for obj in state.objects.iter_mut().chain(state.objects_oos.iter_mut())
            .chain(state.toosmall.iter_mut()).chain(state.toosmall_oos.iter_mut())
        {
            if collision_circles(explo_pos, explo_rad, obj.position, obj.hitbox.int_radius) {
                damage(obj, explo_mass, globals);
            }
        }
    }

    // === Projectile damage to asteroids + self-kill on hit ===
    for proj in state.projectiles.iter_mut() {
        let proj_pos = proj.position;
        let proj_rad = proj.hitbox.ext_radius;
        let mut hit = false;
        for obj in state.objects.iter_mut().chain(state.objects_oos.iter_mut())
            .chain(state.toosmall.iter_mut()).chain(state.toosmall_oos.iter_mut())
        {
            if collision_circles(proj_pos, proj_rad, obj.position, obj.hitbox.int_radius) {
                hit = true;
            }
        }
        if hit {
            proj.health = -1.0; // kill projectile
        }
    }

    // === Destroyed entity accounting (asteroids + fragments) ===
    let nb_destroyed = state.objects.iter().filter(|e| is_dead(e)).count()
        + state.objects_oos.iter().filter(|e| is_dead(e)).count()
        + state.toosmall.iter().filter(|e| is_dead(e)).count()
        + state.toosmall_oos.iter().filter(|e| is_dead(e)).count()
        + state.fragments.iter().filter(|e| is_dead(e)).count();
    globals.game_speed *= RATIO_TIME_DESTR_ASTEROID.powi(nb_destroyed as i32);
    state.score += nb_destroyed as i32;
    globals.shake_score += nb_destroyed as f64;

    // === Fragment vs fragment repulsion + promotion ===
    run_fragment_collisions(state, globals);

    // --- Fragmentation (spawn fragments from dead entities) ---
    spawn_fragments(&state.objects.clone(), &mut state.fragments, FRAGMENT_NUMBER, &mut state.rng);
    spawn_fragments(&state.toosmall.clone(), &mut state.fragments, FRAGMENT_NUMBER, &mut state.rng);
    spawn_fragments(&state.fragments.clone(), &mut state.fragments, FRAGMENT_NUMBER, &mut state.rng);

    // --- Move chunks out of fragments ---
    let new_chunks = drain_filter_stable(&mut state.fragments, ischunk);
    state.chunks.extend(new_chunks);

    // --- Recenter (wrap positions) ---
    wrap_entities(&mut state.objects, globals);
    wrap_entities(&mut state.toosmall, globals);
    wrap_entities(&mut state.objects_oos, globals);
    wrap_entities(&mut state.toosmall_oos, globals);
    wrap_entities(&mut state.fragments, globals);

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
                rand_range(RAND_MIN_LUM, RAND_MAX_LUM, &mut state.rng),
                rand_range(RAND_MIN_LUM, RAND_MAX_LUM, &mut state.rng),
                rand_range(RAND_MIN_LUM, RAND_MAX_LUM, &mut state.rng),
            );
            let new_hdr = hdr(new_col);
            globals.mul_base = {
                let c = saturate(intensify(new_hdr, 1.0), FILTER_SATURATION);
                (c.r, c.g, c.b)
            };
            globals.space_color_goal = {
                let c = saturate(intensify(new_hdr, 10.0), SPACE_SATURATION);
                (c.r, c.g, c.b)
            };
            globals.star_color_goal = {
                let c = saturate(intensify(new_hdr, 100.0), STAR_SATURATION);
                (c.r, c.g, c.b)
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
    despawn(state, globals);

    // --- Ship auto-regeneration ---
    if AUTOREGEN && state.ship.health > 0.0 && state.ship.health < SHIP_MAX_HEALTH {
        state.ship.health += AUTOREGEN_HEALTH * globals.game_speed * globals.dt();
        state.ship.health = state.ship.health.min(SHIP_MAX_HEALTH);
    }

    // --- Lagged health (orange bar) ---
    // Exponential lag: last_health chases ship.health (matches OCaml affiche_hud line 769)
    {
        let target = state.ship.health.max(0.0);
        state.last_health = target + exp_decay(
            state.last_health - target,
            0.5,
            globals.observer_proper_time,
            globals.game_speed,
            globals.time_last_frame,
            globals.time_current_frame,
            state.ship.proper_time,
        );
    }

    // --- Ship death handling ---
    // Step 1: Detect death entry (first time health < 0, not already in mort() phase)
    if state.ship.health < 0.0 && !state.is_dead {
        state.is_dead = true;
        globals.time_of_death = globals.time_current_frame;
        state.lives -= 1;

        // Chunk explosion at death
        if globals.chunks_enabled {
            let death_color = (1500.0, 400.0, 200.0);
            let new_chunks = spawn_n_chunks(
                &state.ship,
                NB_CHUNKS_EXPLO,
                death_color,
                &mut state.rng,
            );
            state.chunks_explo.extend(new_chunks);
        }

        // Death VFX: screenshake + big red flash + game speed slowdown
        globals.game_screenshake += SCREENSHAKE_DEATH;
        if globals.flashes_enabled {
            let death_flash = intensify(HdrColor::new(1000.0, 0.0, 0.0), FLASHES_DEATH);
            globals.add_color = (
                globals.add_color.0 + death_flash.r,
                globals.add_color.1 + death_flash.g,
                globals.add_color.2 + death_flash.b,
            );
        }
        globals.game_speed *= RATIO_TIME_DEATH;
        globals.game_speed_target = GAME_SPEED_TARGET_DEATH;
        globals.game_exposure_target = GAME_EXPOSURE_TARGET_DEATH;

        // Clamp health so death entry does not re-trigger
        state.ship.health = -0.1;
    }

    // Step 2: Per-frame death fire — spawn burning explosion while in mort() phase
    if state.is_dead {
        let elapsed = (globals.time_current_frame - globals.time_last_frame) * globals.game_speed;
        let death_explo = spawn_explosion_death(&state.ship, elapsed, &mut state.rng);
        state.explosions.push(death_explo);
    }

    // Step 3: End the death phase when timer expires or early-exit condition met
    if state.is_dead {
        let t = globals.time_current_frame;
        let tod = globals.time_of_death;
        let timer_expired = t > tod + TIME_STAY_DEAD_MAX;
        let early_exit = t > tod + TIME_STAY_DEAD_MIN && state.ship.health < -100.0;

        if timer_expired || early_exit {
            state.is_dead = false;

            if state.lives <= 0 {
                // Game over: reset and pause
                *state = GameState::new(globals);
                globals.pause = true;
                globals.game_speed_target = GAME_SPEED_TARGET_BOUCLE;
                globals.game_exposure_target = GAME_EXPOSURE_TARGET_BOUCLE;
            } else {
                // Second chunk burst
                if globals.chunks_enabled {
                    let death_color = (1500.0, 400.0, 200.0);
                    let new_chunks = spawn_n_chunks(
                        &state.ship,
                        NB_CHUNKS_EXPLO,
                        death_color,
                        &mut state.rng,
                    );
                    state.chunks_explo.extend(new_chunks);
                }

                // Second screenshake + red flash
                globals.game_screenshake += SCREENSHAKE_DEATH;
                if globals.flashes_enabled {
                    let death_flash = intensify(HdrColor::new(1000.0, 0.0, 0.0), FLASHES_DEATH);
                    globals.add_color = (
                        globals.add_color.0 + death_flash.r,
                        globals.add_color.1 + death_flash.g,
                        globals.add_color.2 + death_flash.b,
                    );
                }
                globals.game_speed *= RATIO_TIME_DEATH;

                // Respawn ship and restore normal targets
                state.ship = spawn_ship();
                globals.game_speed_target = GAME_SPEED_TARGET_BOUCLE;
                globals.game_exposure_target = GAME_EXPOSURE_TARGET_BOUCLE;
            }
        }
    }

    // --- Camera update: translate all entities to keep ship centred ---
    crate::camera::update_camera(state, globals);
}

// ============================================================================
// Projectile / Explosion / Smoke rendering
// ============================================================================

/// Render a light trail (motion blur line) for a fast-moving entity.
/// Used for projectiles. Ported from OCaml render_light_trail.
fn render_light_trail(
    radius: f64,
    pos: Vec2,
    velocity: Vec2,
    hdr_color: HdrColor,
    proper_time: f64,
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    let pos1 = scale_vec(add_vec(pos, globals.game_screenshake_pos), globals.render_scale);
    let dt_game = globals.game_speed
        * (globals.time_current_frame - globals.time_last_frame)
            .max(1.0 / FRAMERATE_RENDER);
    let veloc = scale_vec(velocity, -(globals.observer_proper_time / proper_time) * dt_game);
    let last_pos = scale_vec(
        add_vec(sub_vec(pos, veloc), globals.game_screenshake_previous_pos),
        globals.render_scale,
    );
    let pos2 = lerp_vec(last_pos, pos1, SHUTTER_SPEED);
    let dist = magnitude(sub_vec(pos1, pos2));
    let trail_lum = 0.5 * (radius / (radius + dist)).sqrt();
    let color = to_rgba(intensify(hdr_color, trail_lum), globals);
    let (x1, y1) = dither_vec(pos1, DITHER_AA, globals.current_jitter_double);
    let (x2, y2) = dither_vec(pos2, DITHER_AA, globals.current_jitter_double);
    let line_width = dither_radius(2.0 * radius, DITHER_AA, DITHER_POWER_RADIUS, &mut rand::thread_rng());
    renderer.draw_line(x1, y1, x2, y2, color, line_width.max(1) as f32);
}

/// Render a projectile as four concentric light trails. Ported from OCaml render_projectile.
fn render_projectile(entity: &Entity, renderer: &mut Renderer2D, globals: &Globals, rng: &mut impl Rng) {
    let rad = globals.render_scale
        * rand_range(0.5, 1.0, rng)
        * entity.visuals.radius;
    if globals.retro {
        // Retro mode: simple white filled circle at projectile position
        let pos = scale_vec(entity.position, globals.render_scale);
        let (x, y) = dither_vec(pos, DITHER_AA, globals.current_jitter_double);
        renderer.fill_circle(x as f64, y as f64, rad.max(1.0), [255, 255, 255, 255]);
    } else {
        let pos = entity.position;
        let vel = entity.velocity;
        let col = intensify(hdr(entity.visuals.color), entity.hdr_exposure * globals.game_exposure);
        let pt = entity.proper_time;
        render_light_trail(rad,        pos, vel, intensify(col, 0.25), pt, renderer, globals);
        render_light_trail(rad * 0.75, pos, vel, intensify(col, 0.5),  pt, renderer, globals);
        render_light_trail(rad * 0.5,  pos, vel, col,                  pt, renderer, globals);
        render_light_trail(rad * 0.25, pos, vel, intensify(col, 2.0),  pt, renderer, globals);
    }
}

/// Decay smoke radius and exposure (game-time based half-life).
/// Ported from OCaml decay_smoke.
pub fn decay_smoke(smoke: &mut Entity, globals: &Globals) {
    let dt_game = globals.game_speed * globals.dt();
    let half_r = SMOKE_HALF_RADIUS * smoke.proper_time;
    let half_c = SMOKE_HALF_COL * smoke.proper_time;
    // exp_decay: n * 2^(-(dt_game) / half_life)
    smoke.visuals.radius = smoke.visuals.radius * (2.0_f64).powf(-dt_game / half_r)
        - SMOKE_RADIUS_DECAY * dt_game * globals.observer_proper_time / smoke.proper_time;
    if smoke.hdr_exposure > 0.001 {
        smoke.hdr_exposure *= (2.0_f64).powf(-dt_game / half_c);
    }
}

// ============================================================================
// HUD — Vector font, health bars, hearts, score, stage, debug stats
// ============================================================================

/// Return the polygon (as list of relative coords in [0,1]x[0,1]) for a given character.
/// Each character is a single closed polyline matched exactly to the OCaml shape_char.
fn shape_char(c: char) -> Vec<(f64, f64)> {
    match c {
        '0' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.2),(0.25,0.2),(0.75,0.6),(0.75,0.8),(0.25,0.375),(0.25,0.8),(0.75,0.8),(0.75,0.2),(0.,0.2)],
        '1' => vec![(0.125,0.),(0.875,0.),(0.875,0.2),(0.625,0.2),(0.625,1.),(0.375,1.),(0.,0.75),(0.15,0.65),(0.375,0.8),(0.375,0.2),(0.125,0.2)],
        '2' => vec![(0.,0.),(1.,0.),(1.,0.2),(0.35,0.2),(1.,0.5),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.6),(0.25,0.6),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.,0.2)],
        '3' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.875,0.5),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.6),(0.25,0.6),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.5,0.6),(0.5,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.4),(0.,0.4),(0.,0.2)],
        '4' => vec![(0.5,0.),(0.75,0.),(0.75,1.),(0.5,1.),(0.,0.4),(0.,0.2),(1.,0.2),(1.,0.4),(0.25,0.4),(0.5,0.8)],
        '5' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.5),(0.25,0.7),(0.25,0.8),(1.,0.8),(1.,1.),(0.,1.),(0.,0.6),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.35),(0.,0.4),(0.,0.2),(0.25,0.)],
        '6' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.75,0.6),(0.25,0.6),(0.25,0.8),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.4),(0.,0.4),(0.,0.2)],
        '7' => vec![(0.25,0.),(0.5,0.),(1.,0.8),(1.,1.),(0.,1.),(0.,0.8),(0.75,0.8)],
        '8' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.875,0.5),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.25,0.6),(0.25,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,1.),(0.,0.8),(0.,0.6),(0.125,0.5),(0.,0.4),(0.,0.2)],
        '9' => vec![(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.6),(0.25,0.4),(0.75,0.4),(0.75,0.2),(0.,0.2),(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.6),(0.25,0.6),(0.25,0.8),(0.75,0.8),(0.75,0.6),(1.,0.6),(1.,0.8)],
        ' ' => vec![(0.,0.),(0.,0.),(0.,0.)],
        'A' => vec![(0.,0.),(0.25,0.),(0.25,0.4),(0.75,0.4),(0.75,0.4),(0.75,0.6),(0.25,0.6),(0.25,0.8),(0.75,0.8),(0.75,0.),(1.,0.),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8)],
        'B' => vec![(0.,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.875,0.5),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.25,0.6),(0.25,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.,1.)],
        'C' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.8),(0.75,0.8),(0.75,0.6),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.2)],
        'D' => vec![(0.,0.),(0.75,0.),(1.,0.2),(1.,0.8),(0.75,1.),(0.,1.),(0.,0.2),(0.25,0.2),(0.25,0.8),(0.75,0.8),(0.75,0.2),(0.,0.2)],
        'E' => vec![(0.,0.),(0.75,0.),(1.,0.2),(0.25,0.2),(0.25,0.4),(0.5,0.4),(0.5,0.6),(0.25,0.6),(0.25,0.8),(1.,0.8),(0.75,1.),(0.,1.)],
        'F' => vec![(0.,0.),(0.25,0.),(0.25,0.4),(0.5,0.4),(0.75,0.6),(0.25,0.6),(0.25,0.8),(1.,0.8),(1.,1.),(0.,1.)],
        'G' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.6),(0.5,0.6),(0.5,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.8),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.2)],
        'H' => vec![(0.,1.),(0.25,1.),(0.25,0.6),(0.75,0.6),(0.75,1.),(1.,1.),(1.,0.),(0.75,0.),(0.75,0.4),(0.25,0.4),(0.25,0.),(0.,0.)],
        'I' => vec![(0.125,0.),(0.875,0.),(0.875,0.2),(0.625,0.2),(0.625,0.8),(0.875,0.8),(0.875,1.),(0.125,1.),(0.125,0.8),(0.375,0.8),(0.375,0.2),(0.125,0.2)],
        'J' => vec![(0.25,1.),(0.5,1.),(0.75,0.8),(0.75,0.2),(1.,0.2),(1.,0.),(0.,0.),(0.,0.2),(0.25,0.2),(0.25,0.8),(0.,0.8)],
        'K' => vec![(0.,1.),(0.25,1.),(0.25,0.6),(0.75,1.),(1.,1.),(0.375,0.5),(1.,0.),(0.75,0.),(0.25,0.4),(0.25,0.),(0.,0.)],
        'L' => vec![(0.,0.),(0.,1.),(0.25,1.),(0.25,0.2),(1.,0.2),(1.,0.)],
        'M' => vec![(0.,1.),(0.25,1.),(0.5,0.6),(0.75,1.),(1.,1.),(1.,0.),(0.75,0.),(0.75,0.6),(0.5,0.2),(0.25,0.6),(0.25,0.),(0.,0.)],
        'N' => vec![(0.,1.),(0.25,1.),(0.75,0.4),(0.75,1.),(1.,1.),(1.,0.),(0.75,0.),(0.25,0.6),(0.25,0.),(0.,0.)],
        'O' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.2),(0.25,0.2),(0.25,0.8),(0.75,0.8),(0.75,0.2),(0.,0.2)],
        'P' => vec![(0.,0.),(0.25,0.),(0.25,0.5),(0.75,0.5),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.,1.)],
        'Q' => vec![(0.25,1.),(0.75,1.),(1.,0.8),(1.,0.2),(0.75,0.),(0.25,0.),(0.,0.2),(0.,0.8),(0.25,0.8),(0.25,0.2),(0.75,0.2),(0.75,0.8),(0.,0.8),(0.5,0.6),(1.,1.)],
        'R' => vec![(0.,0.),(0.25,0.),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.25,0.6),(0.25,0.4),(0.75,0.),(1.,0.),(0.5,0.4),(0.75,0.4),(1.,0.6),(1.,0.8),(0.75,1.),(0.,1.)],
        'S' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.75,0.6),(0.25,0.6),(0.25,0.8),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.6),(0.25,0.4),(0.75,0.4),(0.75,0.2),(0.,0.2)],
        'T' => vec![(0.385,0.),(0.625,0.),(0.625,0.8),(1.,0.8),(1.,1.),(0.,1.),(0.,0.8),(0.385,0.8)],
        'U' => vec![(0.,1.),(0.25,1.),(0.25,0.2),(0.75,0.2),(0.75,1.),(1.,1.),(1.,0.),(0.75,0.),(0.25,0.),(0.,0.)],
        'V' => vec![(0.,1.),(0.25,1.),(0.5,0.2),(0.75,1.),(1.,1.),(0.6,0.),(0.4,0.)],
        'W' => vec![(0.,1.),(0.2,0.),(0.4,0.),(0.5,0.2),(0.6,0.),(0.8,0.),(1.,1.),(0.6,0.4),(0.6,0.6),(0.4,0.6),(0.4,0.4),(0.2,1.)],
        'X' => vec![(0.,1.),(0.25,1.),(0.5,0.6),(0.75,1.),(1.,1.),(0.625,0.5),(1.,0.),(0.75,0.),(0.5,0.4),(0.25,0.),(0.,0.),(0.375,0.5)],
        'Y' => vec![(0.,0.),(0.25,0.),(0.5,0.4),(0.75,0.),(1.,0.),(0.625,0.6),(0.625,1.),(0.375,1.),(0.375,0.6)],
        'Z' => vec![(0.,1.),(1.,1.),(1.,0.8),(0.25,0.2),(1.,0.2),(0.75,0.),(0.,0.),(0.,0.2),(0.75,0.8),(0.,0.8)],
        ':' => vec![(0.3,0.8),(0.7,0.8),(0.7,0.6),(0.3,0.6),(0.3,0.4),(0.7,0.4),(0.7,0.2),(0.3,0.2),(0.3,0.4),(0.7,0.4)],
        '-' => vec![(0.1,0.6),(0.9,0.6),(0.9,0.4),(0.1,0.4)],
        '.' => vec![(0.3,1.),(0.7,1.),(0.7,0.8),(0.3,0.8)],
        '!' => vec![(0.35,1.),(0.65,1.),(0.65,0.8),(0.35,0.8),(0.35,0.65),(0.65,0.65),(0.65,0.),(0.35,0.)],
        _ => vec![(0.,0.),(1.,0.),(1.,1.),(0.,1.)],
    }
}

/// Map a relative coordinate (relx, rely) inside a bounding quad to screen pixels.
/// Matches OCaml `displacement`: bilinear interpolation across the 4 bounding points,
/// then multiply by render_scale.
/// Points: [p0=bottom-left, p1=bottom-right, p2=top-right, p3=top-left] (physical coords)
///
/// OCaml formula: lerp_vec a b ratio = a*ratio + b*(1-ratio)
/// top = lerp_vec(p2, p1, rely) = p2*rely + p1*(1-rely)
/// bot = lerp_vec(p3, p0, rely) = p3*rely + p0*(1-rely)
/// result = lerp_vec(top, bot, relx) = top*relx + bot*(1-relx)
fn displacement(
    encadrement: &[(f64, f64); 4],
    rel: (f64, f64),
    render_scale: f64,
) -> (f64, f64) {
    let (relx, rely) = rel;
    let [p0, p1, p2, p3] = encadrement;
    // lerp_vec a b ratio = a*ratio + b*(1-ratio)
    let top = (
        p2.0 * rely + p1.0 * (1.0 - rely),
        p2.1 * rely + p1.1 * (1.0 - rely),
    );
    let bot = (
        p3.0 * rely + p0.0 * (1.0 - rely),
        p3.1 * rely + p0.1 * (1.0 - rely),
    );
    let interp = (
        top.0 * relx + bot.0 * (1.0 - relx),
        top.1 * relx + bot.1 * (1.0 - relx),
    );
    (interp.0 * render_scale, interp.1 * render_scale)
}

/// Convert shape relative coords to screen pixel coords for a given bounding quad.
fn displace_shape(
    encadrement: &[(f64, f64); 4],
    shape: &[(f64, f64)],
    render_scale: f64,
) -> Vec<(i32, i32)> {
    shape
        .iter()
        .map(|&pt| {
            let (x, y) = displacement(encadrement, pt, render_scale);
            (x.round() as i32, y.round() as i32)
        })
        .collect()
}

/// Render a single character at the given bounding quad (physical coords), filled.
fn render_char(
    encadrement: &[(f64, f64); 4],
    c: char,
    color: [u8; 4],
    renderer: &mut Renderer2D,
    render_scale: f64,
) {
    let shape = shape_char(c);
    let pts = displace_shape(encadrement, &shape, render_scale);
    renderer.fill_poly(&pts, color);
}

/// Render a string of characters, each in a bounding quad advancing left to right.
/// Matches OCaml `render_characs` / `render_string`.
/// - pos: bottom-left start in physical coords
/// - l_char: char width in physical units
/// - h_char: char height in physical units
/// - l_space: spacing between chars in physical units
/// - shake: random position shake amplitude
pub(crate) fn render_string(
    s: &str,
    pos: (f64, f64),
    l_char: f64,
    h_char: f64,
    l_space: f64,
    shake: f64,
    color: [u8; 4],
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let mut x0 = pos.0;
    let y0 = pos.1;
    for c in s.chars() {
        let c = c.to_ascii_uppercase();
        let sx0 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sy0 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sx1 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sy1 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sx2 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sy2 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sx3 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sy3 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        // Bounding quad: [bottom-left, bottom-right, top-right, top-left]
        let encadrement: [(f64, f64); 4] = [
            (x0 + sx0, y0 + sy0),
            (x0 + sx1 + l_char, y0 + sy1),
            (x0 + sx2 + l_char, y0 + sy2 + h_char),
            (x0 + sx3, y0 + sy3 + h_char),
        ];
        render_char(&encadrement, c, color, renderer, globals.render_scale);
        x0 += l_char + l_space;
    }
}

/// Fill a quadrilateral bar from 0 (empty) to ratio (full).
/// The quad is given as [p0, p1, p2, p3] in physical ratio coords [0,1],
/// where p0,p1 are the "zero" side and p2,p3 are the "full" side.
/// Matches OCaml `render_bar`.
fn render_bar(
    ratio: f64,
    quad: &[(f64, f64); 4],
    color: [u8; 4],
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    // relative_poly converts [0,1] coords to pixels: multiply by (width, height)
    // p0=quad[0], p1=quad[1], p2=quad[2], p3=quad[3]
    // For bar: use points p0, p1, lerp_vec(p2,p1,ratio), lerp_vec(p3,p0,ratio)
    let p0 = (quad[0].0 * globals.phys_width * globals.render_scale,
              quad[0].1 * globals.phys_height * globals.render_scale);
    let p1 = (quad[1].0 * globals.phys_width * globals.render_scale,
              quad[1].1 * globals.phys_height * globals.render_scale);
    let p2_full = (quad[2].0 * globals.phys_width * globals.render_scale,
                   quad[2].1 * globals.phys_height * globals.render_scale);
    let p3_full = (quad[3].0 * globals.phys_width * globals.render_scale,
                   quad[3].1 * globals.phys_height * globals.render_scale);

    // OCaml: lerp_vec p2 p1 ratio = p2*ratio + p1*(1-ratio)
    // ratio=1 → p2_full (full side) → full bar
    // ratio=0 → p1/p0 (zero side) → empty bar
    let p2 = (
        p2_full.0 * ratio + p1.0 * (1.0 - ratio),
        p2_full.1 * ratio + p1.1 * (1.0 - ratio),
    );
    let p3 = (
        p3_full.0 * ratio + p0.0 * (1.0 - ratio),
        p3_full.1 * ratio + p0.1 * (1.0 - ratio),
    );

    let pts: Vec<(i32, i32)> = vec![
        (p0.0.round() as i32, p0.1.round() as i32),
        (p1.0.round() as i32, p1.1.round() as i32),
        (p2.0.round() as i32, p2.1.round() as i32),
        (p3.0.round() as i32, p3.1.round() as i32),
    ];
    renderer.fill_poly(&pts, color);
}

/// Draw a heart shape: two ellipses + a diamond polygon.
/// Matches OCaml `draw_heart`.
/// pos0, pos1 are bounding box corners in physical coords.
fn draw_heart(
    pos0: (f64, f64),
    pos1: (f64, f64),
    color: [u8; 4],
    renderer: &mut Renderer2D,
    render_scale: f64,
) {
    // Scale to pixels
    let x0 = pos0.0 * render_scale;
    let y0 = pos0.1 * render_scale;
    let x1 = pos1.0 * render_scale;
    let y1 = pos1.1 * render_scale;

    let quartx = (x1 - x0) / 4.0;
    let tiery  = (y1 - y0) / 3.0;

    // Left ellipse center: (x0 + quartx, y1 - tiery)
    let lcx = (x0 + quartx + 0.5) as i32;
    let lcy = (y1 - tiery) as i32;
    let rx  = (quartx + 0.5) as i32;
    let ry  = (tiery  + 0.5) as i32;
    renderer.fill_ellipse(lcx, lcy, rx, ry, color);

    // Right ellipse center: (x1 - quartx, y1 - tiery)
    let rcx = (x1 - quartx + 0.5) as i32;
    renderer.fill_ellipse(rcx, lcy, rx, ry, color);

    // Diamond bottom polygon (matches OCaml fill_poly)
    let decal = 1.0 - (1.0 / 2.0_f64.sqrt());
    let pts: Vec<(i32, i32)> = vec![
        ((x0 + 2.0*quartx) as i32,                          y0 as i32),
        ((x0 + decal*quartx + 0.5) as i32,       (y0 + (1.0+decal)*tiery) as i32),
        ((x0 + 2.0*quartx) as i32,               (y1 - tiery) as i32),
        ((x1 - decal*quartx - 0.5) as i32,       (y0 + (1.0+decal)*tiery) as i32),
    ];
    renderer.fill_poly(&pts, color);
}

/// Render `n` hearts for the lives display. Matches OCaml `draw_n_hearts`.
fn draw_n_hearts(n: i32, color: [u8; 4], renderer: &mut Renderer2D, globals: &Globals) {
    let sx = globals.safe_offset_x;
    let sy = globals.safe_offset_y;
    let sw = globals.safe_phys_width;
    let sh = globals.safe_phys_height;
    let mut lastx = sx + 0.95 * sw;
    for _ in 0..n {
        draw_heart(
            (lastx - 0.03 * sw, sy + 0.75 * sh),
            (lastx,              sy + 0.80 * sh),
            color,
            renderer,
            globals.render_scale,
        );
        lastx -= 0.05 * sw;
    }
}

/// Draw a bar outline (quadrilateral frame) using draw_poly.
fn draw_bar_frame(
    quad: &[(f64, f64); 4],
    color: [u8; 4],
    line_width: f32,
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    let pts: Vec<(i32, i32)> = quad.iter().map(|&(rx, ry)| {
        (
            (rx * globals.phys_width  * globals.render_scale).round() as i32,
            (ry * globals.phys_height * globals.render_scale).round() as i32,
        )
    }).collect();
    renderer.draw_poly(&pts, color, line_width);
}

/// Render the full HUD. Matches OCaml `affiche_hud`.
/// Called at the END of render_frame (on top of everything).
pub fn render_hud(
    state: &GameState,
    globals: &Globals,
    renderer: &mut Renderer2D,
    rng: &mut impl Rng,
) {
    // Skip HUD in retro mode
    if globals.retro {
        return;
    }

    // ----- Colors -----
    let red   : [u8; 4] = [255,  32,  32, 255];
    let orange: [u8; 4] = [255, 128,   0, 255];
    let dark_red: [u8; 4] = [32, 0, 0, 255];
    let cyan  : [u8; 4] = [  0, 192, 255, 255];
    let dark_blue: [u8; 4] = [0, 0, 32, 255];
    let yellow: [u8; 4] = [255, 220,  50, 255];
    let dark_yellow: [u8; 4] = [32, 16, 0, 255];
    let white : [u8; 4] = [255, 255, 255, 255];
    let frame_color: [u8; 4] = [64, 64, 64, 255];
    let frame_width: f32 = 10.0 * globals.render_scale as f32;

    // ----- Safe zone for HUD placement -----
    let sx = globals.safe_offset_x;
    let sy = globals.safe_offset_y;
    let sw = globals.safe_phys_width;
    let sh = globals.safe_phys_height;
    let pw = globals.phys_width;
    let ph = globals.phys_height;

    // Helper: convert safe-zone-relative fraction to full-screen fraction for render_bar
    // render_bar quads use fractions of phys_width/phys_height
    let fx = |frac: f64| -> f64 { (sx + frac * sw) / pw };
    let fy = |frac: f64| -> f64 { (sy + frac * sh) / ph };

    // ----- Hearts (lives) -----
    draw_n_hearts(state.lives, red, renderer, globals);

    // ----- Health bar -----
    // last_health tracks delayed (smooth) health
    let health_quad: [(f64, f64); 4] = [
        (fx(0.95), fy(0.9)),  (fx(0.95), fy(0.85)),
        (fx(0.6),  fy(0.85)), (fx(0.55), fy(0.9)),
    ];
    render_bar(1.0, &health_quad, dark_red,  renderer, globals);
    render_bar(
        (state.last_health / SHIP_MAX_HEALTH).min(1.0).max(0.0),
        &health_quad, orange, renderer, globals,
    );
    render_bar(
        (state.ship.health / SHIP_MAX_HEALTH).min(1.0).max(0.0),
        &health_quad, red, renderer, globals,
    );
    draw_bar_frame(&health_quad, frame_color, frame_width, renderer, globals);

    // ----- Teleport cooldown bar -----
    let tp_quad: [(f64, f64); 4] = [
        (fx(0.95), fy(0.7)),  (fx(0.95), fy(0.65)),
        (fx(0.8),  fy(0.65)), (fx(0.75), fy(0.7)),
    ];
    let tp_ratio = ((COOLDOWN_TP - state.cooldown_tp.max(0.0)) / COOLDOWN_TP).min(1.0).max(0.0);
    render_bar(1.0, &tp_quad, dark_blue, renderer, globals);
    render_bar(tp_ratio, &tp_quad, cyan, renderer, globals);
    draw_bar_frame(&tp_quad, frame_color, frame_width, renderer, globals);

    // Render 'F' indicator when teleport ready
    if state.cooldown_tp <= 0.0 {
        let encadrement: [(f64, f64); 4] = [
            (sx + 0.7  * sw, sy + 0.65 * sh),
            (sx + 0.72 * sw, sy + 0.65 * sh),
            (sx + 0.72 * sw, sy + 0.7  * sh),
            (sx + 0.7  * sw, sy + 0.7  * sh),
        ];
        render_char(&encadrement, 'F', cyan, renderer, globals.render_scale);
    }

    // ----- Weapon cooldown bar -----
    let weapon_quad: [(f64, f64); 4] = [
        (fx(0.95), fy(0.6)),  (fx(0.95), fy(0.55)),
        (fx(0.9),  fy(0.55)), (fx(0.85), fy(0.6)),
    ];
    let weapon_ratio = ((globals.projectile_cooldown - state.cooldown.max(0.0)) / globals.projectile_cooldown)
        .min(1.0)
        .max(0.0);
    render_bar(1.0, &weapon_quad, dark_yellow, renderer, globals);
    render_bar(weapon_ratio, &weapon_quad, yellow, renderer, globals);
    draw_bar_frame(&weapon_quad, frame_color, frame_width, renderer, globals);

    // ----- Score -----
    // Color: warm amber/orange, dimmed by shake_score
    let score_intensity = 1.0 / (1.0 + 10.0 * globals.shake_score);
    let score_col = rgb_of_hdr(
        intensify(HdrColor::new(50000.0, 1000.0, 300.0), score_intensity),
        &HdrColor::zero(),
        &HdrColor::one(),
        1.0,
    );
    let score_str = format!("SCORE {}", state.score);
    let shake = globals.shake_score * 7.0;
    let base_l_char = (1.0 + 0.05 * globals.shake_score) * 0.03 * sw;
    let base_h_char = (1.0 + 0.05 * globals.shake_score) * 0.08 * sh;
    let base_l_space = (1.0 + 0.05 * globals.shake_score) * 0.01 * sw;
    let score_y = sy + 0.82 * sh * (1.0 - 0.05 * globals.shake_score * 0.08);
    render_string(
        &score_str,
        (sx + 0.02 * sw, score_y),
        base_l_char,
        base_h_char,
        base_l_space,
        shake,
        score_col,
        renderer,
        globals,
        rng,
    );

    // ----- Stage -----
    let stage_str = format!("STAGE {}", state.stage);
    render_string(
        &stage_str,
        (sx + 0.02 * sw, sy + 0.7 * sh),
        0.02 * sw,
        0.05 * sh,
        0.01 * sw,
        0.0,
        white,
        renderer,
        globals,
        rng,
    );

    // ----- Death countdown -----
    // Show countdown when ship health <= 0
    if state.ship.health <= 0.0 {
        let time_until_explo = globals.time_of_death + TIME_STAY_DEAD_MAX - globals.time_current_frame;
        if time_until_explo > 0.0 {
            // Flash: show the integer countdown, alternating on/off at 0.5s boundary
            let frac = time_until_explo - time_until_explo.floor();
            if frac > 0.5 {
                let count_str = format!("{}", (time_until_explo + 1.0) as i32);
                render_string(
                    &count_str,
                    (sx + 0.42 * sw, sy + 0.3 * sh),
                    0.16 * sw,
                    0.4  * sh,
                    0.01 * sw,
                    0.0,
                    white,
                    renderer,
                    globals,
                    rng,
                );
            }
        }
    }

    // ----- Debug stats (half size) -----
    let debug_x = sx + 0.01 * sw;
    let debug_l = 0.006 * sw;
    let debug_h = 0.0125 * sh;
    let debug_sp = 0.0015 * sw;
    let debug_color = white;

    let nb_objets   = state.objects.len()   + state.objects_oos.len();
    let nb_toosmall = state.toosmall.len()  + state.toosmall_oos.len();
    let nb_frags    = state.fragments.len();
    let nb_projs    = state.projectiles.len();
    let nb_explos   = state.explosions.len();
    let nb_smoke    = state.smoke.len()     + state.smoke_oos.len();
    let nb_chunks   = state.chunks.len()    + state.chunks_oos.len();
    let nb_chunks_e = state.chunks_explo.len();

    let fps = if globals.time_current_count - globals.time_last_count > 0.0 {
        (globals.last_count as f64).round() as i32
    } else {
        0
    };

    let peak_fps = if globals.frame_compute_secs > 0.0 {
        (1.0 / globals.frame_compute_secs).round() as i32
    } else {
        0
    };

    let debug_lines = [
        format!("FPS        : {}", fps),
        format!("Peak FPS   : {}", peak_fps),
        format!("Objets     : {}", nb_objets),
        format!("TooSmall   : {}", nb_toosmall),
        format!("Frags      : {}", nb_frags),
        format!("Projectiles: {}", nb_projs),
        format!("Explosions : {}", nb_explos),
        format!("Smoke      : {}", nb_smoke),
        format!("Chunks     : {}", nb_chunks),
        format!("ChunksExplo: {}", nb_chunks_e),
    ];

    for (i, line) in debug_lines.iter().enumerate() {
        let y = (debug_h + debug_sp) * i as f64 + debug_sp;
        render_string(
            line,
            (debug_x, y),
            debug_l,
            debug_h,
            debug_sp * 0.5,
            0.0,
            debug_color,
            renderer,
            globals,
            rng,
        );
    }
}

/// Render scanlines effect: draw horizontal black lines every SCANLINES_PERIOD pixels
/// starting at `offset`, across the full screen width.
/// Imitates old CRT monitors that projected the image line by line.
fn render_scanlines(offset: i32, height: i32, renderer: &mut Renderer2D) {
    let width = renderer.width as i32;
    let mut y = offset;
    while y < height {
        renderer.fill_rect(0, y, width, 1, [0, 0, 0, 255]);
        y += SCANLINES_PERIOD;
    }
}

/// Render a complete frame: background, stars, chunks, asteroids, ship
pub fn render_frame(
    state: &mut GameState,
    globals: &mut Globals,
    renderer: &mut Renderer2D,
    mouse_sx: f64,
    mouse_sy: f64,
    mouse_down: bool,
) {
    let (w, h) = (renderer.width as i32, renderer.height as i32);

    // Background
    if globals.retro {
        renderer.fill_rect(0, 0, w, h, [0, 0, 0, 255]);
    } else {
        let bg_color = to_rgba(
            intensify(hdr(globals.space_color), globals.game_exposure),
            globals,
        );
        renderer.fill_rect(0, 0, w, h, bg_color);
    }

    // Stars (not in retro mode)
    if !globals.retro {
        for star in &state.stars {
            render_star_trail(star, renderer, globals, &mut state.rng);
        }
    }

    // Smoke — before ship (OCaml order)
    for s in &state.smoke {
        render_visuals(s, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // Chunks
    for chunk in &state.chunks {
        render_chunk(chunk, renderer, globals, &mut state.rng);
    }

    // Projectiles — before ship (OCaml order)
    for p in &state.projectiles {
        render_projectile(p, renderer, globals, &mut state.rng);
    }

    // Ship
    render_visuals(&state.ship, Vec2::ZERO, renderer, globals, &mut state.rng);

    // Fragments — after ship (OCaml order)
    for entity in &state.fragments {
        render_visuals(entity, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // Toosmall — after ship (OCaml order)
    for entity in &state.toosmall {
        render_visuals(entity, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // Asteroids — after ship (OCaml order)
    for entity in &state.objects {
        render_visuals(entity, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // Explosions — after asteroids (OCaml order)
    for e in &state.explosions {
        render_visuals(e, Vec2::ZERO, renderer, globals, &mut state.rng);
    }

    // HUD overlay — skip when paused (matches OCaml behavior)
    if !globals.pause {
        // Extract rng to avoid simultaneous borrow of state fields
        let rng = &mut state.rng as *mut _;
        render_hud(state, globals, renderer, unsafe { &mut *rng });
    }

    // Pause title overlay + interactive buttons
    if globals.pause {
        crate::pause_menu::render_pause_title(state, globals, renderer, mouse_sx, mouse_sy, mouse_down);
    }

    // Scanlines effect (rendered last, on top of everything)
    if globals.scanlines {
        render_scanlines(globals.scanlines_offset, h, renderer);
        // Advance animation offset each frame
        if ANIMATED_SCANLINES {
            globals.scanlines_offset = (globals.scanlines_offset + 1) % SCANLINES_PERIOD;
        }
    }
}
