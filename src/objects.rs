use crate::math_utils::{
    add_vec, from_polar, magnitude, rand_range, scale_vec, squared, sub_vec, Vec2,
};
use crate::parameters::*;
use rand::Rng;
use std::f64::consts::PI;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum EntityKind {
    Asteroid,
    /// Chunk particle ejected by explosion (distinct from asteroid, no collision)
    Chunk,
    Projectile,
    Ship,
    Explosion,
    Smoke,
    Shotgun,
    Sniper,
    Machinegun,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Polygon(pub Vec<(f64, f64)>);

#[derive(Clone, Debug, serde::Serialize)]
pub struct Hitbox {
    pub ext_radius: f64,
    pub int_radius: f64,
    pub avg_radius: f64,
    pub points: Polygon,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Visuals {
    pub color: (f64, f64, f64), // (r, v, b) tuple for now
    pub radius: f64,
    pub shapes: Vec<((f64, f64, f64), Polygon)>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Entity {
    pub kind: EntityKind,
    pub hitbox: Hitbox,
    pub visuals: Visuals,
    pub mass: f64,
    pub health: f64,
    pub max_health: f64,
    pub dam_ratio: f64,
    pub dam_res: f64,
    pub phys_ratio: f64,
    pub phys_res: f64,
    pub position: Vec2,
    pub velocity: Vec2,
    pub orientation: f64,
    pub moment: f64,
    pub proper_time: f64,
    pub hdr_exposure: f64,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Star {
    pub last_pos: Vec2,
    pub pos: Vec2,
    pub proximity: f64,
    pub lum: f64,
}

// ============================================================================
// Spawn functions
// ============================================================================

pub fn spawn_ship() -> Entity {
    // 16-sided polygon approximating the ship base circle (replaces CPU fill_circle)
    let base_radius = SHIP_RADIUS * 0.9;
    let n_sides = 16usize;
    let circle_poly: Vec<(f64, f64)> = (0..n_sides)
        .map(|i| {
            let angle = 2.0 * PI * i as f64 / n_sides as f64;
            (angle, base_radius)
        })
        .collect();

    let shapes = vec![
        ((1000.0, 100.0, 25.0), Polygon(circle_poly)),
        (
            (200.0, 20.0, 20.0),
            Polygon(vec![
                (0.0, 3.0 * SHIP_RADIUS),
                (3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
                (PI, SHIP_RADIUS),
                (-3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
            ]),
        ),
        (
            (250.0, 25.0, 25.0),
            Polygon(vec![
                (0.0, 3.0 * SHIP_RADIUS),
                (PI, SHIP_RADIUS),
                (-3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
            ]),
        ),
        (
            (120.0, 5.0, 5.0),
            Polygon(vec![
                (0.0, 3.0 * SHIP_RADIUS),
                (3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
                (PI, SHIP_RADIUS),
            ]),
        ),
        (
            (10.0, 10.0, 10.0),
            Polygon(vec![
                (PI, SHIP_RADIUS / 3.0),
                (PI, SHIP_RADIUS),
                (-3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
            ]),
        ),
        (
            (30.0, 30.0, 30.0),
            Polygon(vec![
                (PI, SHIP_RADIUS / 3.0),
                (3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
                (PI, SHIP_RADIUS),
            ]),
        ),
        (
            (200.0, 180.0, 160.0),
            Polygon(vec![
                (0.0, 3.0 * SHIP_RADIUS),
                (0.0, 1.5 * SHIP_RADIUS),
                (-PI / 8.0, 1.5 * SHIP_RADIUS),
            ]),
        ),
        (
            (20.0, 30.0, 40.0),
            Polygon(vec![
                (0.0, 3.0 * SHIP_RADIUS),
                (PI / 8.0, 1.5 * SHIP_RADIUS),
                (0.0, 1.5 * SHIP_RADIUS),
            ]),
        ),
    ];

    Entity {
        kind: EntityKind::Ship,
        visuals: Visuals {
            color: (1000.0, 100.0, 25.0),
            radius: SHIP_RADIUS * 0.9,
            shapes,
        },
        hitbox: Hitbox {
            ext_radius: 3.0 * SHIP_RADIUS,
            int_radius: SHIP_RADIUS,
            avg_radius: (3.0 * SHIP_RADIUS + SHIP_RADIUS) / 2.0,
            points: Polygon(vec![
                (0.0, 3.0 * SHIP_RADIUS),
                (3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
                (PI, SHIP_RADIUS),
                (-3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
            ]),
        },
        mass: PI * squared(SHIP_RADIUS) * SHIP_DENSITY,
        health: SHIP_MAX_HEALTH,
        max_health: SHIP_MAX_HEALTH,
        dam_ratio: SHIP_DAM_RATIO,
        dam_res: SHIP_DAM_RES,
        phys_ratio: SHIP_PHYS_RATIO,
        phys_res: SHIP_PHYS_RES,
        position: Vec2::ZERO, // Will be set by caller
        velocity: Vec2::ZERO,
        orientation: PI / 2.0,
        moment: 0.0,
        proper_time: 1.0,
        hdr_exposure: 1.0,
    }
}

pub fn spawn_projectile(position: Vec2, velocity: Vec2, proper_time: f64) -> Entity {
    Entity {
        kind: EntityKind::Projectile,
        visuals: Visuals {
            color: (2000.0, 400.0, 200.0),
            radius: PROJECTILE_RADIUS,
            shapes: vec![],
        },
        hitbox: Hitbox {
            int_radius: PROJECTILE_RADIUS_HITBOX,
            ext_radius: PROJECTILE_RADIUS_HITBOX,
            avg_radius: PROJECTILE_RADIUS_HITBOX,
            points: Polygon(vec![]),
        },
        mass: 10000.0,
        health: PROJECTILE_HEALTH,
        max_health: PROJECTILE_HEALTH,
        dam_res: 0.0,
        dam_ratio: 1.0,
        phys_res: 0.0,
        phys_ratio: 1.0,
        position,
        velocity,
        orientation: 0.0,
        moment: 0.0,
        proper_time,
        hdr_exposure: 4.0,
    }
}

pub fn spawn_n_projectiles(
    ship: &Entity,
    n: i32,
    projectile_min_speed: f64,
    projectile_max_speed: f64,
    projectile_deviation: f64,
    projectile_herit_speed: bool,
    rng: &mut impl Rng,
) -> Vec<Entity> {
    let mut projectiles = Vec::new();
    for _ in 0..n {
        let deviation_angle = (rng.gen::<f64>() - 0.5) * projectile_deviation + ship.orientation;
        let speed =
            projectile_min_speed + rng.gen::<f64>() * (projectile_max_speed - projectile_min_speed);

        let vel = if projectile_herit_speed {
            add_vec(ship.velocity, from_polar(deviation_angle, speed))
        } else {
            from_polar(deviation_angle, speed)
        };

        let pos = add_vec(
            ship.position,
            from_polar(ship.orientation, ship.hitbox.ext_radius),
        );

        projectiles.push(spawn_projectile(pos, vel, ship.proper_time));
    }
    projectiles
}

pub fn spawn_explosion_chunk(
    position: Vec2,
    velocity: Vec2,
    color: (f64, f64, f64),
    proper_time: f64,
    rng: &mut impl Rng,
) -> Entity {
    Entity {
        kind: EntityKind::Chunk,
        visuals: Visuals {
            color,
            radius: rand_range(CHUNKS_EXPLO_MIN_RADIUS, CHUNKS_EXPLO_MAX_RADIUS, rng),
            shapes: vec![],
        },
        hitbox: Hitbox {
            int_radius: 0.0,
            ext_radius: 0.0,
            avg_radius: 0.0,
            points: Polygon(vec![]),
        },
        mass: 100.0,
        health: PROJECTILE_HEALTH,
        max_health: PROJECTILE_HEALTH,
        dam_res: 0.0,
        dam_ratio: 1.0,
        phys_res: 0.0,
        phys_ratio: 1.0,
        position,
        velocity,
        orientation: 0.0,
        moment: 0.0,
        proper_time,
        hdr_exposure: 4.0,
    }
}

pub fn spawn_n_chunks(
    ship: &Entity,
    n: i32,
    color: (f64, f64, f64),
    rng: &mut impl Rng,
) -> Vec<Entity> {
    let mut chunks = Vec::new();
    for _ in 0..n {
        let angle = rng.gen::<f64>() * 2.0 * PI;
        let speed = rand_range(CHUNKS_EXPLO_MIN_SPEED, CHUNKS_EXPLO_MAX_SPEED, rng);
        let vel = add_vec(ship.velocity, from_polar(angle, speed));

        chunks.push(spawn_explosion_chunk(
            ship.position,
            vel,
            color,
            ship.proper_time,
            rng,
        ));
    }
    chunks
}

pub fn spawn_explosion(projectile: &Entity, rng: &mut impl Rng) -> Entity {
    let rad = rand_range(EXPLOSION_MIN_RADIUS, EXPLOSION_MAX_RADIUS, rng);
    let rand_lum = rand_range(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng);

    Entity {
        kind: EntityKind::Explosion,
        visuals: Visuals {
            color: (2000.0 * rand_lum, 500.0 * rand_lum, 200.0 * rand_lum),
            radius: rad,
            shapes: vec![],
        },
        hitbox: Hitbox {
            int_radius: rad,
            ext_radius: rad,
            avg_radius: rad,
            points: Polygon(vec![]),
        },
        mass: EXPLOSION_DAMAGES_PROJECTILE,
        health: 0.0,
        max_health: 0.0,
        dam_res: 0.0,
        dam_ratio: 0.0,
        phys_res: 0.0,
        phys_ratio: 0.0,
        position: projectile.position,
        velocity: {
            let rand_vel = from_polar(rng.gen::<f64>() * 2.0 * PI, rng.gen::<f64>() * SMOKE_MAX_SPEED);
            crate::math_utils::add_vec(projectile.velocity, rand_vel)
        },
        orientation: 0.0,
        moment: 0.0,
        proper_time: 1.0,
        hdr_exposure: 1.0,
    }
}

#[derive(Clone, Debug)]
pub struct ExplosionObjectSideEffects {
    pub add_color: Option<(f64, f64, f64)>,
    pub exposure_multiplier: Option<f64>,
}

pub fn spawn_explosion_object(
    obj: &Entity,
    flashes_enabled: bool,
    variable_exposure_enabled: bool,
    _flashes_saturate: f64,
    flashes_explosion: f64,
    flashes_normal_mass: f64,
    rng: &mut impl Rng,
) -> (Entity, ExplosionObjectSideEffects) {
    let rad = EXPLOSION_RATIO_RADIUS * obj.hitbox.int_radius;
    let rand_lum = rand_range(
        EXPLOSION_MIN_EXPOSURE_HERITATE,
        EXPLOSION_MAX_EXPOSURE_HERITATE,
        rng,
    );

    let mut side_effects = ExplosionObjectSideEffects {
        add_color: None,
        exposure_multiplier: None,
    };

    if flashes_enabled {
        let base_color = obj.visuals.color;
        let intensified = (
            base_color.0 * rand_lum,
            base_color.1 * rand_lum,
            base_color.2 * rand_lum,
        );
        let flash_intensity = obj.mass * flashes_explosion * rand_lum / flashes_normal_mass;
        side_effects.add_color = Some((
            intensified.0 * flash_intensity,
            intensified.1 * flash_intensity,
            intensified.2 * flash_intensity,
        ));
    }

    if variable_exposure_enabled {
        side_effects.exposure_multiplier = Some(0.99);
    }

    let explosion = Entity {
        kind: EntityKind::Explosion,
        visuals: Visuals {
            color: (
                obj.visuals.color.0 * rand_lum,
                obj.visuals.color.1 * rand_lum,
                obj.visuals.color.2 * rand_lum,
            ),
            radius: rad,
            shapes: vec![],
        },
        hitbox: Hitbox {
            int_radius: rad,
            ext_radius: rad,
            avg_radius: rad,
            points: Polygon(vec![]),
        },
        mass: EXPLOSION_DAMAGES_OBJET,
        health: 0.0,
        max_health: 0.0,
        dam_res: 0.0,
        dam_ratio: 0.0,
        phys_res: 0.0,
        phys_ratio: 0.0,
        position: obj.position,
        velocity: {
            let rand_vel = from_polar(rng.gen::<f64>() * 2.0 * PI, rng.gen::<f64>() * SMOKE_MAX_SPEED);
            crate::math_utils::add_vec(obj.velocity, rand_vel)
        },
        orientation: 0.0,
        moment: 0.0,
        proper_time: obj.proper_time,
        hdr_exposure: rand_lum,
    };

    (explosion, side_effects)
}

pub fn spawn_explosion_death(ship: &Entity, elapsed_time: f64, rng: &mut impl Rng) -> Entity {
    let rad = rand_range(EXPLOSION_DEATH_MIN_RADIUS, EXPLOSION_DEATH_MAX_RADIUS, rng);
    let rand_lum = rand_range(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng);

    Entity {
        kind: EntityKind::Explosion,
        visuals: Visuals {
            color: (2000.0 * rand_lum, 500.0 * rand_lum, 200.0 * rand_lum),
            radius: rad,
            shapes: vec![],
        },
        hitbox: Hitbox {
            int_radius: rad,
            ext_radius: rad,
            avg_radius: rad,
            points: Polygon(vec![]),
        },
        mass: EXPLOSION_DAMAGES_DEATH * elapsed_time,
        health: 0.0,
        max_health: 0.0,
        dam_res: 0.0,
        dam_ratio: 0.0,
        phys_res: 0.0,
        phys_ratio: 0.0,
        position: ship.position,
        velocity: {
            let rand_vel = from_polar(rng.gen::<f64>() * 2.0 * PI, rng.gen::<f64>() * SMOKE_MAX_SPEED);
            crate::math_utils::add_vec(ship.velocity, rand_vel)
        },
        orientation: 0.0,
        moment: 0.0,
        proper_time: ship.proper_time,
        hdr_exposure: 1.0,
    }
}

pub fn spawn_chunk_explosion(
    obj: &Entity,
    flashes_enabled: bool,
    _flashes_saturate: f64,
    flashes_explosion: f64,
    flashes_normal_mass: f64,
    rng: &mut impl Rng,
) -> (Entity, ExplosionObjectSideEffects) {
    let rad = EXPLOSION_RATIO_RADIUS * obj.visuals.radius;
    let rand_lum = rand_range(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng);

    let mut side_effects = ExplosionObjectSideEffects {
        add_color: None,
        exposure_multiplier: None,
    };

    if flashes_enabled {
        let base_color = obj.visuals.color;
        let intensified = (
            base_color.0 * rand_lum,
            base_color.1 * rand_lum,
            base_color.2 * rand_lum,
        );
        let flash_intensity = obj.mass * flashes_explosion * rand_lum / flashes_normal_mass;
        side_effects.add_color = Some((
            intensified.0 * flash_intensity,
            intensified.1 * flash_intensity,
            intensified.2 * flash_intensity,
        ));
    }

    let explosion = Entity {
        kind: EntityKind::Explosion,
        visuals: Visuals {
            color: obj.visuals.color,
            radius: rad,
            shapes: vec![],
        },
        hitbox: Hitbox {
            int_radius: rad,
            ext_radius: rad,
            avg_radius: rad,
            points: Polygon(vec![]),
        },
        mass: EXPLOSION_DAMAGES_CHUNK,
        health: 0.0,
        max_health: 0.0,
        dam_res: 0.0,
        dam_ratio: 0.0,
        phys_res: 0.0,
        phys_ratio: 0.0,
        position: obj.position,
        velocity: {
            let rand_vel = from_polar(rng.gen::<f64>() * 2.0 * PI, rng.gen::<f64>() * SMOKE_MAX_SPEED);
            crate::math_utils::add_vec(obj.velocity, rand_vel)
        },
        orientation: 0.0,
        moment: 0.0,
        proper_time: obj.proper_time,
        hdr_exposure: rand_lum,
    };

    (explosion, side_effects)
}

pub fn spawn_muzzle(projectile: &Entity, rng: &mut impl Rng) -> Entity {
    let rand_lum = rand_range(
        EXPLOSION_MIN_EXPOSURE_HERITATE,
        EXPLOSION_MAX_EXPOSURE_HERITATE,
        rng,
    );

    Entity {
        kind: EntityKind::Smoke,
        visuals: Visuals {
            color: (
                projectile.visuals.color.0 * rand_lum,
                projectile.visuals.color.1 * rand_lum,
                projectile.visuals.color.2 * rand_lum,
            ),
            radius: MUZZLE_RATIO_RADIUS * projectile.visuals.radius,
            shapes: vec![],
        },
        hitbox: Hitbox {
            int_radius: 0.0,
            ext_radius: 0.0,
            avg_radius: 0.0,
            points: Polygon(vec![]),
        },
        mass: 0.0,
        health: 0.0,
        max_health: 0.0,
        dam_res: 0.0,
        dam_ratio: 0.0,
        phys_res: 0.0,
        phys_ratio: 0.0,
        position: projectile.position,
        velocity: scale_vec(projectile.velocity, MUZZLE_RATIO_SPEED),
        orientation: 0.0,
        moment: 0.0,
        proper_time: projectile.proper_time,
        hdr_exposure: rand_range(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng),
    }
}

pub fn spawn_fire(ship: &Entity, thrust_angle: f64, rng: &mut impl Rng) -> Entity {
    Entity {
        kind: EntityKind::Smoke,
        visuals: Visuals {
            color: (1500.0, 400.0, 200.0),
            radius: FIRE_RATIO_RADIUS * ship.hitbox.int_radius,
            shapes: vec![],
        },
        hitbox: Hitbox {
            int_radius: 0.0,
            ext_radius: 0.0,
            avg_radius: 0.0,
            points: Polygon(vec![]),
        },
        mass: 0.0,
        health: 0.0,
        max_health: 0.0,
        dam_res: 0.0,
        dam_ratio: 0.0,
        phys_res: 0.0,
        phys_ratio: 0.0,
        position: add_vec(
            ship.position,
            from_polar(thrust_angle + PI, ship.hitbox.int_radius),
        ),
        velocity: {
            // Backward kick scales with ship speed so fire always ejects visually
            let ship_speed =
                (ship.velocity.x * ship.velocity.x + ship.velocity.y * ship.velocity.y).sqrt();
            let kick =
                ship_speed + FIRE_MIN_SPEED + rng.gen::<f64>() * (FIRE_MAX_SPEED - FIRE_MIN_SPEED);
            add_vec(
                ship.velocity,
                add_vec(
                    from_polar(thrust_angle + PI, kick),
                    from_polar(
                        rng.gen::<f64>() * 2.0 * PI,
                        rng.gen::<f64>() * FIRE_MAX_RANDOM,
                    ),
                ),
            )
        },
        orientation: 0.0,
        moment: 0.0,
        proper_time: ship.proper_time,
        hdr_exposure: rand_range(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng),
    }
}

pub fn generate_asteroid_polygon(radius: f64, rng: &mut impl Rng) -> Polygon {
    let nb_sides =
        (ASTEROID_POLYGON_MIN_SIDES as f64).max(ASTEROID_POLYGON_SIZE_RATIO * radius) as i32;
    let mut points = Vec::new();

    for n in 1..=nb_sides {
        let angle = 2.0 * PI * n as f64 / nb_sides as f64;
        let distance = radius * rand_range(ASTEROID_POLYGON_MIN, ASTEROID_POLYGON_MAX, rng);
        points.push((angle, distance));
    }

    Polygon(points)
}

pub fn spawn_asteroid(pos: Vec2, vel: Vec2, radius: f64, rng: &mut impl Rng) -> Entity {
    let shape = generate_asteroid_polygon(radius, rng);
    let color = (
        rand_range(ASTEROID_MIN_LUM, ASTEROID_MAX_LUM, rng),
        rand_range(ASTEROID_MIN_LUM, ASTEROID_MAX_LUM, rng),
        rand_range(ASTEROID_MIN_LUM, ASTEROID_MAX_LUM, rng),
    );

    Entity {
        kind: EntityKind::Asteroid,
        visuals: Visuals {
            color,
            radius,
            shapes: vec![(color, shape.clone())],
        },
        hitbox: Hitbox {
            int_radius: radius,
            ext_radius: radius * ASTEROID_POLYGON_MAX,
            avg_radius: radius * (1.0 + ASTEROID_POLYGON_MAX) / 2.0,
            points: shape,
        },
        mass: PI * squared(radius) * ASTEROID_DENSITY,
        health: ASTEROID_MASS_HEALTH * PI * squared(radius) * ASTEROID_DENSITY
            + ASTEROID_MIN_HEALTH,
        max_health: ASTEROID_MASS_HEALTH * PI * squared(radius) * ASTEROID_DENSITY
            + ASTEROID_MIN_HEALTH,
        dam_res: ASTEROID_DAM_RES,
        dam_ratio: ASTEROID_DAM_RATIO,
        phys_res: ASTEROID_PHYS_RES,
        phys_ratio: ASTEROID_PHYS_RATIO,
        position: pos,
        velocity: vel,
        orientation: rng.gen::<f64>() * 2.0 * PI,
        moment: rng.gen::<f64>() * 2.0 * ASTEROID_MAX_MOMENT - ASTEROID_MAX_MOMENT,
        proper_time: 1.0,
        hdr_exposure: 1.0,
    }
}

/// Spawn a random asteroid for the given stage, positioned off-screen
pub fn spawn_random_asteroid(stage: i32, phys_w: f64, phys_h: f64, rng: &mut impl Rng) -> Entity {
    let radius = rand_range(ASTEROID_MIN_SPAWN_RADIUS, ASTEROID_MAX_SPAWN_RADIUS, rng);
    let pos = random_offscreen_position(radius, phys_w, phys_h, rng);
    let vel_angle = rng.gen::<f64>() * 2.0 * PI;
    let vel_magnitude = rand_range(
        ASTEROID_MIN_VELOCITY,
        ASTEROID_MAX_VELOCITY + ASTEROID_STAGE_VELOCITY * stage as f64,
        rng,
    );
    let vel = from_polar(vel_angle, vel_magnitude);
    spawn_asteroid(pos, vel, radius, rng)
}

/// Create a single fragment from a parent asteroid
pub fn fragment_asteroid(parent: &Entity, rng: &mut impl Rng) -> Entity {
    // Start with a fresh asteroid at parent's position/velocity/radius
    let mut fragment = spawn_asteroid(
        parent.position,
        parent.velocity,
        parent.hitbox.int_radius,
        rng,
    );

    let orientation = rng.gen::<f64>() * 2.0 * PI;
    let new_radius =
        rand_range(FRAGMENT_MIN_SIZE, FRAGMENT_MAX_SIZE, rng) * fragment.hitbox.int_radius;

    // Regenerate polygon for new size
    let new_shape = generate_asteroid_polygon(new_radius, rng);

    // Offset position from parent center
    fragment.position = add_vec(
        fragment.position,
        from_polar(orientation, fragment.hitbox.int_radius - new_radius),
    );

    // Update visuals with parent color
    fragment.visuals.radius = new_radius;
    fragment.visuals.color = parent.visuals.color;
    fragment.visuals.shapes = vec![(parent.visuals.color, new_shape.clone())];

    // Update hitbox
    fragment.hitbox.int_radius = new_radius;
    fragment.hitbox.ext_radius = new_radius * ASTEROID_POLYGON_MAX;
    fragment.hitbox.points = new_shape;

    // Recalculate mass and health for new size
    fragment.mass = PI * squared(new_radius) * ASTEROID_DENSITY;
    fragment.health = ASTEROID_MASS_HEALTH * fragment.mass + ASTEROID_MIN_HEALTH;
    fragment.max_health = fragment.health;

    // Add random velocity scatter
    fragment.velocity = add_vec(
        fragment.velocity,
        from_polar(
            orientation,
            rand_range(FRAGMENT_MIN_VELOCITY, FRAGMENT_MAX_VELOCITY, rng),
        ),
    );

    // Adjust HDR exposure randomly
    fragment.hdr_exposure *= rand_range(FRAGMENT_MIN_EXPOSURE, FRAGMENT_MAX_EXPOSURE, rng);

    fragment
}

/// Spawn fragment_number fragments for each dead entity in source, appending to dest.
pub fn spawn_fragments(
    source: &[Entity],
    dest: &mut Vec<Entity>,
    fragment_number: i32,
    rng: &mut impl Rng,
) {
    let dead: Vec<&Entity> = source.iter().filter(|e| e.health <= 0.0).collect();
    for _ in 0..fragment_number {
        for parent in &dead {
            dest.push(fragment_asteroid(parent, rng));
        }
    }
}

pub fn random_offscreen_position(
    radius: f64,
    phys_w: f64,
    phys_h: f64,
    rng: &mut impl Rng,
) -> Vec2 {
    loop {
        let x = rng.gen::<f64>() * 3.0 * phys_w - phys_w;
        let y = rng.gen::<f64>() * 3.0 * phys_h - phys_h;

        if !(y + radius > 0.0 && y - radius < phys_h && x + radius > 0.0 && x - radius < phys_w) {
            return Vec2::new(x, y);
        }
    }
}

pub fn spawn_random_star(phys_w: f64, phys_h: f64, rng: &mut impl Rng) -> Star {
    let randpos = Vec2::new(rng.gen::<f64>() * phys_w, rng.gen::<f64>() * phys_h);
    Star {
        last_pos: randpos,
        pos: randpos,
        proximity: rand_range(STAR_MIN_PROX, STAR_MAX_PROX, rng).powf(4.0),
        lum: rand_range(STAR_MIN_LUM, STAR_MAX_LUM, rng),
    }
}

pub fn spawn_stars(n: i32, phys_w: f64, phys_h: f64, rng: &mut impl Rng) -> Vec<Star> {
    let mut stars = Vec::new();
    for _ in 0..n {
        stars.push(spawn_random_star(phys_w, phys_h, rng));
    }
    stars
}

// ============================================================================
// Predicates
// ============================================================================

pub fn is_alive(e: &Entity) -> bool {
    e.health > 0.0
}

pub fn is_dead(e: &Entity) -> bool {
    e.health <= 0.0
}

pub fn positive_radius(e: &Entity) -> bool {
    e.visuals.radius > 0.0
}

pub fn is_chunk(e: &Entity) -> bool {
    e.hitbox.int_radius < CHUNK_MAX_SIZE
}

pub fn not_chunk(e: &Entity) -> bool {
    e.hitbox.int_radius >= CHUNK_MAX_SIZE
}

pub fn too_small(e: &Entity) -> bool {
    e.hitbox.ext_radius < ASTEROID_MIN_SIZE
}

pub fn big_enough(e: &Entity) -> bool {
    e.hitbox.ext_radius >= ASTEROID_MIN_SIZE
}

pub fn close_enough(e: &Entity, phys_w: f64, phys_h: f64) -> bool {
    let center = Vec2::new(phys_w / 2.0, phys_h / 2.0);
    magnitude(sub_vec(e.position, center)) < MAX_DIST
}

pub fn too_far(e: &Entity, phys_w: f64, phys_h: f64) -> bool {
    !close_enough(e, phys_w, phys_h)
}

pub fn check_spawn(e: &Entity, phys_w: f64, phys_h: f64) -> bool {
    let x = e.position.x;
    let y = e.position.y;
    let rad = e.hitbox.ext_radius;
    (x - rad < phys_w) && (x + rad > 0.0) && (y - rad < phys_h) && (y + rad > 0.0)
}

pub fn check_not_spawn(e: &Entity, phys_w: f64, phys_h: f64) -> bool {
    !check_spawn(e, phys_w, phys_h)
}

#[cfg(test)]
mod smoke_velocity_tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use crate::math::Vec2;

    fn make_entity_at_velocity(vx: f64, vy: f64) -> Entity {
        let mut e = spawn_ship();
        e.velocity = Vec2::new(vx, vy);
        e.position = Vec2::new(100.0, 100.0);
        e
    }

    #[test]
    fn spawn_explosion_velocity_includes_parent() {
        let mut rng = SmallRng::seed_from_u64(42);
        let parent = make_entity_at_velocity(1000.0, 0.0);
        let samples = 200;
        let total_vx: f64 = (0..samples)
            .map(|_| spawn_explosion(&parent, &mut rng).velocity.x)
            .sum();
        let avg_vx = total_vx / samples as f64;
        assert!(avg_vx > 500.0, "avg_vx={avg_vx}, expected ~1000.0");
    }

    #[test]
    fn spawn_explosion_death_velocity_includes_parent() {
        let mut rng = SmallRng::seed_from_u64(42);
        let ship = make_entity_at_velocity(0.0, -500.0);
        let samples = 200;
        let total_vy: f64 = (0..samples)
            .map(|_| spawn_explosion_death(&ship, 1.0, &mut rng).velocity.y)
            .sum();
        let avg_vy = total_vy / samples as f64;
        assert!(avg_vy < -200.0, "avg_vy={avg_vy}, expected ~-500.0");
    }
}
