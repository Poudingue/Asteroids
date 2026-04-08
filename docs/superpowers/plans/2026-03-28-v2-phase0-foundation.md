# V2 Phase 0: Foundation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure the Asteroids codebase from a monolithic French-named design to a modular English-named architecture with Vec2 struct, split config, and split modules — with zero behavioral changes, verified by exhaustive mathematical property tests.

**Architecture:** Phase 0 is purely structural. Every function keeps its exact behavior. Tests are written first as a safety net, then the codebase is incrementally restructured: Vec2 struct → HdrColor cleanup → French→English rename → file split → config split → bug fixes. Each step compiles and passes all tests.

**Tech Stack:** Rust, wgpu (unchanged), SDL2 (unchanged), cargo test

---

## Task 0: Add `src/lib.rs` (Required for Integration Tests)

**Why:** The crate has no `[lib]` section in `Cargo.toml` — it is a binary-only crate. Integration tests in the `tests/` directory can only import library crates (`use asteroids::...`), not binary crates. We must expose all modules through a library target before any test files in `tests/` can be written.

**Strategy:** Make `src/main.rs` a thin entry point and move all module declarations into `src/lib.rs`. `main.rs` calls `asteroids::run()`.

### Step 0.1: Create `src/lib.rs`

- [ ] Create `src/lib.rs` that re-exports all modules and defines a `pub fn run()` entry point
- [ ] Add `[lib]` to `Cargo.toml` (or rely on the convention: `src/lib.rs` is auto-detected)
- [ ] Run `cargo check`

**src/lib.rs:**
```rust
pub mod color;
pub mod game;
pub mod math_utils;
pub mod objects;
pub mod parameters;
pub mod renderer;
```

Note: All modules must be `pub mod` (not `mod`) so integration tests can import them.

### Step 0.2: Make `src/main.rs` a thin entry point

- [ ] Remove all `mod` declarations from `src/main.rs` (they move to `lib.rs`)
- [ ] Replace module declarations with `use asteroids::*;` or direct qualified paths
- [ ] Run `cargo check`
- [ ] Run `cargo build` — binary still compiles

**Verify:** `cargo build` produces a working binary. Run the game briefly to confirm it starts.

### Step 0.3: Commit

- [ ] Commit: "refactor: add src/lib.rs to enable integration tests"

---

## Task 1: Exhaustive Math Function Tests (math_utils.rs)

**File:** `tests/math_properties.rs` (new file, integration test)
**Prerequisite:** Task 0 must be complete (lib.rs exists, all modules are `pub mod`)

Write tests for every function in `math_utils.rs` against the CURRENT French-named API. These tests must all pass before any refactoring begins.

### Step 1.1: Create test file

- [ ] Create `tests/math_properties.rs` with the crate import header
- [ ] Run `cargo test --test math_properties` to verify empty test file compiles

**tests/math_properties.rs header:**
```rust
use asteroids::math_utils::*;
```

Note: All functions under test must be `pub` in `math_utils.rs`. If any are not, add `pub` to their signatures before writing the test.

**Verify:** `cargo test --test math_properties -- --list` shows no errors.

### Step 1.2: Tests for `carre` (squared)

- [ ] Write tests for `carre`
- [ ] Run `cargo test --test math_properties test_carre`

**tests/math_properties.rs:**
```rust
use asteroids::math_utils::*;

// ============================================================================
// carre (squared)
// ============================================================================

#[test]
fn test_carre_positive() {
    assert_eq!(carre(3.0), 9.0);
    assert_eq!(carre(5.0), 25.0);
    assert_eq!(carre(10.0), 100.0);
}

#[test]
fn test_carre_negative() {
    assert_eq!(carre(-3.0), 9.0);
    assert_eq!(carre(-5.0), 25.0);
    assert_eq!(carre(-10.0), 100.0);
}

#[test]
fn test_carre_zero() {
    assert_eq!(carre(0.0), 0.0);
}

#[test]
fn test_carre_one() {
    assert_eq!(carre(1.0), 1.0);
    assert_eq!(carre(-1.0), 1.0);
}

#[test]
fn test_carre_symmetry() {
    // (-x)^2 == x^2
    let values = [0.5, 1.0, 2.5, 100.0, 1e-10, 1e10];
    for v in values {
        assert_eq!(carre(v), carre(-v), "carre({v}) != carre(-{v})");
    }
}

#[test]
fn test_carre_non_negative() {
    let values = [-1e15, -1.0, -1e-15, 0.0, 1e-15, 1.0, 1e15];
    for v in values {
        assert!(carre(v) >= 0.0, "carre({v}) = {} is negative", carre(v));
    }
}

#[test]
fn test_carre_large_values() {
    assert_eq!(carre(1e7), 1e14);
    assert_eq!(carre(1e-7), 1e-14);
}

#[test]
fn test_carre_fractional() {
    let eps = 1e-12;
    assert!((carre(0.5) - 0.25).abs() < eps);
    assert!((carre(1.5) - 2.25).abs() < eps);
    assert!((carre(0.1) - 0.01).abs() < eps);
}
```

**Verify:** `cargo test --test math_properties test_carre` — all pass.

### Step 1.3: Tests for tuple arithmetic (`addtuple`, `soustuple`, `multuple`)

- [ ] Write tests for `addtuple`, `soustuple`, `multuple`
- [ ] Run `cargo test --test math_properties test_addtuple test_soustuple test_multuple`

**Append to tests/math_properties.rs:**
```rust
// ============================================================================
// addtuple (vector addition)
// ============================================================================

#[test]
fn test_addtuple_basic() {
    assert_eq!(addtuple((1.0, 2.0), (3.0, 4.0)), (4.0, 6.0));
    assert_eq!(addtuple((0.0, 0.0), (5.0, 5.0)), (5.0, 5.0));
    assert_eq!(addtuple((-1.0, -2.0), (1.0, 2.0)), (0.0, 0.0));
}

#[test]
fn test_addtuple_identity() {
    let v = (42.0, -17.5);
    assert_eq!(addtuple(v, (0.0, 0.0)), v);
    assert_eq!(addtuple((0.0, 0.0), v), v);
}

#[test]
fn test_addtuple_commutativity() {
    let pairs = [
        ((1.0, 2.0), (3.0, 4.0)),
        ((-5.0, 10.0), (7.0, -3.0)),
        ((1e10, 1e-10), (-1e10, 1e-10)),
        ((0.0, 0.0), (0.0, 0.0)),
    ];
    for (a, b) in pairs {
        assert_eq!(addtuple(a, b), addtuple(b, a), "commutativity failed for {a:?} + {b:?}");
    }
}

#[test]
fn test_addtuple_associativity() {
    let a = (1.0, 2.0);
    let b = (3.0, 4.0);
    let c = (5.0, 6.0);
    let lhs = addtuple(addtuple(a, b), c);
    let rhs = addtuple(a, addtuple(b, c));
    let eps = 1e-12;
    assert!((lhs.0 - rhs.0).abs() < eps, "associativity x failed");
    assert!((lhs.1 - rhs.1).abs() < eps, "associativity y failed");
}

#[test]
fn test_addtuple_inverse() {
    let v = (42.0, -17.5);
    let neg = (-42.0, 17.5);
    let result = addtuple(v, neg);
    assert_eq!(result, (0.0, 0.0));
}

#[test]
fn test_addtuple_large_values() {
    let a = (1e15, -1e15);
    let b = (-1e15, 1e15);
    assert_eq!(addtuple(a, b), (0.0, 0.0));
}

// ============================================================================
// soustuple (vector subtraction)
// ============================================================================

#[test]
fn test_soustuple_basic() {
    assert_eq!(soustuple((5.0, 10.0), (3.0, 4.0)), (2.0, 6.0));
    assert_eq!(soustuple((0.0, 0.0), (1.0, 1.0)), (-1.0, -1.0));
}

#[test]
fn test_soustuple_self_is_zero() {
    let values = [(1.0, 2.0), (-5.0, 10.0), (0.0, 0.0), (1e15, -1e15)];
    for v in values {
        let result = soustuple(v, v);
        assert_eq!(result, (0.0, 0.0), "v - v != 0 for {v:?}");
    }
}

#[test]
fn test_soustuple_zero_right() {
    let v = (42.0, -17.5);
    assert_eq!(soustuple(v, (0.0, 0.0)), v);
}

#[test]
fn test_soustuple_is_add_negative() {
    let a = (3.0, 7.0);
    let b = (1.0, 2.0);
    let sub = soustuple(a, b);
    let add_neg = addtuple(a, (-b.0, -b.1));
    assert_eq!(sub, add_neg);
}

#[test]
fn test_soustuple_anti_commutativity() {
    let a = (3.0, 7.0);
    let b = (1.0, 2.0);
    let ab = soustuple(a, b);
    let ba = soustuple(b, a);
    let eps = 1e-12;
    assert!((ab.0 + ba.0).abs() < eps, "a-b + b-a != 0 in x");
    assert!((ab.1 + ba.1).abs() < eps, "a-b + b-a != 0 in y");
}

// ============================================================================
// multuple (scalar multiplication)
// ============================================================================

#[test]
fn test_multuple_basic() {
    assert_eq!(multuple((3.0, 4.0), 2.0), (6.0, 8.0));
    assert_eq!(multuple((1.0, -1.0), 5.0), (5.0, -5.0));
}

#[test]
fn test_multuple_identity() {
    let v = (42.0, -17.5);
    assert_eq!(multuple(v, 1.0), v);
}

#[test]
fn test_multuple_zero() {
    let v = (42.0, -17.5);
    assert_eq!(multuple(v, 0.0), (0.0, 0.0));
}

#[test]
fn test_multuple_negative_one() {
    let v = (42.0, -17.5);
    assert_eq!(multuple(v, -1.0), (-42.0, 17.5));
}

#[test]
fn test_multuple_distributive_over_addition() {
    let a = (3.0, 7.0);
    let b = (1.0, 2.0);
    let k = 2.5;
    let lhs = multuple(addtuple(a, b), k);
    let rhs = addtuple(multuple(a, k), multuple(b, k));
    let eps = 1e-12;
    assert!((lhs.0 - rhs.0).abs() < eps, "distributivity x failed");
    assert!((lhs.1 - rhs.1).abs() < eps, "distributivity y failed");
}

#[test]
fn test_multuple_associative_scalar() {
    let v = (3.0, 7.0);
    let a = 2.0;
    let b = 3.0;
    let lhs = multuple(multuple(v, a), b);
    let rhs = multuple(v, a * b);
    let eps = 1e-12;
    assert!((lhs.0 - rhs.0).abs() < eps, "scalar associativity x failed");
    assert!((lhs.1 - rhs.1).abs() < eps, "scalar associativity y failed");
}

#[test]
fn test_multuple_large_values() {
    let v = (1e10, -1e10);
    let result = multuple(v, 1e5);
    assert_eq!(result, (1e15, -1e15));
}

#[test]
fn test_multuple_small_values() {
    let v = (1e-10, -1e-10);
    let result = multuple(v, 1e-5);
    let eps = 1e-27;
    assert!((result.0 - 1e-15).abs() < eps);
    assert!((result.1 - (-1e-15)).abs() < eps);
}
```

