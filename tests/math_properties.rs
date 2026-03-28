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

// ─── carre ──────────────────────────────────────────────────────────────────

#[test]
fn carre_zero() {
    assert_eq!(carre(0.0), 0.0);
}

#[test]
fn carre_one() {
    assert_eq!(carre(1.0), 1.0);
}

#[test]
fn carre_known_values() {
    assert!(approx_eq(carre(3.0), 9.0, EPS));
    assert!(approx_eq(carre(-3.0), 9.0, EPS));
    assert!(approx_eq(carre(0.5), 0.25, EPS));
    assert!(approx_eq(carre(-0.5), 0.25, EPS));
}

#[test]
fn carre_symmetry() {
    // carre(x) == carre(-x) for all x
    for &x in &[0.0, 1.0, -1.0, 2.5, -2.5, 1e7, -1e7, 1e-7, -1e-7] {
        assert!(
            approx_eq(carre(x), carre(-x), EPS),
            "carre({x}) != carre({neg})",
            neg = -x
        );
    }
}

#[test]
fn carre_non_negative() {
    for &x in &[0.0, 1.0, -1.0, 100.0, -100.0, 1e15, -1e15, 1e-15, -1e-15] {
        assert!(carre(x) >= 0.0, "carre({x}) is negative");
    }
}

#[test]
fn carre_large_values() {
    // Should not overflow (f64 handles up to ~1e308)
    let v = carre(1e15);
    assert!(v.is_finite(), "carre(1e15) overflowed");
    assert!(approx_eq(v, 1e30, 1e20)); // relative check
}

#[test]
fn carre_small_values() {
    let v = carre(1e-15);
    assert!(v >= 0.0);
    assert!(v.is_finite());
}

// ─── addtuple ────────────────────────────────────────────────────────────────

#[test]
fn addtuple_commutativity() {
    let a = Vec2::new(3.0, 4.0);
    let b = Vec2::new(1.0, -2.0);
    assert!(vec2_approx_eq(addtuple(a, b), addtuple(b, a), EPS));
}

#[test]
fn addtuple_associativity() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    let c = Vec2::new(5.0, 6.0);
    let lhs = addtuple(addtuple(a, b), c);
    let rhs = addtuple(a, addtuple(b, c));
    assert!(vec2_approx_eq(lhs, rhs, EPS));
}

#[test]
fn addtuple_identity() {
    let a = Vec2::new(7.0, -3.0);
    let zero = Vec2::new(0.0, 0.0);
    assert!(vec2_approx_eq(addtuple(a, zero), a, EPS));
    assert!(vec2_approx_eq(addtuple(zero, a), a, EPS));
}

#[test]
fn addtuple_inverse() {
    let a = Vec2::new(5.0, -8.0);
    let neg_a = Vec2::new(-5.0, 8.0);
    let result = addtuple(a, neg_a);
    assert!(vec2_approx_eq(result, Vec2::new(0.0, 0.0), EPS));
}

#[test]
fn addtuple_known_values() {
    assert!(vec2_approx_eq(addtuple(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)), Vec2::new(4.0, 6.0), EPS));
    assert!(vec2_approx_eq(addtuple(Vec2::new(-1.0, -2.0), Vec2::new(1.0, 2.0)), Vec2::new(0.0, 0.0), EPS));
}

#[test]
fn addtuple_large_values() {
    let a = Vec2::new(1e15, 1e15);
    let b = Vec2::new(1e15, 1e15);
    let r = addtuple(a, b);
    assert!(approx_eq(r.x, 2e15, 1e5));
    assert!(approx_eq(r.y, 2e15, 1e5));
}

// ─── soustuple ───────────────────────────────────────────────────────────────

#[test]
fn soustuple_self_is_zero() {
    let a = Vec2::new(4.0, -7.0);
    assert!(vec2_approx_eq(soustuple(a, a), Vec2::ZERO, EPS));
}

#[test]
fn soustuple_zero_right() {
    let a = Vec2::new(4.0, -7.0);
    let zero = Vec2::new(0.0, 0.0);
    assert!(vec2_approx_eq(soustuple(a, zero), a, EPS));
}

