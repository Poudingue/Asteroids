use std::f64::consts::PI;

use crate::color::*;
use crate::game::GameState;
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;
use crate::spawning::*;
use crate::update::accelerate_entity;

// ============================================================================
// Input handlers
// ============================================================================

/// Aim the ship at the mouse position (screen coords → phys coords → atan2)
pub fn aim_at_mouse(ship: &mut Entity, mouse_x: i32, mouse_y: i32, globals: &Globals) {
    // Flip SDL2 Y-down to renderer Y-up coordinates
    let mouse_phys = Vec2::new(
        mouse_x as f64 / globals.render.render_scale,
        (globals.render.phys_height * globals.render.render_scale - mouse_y as f64)
            / globals.render.render_scale,
    );
    let polar = to_polar(sub_vec(mouse_phys, ship.position));
    let theta = polar.x;
    ship.orientation = theta;
}

/// World-space keyboard thrust: WASD = cardinal directions, diagonal normalized.
/// Movement is decoupled from aim (ship.orientation).
pub fn world_space_thrust_keyboard(
    state: &mut GameState,
    globals: &Globals,
    keys_pressed: [bool; 4],
) {
    let [w, a, s, d] = keys_pressed;
    let mut dir = Vec2::new(0.0, 0.0);
    if w {
        dir.y += 1.0;
    } // Y-up in physics space
    if s {
        dir.y -= 1.0;
    }
    if a {
        dir.x -= 1.0;
    }
    if d {
        dir.x += 1.0;
    }
    let mag = (dir.x * dir.x + dir.y * dir.y).sqrt();
    if mag > 0.0 {
        let normalized = Vec2::new(dir.x / mag, dir.y / mag);
        accelerate_entity(
            &mut state.ship,
            scale_vec(normalized, SHIP_MAX_ACCEL),
            globals,
        );
        // Engine fire while thrusting — ejects opposite to thrust direction
        if state.ship.health > 0.0 && globals.visual.smoke_enabled {
            let thrust_angle = normalized.y.atan2(normalized.x);
            let fire = spawn_fire(&state.ship, thrust_angle, &mut state.rng);
            state.smoke.push(fire);
        }
    }
}

/// Cone-based teleport: cast a 15° cone along aim direction, find the biggest
/// asteroid (object) in the cone (within screen bounds), teleport to its center.
/// The targeted asteroid's health is set to 0 — normal fragmentation in
/// update_game (spawn_fragments) handles debris spawning.
pub fn teleport(state: &mut GameState, globals: &mut Globals) {
    if state.cooldown_tp > 0.0 {
        return;
    }

    let cone_half_angle = TELEPORT_CONE_HALF_ANGLE_DEG.to_radians();
    let aim_angle = state.ship.orientation;
    let ship_pos = state.ship.position;

    // Screen bounds for visibility check
    let phys_w = globals.render.phys_width;
    let phys_h = globals.render.phys_height;

    // Find the biggest asteroid (object) within the cone
    let mut best_idx: Option<usize> = None;
    let mut best_radius: f64 = 0.0;

    for (i, asteroid) in state.objects.iter().enumerate() {
        let delta = sub_vec(asteroid.position, ship_pos);
        let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if distance < 1.0 {
            continue; // Too close / overlapping
        }

        // Angle from ship to asteroid center
        let angle_to = delta.y.atan2(delta.x);

        // Angular difference (wrapped to [-PI, PI])
        let mut angle_diff = angle_to - aim_angle;
        while angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }
        while angle_diff < -PI {
            angle_diff += 2.0 * PI;
        }

        // Effective cone: widen by the asteroid's angular radius
        let angular_radius = (asteroid.hitbox.int_radius / distance).asin().abs();
        let effective_diff = angle_diff.abs() - angular_radius;

        if effective_diff > cone_half_angle {
            continue; // Outside cone
        }

        // Asteroid must be ahead (not behind ship)
        let aim_dir = from_polar(aim_angle, 1.0);
        let dot = delta.x * aim_dir.x + delta.y * aim_dir.y;
        if dot <= 0.0 {
            continue; // Behind ship
        }

        // Screen bounds check: asteroid center must be visible
        let pos = asteroid.position;
        if pos.x < 0.0 || pos.x > phys_w || pos.y < 0.0 || pos.y > phys_h {
            continue;
        }

        // Pick biggest
        let radius = asteroid.hitbox.int_radius;
        if radius > best_radius {
            best_radius = radius;
            best_idx = Some(i);
        }
    }

    if let Some(idx) = best_idx {
        let target_pos = state.objects[idx].position;

        // Kill the targeted asteroid — set health to 0 so normal fragmentation
        // in update_game (spawn_fragments) handles debris spawning.
        state.objects[idx].health = 0.0;

        // Teleport ship to asteroid center
        state.ship.position = target_pos;
        state.ship.velocity = Vec2::ZERO;

        // Visual flash (blue for teleport)
        if globals.visual.flashes_enabled {
            let flash = intensify(
                HdrColor {
                    r: 0.0,
                    g: 4.0,
                    b: 40.0,
                },
                1.0,
            );
            globals.exposure.add_color = (
                globals.exposure.add_color.0 + flash.r,
                globals.exposure.add_color.1 + flash.g,
                globals.exposure.add_color.2 + flash.b,
            );
        }
        globals.exposure.game_exposure *= GAME_EXPOSURE_TP;
        globals.time.game_speed *= RATIO_TIME_TP;

        // Spawn teleport explosion chunks (blue)
        let tp_color = (0.0, 1000.0, 10000.0);
        let new_chunks = spawn_n_chunks(&state.ship, NB_CHUNKS_EXPLO, tp_color, &mut state.rng);
        state.chunks_explo.extend(new_chunks);

        // Reset cooldown
        state.cooldown_tp += COOLDOWN_TP;
    }
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
        let recoil = from_polar(
            state.ship.orientation + PI,
            globals.weapon.projectile_recoil,
        );
        state.ship.velocity = add_vec(state.ship.velocity, recoil);
    }
}