**Verify:** `cargo test --test math_properties test_addtuple test_soustuple test_multuple` — all pass.

### Step 1.4: Tests for `hypothenuse` (magnitude) and `distancecarre` (distance squared)

- [ ] Write tests for `hypothenuse` and `distancecarre`
- [ ] Run `cargo test --test math_properties test_hypo test_distancecarre`

**Append to tests/math_properties.rs:**
```rust
// ============================================================================
// hypothenuse (vector magnitude)
// ============================================================================

#[test]
fn test_hypothenuse_known_triangles() {
    let eps = 1e-10;
    assert!((hypothenuse((3.0, 4.0)) - 5.0).abs() < eps, "3-4-5 triangle");
    assert!((hypothenuse((5.0, 12.0)) - 13.0).abs() < eps, "5-12-13 triangle");
    assert!((hypothenuse((8.0, 15.0)) - 17.0).abs() < eps, "8-15-17 triangle");
}

#[test]
fn test_hypothenuse_zero() {
    assert_eq!(hypothenuse((0.0, 0.0)), 0.0);
}

#[test]
fn test_hypothenuse_unit_axes() {
    assert_eq!(hypothenuse((1.0, 0.0)), 1.0);
    assert_eq!(hypothenuse((0.0, 1.0)), 1.0);
    assert_eq!(hypothenuse((-1.0, 0.0)), 1.0);
    assert_eq!(hypothenuse((0.0, -1.0)), 1.0);
}

#[test]
fn test_hypothenuse_non_negative() {
    let values: Vec<Vec2> = vec![
        (1.0, 2.0), (-1.0, -2.0), (0.0, 0.0), (1e10, -1e10), (1e-10, 1e-10),
    ];
    for v in values {
        assert!(hypothenuse(v) >= 0.0, "hypothenuse({v:?}) is negative");
    }
}

#[test]
fn test_hypothenuse_scaling() {
    let v = (3.0, 4.0);
    let k = 7.0;
    let eps = 1e-10;
    let scaled = hypothenuse(multuple(v, k));
    let expected = k * hypothenuse(v);
    assert!((scaled - expected).abs() < eps, "|k*v| != k*|v|");
}

#[test]
fn test_hypothenuse_symmetry_negate() {
    let v = (3.0, 4.0);
    assert_eq!(hypothenuse(v), hypothenuse((-v.0, -v.1)));
    assert_eq!(hypothenuse(v), hypothenuse((-v.0, v.1)));
    assert_eq!(hypothenuse(v), hypothenuse((v.0, -v.1)));
}

#[test]
fn test_hypothenuse_triangle_inequality() {
    let a = (3.0, 4.0);
    let b = (1.0, 7.0);
    let sum = addtuple(a, b);
    assert!(hypothenuse(sum) <= hypothenuse(a) + hypothenuse(b) + 1e-10,
            "triangle inequality violated");
}

#[test]
fn test_hypothenuse_equals_sqrt_distancecarre_from_origin() {
    let v = (7.0, 11.0);
    let eps = 1e-10;
    let h = hypothenuse(v);
    let dc = distancecarre((0.0, 0.0), v);
    assert!((h * h - dc).abs() < eps, "hypothenuse^2 != distancecarre from origin");
}

// ============================================================================
// distancecarre (distance squared between two points)
// ============================================================================

#[test]
fn test_distancecarre_basic() {
    assert_eq!(distancecarre((0.0, 0.0), (3.0, 4.0)), 25.0);
    assert_eq!(distancecarre((1.0, 1.0), (4.0, 5.0)), 25.0);
}

#[test]
fn test_distancecarre_self_is_zero() {
    let values: Vec<Vec2> = vec![
        (0.0, 0.0), (42.0, -17.5), (1e15, 1e15),
    ];
    for v in values {
        assert_eq!(distancecarre(v, v), 0.0, "d(v,v) != 0 for {v:?}");
    }
}

#[test]
fn test_distancecarre_symmetry() {
    let pairs = [
        ((1.0, 2.0), (3.0, 4.0)),
        ((-5.0, 10.0), (7.0, -3.0)),
        ((0.0, 0.0), (1e10, 1e10)),
    ];
    for (a, b) in pairs {
        assert_eq!(distancecarre(a, b), distancecarre(b, a),
                   "d(a,b) != d(b,a) for {a:?}, {b:?}");
    }
}

#[test]
fn test_distancecarre_non_negative() {
    let pairs = [
        ((1.0, 2.0), (3.0, 4.0)),
        ((-5.0, 10.0), (7.0, -3.0)),
        ((0.0, 0.0), (0.0, 0.0)),
    ];
    for (a, b) in pairs {
        assert!(distancecarre(a, b) >= 0.0, "d^2({a:?},{b:?}) is negative");
    }
}

#[test]
fn test_distancecarre_triangle_inequality() {
    // d(a,c) <= (sqrt(d(a,b)) + sqrt(d(b,c)))^2
    let a = (0.0, 0.0);
    let b = (3.0, 0.0);
    let c = (3.0, 4.0);
    let dac = distancecarre(a, c);
    let dab = distancecarre(a, b).sqrt();
    let dbc = distancecarre(b, c).sqrt();
    assert!(dac <= (dab + dbc).powi(2) + 1e-10, "triangle inequality on d^2");
}
```

**Verify:** `cargo test --test math_properties test_hypo test_distancecarre` — all pass.

### Step 1.5: Tests for `moytuple`, `moyfloat`, `multuple_parallel`, `entretuple`, `proj`

- [ ] Write tests
- [ ] Run `cargo test --test math_properties test_moy test_multuple_parallel test_entretuple test_proj`

**Append to tests/math_properties.rs:**
```rust
// ============================================================================
// moyfloat (linear interpolation between floats)
// ============================================================================

#[test]
fn test_moyfloat_endpoints() {
    assert_eq!(moyfloat(10.0, 20.0, 1.0), 10.0); // ratio=1 → val1
    assert_eq!(moyfloat(10.0, 20.0, 0.0), 20.0); // ratio=0 → val2
}

#[test]
fn test_moyfloat_midpoint() {
    let eps = 1e-12;
    assert!((moyfloat(10.0, 20.0, 0.5) - 15.0).abs() < eps);
    assert!((moyfloat(0.0, 100.0, 0.5) - 50.0).abs() < eps);
}

#[test]
fn test_moyfloat_quarter() {
    let eps = 1e-12;
    // ratio=0.25: 0.25*val1 + 0.75*val2
    assert!((moyfloat(0.0, 100.0, 0.25) - 75.0).abs() < eps);
    assert!((moyfloat(0.0, 100.0, 0.75) - 25.0).abs() < eps);
}

#[test]
fn test_moyfloat_same_value() {
    assert_eq!(moyfloat(42.0, 42.0, 0.3), 42.0);
    assert_eq!(moyfloat(42.0, 42.0, 0.7), 42.0);
}

// ============================================================================
// moytuple (linear interpolation between vectors)
// ============================================================================

#[test]
fn test_moytuple_endpoints() {
    let a = (10.0, 20.0);
    let b = (30.0, 40.0);
    assert_eq!(moytuple(a, b, 1.0), a); // ratio=1 → tuple1
    assert_eq!(moytuple(a, b, 0.0), b); // ratio=0 → tuple2
}

#[test]
fn test_moytuple_midpoint() {
    let a = (0.0, 0.0);
    let b = (10.0, 20.0);
    let mid = moytuple(a, b, 0.5);
    let eps = 1e-12;
    assert!((mid.0 - 5.0).abs() < eps);
    assert!((mid.1 - 10.0).abs() < eps);
}

#[test]
fn test_moytuple_equidistant() {
    // midpoint should be equidistant from both inputs
    let a = (3.0, 7.0);
    let b = (11.0, -5.0);
    let mid = moytuple(a, b, 0.5);
    let da = distancecarre(mid, a);
    let db = distancecarre(mid, b);
    let eps = 1e-10;
    assert!((da - db).abs() < eps, "midpoint not equidistant: da={da}, db={db}");
}

#[test]
fn test_moytuple_same_value() {
    let v = (42.0, -17.5);
    let result = moytuple(v, v, 0.3);
    let eps = 1e-12;
    assert!((result.0 - v.0).abs() < eps);
    assert!((result.1 - v.1).abs() < eps);
}

// ============================================================================
// multuple_parallel (element-wise multiplication)
// ============================================================================

#[test]
fn test_multuple_parallel_basic() {
    assert_eq!(multuple_parallel((2.0, 3.0), (4.0, 5.0)), (8.0, 15.0));
}

#[test]
fn test_multuple_parallel_identity() {
    let v = (42.0, -17.5);
    assert_eq!(multuple_parallel(v, (1.0, 1.0)), v);
}

#[test]
fn test_multuple_parallel_zero() {
    let v = (42.0, -17.5);
    assert_eq!(multuple_parallel(v, (0.0, 0.0)), (0.0, 0.0));
}

#[test]
fn test_multuple_parallel_commutativity() {
    let a = (3.0, 7.0);
    let b = (2.0, 5.0);
    assert_eq!(multuple_parallel(a, b), multuple_parallel(b, a));
}

// ============================================================================
// entretuple (point-in-AABB test)
// ============================================================================

#[test]
fn test_entretuple_inside() {
    assert!(entretuple((5.0, 5.0), (0.0, 0.0), (10.0, 10.0)));
}

#[test]
fn test_entretuple_outside() {
    assert!(!entretuple((-1.0, 5.0), (0.0, 0.0), (10.0, 10.0)));
    assert!(!entretuple((5.0, -1.0), (0.0, 0.0), (10.0, 10.0)));
    assert!(!entretuple((11.0, 5.0), (0.0, 0.0), (10.0, 10.0)));
    assert!(!entretuple((5.0, 11.0), (0.0, 0.0), (10.0, 10.0)));
}

#[test]
fn test_entretuple_boundary_excluded() {
    // Uses strict inequality (> and <), so boundary points are excluded
    assert!(!entretuple((0.0, 5.0), (0.0, 0.0), (10.0, 10.0)));
    assert!(!entretuple((10.0, 5.0), (0.0, 0.0), (10.0, 10.0)));
    assert!(!entretuple((5.0, 0.0), (0.0, 0.0), (10.0, 10.0)));
    assert!(!entretuple((5.0, 10.0), (0.0, 0.0), (10.0, 10.0)));
}

// ============================================================================
// proj (project tuple2 onto tuple1 with ratio)
// ============================================================================

#[test]
fn test_proj_basic() {
    // proj(a, b, r) = a + b*r
    assert_eq!(proj((1.0, 2.0), (3.0, 4.0), 1.0), (4.0, 6.0));
    assert_eq!(proj((1.0, 2.0), (3.0, 4.0), 0.0), (1.0, 2.0));
    assert_eq!(proj((1.0, 2.0), (3.0, 4.0), 2.0), (7.0, 10.0));
}

#[test]
fn test_proj_negative_ratio() {
    assert_eq!(proj((10.0, 10.0), (3.0, 4.0), -1.0), (7.0, 6.0));
}
```

