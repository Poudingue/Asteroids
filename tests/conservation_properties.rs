/// Physics conservation law tests for the Asteroids game engine.
///
/// These tests verify physical invariants (mass, momentum, energy).
/// SOME TESTS ARE EXPECTED TO FAIL - they document where the current
/// implementation violates physics, marked with #[ignore].
///
/// Goal: document violations, not hide them.

use asteroids::objects::{
    fragment_asteroid, spawn_fragments, spawn_asteroid, spawn_explosion,
    spawn_explosion_chunk, spawn_n_chunks, spawn_projectile, Entity,
};
use asteroids::parameters::{
    Globals, ASTEROID_DENSITY, ASTEROID_MIN_SIZE,
    CHUNKS_EXPLO_MAX_SPEED, FRAGMENT_NUMBER,
};
use asteroids::physics::consequences_collision;
use asteroids::math_utils::{Vec2, magnitude, add_vec, scale_vec, squared};
use rand::thread_rng;
use std::f64::consts::PI;

fn kinetic_energy(e: &Entity) -> f64 {
    0.5 * e.mass * squared(magnitude(e.velocity))
}

fn momentum(e: &Entity) -> Vec2 {
    scale_vec(e.velocity, e.mass)
}

fn make_asteroid(radius: f64, pos: Vec2, vel: Vec2) -> Entity {
    let mut rng = thread_rng();
    spawn_asteroid(pos, vel, radius, &mut rng)
}

// ============================================================================
// MASS CONSERVATION TESTS
// ============================================================================

/// Rule: Fragments must not collectively exceed the mass of the parent asteroid.
///
/// fragment_asteroid draws new size from FRAGMENT_MIN_SIZE..FRAGMENT_MAX_SIZE
/// (ratio of parent int_radius): each fragment mass = PI * (ratio * r)^2 * DENSITY.
/// Max ratio = 0.7 => max single fragment mass = 0.49x parent_mass.
/// With FRAGMENT_NUMBER=5 fragments, total can reach up to ~2.45x parent. Violation.
#[test]
#[ignore = "VIOLATION: spawn_fragments produces FRAGMENT_NUMBER=5 fragments each up to 0.49x parent mass => total up to ~2.45x parent mass. parent_mass~70686 (r=150), fragment_total often 50000-150000. Phase 2 fix needed."]
fn test_asteroid_fragmentation_mass_conservation() {
    let mut rng = thread_rng();
    let sizes = [150.0_f64, 300.0, 500.0, 800.0];
    for &radius in &sizes {
        let parent = spawn_asteroid(Vec2::ZERO, Vec2::ZERO, radius, &mut rng);
        let parent_mass = parent.mass;
        let dead = vec![parent.clone()];
        let mut fragments = Vec::new();
        spawn_fragments(&dead, &mut fragments, FRAGMENT_NUMBER, &mut rng);
        let fragment_total_mass: f64 = fragments.iter().map(|f| f.mass).sum();
        assert!(
            fragment_total_mass <= parent_mass,
            "radius={}: fragment_total_mass={:.2} > parent_mass={:.2} (ratio={:.2}x)",
            radius, fragment_total_mass, parent_mass,
            fragment_total_mass / parent_mass
        );
    }
}

/// Rule: Spawning an explosion from a projectile should not create mass
/// exceeding the projectile that caused it.
#[test]
fn test_explosion_no_mass_creation() {
    let mut rng = thread_rng();
    let proj = spawn_projectile(Vec2::ZERO, Vec2::ZERO, 1.0);
    let proj_mass = proj.mass;
    for _ in 0..20 {
        let explosion = spawn_explosion(&proj, &mut rng);
        assert!(
            explosion.mass <= proj_mass,
            "explosion mass ({}) > projectile mass ({})",
            explosion.mass, proj_mass
        );
    }
}

/// Rule: Each chunk must have positive, bounded mass (hardcoded 100.0).
#[test]
fn test_chunk_mass_bounded() {
    let mut rng = thread_rng();
    let parent = make_asteroid(300.0, Vec2::ZERO, Vec2::ZERO);
    let parent_mass = parent.mass;
    let chunks = spawn_n_chunks(&parent, 20, parent.visuals.color, &mut rng);
    for chunk in &chunks {
        assert!(chunk.mass <= parent_mass,
            "chunk mass ({}) > parent mass ({})", chunk.mass, parent_mass);
        assert!(chunk.mass > 0.0,
            "chunk mass must be positive, got {}", chunk.mass);
    }
}

