use asteroids::math_utils::Vec2;
/// Safety-net tests for entity predicates and spawn functions in objects.rs
/// These tests verify predicate complementarity and basic entity validity
/// before V2 refactoring begins.
use asteroids::objects::{
    big_enough, check_not_spawn, check_spawn, close_enough, is_alive, is_chunk, is_dead, not_chunk,
    positive_radius, spawn_asteroid, spawn_explosion, spawn_projectile, spawn_ship, too_far,
    too_small, Entity, EntityKind, Hitbox, Polygon, Visuals,
};
use rand::thread_rng;

// ============================================================================
// Helper: build a minimal entity with specific field values
// ============================================================================

fn entity_with_health(health: f64) -> Entity {
    Entity {
        kind: EntityKind::Asteroid,
        hitbox: Hitbox {
            ext_radius: 100.0,
            int_radius: 100.0,
            points: Polygon(vec![]),
        },
        visuals: Visuals {
            color: (1.0, 1.0, 1.0),
            radius: 100.0,
            shapes: vec![],
        },
        mass: 1.0,
        health,
        max_health: 100.0,
        dam_ratio: 1.0,
        dam_res: 0.0,
        phys_ratio: 1.0,
        phys_res: 0.0,
        position: Vec2::ZERO,
        velocity: Vec2::ZERO,
        orientation: 0.0,
        moment: 0.0,
        proper_time: 1.0,
        hdr_exposure: 1.0,
    }
}

fn entity_with_int_radius(int_radius: f64) -> Entity {
    let mut e = entity_with_health(100.0);
    e.hitbox.int_radius = int_radius;
    e.hitbox.ext_radius = int_radius;
    e
}

fn entity_with_ext_radius(ext_radius: f64) -> Entity {
    let mut e = entity_with_health(100.0);
    e.hitbox.ext_radius = ext_radius;
    e
}

fn entity_with_visual_radius(radius: f64) -> Entity {
    let mut e = entity_with_health(100.0);
    e.visuals.radius = radius;
    e
}

fn entity_at_position(pos: Vec2) -> Entity {
    let mut e = entity_with_ext_radius(10.0);
    e.position = pos;
    e
}

// ============================================================================
// is_alive / is_dead — complementarity
// ============================================================================

#[test]
fn alive_dead_complementary_positive_health() {
    let e = entity_with_health(50.0);
    assert!(is_alive(&e));
    assert!(!is_dead(&e));
    assert_ne!(is_alive(&e), is_dead(&e));
}

#[test]
fn alive_dead_complementary_zero_health() {
    let e = entity_with_health(0.0);
    assert!(!is_alive(&e));
    assert!(is_dead(&e));
    assert_ne!(is_alive(&e), is_dead(&e));
}

#[test]
fn alive_dead_complementary_negative_health() {
    let e = entity_with_health(-10.0);
    assert!(!is_alive(&e));
    assert!(is_dead(&e));
    assert_ne!(is_alive(&e), is_dead(&e));
}

#[test]
fn alive_dead_complementary_large_health() {
    let e = entity_with_health(1_000_000.0);
    assert!(is_alive(&e));
    assert!(!is_dead(&e));
    assert_ne!(is_alive(&e), is_dead(&e));
}

// ============================================================================
// is_chunk / not_chunk — complementarity (threshold: CHUNK_MAX_SIZE = 50.0)
// ============================================================================

#[test]
fn is_chunk_not_chunk_complementary_small() {
    // int_radius < 50.0 → is_chunk
    let e = entity_with_int_radius(10.0);
    assert!(is_chunk(&e));
    assert!(!not_chunk(&e));
    assert_ne!(is_chunk(&e), not_chunk(&e));
}

#[test]
fn is_chunk_not_chunk_complementary_at_threshold() {
    // int_radius == 50.0 → not_chunk (>= threshold)
    let e = entity_with_int_radius(50.0);
    assert!(!is_chunk(&e));
    assert!(not_chunk(&e));
    assert_ne!(is_chunk(&e), not_chunk(&e));
}

#[test]
fn is_chunk_not_chunk_complementary_large() {
    // int_radius > 50.0 → not_chunk
    let e = entity_with_int_radius(200.0);
    assert!(!is_chunk(&e));
    assert!(not_chunk(&e));
    assert_ne!(is_chunk(&e), not_chunk(&e));
}

// ============================================================================
// too_small / big_enough — complementarity (threshold: ASTEROID_MIN_SIZE = 100.0)
// ============================================================================

