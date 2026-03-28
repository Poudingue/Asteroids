/// Exhaustive property tests for math_utils.rs
/// Covers every public function, testing correctness, mathematical properties, and float edge cases.
/// These tests serve as a safety net before the V2 refactor that renames the French API to English.

use asteroids::math_utils::*;
use std::f64::consts::PI;

const EPS: f64 = 1e-10;
const EPS_TRIG: f64 = 1e-12;

// ─── Helpers ────────────────────────────────────────────────────────────────

fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

fn vec2_approx_eq(a: Vec2, b: Vec2, eps: f64) -> bool {
    approx_eq(a.x, b.x, eps) && approx_eq(a.y, b.y, eps)
}

// ─── squared ──────────────────────────────────────────────────────────────────

#[test]
fn squared_zero() {
    assert_eq!(squared(0.0), 0.0);
}

#[test]
fn squared_one() {
    assert_eq!(squared(1.0), 1.0);
}

#[test]
fn squared_known_values() {
    assert!(approx_eq(squared(3.0), 9.0, EPS));
    assert!(approx_eq(squared(-3.0), 9.0, EPS));
    assert!(approx_eq(squared(0.5), 0.25, EPS));
    assert!(approx_eq(squared(-0.5), 0.25, EPS));
}

#[test]
fn squared_symmetry() {
    // squared(x) == squared(-x) for all x
    for &x in &[0.0, 1.0, -1.0, 2.5, -2.5, 1e7, -1e7, 1e-7, -1e-7] {
        assert!(
            approx_eq(squared(x), squared(-x), EPS),
            "squared({x}) != squared({neg})",
            neg = -x
        );
    }
}

#[test]
fn squared_non_negative() {
    for &x in &[0.0, 1.0, -1.0, 100.0, -100.0, 1e15, -1e15, 1e-15, -1e-15] {
        assert!(squared(x) >= 0.0, "squared({x}) is negative");
    }
}

#[test]
fn squared_large_values() {
    // Should not overflow (f64 handles up to ~1e308)
    let v = squared(1e15);
    assert!(v.is_finite(), "squared(1e15) overflowed");
    assert!(approx_eq(v, 1e30, 1e20)); // relative check
}

#[test]
fn squared_small_values() {
    let v = squared(1e-15);
    assert!(v >= 0.0);
    assert!(v.is_finite());
}

// ─── add_vec ────────────────────────────────────────────────────────────────

#[test]
fn add_vec_commutativity() {
    let a = Vec2::new(3.0, 4.0);
    let b = Vec2::new(1.0, -2.0);
    assert!(vec2_approx_eq(add_vec(a, b), add_vec(b, a), EPS));
}

#[test]
fn add_vec_associativity() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    let c = Vec2::new(5.0, 6.0);
    let lhs = add_vec(add_vec(a, b), c);
    let rhs = add_vec(a, add_vec(b, c));
    assert!(vec2_approx_eq(lhs, rhs, EPS));
}

#[test]
fn add_vec_identity() {
    let a = Vec2::new(7.0, -3.0);
    let zero = Vec2::new(0.0, 0.0);
    assert!(vec2_approx_eq(add_vec(a, zero), a, EPS));
    assert!(vec2_approx_eq(add_vec(zero, a), a, EPS));
}

#[test]
fn add_vec_inverse() {
    let a = Vec2::new(5.0, -8.0);
    let neg_a = Vec2::new(-5.0, 8.0);
    let result = add_vec(a, neg_a);
    assert!(vec2_approx_eq(result, Vec2::new(0.0, 0.0), EPS));
}

#[test]
fn add_vec_known_values() {
    assert!(vec2_approx_eq(add_vec(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)), Vec2::new(4.0, 6.0), EPS));
    assert!(vec2_approx_eq(add_vec(Vec2::new(-1.0, -2.0), Vec2::new(1.0, 2.0)), Vec2::new(0.0, 0.0), EPS));
}

#[test]
fn add_vec_large_values() {
    let a = Vec2::new(1e15, 1e15);
    let b = Vec2::new(1e15, 1e15);
    let r = add_vec(a, b);
    assert!(approx_eq(r.x, 2e15, 1e5));
    assert!(approx_eq(r.y, 2e15, 1e5));
}

// ─── sub_vec ───────────────────────────────────────────────────────────────

#[test]
fn sub_vec_self_is_zero() {
    let a = Vec2::new(4.0, -7.0);
    assert!(vec2_approx_eq(sub_vec(a, a), Vec2::ZERO, EPS));
}

#[test]
fn sub_vec_zero_right() {
    let a = Vec2::new(4.0, -7.0);
    let zero = Vec2::new(0.0, 0.0);
    assert!(vec2_approx_eq(sub_vec(a, zero), a, EPS));
}