**Verify:** `cargo test --test math_properties test_moy test_multuple_parallel test_entretuple test_proj` — all pass.

### Step 1.6: Tests for polar/cartesian conversions

- [ ] Write tests for `polar_to_affine`, `affine_to_polar`, roundtrip
- [ ] Run `cargo test --test math_properties test_polar`

**Append to tests/math_properties.rs:**
```rust
use std::f64::consts::PI;

// ============================================================================
// polar_to_affine / affine_to_polar (polar ↔ cartesian conversion)
// ============================================================================

#[test]
fn test_polar_to_affine_unit_circle() {
    let eps = 1e-10;
    let (x, y) = polar_to_affine(0.0, 1.0);
    assert!((x - 1.0).abs() < eps, "0 rad: x={x}");
    assert!(y.abs() < eps, "0 rad: y={y}");

    let (x, y) = polar_to_affine(PI / 2.0, 1.0);
    assert!(x.abs() < eps, "pi/2: x={x}");
    assert!((y - 1.0).abs() < eps, "pi/2: y={y}");

    let (x, y) = polar_to_affine(PI, 1.0);
    assert!((x - (-1.0)).abs() < eps, "pi: x={x}");
    assert!(y.abs() < eps, "pi: y={y}");

    let (x, y) = polar_to_affine(3.0 * PI / 2.0, 1.0);
    assert!(x.abs() < eps, "3pi/2: x={x}");
    assert!((y - (-1.0)).abs() < eps, "3pi/2: y={y}");
}

#[test]
fn test_polar_to_affine_zero_magnitude() {
    assert_eq!(polar_to_affine(0.0, 0.0), (0.0, 0.0));
    assert_eq!(polar_to_affine(PI, 0.0), (0.0, 0.0));
}

#[test]
fn test_polar_to_affine_scaling() {
    let eps = 1e-10;
    let r = 5.0;
    let (x, y) = polar_to_affine(PI / 4.0, r);
    let expected = r / 2.0_f64.sqrt();
    assert!((x - expected).abs() < eps);
    assert!((y - expected).abs() < eps);
}

#[test]
fn test_affine_to_polar_known() {
    let eps = 1e-10;
    let (angle, mag) = affine_to_polar((1.0, 0.0));
    assert!(angle.abs() < eps, "angle for (1,0): {angle}");
    assert!((mag - 1.0).abs() < eps, "mag for (1,0): {mag}");

    let (angle, mag) = affine_to_polar((0.0, 1.0));
    assert!((angle - PI / 2.0).abs() < eps, "angle for (0,1): {angle}");
    assert!((mag - 1.0).abs() < eps, "mag for (0,1): {mag}");
}

#[test]
fn test_affine_to_polar_zero() {
    assert_eq!(affine_to_polar((0.0, 0.0)), (0.0, 0.0));
}

#[test]
fn test_affine_to_polar_magnitude() {
    let eps = 1e-10;
    let (_, mag) = affine_to_polar((3.0, 4.0));
    assert!((mag - 5.0).abs() < eps);
}

#[test]
fn test_polar_roundtrip() {
    // to_polar then from_polar should give back original (within epsilon)
    let test_vecs: Vec<Vec2> = vec![
        (1.0, 0.0), (0.0, 1.0), (3.0, 4.0), (-5.0, 12.0),
        (1.0, 1.0), (100.0, 200.0), (0.001, 0.002),
    ];
    let eps = 1e-8;
    for v in test_vecs {
        let (angle, mag) = affine_to_polar(v);
        let back = polar_to_affine(angle, mag);
        assert!((back.0 - v.0).abs() < eps, "roundtrip x failed for {v:?}: got {back:?}");
        assert!((back.1 - v.1).abs() < eps, "roundtrip y failed for {v:?}: got {back:?}");
    }
}

#[test]
fn test_polar_to_affine_tuple() {
    let eps = 1e-10;
    let (x1, y1) = polar_to_affine(PI / 3.0, 2.0);
    let (x2, y2) = polar_to_affine_tuple((PI / 3.0, 2.0));
    assert!((x1 - x2).abs() < eps);
    assert!((y1 - y2).abs() < eps);
}
```

**Verify:** `cargo test --test math_properties test_polar test_affine` — all pass.

### Step 1.7: Tests for modulo/wrapping functions

- [ ] Write tests for `modulo_float`, `modulo_reso`, `modulo_3reso`
- [ ] Run `cargo test --test math_properties test_modulo`

**Append to tests/math_properties.rs:**
```rust
// ============================================================================
// modulo_float / modulo_reso / modulo_3reso (wrapping)
// ============================================================================

#[test]
fn test_modulo_float_in_range() {
    // Already in range: unchanged
    assert_eq!(modulo_float(5.0, 10.0), 5.0);
    assert_eq!(modulo_float(0.0, 10.0), 0.0);
    assert_eq!(modulo_float(9.99, 10.0), 9.99);
}

#[test]
fn test_modulo_float_overflow() {
    // value >= modulo: subtract modulo (single step)
    assert_eq!(modulo_float(10.0, 10.0), 0.0);
    assert_eq!(modulo_float(15.0, 10.0), 5.0);
}

#[test]
fn test_modulo_float_underflow() {
    // value < 0: add modulo (single step)
    assert_eq!(modulo_float(-1.0, 10.0), 9.0);
    assert_eq!(modulo_float(-5.0, 10.0), 5.0);
}

#[test]
fn test_modulo_reso_basic() {
    let w = 1000.0;
    let h = 800.0;
    // In-range
    assert_eq!(modulo_reso((500.0, 400.0), w, h), (500.0, 400.0));
    // Overflow
    assert_eq!(modulo_reso((1500.0, 900.0), w, h), (500.0, 100.0));
    // Underflow
    assert_eq!(modulo_reso((-100.0, -200.0), w, h), (900.0, 600.0));
}

#[test]
fn test_modulo_reso_idempotent_in_range() {
    let w = 1000.0;
    let h = 800.0;
    let v = (500.0, 400.0);
    assert_eq!(modulo_reso(v, w, h), modulo_reso(modulo_reso(v, w, h), w, h));
}

#[test]
fn test_modulo_3reso_basic() {
    let w = 1000.0;
    let h = 800.0;
    // A point at (0,0) should stay at (0,0) — it's in the [-w,2w) range
    assert_eq!(modulo_3reso((0.0, 0.0), w, h), (0.0, 0.0));
    // A point at center should stay
    assert_eq!(modulo_3reso((500.0, 400.0), w, h), (500.0, 400.0));
}

#[test]
fn test_modulo_3reso_range() {
    let w = 1000.0;
    let h = 800.0;
    // Output should be in [-w, 2w) range for x, [-h, 2h) for y
    let test_points: Vec<Vec2> = vec![
        (0.0, 0.0), (500.0, 400.0), (-500.0, -400.0),
        (1500.0, 1200.0), (2500.0, 2000.0), (-1500.0, -1200.0),
    ];
    for p in test_points {
        let (rx, ry) = modulo_3reso(p, w, h);
        assert!(rx >= -w && rx < 2.0 * w,
                "modulo_3reso x out of range for {p:?}: got {rx}");
        assert!(ry >= -h && ry < 2.0 * h,
                "modulo_3reso y out of range for {p:?}: got {ry}");
    }
}
```

**Verify:** `cargo test --test math_properties test_modulo` — all pass.

### Step 1.8: Tests for exponential decay functions

- [ ] Write tests for `exp_decay` and `abso_exp_decay`
- [ ] Run `cargo test --test math_properties test_exp_decay test_abso_exp`

**Append to tests/math_properties.rs:**
```rust
// ============================================================================
// exp_decay / abso_exp_decay
// ============================================================================

#[test]
fn test_abso_exp_decay_half_life() {
    // After exactly one half-life, value should be halved
    let eps = 1e-10;
    let n = 100.0;
    let half_life = 2.0;
    let t0 = 0.0;
    let t1 = 2.0; // dt = 2.0 = one half-life
    // abso_exp_decay uses (t0 - t1) / half_life in exponent
    // n * 2^((0 - 2) / 2) = n * 2^(-1) = n/2
    let result = abso_exp_decay(n, half_life, t0, t1);
    assert!((result - 50.0).abs() < eps, "half-life result: {result}");
}

#[test]
fn test_abso_exp_decay_zero_dt() {
    // No time elapsed: value unchanged
    let result = abso_exp_decay(100.0, 2.0, 5.0, 5.0);
    assert_eq!(result, 100.0);
}

#[test]
fn test_abso_exp_decay_monotonic_decrease() {
    // As time increases, value decreases (for positive n)
    let n = 100.0;
    let half_life = 1.0;
    let t0 = 0.0;
    let v1 = abso_exp_decay(n, half_life, t0, 1.0);
    let v2 = abso_exp_decay(n, half_life, t0, 2.0);
    let v3 = abso_exp_decay(n, half_life, t0, 3.0);
    assert!(v1 > v2, "v1={v1} should be > v2={v2}");
    assert!(v2 > v3, "v2={v2} should be > v3={v3}");
    assert!(v3 > 0.0, "v3={v3} should be positive");
}

#[test]
fn test_abso_exp_decay_double_half_life() {
    let eps = 1e-10;
    let n = 100.0;
    let half_life = 2.0;
    // Two half-lives: n * 2^(-2) = n/4
    let result = abso_exp_decay(n, half_life, 0.0, 4.0);
    assert!((result - 25.0).abs() < eps, "two half-lives: {result}");
}

#[test]
fn test_abso_exp_decay_zero_value() {
    assert_eq!(abso_exp_decay(0.0, 1.0, 0.0, 5.0), 0.0);
}

#[test]
fn test_exp_decay_half_life() {
    // exp_decay with observer_proper_time=1, game_speed=1, proper_time=1
    // should match abso_exp_decay
    let eps = 1e-10;
    let n = 100.0;
    let half_life = 2.0;
    let result = exp_decay(n, half_life, 1.0, 1.0, 0.0, 2.0, 1.0);
    assert!((result - 50.0).abs() < eps, "exp_decay half-life: {result}");
}

#[test]
fn test_exp_decay_game_speed_scaling() {
    // Double game speed should double the effective time passage
    let n = 100.0;
    let half_life = 2.0;
    let result_1x = exp_decay(n, half_life, 1.0, 1.0, 0.0, 1.0, 1.0);
    let result_2x = exp_decay(n, half_life, 1.0, 2.0, 0.0, 1.0, 1.0);
    let result_1x_2t = exp_decay(n, half_life, 1.0, 1.0, 0.0, 2.0, 1.0);
    let eps = 1e-10;
    assert!((result_2x - result_1x_2t).abs() < eps,
            "2x speed for 1s should equal 1x speed for 2s");
}

#[test]
fn test_exp_decay_zero_dt() {
    let result = exp_decay(100.0, 2.0, 1.0, 1.0, 5.0, 5.0, 1.0);
    assert_eq!(result, 100.0);
}
```

**Verify:** `cargo test --test math_properties test_exp_decay test_abso_exp` — all pass.

### Step 1.9: Tests for conversion and dithering functions

