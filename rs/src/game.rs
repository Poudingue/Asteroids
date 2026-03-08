use std::f64::consts::PI;

use rand::prelude::*;

use crate::color::*;
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;
use crate::renderer::Renderer2D;

// ============================================================================
// Collision grid
// ============================================================================

/// Identifies an entity by which list it lives in and its index.
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
}

impl GameState {
    pub fn new(globals: &Globals) -> Self {
        let mut rng = thread_rng();
        let mut ship = spawn_ship();
        ship.position = (globals.phys_width / 2.0, globals.phys_height / 2.0);

        Self {
            score: 0,
            lives: SHIP_MAX_LIVES,
            stage: 0,
            cooldown: 0.0,
            cooldown_tp: 0.0,
            last_health: SHIP_MAX_HEALTH,
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
            stars: n_stars(
                globals.stars_nb,
                globals.phys_width,
                globals.phys_height,
                &mut rng,
            ),
            rng,
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
pub fn deplac_objet(entity: &mut Entity, vel: Vec2, globals: &Globals) {
    let time_factor = globals.dt() * globals.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.position = proj(entity.position, vel, time_factor);
}

/// Apply an object's velocity as displacement (inertia)
pub fn inertie_objet(entity: &mut Entity, globals: &Globals) {
    let vel = entity.velocity;
    deplac_objet(entity, vel, globals);
}

/// Accelerate an object (velocity += accel * dt * ...)
pub fn accel_objet(entity: &mut Entity, accel: Vec2, globals: &Globals) {
    let time_factor = globals.dt() * globals.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.velocity = proj(entity.velocity, accel, time_factor);
}

/// Instant velocity change (no time scaling)
pub fn boost_objet(entity: &mut Entity, boost: Vec2) {
    entity.velocity = proj(entity.velocity, boost, 1.0);
}

/// Timed rotation (orientation += rotation * dt * ...)
pub fn rotat_objet(entity: &mut Entity, rotation: f64, globals: &Globals) {
    let time_factor = globals.dt() * globals.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.orientation += rotation * time_factor;
}

/// Instant rotation (no time scaling)
pub fn tourn_objet(entity: &mut Entity, rotation: f64) {
    entity.orientation += rotation;
}

/// Angular acceleration (moment += momentum * dt * ...)
pub fn couple_objet(entity: &mut Entity, momentum: f64, globals: &Globals) {
    let time_factor = globals.dt() * globals.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.moment += momentum * time_factor;
}

/// Instant angular momentum change
pub fn couple_objet_boost(entity: &mut Entity, momentum: f64) {
    entity.moment += momentum;
}

/// Apply moment as rotation (rotational inertia)
pub fn moment_objet(entity: &mut Entity, globals: &Globals) {
    let moment = entity.moment;
    rotat_objet(entity, moment, globals);
}

/// Instant absolute displacement (for camera movement)
pub fn deplac_objet_abso(entity: &mut Entity, velocity: Vec2) {
    entity.position = proj(entity.position, velocity, 1.0);
}

/// Apply inertia to all entities in a list
pub fn inertie_objets(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        inertie_objet(e, globals);
    }
}

/// Apply angular momentum to all entities in a list
pub fn moment_objets(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        moment_objet(e, globals);
    }
}

/// Wrap entity position using 3x-resolution modulo (toroidal world)
pub fn recenter_objet(entity: &mut Entity, globals: &Globals) {
    entity.position = modulo_3reso(entity.position, globals.phys_width, globals.phys_height);
}

/// Wrap all entities' positions (toroidal world)
pub fn recenter_objets(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        recenter_objet(e, globals);
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
    let (x, y) = entity.position;
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

/// Remove dead entities, chunks-in-wrong-list, and zero-radius debris
fn despawn(state: &mut GameState) {
    state.objects.retain(|e| is_alive(e) && notchunk(e));
    state.objects_oos.retain(|e| is_alive(e) && notchunk(e));
    state.toosmall.retain(|e| is_alive(e) && notchunk(e));
    state.toosmall_oos.retain(|e| is_alive(e) && notchunk(e));
    state.fragments.retain(|e| is_alive(e) && notchunk(e));
    state.chunks.retain(positive_radius);
    state.chunks_oos.retain(positive_radius);
    state.chunks_explo.retain(positive_radius);
}

/// Move a star by velocity scaled by its proximity (parallax)
pub fn deplac_star(star: &mut Star, velocity: Vec2, globals: &Globals) {
    star.last_pos = star.pos;
    let next = addtuple(star.pos, multuple(velocity, star.proximity));
    star.pos = modulo_reso(next, globals.phys_width, globals.phys_height);
    // Avoid incorrect motion blur from screen-edge teleport
    if next.0 > globals.phys_width || next.0 < 0.0 || next.1 > globals.phys_height || next.1 < 0.0 {
        star.last_pos = star.pos;
    }
}

// ============================================================================
// Input handlers
// ============================================================================

/// Aim the ship at the mouse position (screen coords → phys coords → atan2)
pub fn aim_at_mouse(ship: &mut Entity, mouse_x: i32, mouse_y: i32, globals: &Globals) {
        // Flip SDL2 Y-down to renderer Y-up coordinates
    let mouse_phys = (
        mouse_x as f64 / globals.ratio_rendu,
        (HEIGHT as f64 - mouse_y as f64) / globals.ratio_rendu,
    );
    let (theta, _) = affine_to_polar(soustuple(mouse_phys, ship.position));
    ship.orientation = theta;
}

/// Forward acceleration (continuous, time-scaled)
pub fn acceleration(ship: &mut Entity, globals: &Globals) {
    accel_objet(
        ship,
        polar_to_affine(ship.orientation, SHIP_MAX_ACCEL),
        globals,
    );
}

/// Forward boost (impulse, instant velocity change)
pub fn boost_forward(ship: &mut Entity) {
    let orientation = ship.orientation;
    boost_objet(ship, polar_to_affine(orientation, SHIP_MAX_BOOST));
}

/// Rotate left — impulse or continuous depending on globals
pub fn handle_left(ship: &mut Entity, globals: &Globals) {
    if globals.ship_impulse_pos {
        if globals.ship_direct_rotat {
            tourn_objet(ship, SHIP_MAX_ROTAT);
        } else {
            couple_objet_boost(ship, SHIP_MAX_TOURN_BOOST);
        }
    } else if globals.ship_direct_rotat {
        rotat_objet(ship, SHIP_MAX_TOURN, globals);
    } else {
        couple_objet(ship, SHIP_MAX_TOURN, globals);
    }
}

/// Rotate right — impulse or continuous depending on globals
pub fn handle_right(ship: &mut Entity, globals: &Globals) {
    if globals.ship_impulse_pos {
        if globals.ship_direct_rotat {
            tourn_objet(ship, -SHIP_MAX_ROTAT);
        } else {
            couple_objet_boost(ship, -SHIP_MAX_TOURN_BOOST);
        }
    } else if globals.ship_direct_rotat {
        rotat_objet(ship, -SHIP_MAX_TOURN, globals);
    } else {
        couple_objet(ship, -SHIP_MAX_TOURN, globals);
    }
}

/// Strafe left (always impulse boost perpendicular to heading)
pub fn strafe_left(ship: &mut Entity) {
    let orientation = ship.orientation + PI / 2.0;
    boost_objet(ship, polar_to_affine(orientation, SHIP_MAX_BOOST));
}

/// Strafe right (always impulse boost perpendicular to heading)
pub fn strafe_right(ship: &mut Entity) {
    let orientation = ship.orientation - PI / 2.0;
    boost_objet(ship, polar_to_affine(orientation, SHIP_MAX_BOOST));
}

// ============================================================================
// Rendering functions — ported from ml/asteroids.ml
// ============================================================================

/// Convert a polar polygon to affine (cartesian) coordinates with rotation and scale
fn poly_to_affine(poly: &[(f64, f64)], rotat: f64, scale: f64) -> Vec<(f64, f64)> {
    poly.iter()
        .map(|&(theta, radius)| polar_to_affine(theta + rotat, radius * scale))
        .collect()
}

/// Displace all points in an affine polygon by a position offset
fn depl_affine_poly(poly: &[(f64, f64)], pos: Vec2) -> Vec<Vec2> {
    poly.iter().map(|&p| addtuple(p, pos)).collect()
}

/// Render a single polar polygon at a position with rotation and color
fn render_poly(
    poly: &[(f64, f64)],
    pos: Vec2,
    rotat: f64,
    color: [u8; 4],
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    let affine = poly_to_affine(poly, rotat, globals.ratio_rendu);
    let displaced = depl_affine_poly(&affine, pos);
    let screen_points: Vec<(i32, i32)> = displaced
        .iter()
        .map(|&p| dither_tuple(p, DITHER_AA, globals.current_jitter_double))
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
    let position = multuple(
        addtuple(
            addtuple(entity.position, globals.game_screenshake_pos),
            offset,
        ),
        globals.ratio_rendu,
    );
    let exposure = globals.game_exposure * entity.hdr_exposure;

    // Base circle (not in retro mode)
    if visuals.radius > 0.0 && !globals.retro {
        let color = to_rgba(intensify(hdr(visuals.color), exposure), globals);
        let (x, y) = dither_tuple(position, DITHER_AA, globals.current_jitter_double);
        let r = dither_radius(
            visuals.radius * globals.ratio_rendu,
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

/// Render a star with motion trail
pub fn render_star_trail(
    star: &Star,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let pos1 = multuple(
        addtuple(star.pos, globals.game_screenshake_pos),
        globals.ratio_rendu,
    );
    let last_position = multuple(
        addtuple(star.last_pos, globals.game_screenshake_previous_pos),
        globals.ratio_rendu,
    );
    let pos2 = moytuple(last_position, pos1, SHUTTER_SPEED);
    let (x1, y1) = dither_tuple(pos1, DITHER_AA, globals.current_jitter_double);
    let (x2, y2) = dither_tuple(pos2, DITHER_AA, globals.current_jitter_double);

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
        let dist = hypothenuse(soustuple(pos1, pos2));
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
    globals.current_jitter_double = (
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
        globals.game_screenshake_previous_pos = globals.game_screenshake_pos;
        if globals.screenshake_enabled {
            globals.game_screenshake_pos = multuple(
                (rng.gen::<f64>() * 2.0 - 1.0, rng.gen::<f64>() * 2.0 - 1.0),
                globals.game_screenshake,
            );
        }

        // Color interpolation (dynamic color mode)
        if globals.dyn_color {
            let dt = t1 - t0;
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
    }
}

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
        collision_poly(pos1, &h1.points.0, obj1.orientation, pos2, h2.int_radius)
            || collision_poly(pos2, &h2.points.0, obj2.orientation, pos1, h1.int_radius)
    }
}

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
        // Note: unlike OCaml, we scale by game_speed so repulsion stays proportional
        // to simulated time during slowdown events.
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

/// Apply fragment-vs-fragment repulsion (no damage).
fn consequences_collision_frags(mut f1: Entity, mut f2: Entity, globals: &Globals) -> (Entity, Entity) {
    let (angle1, _) = affine_to_polar(soustuple(f1.position, f2.position));
    let (angle2, _) = affine_to_polar(soustuple(f2.position, f1.position));
    // Note: unlike OCaml, we scale by game_speed so repulsion stays proportional
    // to simulated time during slowdown events.
    let dt = (globals.time_current_frame - globals.time_last_frame) * globals.game_speed;
    f1.position = addtuple(f1.position, polar_to_affine(angle1, dt * FRAGMENT_MIN_REPULSION));
    f2.position = addtuple(f2.position, polar_to_affine(angle2, dt * FRAGMENT_MIN_REPULSION));
    f1.velocity = addtuple(f1.velocity, polar_to_affine(angle1, dt * FRAGMENT_MIN_BOUNCE));
    f2.velocity = addtuple(f2.velocity, polar_to_affine(angle2, dt * FRAGMENT_MIN_BOUNCE));
    (f1, f2)
}

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
    for c in state.chunks.iter_mut() {
        c.visuals.radius -= CHUNK_RADIUS_DECAY * globals.game_speed * globals.dt();
    }
    for c in state.chunks_oos.iter_mut() {
        c.visuals.radius -= CHUNK_RADIUS_DECAY * globals.game_speed * globals.dt();
    }
    for c in state.chunks_explo.iter_mut() {
        c.visuals.radius -= CHUNK_EXPLO_RADIUS_DECAY * globals.game_speed * globals.dt();
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
                let (explo, se) = spawn_explosion_chunk(
                    c,
                    globals.flashes_enabled,
                    FLASHES_SATURATE,
                    FLASHES_EXPLOSION,
                    FLASHES_NORMAL_MASS,
                    &mut state.rng,
                );
                let _ = se; // side effects handled if needed
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
        inertie_objet(p, globals);
    }

    // --- Explosion inertia (one frame entities) ---
    for e in state.explosions.iter_mut() {
        inertie_objet(e, globals);
    }

    // --- Filter dead or OOS projectiles (projectiles don't wrap, they despawn) ---
    state.projectiles.retain(|p| {
        p.health >= 0.0 && {
            let (x, y) = p.position;
            x >= -globals.phys_width && x <= 2.0 * globals.phys_width
                && y >= -globals.phys_height && y <= 2.0 * globals.phys_height
        }
    });

    // --- Cooldown tick ---
    if state.cooldown > 0.0 {
        state.cooldown -= globals.game_speed * globals.dt();
    }

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

    // === Fragment vs fragment repulsion + promotion ===
    run_fragment_collisions(state, globals);

    // --- Fragmentation (spawn fragments from dead entities) ---
    spawn_n_frags(&state.objects.clone(), &mut state.fragments, FRAGMENT_NUMBER, &mut state.rng);
    spawn_n_frags(&state.toosmall.clone(), &mut state.fragments, FRAGMENT_NUMBER, &mut state.rng);
    spawn_n_frags(&state.fragments.clone(), &mut state.fragments, FRAGMENT_NUMBER, &mut state.rng);

    // --- Move chunks out of fragments ---
    let new_chunks = drain_filter_stable(&mut state.fragments, ischunk);
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
    let pos1 = multuple(addtuple(pos, globals.game_screenshake_pos), globals.ratio_rendu);
    let dt_game = globals.game_speed
        * (globals.time_current_frame - globals.time_last_frame)
            .max(1.0 / FRAMERATE_RENDER);
    let veloc = multuple(velocity, -(globals.observer_proper_time / proper_time) * dt_game);
    let last_pos = multuple(
        addtuple(soustuple(pos, veloc), globals.game_screenshake_previous_pos),
        globals.ratio_rendu,
    );
    let pos2 = moytuple(last_pos, pos1, SHUTTER_SPEED);
    let dist = hypothenuse(soustuple(pos1, pos2));
    let trail_lum = (radius / (radius + dist)).sqrt();
    let color = to_rgba(intensify(hdr_color, trail_lum), globals);
    let (x1, y1) = dither_tuple(pos1, DITHER_AA, globals.current_jitter_double);
    let (x2, y2) = dither_tuple(pos2, DITHER_AA, globals.current_jitter_double);
    let line_width = dither_radius(2.0 * radius, DITHER_AA, DITHER_POWER_RADIUS, &mut rand::thread_rng());
    renderer.draw_line(x1, y1, x2, y2, color, line_width.max(1) as f32);
}

/// Render a projectile as four concentric light trails. Ported from OCaml render_projectile.
fn render_projectile(entity: &Entity, renderer: &mut Renderer2D, globals: &Globals, rng: &mut impl Rng) {
    let rad = globals.ratio_rendu
        * randfloat(0.5, 1.0, rng)
        * entity.visuals.radius;
    let pos = entity.position;
    let vel = entity.velocity;
    let col = intensify(hdr(entity.visuals.color), entity.hdr_exposure * globals.game_exposure);
    let pt = entity.proper_time;
    render_light_trail(rad,        pos, vel, intensify(col, 0.25), pt, renderer, globals);
    render_light_trail(rad * 0.75, pos, vel, intensify(col, 0.5),  pt, renderer, globals);
    render_light_trail(rad * 0.5,  pos, vel, col,                  pt, renderer, globals);
    render_light_trail(rad * 0.25, pos, vel, intensify(col, 2.0),  pt, renderer, globals);
}

/// Decay smoke radius and exposure (game-time based half-life).
/// Ported from OCaml decay_smoke.
pub fn decay_smoke(smoke: &mut Entity, globals: &Globals) {
    let dt_game = globals.game_speed * globals.dt();
    let half_r = SMOKE_HALF_RADIUS * smoke.proper_time;
    let half_c = SMOKE_HALF_COL * smoke.proper_time;
    // exp_decay: n * 2^(-(dt_game) / half_life)
    smoke.visuals.radius = smoke.visuals.radius * (2.0_f64).powf(-dt_game / half_r)
        - SMOKE_RADIUS_DECAY * dt_game / smoke.proper_time;
    if smoke.hdr_exposure > 0.001 {
        smoke.hdr_exposure *= (2.0_f64).powf(-dt_game / half_c);
    }
}

/// Fire projectiles (tir). Called when Space is held and cooldown allows.
/// Ported from OCaml tir.
pub fn tir(state: &mut GameState, globals: &mut Globals) {
    while state.cooldown <= 0.0 {
        // Flash effect
        if globals.flashes_enabled {
            let flash = intensify(hdr((100.0, 50.0, 25.0)), FLASHES_TIR);
            globals.add_color = (
                globals.add_color.0 + flash.r,
                globals.add_color.1 + flash.v,
                globals.add_color.2 + flash.b,
            );
        }
        if globals.variable_exposure {
            globals.game_exposure *= EXPOSURE_TIR;
        }
        globals.game_screenshake += SCREENSHAKE_TIR_RATIO;

        // Spawn projectiles
        let new_projectiles = spawn_n_projectiles(
            &state.ship,
            globals.projectile_number,
            globals.projectile_min_speed,
            globals.projectile_max_speed,
            globals.projectile_deviation,
            PROJECTILE_HERIT_SPEED,
            &mut state.rng,
        );

        // Muzzle smoke
        if globals.smoke_enabled {
            for p in &new_projectiles {
                let muzzle = spawn_muzzle(p, &mut state.rng);
                state.smoke.push(muzzle);
            }
        }

        state.projectiles.extend(new_projectiles);
        state.cooldown += globals.projectile_cooldown;

        // Recoil
        let recoil = polar_to_affine(state.ship.orientation + PI, globals.projectile_recoil);
        state.ship.velocity = addtuple(state.ship.velocity, recoil);
    }
}

/// Render a complete frame: background, stars, chunks, asteroids, ship
pub fn render_frame(state: &mut GameState, globals: &Globals, renderer: &mut Renderer2D) {
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

    // Ship
    render_visuals(&state.ship, (0.0, 0.0), renderer, globals, &mut state.rng);

    // Smoke (rendered as circles via render_visuals)
    for s in &state.smoke {
        render_visuals(s, (0.0, 0.0), renderer, globals, &mut state.rng);
    }

    // Explosions (rendered as circles)
    for e in &state.explosions {
        render_visuals(e, (0.0, 0.0), renderer, globals, &mut state.rng);
    }

    // Projectiles (light trails)
    for p in &state.projectiles {
        render_projectile(p, renderer, globals, &mut state.rng);
    }
}