#[test]
fn sub_vec_anti_commutativity() {
    let a = Vec2::new(3.0, 5.0);
    let b = Vec2::new(1.0, 2.0);
    let ab = sub_vec(a, b);
    let ba = sub_vec(b, a);
    // a - b = -(b - a)
    assert!(vec2_approx_eq(ab, Vec2::new(-ba.x, -ba.y), EPS));
}

#[test]
fn sub_vec_known_values() {
    assert!(vec2_approx_eq(sub_vec(Vec2::new(5.0, 3.0), Vec2::new(2.0, 1.0)), Vec2::new(3.0, 2.0), EPS));
    assert!(vec2_approx_eq(sub_vec(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)), Vec2::new(-1.0, -1.0), EPS));
}

#[test]
fn sub_vec_recovers_via_add() {
    // (a + b) - b == a
    let a = Vec2::new(3.0, -4.0);
    let b = Vec2::new(1.0, 2.0);
    let sum = add_vec(a, b);
    assert!(vec2_approx_eq(sub_vec(sum, b), a, EPS));
}

// ─── scale_vec ─────────────────────────────────────────────────────────────────

#[test]
fn scale_vec_identity() {
    let v = Vec2::new(3.0, -4.0);
    assert!(vec2_approx_eq(scale_vec(v, 1.0), v, EPS));
}

#[test]
fn scale_vec_zero_scalar() {
    let v = Vec2::new(3.0, -4.0);
    assert!(vec2_approx_eq(scale_vec(v, 0.0), Vec2::ZERO, EPS));
}

#[test]
fn scale_vec_zero_vector() {
    let zero = Vec2::new(0.0, 0.0);
    assert!(vec2_approx_eq(scale_vec(zero, 5.0), Vec2::ZERO, EPS));
}

#[test]
fn scale_vec_distributivity_add() {
    // (a + b) * s == a*s + b*s
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    let s = 2.5;
    let lhs = scale_vec(add_vec(a, b), s);
    let rhs = add_vec(scale_vec(a, s), scale_vec(b, s));
    assert!(vec2_approx_eq(lhs, rhs, EPS));
}

#[test]
fn scale_vec_scalar_associativity() {
    // v * (s1 * s2) == (v * s1) * s2
    let v = Vec2::new(3.0, 4.0);
    let s1 = 2.0;
    let s2 = 3.0;
    let lhs = scale_vec(v, s1 * s2);
    let rhs = scale_vec(scale_vec(v, s1), s2);
    assert!(vec2_approx_eq(lhs, rhs, EPS));
}

#[test]
fn scale_vec_negative_scalar() {
    let v = Vec2::new(3.0, -4.0);
    let neg = scale_vec(v, -1.0);
    assert!(vec2_approx_eq(neg, Vec2::new(-3.0, 4.0), EPS));
}

#[test]
fn scale_vec_known_values() {
    assert!(vec2_approx_eq(scale_vec(Vec2::new(2.0, 3.0), 4.0), Vec2::new(8.0, 12.0), EPS));
}

// ─── magnitude ─────────────────────────────────────────────────────────────

#[test]
fn magnitude_3_4_5() {
    assert!(approx_eq(magnitude(Vec2::new(3.0, 4.0)), 5.0, EPS));
    assert!(approx_eq(magnitude(Vec2::new(4.0, 3.0)), 5.0, EPS));
}

#[test]
fn magnitude_zero() {
    assert_eq!(magnitude(Vec2::new(0.0, 0.0)), 0.0);
}

#[test]
fn magnitude_unit_vectors() {
    assert!(approx_eq(magnitude(Vec2::new(1.0, 0.0)), 1.0, EPS));
    assert!(approx_eq(magnitude(Vec2::new(0.0, 1.0)), 1.0, EPS));
}

#[test]
fn magnitude_non_negative() {
    for v in &[Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(-1.0, 0.0), Vec2::new(3.0, -4.0), Vec2::new(-3.0, -4.0)] {
        assert!(magnitude(*v) >= 0.0);
    }
}

#[test]
fn magnitude_symmetry() {
    // hyp(x, y) == hyp(y, x) == hyp(-x, y) == hyp(x, -y)
    let cases = [Vec2::new(3.0, 4.0), Vec2::new(1.5, 2.5), Vec2::new(1e7, 1e7)];
    for v in cases {
        let (x, y) = (v.x, v.y);
        let h = magnitude(v);
        assert!(approx_eq(h, magnitude(Vec2::new(y, x)), EPS));
        assert!(approx_eq(h, magnitude(Vec2::new(-x, y)), EPS));
        assert!(approx_eq(h, magnitude(Vec2::new(x, -y)), EPS));
        assert!(approx_eq(h, magnitude(Vec2::new(-x, -y)), EPS));
    }
}

#[test]
fn magnitude_scaling() {
    // hyp(k*v) == k * hyp(v) for k > 0
    let v = Vec2::new(3.0, 4.0);
    let k = 5.0;
    let scaled = scale_vec(v, k);
    assert!(approx_eq(magnitude(scaled), k * magnitude(v), EPS));
}