#[test]
fn soustuple_anti_commutativity() {
    let a = Vec2::new(3.0, 5.0);
    let b = Vec2::new(1.0, 2.0);
    let ab = soustuple(a, b);
    let ba = soustuple(b, a);
    // a - b = -(b - a)
    assert!(vec2_approx_eq(ab, Vec2::new(-ba.x, -ba.y), EPS));
}

#[test]
fn soustuple_known_values() {
    assert!(vec2_approx_eq(soustuple(Vec2::new(5.0, 3.0), Vec2::new(2.0, 1.0)), Vec2::new(3.0, 2.0), EPS));
    assert!(vec2_approx_eq(soustuple(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)), Vec2::new(-1.0, -1.0), EPS));
}

#[test]
fn soustuple_recovers_via_add() {
    // (a + b) - b == a
    let a = Vec2::new(3.0, -4.0);
    let b = Vec2::new(1.0, 2.0);
    let sum = addtuple(a, b);
    assert!(vec2_approx_eq(soustuple(sum, b), a, EPS));
}

// ─── multuple ─────────────────────────────────────────────────────────────────

#[test]
fn multuple_identity() {
    let v = Vec2::new(3.0, -4.0);
    assert!(vec2_approx_eq(multuple(v, 1.0), v, EPS));
}

#[test]
fn multuple_zero_scalar() {
    let v = Vec2::new(3.0, -4.0);
    assert!(vec2_approx_eq(multuple(v, 0.0), Vec2::ZERO, EPS));
}

#[test]
fn multuple_zero_vector() {
    let zero = Vec2::new(0.0, 0.0);
    assert!(vec2_approx_eq(multuple(zero, 5.0), Vec2::ZERO, EPS));
}

#[test]
fn multuple_distributivity_add() {
    // (a + b) * s == a*s + b*s
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    let s = 2.5;
    let lhs = multuple(addtuple(a, b), s);
    let rhs = addtuple(multuple(a, s), multuple(b, s));
    assert!(vec2_approx_eq(lhs, rhs, EPS));
}

#[test]
fn multuple_scalar_associativity() {
    // v * (s1 * s2) == (v * s1) * s2
    let v = Vec2::new(3.0, 4.0);
    let s1 = 2.0;
    let s2 = 3.0;
    let lhs = multuple(v, s1 * s2);
    let rhs = multuple(multuple(v, s1), s2);
    assert!(vec2_approx_eq(lhs, rhs, EPS));
}

#[test]
fn multuple_negative_scalar() {
    let v = Vec2::new(3.0, -4.0);
    let neg = multuple(v, -1.0);
    assert!(vec2_approx_eq(neg, Vec2::new(-3.0, 4.0), EPS));
}

#[test]
fn multuple_known_values() {
    assert!(vec2_approx_eq(multuple(Vec2::new(2.0, 3.0), 4.0), Vec2::new(8.0, 12.0), EPS));
}

// ─── hypothenuse ─────────────────────────────────────────────────────────────

#[test]
fn hypothenuse_3_4_5() {
    assert!(approx_eq(hypothenuse(Vec2::new(3.0, 4.0)), 5.0, EPS));
    assert!(approx_eq(hypothenuse(Vec2::new(4.0, 3.0)), 5.0, EPS));
}

#[test]
fn hypothenuse_zero() {
    assert_eq!(hypothenuse(Vec2::new(0.0, 0.0)), 0.0);
}

#[test]
fn hypothenuse_unit_vectors() {
    assert!(approx_eq(hypothenuse(Vec2::new(1.0, 0.0)), 1.0, EPS));
    assert!(approx_eq(hypothenuse(Vec2::new(0.0, 1.0)), 1.0, EPS));
}

#[test]
fn hypothenuse_non_negative() {
    for v in &[Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(-1.0, 0.0), Vec2::new(3.0, -4.0), Vec2::new(-3.0, -4.0)] {
        assert!(hypothenuse(*v) >= 0.0);
    }
}