- [ ] Write tests for `inttuple`, `floattuple`, `randfloat`, `diff`
- [ ] Run `cargo test --test math_properties test_inttuple test_floattuple test_randfloat test_diff`

**Append to tests/math_properties.rs:**
```rust
use rand::thread_rng;

// ============================================================================
// inttuple / floattuple
// ============================================================================

#[test]
fn test_inttuple_basic() {
    assert_eq!(inttuple((3.7, 4.2)), (3, 4));
    assert_eq!(inttuple((0.0, 0.0)), (0, 0));
    assert_eq!(inttuple((-1.9, -2.1)), (-1, -2)); // truncation toward zero
}

#[test]
fn test_floattuple_basic() {
    assert_eq!(floattuple((3, 4)), (3.0, 4.0));
    assert_eq!(floattuple((-1, -2)), (-1.0, -2.0));
}

#[test]
fn test_inttuple_floattuple_roundtrip_integers() {
    // For integer-valued f64, roundtrip should be exact
    let v = (42.0, -17.0);
    let back = floattuple(inttuple(v));
    assert_eq!(back, v);
}

// ============================================================================
// randfloat
// ============================================================================

#[test]
fn test_randfloat_in_range() {
    let mut rng = thread_rng();
    for _ in 0..1000 {
        let v = randfloat(5.0, 10.0, &mut rng);
        assert!(v >= 5.0 && v < 10.0, "randfloat out of range: {v}");
    }
}

#[test]
fn test_randfloat_min_eq_max() {
    let mut rng = thread_rng();
    let v = randfloat(5.0, 5.0, &mut rng);
    assert_eq!(v, 5.0);
}

// ============================================================================
// diff (set difference)
// ============================================================================

#[test]
fn test_diff_basic() {
    assert_eq!(diff(&[1, 2, 3, 4], &[2, 4]), vec![1, 3]);
    assert_eq!(diff(&[1, 2, 3], &[1, 2, 3]), Vec::<i32>::new());
    assert_eq!(diff(&[1, 2, 3], &[4, 5, 6]), vec![1, 2, 3]);
    assert_eq!(diff::<i32>(&[], &[1, 2]), Vec::<i32>::new());
}
```

**Verify:** `cargo test --test math_properties test_inttuple test_floattuple test_randfloat test_diff` — all pass.

### Step 1.10: Commit math tests

- [ ] `cargo test --test math_properties` — all math tests pass
- [ ] Commit: "test: exhaustive math_utils property tests (Phase 0 safety net)"

---

## Task 2: Exhaustive Color Function Tests (color.rs)

**File:** `tests/color_properties.rs` (new file, integration test)
**Prerequisite:** Task 0 must be complete.

### Step 2.1: Write color tests

- [ ] Create `tests/color_properties.rs`
- [ ] Run `cargo test --test color_properties`

Note: All functions under test must be `pub` in `color.rs`. `HdrColor` and all color functions must be `pub`.

**tests/color_properties.rs:**
```rust
use asteroids::color::*;

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-10
}

fn colors_eq(a: HdrColor, b: HdrColor) -> bool {
    approx(a.r, b.r) && approx(a.v, b.v) && approx(a.b, b.b)
}

// ============================================================================
// HdrColor constructors
// ============================================================================

#[test]
fn test_hdr_new() {
    let c = HdrColor::new(1.0, 2.0, 3.0);
    assert_eq!(c.r, 1.0);
    assert_eq!(c.v, 2.0);
    assert_eq!(c.b, 3.0);
}

#[test]
fn test_hdr_zero() {
    let c = HdrColor::zero();
    assert_eq!(c.r, 0.0);
    assert_eq!(c.v, 0.0);
    assert_eq!(c.b, 0.0);
}

#[test]
fn test_hdr_one() {
    let c = HdrColor::one();
    assert_eq!(c.r, 1.0);
    assert_eq!(c.v, 1.0);
    assert_eq!(c.b, 1.0);
}

// ============================================================================
// hdr_add
// ============================================================================

#[test]
fn test_hdr_add_basic() {
    let a = HdrColor::new(1.0, 2.0, 3.0);
    let b = HdrColor::new(4.0, 5.0, 6.0);
    let c = hdr_add(a, b);
    assert!(colors_eq(c, HdrColor::new(5.0, 7.0, 9.0)));
}

#[test]
fn test_hdr_add_identity() {
    let a = HdrColor::new(10.0, 20.0, 30.0);
    let c = hdr_add(a, HdrColor::zero());
    assert!(colors_eq(c, a));
}

#[test]
fn test_hdr_add_commutativity() {
    let a = HdrColor::new(1.0, 2.0, 3.0);
    let b = HdrColor::new(4.0, 5.0, 6.0);
    assert!(colors_eq(hdr_add(a, b), hdr_add(b, a)));
}

#[test]
fn test_hdr_add_associativity() {
    let a = HdrColor::new(1.0, 2.0, 3.0);
    let b = HdrColor::new(4.0, 5.0, 6.0);
    let c = HdrColor::new(7.0, 8.0, 9.0);
    let lhs = hdr_add(hdr_add(a, b), c);
    let rhs = hdr_add(a, hdr_add(b, c));
    assert!(colors_eq(lhs, rhs));
}

// ============================================================================
// hdr_sous
// ============================================================================

#[test]
fn test_hdr_sous_basic() {
    let a = HdrColor::new(5.0, 7.0, 9.0);
    let b = HdrColor::new(1.0, 2.0, 3.0);
    let c = hdr_sous(a, b);
    assert!(colors_eq(c, HdrColor::new(4.0, 5.0, 6.0)));
}

#[test]
fn test_hdr_sous_self_is_zero() {
    let a = HdrColor::new(10.0, 20.0, 30.0);
    let c = hdr_sous(a, a);
    assert!(colors_eq(c, HdrColor::zero()));
}

#[test]
fn test_hdr_sous_zero_right() {
    let a = HdrColor::new(10.0, 20.0, 30.0);
    let c = hdr_sous(a, HdrColor::zero());
    assert!(colors_eq(c, a));
}

// ============================================================================
// hdr_mul
// ============================================================================

#[test]
fn test_hdr_mul_basic() {
    let a = HdrColor::new(2.0, 3.0, 4.0);
    let b = HdrColor::new(5.0, 6.0, 7.0);
    let c = hdr_mul(a, b);
    assert!(colors_eq(c, HdrColor::new(10.0, 18.0, 28.0)));
}

#[test]
fn test_hdr_mul_identity() {
    let a = HdrColor::new(10.0, 20.0, 30.0);
    let c = hdr_mul(a, HdrColor::one());
    assert!(colors_eq(c, a));
}

#[test]
fn test_hdr_mul_zero() {
    let a = HdrColor::new(10.0, 20.0, 30.0);
    let c = hdr_mul(a, HdrColor::zero());
    assert!(colors_eq(c, HdrColor::zero()));
}

#[test]
fn test_hdr_mul_commutativity() {
    let a = HdrColor::new(2.0, 3.0, 4.0);
    let b = HdrColor::new(5.0, 6.0, 7.0);
    assert!(colors_eq(hdr_mul(a, b), hdr_mul(b, a)));
}

// ============================================================================
// intensify
// ============================================================================

#[test]
fn test_intensify_basic() {
    let c = HdrColor::new(10.0, 20.0, 30.0);
    let result = intensify(c, 2.0);
    assert!(colors_eq(result, HdrColor::new(20.0, 40.0, 60.0)));
}

#[test]
fn test_intensify_one() {
    let c = HdrColor::new(10.0, 20.0, 30.0);
    let result = intensify(c, 1.0);
    assert!(colors_eq(result, c));
}

#[test]
fn test_intensify_zero() {
    let c = HdrColor::new(10.0, 20.0, 30.0);
    let result = intensify(c, 0.0);
    assert!(colors_eq(result, HdrColor::zero()));
}

#[test]
fn test_intensify_is_scalar_mul() {
    // intensify(c, k) == hdr_mul(c, HdrColor::new(k, k, k))
    let c = HdrColor::new(10.0, 20.0, 30.0);
    let k = 3.5;
    let via_intensify = intensify(c, k);
    let via_mul = hdr_mul(c, HdrColor::new(k, k, k));
    assert!(colors_eq(via_intensify, via_mul));
}

// ============================================================================
// half_color (exponential interpolation)
// ============================================================================

#[test]
fn test_half_color_zero_dt() {
    let a = HdrColor::new(100.0, 200.0, 300.0);
    let b = HdrColor::new(10.0, 20.0, 30.0);
    let result = half_color(a, b, 1.0, 0.0);
    // dt=0: no decay, result = b + (a-b)*1 = a
    assert!(colors_eq(result, a));
}

#[test]
fn test_half_color_large_dt() {
    let a = HdrColor::new(100.0, 200.0, 300.0);
    let b = HdrColor::new(10.0, 20.0, 30.0);
    // After very large dt, should converge to b
    let result = half_color(a, b, 0.01, 100.0);
    assert!(approx(result.r, b.r));
    assert!(approx(result.v, b.v));
    assert!(approx(result.b, b.b));
}

#[test]
fn test_half_color_same_colors() {
    let c = HdrColor::new(50.0, 50.0, 50.0);
    let result = half_color(c, c, 1.0, 5.0);
    assert!(colors_eq(result, c));
}

// ============================================================================
// redirect_spectre_wide
// ============================================================================

#[test]
fn test_redirect_spectre_wide_no_overflow() {
    // All channels under 255: no redistribution
    let c = HdrColor::new(100.0, 100.0, 100.0);
    let result = redirect_spectre_wide(c);
    assert!(colors_eq(result, c));
}

#[test]
fn test_redirect_spectre_wide_green_overflow() {
    // v > 255: bleeds into r and b
    let c = HdrColor::new(100.0, 300.0, 100.0);
    let result = redirect_spectre_wide(c);
    // r gets col.v - 255 = 45 added: 100 + 45 = 145
    assert!(approx(result.r, 145.0));
    // v unchanged by itself (unless r or b > 255)
    assert!(approx(result.v, 300.0));
    // b gets col.v - 255 = 45 added: 100 + 45 = 145
    assert!(approx(result.b, 145.0));
}

#[test]
fn test_redirect_spectre_wide_known_case() {
    let c = HdrColor::new(50.0, 100.0, 50.0);
    let result = redirect_spectre_wide(c);
    // No channel > 255, so no change
    assert!(colors_eq(result, c));
}

// ============================================================================
// rgb_of_hdr
// ============================================================================

#[test]
fn test_rgb_of_hdr_black() {
    let result = rgb_of_hdr(
        HdrColor::zero(),
        &HdrColor::zero(),
        &HdrColor::one(),
        1.0,
    );
    assert_eq!(result, [0, 0, 0, 255]);
}

#[test]
fn test_rgb_of_hdr_clamping() {
    // Very bright color should clamp to 255
    let result = rgb_of_hdr(
        HdrColor::new(1000.0, 1000.0, 1000.0),
        &HdrColor::zero(),
        &HdrColor::one(),
        1.0,
    );
    assert_eq!(result[0], 255);
    assert_eq!(result[1], 255);
    assert_eq!(result[2], 255);
    assert_eq!(result[3], 255);
}

#[test]
fn test_rgb_of_hdr_negative_clamped_to_zero() {
    let result = rgb_of_hdr(
        HdrColor::new(-100.0, -100.0, -100.0),
        &HdrColor::zero(),
        &HdrColor::one(),
        1.0,
    );
    assert_eq!(result, [0, 0, 0, 255]);
}

#[test]
fn test_rgb_of_hdr_alpha_always_255() {
    let test_colors = [
        HdrColor::zero(),
        HdrColor::new(128.0, 128.0, 128.0),
        HdrColor::new(1000.0, 0.0, -50.0),
    ];
    for c in test_colors {
        let result = rgb_of_hdr(c, &HdrColor::zero(), &HdrColor::one(), 1.0);
        assert_eq!(result[3], 255, "alpha not 255 for {c:?}");
    }
}

// ============================================================================
// saturate
// ============================================================================

#[test]
fn test_saturate_identity() {
    let c = HdrColor::new(100.0, 200.0, 50.0);
    let result = saturate(c, 1.0);
    assert!(colors_eq(result, c));
}

#[test]
fn test_saturate_grayscale() {
    let c = HdrColor::new(100.0, 200.0, 50.0);
    let result = saturate(c, 0.0);
    let avg = (100.0 + 200.0 + 50.0) / 3.0;
    assert!(approx(result.r, avg));
    assert!(approx(result.v, avg));
    assert!(approx(result.b, avg));
}

#[test]
fn test_saturate_uniform_color_unchanged() {
    // If all channels equal, saturation change has no effect
    let c = HdrColor::new(100.0, 100.0, 100.0);
    let result = saturate(c, 5.0);
    assert!(colors_eq(result, c));
}
```