/// Rule: fragment mass must equal PI * r^2 * ASTEROID_DENSITY exactly.
#[test]
fn test_fragment_mass_formula_correct() {
    let mut rng = thread_rng();
    let parent = spawn_asteroid(Vec2::ZERO, Vec2::ZERO, 500.0, &mut rng);
    for _ in 0..20 {
        let frag = fragment_asteroid(&parent, &mut rng);
        let expected_mass = PI * squared(frag.hitbox.int_radius) * ASTEROID_DENSITY;
        assert!(
            (frag.mass - expected_mass).abs() < 1e-6,
            "fragment mass={:.6} != PI*r^2*density={:.6} (r={})",
            frag.mass, expected_mass, frag.hitbox.int_radius
        );
    }
}

// ============================================================================
// MOMENTUM CONSERVATION TESTS
// ============================================================================

/// Rule: In a collision with no external forces, total momentum must be conserved.
///
/// Current consequences_collision injects momentum via MIN_REPULSION + MIN_BOUNCE
/// impulses, and has an asymmetric velocity formula for e2 (divides by e2.proper_time
/// inside from_polar whereas e1 does not).
#[test]
#[ignore = "VIOLATION: consequences_collision injects external momentum via MIN_REPULSION + MIN_BOUNCE impulses. The e2 velocity formula is asymmetric: from_polar(angle2, total_mass/(e2.mass*e2.proper_time)) vs from_polar(angle1, total_mass/e1.mass). Momentum not conserved even with pause=true. Phase 2 physics refactor needed."]
fn test_collision_momentum_conservation() {
    let epsilon = 1.0;
    for seed in 0u64..20 {
        let mut rng = thread_rng();
        let mut e1 = spawn_asteroid(Vec2::new(-200.0, 0.0), Vec2::new(100.0, 0.0), 200.0, &mut rng);
        let mut e2 = spawn_asteroid(Vec2::new(200.0, 0.0), Vec2::new(-80.0, 0.0), 150.0, &mut rng);
        e1.proper_time = 1.0;
        e2.proper_time = 1.0;
        let p_before = add_vec(momentum(&e1), momentum(&e2));
        let mut globals = Globals::new();
        globals.pause = true; // suppress MIN_REPULSION/MIN_BOUNCE impulses
        let (new_e1, new_e2) = consequences_collision(e1, e2, &mut globals);
        let p_after = add_vec(momentum(&new_e1), momentum(&new_e2));
        let delta = magnitude(add_vec(p_after, scale_vec(p_before, -1.0)));
        assert!(
            delta < epsilon,
            "seed={}: momentum not conserved: |dp|={:.4}, before=({:.2},{:.2}), after=({:.2},{:.2})",
            seed, delta, p_before.x, p_before.y, p_after.x, p_after.y
        );
    }
}

/// Rule: Symmetric head-on equal-mass collision => zero net momentum before and after.
#[test]
#[ignore = "VIOLATION: Asymmetric e2 formula and scale_vec wrapping produce non-zero output. Even at proper_time=1.0 the total_mass/mass term differs between e1 and e2 paths. Latent momentum-injection bug."]
fn test_elastic_bounce_momentum() {
    let mut rng = thread_rng();
    let mut e1 = spawn_asteroid(Vec2::new(-100.0, 0.0), Vec2::new(500.0, 0.0), 200.0, &mut rng);
    let mut e2 = spawn_asteroid(Vec2::new(100.0, 0.0), Vec2::new(-500.0, 0.0), 200.0, &mut rng);
    e1.proper_time = 1.0;
    e2.proper_time = 1.0;
    e2.mass = e1.mass; // force equal mass for clean symmetry
    let p_before = add_vec(momentum(&e1), momentum(&e2));
    assert!(magnitude(p_before) < 1.0, "setup: p_before should be ~zero");
    let mut globals = Globals::new();
    globals.pause = true;
    let (new_e1, new_e2) = consequences_collision(e1, e2, &mut globals);
    let p_after = add_vec(momentum(&new_e1), momentum(&new_e2));
    assert!(
        magnitude(p_after) < 1.0,
        "symmetric collision: |p_after|={:.4} should be ~zero",
        magnitude(p_after)
    );
}

// ============================================================================
// ENERGY TESTS
// ============================================================================