// ============================================================================
// Gamepad input helpers
// ============================================================================

/// Process a single stick axis: subtract drift offset, apply inner/outer dead zone, remap to [0, 1].
pub fn process_stick_axis(raw: f64, center_offset: f64) -> f64 {
    let adjusted = raw - center_offset;
    let abs_val = adjusted.abs();
    if abs_val < STICK_DEAD_ZONE_INNER {
        return 0.0;
    }
    if abs_val > STICK_DEAD_ZONE_OUTER {
        return adjusted.signum();
    }
    let remapped =
        (abs_val - STICK_DEAD_ZONE_INNER) / (STICK_DEAD_ZONE_OUTER - STICK_DEAD_ZONE_INNER);
    remapped * adjusted.signum()
}

/// World-space gamepad stick thrust: analog magnitude proportional to stick deflection.
pub fn world_space_thrust_stick(state: &mut GameState, globals: &Globals, stick: Vec2) {
    let mag = (stick.x * stick.x + stick.y * stick.y).sqrt();
    if mag > 0.0 {
        let clamped_mag = mag.min(1.0);
        let direction = Vec2::new(stick.x / mag, stick.y / mag);
        accelerate_entity(
            &mut state.ship,
            scale_vec(direction, SHIP_MAX_ACCEL * clamped_mag),
            globals,
        );
        // Engine fire while thrusting via stick — ejects opposite to thrust direction
        if state.ship.health > 0.0 && globals.visual.smoke_enabled {
            let thrust_angle = direction.y.atan2(direction.x);
            let fire = spawn_fire(&state.ship, thrust_angle, &mut state.rng);
            state.smoke.push(fire);
        }
    }
}

/// Set ship aim direction from right stick. Only updates when stick magnitude exceeds dead zone
/// (keeps last aim direction when stick is released / in dead zone).
pub fn aim_from_stick(ship: &mut Entity, stick: Vec2) {
    let mag = (stick.x * stick.x + stick.y * stick.y).sqrt();
    if mag > 0.0 {
        ship.orientation = stick.y.atan2(stick.x);
    }
}

/// Update adaptive drift compensation for a stick.
/// When no buttons are pressed and stick is stable for DRIFT_RECENTER_DELAY seconds,
/// slowly lerp the center offset toward the current raw reading.
pub fn update_drift_compensation(
    center_offset: &mut Vec2,
    raw: Vec2,
    any_button_pressed: bool,
    last_idle_time: &mut f64,
    current_time: f64,
    dt: f64,
) {
    if any_button_pressed || raw.x.abs() > 0.5 || raw.y.abs() > 0.5 {
        // Stick is actively in use — reset idle timer
        *last_idle_time = current_time;
        return;
    }
    let idle_duration = current_time - *last_idle_time;
    if idle_duration >= DRIFT_RECENTER_DELAY {
        let lerp_factor = (DRIFT_RECENTER_SPEED * dt).min(1.0);
        center_offset.x += (raw.x - center_offset.x) * lerp_factor;
        center_offset.y += (raw.y - center_offset.y) * lerp_factor;
    }
}