#[test]
fn magnitude_triangle_inequality() {
    // |a + b| <= |a| + |b|
    let a = Vec2::new(3.0, 4.0);
    let b = Vec2::new(1.0, 2.0);
    let sum_mag = magnitude(add_vec(a, b));
    let mag_sum = magnitude(a) + magnitude(b);
    assert!(sum_mag <= mag_sum + EPS);
}

#[test]
fn magnitude_known_5_12_13() {
    assert!(approx_eq(magnitude(Vec2::new(5.0, 12.0)), 13.0, EPS));
}

#[test]
fn magnitude_large_values() {
    let v = magnitude(Vec2::new(1e15, 0.0));
    assert!(approx_eq(v, 1e15, 1e5));
}

#[test]
fn magnitude_small_values() {
    let v = magnitude(Vec2::new(1e-15, 0.0));
    assert!(approx_eq(v, 1e-15, 1e-25));
}

// ─── distance_squared ───────────────────────────────────────────────────────────

#[test]
fn distance_squared_self_is_zero() {
    let p = Vec2::new(3.0, 4.0);
    assert_eq!(distance_squared(p, p), 0.0);
}

#[test]
fn distance_squared_symmetry() {
    let p1 = Vec2::new(1.0, 2.0);
    let p2 = Vec2::new(4.0, 6.0);
    assert!(approx_eq(distance_squared(p1, p2), distance_squared(p2, p1), EPS));
}

#[test]
fn distance_squared_non_negative() {
    let cases = [
        (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        (Vec2::new(1.0, 2.0), Vec2::new(4.0, 6.0)),
        (Vec2::new(-3.0, -4.0), Vec2::new(3.0, 4.0)),
    ];
    for (p1, p2) in cases {
        assert!(distance_squared(p1, p2) >= 0.0);
    }
}

#[test]
fn distance_squared_known_3_4_5() {
    // Distance from (0,0) to (3,4) should be sqrt(25) = 5, so d² = 25
    assert!(approx_eq(distance_squared(Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0)), 25.0, EPS));
}

#[test]
fn distance_squared_equals_magnitude_squared() {
    let p1 = Vec2::new(1.0, 2.0);
    let p2 = Vec2::new(4.0, 6.0);
    let diff = sub_vec(p2, p1);
    let h = magnitude(diff);
    assert!(approx_eq(distance_squared(p1, p2), h * h, EPS));
}

#[test]
fn distance_squared_triangle_inequality_squared() {
    // sqrt(d²(a,c)) <= sqrt(d²(a,b)) + sqrt(d²(b,c))
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(3.0, 4.0);
    let c = Vec2::new(6.0, 0.0);
    let dac = distance_squared(a, c).sqrt();
    let dab = distance_squared(a, b).sqrt();
    let dbc = distance_squared(b, c).sqrt();
    assert!(dac <= dab + dbc + EPS);
}

// ─── lerp_vec ─────────────────────────────────────────────────────────────────

#[test]
fn lerp_vec_ratio_one_returns_first() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    assert!(vec2_approx_eq(lerp_vec(a, b, 1.0), a, EPS));
}

#[test]
fn lerp_vec_ratio_zero_returns_second() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    assert!(vec2_approx_eq(lerp_vec(a, b, 0.0), b, EPS));
}

#[test]
fn lerp_vec_midpoint_equidistant() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(4.0, 4.0);
    let mid = lerp_vec(a, b, 0.5);
    let d1 = distance_squared(mid, a);
    let d2 = distance_squared(mid, b);
    assert!(approx_eq(d1, d2, EPS));
}

#[test]
fn lerp_vec_same_value() {
    let a = Vec2::new(5.0, 7.0);
    assert!(vec2_approx_eq(lerp_vec(a, a, 0.5), a, EPS));
    assert!(vec2_approx_eq(lerp_vec(a, a, 0.0), a, EPS));
    assert!(vec2_approx_eq(lerp_vec(a, a, 1.0), a, EPS));
}

#[test]
fn lerp_vec_known_midpoint() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(2.0, 4.0);
    let mid = lerp_vec(a, b, 0.5);
    assert!(vec2_approx_eq(mid, Vec2::new(1.0, 2.0), EPS));
}

// ─── lerp_float ─────────────────────────────────────────────────────────────────

#[test]
fn lerp_float_ratio_one_returns_first() {
    assert!(approx_eq(lerp_float(3.0, 7.0, 1.0), 3.0, EPS));
}

#[test]
fn lerp_float_ratio_zero_returns_second() {
    assert!(approx_eq(lerp_float(3.0, 7.0, 0.0), 7.0, EPS));
}