**Verify:** `cargo test --test color_properties` — all pass.

### Step 2.2: Commit color tests

- [ ] `cargo test --test color_properties` — all color tests pass
- [ ] Commit: "test: exhaustive color property tests (Phase 0 safety net)"

---

## Task 3: Physics and Movement Function Tests (game.rs)

**File:** `tests/physics_properties.rs` (new file, integration test)
**Prerequisite:** Task 0 must be complete.

### Step 3.1: Write physics/movement tests

- [ ] Create `tests/physics_properties.rs`
- [ ] Run `cargo test --test physics_properties`

Note: All functions under test must be `pub` in `game.rs` and related modules.

**tests/physics_properties.rs:**
```rust
use asteroids::game::*;
use asteroids::math_utils::*;
use asteroids::objects::*;
use asteroids::parameters::*;

/// Create a minimal Globals for testing with a known dt
fn test_globals(dt: f64) -> Globals {
    let mut g = Globals::new();
    g.time_last_frame = 0.0;
    g.time_current_frame = dt;
    g.game_speed = 1.0;
    g.observer_proper_time = 1.0;
    g
}

/// Create a simple test entity at origin with given velocity
fn test_entity(vel: Vec2) -> Entity {
    let mut e = spawn_ship();
    e.position = (0.0, 0.0);
    e.velocity = vel;
    e.proper_time = 1.0;
    e.orientation = 0.0;
    e.moment = 0.0;
    e
}

// ============================================================================
// deplac_objet (move entity by velocity)
// ============================================================================

#[test]
fn test_deplac_objet_basic() {
    let globals = test_globals(1.0);
    let mut entity = test_entity((100.0, 200.0));
    deplac_objet(&mut entity, (100.0, 200.0), &globals);
    let eps = 1e-8;
    assert!((entity.position.0 - 100.0).abs() < eps, "x: {}", entity.position.0);
    assert!((entity.position.1 - 200.0).abs() < eps, "y: {}", entity.position.1);
}

#[test]
fn test_deplac_objet_zero_velocity() {
    let globals = test_globals(1.0);
    let mut entity = test_entity((0.0, 0.0));
    entity.position = (500.0, 300.0);
    deplac_objet(&mut entity, (0.0, 0.0), &globals);
    assert_eq!(entity.position, (500.0, 300.0));
}

#[test]
fn test_deplac_objet_zero_dt() {
    let globals = test_globals(0.0);
    let mut entity = test_entity((100.0, 200.0));
    deplac_objet(&mut entity, (100.0, 200.0), &globals);
    assert_eq!(entity.position, (0.0, 0.0));
}

#[test]
fn test_deplac_objet_game_speed_scaling() {
    let mut globals_1x = test_globals(1.0);
    globals_1x.game_speed = 1.0;
    let mut globals_2x = test_globals(1.0);
    globals_2x.game_speed = 2.0;

    let mut e1 = test_entity((100.0, 0.0));
    let mut e2 = test_entity((100.0, 0.0));
    deplac_objet(&mut e1, (100.0, 0.0), &globals_1x);
    deplac_objet(&mut e2, (100.0, 0.0), &globals_2x);

    let eps = 1e-8;
    assert!((e2.position.0 - 2.0 * e1.position.0).abs() < eps,
            "2x game speed should double displacement");
}

// ============================================================================
// inertie_objet (apply entity's own velocity as displacement)
// ============================================================================

#[test]
fn test_inertie_objet_basic() {
    let globals = test_globals(1.0);
    let mut entity = test_entity((50.0, -30.0));
    inertie_objet(&mut entity, &globals);
    let eps = 1e-8;
    assert!((entity.position.0 - 50.0).abs() < eps);
    assert!((entity.position.1 - (-30.0)).abs() < eps);
}

#[test]
fn test_inertie_objet_stationary() {
    let globals = test_globals(1.0);
    let mut entity = test_entity((0.0, 0.0));
    entity.position = (100.0, 200.0);
    inertie_objet(&mut entity, &globals);
    assert_eq!(entity.position, (100.0, 200.0));
}

// ============================================================================
// accel_objet (accelerate entity)
// ============================================================================

#[test]
fn test_accel_objet_basic() {
    let globals = test_globals(1.0);
    let mut entity = test_entity((0.0, 0.0));
    accel_objet(&mut entity, (100.0, 200.0), &globals);
    let eps = 1e-8;
    assert!((entity.velocity.0 - 100.0).abs() < eps);
    assert!((entity.velocity.1 - 200.0).abs() < eps);
}

#[test]
fn test_accel_objet_cumulative() {
    let globals = test_globals(1.0);
    let mut entity = test_entity((50.0, 50.0));
    accel_objet(&mut entity, (10.0, 20.0), &globals);
    let eps = 1e-8;
    assert!((entity.velocity.0 - 60.0).abs() < eps);
    assert!((entity.velocity.1 - 70.0).abs() < eps);
}

// ============================================================================
// boost_objet (instant velocity change)
// ============================================================================

#[test]
fn test_boost_objet_basic() {
    let mut entity = test_entity((100.0, 200.0));
    boost_objet(&mut entity, (50.0, -50.0));
    assert_eq!(entity.velocity, (150.0, 150.0));
}

// ============================================================================
// rotat_objet / tourn_objet / moment_objet
// ============================================================================

#[test]
fn test_rotat_objet_basic() {
    let globals = test_globals(1.0);
    let mut entity = test_entity((0.0, 0.0));
    entity.orientation = 0.0;
    rotat_objet(&mut entity, 1.0, &globals);
    let eps = 1e-8;
    assert!((entity.orientation - 1.0).abs() < eps);
}

#[test]
fn test_tourn_objet_basic() {
    let mut entity = test_entity((0.0, 0.0));
    entity.orientation = 0.5;
    tourn_objet(&mut entity, 0.3);
    let eps = 1e-12;
    assert!((entity.orientation - 0.8).abs() < eps);
}

#[test]
fn test_moment_objet_applies_angular_velocity() {
    let globals = test_globals(1.0);
    let mut entity = test_entity((0.0, 0.0));
    entity.orientation = 0.0;
    entity.moment = 2.0;
    moment_objet(&mut entity, &globals);
    let eps = 1e-8;
    assert!((entity.orientation - 2.0).abs() < eps);
}

// ============================================================================
// Collision functions
// ============================================================================

#[test]
fn test_collision_circles_overlapping() {
    use asteroids::game::collision_circles;
    assert!(collision_circles((0.0, 0.0), 10.0, (15.0, 0.0), 10.0));
}

#[test]
fn test_collision_circles_not_overlapping() {
    use asteroids::game::collision_circles;
    assert!(!collision_circles((0.0, 0.0), 10.0, (25.0, 0.0), 10.0));
}

#[test]
fn test_collision_circles_touching() {
    use asteroids::game::collision_circles;
    // Exactly touching: d^2 = (r1+r2)^2, uses < so touching is NOT colliding
    assert!(!collision_circles((0.0, 0.0), 10.0, (20.0, 0.0), 10.0));
}

#[test]
fn test_collision_circles_symmetry() {
    use asteroids::game::collision_circles;
    let pos0 = (5.0, 3.0);
    let pos1 = (12.0, 7.0);
    let r0 = 8.0;
    let r1 = 6.0;
    assert_eq!(
        collision_circles(pos0, r0, pos1, r1),
        collision_circles(pos1, r1, pos0, r0),
    );
}

#[test]
fn test_collision_point_inside() {
    use asteroids::game::collision_point;
    assert!(collision_point((5.0, 5.0), (5.0, 5.0), 1.0)); // same point
    assert!(collision_point((5.0, 5.0), (6.0, 5.0), 2.0)); // 1 unit away, radius 2
}

#[test]
fn test_collision_point_outside() {
    use asteroids::game::collision_point;
    assert!(!collision_point((0.0, 0.0), (10.0, 0.0), 5.0));
}
```

Note: `collision_circles` and `collision_point` are currently private in `game.rs`. Integration tests are external to the crate, so they need full `pub` visibility (not `pub(crate)`).

- [ ] Change `fn collision_circles(` to `pub fn collision_circles(` in `src/game.rs`
- [ ] Change `fn collision_point(` to `pub fn collision_point(` in `src/game.rs`

**Verify:** `cargo test --test physics_properties` — all pass.

### Step 3.2: Commit physics tests

- [ ] `cargo test --test physics_properties` — all physics tests pass
- [ ] Commit: "test: physics and movement function tests (Phase 0 safety net)"

---

## Task 4: Entity Predicate Tests (objects.rs)

**File:** `tests/entity_properties.rs` (new file, integration test)
**Prerequisite:** Task 0 must be complete.

### Step 4.1: Write entity predicate tests

- [ ] Create `tests/entity_properties.rs`
- [ ] Run `cargo test --test entity_properties`

Note: All functions and types under test must be `pub` in `objects.rs` and `math_utils.rs`.