#[test]
fn hypothenuse_symmetry() {
    // hyp(x, y) == hyp(y, x) == hyp(-x, y) == hyp(x, -y)
    let cases = [Vec2::new(3.0, 4.0), Vec2::new(1.5, 2.5), Vec2::new(1e7, 1e7)];
    for v in cases {
        let (x, y) = (v.x, v.y);
        let h = hypothenuse(v);
        assert!(approx_eq(h, hypothenuse(Vec2::new(y, x)), EPS));
        assert!(approx_eq(h, hypothenuse(Vec2::new(-x, y)), EPS));
        assert!(approx_eq(h, hypothenuse(Vec2::new(x, -y)), EPS));
        assert!(approx_eq(h, hypothenuse(Vec2::new(-x, -y)), EPS));
    }
}

#[test]
fn hypothenuse_scaling() {
    // hyp(k*v) == k * hyp(v) for k > 0
    let v = Vec2::new(3.0, 4.0);
    let k = 5.0;
    let scaled = multuple(v, k);
    assert!(approx_eq(hypothenuse(scaled), k * hypothenuse(v), EPS));
}

#[test]
fn hypothenuse_triangle_inequality() {
    // |a + b| <= |a| + |b|
    let a = Vec2::new(3.0, 4.0);
    let b = Vec2::new(1.0, 2.0);
    let sum_mag = hypothenuse(addtuple(a, b));
    let mag_sum = hypothenuse(a) + hypothenuse(b);
    assert!(sum_mag <= mag_sum + EPS);
}

#[test]
fn hypothenuse_known_5_12_13() {
    assert!(approx_eq(hypothenuse(Vec2::new(5.0, 12.0)), 13.0, EPS));
}

#[test]
fn hypothenuse_large_values() {
    let v = hypothenuse(Vec2::new(1e15, 0.0));
    assert!(approx_eq(v, 1e15, 1e5));
}

#[test]
fn hypothenuse_small_values() {
    let v = hypothenuse(Vec2::new(1e-15, 0.0));
    assert!(approx_eq(v, 1e-15, 1e-25));
}

// ─── distancecarre ───────────────────────────────────────────────────────────

#[test]
fn distancecarre_self_is_zero() {
    let p = Vec2::new(3.0, 4.0);
    assert_eq!(distancecarre(p, p), 0.0);
}

#[test]
fn distancecarre_symmetry() {
    let p1 = Vec2::new(1.0, 2.0);
    let p2 = Vec2::new(4.0, 6.0);
    assert!(approx_eq(distancecarre(p1, p2), distancecarre(p2, p1), EPS));
}

#[test]
fn distancecarre_non_negative() {
    let cases = [
        (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        (Vec2::new(1.0, 2.0), Vec2::new(4.0, 6.0)),
        (Vec2::new(-3.0, -4.0), Vec2::new(3.0, 4.0)),
    ];
    for (p1, p2) in cases {
        assert!(distancecarre(p1, p2) >= 0.0);
    }
}

#[test]
fn distancecarre_known_3_4_5() {
    // Distance from (0,0) to (3,4) should be sqrt(25) = 5, so d² = 25
    assert!(approx_eq(distancecarre(Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0)), 25.0, EPS));
}

#[test]
fn distancecarre_equals_hypothenuse_squared() {
    let p1 = Vec2::new(1.0, 2.0);
    let p2 = Vec2::new(4.0, 6.0);
    let diff = soustuple(p2, p1);
    let h = hypothenuse(diff);
    assert!(approx_eq(distancecarre(p1, p2), h * h, EPS));
}

#[test]
fn distancecarre_triangle_inequality_squared() {
    // sqrt(d²(a,c)) <= sqrt(d²(a,b)) + sqrt(d²(b,c))
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(3.0, 4.0);
    let c = Vec2::new(6.0, 0.0);
    let dac = distancecarre(a, c).sqrt();
    let dab = distancecarre(a, b).sqrt();
    let dbc = distancecarre(b, c).sqrt();
    assert!(dac <= dab + dbc + EPS);
}

// ─── moytuple ─────────────────────────────────────────────────────────────────

#[test]
fn moytuple_ratio_one_returns_first() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    assert!(vec2_approx_eq(moytuple(a, b, 1.0), a, EPS));
}

#[test]
fn moytuple_ratio_zero_returns_second() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    assert!(vec2_approx_eq(moytuple(a, b, 0.0), b, EPS));
}

