use asteroids::color::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-10
}

fn color_approx(a: HdrColor, b: HdrColor) -> bool {
    approx(a.r, b.r) && approx(a.g, b.g) && approx(a.b, b.b)
}

fn c(r: f64, g: f64, b: f64) -> HdrColor {
    HdrColor::new(r, g, b)
}

// ---------------------------------------------------------------------------
// HdrColor constructors
// ---------------------------------------------------------------------------

#[test]
fn new_stores_fields() {
    let col = HdrColor::new(1.0, 2.0, 3.0);
    assert!(approx(col.r, 1.0));
    assert!(approx(col.g, 2.0));
    assert!(approx(col.b, 3.0));
}

#[test]
fn zero_is_all_zeros() {
    let z = HdrColor::zero();
    assert!(approx(z.r, 0.0));
    assert!(approx(z.g, 0.0));
    assert!(approx(z.b, 0.0));
}

#[test]
fn one_is_all_ones() {
    let o = HdrColor::one();
    assert!(approx(o.r, 1.0));
    assert!(approx(o.g, 1.0));
    assert!(approx(o.b, 1.0));
}

#[test]
fn default_equals_zero() {
    let d: HdrColor = Default::default();
    let z = HdrColor::zero();
    assert!(color_approx(d, z));
}

#[test]
fn new_accepts_negative_values() {
    let col = HdrColor::new(-1.0, -2.0, -3.0);
    assert!(approx(col.r, -1.0));
    assert!(approx(col.g, -2.0));
    assert!(approx(col.b, -3.0));
}

#[test]
fn new_accepts_large_values() {
    let col = HdrColor::new(1e9, 1e9, 1e9);
    assert!(approx(col.r, 1e9));
    assert!(approx(col.g, 1e9));
    assert!(approx(col.b, 1e9));
}

// ---------------------------------------------------------------------------
// hdr_add
// ---------------------------------------------------------------------------

#[test]
fn hdr_add_basic() {
    let result = hdr_add(c(1.0, 2.0, 3.0), c(10.0, 20.0, 30.0));
    assert!(color_approx(result, c(11.0, 22.0, 33.0)));
}

#[test]
fn hdr_add_commutativity() {
    let a = c(1.5, 2.5, 3.5);
    let b = c(4.0, 5.0, 6.0);
    assert!(color_approx(hdr_add(a, b), hdr_add(b, a)));
}

#[test]
fn hdr_add_associativity() {
    let a = c(1.0, 2.0, 3.0);
    let b = c(4.0, 5.0, 6.0);
    let cc = c(7.0, 8.0, 9.0);
    assert!(color_approx(
        hdr_add(hdr_add(a, b), cc),
        hdr_add(a, hdr_add(b, cc))
    ));
}

#[test]
fn hdr_add_identity_zero() {
    let a = c(1.5, 2.5, 3.5);
    let z = HdrColor::zero();
    assert!(color_approx(hdr_add(a, z), a));
    assert!(color_approx(hdr_add(z, a), a));
}

#[test]
fn hdr_add_negative_values() {
    let result = hdr_add(c(-1.0, -2.0, -3.0), c(1.0, 2.0, 3.0));
    assert!(color_approx(result, HdrColor::zero()));
}

#[test]
fn hdr_add_large_values() {
    let result = hdr_add(c(1e8, 1e8, 1e8), c(1e8, 1e8, 1e8));
    assert!(color_approx(result, c(2e8, 2e8, 2e8)));
}

// ---------------------------------------------------------------------------
// hdr_sub (subtract)
// ---------------------------------------------------------------------------

#[test]
fn hdr_sub_basic() {
    let result = hdr_sub(c(10.0, 20.0, 30.0), c(1.0, 2.0, 3.0));
    assert!(color_approx(result, c(9.0, 18.0, 27.0)));
}

#[test]
fn hdr_sub_self_is_zero() {
    let a = c(5.0, 10.0, 15.0);
    assert!(color_approx(hdr_sub(a, a), HdrColor::zero()));
}

