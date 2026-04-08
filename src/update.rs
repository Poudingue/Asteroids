use rand::prelude::*;

use crate::color::*;
use crate::game::{GameState, GamepadState};
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;

// ============================================================================
// Movement functions — ported from ml/asteroids.ml
// ============================================================================

/// Displace an object by a velocity vector, scaled by dt * game_speed * observer/proper time
pub fn move_entity(entity: &mut Entity, vel: Vec2, globals: &Globals) {
    let time_factor =
        globals.dt() * globals.time.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.position = proj(entity.position, vel, time_factor);
}

/// Apply an object's velocity as displacement (inertia)
pub fn apply_inertia(entity: &mut Entity, globals: &Globals) {
    let vel = entity.velocity;
    move_entity(entity, vel, globals);
}

/// Accelerate an object (velocity += accel * dt * ...)
pub fn accelerate_entity(entity: &mut Entity, accel: Vec2, globals: &Globals) {
    let time_factor =
        globals.dt() * globals.time.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.velocity = proj(entity.velocity, accel, time_factor);
}

/// Instant velocity change (no time scaling)
pub fn boost_entity(entity: &mut Entity, boost: Vec2) {
    entity.velocity = proj(entity.velocity, boost, 1.0);
}

/// Timed rotation (orientation += rotation * dt * ...)
pub fn rotate_entity(entity: &mut Entity, rotation: f64, globals: &Globals) {
    let time_factor =
        globals.dt() * globals.time.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
    entity.orientation += rotation * time_factor;
}

/// Instant rotation (no time scaling)
pub fn turn_entity(entity: &mut Entity, rotation: f64) {
    entity.orientation += rotation;
}

/// Angular acceleration (moment += momentum * dt * ...)
pub fn apply_torque(entity: &mut Entity, momentum: f64, globals: &Globals) {
    let time_factor =
        globals.dt() * globals.time.game_speed * OBSERVER_PROPER_TIME / entity.proper_time;
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
    entity.position = wrap_toroidal(
        entity.position,
        globals.render.phys_width,
        globals.render.phys_height,
    );
}

/// Wrap all entities' positions (toroidal world)
pub fn wrap_entities(entities: &mut [Entity], globals: &Globals) {
    for e in entities.iter_mut() {
        wrap_entity(e, globals);
    }
}

// --- Entity predicates ---

pub fn is_alive(entity: &Entity) -> bool {
    entity.health > 0.0
}

pub fn is_dead(entity: &Entity) -> bool {
    entity.health <= 0.0
}

pub fn ischunk(entity: &Entity) -> bool {
    entity.hitbox.int_radius < CHUNK_MAX_SIZE
}

pub fn big_enough(entity: &Entity) -> bool {
    entity.hitbox.int_radius >= ASTEROID_MIN_SIZE
}

pub fn too_small(entity: &Entity) -> bool {
    !big_enough(entity)
}

pub fn positive_radius(entity: &Entity) -> bool {
    entity.visuals.radius > 0.0
}

/// Check if entity is within visible screen area (with radius margin)
pub fn checkspawn_objet(entity: &Entity, globals: &Globals) -> bool {
    let x = entity.position.x;
    let y = entity.position.y;
    let rad = entity.hitbox.ext_radius;
    (x - rad < globals.render.phys_width)
        && (x + rad > 0.0)
        && (y - rad < globals.render.phys_height)
        && (y + rad > 0.0)
}