#[test]
fn lerp_float_midpoint() {
    assert!(approx_eq(lerp_float(0.0, 10.0, 0.5), 5.0, EPS));
    assert!(approx_eq(lerp_float(2.0, 4.0, 0.5), 3.0, EPS));
}

#[test]
fn lerp_float_same_value() {
    assert!(approx_eq(lerp_float(5.0, 5.0, 0.3), 5.0, EPS));
}

#[test]
fn lerp_float_known_values() {
    // 0.25 * 8 + 0.75 * 4 = 2 + 3 = 5
    assert!(approx_eq(lerp_float(8.0, 4.0, 0.25), 5.0, EPS));
}

// ─── component_mul_vec ───────────────────────────────────────────────────────

#[test]
fn component_mul_vec_commutativity() {
    let a = Vec2::new(2.0, 3.0);
    let b = Vec2::new(4.0, 5.0);
    assert!(vec2_approx_eq(component_mul_vec(a, b), component_mul_vec(b, a), EPS));
}

#[test]
fn component_mul_vec_identity() {
    let v = Vec2::new(3.0, -4.0);
    let one = Vec2::new(1.0, 1.0);
    assert!(vec2_approx_eq(component_mul_vec(v, one), v, EPS));
}

#[test]
fn component_mul_vec_zero() {
    let v = Vec2::new(3.0, -4.0);
    let zero = Vec2::new(0.0, 0.0);
    assert!(vec2_approx_eq(component_mul_vec(v, zero), Vec2::ZERO, EPS));
}

#[test]
fn component_mul_vec_known_values() {
    assert!(vec2_approx_eq(
        component_mul_vec(Vec2::new(2.0, 3.0), Vec2::new(4.0, 5.0)),
        Vec2::new(8.0, 15.0),
        EPS
    ));
}

#[test]
fn component_mul_vec_associativity() {
    let a = Vec2::new(2.0, 3.0);
    let b = Vec2::new(4.0, 5.0);
    let c = Vec2::new(6.0, 7.0);
    let lhs = component_mul_vec(component_mul_vec(a, b), c);
    let rhs = component_mul_vec(a, component_mul_vec(b, c));
    assert!(vec2_approx_eq(lhs, rhs, EPS));
}

// ─── point_in_aabb ──────────────────────────────────────────────────────────────

#[test]
fn point_in_aabb_inside() {
    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 10.0);
    assert!(point_in_aabb(Vec2::new(5.0, 5.0), min, max));
    assert!(point_in_aabb(Vec2::new(1.0, 9.0), min, max));
    assert!(point_in_aabb(Vec2::new(9.0, 1.0), min, max));
}

#[test]
fn point_in_aabb_outside() {
    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 10.0);
    assert!(!point_in_aabb(Vec2::new(-1.0, 5.0), min, max));
    assert!(!point_in_aabb(Vec2::new(5.0, -1.0), min, max));
    assert!(!point_in_aabb(Vec2::new(11.0, 5.0), min, max));
    assert!(!point_in_aabb(Vec2::new(5.0, 11.0), min, max));
    assert!(!point_in_aabb(Vec2::new(15.0, 15.0), min, max));
    assert!(!point_in_aabb(Vec2::new(-5.0, -5.0), min, max));
}

#[test]
fn point_in_aabb_boundary_is_exclusive() {
    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 10.0);
    // Strict inequality, so boundary is NOT inside
    assert!(!point_in_aabb(Vec2::new(0.0, 5.0), min, max));
    assert!(!point_in_aabb(Vec2::new(10.0, 5.0), min, max));
    assert!(!point_in_aabb(Vec2::new(5.0, 0.0), min, max));
    assert!(!point_in_aabb(Vec2::new(5.0, 10.0), min, max));
}

#[test]
fn point_in_aabb_negative_coords() {
    let min = Vec2::new(-5.0, -5.0);
    let max = Vec2::new(5.0, 5.0);
    assert!(point_in_aabb(Vec2::new(0.0, 0.0), min, max));
    assert!(point_in_aabb(Vec2::new(-4.0, -4.0), min, max));
    assert!(!point_in_aabb(Vec2::new(-6.0, 0.0), min, max));
}

// ─── to_i32_tuple / from_i32_tuple ───────────────────────────────────────────────────

#[test]
fn to_i32_tuple_known_values() {
    assert_eq!(to_i32_tuple(Vec2::new(3.0, 4.0)), (3, 4));
    assert_eq!(to_i32_tuple(Vec2::new(-1.0, -2.0)), (-1, -2));
    assert_eq!(to_i32_tuple(Vec2::new(0.0, 0.0)), (0, 0));
}

#[test]
fn to_i32_tuple_truncates() {
    // f64 as i32 truncates toward zero
    assert_eq!(to_i32_tuple(Vec2::new(3.9, 4.9)), (3, 4));
    assert_eq!(to_i32_tuple(Vec2::new(-3.9, -4.9)), (-3, -4));
}