#[test]
fn hdr_sub_zero_right_is_identity() {
    let a = c(5.0, 10.0, 15.0);
    assert!(color_approx(hdr_sub(a, HdrColor::zero()), a));
}

#[test]
fn hdr_sub_zero_left_negates() {
    let a = c(5.0, 10.0, 15.0);
    let result = hdr_sub(HdrColor::zero(), a);
    assert!(color_approx(result, c(-5.0, -10.0, -15.0)));
}

#[test]
fn hdr_sub_not_commutative() {
    let a = c(5.0, 10.0, 15.0);
    let b = c(1.0, 2.0, 3.0);
    let ab = hdr_sub(a, b);
    let ba = hdr_sub(b, a);
    // a-b != b-a (unless equal), specifically ab.r should be 4, ba.r should be -4
    assert!(approx(ab.r, 4.0));
    assert!(approx(ba.r, -4.0));
}

// ---------------------------------------------------------------------------
// hdr_mul (component-wise multiply)
// ---------------------------------------------------------------------------

#[test]
fn hdr_mul_basic() {
    let result = hdr_mul(c(2.0, 3.0, 4.0), c(5.0, 6.0, 7.0));
    assert!(color_approx(result, c(10.0, 18.0, 28.0)));
}

#[test]
fn hdr_mul_commutativity() {
    let a = c(2.0, 3.0, 4.0);
    let b = c(5.0, 6.0, 7.0);
    assert!(color_approx(hdr_mul(a, b), hdr_mul(b, a)));
}

#[test]
fn hdr_mul_identity_one() {
    let a = c(2.0, 3.0, 4.0);
    assert!(color_approx(hdr_mul(a, HdrColor::one()), a));
    assert!(color_approx(hdr_mul(HdrColor::one(), a), a));
}

#[test]
fn hdr_mul_zero_annihilates() {
    let a = c(2.0, 3.0, 4.0);
    assert!(color_approx(hdr_mul(a, HdrColor::zero()), HdrColor::zero()));
    assert!(color_approx(hdr_mul(HdrColor::zero(), a), HdrColor::zero()));
}

#[test]
fn hdr_mul_negative_values() {
    let result = hdr_mul(c(-2.0, 3.0, -4.0), c(-1.0, -1.0, -1.0));
    assert!(color_approx(result, c(2.0, -3.0, 4.0)));
}

// ---------------------------------------------------------------------------
// intensify (scalar multiply)
// ---------------------------------------------------------------------------

#[test]
fn intensify_basic() {
    let result = intensify(c(1.0, 2.0, 3.0), 2.0);
    assert!(color_approx(result, c(2.0, 4.0, 6.0)));
}

#[test]
fn intensify_identity_one() {
    let a = c(1.5, 2.5, 3.5);
    assert!(color_approx(intensify(a, 1.0), a));
}

#[test]
fn intensify_zero_annihilates() {
    let a = c(1.5, 2.5, 3.5);
    assert!(color_approx(intensify(a, 0.0), HdrColor::zero()));
}

#[test]
fn intensify_equivalence_with_hdr_mul() {
    // intensify(col, s) should equal hdr_mul(col, HdrColor::new(s, s, s))
    let a = c(2.0, 3.0, 4.0);
    let s = 3.5;
    let via_intensify = intensify(a, s);
    let via_mul = hdr_mul(a, HdrColor::new(s, s, s));
    assert!(color_approx(via_intensify, via_mul));
}

#[test]
fn intensify_negative_scalar() {
    let result = intensify(c(1.0, 2.0, 3.0), -1.0);
    assert!(color_approx(result, c(-1.0, -2.0, -3.0)));
}

#[test]
fn intensify_large_scalar() {
    let result = intensify(c(1.0, 1.0, 1.0), 1e6);
    assert!(color_approx(result, c(1e6, 1e6, 1e6)));
}

// ---------------------------------------------------------------------------
// half_color (also exercises abso_exp_decay indirectly)
// ---------------------------------------------------------------------------

