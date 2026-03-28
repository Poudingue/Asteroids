use crate::parameters::{
    ASTEROID_MAX_SPAWN_RADIUS, ASTEROID_MAX_VELOCITY, ASTEROID_MIN_SPAWN_RADIUS,
    ASTEROID_MIN_VELOCITY, ASTEROID_STAGE_VELOCITY,
    FRAGMENT_MIN_SIZE, FRAGMENT_MAX_SIZE, FRAGMENT_MIN_VELOCITY, FRAGMENT_MAX_VELOCITY,
    FRAGMENT_MIN_EXPOSURE, FRAGMENT_MAX_EXPOSURE,
};
use crate::math_utils::{
    addtuple, carre, hypothenuse, multuple, polar_to_affine, randfloat, soustuple, Vec2,
};
use rand::Rng;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub enum EntityKind {
    Asteroid,
    Projectile,
    Ship,
    Explosion,
    Smoke,
    Spark,
    Shotgun,
    Sniper,
    Machinegun,
}

#[derive(Clone, Debug)]
pub struct Polygon(pub Vec<(f64, f64)>);

#[derive(Clone, Debug)]
pub struct Hitbox {
    pub ext_radius: f64,
    pub int_radius: f64,
    pub points: Polygon,
}

#[derive(Clone, Debug)]
pub struct Visuals {
    pub color: (f64, f64, f64), // (r, v, b) tuple for now
    pub radius: f64,
    pub shapes: Vec<((f64, f64, f64), Polygon)>,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Star {
    pub last_pos: Vec2,
    pub pos: Vec2,
    pub proximity: f64,
    pub lum: f64,
}

// ============================================================================
// Constants (hardcoded from parameters.ml for now)
// ============================================================================

const PI: f64 = std::f64::consts::PI;
const SHIP_RADIUS: f64 = 25.0;
const SHIP_DENSITY: f64 = 100.0;
const SHIP_MAX_HEALTH: f64 = 100.0;
const SHIP_DAM_RATIO: f64 = 0.8;
const SHIP_DAM_RES: f64 = 10.0;
const SHIP_PHYS_RATIO: f64 = 0.005;
const SHIP_PHYS_RES: f64 = 0.0;

const PROJECTILE_RADIUS: f64 = 15.0;
const PROJECTILE_RADIUS_HITBOX: f64 = 20.0;
const PROJECTILE_HEALTH: f64 = 0.0;

const ASTEROID_POLYGON_MIN_SIDES: i32 = 7;
const ASTEROID_POLYGON_SIZE_RATIO: f64 = 0.02;
const ASTEROID_POLYGON_MIN: f64 = 1.0;
const ASTEROID_POLYGON_MAX: f64 = 1.3;
const ASTEROID_DENSITY: f64 = 1.0;
const ASTEROID_MIN_HEALTH: f64 = 50.0;
const ASTEROID_MASS_HEALTH: f64 = 0.01;
const ASTEROID_DAM_RATIO: f64 = 1.0;
const ASTEROID_DAM_RES: f64 = 0.0;
const ASTEROID_PHYS_RATIO: f64 = 1.0;
const ASTEROID_PHYS_RES: f64 = 100.0;
const ASTEROID_MIN_LUM: f64 = 40.0;
const ASTEROID_MAX_LUM: f64 = 120.0;
const ASTEROID_MIN_SATUR: f64 = 0.4;
const ASTEROID_MAX_SATUR: f64 = 0.5;
const ASTEROID_MAX_MOMENT: f64 = 1.0;

const CHUNKS_EXPLO_MIN_RADIUS: f64 = 150.0;
const CHUNKS_EXPLO_MAX_RADIUS: f64 = 300.0;
const CHUNKS_EXPLO_MIN_SPEED: f64 = 10000.0;
const CHUNKS_EXPLO_MAX_SPEED: f64 = 20000.0;

const EXPLOSION_MAX_RADIUS: f64 = 250.0;
const EXPLOSION_MIN_RADIUS: f64 = 200.0;
const EXPLOSION_MIN_EXPOSURE: f64 = 0.4;
const EXPLOSION_MAX_EXPOSURE: f64 = 1.3;
const EXPLOSION_RATIO_RADIUS: f64 = 2.0;
const EXPLOSION_DEATH_MAX_RADIUS: f64 = 150.0;
const EXPLOSION_DEATH_MIN_RADIUS: f64 = 100.0;
const EXPLOSION_MIN_EXPOSURE_HERITATE: f64 = 2.0;
const EXPLOSION_MAX_EXPOSURE_HERITATE: f64 = 6.0;
const EXPLOSION_DAMAGES_PROJECTILE: f64 = 5000.0;
const EXPLOSION_DAMAGES_OBJET: f64 = 50.0;
const EXPLOSION_DAMAGES_CHUNK: f64 = 150.0;
const EXPLOSION_DAMAGES_DEATH: f64 = 50.0;
const EXPLOSION_SATURATE: f64 = 10.0;

const MUZZLE_RATIO_RADIUS: f64 = 3.0;
const MUZZLE_RATIO_SPEED: f64 = 0.05;

const FIRE_MAX_RANDOM: f64 = 100.0;
const FIRE_MIN_SPEED: f64 = 500.0;
const FIRE_MAX_SPEED: f64 = 1000.0;
const FIRE_RATIO_RADIUS: f64 = 1.4;

const SMOKE_MAX_SPEED: f64 = 400.0;

const STAR_MIN_PROX: f64 = 0.3;
const STAR_MAX_PROX: f64 = 0.9;
const STAR_MIN_LUM: f64 = 0.0;
const STAR_MAX_LUM: f64 = 4.0;

const CHUNK_MAX_SIZE: f64 = 50.0;
const ASTEROID_MIN_SIZE: f64 = 100.0;
const MAX_DIST: f64 = 20000.0;

// ============================================================================
// Spawn functions
// ============================================================================

pub fn spawn_ship() -> Entity {
    let shapes = vec![
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
            points: Polygon(vec![
                (0.0, 3.0 * SHIP_RADIUS),
                (3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
                (PI, SHIP_RADIUS),
                (-3.0 * PI / 4.0, 2.0 * SHIP_RADIUS),
            ]),
        },
        mass: PI * carre(SHIP_RADIUS) * SHIP_DENSITY,
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
            addtuple(ship.velocity, polar_to_affine(deviation_angle, speed))
        } else {
            polar_to_affine(deviation_angle, speed)
        };

        let pos = addtuple(
            ship.position,
            polar_to_affine(ship.orientation, ship.hitbox.ext_radius),
        );

        projectiles.push(spawn_projectile(pos, vel, ship.proper_time));
    }
    projectiles
}