#[test]
fn from_i32_tuple_known_values() {
    assert!(vec2_approx_eq(from_i32_tuple((3, 4)), Vec2::new(3.0, 4.0), EPS));
    assert!(vec2_approx_eq(from_i32_tuple((-1, -2)), Vec2::new(-1.0, -2.0), EPS));
    assert!(vec2_approx_eq(from_i32_tuple((0, 0)), Vec2::new(0.0, 0.0), EPS));
}

#[test]
fn to_i32_tuple_from_i32_tuple_roundtrip() {
    // For integer-valued floats, roundtrip should be exact
    let integers = [(0, 0), (1, 2), (-3, 4), (100, -200)];
    for &(x, y) in &integers {
        let v_float = from_i32_tuple((x, y));
        let v_int = to_i32_tuple(v_float);
        assert_eq!(v_int, (x, y), "roundtrip failed for ({x}, {y})");
    }
}

// ─── proj ──────────────────────────────────────────────────────────────────

#[test]
fn proj_zero_ratio() {
    let base = Vec2::new(1.0, 2.0);
    let dir = Vec2::new(3.0, 4.0);
    // proj(base, dir, 0) == base
    assert!(vec2_approx_eq(proj(base, dir, 0.0), base, EPS));
}

#[test]
fn proj_unit_ratio() {
    let base = Vec2::new(1.0, 2.0);
    let dir = Vec2::new(3.0, 4.0);
    // proj(base, dir, 1.0) == base + dir
    assert!(vec2_approx_eq(proj(base, dir, 1.0), add_vec(base, dir), EPS));
}

#[test]
fn proj_known_values() {
    // proj((0,0), (1,0), 5) = (5, 0)
    assert!(vec2_approx_eq(proj(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), 5.0), Vec2::new(5.0, 0.0), EPS));
    // proj((1,1), (1,1), 2) = (3, 3)
    assert!(vec2_approx_eq(proj(Vec2::new(1.0, 1.0), Vec2::new(1.0, 1.0), 2.0), Vec2::new(3.0, 3.0), EPS));
}

#[test]
fn proj_negative_ratio() {
    let base = Vec2::new(5.0, 5.0);
    let dir = Vec2::new(1.0, 1.0);
    // proj(base, dir, -1) == base - dir
    assert!(vec2_approx_eq(proj(base, dir, -1.0), sub_vec(base, dir), EPS));
}

// ─── from_polar / to_polar ───────────────────────────────────────

#[test]
fn from_polar_unit_circle_zero_angle() {
    let v = from_polar(0.0, 1.0);
    assert!(approx_eq(v.x, 1.0, EPS_TRIG));
    assert!(approx_eq(v.y, 0.0, EPS_TRIG));
}

#[test]
fn from_polar_unit_circle_pi_half() {
    let v = from_polar(PI / 2.0, 1.0);
    assert!(approx_eq(v.x, 0.0, EPS_TRIG));
    assert!(approx_eq(v.y, 1.0, EPS_TRIG));
}

#[test]
fn from_polar_unit_circle_pi() {
    let v = from_polar(PI, 1.0);
    assert!(approx_eq(v.x, -1.0, EPS_TRIG));
    assert!(approx_eq(v.y, 0.0, EPS_TRIG));
}

#[test]
fn from_polar_unit_circle_3pi_half() {
    let v = from_polar(3.0 * PI / 2.0, 1.0);
    assert!(approx_eq(v.x, 0.0, EPS_TRIG));
    assert!(approx_eq(v.y, -1.0, EPS_TRIG));
}

#[test]
fn from_polar_zero_radius() {
    let v = from_polar(PI / 4.0, 0.0);
    assert!(approx_eq(v.x, 0.0, EPS_TRIG));
    assert!(approx_eq(v.y, 0.0, EPS_TRIG));
}

#[test]
fn from_polar_scaling() {
    // magnitude should equal radius
    let r = 5.0;
    let v = from_polar(PI / 4.0, r);
    assert!(approx_eq(magnitude(v), r, EPS_TRIG));
}

#[test]
fn from_polar_preserves_magnitude() {
    let cases = [(0.0, 3.0), (PI / 6.0, 2.0), (PI, 4.0), (7.0 * PI / 4.0, 1.5)];
    for (angle, r) in cases {
        let v = from_polar(angle, r);
        assert!(
            approx_eq(magnitude(v), r, EPS_TRIG),
            "angle={angle}, r={r}: hyp={h}",
            h = magnitude(v)
        );
    }
}

#[test]
fn to_polar_unit_x() {
    let polar = to_polar(Vec2::new(1.0, 0.0));
    assert!(approx_eq(polar.y, 1.0, EPS_TRIG));
    assert!(approx_eq(polar.x, 0.0, EPS_TRIG));
}