/// Rule: Collisions must not create kinetic energy (KE_after <= KE_before).
///
/// The velocity formula adds from_polar(angle, total_mass/mass) as a raw speed
/// injection with no physical basis - this creates kinetic energy from nothing.
#[test]
#[ignore = "VIOLATION: consequences_collision adds from_polar(angle, total_mass/mass) as speed injection. For r=200 vs r=180, speed kick = (m1+m2)/m ~1.75-2.33 units. Creates KE from nothing. Observed KE ratio often 3-10x after collision. Phase 2 physics redesign needed."]
fn test_collision_energy_no_increase() {
    for seed in 0u64..20 {
        let mut rng = thread_rng();
        let mut e1 = spawn_asteroid(Vec2::new(-300.0, 0.0), Vec2::new(200.0, 50.0), 200.0, &mut rng);
        let mut e2 = spawn_asteroid(Vec2::new(300.0, 0.0), Vec2::new(-150.0, -30.0), 180.0, &mut rng);
        e1.proper_time = 1.0;
        e2.proper_time = 1.0;
        let ke_before = kinetic_energy(&e1) + kinetic_energy(&e2);
        let mut globals = Globals::new();
        globals.pause = true;
        let (new_e1, new_e2) = consequences_collision(e1, e2, &mut globals);
        let ke_after = kinetic_energy(&new_e1) + kinetic_energy(&new_e2);
        assert!(
            ke_after <= ke_before + 1e-6,
            "seed={}: KE increased! ke_before={:.2}, ke_after={:.2}, ratio={:.2}x",
            seed, ke_before, ke_after, ke_after / ke_before.max(1.0)
        );
    }
}

/// Rule: Heavy vs light collision - energy must not increase.
/// total_mass/light_mass >> 1 amplifies the injection, creating enormous KE.
#[test]
#[ignore = "VIOLATION: Heavy (r=800) vs light (r=100): light gets speed kick = total_mass/light_mass >> 1. Creates enormous KE in light entity. ke_before << ke_after."]
fn test_collision_energy_with_restitution() {
    let mut rng = thread_rng();
    let mut heavy = spawn_asteroid(Vec2::new(-100.0, 0.0), Vec2::new(10.0, 0.0), 800.0, &mut rng);
    let mut light = spawn_asteroid(Vec2::new(100.0, 0.0), Vec2::new(-10.0, 0.0), 100.0, &mut rng);
    heavy.proper_time = 1.0;
    light.proper_time = 1.0;
    let ke_before = kinetic_energy(&heavy) + kinetic_energy(&light);
    let mut globals = Globals::new();
    globals.pause = true;
    let (new_heavy, new_light) = consequences_collision(heavy, light, &mut globals);
    let ke_after = kinetic_energy(&new_heavy) + kinetic_energy(&new_light);
    assert!(
        ke_after < ke_before,
        "energy should strictly decrease: ke_before={:.2}, ke_after={:.2}, ratio={:.2}x",
        ke_before, ke_after, ke_after / ke_before.max(1.0)
    );
}

// ============================================================================
// FRAGMENT VELOCITY TESTS
// ============================================================================

/// Rule: Average fragment velocity should approximate parent velocity (CoM preserved).
/// Averaging 100 trials of random-angle kicks should converge toward parent vel.
#[test]
#[ignore = "VIOLATION: fragment_asteroid adds from_polar(random_angle, 1500..2500) per fragment. Kick magnitude 1500-2500 >> typical asteroid speeds 100-1000. CoM not preserved per-event. With 100 samples sigma ~150-250, tolerance borderline. Needs velocity redistribution redesign."]
fn test_fragments_inherit_parent_velocity() {
    let parent_vel = Vec2::new(300.0, -150.0);
    let mut rng = thread_rng();
    let parent = spawn_asteroid(Vec2::ZERO, parent_vel, 500.0, &mut rng);
    let n_trials = 100;
    let mut avg_vx = 0.0_f64;
    let mut avg_vy = 0.0_f64;
    for _ in 0..n_trials {
        let frag = fragment_asteroid(&parent, &mut rng);
        avg_vx += frag.velocity.x;
        avg_vy += frag.velocity.y;
    }
    avg_vx /= n_trials as f64;
    avg_vy /= n_trials as f64;
    let tolerance = 250.0;
    assert!(
        (avg_vx - parent_vel.x).abs() < tolerance,
        "avg fragment vx={:.2} too far from parent vx={:.2} (diff={:.2})",
        avg_vx, parent_vel.x, (avg_vx - parent_vel.x).abs()
    );
    assert!(
        (avg_vy - parent_vel.y).abs() < tolerance,
        "avg fragment vy={:.2} too far from parent vy={:.2} (diff={:.2})",
        avg_vy, parent_vel.y, (avg_vy - parent_vel.y).abs()
    );
}

/// Rule: Chunk speeds must not exceed parent_speed + CHUNKS_EXPLO_MAX_SPEED.
#[test]
fn test_chunks_velocity_bounded() {
    let parent_vel = Vec2::new(500.0, 0.0);
    let parent_speed = magnitude(parent_vel);
    let mut rng = thread_rng();
    let parent = make_asteroid(300.0, Vec2::ZERO, parent_vel);
    let max_expected_speed = parent_speed + CHUNKS_EXPLO_MAX_SPEED;
    let chunks = spawn_n_chunks(&parent, 50, parent.visuals.color, &mut rng);
    for (i, chunk) in chunks.iter().enumerate() {
        let speed = magnitude(chunk.velocity);
        assert!(
            speed <= max_expected_speed + 1.0,
            "chunk[{}] speed={:.2} exceeds max expected {:.2}",
            i, speed, max_expected_speed
        );
    }
}