#[test]
fn half_color_zero_dt_returns_start() {
    // At dt=0: result = col2 + (col1 - col2) * 2^0 = col2 + col1 - col2 = col1
    let col1 = c(10.0, 20.0, 30.0);
    let col2 = c(1.0, 2.0, 3.0);
    let result = half_color(col1, col2, 1.0, 0.0);
    assert!(color_approx(result, col1));
}

#[test]
fn half_color_large_dt_approaches_target() {
    let col1 = c(10.0, 20.0, 30.0);
    let col2 = c(1.0, 2.0, 3.0);
    let result = half_color(col1, col2, 1.0, 1000.0);
    // After very large dt, decay term ≈ 0, result ≈ col2
    assert!((result.r - col2.r).abs() < 1e-200);
    assert!((result.g - col2.g).abs() < 1e-200);
    assert!((result.b - col2.b).abs() < 1e-200);
}

#[test]
fn half_color_one_half_life_midpoint() {
    // At dt=half_life: result = col2 + (col1-col2)*0.5 = midpoint
    let col1 = c(10.0, 20.0, 30.0);
    let col2 = c(0.0, 0.0, 0.0);
    let result = half_color(col1, col2, 1.0, 1.0);
    assert!(color_approx(result, c(5.0, 10.0, 15.0)));
}

#[test]
fn half_color_two_half_lives_is_quarter() {
    // After 2 half-lives the remaining difference is 1/4 of the original
    let col1 = c(16.0, 0.0, 0.0);
    let col2 = c(0.0, 0.0, 0.0);
    let result = half_color(col1, col2, 1.0, 2.0);
    // diff starts at 16, after 2 half-lives = 16 * 0.25 = 4
    assert!(color_approx(result, c(4.0, 0.0, 0.0)));
}

#[test]
fn half_color_monotonically_approaches_target() {
    let col1 = c(100.0, 0.0, 0.0);
    let col2 = c(0.0, 0.0, 0.0);
    let hl = 1.0;
    let r0 = half_color(col1, col2, hl, 0.0).r;
    let r1 = half_color(col1, col2, hl, 0.5).r;
    let r2 = half_color(col1, col2, hl, 1.0).r;
    let r3 = half_color(col1, col2, hl, 2.0).r;
    assert!(r0 > r1);
    assert!(r1 > r2);
    assert!(r2 > r3);
    assert!(r3 > 0.0);
}

#[test]
fn half_color_same_colors_stays_same() {
    let col = c(5.0, 10.0, 15.0);
    let result = half_color(col, col, 1.0, 1.0);
    assert!(color_approx(result, col));
}

#[test]
fn half_color_negative_difference() {
    // col1 < col2: decay should bring result from col1 toward col2
    let col1 = c(0.0, 0.0, 0.0);
    let col2 = c(10.0, 10.0, 10.0);
    let result = half_color(col1, col2, 1.0, 1.0);
    // result = col2 + (col1-col2)*0.5 = 10 + (-10)*0.5 = 5
    assert!(color_approx(result, c(5.0, 5.0, 5.0)));
}

// ---------------------------------------------------------------------------
// redirect_spectre_wide
// ---------------------------------------------------------------------------

#[test]
fn redirect_spectre_wide_no_overflow_passthrough() {
    // When all channels ≤ 255, color passes through unchanged
    let col = c(100.0, 150.0, 200.0);
    let result = redirect_spectre_wide(col);
    assert!(color_approx(result, col));
}

#[test]
fn redirect_spectre_wide_zero_passthrough() {
    let result = redirect_spectre_wide(HdrColor::zero());
    assert!(color_approx(result, HdrColor::zero()));
}

#[test]
fn redirect_spectre_wide_v_overflow_bleeds_to_r_and_b() {
    // v > 255, r and b ≤ 255: r and b should receive bleed from v
    let col = c(100.0, 300.0, 100.0);
    let result = redirect_spectre_wide(col);
    // r should get v bleed: r + (v - 255) = 100 + 45 = 145
    assert!(approx(result.r, 145.0));
    // b should also get v bleed: b + (v - 255) = 100 + 45 = 145
    assert!(approx(result.b, 145.0));
    // v itself stays as-is (no r or b overflow)
    assert!(approx(result.g, 300.0));
}

