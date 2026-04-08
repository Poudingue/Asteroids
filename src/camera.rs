use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;
use crate::update::{move_star, translate_entity};

// ============================================================================
// Camera system
// ============================================================================

/// Compute the weighted average pull of all asteroids, used for camera offset.
///
/// Mirrors OCaml `center_of_attention`: each asteroid contributes
/// `(asteroid_pos - screen_center) * mass / (10 + dist²_from_ship)`.
/// The resulting vector is in world-space and can be scaled by `CAMERA_RATIO_OBJECTS`.
fn center_of_attention(objects: &[Entity], ship_pos: Vec2, globals: &Globals) -> Vec2 {
    let screen_center = Vec2::new(
        globals.render.phys_width / 2.0,
        globals.render.phys_height / 2.0,
    );
    objects.iter().fold(Vec2::ZERO, |acc, obj| {
        let rel_pos = sub_vec(obj.position, ship_pos);
        let dist2 = distance_squared(rel_pos, Vec2::ZERO);
        let weight = obj.mass / (10.0 + dist2);
        let pull = scale_vec(sub_vec(obj.position, screen_center), weight);
        add_vec(acc, pull)
    })
}

/// Apply camera movement each frame: translate all entities so the ship stays centred.
///
/// Mirrors OCaml `affiche_etat` lines 900-948 (the camera block before rendering).
/// Must be called after physics but before rendering.
pub fn update_camera(state: &mut crate::game::GameState, globals: &Globals) {
    let ship = &state.ship;

    // 1. Compute camera target (next_x, next_y)
    let facing_offset = from_polar(
        ship.orientation,
        globals.render.phys_width * CAMERA_RATIO_VISION,
    );

    let next = if globals.time.pause {
        // Paused: just keep ship in view with facing offset, no velocity lookahead
        add_vec(ship.position, facing_offset)
    } else {
        // Active: ship pos + velocity lookahead + asteroid pull + facing offset
        let velocity_lookahead = scale_vec(ship.velocity, CAMERA_PREDICTION);
        let mut combined: Vec<Entity> = state.objects.clone();
        combined.extend(state.objects_oos.clone());
        let asteroid_pull = scale_vec(
            center_of_attention(&combined, ship.position, globals),
            CAMERA_RATIO_OBJECTS,
        );
        add_vec(
            facing_offset,
            add_vec(add_vec(ship.position, velocity_lookahead), asteroid_pull),
        )
    };

    // 2. Compute raw camera displacement via exponential decay
    //    move_camera = (center - next) - abso_exp_decay(center - next, CAMERA_HALF_DEPL)
    let t0 = globals.time.time_last_frame;
    let t1 = globals.time.time_current_frame;
    let cx = globals.render.phys_width / 2.0;
    let cy = globals.render.phys_height / 2.0;
    let dx = cx - next.x;
    let dy = cy - next.y;
    let mut movex = dx - abso_exp_decay(dx, CAMERA_HALF_DEPL, t0, t1);
    let mut movey = dy - abso_exp_decay(dy, CAMERA_HALF_DEPL, t0, t1);

    // 3. Boundary clamping: if ship would go past CAMERA_START_BOUND, push it back
    //    elapsed_time = game_speed * (time_last - time_current)  [OCaml sign convention: t_last - t_current > 0]
    let elapsed_time = globals.time.game_speed * (t0 - t1);
    let sx = state.ship.position.x;
    let sy = state.ship.position.y;
    let bound_lo_x = CAMERA_START_BOUND * globals.render.phys_width;
    let bound_hi_x = (1.0 - CAMERA_START_BOUND) * globals.render.phys_width;
    let bound_lo_y = CAMERA_START_BOUND * globals.render.phys_height;
    let bound_hi_y = (1.0 - CAMERA_START_BOUND) * globals.render.phys_height;

    if sx + movex < bound_lo_x {
        movex -= CAMERA_MAX_FORCE * elapsed_time * (-sx - movex + bound_lo_x);
    } else if sx + movex > bound_hi_x {
        movex -= CAMERA_MAX_FORCE * elapsed_time * (-sx - movex + bound_hi_x);
    }
    if sy + movey < bound_lo_y {
        movey -= CAMERA_MAX_FORCE * elapsed_time * (-sy - movey + bound_lo_y);
    } else if sy + movey > bound_hi_y {
        movey -= CAMERA_MAX_FORCE * elapsed_time * (-sy - movey + bound_hi_y);
    }

    let move_camera = Vec2::new(movex, movey);

    // 4. Apply displacement to all entities
    translate_entity(&mut state.ship, move_camera);
    for e in state.objects.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.objects_oos.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.toosmall.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.toosmall_oos.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.fragments.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.chunks.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.chunks_oos.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.chunks_explo.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.projectiles.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.explosions.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.smoke.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.smoke_oos.iter_mut() {
        translate_entity(e, move_camera);
    }
    for e in state.sparks.iter_mut() {
        translate_entity(e, move_camera);
    }
    // Stars get parallax treatment (proximity-scaled displacement)
    for star in state.stars.iter_mut() {
        move_star(star, move_camera, globals);
    }
}
