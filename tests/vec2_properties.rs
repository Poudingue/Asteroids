/// Property tests for the Vec2 struct in math.rs
/// Covers all operators, methods, conversions, and edge cases.

use asteroids::math::Vec2;
use std::f64::consts::PI;

const EPS: f64 = 1e-10;

fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

fn v2_approx_eq(a: Vec2, b: Vec2, eps: f64) -> bool {
    approx_eq(a.x, b.x, eps) && approx_eq(a.y, b.y, eps)
}

// ─── Construction ──────────────────────────────────────────────────────────

#[test]
fn new_creates_correct_fields() {
    let v = Vec2::new(3.0, 4.0);
    assert_eq!(v.x, 3.0);
    assert_eq!(v.y, 4.0);
}

#[test]
fn zero_constant() {
    assert_eq!(Vec2::ZERO.x, 0.0);
    assert_eq!(Vec2::ZERO.y, 0.0);
}

#[test]
fn default_is_zero() {
    assert_eq!(Vec2::default(), Vec2::ZERO);
}

// ─── Add ───────────────────────────────────────────────────────────────────

#[test]
fn add_basic() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    assert_eq!(a + b, Vec2::new(4.0, 6.0));
}

#[test]
fn add_commutative() {
    let a = Vec2::new(1.5, -2.5);
    let b = Vec2::new(-3.0, 7.0);
    assert_eq!(a + b, b + a);
}

#[test]
fn add_identity() {
    let a = Vec2::new(42.0, -17.0);
    assert_eq!(a + Vec2::ZERO, a);
}

#[test]
fn add_assign() {
    let mut a = Vec2::new(1.0, 2.0);
    a += Vec2::new(3.0, 4.0);
    assert_eq!(a, Vec2::new(4.0, 6.0));
}

// ─── Sub ───────────────────────────────────────────────────────────────────

#[test]
fn sub_basic() {
    let a = Vec2::new(5.0, 3.0);
    let b = Vec2::new(2.0, 1.0);
    assert_eq!(a - b, Vec2::new(3.0, 2.0));
}

#[test]
fn sub_self_is_zero() {
    let a = Vec2::new(123.456, -789.0);
    assert_eq!(a - a, Vec2::ZERO);
}

#[test]
fn sub_assign() {
    let mut a = Vec2::new(5.0, 3.0);
    a -= Vec2::new(2.0, 1.0);
    assert_eq!(a, Vec2::new(3.0, 2.0));
}

// ─── Mul ───────────────────────────────────────────────────────────────────

#[test]
fn mul_scalar_right() {
    let v = Vec2::new(2.0, 3.0);
    assert_eq!(v * 4.0, Vec2::new(8.0, 12.0));
}

#[test]
fn mul_scalar_left() {
    let v = Vec2::new(2.0, 3.0);
    assert_eq!(4.0 * v, Vec2::new(8.0, 12.0));
}

#[test]
fn mul_by_zero() {
    let v = Vec2::new(999.0, -888.0);
    assert_eq!(v * 0.0, Vec2::ZERO);
}

#[test]
fn mul_by_one_identity() {
    let v = Vec2::new(5.0, -3.0);
    assert_eq!(v * 1.0, v);
}

#[test]
fn mul_assign() {
    let mut v = Vec2::new(2.0, 3.0);
    v *= 4.0;
    assert_eq!(v, Vec2::new(8.0, 12.0));
}

// ─── Neg ───────────────────────────────────────────────────────────────────

#[test]
fn neg_basic() {
    let v = Vec2::new(3.0, -4.0);
    assert_eq!(-v, Vec2::new(-3.0, 4.0));
}

#[test]
fn neg_double_is_identity() {
    let v = Vec2::new(1.5, -2.5);
    assert_eq!(-(-v), v);
}

#[test]
fn neg_zero() {
    // -0.0 == 0.0 in f64
    let z = -Vec2::ZERO;
    assert_eq!(z.x, 0.0);
    assert_eq!(z.y, 0.0);
}

// ─── Length ────────────────────────────────────────────────────────────────

#[test]
fn length_345() {
    let v = Vec2::new(3.0, 4.0);
    assert!(approx_eq(v.length(), 5.0, EPS));
}

#[test]
fn length_zero() {
    assert_eq!(Vec2::ZERO.length(), 0.0);
}

#[test]
fn length_squared_avoids_sqrt() {
    let v = Vec2::new(3.0, 4.0);
    assert!(approx_eq(v.length_squared(), 25.0, EPS));
}

// ─── Distance ──────────────────────────────────────────────────────────────