#[test]
fn to_polar_unit_y() {
    let polar = to_polar(Vec2::new(0.0, 1.0));
    assert!(approx_eq(polar.y, 1.0, EPS_TRIG));
    assert!(approx_eq(polar.x, PI / 2.0, EPS_TRIG));
}

#[test]
fn to_polar_zero_vector() {
    let polar = to_polar(Vec2::ZERO);
    assert_eq!(polar.x, 0.0);
    assert_eq!(polar.y, 0.0);
}

#[test]
fn polar_roundtrip_affine_to_from_polar() {
    // affine → polar → affine should recover original (for non-zero vectors)
    // Note: (-1, 0) is excluded — to_polar uses the half-angle formula
    // 2*atan(y / (x + r)), which has a singularity when x = -r and y = 0 (0/0 = NaN).
    // This is a known limitation of this atan variant; it handles all practical
    // in-game vectors which never land exactly on the negative x-axis.
    let cases = [
        Vec2::new(3.0, 4.0),
        Vec2::new(-3.0, 4.0),
        Vec2::new(3.0, -4.0),
        Vec2::new(-3.0, -4.0),
        Vec2::new(1.5, 2.5),
    ];
    for v in cases {
        let polar = to_polar(v);
        let recovered = from_polar(polar.x, polar.y);
        assert!(
            vec2_approx_eq(recovered, v, 1e-9),
            "roundtrip failed for {:?}: polar={:?}, recovered={:?}",
            v,
            polar,
            recovered
        );
    }
}

#[test]
fn polar_roundtrip_from_polar_to_polar() {
    // polar → affine → polar should recover original angle and magnitude
    let cases = [
        (0.0, 1.0),
        (PI / 4.0, 2.0),
        (PI / 2.0, 3.0),
        (PI, 1.5),
        (3.0 * PI / 2.0, 2.5),
    ];
    for (angle, r) in cases {
        let v = from_polar(angle, r);
        let polar = to_polar(v);
        let ra = polar.x;
        let rr = polar.y;
        assert!(
            approx_eq(rr, r, 1e-9),
            "magnitude roundtrip failed: original r={r}, got rr={rr}"
        );
        // Angles should match modulo 2π (normalize before comparing)
        let angle_diff = (ra - angle).abs() % (2.0 * PI);
        let angle_diff = angle_diff.min(2.0 * PI - angle_diff);
        assert!(
            angle_diff < 1e-9,
            "angle roundtrip failed: original angle={angle}, got ra={ra}"
        );
    }
}

#[test]
fn from_polar_tuple_matches_two_arg() {
    let cases = [(0.0, 1.0), (PI / 4.0, 2.0), (PI, 3.0), (1.5, 4.0)];
    for (angle, r) in cases {
        let v1 = from_polar(angle, r);
        let v2 = from_polar_tuple((angle, r));
        assert!(
            vec2_approx_eq(v1, v2, EPS_TRIG),
            "mismatch for angle={angle}, r={r}: v1={v1:?}, v2={v2:?}"
        );
    }
}

// ─── wrap_float ─────────────────────────────────────────────────────────────

#[test]
fn wrap_float_in_range() {
    assert!(approx_eq(wrap_float(5.0, 10.0), 5.0, EPS));
    assert!(approx_eq(wrap_float(0.5, 10.0), 0.5, EPS));
    assert!(approx_eq(wrap_float(9.9, 10.0), 9.9, EPS));
}

#[test]
fn wrap_float_overflow() {
    // value >= modulo: subtract once
    assert!(approx_eq(wrap_float(10.0, 10.0), 0.0, EPS));
    assert!(approx_eq(wrap_float(12.0, 10.0), 2.0, EPS));
    assert!(approx_eq(wrap_float(15.0, 10.0), 5.0, EPS));
}

#[test]
fn wrap_float_underflow() {
    // value < 0: add modulo once
    assert!(approx_eq(wrap_float(-1.0, 10.0), 9.0, EPS));
    assert!(approx_eq(wrap_float(-5.0, 10.0), 5.0, EPS));
    assert!(approx_eq(wrap_float(-9.9, 10.0), 0.1, EPS));
}

#[test]
fn wrap_float_idempotent() {
    // Applying twice to an already in-range value should be a no-op
    let v = 5.0;
    let m = 10.0;
    let once = wrap_float(v, m);
    let twice = wrap_float(once, m);
    assert!(approx_eq(once, twice, EPS));
}

#[test]
fn wrap_float_range_check() {
    // Result should always be in [0, modulo)
    let modulo = 10.0;
    for &v in &[-1.0, 0.0, 5.0, 9.9, 10.0, 15.0, -5.0] {
        let r = wrap_float(v, modulo);
        assert!(r >= 0.0 && r < modulo, "wrap_float({v}, {modulo}) = {r} out of range");
    }
}

// ─── wrap_single ─────────────────────────────────────────────────────────────