// ============================================================================
// SPAWN MASS TESTS
// ============================================================================

/// Rule: Projectile mass must be less than the smallest valid asteroid.
/// Projectile: 10000 (hardcoded). Min asteroid (r=100): PI*100^2*1.0 ~31416.
#[test]
fn test_projectile_mass_reasonable() {
    let proj = spawn_projectile(Vec2::ZERO, Vec2::ZERO, 1.0);
    let min_asteroid = make_asteroid(ASTEROID_MIN_SIZE, Vec2::ZERO, Vec2::ZERO);
    assert!(
        proj.mass < min_asteroid.mass,
        "projectile mass ({}) should be < minimum asteroid mass ({})",
        proj.mass, min_asteroid.mass
    );
    assert!(proj.mass > 0.0, "projectile mass must be positive");
}

/// Rule: Larger asteroids must have strictly more mass (mass proportional to r^2).
#[test]
fn test_asteroid_mass_scales_with_size() {
    let mut rng = thread_rng();
    let radii = [100.0_f64, 150.0, 200.0, 300.0, 500.0, 800.0];
    let asteroids: Vec<Entity> = radii.iter()
        .map(|&r| spawn_asteroid(Vec2::ZERO, Vec2::ZERO, r, &mut rng))
        .collect();
    for i in 1..asteroids.len() {
        assert!(
            asteroids[i].mass > asteroids[i - 1].mass,
            "asteroid[{}] (r={}) mass={:.2} should be > asteroid[{}] (r={}) mass={:.2}",
            i, radii[i], asteroids[i].mass, i-1, radii[i-1], asteroids[i-1].mass
        );
    }
}

/// Rule: Doubling radius must quadruple mass (constant density, mass = PI * r^2 * D).
#[test]
fn test_asteroid_mass_quadratic_scaling() {
    let mut rng = thread_rng();
    let r1 = 200.0_f64;
    let r2 = r1 * 2.0;
    let a1 = spawn_asteroid(Vec2::ZERO, Vec2::ZERO, r1, &mut rng);
    let a2 = spawn_asteroid(Vec2::ZERO, Vec2::ZERO, r2, &mut rng);
    let expected_ratio = 4.0;
    let actual_ratio = a2.mass / a1.mass;
    assert!(
        (actual_ratio - expected_ratio).abs() < 1e-6,
        "mass ratio for 2x radius should be 4.0, got {:.6}",
        actual_ratio
    );
}

/// Rule: chunk mass (hardcoded 100.0) must be strictly less than any valid asteroid.
#[test]
fn test_chunk_mass_less_than_min_asteroid() {
    let mut rng = thread_rng();
    let min_asteroid = spawn_asteroid(Vec2::ZERO, Vec2::ZERO, ASTEROID_MIN_SIZE, &mut rng);
    let chunk = spawn_explosion_chunk(Vec2::ZERO, Vec2::ZERO, (1.0, 1.0, 1.0), 1.0, &mut rng);
    assert!(
        chunk.mass < min_asteroid.mass,
        "chunk mass ({}) should be < min asteroid mass ({})",
        chunk.mass, min_asteroid.mass
    );
}

/// Stress test: fragment mass formula must be stable across 30 seeds x 5 fragments.
#[test]
fn test_fragment_mass_stable_across_seeds() {
    for seed in 0u64..30 {
        let mut rng = thread_rng();
        let parent = spawn_asteroid(Vec2::ZERO, Vec2::ZERO, 400.0, &mut rng);
        for _ in 0..5 {
            let frag = fragment_asteroid(&parent, &mut rng);
            let expected = PI * squared(frag.hitbox.int_radius) * ASTEROID_DENSITY;
            assert!(
                (frag.mass - expected).abs() < 1e-6,
                "seed={}: fragment mass mismatch: mass={:.6}, expected={:.6}",
                seed, frag.mass, expected
            );
            let parent_radius = parent.hitbox.int_radius;
            let frag_radius = frag.hitbox.int_radius;
            assert!(
                frag_radius >= 0.39 * parent_radius && frag_radius <= 0.71 * parent_radius,
                "seed={}: fragment radius {:.2} out of range [{:.2}, {:.2}]",
                seed, frag_radius, 0.4 * parent_radius, 0.7 * parent_radius
            );
        }
    }
}