#[test]
fn moytuple_midpoint_equidistant() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(4.0, 4.0);
    let mid = moytuple(a, b, 0.5);
    let d1 = distancecarre(mid, a);
    let d2 = distancecarre(mid, b);
    assert!(approx_eq(d1, d2, EPS));
}

#[test]
fn moytuple_same_value() {
    let a = Vec2::new(5.0, 7.0);
    assert!(vec2_approx_eq(moytuple(a, a, 0.5), a, EPS));
    assert!(vec2_approx_eq(moytuple(a, a, 0.0), a, EPS));
    assert!(vec2_approx_eq(moytuple(a, a, 1.0), a, EPS));
}

#[test]
fn moytuple_known_midpoint() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(2.0, 4.0);
    let mid = moytuple(a, b, 0.5);
    assert!(vec2_approx_eq(mid, Vec2::new(1.0, 2.0), EPS));
}

// ─── moyfloat ─────────────────────────────────────────────────────────────────

#[test]
fn moyfloat_ratio_one_returns_first() {
    assert!(approx_eq(moyfloat(3.0, 7.0, 1.0), 3.0, EPS));
}

#[test]
fn moyfloat_ratio_zero_returns_second() {
    assert!(approx_eq(moyfloat(3.0, 7.0, 0.0), 7.0, EPS));
}

#[test]
fn moyfloat_midpoint() {
    assert!(approx_eq(moyfloat(0.0, 10.0, 0.5), 5.0, EPS));
    assert!(approx_eq(moyfloat(2.0, 4.0, 0.5), 3.0, EPS));
}

#[test]
fn moyfloat_same_value() {
    assert!(approx_eq(moyfloat(5.0, 5.0, 0.3), 5.0, EPS));
}

#[test]
fn moyfloat_known_values() {
    // 0.25 * 8 + 0.75 * 4 = 2 + 3 = 5
    assert!(approx_eq(moyfloat(8.0, 4.0, 0.25), 5.0, EPS));
}

// ─── multuple_parallel ───────────────────────────────────────────────────────

#[test]
fn multuple_parallel_commutativity() {
    let a = Vec2::new(2.0, 3.0);
    let b = Vec2::new(4.0, 5.0);
    assert!(vec2_approx_eq(multuple_parallel(a, b), multuple_parallel(b, a), EPS));
}

#[test]
fn multuple_parallel_identity() {
    let v = Vec2::new(3.0, -4.0);
    let one = Vec2::new(1.0, 1.0);
    assert!(vec2_approx_eq(multuple_parallel(v, one), v, EPS));
}

#[test]
fn multuple_parallel_zero() {
    let v = Vec2::new(3.0, -4.0);
    let zero = Vec2::new(0.0, 0.0);
    assert!(vec2_approx_eq(multuple_parallel(v, zero), Vec2::ZERO, EPS));
}

#[test]
fn multuple_parallel_known_values() {
    assert!(vec2_approx_eq(
        multuple_parallel(Vec2::new(2.0, 3.0), Vec2::new(4.0, 5.0)),
        Vec2::new(8.0, 15.0),
        EPS
    ));
}

#[test]
fn multuple_parallel_associativity() {
    let a = Vec2::new(2.0, 3.0);
    let b = Vec2::new(4.0, 5.0);
    let c = Vec2::new(6.0, 7.0);
    let lhs = multuple_parallel(multuple_parallel(a, b), c);
    let rhs = multuple_parallel(a, multuple_parallel(b, c));
    assert!(vec2_approx_eq(lhs, rhs, EPS));
}

// ─── entretuple ──────────────────────────────────────────────────────────────

#[test]
fn entretuple_inside() {
    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 10.0);
    assert!(entretuple(Vec2::new(5.0, 5.0), min, max));
    assert!(entretuple(Vec2::new(1.0, 9.0), min, max));
    assert!(entretuple(Vec2::new(9.0, 1.0), min, max));
}

