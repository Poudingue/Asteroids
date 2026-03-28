/// Property tests for movement and collision functions in game.rs
/// Covers move_entity, apply_inertia, accelerate_entity, boost_entity, rotate_entity,
/// turn_entity, apply_torque (apply_angular_momentum), collision_circles, collision_point.
/// These tests serve as a safety net before the V2 refactor.

use asteroids::game::*;
use asteroids::math_utils::*;
use asteroids::objects::*;
use asteroids::parameters::*;

const EPS: f64 = 1e-10;

// ─── Helpers ────────────────────────────────────────────────────────────────

fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

fn vec2_approx_eq(a: Vec2, b: Vec2, eps: f64) -> bool {
    approx_eq(a.x, b.x, eps) && approx_eq(a.y, b.y, eps)
}

/// Create a Globals with a fixed dt and game_speed=1.0.
/// dt = time_current_frame - time_last_frame
fn make_globals(dt: f64) -> Globals {
    let mut g = Globals::new();
    g.game_speed = 1.0;
    g.time_last_frame = 0.0;
    g.time_current_frame = dt;
    g
}

/// Create a ship entity at origin with zero velocity and proper_time=1.0.
fn make_entity() -> Entity {
    let mut e = spawn_ship();
    e.position = Vec2::ZERO;
    e.velocity = Vec2::ZERO;
    e.orientation = 0.0;
    e.moment = 0.0;
    e.proper_time = 1.0;
    e
}

// ─── move_entity ────────────────────────────────────────────────────────────

#[test]
fn move_entity_basic_movement() {
    // With dt=1, game_speed=1, OBSERVER_PROPER_TIME=1, proper_time=1:
    // time_factor = 1 * 1 * 1 / 1 = 1
    // new_pos = (0,0) + (3,4) * 1 = (3,4)
    let mut e = make_entity();
    let g = make_globals(1.0);
    move_entity(&mut e, Vec2::new(3.0, 4.0), &g);
    assert!(
        vec2_approx_eq(e.position, Vec2::new(3.0, 4.0), EPS),
        "expected (3,4), got {:?}",
        e.position
    );
}

#[test]
fn move_entity_zero_velocity() {
    let mut e = make_entity();
    let g = make_globals(1.0);
    e.position = Vec2::new(5.0, 7.0);
    move_entity(&mut e, Vec2::new(0.0, 0.0), &g);
    assert!(
        vec2_approx_eq(e.position, Vec2::new(5.0, 7.0), EPS),
        "zero velocity should not move entity"
    );
}

#[test]
fn move_entity_zero_dt() {
    let mut e = make_entity();
    e.position = Vec2::new(5.0, 7.0);
    let g = make_globals(0.0);
    move_entity(&mut e, Vec2::new(3.0, 4.0), &g);
    assert!(
        vec2_approx_eq(e.position, Vec2::new(5.0, 7.0), EPS),
        "zero dt should not move entity"
    );
}

#[test]
fn move_entity_scales_with_dt() {
    // With dt=2 and vel=(1,0): displacement = 2 (since game_speed=1, OBSERVER_PROPER_TIME=1, proper_time=1)
    let mut e = make_entity();
    let g = make_globals(2.0);
    move_entity(&mut e, Vec2::new(1.0, 0.0), &g);
    assert!(
        approx_eq(e.position.x, 2.0, EPS),
        "position.x should be 2, got {}",
        e.position.x
    );
}

#[test]
fn move_entity_scales_with_game_speed() {
    // game_speed=2 doubles displacement
    let mut e1 = make_entity();
    let mut e2 = make_entity();
    let mut g1 = make_globals(1.0);
    let mut g2 = make_globals(1.0);
    g1.game_speed = 1.0;
    g2.game_speed = 2.0;
    let vel = Vec2::new(3.0, 4.0);
    move_entity(&mut e1, vel, &g1);
    move_entity(&mut e2, vel, &g2);
    assert!(
        vec2_approx_eq(e2.position, Vec2::new(e1.position.x * 2.0, e1.position.y * 2.0), EPS),
        "game_speed=2 should double displacement"
    );
}