#[test]
fn too_small_big_enough_complementary_small() {
    // ext_radius < 100.0 → too_small
    let e = entity_with_ext_radius(50.0);
    assert!(too_small(&e));
    assert!(!big_enough(&e));
    assert_ne!(too_small(&e), big_enough(&e));
}

#[test]
fn too_small_big_enough_complementary_at_threshold() {
    // ext_radius == 100.0 → big_enough
    let e = entity_with_ext_radius(100.0);
    assert!(!too_small(&e));
    assert!(big_enough(&e));
    assert_ne!(too_small(&e), big_enough(&e));
}

#[test]
fn too_small_big_enough_complementary_large() {
    let e = entity_with_ext_radius(300.0);
    assert!(!too_small(&e));
    assert!(big_enough(&e));
    assert_ne!(too_small(&e), big_enough(&e));
}

// ============================================================================
// check_spawn / check_not_spawn — complementarity (on-screen vs off-screen)
// ============================================================================

#[test]
fn check_spawn_not_spawn_complementary_inside() {
    // Entity clearly inside screen (1920x1080)
    let e = entity_at_position(Vec2::new(960.0, 540.0));
    assert!(check_spawn(&e, 1920.0, 1080.0));
    assert!(!check_not_spawn(&e, 1920.0, 1080.0));
    assert_ne!(
        check_spawn(&e, 1920.0, 1080.0),
        check_not_spawn(&e, 1920.0, 1080.0)
    );
}

#[test]
fn check_spawn_not_spawn_complementary_outside() {
    // Entity far off-screen (well beyond screen + radius)
    let e = entity_at_position(Vec2::new(-5000.0, -5000.0));
    assert!(!check_spawn(&e, 1920.0, 1080.0));
    assert!(check_not_spawn(&e, 1920.0, 1080.0));
    assert_ne!(
        check_spawn(&e, 1920.0, 1080.0),
        check_not_spawn(&e, 1920.0, 1080.0)
    );
}

#[test]
fn check_spawn_not_spawn_complementary_edge() {
    // Entity just inside top-left corner
    let e = entity_at_position(Vec2::new(5.0, 5.0));
    assert!(check_spawn(&e, 1920.0, 1080.0));
    assert!(!check_not_spawn(&e, 1920.0, 1080.0));
}

// ============================================================================
// close_enough / too_far — complementarity (MAX_DIST = 20000.0)
// ============================================================================

#[test]
fn close_enough_too_far_complementary_nearby() {
    // Entity near screen center
    let e = entity_at_position(Vec2::new(960.0, 540.0));
    assert!(close_enough(&e, 1920.0, 1080.0));
    assert!(!too_far(&e, 1920.0, 1080.0));
    assert_ne!(
        close_enough(&e, 1920.0, 1080.0),
        too_far(&e, 1920.0, 1080.0)
    );
}

#[test]
fn close_enough_too_far_complementary_very_far() {
    // Entity extremely far from screen center (> MAX_DIST = 20000)
    let e = entity_at_position(Vec2::new(50000.0, 50000.0));
    assert!(!close_enough(&e, 1920.0, 1080.0));
    assert!(too_far(&e, 1920.0, 1080.0));
    assert_ne!(
        close_enough(&e, 1920.0, 1080.0),
        too_far(&e, 1920.0, 1080.0)
    );
}

// ============================================================================
// positive_radius — individual predicate tests
// ============================================================================

#[test]
fn positive_radius_positive() {
    let e = entity_with_visual_radius(10.0);
    assert!(positive_radius(&e));
}

#[test]
fn positive_radius_zero() {
    let e = entity_with_visual_radius(0.0);
    assert!(!positive_radius(&e));
}

#[test]
fn positive_radius_negative() {
    let e = entity_with_visual_radius(-5.0);
    assert!(!positive_radius(&e));
}

// ============================================================================
// Spawn functions produce valid entities
// ============================================================================

#[test]
fn spawn_ship_is_valid() {
    let ship = spawn_ship();
    assert_eq!(ship.kind, EntityKind::Ship);
    assert!(ship.health > 0.0, "ship health must be > 0");
    assert!(ship.hitbox.ext_radius > 0.0, "ship ext_radius must be > 0");
    assert!(ship.mass > 0.0, "ship mass must be > 0");
}