**tests/entity_properties.rs:**
```rust
use asteroids::objects::*;
use asteroids::math_utils::*;
use rand::thread_rng;

fn make_entity_with_health(h: f64) -> Entity {
    let mut e = spawn_ship();
    e.health = h;
    e
}

fn make_entity_with_radii(int_r: f64, ext_r: f64, vis_r: f64) -> Entity {
    let mut e = spawn_ship();
    e.hitbox.int_radius = int_r;
    e.hitbox.ext_radius = ext_r;
    e.visuals.radius = vis_r;
    e
}

// ============================================================================
// is_alive / is_dead — complementary
// ============================================================================

#[test]
fn test_is_alive_positive_health() {
    let e = make_entity_with_health(50.0);
    assert!(is_alive(&e));
    assert!(!is_dead(&e));
}

#[test]
fn test_is_dead_zero_health() {
    let e = make_entity_with_health(0.0);
    assert!(is_dead(&e));
    assert!(!is_alive(&e));
}

#[test]
fn test_is_dead_negative_health() {
    let e = make_entity_with_health(-10.0);
    assert!(is_dead(&e));
    assert!(!is_alive(&e));
}

#[test]
fn test_alive_dead_complementary() {
    let health_values = [-100.0, -0.001, 0.0, 0.001, 1.0, 100.0];
    for h in health_values {
        let e = make_entity_with_health(h);
        assert_ne!(is_alive(&e), is_dead(&e),
                   "is_alive and is_dead must be complementary for health={h}");
    }
}

// ============================================================================
// too_small / big_enough — complementary
// ============================================================================

#[test]
fn test_big_enough_large() {
    let e = make_entity_with_radii(200.0, 200.0, 200.0);
    assert!(big_enough(&e));
    assert!(!too_small(&e));
}

#[test]
fn test_too_small_tiny() {
    let e = make_entity_with_radii(10.0, 50.0, 10.0);
    assert!(too_small(&e));
    assert!(!big_enough(&e));
}

#[test]
fn test_too_small_big_enough_complementary() {
    let radii = [0.0, 50.0, 99.9, 100.0, 100.1, 500.0];
    for r in radii {
        let e = make_entity_with_radii(r, r, r);
        assert_ne!(too_small(&e), big_enough(&e),
                   "too_small and big_enough must be complementary for ext_radius={r}");
    }
}

// ============================================================================
// positive_radius
// ============================================================================

#[test]
fn test_positive_radius_yes() {
    let e = make_entity_with_radii(10.0, 10.0, 5.0);
    assert!(positive_radius(&e));
}

#[test]
fn test_positive_radius_zero() {
    let e = make_entity_with_radii(10.0, 10.0, 0.0);
    assert!(!positive_radius(&e));
}

#[test]
fn test_positive_radius_negative() {
    let e = make_entity_with_radii(10.0, 10.0, -1.0);
    assert!(!positive_radius(&e));
}

// ============================================================================
// is_chunk / not_chunk
// ============================================================================

#[test]
fn test_is_chunk_small() {
    let e = make_entity_with_radii(30.0, 30.0, 30.0);
    assert!(is_chunk(&e));
    assert!(!not_chunk(&e));
}

#[test]
fn test_not_chunk_large() {
    let e = make_entity_with_radii(100.0, 100.0, 100.0);
    assert!(not_chunk(&e));
    assert!(!is_chunk(&e));
}

// ============================================================================
// check_spawn / check_not_spawn — complementary
// ============================================================================

#[test]
fn test_check_spawn_visible() {
    let mut e = spawn_ship();
    e.position = (500.0, 400.0);
    e.hitbox.ext_radius = 50.0;
    assert!(check_spawn(&e, 1000.0, 800.0));
    assert!(!check_not_spawn(&e, 1000.0, 800.0));
}

#[test]
fn test_check_spawn_offscreen() {
    let mut e = spawn_ship();
    e.position = (-200.0, -200.0);
    e.hitbox.ext_radius = 50.0;
    assert!(!check_spawn(&e, 1000.0, 800.0));
    assert!(check_not_spawn(&e, 1000.0, 800.0));
}

#[test]
fn test_check_spawn_complementary() {
    let positions: Vec<Vec2> = vec![
        (500.0, 400.0), (-200.0, 400.0), (500.0, -200.0),
        (1100.0, 400.0), (500.0, 900.0), (0.0, 0.0),
    ];
    for pos in positions {
        let mut e = spawn_ship();
        e.position = pos;
        e.hitbox.ext_radius = 50.0;
        assert_ne!(check_spawn(&e, 1000.0, 800.0), check_not_spawn(&e, 1000.0, 800.0),
                   "check_spawn and check_not_spawn not complementary at {pos:?}");
    }
}

// ============================================================================
// Spawn functions produce valid entities
// ============================================================================

#[test]
fn test_spawn_ship_valid() {
    let ship = spawn_ship();
    assert_eq!(ship.kind, EntityKind::Ship);
    assert!(ship.health > 0.0);
    assert!(ship.hitbox.ext_radius > 0.0);
    assert!(ship.mass > 0.0);
}

#[test]
fn test_spawn_asteroid_valid() {
    let mut rng = thread_rng();
    let asteroid = spawn_asteroid((100.0, 100.0), (50.0, 50.0), 300.0, &mut rng);
    assert_eq!(asteroid.kind, EntityKind::Asteroid);
    assert!(asteroid.health > 0.0);
    assert!(asteroid.hitbox.ext_radius > 0.0);
    assert!(asteroid.mass > 0.0);
    assert!(!asteroid.hitbox.points.0.is_empty());
}

#[test]
fn test_spawn_projectile_dead_on_arrival() {
    let proj = spawn_projectile((0.0, 0.0), (100.0, 0.0), 1.0);
    assert_eq!(proj.kind, EntityKind::Projectile);
    // Projectile health is 0.0; it's "dead" by health < 0 check, but
    // actually uses health < 0 (not <=) for projectile death, so health=0 is alive
    assert_eq!(proj.health, 0.0);
}

#[test]
fn test_spawn_explosion_valid() {
    let mut rng = thread_rng();
    let proj = spawn_projectile((0.0, 0.0), (100.0, 0.0), 1.0);
    let explo = spawn_explosion(&proj, &mut rng);
    assert_eq!(explo.kind, EntityKind::Explosion);
    assert!(explo.hitbox.ext_radius > 0.0);
}
```

**Verify:** `cargo test --test entity_properties` — all pass.

### Step 4.2: Commit entity tests

- [ ] `cargo test --test entity_properties` — all entity tests pass
- [ ] Commit: "test: entity predicate and spawn tests (Phase 0 safety net)"

---

## Task 5: Vec2 Struct

### Step 5.1: Create `src/math.rs` with Vec2 struct

- [ ] Create `src/math.rs` with Vec2 struct and all std::ops implementations
- [ ] Add `pub mod math;` to `src/lib.rs` (not main.rs — lib.rs owns module declarations after Task 0)
- [ ] Run `cargo check`

**src/math.rs:**
```rust
use std::ops::{Add, Sub, Mul, Neg, AddAssign, SubAssign, MulAssign};

/// 2D vector with f64 components.
/// Replaces the old `type Vec2 = (f64, f64)` tuple alias.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    /// Vector magnitude (length).
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Squared magnitude (avoids sqrt).
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Distance to another point.
    pub fn distance(self, other: Vec2) -> f64 {
        (self - other).length()
    }

    /// Squared distance to another point.
    pub fn distance_squared(self, other: Vec2) -> f64 {
        (self - other).length_squared()
    }

    /// Linear interpolation: self * ratio + other * (1 - ratio).
    /// ratio=1 returns self, ratio=0 returns other.
    pub fn lerp(self, other: Vec2, ratio: f64) -> Vec2 {
        Vec2 {
            x: self.x * ratio + other.x * (1.0 - ratio),
            y: self.y * ratio + other.y * (1.0 - ratio),
        }
    }

    /// Element-wise multiplication.
    pub fn component_mul(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    /// Convert to polar (angle, magnitude).
    pub fn to_polar(self) -> (f64, f64) {
        let r = self.length();
        if r == 0.0 {
            (0.0, 0.0)
        } else {
            (2.0 * (self.y / (self.x + r)).atan(), r)
        }
    }

    /// Create from polar (angle, magnitude).
    pub fn from_polar(angle: f64, magnitude: f64) -> Vec2 {
        Vec2 {
            x: magnitude * angle.cos(),
            y: magnitude * angle.sin(),
        }
    }

    /// Convert to integer tuple (truncation).
    pub fn to_i32(self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }

    /// Create from integer tuple.
    pub fn from_i32(v: (i32, i32)) -> Vec2 {
        Vec2 {
            x: v.0 as f64,
            y: v.1 as f64,
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Vec2 {
        Vec2 { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Mul<Vec2> for f64 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self * rhs.x, y: self * rhs.y }
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2 { x: -self.x, y: -self.y }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

/// Convenience: construct Vec2 from a tuple (for migration).
impl From<(f64, f64)> for Vec2 {
    fn from(t: (f64, f64)) -> Vec2 {
        Vec2 { x: t.0, y: t.1 }
    }
}

/// Convenience: convert Vec2 back to tuple (for migration).
impl From<Vec2> for (f64, f64) {
    fn from(v: Vec2) -> (f64, f64) {
        (v.x, v.y)
    }
}
```

**Verify:** `cargo check` — compiles.

### Step 5.2: Write Vec2 tests

- [ ] Create `tests/vec2_properties.rs`
- [ ] Run `cargo test --test vec2_properties`

Note: `Vec2` must be `pub` in `src/math.rs`, and `math` must be `pub mod` in `src/lib.rs`.

**tests/vec2_properties.rs:**
```rust
use asteroids::math::Vec2;

#[test]
fn test_vec2_add() {
    assert_eq!(Vec2::new(1.0, 2.0) + Vec2::new(3.0, 4.0), Vec2::new(4.0, 6.0));
}

#[test]
fn test_vec2_sub() {
    assert_eq!(Vec2::new(5.0, 10.0) - Vec2::new(3.0, 4.0), Vec2::new(2.0, 6.0));
}

#[test]
fn test_vec2_mul_scalar() {
    assert_eq!(Vec2::new(3.0, 4.0) * 2.0, Vec2::new(6.0, 8.0));
    assert_eq!(2.0 * Vec2::new(3.0, 4.0), Vec2::new(6.0, 8.0));
}

#[test]
fn test_vec2_neg() {
    assert_eq!(-Vec2::new(3.0, -4.0), Vec2::new(-3.0, 4.0));
}

#[test]
fn test_vec2_length() {
    let eps = 1e-10;
    assert!((Vec2::new(3.0, 4.0).length() - 5.0).abs() < eps);
    assert_eq!(Vec2::ZERO.length(), 0.0);
}

#[test]
fn test_vec2_distance() {
    let eps = 1e-10;
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(3.0, 4.0);
    assert!((a.distance(b) - 5.0).abs() < eps);
    assert!((b.distance(a) - 5.0).abs() < eps); // symmetry
}

#[test]
fn test_vec2_lerp() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(10.0, 20.0);
    assert_eq!(a.lerp(b, 1.0), a); // ratio=1 → self
    assert_eq!(a.lerp(b, 0.0), b); // ratio=0 → other
    let mid = a.lerp(b, 0.5);
    let eps = 1e-12;
    assert!((mid.x - 5.0).abs() < eps);
    assert!((mid.y - 10.0).abs() < eps);
}

#[test]
fn test_vec2_polar_roundtrip() {
    let eps = 1e-8;
    let test_vecs = [
        Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0),
        Vec2::new(3.0, 4.0), Vec2::new(-5.0, 12.0),
    ];
    for v in test_vecs {
        let (angle, mag) = v.to_polar();
        let back = Vec2::from_polar(angle, mag);
        assert!((back.x - v.x).abs() < eps, "roundtrip x for {v:?}");
        assert!((back.y - v.y).abs() < eps, "roundtrip y for {v:?}");
    }
}

#[test]
fn test_vec2_from_tuple() {
    let v: Vec2 = (3.0, 4.0).into();
    assert_eq!(v, Vec2::new(3.0, 4.0));
}

#[test]
fn test_vec2_into_tuple() {
    let v = Vec2::new(3.0, 4.0);
    let t: (f64, f64) = v.into();
    assert_eq!(t, (3.0, 4.0));
}

#[test]
fn test_vec2_add_assign() {
    let mut v = Vec2::new(1.0, 2.0);
    v += Vec2::new(3.0, 4.0);
    assert_eq!(v, Vec2::new(4.0, 6.0));
}

#[test]
fn test_vec2_zero() {
    assert_eq!(Vec2::ZERO, Vec2::new(0.0, 0.0));
    assert_eq!(Vec2::new(5.0, 3.0) + Vec2::ZERO, Vec2::new(5.0, 3.0));
}
```

