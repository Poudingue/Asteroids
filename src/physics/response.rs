//! Collision response and damage application.
//! Elastic bounce, repulsion, explosion damage, physical collision damage.

use crate::color::*;
use crate::math_utils::*;
use crate::objects::Entity;
use crate::parameters::*;

/// Apply explosion/direct damage to an entity.
pub fn damage(entity: &mut Entity, amount: f64, globals: &mut Globals) {
    let actual = (entity.dam_ratio * amount - entity.dam_res).max(0.0);
    entity.health -= actual;
    globals.screenshake.game_screenshake += amount * SCREENSHAKE_DAM_RATIO;
    if globals.visual.variable_exposure {
        globals.exposure.game_exposure *= EXPOSURE_RATIO_DAMAGE;
    }
    if globals.visual.flashes_enabled {
        let flash = intensify(HdrColor { r: 1.0, g: 0.7, b: 0.5 }, amount * FLASHES_DAMAGE);
        globals.exposure.add_color = (
            globals.exposure.add_color.0 + flash.r,
            globals.exposure.add_color.1 + flash.g,
            globals.exposure.add_color.2 + flash.b,
        );
    }
}

/// Apply physical-collision damage to an entity.
pub fn phys_damage(entity: &mut Entity, amount: f64, globals: &mut Globals) {
    let actual = (entity.phys_ratio * amount - entity.phys_res).max(0.0);
    entity.health -= actual;
    globals.screenshake.game_screenshake +=
        actual * SCREENSHAKE_PHYS_RATIO * entity.mass / SCREENSHAKE_PHYS_MASS;
}

/// Apply physical collision consequences to two entities.
/// Returns updated (e1, e2). Matches OCaml consequences_collision physical branch.
pub fn consequences_collision(
    mut e1: Entity,
    mut e2: Entity,
    globals: &mut Globals,
) -> (Entity, Entity) {
    let total_mass = e1.mass + e2.mass;
    // Mass-weighted average velocity (accounts for proper time)
    let moy_vel = lerp_vec(
        scale_vec(e1.velocity, 1.0 / e1.proper_time),
        scale_vec(e2.velocity, 1.0 / e2.proper_time),
        e1.mass / total_mass,
    );
    let angle1 = to_polar(sub_vec(e1.position, e2.position)).x;
    let angle2 = to_polar(sub_vec(e2.position, e1.position)).x;

    let old_vel1 = e1.velocity;
    let old_vel2 = e2.velocity;

    // New velocities — elastic bounce scaled by proper time
    e1.velocity = scale_vec(
        add_vec(moy_vel, from_polar(angle1, total_mass / e1.mass)),
        e1.proper_time,
    );
    e2.velocity = scale_vec(
        add_vec(moy_vel, from_polar(angle2, total_mass / (e2.mass * e2.proper_time))),
        e2.proper_time,
    );

    if !globals.time.pause {
        // Note: unlike OCaml, we scale by game_speed so repulsion stays proportional
        // to simulated time during slowdown events.
        let dt = (globals.time.time_current_frame - globals.time.time_last_frame) * globals.time.game_speed;
        // Positional repulsion to separate overlapping entities
        e1.position = add_vec(e1.position, from_polar(angle1, MIN_REPULSION * dt));
        e2.position = add_vec(e2.position, from_polar(angle2, MIN_REPULSION * dt));
        // Velocity bounce impulse
        e1.velocity = add_vec(e1.velocity, from_polar(angle1, MIN_BOUNCE * dt));
        e2.velocity = add_vec(e2.velocity, from_polar(angle2, MIN_BOUNCE * dt));
        // Physical damage proportional to velocity change²
        let g1 = magnitude(sub_vec(old_vel1, e1.velocity));
        let g2 = magnitude(sub_vec(old_vel2, e2.velocity));
        phys_damage(&mut e1, globals.weapon.physics_damage_ratio * squared(g1), globals);
        phys_damage(&mut e2, globals.weapon.physics_damage_ratio * squared(g2), globals);
    }
    (e1, e2)
}

/// Apply fragment-vs-fragment repulsion (no damage).
pub fn consequences_collision_frags(
    mut f1: Entity,
    mut f2: Entity,
    globals: &Globals,
) -> (Entity, Entity) {
    let angle1 = to_polar(sub_vec(f1.position, f2.position)).x;
    let angle2 = to_polar(sub_vec(f2.position, f1.position)).x;
    // Note: unlike OCaml, we scale by game_speed so repulsion stays proportional
    // to simulated time during slowdown events.
    let dt = (globals.time.time_current_frame - globals.time.time_last_frame) * globals.time.game_speed;
    f1.position = add_vec(f1.position, from_polar(angle1, dt * FRAGMENT_MIN_REPULSION));
    f2.position = add_vec(f2.position, from_polar(angle2, dt * FRAGMENT_MIN_REPULSION));
    f1.velocity = add_vec(f1.velocity, from_polar(angle1, dt * FRAGMENT_MIN_BOUNCE));
    f2.velocity = add_vec(f2.velocity, from_polar(angle2, dt * FRAGMENT_MIN_BOUNCE));
    (f1, f2)
}