#[test]
fn move_entity_negative_velocity() {
    let mut e = make_entity();
    e.position = Vec2::new(5.0, 5.0);
    let g = make_globals(1.0);
    move_entity(&mut e, Vec2::new(-2.0, -3.0), &g);
    assert!(
        vec2_approx_eq(e.position, Vec2::new(3.0, 2.0), EPS),
        "expected (3,2), got {:?}",
        e.position
    );
}

// ─── apply_inertia ───────────────────────────────────────────────────────────

#[test]
fn apply_inertia_basic() {
    // entity with velocity (1,2), dt=1 => moves by (1,2)
    let mut e = make_entity();
    e.velocity = Vec2::new(1.0, 2.0);
    let g = make_globals(1.0);
    apply_inertia(&mut e, &g);
    assert!(
        vec2_approx_eq(e.position, Vec2::new(1.0, 2.0), EPS),
        "expected (1,2), got {:?}",
        e.position
    );
}

#[test]
fn apply_inertia_stationary() {
    let mut e = make_entity();
    e.velocity = Vec2::new(0.0, 0.0);
    e.position = Vec2::new(3.0, 5.0);
    let g = make_globals(1.0);
    apply_inertia(&mut e, &g);
    assert!(
        vec2_approx_eq(e.position, Vec2::new(3.0, 5.0), EPS),
        "stationary entity should not move"
    );
}

#[test]
fn apply_inertia_does_not_change_velocity() {
    let mut e = make_entity();
    e.velocity = Vec2::new(2.0, 3.0);
    let g = make_globals(1.0);
    apply_inertia(&mut e, &g);
    assert!(
        vec2_approx_eq(e.velocity, Vec2::new(2.0, 3.0), EPS),
        "inertia should not alter velocity"
    );
}

#[test]
fn apply_inertia_uses_own_velocity() {
    // Displacement should equal velocity * time_factor
    let mut e1 = make_entity();
    let mut e2 = make_entity();
    e1.velocity = Vec2::new(5.0, 0.0);
    e2.velocity = Vec2::new(10.0, 0.0);
    let g = make_globals(1.0);
    apply_inertia(&mut e1, &g);
    apply_inertia(&mut e2, &g);
    // e2 should move twice as far as e1
    assert!(
        approx_eq(e2.position.x, e1.position.x * 2.0, EPS),
        "faster entity should move proportionally farther"
    );
}

// ─── accelerate_entity ─────────────────────────────────────────────────────────────

#[test]
fn accelerate_entity_basic() {
    // dt=1, game_speed=1 => velocity += accel * 1
    let mut e = make_entity();
    e.velocity = Vec2::new(0.0, 0.0);
    let g = make_globals(1.0);
    accelerate_entity(&mut e, Vec2::new(2.0, 3.0), &g);
    assert!(
        vec2_approx_eq(e.velocity, Vec2::new(2.0, 3.0), EPS),
        "expected velocity (2,3), got {:?}",
        e.velocity
    );
}

#[test]
fn accelerate_entity_cumulative() {
    let mut e = make_entity();
    e.velocity = Vec2::new(1.0, 0.0);
    let g = make_globals(1.0);
    accelerate_entity(&mut e, Vec2::new(1.0, 0.0), &g);
    accelerate_entity(&mut e, Vec2::new(1.0, 0.0), &g);
    assert!(
        approx_eq(e.velocity.x, 3.0, EPS),
        "two accelerations should accumulate: expected 3, got {}",
        e.velocity.x
    );
}

#[test]
fn accelerate_entity_zero_acceleration() {
    let mut e = make_entity();
    e.velocity = Vec2::new(5.0, 7.0);
    let g = make_globals(1.0);
    accelerate_entity(&mut e, Vec2::new(0.0, 0.0), &g);
    assert!(
        vec2_approx_eq(e.velocity, Vec2::new(5.0, 7.0), EPS),
        "zero acceleration should not change velocity"
    );
}

#[test]
fn accelerate_entity_does_not_change_position() {
    let mut e = make_entity();
    e.position = Vec2::new(3.0, 4.0);
    let g = make_globals(1.0);
    accelerate_entity(&mut e, Vec2::new(5.0, 6.0), &g);
    assert!(
        vec2_approx_eq(e.position, Vec2::new(3.0, 4.0), EPS),
        "accelerate_entity should not change position"
    );
}

