# Distortion Field System — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a persistent spatial field system that replaces the one-shot explosion shockwave push with a general-purpose mechanism for velocity fields (wind), gravity, and time dilation zones. Phased: infrastructure first, then expanding ring shockwave, then time dilation, then gravity wells.

**Architecture:** Analytical/mathematical field evaluation — O(entities × sources) with early-out on distance. No grid or quadtree. Each `FieldSource` is a mathematical function evaluated at entity positions. Field evaluation produces a `FieldSample` (wind + gravity + time_dilation) per entity. Sources live in `GameState.field_sources: Vec<FieldSource>`, age each frame, and are removed when expired.

**Tech Stack:** Rust, existing Vec2 math, existing `proper_time` infrastructure on Entity

**Design spec:** `docs/superpowers/specs/2026-04-01-distortion-field-design.md`

**Phasing note:** Phases 0-2 (infrastructure, expanding ring, time dilation) build on each other sequentially and belong in one plan. Phase 3 (gravity wells/black holes) introduces new gameplay and should be a separate plan. Phase 4 (GPU visualization) is a rendering concern and should be a separate plan.

---

## File Structure

| File | Role | Status |
|------|------|--------|
| `src/field.rs` | **NEW** — FieldSource, FieldSourceKind, FieldSample types, evaluate_field(), age/remove logic | Create |
| `src/lib.rs` | Add `pub mod field;` declaration | Modify |
| `src/game.rs` | Add `field_sources` to GameState, spawn ShockwaveRing on explosion, call field evaluation, remove old shockwave loop | Modify |
| `src/parameters.rs` | Add field system constants (ring speed, width, decay, etc.), keep existing SHOCKWAVE constants until migration complete | Modify |
| `tests/field_tests.rs` | **NEW** — Unit tests for field evaluation, ring expansion, decay, toroidal wrap | Create |
| `tests/scenario_tests.rs` | Add determinism test covering field-based shockwave | Modify |

---

## Phase 0: Infrastructure (Tasks 1-3)

### Task 1: FieldSource and FieldSample Types

**Goal:** Define the core data structures in a new `src/field.rs` module.

**Files:**
- Create: `src/field.rs`
- Modify: `src/lib.rs` — add module declaration

#### Step 1.1: Create `src/field.rs` with types

- [ ] Create `src/field.rs` with the following content:

```rust
//! Spatial distortion field system.
//!
//! Analytical field evaluation: each FieldSource is a mathematical function
//! evaluated at entity positions. O(entities × sources) with early-out.

use crate::math_utils::Vec2;
use serde::Serialize;

// ============================================================================
// Types
// ============================================================================

/// A spatial field source — produces wind, gravity, or time dilation.
#[derive(Clone, Debug, Serialize)]
pub struct FieldSource {
    pub kind: FieldSourceKind,
    pub position: Vec2,
    pub age: f64,
    pub max_age: f64, // f64::INFINITY for persistent sources
}

/// The kind-specific parameters of a field source.
#[derive(Clone, Debug, Serialize)]
pub enum FieldSourceKind {
    /// Expanding shockwave ring (replaces one-shot push).
    /// Ring expands outward at `ring_speed` units/s with width `ring_width`.
    ShockwaveRing {
        impulse: f64,
        ring_radius: f64,
        ring_speed: f64,
        ring_width: f64,
        strength_decay: f64, // half-life in seconds
    },
}

/// Result of evaluating all field sources at a single position.
#[derive(Clone, Debug)]
pub struct FieldSample {
    /// Velocity offset (wind / shockwave push).
    pub wind: Vec2,
    /// Multiplier on proper_time (1.0 = normal, >1.0 = slower).
    pub time_dilation: f64,
    /// Gravitational acceleration toward source(s).
    pub gravity: Vec2,
}

impl Default for FieldSample {
    fn default() -> Self {
        Self {
            wind: Vec2::new(0.0, 0.0),
            time_dilation: 1.0,
            gravity: Vec2::new(0.0, 0.0),
        }
    }
}
```

#### Step 1.2: Add module declaration

- [ ] In `src/lib.rs`, add `pub mod field;` in alphabetical order (after `pub mod color;`, before `pub mod game;`):

```rust
pub mod camera;
pub mod color;
pub mod field;
pub mod game;
```

#### Step 1.3: Verify and commit

- [ ] Run:
```bash
rtk cargo check
rtk cargo test
```

- [ ] Expected: compiles, all existing tests pass. No behavioral change.

```bash
rtk git add src/field.rs src/lib.rs && rtk git commit -m "feat(field): add FieldSource, FieldSourceKind, FieldSample types"
```

---

### Task 2: Field Evaluation Function

**Goal:** Implement `evaluate_field()` — iterates sources, early-outs on distance, accumulates contributions. Also implement `advance_sources()` for aging and removal.

**Files:**
- Modify: `src/field.rs` — add evaluate + advance functions
- Create: `tests/field_tests.rs` — unit tests

#### Step 2.1: Write failing tests

- [ ] Create `tests/field_tests.rs`:

