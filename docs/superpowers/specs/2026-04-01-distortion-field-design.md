# Velocity/Wind and Spatiotemporal Distortion Field System — Design Exploration

**Date:** 2026-04-01
**Scope:** Design exploration for a persistent spatial field system that replaces/extends the one-shot explosion shockwave push with a general-purpose mechanism for velocity fields (wind), gravity, and time dilation zones.

## 1. Recommendation: Analytical (SDF-like) Approach

Use analytical/mathematical field evaluation exclusively. The game's entity counts (~3500 max) and source counts (~100 max) make O(entities × sources) cost negligible (~0.1ms with early-out). No grid/quadtree needed.

**Why not quadtree?** Physics space is ~7300×4100 units — not large enough to justify grid overhead. Grid artifacts (discontinuities at cell boundaries) would be visible on slow-moving large asteroids. Adaptive quadtree adds significant code complexity for marginal benefit.

**Why analytical?** Infinite precision, zero memory, trivial determinism, maps directly onto existing shockwave code pattern. Each source is a mathematical function evaluated at entity positions.

## 2. Key Insight: proper_time Infrastructure

Every Entity already has `proper_time: f64`, and every movement function divides by it:
```
time_factor = dt * game_speed * OBSERVER_PROPER_TIME / entity.proper_time
```
Setting `proper_time = 2.0` makes an entity experience time at half speed. This has been dormant since the OCaml port — the field system activates it.

## 3. Data Structures

```rust
pub struct FieldSource {
    pub kind: FieldSourceKind,
    pub position: Vec2,
    pub age: f64,
    pub max_age: f64, // f64::INFINITY for persistent sources
}

pub enum FieldSourceKind {
    /// Expanding shockwave ring (replaces one-shot push)
    ShockwaveRing {
        impulse: f64,
        ring_radius: f64,
        ring_speed: f64,
        ring_width: f64,
        strength_decay: f64, // half-life
    },
    /// Persistent gravity well (black hole)
    GravityWell {
        strength: f64,
        max_radius: f64,
        time_dilation_strength: f64,
    },
    /// Spinning field
    Vortex {
        strength: f64,
        max_radius: f64,
        spin_direction: f64,
    },
    /// Directional wind zone
    WindZone {
        direction: Vec2,
        strength: f64,
        max_radius: f64,
    },
}

pub struct FieldSample {
    pub wind: Vec2,          // velocity offset
    pub time_dilation: f64,  // multiplier on proper_time (1.0 = normal)
    pub gravity: Vec2,       // gravitational acceleration
}
```

## 4. Evaluation

```rust
fn evaluate_field(position: Vec2, sources: &[FieldSource]) -> FieldSample
```

Per entity: iterate all sources, early-out on distance, accumulate contributions. Unified evaluation — one pass produces wind + gravity + time dilation.

### ShockwaveRing
Expanding pressure wave. At distance `d` from source, ring at radius `r`, width `w`:
- If `|d - r| > w`: no contribution
- Else: `strength = base * (1.0 - |d - r| / w)`, radially outward
- Each frame: `ring_radius += ring_speed * dt`

### GravityWell
- Gravity: `S / max(d², min_dist²)` toward source
- Time dilation: `1.0 + strength * (1.0 - d/R)²` — closer = slower time

## 5. Integration Points

- **Source creation**: Explosions spawn `ShockwaveRing` entries (alongside current explosion entities)
- **Field evaluation**: After `apply_inertia_all`, before collision detection (~game.rs:932)
- **Wind**: `entity.velocity += wind * dt * game_speed`
- **Gravity**: `entity.velocity += gravity * dt * game_speed` (uniform, like real gravity)
- **Time dilation**: `entity.proper_time = base * field_sample.time_dilation`
- **Source decay**: End of physics update, advance age, remove dead sources

## 6. Performance

| Scenario | Entities | Sources | Raw iterations | With early-out | Time |
|----------|----------|---------|---------------|----------------|------|
| Typical  | 1000     | 20      | 20,000        | ~4,000         | 0.03ms |
| Heavy    | 2000     | 50      | 100,000       | ~15,000        | 0.1ms  |
| Worst    | 3500     | 100     | 350,000       | ~50,000        | 0.5ms  |

Current shockwave already does ~900K iterations and runs fine. Field system is comparable or cheaper.

GPU evaluation not needed — CPU cost is negligible. GPU would add round-trip latency exceeding any savings.

## 7. Phased Implementation

### Phase 0: Infrastructure
- FieldSource, FieldSourceKind, FieldSample types
- `field_sources: Vec<FieldSource>` in GameState
- evaluate_field function, source aging/removal

### Phase 1: Replace shockwave with expanding ring
- Explosions spawn ShockwaveRing sources
- Field evaluation applies wind from rings
- Remove old hardcoded shockwave loop
- Tune ring_speed, ring_width, decay to match/improve current feel

### Phase 2: Time dilation from explosions
- ShockwaveRings carry time dilation effect
- Brief "bullet time" around explosions
- Activates the dormant proper_time infrastructure

### Phase 3: Persistent gravity wells (black holes)
- GravityWell sources with attraction + time dilation
- First new gameplay enabled by field system

### Phase 4: GPU visualization
- Post-process shader reads source list
- Flow lines, gravitational lensing, time-dilation haze
- Shader evaluates same analytical functions as CPU

## 8. Open Questions

1. **Particles in time dilation**: Should smoke freeze near a black hole? Recommendation: yes — striking visual.
2. **Ship self-knockback**: Should the ship be pushed by its own explosions? Currently excluded.
3. **Toroidal wrap**: Evaluate sources at nearest wrapped position (same as collision system).
4. **Determinism**: Iterate sources in index order (Vec guarantees this). Source spawning follows deterministic RNG.

## 9. Critical Files

- `src/game.rs` — physics loop, shockwave to replace, field evaluation point
- `src/objects.rs` — Entity with proper_time, potential home for FieldSource
- `src/parameters.rs` — field source default constants
- `src/math_utils.rs` — existing exp_decay, distance functions
- New: `src/field.rs` — field source types, evaluation, decay logic