#[test]
fn accelerate_entity_negative_deceleration() {
    let mut e = make_entity();
    e.velocity = Vec2::new(5.0, 0.0);
    let g = make_globals(1.0);
    accelerate_entity(&mut e, Vec2::new(-2.0, 0.0), &g);
    assert!(
        approx_eq(e.velocity.x, 3.0, EPS),
        "negative accel should reduce velocity: expected 3, got {}",
        e.velocity.x
    );
}

// ─── boost_entity ─────────────────────────────────────────────────────────────

#[test]
fn boost_entity_basic() {
    let mut e = make_entity();
    e.velocity = Vec2::new(1.0, 2.0);
    boost_entity(&mut e, Vec2::new(3.0, 4.0));
    assert!(
        vec2_approx_eq(e.velocity, Vec2::new(4.0, 6.0), EPS),
        "expected velocity (4,6), got {:?}",
        e.velocity
    );
}

#[test]
fn boost_entity_zero_boost() {
    let mut e = make_entity();
    e.velocity = Vec2::new(5.0, 6.0);
    boost_entity(&mut e, Vec2::new(0.0, 0.0));
    assert!(
        vec2_approx_eq(e.velocity, Vec2::new(5.0, 6.0), EPS),
        "zero boost should not change velocity"
    );
}

#[test]
fn boost_entity_no_time_scaling() {
    // boost_entity has no dt — apply same boost with dt=0 and dt=100: same result
    let mut e1 = make_entity();
    let mut e2 = make_entity();
    e1.velocity = Vec2::ZERO;
    e2.velocity = Vec2::ZERO;
    boost_entity(&mut e1, Vec2::new(3.0, 4.0));
    boost_entity(&mut e2, Vec2::new(3.0, 4.0));
    assert!(
        vec2_approx_eq(e1.velocity, e2.velocity, EPS),
        "boost_entity is time-independent"
    );
}

#[test]
fn boost_entity_negative() {
    let mut e = make_entity();
    e.velocity = Vec2::new(5.0, 5.0);
    boost_entity(&mut e, Vec2::new(-3.0, -2.0));
    assert!(
        vec2_approx_eq(e.velocity, Vec2::new(2.0, 3.0), EPS),
        "expected (2,3), got {:?}",
        e.velocity
    );
}

// ─── rotate_entity ─────────────────────────────────────────────────────────────

#[test]
fn rotate_entity_basic() {
    // dt=1, game_speed=1, OBSERVER_PROPER_TIME=1, proper_time=1 => time_factor=1
    let mut e = make_entity();
    e.orientation = 0.0;
    let g = make_globals(1.0);
    rotate_entity(&mut e, 1.0, &g);
    assert!(
        approx_eq(e.orientation, 1.0, EPS),
        "expected orientation 1.0, got {}",
        e.orientation
    );
}

#[test]
fn rotate_entity_zero_rotation() {
    let mut e = make_entity();
    e.orientation = 2.0;
    let g = make_globals(1.0);
    rotate_entity(&mut e, 0.0, &g);
    assert!(
        approx_eq(e.orientation, 2.0, EPS),
        "zero rotation should not change orientation"
    );
}

#[test]
fn rotate_entity_zero_dt() {
    let mut e = make_entity();
    e.orientation = 1.5;
    let g = make_globals(0.0);
    rotate_entity(&mut e, 10.0, &g);
    assert!(
        approx_eq(e.orientation, 1.5, EPS),
        "zero dt should not change orientation"
    );
}

#[test]
fn rotate_entity_negative_rotation() {
    let mut e = make_entity();
    e.orientation = 2.0;
    let g = make_globals(1.0);
    rotate_entity(&mut e, -1.0, &g);
    assert!(
        approx_eq(e.orientation, 1.0, EPS),
        "expected orientation 1.0, got {}",
        e.orientation
    );
}

#[test]
fn rotate_entity_cumulative() {
    let mut e = make_entity();
    e.orientation = 0.0;
    let g = make_globals(1.0);
    rotate_entity(&mut e, 1.0, &g);
    rotate_entity(&mut e, 1.0, &g);
    assert!(
        approx_eq(e.orientation, 2.0, EPS),
        "two rotations should accumulate: expected 2.0, got {}",
        e.orientation
    );
}

// ─── turn_entity ─────────────────────────────────────────────────────────────