pub fn spawn_chunk_explo(
    position: Vec2,
    velocity: Vec2,
    color: (f64, f64, f64),
    proper_time: f64,
    rng: &mut impl Rng,
) -> Entity {
    Entity {
        kind: EntityKind::Asteroid,
        visuals: Visuals {
            color,
            radius: randfloat(CHUNKS_EXPLO_MIN_RADIUS, CHUNKS_EXPLO_MAX_RADIUS, rng),
            shapes: vec![],
        },
        hitbox: Hitbox {
            int_radius: 0.0,
            ext_radius: 0.0,
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
        let speed = randfloat(CHUNKS_EXPLO_MIN_SPEED, CHUNKS_EXPLO_MAX_SPEED, rng);
        let vel = addtuple(ship.velocity, polar_to_affine(angle, speed));

        chunks.push(spawn_chunk_explo(
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
    let rad = randfloat(EXPLOSION_MIN_RADIUS, EXPLOSION_MAX_RADIUS, rng);
    let rand_lum = randfloat(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng);

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
        velocity: polar_to_affine(
            rng.gen::<f64>() * 2.0 * PI,
            rng.gen::<f64>() * SMOKE_MAX_SPEED,
        ),
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
    let rand_lum = randfloat(
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
        velocity: polar_to_affine(
            rng.gen::<f64>() * 2.0 * PI,
            rng.gen::<f64>() * SMOKE_MAX_SPEED,
        ),
        orientation: 0.0,
        moment: 0.0,
        proper_time: obj.proper_time,
        hdr_exposure: rand_lum,
    };

    (explosion, side_effects)
}

pub fn spawn_explosion_death(ship: &Entity, elapsed_time: f64, rng: &mut impl Rng) -> Entity {
    let rad = randfloat(EXPLOSION_DEATH_MIN_RADIUS, EXPLOSION_DEATH_MAX_RADIUS, rng);
    let rand_lum = randfloat(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng);

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
        velocity: polar_to_affine(
            rng.gen::<f64>() * 2.0 * PI,
            rng.gen::<f64>() * SMOKE_MAX_SPEED,
        ),
        orientation: 0.0,
        moment: 0.0,
        proper_time: ship.proper_time,
        hdr_exposure: 1.0,
    }
}

pub fn spawn_explosion_chunk(
    obj: &Entity,
    flashes_enabled: bool,
    _flashes_saturate: f64,
    flashes_explosion: f64,
    flashes_normal_mass: f64,
    rng: &mut impl Rng,
) -> (Entity, ExplosionObjectSideEffects) {
    let rad = EXPLOSION_RATIO_RADIUS * obj.visuals.radius;
    let rand_lum = randfloat(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng);

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
        velocity: polar_to_affine(
            rng.gen::<f64>() * 2.0 * PI,
            rng.gen::<f64>() * SMOKE_MAX_SPEED,
        ),
        orientation: 0.0,
        moment: 0.0,
        proper_time: obj.proper_time,
        hdr_exposure: rand_lum,
    };

    (explosion, side_effects)
}

pub fn spawn_muzzle(projectile: &Entity, rng: &mut impl Rng) -> Entity {
    let rand_lum = randfloat(
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
        velocity: multuple(projectile.velocity, MUZZLE_RATIO_SPEED),
        orientation: 0.0,
        moment: 0.0,
        proper_time: projectile.proper_time,
        hdr_exposure: randfloat(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng),
    }
}

pub fn spawn_fire(ship: &Entity, rng: &mut impl Rng) -> Entity {
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
            points: Polygon(vec![]),
        },
        mass: 0.0,
        health: 0.0,
        max_health: 0.0,
        dam_res: 0.0,
        dam_ratio: 0.0,
        phys_res: 0.0,
        phys_ratio: 0.0,
        position: addtuple(
            ship.position,
            polar_to_affine(ship.orientation + PI, ship.hitbox.int_radius),
        ),
        velocity: {
            // Backward kick scales with ship speed so fire always ejects visually
            let ship_speed = (ship.velocity.x * ship.velocity.x + ship.velocity.y * ship.velocity.y).sqrt();
            let kick = ship_speed + FIRE_MIN_SPEED + rng.gen::<f64>() * (FIRE_MAX_SPEED - FIRE_MIN_SPEED);
            addtuple(
                ship.velocity,
                addtuple(
                    polar_to_affine(ship.orientation + PI, kick),
                    polar_to_affine(
                        rng.gen::<f64>() * 2.0 * PI,
                        rng.gen::<f64>() * FIRE_MAX_RANDOM,
                    ),
                ),
            )
        },
        orientation: 0.0,
        moment: 0.0,
        proper_time: ship.proper_time,
        hdr_exposure: randfloat(EXPLOSION_MIN_EXPOSURE, EXPLOSION_MAX_EXPOSURE, rng),
    }
}

