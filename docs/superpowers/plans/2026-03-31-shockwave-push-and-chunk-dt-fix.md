# Explosion Shockwave Push — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explosion shockwave velocity impulse that pushes nearby objects away from blast center, plus precompute `avg_radius` on Hitbox for cheap closest-point distance.

**Architecture:** Extend the existing explosion damage loop in `game.rs` with a second pass that applies a one-shot velocity impulse. Linear falloff within 1.5× blast radius, scaled by explosion mass and divided by target mass (Newtonian). Add `avg_radius` to `Hitbox` for closest-point approximation.

**Tech Stack:** Rust, existing Vec2 math

**Design spec:** `docs/superpowers/specs/2026-03-31-shockwave-push-and-chunk-dt-fix-design.md`

**Note on chunk damage dt-independence:** Verified as already correct. The explosion damage loop uses `dt * 60` scaling, and chunks_explo lifetime is dt-scaled via radius decay. Total damage over a chunk's lifetime is framerate-independent. Will be removed from backlog in Task 3.

---

## File Structure

| File | Role |
|------|------|
| `src/objects.rs` | Add `avg_radius` to `Hitbox`, update all 10 spawn sites |
| `src/parameters.rs` | Add `SHOCKWAVE_IMPULSE_SCALE` and `SHOCKWAVE_RANGE_MULTIPLIER` constants |
| `src/game.rs` | Shockwave push logic in explosion damage loop |
| `tests/scenario_tests.rs` | Determinism test covering shockwave behavior |

---

## Task 1: Add `avg_radius` to Hitbox

**Goal:** Precompute average radius on all Hitbox constructions.

**Files:**
- Modify: `src/objects.rs:30-34` — Hitbox struct
- Modify: `src/objects.rs` — 10 spawn sites (lines 154, 188, 254, 309, 390, 427, 489, 533, 562, 631)

### Step 1.1: Add field to Hitbox struct

- [ ] In `src/objects.rs`, add `avg_radius` to the `Hitbox` struct:

```rust
#[derive(Clone, Debug, serde::Serialize)]
pub struct Hitbox {
    pub ext_radius: f64,
    pub int_radius: f64,
    pub avg_radius: f64,
    pub points: Polygon,
}
```

### Step 1.2: Update all Hitbox construction sites

- [ ] For every `Hitbox { ext_radius, int_radius, points }` in `src/objects.rs`, add the `avg_radius` field. The pattern is always:

```rust
hitbox: Hitbox {
    ext_radius: <ext>,
    int_radius: <int>,
    avg_radius: (<ext> + <int>) / 2.0,
    points: <points>,
},
```

There are 10 sites to update. For each, read the existing `ext_radius` and `int_radius` expressions, then compute the average. Many sites use `ext_radius: rad` and `int_radius: rad` (same value), so `avg_radius: rad` works for those.

Specific sites in `src/objects.rs`:

1. **Line 154** — `spawn_ship`: `ext_radius: SHIP_RADIUS, int_radius: SHIP_RADIUS` → `avg_radius: SHIP_RADIUS`
2. **Line 188** — `spawn_smoke`: `ext_radius: rad, int_radius: rad` → `avg_radius: rad`
3. **Line 254** — `spawn_projectile`: `ext_radius: PROJECTILE_RADIUS, int_radius: PROJECTILE_RADIUS` → `avg_radius: PROJECTILE_RADIUS`
4. **Line 309** — `spawn_explosion` (from projectile): `ext_radius: rad, int_radius: rad` → `avg_radius: rad`
5. **Line 390** — `spawn_explosion_object`: read the two values and compute average
6. **Line 427** — `spawn_explosion_death`: `ext_radius: rad, int_radius: rad` → `avg_radius: rad`
7. **Line 489** — `spawn_chunk_explosion`: read the two values and compute average
8. **Line 533** — `spawn_chunk`: `ext_radius: rad, int_radius: rad` → `avg_radius: rad`
9. **Line 562** — `spawn_spark`: `ext_radius: rad, int_radius: rad` → `avg_radius: rad`
10. **Line 631** — `spawn_asteroid`: has different ext/int radii — compute `avg_radius: (ext_radius + int_radius) / 2.0` using the same expressions

Note: Read each site carefully. Some compute ext/int from variables, so you need to replicate the expressions correctly.

### Step 1.3: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Run `cargo test` — all existing tests should pass.

```bash
git add src/objects.rs
git commit -m "refactor: add avg_radius to Hitbox for cheap closest-point"
```

---

## Task 2: Shockwave Push

**Goal:** Add velocity impulse from explosions to nearby objects.

**Files:**
- Modify: `src/parameters.rs` — add 2 constants
- Modify: `src/game.rs:1047-1067` — shockwave logic after explosion damage

### Step 2.1: Add constants to parameters.rs

- [ ] Add after the explosion damage constants (around line 305):