/// Transfer entities between on-screen and off-screen lists.
pub fn transfer_oos(onscreen: &mut Vec<Entity>, oos: &mut Vec<Entity>, globals: &Globals) {
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

/// Remove dead entities, transfer chunk-sized asteroids to chunks list, and remove zero-radius debris.
/// Matches OCaml despawn: collects ischunk from all asteroid lists before filtering notchunk.
pub fn despawn(state: &mut GameState, globals: &Globals) {
    if globals.visual.chunks_enabled {
        // Collect chunk-sized asteroids from all asteroid lists (OCaml: ischunk filter then append to ref_chunks)
        let new_from_objects = state
            .objects
            .extract_if(.., |e| ischunk(e))
            .collect::<Vec<_>>();
        let new_from_objects_oos = state
            .objects_oos
            .extract_if(.., |e| ischunk(e))
            .collect::<Vec<_>>();
        let new_from_toosmall = state
            .toosmall
            .extract_if(.., |e| ischunk(e))
            .collect::<Vec<_>>();
        let new_from_toosmall_oos = state
            .toosmall_oos
            .extract_if(.., |e| ischunk(e))
            .collect::<Vec<_>>();
        let new_from_fragments = state
            .fragments
            .extract_if(.., |e| ischunk(e))
            .collect::<Vec<_>>();

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

    // Now filter dead entities from asteroid lists
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
    star.pos = wrap_single(next, globals.render.phys_width, globals.render.phys_height);
    // Avoid incorrect motion blur from screen-edge teleport
    if next.x > globals.render.phys_width
        || next.x < 0.0
        || next.y > globals.render.phys_height
        || next.y < 0.0
    {
        star.last_pos = star.pos;
    }
}

// ============================================================================
// Per-frame update
// ============================================================================

/// Update per-frame globals: jitter, game speed interpolation, exposure
pub fn update_frame(globals: &mut Globals, rng: &mut impl Rng) {
    // Jitter for dithering
    globals.render.current_jitter_double = Vec2::new(
        rng.gen::<f64>() * DITHER_POWER,
        rng.gen::<f64>() * DITHER_POWER,
    );

    if !globals.time.pause {
        let t0 = globals.time.time_last_frame;
        let t1 = globals.time.time_current_frame;

        // Game speed interpolation (real-time based, not game-time)
        globals.time.game_speed = globals.time.game_speed_target
            + abso_exp_decay(
                globals.time.game_speed - globals.time.game_speed_target,
                HALF_SPEED_CHANGE,
                t0,
                t1,
            );

        // Exposure interpolation
        globals.exposure.game_exposure = globals.exposure.game_exposure_target
            + abso_exp_decay(
                globals.exposure.game_exposure - globals.exposure.game_exposure_target,
                EXPOSURE_HALF_LIFE,
                t0,
                t1,
            );

        // Flash decay
        let flash_decay = abso_exp_decay(1.0, FLASHES_HALF_LIFE, t0, t1);
        globals.exposure.add_color = (
            globals.exposure.add_color.0 * flash_decay,
            globals.exposure.add_color.1 * flash_decay,
            globals.exposure.add_color.2 * flash_decay,
        );

        // Screenshake decay
        globals.screenshake.game_screenshake = abso_exp_decay(
            globals.screenshake.game_screenshake,
            SCREENSHAKE_HALF_LIFE,
            t0,
            t1,
        );

        // Score shake decay
        globals.screenshake.shake_score = abso_exp_decay(
            globals.screenshake.shake_score,
            SHAKE_SCORE_HALF_LIFE,
            t0,
            t1,
        );
        globals.screenshake.game_screenshake_previous_pos =
            globals.screenshake.game_screenshake_pos;
        if globals.visual.screenshake_enabled {
            globals.screenshake.game_screenshake_pos = scale_vec(
                Vec2::new(rng.gen::<f64>() * 2.0 - 1.0, rng.gen::<f64>() * 2.0 - 1.0),
                globals.screenshake.game_screenshake,
            );
            // Smooth screenshake: blend toward previous position for a low-pass effect.
            // Matches OCaml: game_screenshake_pos := lerp_vec !game_screenshake_previous_pos !game_screenshake_pos screenshake_smoothness
            if SCREENSHAKE_SMOOTH {
                globals.screenshake.game_screenshake_pos = lerp_vec(
                    globals.screenshake.game_screenshake_previous_pos,
                    globals.screenshake.game_screenshake_pos,
                    SCREENSHAKE_SMOOTHNESS,
                );
            }
        }

        // Color interpolation (dynamic color mode)
        if globals.visual.dyn_color {
            let dt = t1 - t0;
            globals.exposure.mul_color = {
                let c = half_color(
                    hdr(globals.exposure.mul_color),
                    hdr(globals.exposure.mul_base),
                    FILTER_HALF_LIFE,
                    dt,
                );
                (c.r, c.g, c.b)
            };
            globals.visual.space_color = {
                let c = half_color(
                    hdr(globals.visual.space_color),
                    hdr(globals.visual.space_color_goal),
                    SPACE_HALF_LIFE,
                    dt,
                );
                (c.r, c.g, c.b)
            };
            globals.visual.star_color = {
                let c = half_color(
                    hdr(globals.visual.star_color),
                    hdr(globals.visual.star_color_goal),
                    SPACE_HALF_LIFE,
                    dt,
                );
                (c.r, c.g, c.b)
            };
        }
    }

    // --- FPS counter (matches OCaml end-of-frame block) ---
    globals.framerate.time_current_count = globals.time.time_current_frame;
    globals.framerate.current_count += 1;
    if globals.framerate.time_current_count - globals.framerate.time_last_count > 1.0 {
        globals.framerate.last_count = globals.framerate.current_count;
        globals.framerate.current_count = 0;
        globals.framerate.time_last_count = globals.framerate.time_current_count;
    }
}

/// Update the smoothed visual aim angle to track ship.orientation.
/// Uses exponential approach: visual angle lerps toward true aim at a rate
/// controlled by AIM_VISUAL_SMOOTHING. Handles angle wrapping correctly.
pub fn update_visual_aim(gamepad: &mut GamepadState, target: f64, dt: f64) {
    use std::f64::consts::PI;
    let mut diff = target - gamepad.visual_aim_angle;
    // Wrap to [-PI, PI]
    while diff > PI {
        diff -= 2.0 * PI;
    }
    while diff < -PI {
        diff += 2.0 * PI;
    }

    if AIM_VISUAL_SMOOTHING <= 0.0 {
        gamepad.visual_aim_angle = target;
    } else {
        let factor = (AIM_VISUAL_SMOOTHING * dt).min(1.0);
        gamepad.visual_aim_angle += diff * factor;
    }
}

/// Enforce per-collection particle caps, removing oldest/smallest particles first.
pub fn enforce_particle_budgets(state: &mut GameState) {
    // Smoke: oldest-first (front of Vec is oldest)
    if state.smoke.len() > PARTICLE_BUDGET_SMOKE {
        let excess = state.smoke.len() - PARTICLE_BUDGET_SMOKE;
        state.smoke.drain(0..excess);
    }
    if state.smoke_oos.len() > PARTICLE_BUDGET_SMOKE {
        let excess = state.smoke_oos.len() - PARTICLE_BUDGET_SMOKE;
        state.smoke_oos.drain(0..excess);
    }
    // Chunks: lowest-radius first (proxy for most-faded)
    if state.chunks.len() > PARTICLE_BUDGET_CHUNKS {
        state.chunks.sort_by(|a, b| {
            a.visuals
                .radius
                .partial_cmp(&b.visuals.radius)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let excess = state.chunks.len() - PARTICLE_BUDGET_CHUNKS;
        state.chunks.drain(0..excess);
    }
    if state.chunks_oos.len() > PARTICLE_BUDGET_CHUNKS {
        state.chunks_oos.sort_by(|a, b| {
            a.visuals
                .radius
                .partial_cmp(&b.visuals.radius)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let excess = state.chunks_oos.len() - PARTICLE_BUDGET_CHUNKS;
        state.chunks_oos.drain(0..excess);
    }
    if state.chunks_explo.len() > PARTICLE_BUDGET_CHUNKS {
        state.chunks_explo.sort_by(|a, b| {
            a.visuals
                .radius
                .partial_cmp(&b.visuals.radius)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let excess = state.chunks_explo.len() - PARTICLE_BUDGET_CHUNKS;
        state.chunks_explo.drain(0..excess);
    }
    // Explosions: oldest-first
    if state.explosions.len() > PARTICLE_BUDGET_EXPLOSIONS {
        let excess = state.explosions.len() - PARTICLE_BUDGET_EXPLOSIONS;
        state.explosions.drain(0..excess);
    }
}

/// Decay smoke radius and exposure with an optional fade multiplier for graceful degradation.
pub fn decay_smoke_multiplied(smoke: &mut Entity, globals: &Globals, fade_multiplier: f64) {
    let dt_game = globals.time.game_speed * globals.dt() * fade_multiplier;
    let half_r = SMOKE_HALF_RADIUS * smoke.proper_time;
    let half_c = SMOKE_HALF_COL * smoke.proper_time;
    // exp_decay: n * 2^(-(dt_game) / half_life)
    smoke.visuals.radius = smoke.visuals.radius * (2.0_f64).powf(-dt_game / half_r)
        - SMOKE_RADIUS_DECAY * dt_game * globals.observer_proper_time / smoke.proper_time;
    if smoke.hdr_exposure > 0.001 {
        smoke.hdr_exposure *= (2.0_f64).powf(-dt_game / half_c);
    }
}

/// Decay smoke radius and exposure (game-time based half-life).
/// Ported from OCaml decay_smoke.
pub fn decay_smoke(smoke: &mut Entity, globals: &Globals) {
    let dt_game = globals.time.game_speed * globals.dt();
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
// Local helper (mirrors game::hdr, needed by update_frame)
// ============================================================================

fn hdr(color: (f64, f64, f64)) -> HdrColor {
    HdrColor::new(color.0, color.1, color.2)
}