```rust
use asteroids::field::*;
use asteroids::math_utils::Vec2;

#[test]
fn test_empty_sources_returns_default() {
    let sample = evaluate_field(Vec2::new(100.0, 100.0), &[], 0.0, 0.0);
    assert_eq!(sample.wind.x, 0.0);
    assert_eq!(sample.wind.y, 0.0);
    assert_eq!(sample.time_dilation, 1.0);
    assert_eq!(sample.gravity.x, 0.0);
    assert_eq!(sample.gravity.y, 0.0);
}

#[test]
fn test_shockwave_ring_push_at_ring_edge() {
    // Source at origin, ring_radius = 100, ring_width = 20.
    // Entity at (100, 0) — exactly on ring → maximum push.
    let source = FieldSource {
        kind: FieldSourceKind::ShockwaveRing {
            impulse: 10.0,
            ring_radius: 100.0,
            ring_speed: 200.0,
            ring_width: 20.0,
            strength_decay: 1.0,
        },
        position: Vec2::new(0.0, 0.0),
        age: 0.0,
        max_age: 5.0,
    };
    let sample = evaluate_field(Vec2::new(100.0, 0.0), &[source], 0.0, 0.0);
    // On the ring center → full strength, direction = +x
    assert!(sample.wind.x > 0.0, "expected positive x push, got {}", sample.wind.x);
    assert!(sample.wind.y.abs() < 1e-10, "expected ~0 y push, got {}", sample.wind.y);
    assert!(
        (sample.wind.x - 10.0).abs() < 1e-10,
        "expected impulse=10 at ring center, got {}",
        sample.wind.x
    );
}

#[test]
fn test_shockwave_ring_no_push_outside_ring() {
    // Ring at radius 100, width 20 → affects [80, 120].
    // Entity at (200, 0) — outside ring → no push.
    let source = FieldSource {
        kind: FieldSourceKind::ShockwaveRing {
            impulse: 10.0,
            ring_radius: 100.0,
            ring_speed: 200.0,
            ring_width: 20.0,
            strength_decay: 1.0,
        },
        position: Vec2::new(0.0, 0.0),
        age: 0.0,
        max_age: 5.0,
    };
    let sample = evaluate_field(Vec2::new(200.0, 0.0), &[source], 0.0, 0.0);
    assert_eq!(sample.wind.x, 0.0);
    assert_eq!(sample.wind.y, 0.0);
}

#[test]
fn test_shockwave_ring_linear_falloff() {
    // Ring at radius 100, width 20.
    // Entity at (110, 0) — halfway between ring center and edge.
    let source = FieldSource {
        kind: FieldSourceKind::ShockwaveRing {
            impulse: 10.0,
            ring_radius: 100.0,
            ring_speed: 200.0,
            ring_width: 20.0,
            strength_decay: f64::INFINITY, // no decay for this test
        },
        position: Vec2::new(0.0, 0.0),
        age: 0.0,
        max_age: 5.0,
    };
    let sample = evaluate_field(Vec2::new(110.0, 0.0), &[source], 0.0, 0.0);
    // |d - r| = 10, w = 20 → strength = 1.0 - 10/20 = 0.5
    assert!(
        (sample.wind.x - 5.0).abs() < 1e-10,
        "expected 5.0, got {}",
        sample.wind.x
    );
}

#[test]
fn test_shockwave_decay_over_time() {
    // Same source, but age > 0 → decayed strength.
    let source = FieldSource {
        kind: FieldSourceKind::ShockwaveRing {
            impulse: 10.0,
            ring_radius: 100.0,
            ring_speed: 200.0,
            ring_width: 20.0,
            strength_decay: 0.5, // half-life = 0.5s
        },
        position: Vec2::new(0.0, 0.0),
        age: 0.5, // one half-life elapsed
        max_age: 5.0,
    };
    // At ring center (100, 0) → base strength 10, decayed by 2^(-0.5/0.5) = 0.5
    let sample = evaluate_field(Vec2::new(100.0, 0.0), &[source], 0.0, 0.0);
    assert!(
        (sample.wind.x - 5.0).abs() < 1e-10,
        "expected 5.0, got {}",
        sample.wind.x
    );
}

#[test]
fn test_advance_sources_ages_and_removes() {
    let mut sources = vec![
        FieldSource {
            kind: FieldSourceKind::ShockwaveRing {
                impulse: 10.0,
                ring_radius: 0.0,
                ring_speed: 200.0,
                ring_width: 20.0,
                strength_decay: 1.0,
            },
            position: Vec2::new(0.0, 0.0),
            age: 0.0,
            max_age: 1.0,
        },
        FieldSource {
            kind: FieldSourceKind::ShockwaveRing {
                impulse: 5.0,
                ring_radius: 0.0,
                ring_speed: 100.0,
                ring_width: 10.0,
                strength_decay: 1.0,
            },
            position: Vec2::new(50.0, 50.0),
            age: 0.9,
            max_age: 1.0,
        },
    ];
    // Advance by 0.2s — first survives (age=0.2 < 1.0), second dies (age=1.1 >= 1.0)
    advance_sources(&mut sources, 0.2);
    assert_eq!(sources.len(), 1, "expected 1 surviving source");
    assert!((sources[0].age - 0.2).abs() < 1e-10);

    // Ring radius should have expanded: 0 + 200 * 0.2 = 40
    match &sources[0].kind {
        FieldSourceKind::ShockwaveRing { ring_radius, .. } => {
            assert!(
                (*ring_radius - 40.0).abs() < 1e-10,
                "expected ring_radius=40, got {}",
                ring_radius
            );
        }
    }
}

#[test]
fn test_toroidal_wrap_nearest_source() {
    // Source at (10, 10), entity at (phys_w - 10, 10). With wrapping, nearest distance = 20.
    // Ring radius = 20, width = 40 → entity is on the ring.
    let source = FieldSource {
        kind: FieldSourceKind::ShockwaveRing {
            impulse: 10.0,
            ring_radius: 20.0,
            ring_speed: 200.0,
            ring_width: 40.0,
            strength_decay: f64::INFINITY,
        },
        position: Vec2::new(10.0, 10.0),
        age: 0.0,
        max_age: 5.0,
    };
    let phys_w = 1000.0;
    let phys_h = 1000.0;
    let entity_pos = Vec2::new(phys_w - 10.0, 10.0);
    let sample = evaluate_field(entity_pos, &[source], phys_w, phys_h);
    // Wrapped distance: entity is 20 units to the LEFT of source via wrap.
    // Direction should be -x (push away from source, which is to the right via wrap).
    assert!(
        sample.wind.x < -0.1,
        "expected negative x push via wrap, got {}",
        sample.wind.x
    );
}
```