```rust
/// Scaling factor for shockwave velocity impulse. Higher = stronger push.
pub const SHOCKWAVE_IMPULSE_SCALE: f64 = 0.5;

/// Blast range as a multiplier of explosion ext_radius.
pub const SHOCKWAVE_RANGE_MULTIPLIER: f64 = 1.5;
```

### Step 2.2: Implement shockwave push

- [ ] In `src/game.rs`, after the explosion damage loop (after line 1067), add the shockwave push loop:

```rust
    // === Explosion shockwave push ===
    // One-shot velocity impulse: no dt scaling needed (explosions are one-frame).
    for explo in &state.explosions {
        let explo_pos = explo.position;
        let blast_range = explo.hitbox.ext_radius * SHOCKWAVE_RANGE_MULTIPLIER;
        let explo_impulse = explo.mass * SHOCKWAVE_IMPULSE_SCALE;

        for obj in state
            .objects
            .iter_mut()
            .chain(state.objects_oos.iter_mut())
            .chain(state.toosmall.iter_mut())
            .chain(state.toosmall_oos.iter_mut())
            .chain(state.chunks.iter_mut())
            .chain(state.chunks_oos.iter_mut())
        {
            let diff = obj.position - explo_pos;
            let center_dist = diff.length();
            if center_dist < 1e-6 {
                continue; // Skip objects exactly at explosion center (avoid NaN)
            }
            let effective_dist = (center_dist - obj.hitbox.avg_radius).max(0.0);
            if effective_dist >= blast_range {
                continue; // Out of range
            }
            let strength = 1.0 - (effective_dist / blast_range);
            let impulse = explo_impulse * strength / obj.mass;
            let direction = diff * (1.0 / center_dist); // normalized
            obj.velocity = obj.velocity + direction * impulse;
        }
    }
```

### Step 2.3: Add import for shockwave constants

- [ ] At the top of `src/game.rs` or in the existing `use crate::parameters::*;` — verify `SHOCKWAVE_IMPULSE_SCALE` and `SHOCKWAVE_RANGE_MULTIPLIER` are accessible (they should be via the wildcard import).

### Step 2.4: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Run `cargo test` — all tests should pass (shockwave is additive, doesn't break existing behavior).

```bash
git add src/parameters.rs src/game.rs
git commit -m "feat: explosion shockwave push with linear falloff"
```

---

## Task 3: Determinism Test + Backlog Cleanup

**Goal:** Add a scenario test that exercises shockwave, verify determinism. Remove chunk-dt item from backlog.

**Files:**
- Modify: `tests/scenario_tests.rs` — new test
- Modify: `BACKLOG.md` — remove chunk-dt item, add shockwave as done
- Modify: `DONE.md` — add completed entries

### Step 3.1: Add shockwave determinism test

- [ ] Add to `tests/scenario_tests.rs`:

```rust
#[test]
fn test_shockwave_determinism() {
    // Spawn asteroid near ship, fire at it, verify explosion pushes second asteroid
    let run = || {
        Scenario::builder()
            .seed(42)
            .fps(60)
            .ship_at(400.0, 300.0)
            .spawn_asteroid((500.0, 300.0), 40.0) // Small, close — will get hit
            .spawn_asteroid((600.0, 300.0), 80.0) // Nearby — should get pushed
            .at_frame(1, Action::AimAt(0.0)) // Aim right
            .at_frame(2, Action::Fire)
            .snapshot_at(30)
            .snapshot_at(59)
            .run_until(60)
            .run()
    };

    let result_a = run();
    let result_b = run();

    assert_eq!(result_a.snapshots.len(), result_b.snapshots.len());
    for (a, b) in result_a.snapshots.iter().zip(result_b.snapshots.iter()) {
        assert_eq!(a.data, b.data, "Shockwave state diverged at frame {}", a.frame);
    }
}
```

### Step 3.2: Run tests

- [ ] Run `cargo test -- test_shockwave_determinism`.
- [ ] Run `cargo test` — full suite.

### Step 3.3: Update backlog

- [ ] Remove the "chunk explosion damage framerate-independence" entry from `BACKLOG.md` (verified as already correct).
- [ ] Remove the "explosion shockwave push" entry from `BACKLOG.md`.
- [ ] Add both to `DONE.md`:
  - `- [x] [physics] Explosion shockwave push — linear falloff velocity impulse (completed: 2026-03-31)`
  - `- [x] [physics] Chunk explosion damage framerate-independence — verified already correct (completed: 2026-03-31)`

### Step 3.4: Commit

```bash
git add tests/scenario_tests.rs BACKLOG.md DONE.md
git commit -m "test: shockwave determinism test + backlog cleanup"
```

---

## Summary

| Task | What | Files |
|------|------|-------|
| 1 | `avg_radius` on Hitbox | objects.rs |
| 2 | Shockwave push | parameters.rs, game.rs |
| 3 | Determinism test + backlog | scenario_tests.rs, BACKLOG.md, DONE.md |