**Verify:** `cargo test --test vec2_properties` — all pass.

### Step 5.3: Migrate callers to Vec2 struct

This is a large mechanical step. The migration strategy:

1. In `math_utils.rs`, change `pub type Vec2 = (f64, f64);` to re-export: `pub use crate::math::Vec2;`
2. Update all functions in `math_utils.rs` to use `Vec2 { x, y }` instead of `(v.0, v.1)`
3. Update `objects.rs`: all `position: Vec2`, `velocity: Vec2`, etc.
4. Update `game.rs`: all tuple access to `.x` / `.y`
5. Update `parameters.rs`: all `(f64, f64)` fields that represent positions/velocities
6. Update `renderer.rs`: no changes needed (uses `f64` directly)

- [ ] Change `math_utils.rs` type alias to re-export
- [ ] Update all `math_utils.rs` function bodies
- [ ] Update `objects.rs`
- [ ] Update `parameters.rs` tuple fields
- [ ] Update `game.rs`
- [ ] Run `cargo check` after each file
- [ ] Run `cargo test` — all existing tests still pass
- [ ] Commit: "refactor: replace Vec2 tuple alias with Vec2 struct"

**Key migration patterns:**

| Old code | New code |
|----------|----------|
| `(x, y)` literal | `Vec2::new(x, y)` |
| `v.0` | `v.x` |
| `v.1` | `v.y` |
| `let (x, y) = v;` | `let Vec2 { x, y } = v;` or `let x = v.x; let y = v.y;` |
| `addtuple(a, b)` | Keep for now (renamed in Task 7) |
| `(0.0, 0.0)` | `Vec2::ZERO` |

The `From<(f64, f64)>` and `Into<(f64, f64)>` impls allow gradual migration — callers that still use tuples can `.into()` at boundaries.

---

## Task 6: HdrColor Cleanup

### Step 6.1: Rename `v` field to `g`

- [ ] In `src/color.rs`, rename field `v` to `g` in `HdrColor` struct
- [ ] Update all references in `color.rs` (constructors, `hdr_add`, `hdr_sous`, `hdr_mul`, `intensify`, `half_color`, `redirect_spectre`, `redirect_spectre_wide`, `rgb_of_hdr`, `saturate`)
- [ ] Update all references in `game.rs` (every `HdrColor { r: ..., v: ..., b: ... }` and `.v` access)
- [ ] Update all references in `objects.rs` if any
- [ ] Run `cargo check`
- [ ] Update color tests to use `.g` instead of `.v`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "refactor: rename HdrColor.v to HdrColor.g (vert → green)"

### Step 6.2: Add operator impls to HdrColor