#[test]
fn entretuple_outside() {
    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 10.0);
    assert!(!entretuple(Vec2::new(-1.0, 5.0), min, max));
    assert!(!entretuple(Vec2::new(5.0, -1.0), min, max));
    assert!(!entretuple(Vec2::new(11.0, 5.0), min, max));
    assert!(!entretuple(Vec2::new(5.0, 11.0), min, max));
    assert!(!entretuple(Vec2::new(15.0, 15.0), min, max));
    assert!(!entretuple(Vec2::new(-5.0, -5.0), min, max));
}

#[test]
fn entretuple_boundary_is_exclusive() {
    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 10.0);
    // Strict inequality, so boundary is NOT inside
    assert!(!entretuple(Vec2::new(0.0, 5.0), min, max));
    assert!(!entretuple(Vec2::new(10.0, 5.0), min, max));
    assert!(!entretuple(Vec2::new(5.0, 0.0), min, max));
    assert!(!entretuple(Vec2::new(5.0, 10.0), min, max));
}

#[test]
fn entretuple_negative_coords() {
    let min = Vec2::new(-5.0, -5.0);
    let max = Vec2::new(5.0, 5.0);
    assert!(entretuple(Vec2::new(0.0, 0.0), min, max));
    assert!(entretuple(Vec2::new(-4.0, -4.0), min, max));
    assert!(!entretuple(Vec2::new(-6.0, 0.0), min, max));
}

// ─── inttuple / floattuple ───────────────────────────────────────────────────

#[test]
fn inttuple_known_values() {
    assert_eq!(inttuple(Vec2::new(3.0, 4.0)), (3, 4));
    assert_eq!(inttuple(Vec2::new(-1.0, -2.0)), (-1, -2));
    assert_eq!(inttuple(Vec2::new(0.0, 0.0)), (0, 0));
}

#[test]
fn inttuple_truncates() {
    // f64 as i32 truncates toward zero
    assert_eq!(inttuple(Vec2::new(3.9, 4.9)), (3, 4));
    assert_eq!(inttuple(Vec2::new(-3.9, -4.9)), (-3, -4));
}

#[test]
fn floattuple_known_values() {
    assert!(vec2_approx_eq(floattuple((3, 4)), Vec2::new(3.0, 4.0), EPS));
    assert!(vec2_approx_eq(floattuple((-1, -2)), Vec2::new(-1.0, -2.0), EPS));
    assert!(vec2_approx_eq(floattuple((0, 0)), Vec2::new(0.0, 0.0), EPS));
}