pub fn polygon_asteroid(radius: f64, rng: &mut impl Rng) -> Polygon {
    let nb_sides =
        (ASTEROID_POLYGON_MIN_SIDES as f64).max(ASTEROID_POLYGON_SIZE_RATIO * radius) as i32;
    let mut points = Vec::new();

    for n in 1..=nb_sides {
        let angle = 2.0 * PI * n as f64 / nb_sides as f64;
        let distance = radius * randfloat(ASTEROID_POLYGON_MIN, ASTEROID_POLYGON_MAX, rng);
        points.push((angle, distance));
    }

    Polygon(points)
}

pub fn spawn_asteroid(pos: Vec2, vel: Vec2, radius: f64, rng: &mut impl Rng) -> Entity {
    let shape = polygon_asteroid(radius, rng);
    let color = (
        randfloat(ASTEROID_MIN_LUM, ASTEROID_MAX_LUM, rng),
        randfloat(ASTEROID_MIN_LUM, ASTEROID_MAX_LUM, rng),
        randfloat(ASTEROID_MIN_LUM, ASTEROID_MAX_LUM, rng),
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
            points: shape,
        },
        mass: PI * carre(radius) * ASTEROID_DENSITY,
        health: ASTEROID_MASS_HEALTH * PI * carre(radius) * ASTEROID_DENSITY + ASTEROID_MIN_HEALTH,
        max_health: ASTEROID_MASS_HEALTH * PI * carre(radius) * ASTEROID_DENSITY
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
    let radius = randfloat(ASTEROID_MIN_SPAWN_RADIUS, ASTEROID_MAX_SPAWN_RADIUS, rng);
    let pos = random_out_of_screen(radius, phys_w, phys_h, rng);
    let vel_angle = rng.gen::<f64>() * 2.0 * PI;
    let vel_magnitude = randfloat(
        ASTEROID_MIN_VELOCITY,
        ASTEROID_MAX_VELOCITY + ASTEROID_STAGE_VELOCITY * stage as f64,
        rng,
    );
    let vel = polar_to_affine(vel_angle, vel_magnitude);
    spawn_asteroid(pos, vel, radius, rng)
}