- [ ] Add `impl Add for HdrColor` (replaces `hdr_add`)
- [ ] Add `impl Sub for HdrColor` (replaces `hdr_sous`)
- [ ] Add `impl Mul for HdrColor` (replaces `hdr_mul`, component-wise)
- [ ] Add `impl Mul<f64> for HdrColor` (replaces `intensify`)
- [ ] Keep the free functions as wrappers for now (they'll be removed in the rename task)
- [ ] Run `cargo test` — all pass
- [ ] Commit: "refactor: add std::ops to HdrColor"

**Add to `src/color.rs`:**
```rust
impl std::ops::Add for HdrColor {
    type Output = HdrColor;
    fn add(self, rhs: HdrColor) -> HdrColor {
        HdrColor { r: self.r + rhs.r, g: self.g + rhs.g, b: self.b + rhs.b }
    }
}

impl std::ops::Sub for HdrColor {
    type Output = HdrColor;
    fn sub(self, rhs: HdrColor) -> HdrColor {
        HdrColor { r: self.r - rhs.r, g: self.g - rhs.g, b: self.b - rhs.b }
    }
}

impl std::ops::Mul for HdrColor {
    type Output = HdrColor;
    fn mul(self, rhs: HdrColor) -> HdrColor {
        HdrColor { r: self.r * rhs.r, g: self.g * rhs.g, b: self.b * rhs.b }
    }
}

impl std::ops::Mul<f64> for HdrColor {
    type Output = HdrColor;
    fn mul(self, rhs: f64) -> HdrColor {
        HdrColor { r: self.r * rhs, g: self.g * rhs, b: self.b * rhs }
    }
}

impl std::ops::Mul<HdrColor> for f64 {
    type Output = HdrColor;
    fn mul(self, rhs: HdrColor) -> HdrColor {
        HdrColor { r: self * rhs.r, g: self * rhs.g, b: self * rhs.b }
    }
}

impl std::ops::AddAssign for HdrColor {
    fn add_assign(&mut self, rhs: HdrColor) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}
```

---

## Task 7: French to English Rename

This is a single atomic commit. Use automated find-replace + `cargo check` after each rename.

### Step 7.1: Rename math_utils.rs functions

- [ ] `carre` → `squared`
- [ ] `hypothenuse` → `magnitude` (free function; Vec2 has `.length()`)
- [ ] `addtuple` → `add_vec`
- [ ] `soustuple` → `sub_vec`
- [ ] `multuple` → `scale_vec`
- [ ] `moytuple` → `lerp_vec`
- [ ] `moyfloat` → `lerp_float`
- [ ] `multuple_parallel` → `component_mul_vec`
- [ ] `entretuple` → `point_in_aabb`
- [ ] `inttuple` → `to_i32_tuple`
- [ ] `floattuple` → `from_i32_tuple`
- [ ] `polar_to_affine` → `from_polar`
- [ ] `polar_to_affine_tuple` → `from_polar_tuple`
- [ ] `affine_to_polar` → `to_polar`
- [ ] `distancecarre` → `distance_squared`
- [ ] `modulo_reso` → `wrap_single`
- [ ] `modulo_3reso` → `wrap_toroidal`
- [ ] `modulo_float` → `wrap_float`
- [ ] `dither_tuple` → `dither_vec`
- [ ] `randfloat` → `rand_range`
- [ ] Run `cargo check` after all renames

### Step 7.2: Rename game.rs functions

- [ ] `deplac_objet` → `move_entity`
- [ ] `inertie_objet` → `apply_inertia`
- [ ] `inertie_objets` → `apply_inertia_all`
- [ ] `accel_objet` → `accelerate_entity`
- [ ] `boost_objet` → `boost_entity`
- [ ] `rotat_objet` → `rotate_entity`
- [ ] `tourn_objet` → `turn_entity`
- [ ] `couple_objet` → `apply_torque`
- [ ] `couple_objet_boost` → `boost_torque`
- [ ] `moment_objet` → `apply_angular_momentum`
- [ ] `moment_objets` → `apply_angular_momentum_all`
- [ ] `deplac_objet_abso` → `translate_entity`
- [ ] `deplac_star` → `move_star`
- [ ] `recenter_objet` → `wrap_entity`
- [ ] `recenter_objets` → `wrap_entities`
- [ ] `checkspawn_objet` → `is_on_screen`
- [ ] `poly_to_affine` → `polygon_to_cartesian`
- [ ] `depl_affine_poly` → `translate_polygon`
- [ ] `affiche_barre` → `render_bar`
- [ ] `decay_smoke` → `decay_smoke` (already English)
- [ ] `ischunk` → `is_chunk`
- [ ] `notchunk` → `is_not_chunk`
- [ ] `drain_filter_stable` → `drain_filter_stable` (already English, will be replaced in Task 14)
- [ ] Run `cargo check`

### Step 7.3: Rename objects.rs functions

- [ ] `spawn_chunk_explo` → `spawn_explosion_chunk_particle`
- [ ] `polygon_asteroid` → `generate_asteroid_polygon`
- [ ] `frag_asteroid` → `fragment_asteroid`
- [ ] `spawn_n_frags` → `spawn_fragments`
- [ ] `random_out_of_screen` → `random_offscreen_position`
- [ ] `n_stars` → `spawn_stars`
- [ ] Run `cargo check`

### Step 7.4: Rename parameters.rs French constants and comments

- [ ] `GAME_SPEED_TARGET_BOUCLE` → `GAME_SPEED_TARGET_LOOP`
- [ ] `ratio_rendu` → `render_scale`
- [ ] `ratio_phys_deg` → `physics_damage_ratio`
- [ ] Any other French identifiers or comments → English
- [ ] Run `cargo check`

### Step 7.5: Update all tests to use new names

- [ ] Update `tests/math_properties.rs` — replace all old French function names with new English names
- [ ] Update `tests/color_properties.rs` — update `.v` → `.g` (HdrColor field rename done in Task 6)
- [ ] Update `tests/physics_properties.rs` — replace all old French function names
- [ ] Update `tests/entity_properties.rs` — replace all old French function names
- [ ] Update `tests/vec2_properties.rs` — verify still correct (Vec2 API is already English)
- [ ] Run `cargo test` — all pass

### Step 7.6: Commit

- [ ] `cargo test` — all pass
- [ ] `cargo clippy` — no warnings
- [ ] Commit: "refactor: French → English rename (all identifiers)"

---

## Task 8: Split game.rs — Extract `src/input.rs`

- [ ] Create `src/input.rs` with functions: `aim_at_mouse`, `acceleration`, `boost_forward`, `teleport`, `handle_left`, `handle_right`, `strafe_left`, `strafe_right`, `tir` (renamed to `fire`)
- [ ] Add `pub mod input;` to `src/main.rs`
- [ ] Update `src/main.rs` call sites to use `input::` prefix
- [ ] Remove moved functions from `src/game.rs`
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "refactor: extract input.rs from game.rs"

---

## Task 9: Split game.rs — Extract `src/camera.rs`

- [ ] Create `src/camera.rs` with functions: `center_of_attention`, `update_camera`
- [ ] Add `pub mod camera;` to `src/main.rs`
- [ ] Update call site in `update_game` to use `camera::update_camera`
- [ ] Remove moved functions from `src/game.rs`
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "refactor: extract camera.rs from game.rs"

---

## Task 10: Split game.rs — Extract `src/pause_menu.rs`

- [ ] Create `src/pause_menu.rs` with: `ButtonBoolean` struct, `make_buttons`, `screen_to_phys_y`, `applique_button` (→ `apply_button`), `render_button_tooltip`, `render_pause_title`
- [ ] Add `pub mod pause_menu;` to `src/main.rs`
- [ ] Update call site in `render_frame`
- [ ] Remove moved code from `src/game.rs`
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "refactor: extract pause_menu.rs from game.rs"

---

## Task 11: Split game.rs — Extract rendering into `src/rendering/`

- [ ] Create `src/rendering/mod.rs` — re-export `Renderer2D` from existing `renderer.rs` (move `renderer.rs` into `rendering/mod.rs`)
- [ ] Create `src/rendering/world.rs` with: `render_visuals`, `render_chunk`, `render_star_trail`, `render_projectile`, `render_light_trail`, `render_poly`, `render_shapes`
- [ ] Create `src/rendering/hud.rs` with: `render_hud`, `render_string`, `render_char`, `shape_char`, `displacement`, `displace_shape`, `render_bar`, `draw_heart`, `draw_n_hearts`, `draw_bar_frame`, `render_scanlines`
- [ ] Update `src/main.rs` module declarations
- [ ] Update `game.rs` `render_frame` to call into rendering submodules
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "refactor: extract rendering modules from game.rs"

---

## Task 12: Split game.rs — Extract collision into `src/physics/`

- [ ] Create `src/physics/mod.rs` — collision grid, detection orchestration
- [ ] Create `src/physics/collision.rs` — `collision_circles`, `collision_point`, `collisions_points`, `collision_poly`, `collision_entities`
- [ ] Create `src/physics/response.rs` — `consequences_collision`, `consequences_collision_frags`, `damage`, `phys_damage`
- [ ] Move `GridEntry`, `CollisionGrid`, `make_grid`, `insert_into_grid`, `collect_pairs_for_cell`, `apply_collision_pairs`, `calculate_collision_tables`, `run_fragment_collisions` into `physics/mod.rs`
- [ ] Update `game.rs` to call `physics::` functions
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "refactor: extract physics modules from game.rs"

After this task, `game.rs` should contain only: `GameState`, `update_game` (orchestration), `render_frame` (orchestration), `despawn`, `transfer_oos`, `update_frame`, and helpers like `hdr`/`to_rgba`.

---

## Task 13: Config Restructure — Split Globals

### Step 13.1: Create sub-structs in `src/parameters.rs`

- [ ] Create `TimeConfig` struct (game_speed, game_speed_target, time fields, pause, restart, quit)
- [ ] Create `ExposureConfig` struct (game_exposure, game_exposure_target, add_color, mul_color, mul_base)
- [ ] Create `VisualConfig` struct (retro, oldschool, scanlines, motion_blur, screenshake_enabled, smoke_enabled, chunks_enabled, flashes_enabled, dyn_color, variable_exposure, space_color/goal, star_color/goal)
- [ ] Create `ShipControlConfig` struct (ship_direct_pos, ship_direct_rotat, ship_impulse_pos, ship_impulse_rotat)
- [ ] Create `RenderState` struct (render_scale, phys_width/height, safe zone fields, jitter)
- [ ] Create `ScreenshakeState` struct (game_screenshake, positions, shake_score)
- [ ] Create `FramerateState` struct (frame_compute_secs, locked_framerate, counters)
- [ ] Create `SpawnState` struct (current_stage_asteroids, time_since_last_spawn, stars_nb)
- [ ] Create `WeaponState` struct (all projectile_ fields, ratio_phys_deg)

### Step 13.2: Compose Globals from sub-structs

- [ ] Refactor `Globals` to contain sub-structs as fields
- [ ] Update `Globals::new()` to construct sub-structs
- [ ] Update all access sites (game.rs, main.rs, etc.) — e.g. `globals.game_speed` → `globals.time.game_speed`
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "refactor: split Globals into focused sub-structs"

**Target `Globals` structure:**
```rust
pub struct Globals {
    pub time: TimeConfig,
    pub exposure: ExposureConfig,
    pub visual: VisualConfig,
    pub ship_control: ShipControlConfig,
    pub render: RenderState,
    pub screenshake: ScreenshakeState,
    pub framerate: FramerateState,
    pub spawn: SpawnState,
    pub weapon: WeaponState,
    pub advanced_hitbox: bool,
    pub observer_proper_time: f64,
}
```

---

## Task 14: Bug Fixes and Cleanups

### Step 14.1: Remove raw pointer hacks

- [ ] In `render_pause_title`: extract `rng` from `GameState`, pass as separate `&mut ThreadRng` parameter
- [ ] In `render_frame`: same pattern — pass `rng` separately
- [ ] In `render_hud`: same pattern
- [ ] Remove all `unsafe` blocks and raw pointer casts in `game.rs`
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "fix: remove raw pointer hacks by extracting rng parameter"

### Step 14.2: Replace `drain_filter_stable` with `Vec::extract_if`

- [ ] Replace all calls to `drain_filter_stable(&mut vec, pred)` with `vec.extract_if(pred).collect()`
- [ ] Remove the `drain_filter_stable` function definition
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "refactor: replace drain_filter_stable with Vec::extract_if"

Note: `Vec::extract_if` requires Rust 1.87+. Verify with `rustc --version`. If not available, keep `drain_filter_stable` and add a comment.

### Step 14.3: Remove dead code

- [ ] Remove `diff` function from `math_utils.rs` (unused)
- [ ] Remove `EntityKind::Spark` variant (never constructed)
- [ ] Remove `dither` function if unused outside scanlines
- [ ] Remove `dither_radius` if only used by removed scanlines
- [ ] Run `cargo check` — verify no compilation errors from removed items
- [ ] Run `cargo test` — all pass
- [ ] Commit: "cleanup: remove dead code (diff, Spark, unused dithering)"

### Step 14.4: Fix EntityKind semantic lies

- [ ] Add `EntityKind::Chunk` variant
- [ ] In `spawn_chunk_explo`, use `EntityKind::Chunk` instead of `EntityKind::Asteroid`
- [ ] Verify no match statements break (search for `EntityKind::Asteroid` matches)
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "fix: EntityKind::Chunk for chunk entities (was incorrectly Asteroid)"

### Step 14.5: Deduplicate constants

- [ ] Remove duplicated constants from `objects.rs` that shadow `parameters.rs`
- [ ] In `objects.rs`, import from `parameters.rs` instead of local `const` declarations
- [ ] Specifically: `SHIP_RADIUS`, `SHIP_DENSITY`, `SHIP_MAX_HEALTH`, `SHIP_DAM_RATIO`, `SHIP_DAM_RES`, `SHIP_PHYS_RATIO`, `SHIP_PHYS_RES`, `PROJECTILE_RADIUS`, `PROJECTILE_RADIUS_HITBOX`, `PROJECTILE_HEALTH`, all `ASTEROID_*`, all `EXPLOSION_*`, all `MUZZLE_*`, all `FIRE_*`, `SMOKE_MAX_SPEED`, `STAR_*`, `CHUNK_MAX_SIZE`, `ASTEROID_MIN_SIZE`, `MAX_DIST`
- [ ] Run `cargo check`
- [ ] Run `cargo test` — all pass
- [ ] Commit: "cleanup: deduplicate constants between objects.rs and parameters.rs"

### Step 14.6: Final verification

- [ ] `cargo check` — no errors
- [ ] `cargo clippy` — no warnings
- [ ] `cargo test` — all tests pass
- [ ] `cargo build --release` — builds successfully
- [ ] Run the game manually — verify it plays identically to V1
- [ ] Commit any final fixes

---

## Summary: Task Checklist

| Task | Description | Commits |
|------|-------------|---------|
| 0 | Add src/lib.rs (prerequisite for integration tests) | 1 |
| 1 | Math function tests (`tests/math_properties.rs`) | 1 |
| 2 | Color function tests (`tests/color_properties.rs`) | 1 |
| 3 | Physics/movement tests (`tests/physics_properties.rs`) | 1 |
| 4 | Entity predicate tests (`tests/entity_properties.rs`) | 1 |
| 5 | Vec2 struct + migration | 2 |
| 6 | HdrColor cleanup (v→g, ops) | 2 |
| 7 | French → English rename | 1 |
| 8 | Extract input.rs | 1 |
| 9 | Extract camera.rs | 1 |
| 10 | Extract pause_menu.rs | 1 |
| 11 | Extract rendering/ modules | 1 |
| 12 | Extract physics/ modules | 1 |
| 13 | Config restructure (Globals split) | 1 |
| 14 | Bug fixes and cleanups | 5 |
| **Total** | | **~21 commits** |

Each task compiles and passes all tests. Zero behavioral changes throughout.

---

## Self-Review Checklist

1. **Does every spec requirement from Phase 0 have a task?**
   - [x] lib.rs prerequisite for integration tests (Task 0)
   - [x] Exhaustive tests in `tests/` directory (Tasks 1-4)
   - [x] Vec2 struct (Task 5)
   - [x] HdrColor cleanup v→g + ops (Task 6)
   - [x] French→English rename (Task 7)
   - [x] File restructure (Tasks 8-12)
   - [x] Config restructure (Task 13)
   - [x] Bug fixes: raw pointers, drain_filter_stable, dead code, EntityKind lies, constant dedup (Task 14)

2. **Are there any placeholders or TODO markers?** No.

3. **Are type names and function signatures consistent across tasks?** Yes — Vec2 struct introduced in Task 5, used thereafter; HdrColor.g after Task 6; English names after Task 7.

4. **Do tests cover all mathematical properties listed in requirements?**
   - [x] carre: non-negative, zero, one, symmetry, large/small values
   - [x] addtuple: commutativity, associativity, identity, inverse
   - [x] soustuple: self-zero, anti-commutativity, zero-right
   - [x] multuple: identity, zero, distributivity, associativity, large/small
   - [x] hypothenuse: known triangles, non-negative, scaling, symmetry, triangle inequality
   - [x] distancecarre: self-zero, symmetry, non-negative, triangle inequality
   - [x] moytuple: endpoints, midpoint equidistant, same-value
   - [x] moyfloat: endpoints, midpoint, same-value
   - [x] polar conversions: unit circle, zero magnitude, scaling, roundtrip
   - [x] modulo functions: in-range, overflow, underflow, idempotent, range check
   - [x] exp_decay: half-life, zero dt, monotonic, game speed scaling
   - [x] HdrColor ops: commutativity, associativity, identity, zero
   - [x] redirect_spectre_wide: no-overflow passthrough, overflow redistribution
   - [x] rgb_of_hdr: black, clamping, negative, alpha
   - [x] saturate: identity, grayscale, uniform color
   - [x] collision_circles: overlapping, non-overlapping, touching, symmetry
   - [x] collision_point: inside, outside
   - [x] Entity predicates: alive/dead complementary, too_small/big_enough complementary, spawn/not_spawn complementary
   - [x] Float edge cases: large values (1e15), small values (1e-15), negative, zero throughout
