//! Physics subsystem: broadphase collision grid, pair detection, and response orchestration.
//!
//! # Module layout
//! - `collision` — pure detection primitives (circle/point/polygon/entity)
//! - `response`  — collision response and damage application
//! - This module (`mod`) — grid infrastructure: GridEntry, CollisionGrid, make_grid,
//!   insert_into_grid, collect_pairs_for_cell, apply_collision_pairs,
//!   calculate_collision_tables, run_fragment_collisions.
//!
//! Note: the grid orchestration functions (calculate_collision_tables,
//! run_fragment_collisions, etc.) reference GameState and therefore live as
//! free functions in game.rs that delegate here. This avoids a circular
//! dependency between `physics` and `game`.

pub mod collision;
pub mod response;

// Re-export key types and functions for convenience
pub use collision::{collision_circles, collision_entities, collision_point, collisions_points};
pub use response::{
    consequences_collision, consequences_collision_frags, damage, phys_damage,
};

use crate::math_utils::Vec2;
use crate::parameters::*;

/// Identifies an entity by which list it lives in and its index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GridEntry {
    Object(usize),
    ObjectOos(usize),
    TooSmall(usize),
    TooSmallOos(usize),
    Fragment(usize),
    Ship,
}

pub type CollisionGrid = Vec<Vec<GridEntry>>;

pub fn make_grid() -> CollisionGrid {
    vec![Vec::new(); (WIDTH_COLLISION_TABLE * HEIGHT_COLLISION_TABLE) as usize]
}

/// Insert a slice of (entry, position) pairs into the collision grid.
/// Matches OCaml rev_filtertable: each entity goes into one cell (its center).
pub fn insert_into_grid(
    entries: &[(GridEntry, Vec2)],
    grid: &mut CollisionGrid,
    globals: &Globals,
) {
    let gw = WIDTH_COLLISION_TABLE as f64;
    let gh = HEIGHT_COLLISION_TABLE as f64;
    let jx = globals.current_jitter_coll_table.x;
    let jy = globals.current_jitter_coll_table.y;
    for &(entry, pos) in entries {
        let x2 = jx + gw * (pos.x + globals.phys_width) / (3.0 * globals.phys_width);
        let y2 = jy + gh * (pos.y + globals.phys_height) / (3.0 * globals.phys_height);
        if x2 < 0.0 || y2 < 0.0 || x2 >= gw || y2 >= gh {
            continue;
        }
        let xi = x2 as usize;
        let yi = y2 as usize;
        let idx = xi * HEIGHT_COLLISION_TABLE as usize + yi;
        grid[idx].push(entry);
    }
}
