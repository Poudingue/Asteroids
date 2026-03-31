# Explosion Shockwave Push & Chunk Damage DT-Fix — Design Spec

**Date:** 2026-03-31
**Scope:** Two small physics improvements — explosion shockwave velocity impulse and chunk explosion damage framerate-independence verification/fix.

---

## 1. Hitbox: Add `avg_radius`

**What:** Add an `avg_radius` field to `Hitbox`, precomputed at spawn as `(int_radius + ext_radius) / 2.0`.

**Why:** Cheap closest-point approximation for shockwave push. Also useful for future distance-based calculations without paying for polygon queries.

**Where:** `src/objects.rs` — `Hitbox` struct + all spawn functions that construct `Hitbox { ... }`.

**Details:**
- New field: `pub avg_radius: f64`
- Computed at every `Hitbox` construction site: `avg_radius: (int_radius + ext_radius) / 2.0`
- Add `serde::Serialize` is already derived on `Hitbox` — field will serialize automatically.

---

## 2. Explosion Shockwave Push

**What:** Explosions apply a one-shot velocity impulse to nearby objects, pushing them away from the blast center.

**Where:** `src/game.rs` — same loop as explosion damage (after the damage application), plus a new constant in `src/parameters.rs`.

### Physics Model

- **Blast range:** `1.5 × explosion.hitbox.ext_radius`
- **Effective distance:** `max(0, center_distance - target.hitbox.avg_radius)` — closest-point-on-circle approximation
- **Falloff:** Linear within blast range: `strength = 1.0 - (effective_dist / blast_range)`, clamped to `[0.0, 1.0]`
- **Impulse magnitude:** `explosion.mass * strength * SHOCKWAVE_IMPULSE_SCALE`
- **Direction:** `(target.position - explosion.position).normalized()` — radially outward from blast center
- **Application:** `target.velocity += direction * impulse / target.mass`
  - Dividing by target mass gives Newtonian behavior: small chunks fly, big asteroids nudge
- **Edge case:** If `center_distance < 1e-6` (object sitting exactly on explosion center), skip or use a random direction to avoid NaN from normalization

### Constants

```rust
/// Scaling factor for shockwave velocity impulse.
/// Higher = stronger push. Tune to feel.
pub const SHOCKWAVE_IMPULSE_SCALE: f64 = 0.5;

/// Blast range as a multiplier of explosion ext_radius.
pub const SHOCKWAVE_RANGE_MULTIPLIER: f64 = 1.5;
```

### DT-Independence

Explosions are one-frame entities. The shockwave impulse is a single velocity edit (not a per-second force), so it is inherently framerate-independent — no dt scaling needed.

### Targets

Push applies to the same entity sets as explosion damage:
- `state.objects` + `state.objects_oos` (asteroids)
- `state.toosmall` + `state.toosmall_oos` (small fragments)
- `state.chunks` + `state.chunks_oos` (debris chunks — optional, for visual flair)

Does NOT push: ship, projectiles, other explosions, smoke, sparks.

---

## 3. Chunk Explosion Damage Framerate-Independence

**Status:** Needs verification. The explosion damage loop already has `explo_dt_scale = globals.dt() * 60.0`. The question is whether this fully covers the `chunks_explo` path.

### Analysis

`chunks_explo` entities:
1. Exist for multiple frames (shrink each frame until `radius <= 0`)
2. Each frame, they spawn new one-frame `Explosion` entities (`spawn_chunk_explosion`)
3. Those explosions enter the damage loop with `explo_dt_scale`

**If the damage loop dt-scaling is correct**, then:
- At 60fps: each explosion does `150 * (1/60) * 60 = 150` damage per frame
- At 120fps: each explosion does `150 * (1/120) * 60 = 75` damage per frame, but 2× frames → same total/second

**Potential issue:** The `chunks_explo` lifetime is dt-scaled (radius decay uses `dt`), so the number of frames a chunk_explo exists should also be framerate-independent. If both paths use dt correctly, the total damage over a chunk_explo's lifetime should be the same regardless of framerate.

**Action:** Verify both paths numerically. If already correct, remove from backlog. If there's a gap, apply the fix.

---

## 4. Implementation Order

1. Add `avg_radius` to `Hitbox` + update all spawn sites
2. Add shockwave constants to `parameters.rs`
3. Implement shockwave push in the explosion damage loop in `game.rs`
4. Verify chunk damage dt-independence (code audit + optional test)
5. Verify with `cargo check && cargo clippy && cargo test`
6. Commit

---

## 5. Testing

- Existing tests should pass unchanged (shockwave is additive behavior)
- Scenario builder can verify shockwave: spawn asteroid near explosion, check velocity changed
- Determinism tests already cover the explosion code path (same seed = same result)
