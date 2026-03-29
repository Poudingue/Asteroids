use std::f64::consts::PI;

use crate::color::*;
use crate::game::{
    accelerate_entity, apply_torque, boost_entity, boost_torque, rotate_entity, turn_entity,
    GameState,
};
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;

// ============================================================================
// Input handlers
// ============================================================================

/// Aim the ship at the mouse position (screen coords → phys coords → atan2)
pub fn aim_at_mouse(ship: &mut Entity, mouse_x: i32, mouse_y: i32, globals: &Globals) {
        // Flip SDL2 Y-down to renderer Y-up coordinates
    let mouse_phys = Vec2::new(
        mouse_x as f64 / globals.render.render_scale,
        (globals.render.phys_height * globals.render.render_scale - mouse_y as f64) / globals.render.render_scale,
    );
    let polar = to_polar(sub_vec(mouse_phys, ship.position));
    let theta = polar.x;
    ship.orientation = theta;
}

pub fn acceleration(state: &mut GameState, globals: &Globals) {
    let orientation = state.ship.orientation;
    accelerate_entity(
        &mut state.ship,
        from_polar(orientation, SHIP_MAX_ACCEL),
        globals,
    );
    // Engine fire: spawn 1 particle when accelerating (OCaml: spawn_fire)
    if state.ship.health > 0.0 && globals.visual.smoke_enabled {
        let fire = spawn_fire(&state.ship, &mut state.rng);
        state.smoke.push(fire);
    }
}

/// Forward boost (impulse, instant velocity change).
/// Also spawns 9 engine fire particles for a more intense thrust effect (matches OCaml `boost`).
pub fn boost_forward(state: &mut GameState, globals: &Globals) {
    let orientation = state.ship.orientation;
    boost_entity(&mut state.ship, from_polar(orientation, SHIP_MAX_BOOST));
    // Engine fire: spawn 9 particles on boost (OCaml: 3 lists of 3)
    if state.ship.health > 0.0 && globals.visual.smoke_enabled {
        for _ in 0..9 {
            let fire = spawn_fire(&state.ship, &mut state.rng);
            state.smoke.push(fire);
        }
    }
}

/// Teleport ship to mouse position (F key). Edge-triggered; respects cooldown.
/// Matches OCaml `teleport`: sets position/velocity, spawns explosion chunks, adjusts exposure/game_speed.
pub fn teleport(state: &mut GameState, globals: &mut Globals, mouse_x: f64, mouse_y: f64) {
    if state.cooldown_tp <= 0.0 {
        // Teleport to mouse position in physics space
        let new_pos = Vec2::new(mouse_x / globals.render.render_scale, mouse_y / globals.render.render_scale);
        state.ship.position = new_pos;
        state.ship.velocity = Vec2::ZERO;

        // Visual flash + slow-mo (matches OCaml: add_color intensify, game_exposure *= tp, game_speed *= ratio_time_tp)
        if globals.visual.flashes_enabled {
            let flash = intensify(HdrColor { r: 0.0, g: 4.0, b: 40.0 }, 1.0);
            globals.exposure.add_color = (
                globals.exposure.add_color.0 + flash.r,
                globals.exposure.add_color.1 + flash.g,
                globals.exposure.add_color.2 + flash.b,
            );
        }
        globals.exposure.game_exposure *= GAME_EXPOSURE_TP;
        globals.time.game_speed *= RATIO_TIME_TP;

        // Spawn teleport explosion chunks
        let tp_color = (0.0, 1000.0, 10000.0);
        let new_chunks = spawn_n_chunks(&state.ship, NB_CHUNKS_EXPLO, tp_color, &mut state.rng);
        state.chunks_explo.extend(new_chunks);

        // Reset cooldown
        state.cooldown_tp += COOLDOWN_TP;
    }
}

/// Rotate left — impulse or continuous depending on globals
pub fn handle_left(ship: &mut Entity, globals: &Globals) {
    if globals.ship_control.ship_impulse_pos {
        if globals.ship_control.ship_direct_rotat {
            turn_entity(ship, SHIP_MAX_ROTAT);
        } else {
            boost_torque(ship, SHIP_MAX_TOURN_BOOST);
        }
    } else if globals.ship_control.ship_direct_rotat {
        rotate_entity(ship, SHIP_MAX_TOURN, globals);
    } else {
        apply_torque(ship, SHIP_MAX_TOURN, globals);
    }
}

/// Rotate right — impulse or continuous depending on globals
pub fn handle_right(ship: &mut Entity, globals: &Globals) {
    if globals.ship_control.ship_impulse_pos {
        if globals.ship_control.ship_direct_rotat {
            turn_entity(ship, -SHIP_MAX_ROTAT);
        } else {
            boost_torque(ship, -SHIP_MAX_TOURN_BOOST);
        }
    } else if globals.ship_control.ship_direct_rotat {
        rotate_entity(ship, -SHIP_MAX_TOURN, globals);
    } else {
        apply_torque(ship, -SHIP_MAX_TOURN, globals);
    }
}

/// Strafe left (always impulse boost perpendicular to heading)
pub fn strafe_left(ship: &mut Entity) {
    let orientation = ship.orientation + PI / 2.0;
    boost_entity(ship, from_polar(orientation, SHIP_MAX_BOOST));
}

/// Strafe right (always impulse boost perpendicular to heading)
pub fn strafe_right(ship: &mut Entity) {
    let orientation = ship.orientation - PI / 2.0;
    boost_entity(ship, from_polar(orientation, SHIP_MAX_BOOST));
}

/// Fire projectiles. Called when Space is held and cooldown allows.
/// Ported from OCaml tir.
pub fn fire(state: &mut GameState, globals: &mut Globals) {
    while state.cooldown <= 0.0 {
        // Flash effect
        if globals.visual.flashes_enabled {
            let flash = intensify(HdrColor::new(100.0, 50.0, 25.0), FLASHES_TIR);
            globals.exposure.add_color = (
                globals.exposure.add_color.0 + flash.r,
                globals.exposure.add_color.1 + flash.g,
                globals.exposure.add_color.2 + flash.b,
            );
        }
        if globals.visual.variable_exposure {
            globals.exposure.game_exposure *= EXPOSURE_TIR;
        }
        globals.screenshake.game_screenshake += SCREENSHAKE_TIR_RATIO;

        // Spawn projectiles
        let new_projectiles = spawn_n_projectiles(
            &state.ship,
            globals.weapon.projectile_number,
            globals.weapon.projectile_min_speed,
            globals.weapon.projectile_max_speed,
            globals.weapon.projectile_deviation,
            PROJECTILE_HERIT_SPEED,
            &mut state.rng,
        );

        // Muzzle smoke
        if globals.visual.smoke_enabled {
            for p in &new_projectiles {
                let muzzle = spawn_muzzle(p, &mut state.rng);
                state.smoke.push(muzzle);
            }
        }

        state.projectiles.extend(new_projectiles);
        state.cooldown += globals.weapon.projectile_cooldown;

        // Recoil
        let recoil = from_polar(state.ship.orientation + PI, globals.weapon.projectile_recoil);
        state.ship.velocity = add_vec(state.ship.velocity, recoil);
    }
}