#[test]
fn redirect_spectre_wide_r_overflow_bleeds_to_v() {
    // r > 255, b ≤ 510, v ≤ 255: v gets bleed from r
    let col = c(300.0, 100.0, 100.0);
    let result = redirect_spectre_wide(col);
    // v: r > 255, b ≤ 255 → v + r - 255 = 100 + 300 - 255 = 145
    assert!(approx(result.g, 145.0));
    // r unchanged: b ≤ 510, v ≤ 255 → r = 300
    assert!(approx(result.r, 300.0));
    // b: r ≤ 510, v ≤ 255 → b unchanged = 100
    assert!(approx(result.b, 100.0));
}

#[test]
fn redirect_spectre_wide_b_overflow_bleeds_to_v() {
    // b > 255, r ≤ 255: v gets bleed from b
    let col = c(100.0, 100.0, 300.0);
    let result = redirect_spectre_wide(col);
    // v: b > 255, r ≤ 255 → v + b - 255 = 100 + 300 - 255 = 145
    assert!(approx(result.g, 145.0));
}

#[test]
fn redirect_spectre_wide_r_and_b_overflow_v_gets_both() {
    // r > 255 and b > 255: v gets bleed from both
    let col = c(300.0, 100.0, 300.0);
    let result = redirect_spectre_wide(col);
    // v: r > 255 and b > 255 → v + r + b - 510 = 100 + 300 + 300 - 510 = 190
    assert!(approx(result.g, 190.0));
}

#[test]
fn redirect_spectre_wide_b_extreme_overflow_bleeds_to_r() {
    // b > 510, v ≤ 255: r gets bleed from b extreme
    let col = c(100.0, 100.0, 600.0);
    let result = redirect_spectre_wide(col);
    // r: b > 510, v ≤ 255 → r + b - 510 = 100 + 600 - 510 = 190
    assert!(approx(result.r, 190.0));
}

// ---------------------------------------------------------------------------
// rgb_of_hdr
// ---------------------------------------------------------------------------

#[test]
fn rgb_of_hdr_black_is_zeros() {
    let result = rgb_of_hdr(HdrColor::zero(), &HdrColor::zero(), &HdrColor::one(), 1.0);
    assert_eq!(result, [0, 0, 0, 255]);
}

#[test]
fn rgb_of_hdr_alpha_always_255() {
    let result = rgb_of_hdr(
        c(100.0, 150.0, 200.0),
        &HdrColor::zero(),
        &HdrColor::one(),
        1.0,
    );
    assert_eq!(result[3], 255);
}

#[test]
fn rgb_of_hdr_clamping_above_255() {
    // Channel > 255 should be clamped to 255
    let result = rgb_of_hdr(
        c(1000.0, 0.0, 0.0),
        &HdrColor::zero(),
        &HdrColor::one(),
        1.0,
    );
    assert_eq!(result[0], 255);
}

#[test]
fn rgb_of_hdr_negative_clamped_to_zero() {
    let result = rgb_of_hdr(
        c(-100.0, -200.0, -300.0),
        &HdrColor::zero(),
        &HdrColor::one(),
        1.0,
    );
    assert_eq!(result[0], 0);
    assert_eq!(result[1], 0);
    assert_eq!(result[2], 0);
}

#[test]
fn rgb_of_hdr_mid_range_passthrough() {
    // No add color, identity mul, exposure=1 → values pass through as-is
    let result = rgb_of_hdr(
        c(100.0, 150.0, 200.0),
        &HdrColor::zero(),
        &HdrColor::one(),
        1.0,
    );
    assert_eq!(result[0], 100);
    assert_eq!(result[1], 150);
    assert_eq!(result[2], 200);
    assert_eq!(result[3], 255);
}