#[test]
fn distance_basic() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(3.0, 4.0);
    assert!(approx_eq(a.distance(b), 5.0, EPS));
}

#[test]
fn distance_symmetric() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(4.0, 6.0);
    assert!(approx_eq(a.distance(b), b.distance(a), EPS));
}

#[test]
fn distance_to_self_is_zero() {
    let a = Vec2::new(42.0, -17.0);
    assert_eq!(a.distance(a), 0.0);
}

#[test]
fn distance_squared_basic() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(3.0, 4.0);
    assert!(approx_eq(a.distance_squared(b), 25.0, EPS));
}

// ─── Lerp ──────────────────────────────────────────────────────────────────

#[test]
fn lerp_t1_returns_self() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(5.0, 6.0);
    assert!(v2_approx_eq(a.lerp(b, 1.0), a, EPS));
}

#[test]
fn lerp_t0_returns_other() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(5.0, 6.0);
    assert!(v2_approx_eq(a.lerp(b, 0.0), b, EPS));
}

#[test]
fn lerp_t05_midpoint() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(10.0, 20.0);
    assert!(v2_approx_eq(a.lerp(b, 0.5), Vec2::new(5.0, 10.0), EPS));
}

// ─── Component mul ─────────────────────────────────────────────────────────

#[test]
fn component_mul_basic() {
    let a = Vec2::new(2.0, 3.0);
    let b = Vec2::new(4.0, 5.0);
    assert_eq!(a.component_mul(b), Vec2::new(8.0, 15.0));
}

// ─── Polar roundtrip ───────────────────────────────────────────────────────

#[test]
fn polar_roundtrip_first_quadrant() {
    let v = Vec2::new(3.0, 4.0);
    let (angle, mag) = v.to_polar();
    let reconstructed = Vec2::from_polar(angle, mag);
    assert!(v2_approx_eq(v, reconstructed, EPS));
}

#[test]
fn polar_roundtrip_negative() {
    let v = Vec2::new(-5.0, 12.0);
    let (angle, mag) = v.to_polar();
    let reconstructed = Vec2::from_polar(angle, mag);
    assert!(v2_approx_eq(v, reconstructed, 1e-8));
}

#[test]
fn polar_zero_vector() {
    let (angle, mag) = Vec2::ZERO.to_polar();
    assert_eq!(angle, 0.0);
    assert_eq!(mag, 0.0);
}

#[test]
fn from_polar_right() {
    let v = Vec2::from_polar(0.0, 5.0);
    assert!(v2_approx_eq(v, Vec2::new(5.0, 0.0), EPS));
}

#[test]
fn from_polar_up() {
    let v = Vec2::from_polar(PI / 2.0, 5.0);
    assert!(v2_approx_eq(v, Vec2::new(0.0, 5.0), EPS));
}

// ─── Conversions ───────────────────────────────────────────────────────────

#[test]
fn from_tuple() {
    let v: Vec2 = (3.0, 4.0).into();
    assert_eq!(v, Vec2::new(3.0, 4.0));
}

#[test]
fn into_tuple() {
    let v = Vec2::new(3.0, 4.0);
    let t: (f64, f64) = v.into();
    assert_eq!(t, (3.0, 4.0));
}

#[test]
fn roundtrip_tuple() {
    let original = (1.5, -2.5);
    let v: Vec2 = original.into();
    let back: (f64, f64) = v.into();
    assert_eq!(original, back);
}

#[test]
fn to_i32_truncates() {
    let v = Vec2::new(3.9, -2.1);
    assert_eq!(v.to_i32(), (3, -2));
}

#[test]
fn from_i32_converts() {
    let v = Vec2::from_i32((3, -4));
    assert_eq!(v, Vec2::new(3.0, -4.0));
}

// ─── Edge cases ────────────────────────────────────────────────────────────

#[test]
fn operations_with_large_values() {
    let v = Vec2::new(1e15, 1e15);
    let sum = v + v;
    assert!(sum.x.is_finite());
    assert!(sum.y.is_finite());
}

#[test]
fn operations_with_small_values() {
    let v = Vec2::new(1e-15, 1e-15);
    let len = v.length();
    assert!(len.is_finite());
    assert!(len > 0.0);
}

#[test]
fn clone_and_copy() {
    let a = Vec2::new(1.0, 2.0);
    let b = a; // Copy
    let c = a.clone(); // Clone
    assert_eq!(a, b);
    assert_eq!(a, c);
}

#[test]
fn debug_format() {
    let v = Vec2::new(1.0, 2.0);
    let s = format!("{:?}", v);
    assert!(s.contains("Vec2"));
}