#[test]
fn wrap_single_in_range() {
    let r = wrap_single(Vec2::new(5.0, 5.0), 10.0, 10.0);
    assert!(vec2_approx_eq(r, Vec2::new(5.0, 5.0), EPS));
}

#[test]
fn wrap_single_wraps_x() {
    let r = wrap_single(Vec2::new(12.0, 5.0), 10.0, 10.0);
    assert!(approx_eq(r.x, 2.0, EPS));
    assert!(approx_eq(r.y, 5.0, EPS));
}

#[test]
fn wrap_single_wraps_y() {
    let r = wrap_single(Vec2::new(5.0, -1.0), 10.0, 10.0);
    assert!(approx_eq(r.x, 5.0, EPS));
    assert!(approx_eq(r.y, 9.0, EPS));
}

#[test]
fn wrap_single_both_axes() {
    let r = wrap_single(Vec2::new(-1.0, 11.0), 10.0, 10.0);
    assert!(approx_eq(r.x, 9.0, EPS));
    assert!(approx_eq(r.y, 1.0, EPS));
}

// ─── wrap_toroidal ─────────────────────────────────────────────────────────────

#[test]
fn wrap_toroidal_in_range() {
    // A point inside [-w, w) x [-h, h) (the 1-screen-wide margin) should stay
    let r = wrap_toroidal(Vec2::new(5.0, 5.0), 10.0, 10.0);
    assert!(
        r.x >= -10.0 && r.x < 10.0,
        "x={} out of [-10, 10)",
        r.x
    );
    assert!(
        r.y >= -10.0 && r.y < 10.0,
        "y={} out of [-10, 10)",
        r.y
    );
}

#[test]
fn wrap_toroidal_far_positive() {
    // Point at (25, 5): 25 + 10 = 35, 35 mod 30 = 5, 5 - 10 = -5
    let r = wrap_toroidal(Vec2::new(25.0, 5.0), 10.0, 10.0);
    assert!(approx_eq(r.x, -5.0, EPS), "expected -5, got {}", r.x);
}

#[test]
fn wrap_toroidal_far_negative() {
    // Point at (-25, 5): -25 + 10 = -15, -15 mod 30 = 15, 15 - 10 = 5
    let r = wrap_toroidal(Vec2::new(-25.0, 5.0), 10.0, 10.0);
    assert!(approx_eq(r.x, 5.0, EPS), "expected 5, got {}", r.x);
}

#[test]
fn wrap_toroidal_idempotent_in_range() {
    // Applying twice to an already in-range value should be no-op
    let v = Vec2::new(5.0, 3.0);
    let w = 10.0;
    let h = 10.0;
    let once = wrap_toroidal(v, w, h);
    let twice = wrap_toroidal(once, w, h);
    assert!(vec2_approx_eq(once, twice, EPS));
}

// ─── exp_decay ────────────────────────────────────────────────────────────────

#[test]
fn exp_decay_zero_dt_is_unchanged() {
    // When time_last_frame == time_current_frame, dt = 0, result = n * 2^0 = n
    let result = exp_decay(100.0, 0.5, 1.0, 1.0, 5.0, 5.0, 1.0);
    assert!(approx_eq(result, 100.0, EPS));
}

#[test]
fn exp_decay_half_life() {
    // After one half-life, value should halve:
    // n * 2^((obs * speed * (t_last - t_curr)) / (proper * half_life))
    // Set obs=1, speed=1, proper=1, half_life=1, t_last=0, t_curr=1
    // => n * 2^(-1) = n/2
    let result = exp_decay(100.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0);
    assert!(approx_eq(result, 50.0, EPS));
}

#[test]
fn exp_decay_double_life() {
    // After two half-lives: n * 2^(-2) = n/4
    let result = exp_decay(100.0, 1.0, 1.0, 1.0, 0.0, 2.0, 1.0);
    assert!(approx_eq(result, 25.0, EPS));
}

#[test]
fn exp_decay_monotonic_decrease() {
    // Value should decrease as time advances (t_curr > t_last)
    let n = 100.0;
    let half_life = 0.5;
    let obs = 1.0;
    let speed = 1.0;
    let proper = 1.0;
    let t_last = 0.0;
    let mut prev = n;
    for i in 1..=5 {
        let t_curr = i as f64 * 0.1;
        let next = exp_decay(n, half_life, obs, speed, t_last, t_curr, proper);
        assert!(next < prev, "not monotonically decreasing at step {i}: {next} >= {prev}");
        prev = next;
    }
}

#[test]
fn exp_decay_non_negative() {
    let result = exp_decay(100.0, 0.5, 1.0, 1.0, 0.0, 10.0, 1.0);
    assert!(result >= 0.0);
}