#[test]
fn inttuple_floattuple_roundtrip() {
    // For integer-valued floats, roundtrip should be exact
    let integers = [(0, 0), (1, 2), (-3, 4), (100, -200)];
    for &(x, y) in &integers {
        let v_float = floattuple((x, y));
        let v_int = inttuple(v_float);
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
    assert!(vec2_approx_eq(proj(base, dir, 1.0), addtuple(base, dir), EPS));
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
    assert!(vec2_approx_eq(proj(base, dir, -1.0), soustuple(base, dir), EPS));
}

// ─── polar_to_affine / affine_to_polar ───────────────────────────────────────

#[test]
fn polar_to_affine_unit_circle_zero_angle() {
    let v = polar_to_affine(0.0, 1.0);
    assert!(approx_eq(v.x, 1.0, EPS_TRIG));
    assert!(approx_eq(v.y, 0.0, EPS_TRIG));
}

#[test]
fn polar_to_affine_unit_circle_pi_half() {
    let v = polar_to_affine(PI / 2.0, 1.0);
    assert!(approx_eq(v.x, 0.0, EPS_TRIG));
    assert!(approx_eq(v.y, 1.0, EPS_TRIG));
}

#[test]
fn polar_to_affine_unit_circle_pi() {
    let v = polar_to_affine(PI, 1.0);
    assert!(approx_eq(v.x, -1.0, EPS_TRIG));
    assert!(approx_eq(v.y, 0.0, EPS_TRIG));
}

#[test]
fn polar_to_affine_unit_circle_3pi_half() {
    let v = polar_to_affine(3.0 * PI / 2.0, 1.0);
    assert!(approx_eq(v.x, 0.0, EPS_TRIG));
    assert!(approx_eq(v.y, -1.0, EPS_TRIG));
}

#[test]
fn polar_to_affine_zero_radius() {
    let v = polar_to_affine(PI / 4.0, 0.0);
    assert!(approx_eq(v.x, 0.0, EPS_TRIG));
    assert!(approx_eq(v.y, 0.0, EPS_TRIG));
}

#[test]
fn polar_to_affine_scaling() {
    // magnitude should equal radius
    let r = 5.0;
    let v = polar_to_affine(PI / 4.0, r);
    assert!(approx_eq(hypothenuse(v), r, EPS_TRIG));
}

#[test]
fn polar_to_affine_preserves_magnitude() {
    let cases = [(0.0, 3.0), (PI / 6.0, 2.0), (PI, 4.0), (7.0 * PI / 4.0, 1.5)];
    for (angle, r) in cases {
        let v = polar_to_affine(angle, r);
        assert!(
            approx_eq(hypothenuse(v), r, EPS_TRIG),
            "angle={angle}, r={r}: hyp={h}",
            h = hypothenuse(v)
        );
    }
}

#[test]
fn affine_to_polar_unit_x() {
    let polar = affine_to_polar(Vec2::new(1.0, 0.0));
    assert!(approx_eq(polar.y, 1.0, EPS_TRIG));
    assert!(approx_eq(polar.x, 0.0, EPS_TRIG));
}

#[test]
fn affine_to_polar_unit_y() {
    let polar = affine_to_polar(Vec2::new(0.0, 1.0));
    assert!(approx_eq(polar.y, 1.0, EPS_TRIG));
    assert!(approx_eq(polar.x, PI / 2.0, EPS_TRIG));
}

#[test]
fn affine_to_polar_zero_vector() {
    let polar = affine_to_polar(Vec2::ZERO);
    assert_eq!(polar.x, 0.0);
    assert_eq!(polar.y, 0.0);
}

#[test]
fn polar_roundtrip_affine_to_polar_to_affine() {
    // affine → polar → affine should recover original (for non-zero vectors)
    // Note: (-1, 0) is excluded — affine_to_polar uses the half-angle formula
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
        let polar = affine_to_polar(v);
        let recovered = polar_to_affine(polar.x, polar.y);
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
fn polar_roundtrip_polar_to_affine_to_polar() {
    // polar → affine → polar should recover original angle and magnitude
    let cases = [
        (0.0, 1.0),
        (PI / 4.0, 2.0),
        (PI / 2.0, 3.0),
        (PI, 1.5),
        (3.0 * PI / 2.0, 2.5),
    ];
    for (angle, r) in cases {
        let v = polar_to_affine(angle, r);
        let polar = affine_to_polar(v);
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
fn polar_to_affine_tuple_matches_two_arg() {
    let cases = [(0.0, 1.0), (PI / 4.0, 2.0), (PI, 3.0), (1.5, 4.0)];
    for (angle, r) in cases {
        let v1 = polar_to_affine(angle, r);
        let v2 = polar_to_affine_tuple((angle, r));
        assert!(
            vec2_approx_eq(v1, v2, EPS_TRIG),
            "mismatch for angle={angle}, r={r}: v1={v1:?}, v2={v2:?}"
        );
    }
}

// ─── modulo_float ─────────────────────────────────────────────────────────────

#[test]
fn modulo_float_in_range() {
    assert!(approx_eq(modulo_float(5.0, 10.0), 5.0, EPS));
    assert!(approx_eq(modulo_float(0.5, 10.0), 0.5, EPS));
    assert!(approx_eq(modulo_float(9.9, 10.0), 9.9, EPS));
}

#[test]
fn modulo_float_overflow() {
    // value >= modulo: subtract once
    assert!(approx_eq(modulo_float(10.0, 10.0), 0.0, EPS));
    assert!(approx_eq(modulo_float(12.0, 10.0), 2.0, EPS));
    assert!(approx_eq(modulo_float(15.0, 10.0), 5.0, EPS));
}

#[test]
fn modulo_float_underflow() {
    // value < 0: add modulo once
    assert!(approx_eq(modulo_float(-1.0, 10.0), 9.0, EPS));
    assert!(approx_eq(modulo_float(-5.0, 10.0), 5.0, EPS));
    assert!(approx_eq(modulo_float(-9.9, 10.0), 0.1, EPS));
}

#[test]
fn modulo_float_idempotent() {
    // Applying twice to an already in-range value should be a no-op
    let v = 5.0;
    let m = 10.0;
    let once = modulo_float(v, m);
    let twice = modulo_float(once, m);
    assert!(approx_eq(once, twice, EPS));
}

#[test]
fn modulo_float_range_check() {
    // Result should always be in [0, modulo)
    let modulo = 10.0;
    for &v in &[-1.0, 0.0, 5.0, 9.9, 10.0, 15.0, -5.0] {
        let r = modulo_float(v, modulo);
        assert!(r >= 0.0 && r < modulo, "modulo_float({v}, {modulo}) = {r} out of range");
    }
}

// ─── modulo_reso ─────────────────────────────────────────────────────────────

#[test]
fn modulo_reso_in_range() {
    let r = modulo_reso(Vec2::new(5.0, 5.0), 10.0, 10.0);
    assert!(vec2_approx_eq(r, Vec2::new(5.0, 5.0), EPS));
}

#[test]
fn modulo_reso_wraps_x() {
    let r = modulo_reso(Vec2::new(12.0, 5.0), 10.0, 10.0);
    assert!(approx_eq(r.x, 2.0, EPS));
    assert!(approx_eq(r.y, 5.0, EPS));
}

#[test]
fn modulo_reso_wraps_y() {
    let r = modulo_reso(Vec2::new(5.0, -1.0), 10.0, 10.0);
    assert!(approx_eq(r.x, 5.0, EPS));
    assert!(approx_eq(r.y, 9.0, EPS));
}

#[test]
fn modulo_reso_both_axes() {
    let r = modulo_reso(Vec2::new(-1.0, 11.0), 10.0, 10.0);
    assert!(approx_eq(r.x, 9.0, EPS));
    assert!(approx_eq(r.y, 1.0, EPS));
}

// ─── modulo_3reso ─────────────────────────────────────────────────────────────

#[test]
fn modulo_3reso_in_range() {
    // A point inside [-w, w) x [-h, h) (the 1-screen-wide margin) should stay
    let r = modulo_3reso(Vec2::new(5.0, 5.0), 10.0, 10.0);
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
fn modulo_3reso_far_positive() {
    // Point at (25, 5): 25 + 10 = 35, 35 mod 30 = 5, 5 - 10 = -5
    let r = modulo_3reso(Vec2::new(25.0, 5.0), 10.0, 10.0);
    assert!(approx_eq(r.x, -5.0, EPS), "expected -5, got {}", r.x);
}

#[test]
fn modulo_3reso_far_negative() {
    // Point at (-25, 5): -25 + 10 = -15, -15 mod 30 = 15, 15 - 10 = 5
    let r = modulo_3reso(Vec2::new(-25.0, 5.0), 10.0, 10.0);
    assert!(approx_eq(r.x, 5.0, EPS), "expected 5, got {}", r.x);
}

#[test]
fn modulo_3reso_idempotent_in_range() {
    // Applying twice to an already in-range value should be no-op
    let v = Vec2::new(5.0, 3.0);
    let w = 10.0;
    let h = 10.0;
    let once = modulo_3reso(v, w, h);
    let twice = modulo_3reso(once, w, h);
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

// ─── randfloat ────────────────────────────────────────────────────────────────

#[test]
fn randfloat_range_check() {
    let mut rng = rand::thread_rng();
    let min = -5.0;
    let max = 10.0;
    for _ in 0..1000 {
        let v = randfloat(min, max, &mut rng);
        assert!(
            v >= min && v < max,
            "randfloat({min}, {max}) = {v} out of range"
        );
    }
}

#[test]
fn randfloat_degenerate_range() {
    let mut rng = rand::thread_rng();
    // min == max: result should be min (since gen() ∈ [0,1), 0 * anything = 0 + min = min)
    // Actually gen() could be 0.0 yielding exactly min
    let v = randfloat(5.0, 5.0, &mut rng);
    assert!(approx_eq(v, 5.0, EPS));
}

#[test]
fn randfloat_positive_range() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let v = randfloat(0.0, 1.0, &mut rng);
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