#[test]
fn turn_entity_basic() {
    let mut e = make_entity();
    e.orientation = 0.5;
    turn_entity(&mut e, 1.0);
    assert!(
        approx_eq(e.orientation, 1.5, EPS),
        "expected orientation 1.5, got {}",
        e.orientation
    );
}

#[test]
fn turn_entity_zero() {
    let mut e = make_entity();
    e.orientation = 2.0;
    turn_entity(&mut e, 0.0);
    assert!(
        approx_eq(e.orientation, 2.0, EPS),
        "zero rotation should not change orientation"
    );
}

#[test]
fn turn_entity_no_time_scaling() {
    // Same result regardless of dt — turn_entity is instant
    let mut e1 = make_entity();
    let mut e2 = make_entity();
    e1.orientation = 0.0;
    e2.orientation = 0.0;
    turn_entity(&mut e1, 1.5);
    turn_entity(&mut e2, 1.5);
    assert!(
        approx_eq(e1.orientation, e2.orientation, EPS),
        "turn_entity is time-independent"
    );
}

#[test]
fn turn_entity_negative() {
    let mut e = make_entity();
    e.orientation = 3.0;
    turn_entity(&mut e, -1.0);
    assert!(
        approx_eq(e.orientation, 2.0, EPS),
        "expected 2.0, got {}",
        e.orientation
    );
}

// ─── apply_angular_momentum ────────────────────────────────────────────────────────────

#[test]
fn apply_angular_momentum_basic() {
    // entity with moment=1.0, dt=1 => orientation += 1 * 1 = 1
    let mut e = make_entity();
    e.orientation = 0.0;
    e.moment = 1.0;
    let g = make_globals(1.0);
    apply_angular_momentum(&mut e, &g);
    assert!(
        approx_eq(e.orientation, 1.0, EPS),
        "expected orientation 1.0, got {}",
        e.orientation
    );
}

#[test]
fn apply_angular_momentum_zero_moment() {
    let mut e = make_entity();
    e.orientation = 2.0;
    e.moment = 0.0;
    let g = make_globals(1.0);
    apply_angular_momentum(&mut e, &g);
    assert!(
        approx_eq(e.orientation, 2.0, EPS),
        "zero moment should not change orientation"
    );
}

#[test]
fn apply_angular_momentum_does_not_change_moment_field() {
    let mut e = make_entity();
    e.moment = 3.0;
    let g = make_globals(1.0);
    apply_angular_momentum(&mut e, &g);
    assert!(
        approx_eq(e.moment, 3.0, EPS),
        "apply_angular_momentum should not alter moment field"
    );
}

#[test]
fn apply_angular_momentum_scales_with_dt() {
    // dt=2 => twice the angular displacement
    let mut e1 = make_entity();
    let mut e2 = make_entity();
    e1.moment = 1.0;
    e2.moment = 1.0;
    let g1 = make_globals(1.0);
    let g2 = make_globals(2.0);
    apply_angular_momentum(&mut e1, &g1);
    apply_angular_momentum(&mut e2, &g2);
    assert!(
        approx_eq(e2.orientation, e1.orientation * 2.0, EPS),
        "doubling dt should double angular displacement"
    );
}

// ─── collision_circles ───────────────────────────────────────────────────────

#[test]
fn collision_circles_overlapping() {
    // Two circles of radius 5 at distance 3 apart — clearly overlapping
    let pos0 = Vec2::new(0.0, 0.0);
    let pos1 = Vec2::new(3.0, 0.0);
    assert!(
        collision_circles(pos0, 5.0, pos1, 5.0),
        "overlapping circles should collide"
    );
}

#[test]
fn collision_circles_non_overlapping() {
    // Two circles of radius 1 at distance 10 apart — no collision
    let pos0 = Vec2::new(0.0, 0.0);
    let pos1 = Vec2::new(10.0, 0.0);
    assert!(
        !collision_circles(pos0, 1.0, pos1, 1.0),
        "non-overlapping circles should not collide"
    );
}

#[test]
fn collision_circles_exactly_touching_not_colliding() {
    // Distance = sum of radii → strict < means NOT colliding
    let pos0 = Vec2::new(0.0, 0.0);
    let pos1 = Vec2::new(10.0, 0.0);
    // radii sum = 10, distance = 10: d² = 100, (r0+r1)² = 100 => not strictly less
    assert!(
        !collision_circles(pos0, 5.0, pos1, 5.0),
        "circles touching exactly (not overlapping) should not collide (strict <)"
    );
}

