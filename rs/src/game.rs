use std::f64::consts::PI;

use rand::prelude::*;

use crate::color::*;
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;
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
    }
}

/// Render a complete frame: background, stars, ship
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

    // Ship
    render_visuals(&state.ship, (0.0, 0.0), renderer, globals, &mut state.rng);
}
