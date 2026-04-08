use crate::math_utils::{magnitude, sub_vec, Vec2};
use crate::parameters::*;

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

#[derive(Clone, Debug)]
pub struct ExplosionObjectSideEffects {
    pub add_color: Option<(f64, f64, f64)>,
    pub exposure_multiplier: Option<f64>,
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