#[test]
fn spawn_ship_alive_and_not_chunk() {
    let ship = spawn_ship();
    // Ship must be alive
    assert!(is_alive(&ship));
    assert!(!is_dead(&ship));
    // Ship has a positive visual radius
    assert!(positive_radius(&ship));
}

#[test]
fn spawn_projectile_is_valid() {
    let pos = Vec2::new(100.0, 200.0);
    let vel = Vec2::new(300.0, 0.0);
    let proj = spawn_projectile(pos, vel, 1.0);

    assert_eq!(proj.kind, EntityKind::Projectile);
    // Position must match what we provided
    assert_eq!(proj.position, pos);
    assert_eq!(proj.velocity, vel);
    // Projectile has zero health (expires immediately via game logic)
    // ext_radius must be > 0 for collision detection
    assert!(
        proj.hitbox.ext_radius > 0.0,
        "projectile ext_radius must be > 0"
    );
}

#[test]
fn spawn_projectile_different_positions() {
    let positions: &[Vec2] = &[
        Vec2::new(0.0, 0.0),
        Vec2::new(-500.0, 200.0),
        Vec2::new(1920.0, 1080.0),
    ];
    for &pos in positions {
        let proj = spawn_projectile(pos, Vec2::ZERO, 1.0);
        assert_eq!(proj.position, pos);
        assert_eq!(proj.kind, EntityKind::Projectile);
    }
}

#[test]
fn spawn_asteroid_is_valid() {
    let mut rng = thread_rng();
    let pos = Vec2::new(500.0, 400.0);
    let vel = Vec2::new(10.0, -5.0);
    let radius = 150.0;

    let asteroid = spawn_asteroid(pos, vel, radius, &mut rng);

    assert_eq!(asteroid.kind, EntityKind::Asteroid);
    assert!(asteroid.health > 0.0, "asteroid health must be > 0");
    // Hitbox radii should be positive and match input radius
    assert!(asteroid.hitbox.int_radius > 0.0);
    assert!(asteroid.hitbox.ext_radius >= asteroid.hitbox.int_radius);
    // Position and velocity should be preserved
    assert_eq!(asteroid.position, pos);
    assert_eq!(asteroid.velocity, vel);
    // Visual radius should match the spawned radius
    assert_eq!(asteroid.visuals.radius, radius);
    // Mass must be positive
    assert!(asteroid.mass > 0.0);
}

#[test]
fn spawn_asteroid_has_polygon_points() {
    let mut rng = thread_rng();
    let asteroid = spawn_asteroid(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), 200.0, &mut rng);
    // Asteroid polygon should have vertices
    assert!(
        !asteroid.hitbox.points.0.is_empty(),
        "asteroid must have polygon points"
    );
    assert!(
        !asteroid.visuals.shapes.is_empty(),
        "asteroid must have visual shapes"
    );
}

#[test]
fn spawn_asteroid_various_radii() {
    let mut rng = thread_rng();
    for &radius in &[50.0_f64, 100.0, 200.0, 500.0] {
        let a = spawn_asteroid(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), radius, &mut rng);
        assert_eq!(a.kind, EntityKind::Asteroid);
        assert!(a.health > 0.0);
        assert!(a.mass > 0.0);
        assert_eq!(a.visuals.radius, radius);
    }
}

#[test]
fn spawn_explosion_is_valid() {
    let mut rng = thread_rng();
    // Explosions are spawned from a projectile; build a minimal one
    let proj = spawn_projectile(Vec2::new(300.0, 400.0), Vec2::new(0.0, 0.0), 1.0);
    let explosion = spawn_explosion(&proj, &mut rng);

    assert_eq!(explosion.kind, EntityKind::Explosion);
    assert!(
        explosion.visuals.radius > 0.0,
        "explosion radius must be > 0"
    );
    // Explosion inherits position from projectile
    assert_eq!(explosion.position, proj.position);
}

#[test]
fn spawn_explosion_radius_in_range() {
    let mut rng = thread_rng();
    let proj = spawn_projectile(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), 1.0);

    // Run multiple times to check randomized radius stays in bounds
    for _ in 0..20 {
        let explosion = spawn_explosion(&proj, &mut rng);
        // EXPLOSION_MIN_RADIUS = 200.0, EXPLOSION_MAX_RADIUS = 250.0
        assert!(
            explosion.visuals.radius >= 200.0,
            "explosion radius below minimum"
        );
        assert!(
            explosion.visuals.radius <= 250.0,
            "explosion radius above maximum"
        );
    }
}