/// Create a single fragment from a parent asteroid
pub fn frag_asteroid(parent: &Entity, rng: &mut impl Rng) -> Entity {
    // Start with a fresh asteroid at parent's position/velocity/radius
    let mut fragment = spawn_asteroid(
        parent.position,
        parent.velocity,
        parent.hitbox.int_radius,
        rng,
    );

    let orientation = rng.gen::<f64>() * 2.0 * PI;
    let new_radius = randfloat(FRAGMENT_MIN_SIZE, FRAGMENT_MAX_SIZE, rng)
        * fragment.hitbox.int_radius;

    // Regenerate polygon for new size
    let new_shape = polygon_asteroid(new_radius, rng);

    // Offset position from parent center
    fragment.position = addtuple(
        fragment.position,
        polar_to_affine(orientation, fragment.hitbox.int_radius - new_radius),
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
    fragment.mass = PI * carre(new_radius) * ASTEROID_DENSITY;
    fragment.health = ASTEROID_MASS_HEALTH * fragment.mass + ASTEROID_MIN_HEALTH;
    fragment.max_health = fragment.health;

    // Add random velocity scatter
    fragment.velocity = addtuple(
        fragment.velocity,
        polar_to_affine(
            orientation,
            randfloat(FRAGMENT_MIN_VELOCITY, FRAGMENT_MAX_VELOCITY, rng),
        ),
    );

    // Adjust HDR exposure randomly
    fragment.hdr_exposure *= randfloat(FRAGMENT_MIN_EXPOSURE, FRAGMENT_MAX_EXPOSURE, rng);

    fragment
}

/// Spawn fragment_number fragments for each dead entity in source, appending to dest.
pub fn spawn_n_frags(
    source: &[Entity],
    dest: &mut Vec<Entity>,
    fragment_number: i32,
    rng: &mut impl Rng,
) {
    let dead: Vec<&Entity> = source.iter().filter(|e| e.health <= 0.0).collect();
    for _ in 0..fragment_number {
        for parent in &dead {
            dest.push(frag_asteroid(parent, rng));
        }
    }
}

pub fn random_out_of_screen(radius: f64, phys_w: f64, phys_h: f64, rng: &mut impl Rng) -> Vec2 {
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
        proximity: randfloat(STAR_MIN_PROX, STAR_MAX_PROX, rng).powf(4.0),
        lum: randfloat(STAR_MIN_LUM, STAR_MAX_LUM, rng),
    }
}

pub fn n_stars(n: i32, phys_w: f64, phys_h: f64, rng: &mut impl Rng) -> Vec<Star> {
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
    hypothenuse(soustuple(e.position, center)) < MAX_DIST
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