#[test]
fn rgb_of_hdr_mul_color_darkens() {
    let half = HdrColor::new(0.5, 0.5, 0.5);
    let result = rgb_of_hdr(c(100.0, 100.0, 100.0), &HdrColor::zero(), &half, 1.0);
    assert_eq!(result[0], 50);
    assert_eq!(result[1], 50);
    assert_eq!(result[2], 50);
}

#[test]
fn rgb_of_hdr_add_color_brightens() {
    let add = HdrColor::new(50.0, 50.0, 50.0);
    let result = rgb_of_hdr(c(100.0, 100.0, 100.0), &add, &HdrColor::one(), 1.0);
    assert_eq!(result[0], 150);
    assert_eq!(result[1], 150);
    assert_eq!(result[2], 150);
}

#[test]
fn rgb_of_hdr_zero_exposure_ignores_add_color() {
    let add = HdrColor::new(100.0, 100.0, 100.0);
    // exposure=0 → intensify(add, 0)=zero → add has no effect
    let result = rgb_of_hdr(c(100.0, 150.0, 200.0), &add, &HdrColor::one(), 0.0);
    assert_eq!(result[0], 100);
    assert_eq!(result[1], 150);
    assert_eq!(result[2], 200);
}

// ---------------------------------------------------------------------------
// saturate
// ---------------------------------------------------------------------------

#[test]
fn saturate_identity_one() {
    let a = c(100.0, 150.0, 200.0);
    assert!(color_approx(saturate(a, 1.0), a));
}

#[test]
fn saturate_zero_gives_grayscale() {
    let a = c(90.0, 150.0, 210.0);
    let result = saturate(a, 0.0);
    let expected_value = (90.0 + 150.0 + 210.0) / 3.0;
    assert!(approx(result.r, expected_value));
    assert!(approx(result.g, expected_value));
    assert!(approx(result.b, expected_value));
}

#[test]
fn saturate_uniform_color_unchanged() {
    // A color with all channels equal should be unchanged at any saturation
    let a = c(100.0, 100.0, 100.0);
    assert!(color_approx(saturate(a, 0.0), a));
    assert!(color_approx(saturate(a, 0.5), a));
    assert!(color_approx(saturate(a, 2.0), a));
}

#[test]
fn saturate_increases_contrast_above_one() {
    // i > 1 should push channels further from the mean
    let a = c(50.0, 100.0, 150.0);
    let result = saturate(a, 2.0);
    // mean = 100, r is below mean: should go further below
    assert!(result.r < 50.0);
    // b is above mean: should go further above
    assert!(result.b > 150.0);
    // v is at mean: should stay at mean
    assert!(approx(result.g, 100.0));
}

#[test]
fn saturate_partial_moves_toward_gray() {
    let a = c(0.0, 0.0, 300.0);
    let result = saturate(a, 0.5);
    let mean = 100.0;
    // r: 0.5*0 + 0.5*100 = 50
    assert!(approx(result.r, 50.0));
    // v: 0.5*0 + 0.5*100 = 50
    assert!(approx(result.g, 50.0));
    // b: 0.5*300 + 0.5*100 = 200
    assert!(approx(result.b, 200.0));
    let _ = mean;
}

#[test]
fn saturate_negative_i_inverts_deviation() {
    // i = -1: channels mirror around mean
    let a = c(50.0, 100.0, 150.0);
    let result = saturate(a, -1.0);
    let mean = 100.0;
    // r: -1*50 + 2*100 = 150
    assert!(approx(result.r, -1.0 * 50.0 + 2.0 * mean));
    // b: -1*150 + 2*100 = 50
    assert!(approx(result.b, -1.0 * 150.0 + 2.0 * mean));
    // v: -1*100 + 2*100 = 100 (unchanged, at mean)
    assert!(approx(result.g, 100.0));
}

#[test]
fn saturate_zero_color_stays_zero() {
    assert!(color_approx(
        saturate(HdrColor::zero(), 0.5),
        HdrColor::zero()
    ));
}