- [ ] Run and confirm failures:
```bash
rtk cargo test --test field_tests
```

Expected: compilation errors (functions don't exist yet).

#### Step 2.2: Implement evaluate_field and advance_sources

- [ ] Add to `src/field.rs` after the type definitions:

```rust
// ============================================================================
// Field evaluation
// ============================================================================

/// Compute the nearest wrapped displacement from `source` to `entity` in toroidal space.
/// Returns the vector from source to entity (shortest path via wrapping).
/// `phys_w`/`phys_h` of 0.0 disables wrapping on that axis.
fn nearest_displacement(entity: Vec2, source: Vec2, phys_w: f64, phys_h: f64) -> Vec2 {
    let mut dx = entity.x - source.x;
    let mut dy = entity.y - source.y;
    if phys_w > 0.0 {
        let half_w = phys_w * 1.5; // 3x world: total range is 3*phys_w, half is 1.5*phys_w
        let full_w = phys_w * 3.0;
        if dx > half_w {
            dx -= full_w;
        } else if dx < -half_w {
            dx += full_w;
        }
    }
    if phys_h > 0.0 {
        let half_h = phys_h * 1.5;
        let full_h = phys_h * 3.0;
        if dy > half_h {
            dy -= full_h;
        } else if dy < -half_h {
            dy += full_h;
        }
    }
    Vec2::new(dx, dy)
}

/// Evaluate all field sources at a single position.
///
/// `phys_w`/`phys_h`: toroidal world dimensions (0.0 to disable wrapping).
/// Returns accumulated FieldSample (wind + gravity + time_dilation).
pub fn evaluate_field(
    position: Vec2,
    sources: &[FieldSource],
    phys_w: f64,
    phys_h: f64,
) -> FieldSample {
    let mut sample = FieldSample::default();

    for source in sources {
        let disp = nearest_displacement(position, source.position, phys_w, phys_h);
        let dist = (disp.x * disp.x + disp.y * disp.y).sqrt();

        match &source.kind {
            FieldSourceKind::ShockwaveRing {
                impulse,
                ring_radius,
                ring_width,
                strength_decay,
                ..
            } => {
                // Distance from entity to ring edge
                let ring_dist = (dist - ring_radius).abs();
                if ring_dist >= *ring_width {
                    continue; // Outside ring influence
                }
                if dist < 1e-6 {
                    continue; // At source center — skip to avoid division by zero
                }

                // Linear falloff from ring center
                let spatial_strength = 1.0 - ring_dist / ring_width;

                // Temporal decay: 2^(-age / half_life)
                let temporal_strength = if *strength_decay == f64::INFINITY
                    || *strength_decay <= 0.0
                {
                    1.0
                } else {
                    (2.0_f64).powf(-source.age / strength_decay)
                };

                let total_strength = impulse * spatial_strength * temporal_strength;

                // Direction: radially outward from source
                let direction = Vec2::new(disp.x / dist, disp.y / dist);
                sample.wind.x += direction.x * total_strength;
                sample.wind.y += direction.y * total_strength;
            }
        }
    }

    sample
}

// ============================================================================
// Source lifecycle
// ============================================================================

/// Advance all sources by `dt` seconds. Expand rings, increment age, remove expired.
pub fn advance_sources(sources: &mut Vec<FieldSource>, dt: f64) {
    for source in sources.iter_mut() {
        source.age += dt;

        // Expand rings
        match &mut source.kind {
            FieldSourceKind::ShockwaveRing {
                ring_radius,
                ring_speed,
                ..
            } => {
                *ring_radius += *ring_speed * dt;
            }
        }
    }

    // Remove expired sources
    sources.retain(|s| s.age < s.max_age);
}
```

#### Step 2.3: Verify tests pass

- [ ] Run:
```bash
rtk cargo test --test field_tests
```

Expected: all 7 tests pass.

- [ ] Run full suite:
```bash
rtk cargo check && rtk cargo clippy && rtk cargo test
```

Expected: all tests pass, no warnings.

```bash
rtk git add src/field.rs tests/field_tests.rs && rtk git commit -m "feat(field): evaluate_field and advance_sources with tests"
```

---

### Task 3: Add field_sources to GameState

**Goal:** Wire `field_sources: Vec<FieldSource>` into GameState and call `advance_sources` in the physics loop.

**Files:**
- Modify: `src/game.rs` — add field to GameState, call advance in update_game

#### Step 3.1: Add field_sources to GameState

- [ ] In `src/game.rs`, add the import at the top (with existing `use` statements):

```rust
use crate::field::{self, FieldSource};
```

- [ ] In the `GameState` struct (line 70-105), add after the `sparks` field (line 94):

```rust
    pub field_sources: Vec<FieldSource>,
```

- [ ] In every `GameState` construction site (find all `GameState {` initializers — likely in `init_game` or similar), add:

```rust
    field_sources: Vec::new(),
```

#### Step 3.2: Call advance_sources in update_game

- [ ] In `update_game()`, after the explosion shockwave push block (after line ~1133) and before the projectile damage section, add:

```rust
    // === Field source lifecycle ===
    let field_dt = globals.dt() * globals.time.game_speed;
    field::advance_sources(&mut state.field_sources, field_dt);
```

#### Step 3.3: Verify and commit

- [ ] Run:
```bash
rtk cargo check && rtk cargo clippy && rtk cargo test
```

Expected: compiles, all tests pass. No behavioral change — field_sources is always empty.

```bash
rtk git add src/game.rs && rtk git commit -m "feat(field): add field_sources to GameState with advance lifecycle"
```

---

## Phase 1: Replace Shockwave with Expanding Ring (Tasks 4-6)

### Task 4: Add Field Constants to parameters.rs

**Goal:** Define tuning constants for the ShockwaveRing field source.

**Files:**
- Modify: `src/parameters.rs`

#### Step 4.1: Add field constants

- [ ] In `src/parameters.rs`, after the existing SHOCKWAVE constants (after line 312), add:

```rust
// --- Field system: ShockwaveRing ---
/// Speed at which the shockwave ring expands (units/s).
pub const FIELD_RING_SPEED: f64 = 400.0;

/// Width of the ring's influence zone (units). Entities within ring_radius ± ring_width are affected.
pub const FIELD_RING_WIDTH: f64 = 60.0;

/// Half-life of ring strength decay (seconds). After this time, strength is halved.
pub const FIELD_RING_DECAY: f64 = 0.8;

/// Maximum lifetime of a shockwave ring source (seconds).
pub const FIELD_RING_MAX_AGE: f64 = 3.0;

/// Impulse scale for field-based shockwave (applied to explosion mass).
/// Replaces SHOCKWAVE_IMPULSE_SCALE for field-based push.
pub const FIELD_RING_IMPULSE_SCALE: f64 = 0.5;

/// Fixed push impulse for particles (smoke, sparks) from field ring.
/// Replaces SHOCKWAVE_PARTICLE_PUSH.
pub const FIELD_RING_PARTICLE_PUSH: f64 = 0.3;
```

#### Step 4.2: Verify and commit

- [ ] Run:
```bash
rtk cargo check
```

```bash
rtk git add src/parameters.rs && rtk git commit -m "feat(field): add ShockwaveRing tuning constants"
```

---

### Task 5: Spawn ShockwaveRing on Explosion

**Goal:** When explosions are created, also spawn a ShockwaveRing field source at the same position.

**Files:**
- Modify: `src/game.rs` — spawn source when explosions are spawned

#### Step 5.1: Identify explosion spawn sites and add field source spawning

The explosion entities are pushed to `state.explosions` in the physics loop. Search for `explosions.push(` in `game.rs` to find all spawn sites. For each one, immediately after the push, also push a FieldSource.

- [ ] Add the import for field constants in `src/game.rs` (if not already present from Task 3):

```rust
use crate::field::FieldSourceKind;
```

- [ ] At each site where an explosion entity is pushed to `state.explosions`, add immediately after. Use the explosion entity's `mass` and `position` fields to parameterize the source:

```rust
state.field_sources.push(FieldSource {
    kind: FieldSourceKind::ShockwaveRing {
        impulse: explo_entity.mass * FIELD_RING_IMPULSE_SCALE,
        ring_radius: 0.0,
        ring_speed: FIELD_RING_SPEED,
        ring_width: FIELD_RING_WIDTH,
        strength_decay: FIELD_RING_DECAY,
    },
    position: explo_entity.position,
    age: 0.0,
    max_age: FIELD_RING_MAX_AGE,
});
```

Where `explo_entity` is the explosion entity that was just pushed. The exact variable name depends on the spawn site — read each site to get the correct reference.

**Important:** There are multiple explosion spawn functions (`spawn_explosion`, `spawn_explosion_object`, `spawn_explosion_death`). The field source should be spawned at the call site where the explosion entity is added to `state.explosions`, not inside the spawn functions themselves (since the spawn functions return an Entity, they don't have access to `state.field_sources`).

#### Step 5.2: Verify and commit

- [ ] Run:
```bash
rtk cargo check && rtk cargo clippy && rtk cargo test
```

Expected: compiles. Field sources are now spawned but not yet evaluated for push — no behavioral change on entities yet (the `advance_sources` call just ages and removes them).

```bash
rtk git add src/game.rs && rtk git commit -m "feat(field): spawn ShockwaveRing sources on explosion"
```

---

### Task 6: Apply Field Wind and Remove Old Shockwave

**Goal:** Evaluate field sources at each entity position, apply wind to velocity. Then remove the old hardcoded shockwave push loop.

**Files:**
- Modify: `src/game.rs` — add field evaluation, remove old shockwave code
- Modify: `tests/field_tests.rs` — add integration-style test

#### Step 6.1: Write test for ring expansion behavior

- [ ] Add to `tests/field_tests.rs`:

```rust
#[test]
fn test_field_ring_expands_over_time() {
    // Ring starts at radius 0, speed 200. After 0.5s → radius 100.
    // Entity at (100, 0) should be on the ring.
    let mut sources = vec![FieldSource {
        kind: FieldSourceKind::ShockwaveRing {
            impulse: 10.0,
            ring_radius: 0.0,
            ring_speed: 200.0,
            ring_width: 20.0,
            strength_decay: f64::INFINITY,
        },
        position: Vec2::new(0.0, 0.0),
        age: 0.0,
        max_age: 5.0,
    }];

    // Initially, entity at (100, 0) is outside the ring (radius=0, width=20 → affects [0, 20])
    let sample_before = evaluate_field(Vec2::new(100.0, 0.0), &sources, 0.0, 0.0);
    assert_eq!(
        sample_before.wind.x, 0.0,
        "should not be affected before ring reaches"
    );

    // Advance 0.5s → ring_radius = 100
    advance_sources(&mut sources, 0.5);

    // Now entity is exactly on the ring
    let sample_after = evaluate_field(Vec2::new(100.0, 0.0), &sources, 0.0, 0.0);
    assert!(
        sample_after.wind.x > 0.0,
        "should be pushed after ring reaches entity"
    );
}
```

- [ ] Run and confirm it passes:
```bash
rtk cargo test --test field_tests test_field_ring_expands_over_time
```

#### Step 6.2: Add field evaluation to update_game

- [ ] In `src/game.rs`, in `update_game()`, **after** the inertia block (after line 932 — `apply_inertia_all(&mut state.chunks_explo, globals);`) and **before** the rotation block (`// --- Rotation (moment update) ---`), add:

```rust
    // === Field evaluation (wind push) ===
    if !state.field_sources.is_empty() {
        let phys_w = globals.render.phys_width;
        let phys_h = globals.render.phys_height;

        // Physics objects: Newtonian impulse / mass (no dt scaling — impulse is per-frame)
        for entity in std::iter::once(&mut state.ship)
            .chain(state.objects.iter_mut())
            .chain(state.objects_oos.iter_mut())
            .chain(state.toosmall.iter_mut())
            .chain(state.toosmall_oos.iter_mut())
            .chain(state.fragments.iter_mut())
        {
            let sample =
                field::evaluate_field(entity.position, &state.field_sources, phys_w, phys_h);
            if sample.wind.x != 0.0 || sample.wind.y != 0.0 {
                let inv_mass = if entity.mass > 1e-6 {
                    1.0 / entity.mass
                } else {
                    1.0
                };
                entity.velocity.x += sample.wind.x * inv_mass;
                entity.velocity.y += sample.wind.y * inv_mass;
            }
        }

        // Particles (fixed impulse, no mass division)
        for particle in state
            .chunks
            .iter_mut()
            .chain(state.chunks_oos.iter_mut())
            .chain(state.chunks_explo.iter_mut())
            .chain(state.smoke.iter_mut())
            .chain(state.smoke_oos.iter_mut())
            .chain(state.sparks.iter_mut())
        {
            let sample =
                field::evaluate_field(particle.position, &state.field_sources, phys_w, phys_h);
            if sample.wind.x != 0.0 || sample.wind.y != 0.0 {
                particle.velocity.x += sample.wind.x * FIELD_RING_PARTICLE_PUSH;
                particle.velocity.y += sample.wind.y * FIELD_RING_PARTICLE_PUSH;
            }
        }
    }
```

**Design note:** The field returns impulse (not force), so physics objects divide by mass. Particles get a fixed fraction via `FIELD_RING_PARTICLE_PUSH`. This matches the old shockwave behavior. The expanding ring applies push each frame the entity is within the ring width — this is a continuous sweep rather than one-shot, so `FIELD_RING_IMPULSE_SCALE` may need tuning downward vs the old `SHOCKWAVE_IMPULSE_SCALE`.

#### Step 6.3: Remove old shockwave push code

- [ ] In `src/game.rs`, find the block starting with `// === Explosion shockwave push ===` (line ~1069) and ending after the particle push loop closing brace (line ~1133). Delete the entire block and replace with:

```rust
    // Shockwave push is now handled by the field system (see field evaluation above).
```

#### Step 6.4: Verify and commit

- [ ] Run:
```bash
rtk cargo check && rtk cargo clippy && rtk cargo test
```

Expected: compiles, all tests pass.

- [ ] **Visual test**: Run the game, trigger explosions, verify nearby asteroids are pushed away by the expanding ring. The feel should be similar to before — debris pushed outward, with the new expanding ring making the push propagate outward over time instead of being instantaneous.

```bash
rtk git add src/game.rs tests/field_tests.rs && rtk git commit -m "feat(field): apply ShockwaveRing wind, remove old one-shot shockwave push"
```

---

## Phase 2: Time Dilation from Explosions (Tasks 7-9)

### Task 7: Add Time Dilation to ShockwaveRing Evaluation

**Goal:** ShockwaveRing sources also produce a time dilation effect — entities near the ring experience slowed time (increased `proper_time`).

**Files:**
- Modify: `src/field.rs` — add time dilation to ShockwaveRing evaluation
- Modify: `src/parameters.rs` — add time dilation constant
- Modify: `tests/field_tests.rs` — add tests

#### Step 7.1: Add constant

- [ ] In `src/parameters.rs`, after the existing field ring constants:

```rust
/// Time dilation strength for shockwave ring. At ring center, proper_time multiplier increases by this amount.
/// 0.5 means proper_time becomes 1.5 at full strength → entities experience time at 2/3 speed.
pub const FIELD_RING_TIME_DILATION: f64 = 0.5;
```

#### Step 7.2: Write failing test

- [ ] Add to `tests/field_tests.rs`:

```rust
#[test]
fn test_shockwave_ring_time_dilation() {
    let source = FieldSource {
        kind: FieldSourceKind::ShockwaveRing {
            impulse: 10.0,
            ring_radius: 100.0,
            ring_speed: 200.0,
            ring_width: 20.0,
            strength_decay: f64::INFINITY,
        },
        position: Vec2::new(0.0, 0.0),
        age: 0.0,
        max_age: 5.0,
    };
    // Entity on ring center → full time dilation
    let sample = evaluate_field(Vec2::new(100.0, 0.0), &[source.clone()], 0.0, 0.0);
    assert!(
        sample.time_dilation > 1.0,
        "expected time dilation > 1.0, got {}",
        sample.time_dilation
    );

    // Entity outside ring → no time dilation (1.0)
    let sample_outside = evaluate_field(Vec2::new(200.0, 0.0), &[source], 0.0, 0.0);
    assert_eq!(
        sample_outside.time_dilation, 1.0,
        "expected no dilation outside ring"
    );
}

#[test]
fn test_shockwave_ring_time_dilation_falloff() {
    let source = FieldSource {
        kind: FieldSourceKind::ShockwaveRing {
            impulse: 10.0,
            ring_radius: 100.0,
            ring_speed: 200.0,
            ring_width: 20.0,
            strength_decay: f64::INFINITY,
        },
        position: Vec2::new(0.0, 0.0),
        age: 0.0,
        max_age: 5.0,
    };
    // At ring center (dist=100, ring_radius=100): spatial_strength = 1.0
    let on_ring = evaluate_field(Vec2::new(100.0, 0.0), &[source.clone()], 0.0, 0.0);
    // At ring half-edge (dist=110, ring_radius=100): spatial_strength = 0.5
    let half_edge = evaluate_field(Vec2::new(110.0, 0.0), &[source], 0.0, 0.0);

    // Dilation at half-edge should be between 1.0 and on_ring dilation
    assert!(
        half_edge.time_dilation > 1.0 && half_edge.time_dilation < on_ring.time_dilation,
        "expected intermediate dilation at half-edge: {} (on_ring={})",
        half_edge.time_dilation,
        on_ring.time_dilation
    );
}
```

- [ ] Run, confirm fail:
```bash
rtk cargo test --test field_tests test_shockwave_ring_time_dilation
```

#### Step 7.3: Implement time dilation in evaluate_field

- [ ] In `src/field.rs`, inside the `ShockwaveRing` match arm in `evaluate_field()`, after the wind accumulation lines (`sample.wind.y += ...`), add:

```rust
                // Time dilation: additive contribution on top of base 1.0
                // spatial_strength * temporal_strength gives [0, 1], scaled by FIELD_RING_TIME_DILATION
                let dilation_contribution =
                    crate::parameters::FIELD_RING_TIME_DILATION * spatial_strength * temporal_strength;
                sample.time_dilation += dilation_contribution;
```

This means `time_dilation` defaults to 1.0, and each source adds up to `FIELD_RING_TIME_DILATION` (0.5) at full strength. Multiple overlapping sources stack additively.

#### Step 7.4: Verify and commit

- [ ] Run:
```bash
rtk cargo test --test field_tests && rtk cargo check && rtk cargo clippy
```

```bash
rtk git add src/field.rs src/parameters.rs tests/field_tests.rs && rtk git commit -m "feat(field): add time dilation to ShockwaveRing evaluation"
```

---

### Task 8: Apply Time Dilation to Entities

**Goal:** Use the `time_dilation` field from `FieldSample` to modulate each entity's `proper_time`.

**Files:**
- Modify: `src/game.rs` — apply time_dilation in the field evaluation block

#### Step 8.1: Reset proper_time before field modulation

The field system needs to set `proper_time` each frame without drift. Strategy: reset to base at start of field pass, then multiply by sample's `time_dilation`.

- [ ] In `src/game.rs`, in the field evaluation block (from Task 6), **before** the physics object loop, add:

```rust
        // Reset proper_time to base before field modulation
        let base_pt = OBSERVER_PROPER_TIME;
        state.ship.proper_time = base_pt;
        for e in state.objects.iter_mut()
            .chain(state.objects_oos.iter_mut())
            .chain(state.toosmall.iter_mut())
            .chain(state.toosmall_oos.iter_mut())
            .chain(state.fragments.iter_mut())
            .chain(state.chunks.iter_mut())
            .chain(state.chunks_oos.iter_mut())
            .chain(state.chunks_explo.iter_mut())
            .chain(state.smoke.iter_mut())
            .chain(state.smoke_oos.iter_mut())
            .chain(state.sparks.iter_mut())
        {
            e.proper_time = base_pt;
        }
```

#### Step 8.2: Apply time dilation from field samples

- [ ] In the physics object loop, after the wind application, add:

```rust
            if sample.time_dilation != 1.0 {
                entity.proper_time *= sample.time_dilation;
            }
```

- [ ] In the particle loop, after the wind application, add:

```rust
            if sample.time_dilation != 1.0 {
                particle.proper_time *= sample.time_dilation;
            }
```

#### Step 8.3: Update observer_proper_time after field pass

- [ ] After the field evaluation block's closing brace, add:

```rust
    // Update observer proper time after field modulation
    globals.observer_proper_time = state.ship.proper_time;
```

This ensures the rest of the frame (smoke decay, chunk decay, etc.) uses the field-modulated proper_time. The existing assignment at line 755 (`globals.observer_proper_time = state.ship.proper_time;`) runs before inertia and can stay — it captures the pre-field value, and this post-field update overrides it.

#### Step 8.4: Handle the no-field-sources case

- [ ] The existing code in Task 6 wraps the field eval in `if !state.field_sources.is_empty()`. When there are no sources, proper_time stays at whatever it was set to at spawn (1.0). This is correct — no reset needed when no sources exist.

However, if sources existed last frame but were all removed this frame, proper_time would be stuck at last frame's value. Fix: move the proper_time reset **outside** the `if !state.field_sources.is_empty()` guard, so it always resets to base. Or better: only reset when field_sources is non-empty OR was non-empty last frame. Simplest: always reset. The cost is negligible.

- [ ] Move the proper_time reset block to **before** the `if !state.field_sources.is_empty()` check:

```rust
    // Reset proper_time to base (recomputed each frame by field system)
    {
        let base_pt = OBSERVER_PROPER_TIME;
        state.ship.proper_time = base_pt;
        for e in state.objects.iter_mut()
            .chain(state.objects_oos.iter_mut())
            .chain(state.toosmall.iter_mut())
            .chain(state.toosmall_oos.iter_mut())
            .chain(state.fragments.iter_mut())
            .chain(state.chunks.iter_mut())
            .chain(state.chunks_oos.iter_mut())
            .chain(state.chunks_explo.iter_mut())
            .chain(state.smoke.iter_mut())
            .chain(state.smoke_oos.iter_mut())
            .chain(state.sparks.iter_mut())
        {
            e.proper_time = base_pt;
        }
    }

    // === Field evaluation (wind push + time dilation) ===
    if !state.field_sources.is_empty() {
        // ... (existing field eval code with time dilation additions)
    }

    // Update observer proper time after field modulation
    globals.observer_proper_time = state.ship.proper_time;
```

#### Step 8.5: Verify and commit

- [ ] Run:
```bash
rtk cargo check && rtk cargo clippy && rtk cargo test
```

Expected: compiles, all tests pass.

- [ ] **Visual test**: Run the game, trigger explosions near asteroids. Objects near the expanding ring should briefly slow down (their proper_time increases, making their movement factor smaller via `dt * game_speed * OBSERVER_PROPER_TIME / entity.proper_time`). The effect should be subtle with `FIELD_RING_TIME_DILATION = 0.5`.

```bash
rtk git add src/game.rs && rtk git commit -m "feat(field): apply time dilation from field sources to proper_time"
```

---

### Task 9: Determinism Test for Field System

**Goal:** Add a scenario-based determinism test that verifies field-based shockwave produces identical results across runs.

**Files:**
- Modify: `tests/scenario_tests.rs` — add field determinism test

#### Step 9.1: Examine existing scenario test pattern

- [ ] Read `tests/scenario_tests.rs` to understand how existing determinism tests are structured (init_game, update loop, seed, assertion pattern). Match that pattern exactly.

#### Step 9.2: Write determinism test

- [ ] Add to `tests/scenario_tests.rs` a test that:
  1. Runs two identical simulations with the same seed for 120 frames (2s at 60fps)
  2. Compares final positions and velocities of all surviving objects
  3. Also compares `field_sources` count (both should have same number of active sources)

```rust
#[test]
fn test_field_shockwave_determinism() {
    // Run two identical simulations and verify same final state.
    // The specific init and update functions depend on the existing test pattern —
    // read scenario_tests.rs first and match the existing setup exactly.

    let seed = 42u64;
    let frames = 120;

    let run_simulation = |s: u64| {
        // Match existing test pattern for init + globals setup
        let mut globals = /* ... existing pattern ... */;
        let mut state = /* ... init_game with seed s ... */;

        for _ in 0..frames {
            update_game(&mut state, &mut globals);
        }

        // Collect deterministic snapshot
        let positions: Vec<(f64, f64)> =
            state.objects.iter().map(|e| (e.position.x, e.position.y)).collect();
        let velocities: Vec<(f64, f64)> =
            state.objects.iter().map(|e| (e.velocity.x, e.velocity.y)).collect();
        let field_count = state.field_sources.len();
        (positions, velocities, field_count)
    };

    let (pos_a, vel_a, fc_a) = run_simulation(seed);
    let (pos_b, vel_b, fc_b) = run_simulation(seed);

    assert_eq!(pos_a.len(), pos_b.len(), "different object counts");
    assert_eq!(fc_a, fc_b, "different field source counts");
    for (i, ((pa, pb), (va, vb))) in pos_a.iter().zip(&pos_b).zip(vel_a.iter().zip(&vel_b)).enumerate() {
        assert_eq!(pa, pb, "object {} position diverged", i);
        assert_eq!(va, vb, "object {} velocity diverged", i);
    }
}
```

**Note:** The pseudo-code above must be adapted to match the actual test infrastructure in `tests/scenario_tests.rs`. Read the existing tests first to get the correct `Globals` construction, `init_game` call pattern, and any required setup.

#### Step 9.3: Verify and commit

- [ ] Run:
```bash
rtk cargo test test_field_shockwave_determinism
```

Expected: passes — field system is deterministic (sources iterated in index order, no random choices in evaluation, SmallRng for spawning).

```bash
rtk git add tests/scenario_tests.rs && rtk git commit -m "test(field): determinism test for field-based shockwave"
```

---

## Task 10: Cleanup — Remove Dead Constants

**Goal:** Remove the old shockwave constants that are no longer used.

**Files:**
- Modify: `src/parameters.rs`

#### Step 10.1: Check if old constants are still referenced

- [ ] Search for uses of `SHOCKWAVE_IMPULSE_SCALE`, `SHOCKWAVE_RANGE_MULTIPLIER`, `SHOCKWAVE_PARTICLE_PUSH` across the codebase (excluding `parameters.rs` itself). If they are only defined but never used, remove them.

#### Step 10.2: Remove dead constants

- [ ] If no remaining references, delete from `src/parameters.rs`:

```rust
// DELETE these three constants:
pub const SHOCKWAVE_IMPULSE_SCALE: f64 = 0.5;
pub const SHOCKWAVE_RANGE_MULTIPLIER: f64 = 1.5;
pub const SHOCKWAVE_PARTICLE_PUSH: f64 = 0.3;
```

#### Step 10.3: Verify and commit

- [ ] Run:
```bash
rtk cargo check && rtk cargo clippy && rtk cargo test
```

```bash
rtk git add src/parameters.rs && rtk git commit -m "refactor: remove obsolete SHOCKWAVE constants (replaced by FIELD_RING_*)"
```

---

## Summary

| Task | Description | Est. time | Phase |
|------|-------------|-----------|-------|
| 1 | FieldSource/FieldSample types + module | 3 min | 0: Infrastructure |
| 2 | evaluate_field + advance_sources + tests | 5 min | 0: Infrastructure |
| 3 | field_sources in GameState + lifecycle call | 3 min | 0: Infrastructure |
| 4 | Field ring constants in parameters.rs | 2 min | 1: Expanding ring |
| 5 | Spawn ShockwaveRing on explosion | 4 min | 1: Expanding ring |
| 6 | Apply field wind + remove old shockwave | 5 min | 1: Expanding ring |
| 7 | Time dilation in ShockwaveRing evaluation | 4 min | 2: Time dilation |
| 8 | Apply time dilation to entity proper_time | 4 min | 2: Time dilation |
| 9 | Determinism test | 3 min | 2: Time dilation |
| 10 | Remove old SHOCKWAVE constants | 2 min | Cleanup |

**Total: ~35 min**

## Future Plans (Separate Documents)

- **Gravity wells / black holes** — GravityWell variant in FieldSourceKind, attraction + time dilation, new gameplay mechanic. Requires separate design iteration for balance.
- **Vortex / WindZone** — Additional field types, separate plan when needed.
- **GPU visualization** — Post-process shader reads source list, renders flow lines, gravitational lensing, time-dilation haze. Separate rendering plan.