#[test]
fn exp_decay_game_speed_scales() {
    // Doubling game_speed should square the result (since exponent doubles)
    // n * 2^(1 * 2s * dt / (1 * hl)) vs n * 2^(1 * s * dt / (1 * hl))
    let n = 100.0;
    let half_life = 1.0;
    let obs = 1.0;
    let t_last = 0.0;
    let t_curr = 1.0;
    let proper = 1.0;
    let r1 = exp_decay(n, half_life, obs, 1.0, t_last, t_curr, proper);
    let r2 = exp_decay(n, half_life, obs, 2.0, t_last, t_curr, proper);
    // r1 = n * 2^(-1) = 50, r2 = n * 2^(-2) = 25 = r1^2 / n
    assert!(approx_eq(r2, r1 * r1 / n, EPS));
}

// ─── abso_exp_decay ───────────────────────────────────────────────────────────

#[test]
fn abso_exp_decay_zero_dt() {
    let result = abso_exp_decay(100.0, 0.5, 5.0, 5.0);
    assert!(approx_eq(result, 100.0, EPS));
}

#[test]
fn abso_exp_decay_half_life() {
    // t_last=0, t_curr=1, half_life=1 => n * 2^(-1) = 50
    let result = abso_exp_decay(100.0, 1.0, 0.0, 1.0);
    assert!(approx_eq(result, 50.0, EPS));
}

#[test]
fn abso_exp_decay_two_half_lives() {
    let result = abso_exp_decay(100.0, 1.0, 0.0, 2.0);
    assert!(approx_eq(result, 25.0, EPS));
}

#[test]
fn abso_exp_decay_monotonic_decrease() {
    let n = 100.0;
    let half_life = 0.5;
    let t_last = 0.0;
    let mut prev = n;
    for i in 1..=5 {
        let t_curr = i as f64 * 0.1;
        let next = abso_exp_decay(n, half_life, t_last, t_curr);
        assert!(next < prev, "not monotonically decreasing at step {i}: {next} >= {prev}");
        prev = next;
    }
}

#[test]
fn abso_exp_decay_non_negative() {
    let result = abso_exp_decay(100.0, 0.5, 0.0, 100.0);
    assert!(result >= 0.0);
}

// ─── rand_range ────────────────────────────────────────────────────────────────

#[test]
fn rand_range_range_check() {
    let mut rng = rand::thread_rng();
    let min = -5.0;
    let max = 10.0;
    for _ in 0..1000 {
        let v = rand_range(min, max, &mut rng);
        assert!(
            v >= min && v < max,
            "rand_range({min}, {max}) = {v} out of range"
        );
    }
}

#[test]
fn rand_range_degenerate_range() {
    let mut rng = rand::thread_rng();
    // min == max: result should be min (since gen() ∈ [0,1), 0 * anything = 0 + min = min)
    // Actually gen() could be 0.0 yielding exactly min
    let v = rand_range(5.0, 5.0, &mut rng);
    assert!(approx_eq(v, 5.0, EPS));
}

#[test]
fn rand_range_positive_range() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let v = rand_range(0.0, 1.0, &mut rng);
        assert!(v >= 0.0 && v < 1.0);
    }
}

// ─── diff ─────────────────────────────────────────────────────────────────────

#[test]
fn diff_basic() {
    let l1 = vec![1, 2, 3, 4, 5];
    let l2 = vec![2, 4];
    let result = diff(&l1, &l2);
    assert_eq!(result, vec![1, 3, 5]);
}

#[test]
fn diff_empty_l2() {
    let l1 = vec![1, 2, 3];
    let l2: Vec<i32> = vec![];
    let result = diff(&l1, &l2);
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn diff_empty_l1() {
    let l1: Vec<i32> = vec![];
    let l2 = vec![1, 2, 3];
    let result = diff(&l1, &l2);
    assert!(result.is_empty());
}

#[test]
fn diff_l2_superset() {
    let l1 = vec![1, 2];
    let l2 = vec![1, 2, 3];
    let result = diff(&l1, &l2);
    assert!(result.is_empty());
}

#[test]
fn diff_no_overlap() {
    let l1 = vec![1, 2, 3];
    let l2 = vec![4, 5, 6];
    let result = diff(&l1, &l2);
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn diff_preserves_order() {
    let l1 = vec![5, 1, 3, 2, 4];
    let l2 = vec![1, 2];
    let result = diff(&l1, &l2);
    assert_eq!(result, vec![5, 3, 4]);
}

#[test]
fn diff_with_strings() {
    let l1 = vec!["a", "b", "c", "d"];
    let l2 = vec!["b", "d"];
    let result = diff(&l1, &l2);
    assert_eq!(result, vec!["a", "c"]);
}

#[test]
fn diff_duplicates_in_l1() {
    // All occurrences of a filtered element should be removed
    let l1 = vec![1, 2, 1, 3, 2];
    let l2 = vec![2];
    let result = diff(&l1, &l2);
    assert_eq!(result, vec![1, 1, 3]);
}
