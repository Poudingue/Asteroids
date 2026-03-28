//! Collision detection primitives.
//! Circle-circle, point-circle, polygon-circle, and entity-vs-entity tests.

use crate::math_utils::*;
use crate::objects::Entity;

pub fn collision_circles(pos0: Vec2, r0: f64, pos1: Vec2, r1: f64) -> bool {
    distance_squared(pos0, pos1) < squared(r0 + r1)
}

pub fn collision_point(pos_point: Vec2, pos_circle: Vec2, radius: f64) -> bool {
    distance_squared(pos_point, pos_circle) < squared(radius)
}

pub fn collisions_points(points: &[Vec2], pos_circle: Vec2, radius: f64) -> bool {
    points.iter().any(|&p| collision_point(p, pos_circle, radius))
}

pub fn collision_poly(
    pos: Vec2,
    poly: &[(f64, f64)],
    rotat: f64,
    circle_pos: Vec2,
    radius: f64,
) -> bool {
    let pts = translate_polygon(&polygon_to_cartesian(poly, rotat, 1.0), pos);
    collisions_points(&pts, circle_pos, radius)
}

/// Test collision between two entities.
/// `precis`: true = polygon check after circle broadphase; false = circle only.
pub fn collision_entities(
    obj1: &Entity,
    obj2: &Entity,
    precis: bool,
    advanced_hitbox: bool,
) -> bool {
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