#[test]
fn collision_circles_symmetry() {
    let pos0 = Vec2::new(1.0, 2.0);
    let pos1 = Vec2::new(4.0, 6.0);
    let r0 = 2.0;
    let r1 = 3.0;
    assert_eq!(
        collision_circles(pos0, r0, pos1, r1),
        collision_circles(pos1, r1, pos0, r0),
        "collision detection should be symmetric"
    );
}

#[test]
fn collision_circles_same_position() {
    // Two circles at same position always collide (d²=0 < (r0+r1)²)
    let pos = Vec2::new(3.0, 4.0);
    assert!(
        collision_circles(pos, 1.0, pos, 1.0),
        "circles at same position should always collide"
    );
}

#[test]
fn collision_circles_zero_radius_inside() {
    // A point (r=0) inside a larger circle
    let pos_circle = Vec2::new(0.0, 0.0);
    let pos_point = Vec2::new(1.0, 0.0);
    assert!(
        collision_circles(pos_circle, 5.0, pos_point, 0.0),
        "point inside circle should collide"
    );
}

#[test]
fn collision_circles_diagonal_overlap() {
    // 3-4-5 triangle: distance=5, radii sum=6 => overlap
    let pos0 = Vec2::new(0.0, 0.0);
    let pos1 = Vec2::new(3.0, 4.0); // distance = 5
    assert!(
        collision_circles(pos0, 3.0, pos1, 3.0),
        "circles with distance 5 and combined radius 6 should collide"
    );
}

#[test]
fn collision_circles_large_radii() {
    // Giant circles spanning huge distances
    let pos0 = Vec2::new(0.0, 0.0);
    let pos1 = Vec2::new(1000.0, 0.0);
    assert!(
        collision_circles(pos0, 800.0, pos1, 800.0),
        "large overlapping circles should collide"
    );
}

// ─── collision_point ─────────────────────────────────────────────────────────

#[test]
fn collision_point_inside() {
    // Point at (1,0) inside circle centered at origin with radius 5
    assert!(
        collision_point(Vec2::new(1.0, 0.0), Vec2::new(0.0, 0.0), 5.0),
        "point inside circle should collide"
    );
}

#[test]
fn collision_point_outside() {
    // Point at (10,0) outside circle at origin with radius 5
    assert!(
        !collision_point(Vec2::new(10.0, 0.0), Vec2::new(0.0, 0.0), 5.0),
        "point outside circle should not collide"
    );
}

#[test]
fn collision_point_on_boundary_not_colliding() {
    // Point exactly on boundary: d²=r² => not strictly less => no collision
    let pos_point = Vec2::new(5.0, 0.0);
    let pos_circle = Vec2::new(0.0, 0.0);
    assert!(
        !collision_point(pos_point, pos_circle, 5.0),
        "point exactly on circle boundary should not collide (strict <)"
    );
}

#[test]
fn collision_point_at_center() {
    // Point at circle center always inside (d²=0 < r²)
    assert!(
        collision_point(Vec2::new(3.0, 4.0), Vec2::new(3.0, 4.0), 1.0),
        "point at circle center should always collide"
    );
}

#[test]
fn collision_point_zero_radius() {
    // Zero radius circle: only a coincident point would collide, but d²=0 < 0 is false
    // So nothing ever collides with a zero-radius circle
    assert!(
        !collision_point(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), 0.0),
        "zero-radius circle should not collide even with coincident point"
    );
}

#[test]
fn collision_point_diagonal_inside() {
    // Point at (3, 4): d² = 25, radius = 6, r² = 36 => inside
    assert!(
        collision_point(Vec2::new(3.0, 4.0), Vec2::new(0.0, 0.0), 6.0),
        "point at distance 5 inside circle of radius 6 should collide"
    );
}

#[test]
fn collision_point_diagonal_outside() {
    // Point at (3, 4): d² = 25, radius = 4, r² = 16 => outside
    assert!(
        !collision_point(Vec2::new(3.0, 4.0), Vec2::new(0.0, 0.0), 4.0),
        "point at distance 5 outside circle of radius 4 should not collide"
    );
}
