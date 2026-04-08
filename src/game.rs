use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::color::*;
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;
use crate::spawning::*;
use crate::update::*;
pub use crate::physics::{collision_circles, collision_point};
use crate::physics::{
    collision_entities, consequences_collision, consequences_collision_frags, damage,
    insert_into_grid, make_grid, CollisionGrid, GridEntry,
};
use crate::rendering::hud::render_hud;
use crate::rendering::world::{render_chunk, render_projectile, render_star_trail, render_visuals};
use crate::rendering::Renderer2D;

// ============================================================================
// GamepadState
// ============================================================================

/// Runtime state for gamepad input processing.
#[derive(serde::Serialize)]
pub struct GamepadState {
    /// Whether a gamepad is currently connected
    pub connected: bool,
    /// Drift compensation offset for left stick
    pub left_center_offset: Vec2,
    /// Drift compensation offset for right stick
    pub right_center_offset: Vec2,
    /// Timestamp when sticks last went idle (for drift recalibration)
    pub last_idle_time: f64,
    /// Smoothed visual aim angle for ship rendering (lags behind true orientation)
    pub visual_aim_angle: f64,
    /// Raw left stick axes after normalization [-1.0, 1.0], before dead zone
    pub left_stick_raw: Vec2,
    /// Raw right stick axes after normalization [-1.0, 1.0], before dead zone
    pub right_stick_raw: Vec2,
    /// Whether any gamepad button is currently pressed (for drift detection)
    pub any_button_pressed: bool,
    /// Whether left trigger is currently past the activation threshold (for edge detection)
    pub left_trigger_pressed: bool,
}

impl GamepadState {
    pub fn new() -> Self {
        Self {
            connected: false,
            left_center_offset: Vec2::ZERO,
            right_center_offset: Vec2::ZERO,
            last_idle_time: 0.0,
            visual_aim_angle: std::f64::consts::PI / 2.0, // Match ship's initial orientation
            left_stick_raw: Vec2::ZERO,
            right_stick_raw: Vec2::ZERO,
            any_button_pressed: false,
            left_trigger_pressed: false,
        }
    }
}

impl Default for GamepadState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// GameState
// ============================================================================

#[derive(serde::Serialize)]
pub struct GameState {
    pub score: i32,
    pub lives: i32,
    pub stage: i32,
    pub cooldown: f64,
    pub cooldown_tp: f64,
    pub last_health: f64,
    // Death / respawn state
    pub is_dead: bool,      // true while in mort() loop
    pub time_of_death: f64, // wall-clock time when ship died
    pub ship: Entity,
    pub objects: Vec<Entity>,
    pub objects_oos: Vec<Entity>,
    pub toosmall: Vec<Entity>,
    pub toosmall_oos: Vec<Entity>,
    pub fragments: Vec<Entity>,
    pub chunks: Vec<Entity>,
    pub chunks_oos: Vec<Entity>,
    pub chunks_explo: Vec<Entity>,
    pub projectiles: Vec<Entity>,
    pub explosions: Vec<Entity>,
    pub smoke: Vec<Entity>,
    pub smoke_oos: Vec<Entity>,
    pub sparks: Vec<Entity>,
    pub stars: Vec<Star>,
    #[serde(skip)]
    pub rng: SmallRng,
    /// Pause menu interactive buttons.
    #[serde(skip)]
    pub pause_menu: crate::pause_menu::PauseMenu,
    /// Left mouse button state — used for rising-edge click detection.
    pub mouse_button_down: bool,
    /// Gamepad input and processing state.
    pub gamepad: GamepadState,
}

impl GameState {
    pub fn new(globals: &Globals) -> Self {
        let mut rng = SmallRng::from_entropy();
        let mut ship = spawn_ship();
        ship.position = Vec2::new(
            globals.render.phys_width / 2.0,
            globals.render.phys_height / 2.0,
        );

        Self {
            score: 0,
            lives: SHIP_MAX_LIVES,
            stage: 0,
            cooldown: 0.0,
            cooldown_tp: 0.0,
            last_health: SHIP_MAX_HEALTH,
            is_dead: false,
            time_of_death: 0.0,
            ship,
            objects: Vec::new(),
            objects_oos: Vec::new(),
            toosmall: Vec::new(),
            toosmall_oos: Vec::new(),
            fragments: Vec::new(),
            chunks: Vec::new(),
            chunks_oos: Vec::new(),
            chunks_explo: Vec::new(),
            projectiles: Vec::new(),
            explosions: Vec::new(),
            smoke: Vec::new(),
            smoke_oos: Vec::new(),
            sparks: Vec::new(),
            stars: spawn_stars(
                globals.spawn.stars_nb,
                globals.render.phys_width,
                globals.render.phys_height,
                &mut rng,
            ),
            rng,
            pause_menu: crate::pause_menu::PauseMenu::new(),
            mouse_button_down: false,
            gamepad: GamepadState::new(),
        }
    }

    /// Create a new GameState with a fixed seed for deterministic simulation.
    pub fn new_with_seed(globals: &Globals, seed: u64) -> Self {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut ship = spawn_ship();
        ship.position = Vec2::new(
            globals.render.phys_width / 2.0,
            globals.render.phys_height / 2.0,
        );

        Self {
            score: 0,
            lives: SHIP_MAX_LIVES,
            stage: 0,
            cooldown: 0.0,
            cooldown_tp: 0.0,
            last_health: SHIP_MAX_HEALTH,
            is_dead: false,
            time_of_death: 0.0,
            ship,
            objects: Vec::new(),
            objects_oos: Vec::new(),
            toosmall: Vec::new(),
            toosmall_oos: Vec::new(),
            fragments: Vec::new(),
            chunks: Vec::new(),
            chunks_oos: Vec::new(),
            chunks_explo: Vec::new(),
            projectiles: Vec::new(),
            explosions: Vec::new(),
            smoke: Vec::new(),
            smoke_oos: Vec::new(),
            sparks: Vec::new(),
            stars: spawn_stars(
                globals.spawn.stars_nb,
                globals.render.phys_width,
                globals.render.phys_height,
                &mut rng,
            ),
            rng,
            pause_menu: crate::pause_menu::PauseMenu::new(),
            mouse_button_down: false,
            gamepad: GamepadState::new(),
        }
    }
}

// ============================================================================
// Color helpers
// ============================================================================

/// Convert a (r,v,b) color tuple to HdrColor
pub(crate) fn hdr(color: (f64, f64, f64)) -> HdrColor {
    HdrColor::new(color.0, color.1, color.2)
}

/// Convert an HDR color (already intensified) to renderer [u8; 4] RGBA
/// NOTE: kept as reference, no longer called in the render path (GPU post-process handles tonemapping)
#[allow(dead_code)]
pub(crate) fn to_rgba(color: HdrColor, globals: &Globals) -> [u8; 4] {
    rgb_of_hdr(
        color,
        &hdr(globals.exposure.add_color),
        &hdr(globals.exposure.mul_color),
        globals.exposure.game_exposure,
    )
}

/// Convert an HDR color to raw HDR [f32; 4] for GPU tonemapping.
/// Colors are in 0-255 HDR range (may exceed 255). Alpha is always 1.0 (opaque).
pub(crate) fn to_hdr_rgba(color: HdrColor) -> [f32; 4] {
    [color.r as f32, color.g as f32, color.b as f32, 1.0]
}

// damage, phys_damage, collision_* and consequences_collision* moved to crate::physics
// movement, lifecycle, particle, visual functions moved to crate::update

fn get_entity(state: &GameState, entry: GridEntry) -> &Entity {
    match entry {
        GridEntry::Object(i) => &state.objects[i],
        GridEntry::ObjectOos(i) => &state.objects_oos[i],
        GridEntry::TooSmall(i) => &state.toosmall[i],
        GridEntry::TooSmallOos(i) => &state.toosmall_oos[i],
        GridEntry::Fragment(i) => &state.fragments[i],
        GridEntry::Ship => &state.ship,
    }
}

fn get_entity_mut(state: &mut GameState, entry: GridEntry) -> &mut Entity {
    match entry {
        GridEntry::Object(i) => &mut state.objects[i],
        GridEntry::ObjectOos(i) => &mut state.objects_oos[i],
        GridEntry::TooSmall(i) => &mut state.toosmall[i],
        GridEntry::TooSmallOos(i) => &mut state.toosmall_oos[i],
        GridEntry::Fragment(i) => &mut state.fragments[i],
        GridEntry::Ship => &mut state.ship,
    }
}

// consequences_collision and consequences_collision_frags moved to crate::physics::response

/// Collect all colliding (e1, e2) pairs from two grid cell lists.
/// Matches OCaml: iterates list1 × list2; for same list (tab1==tab2) this gives both directions.
fn collect_pairs_for_cell(
    cell1: &[GridEntry],
    cell2: &[GridEntry],
    state: &GameState,
    precis: bool,
    globals: &Globals,
    pairs: &mut Vec<(GridEntry, GridEntry)>,
) {
    for &e1 in cell1 {
        for &e2 in cell2 {
            if e1 == e2 {
                continue; // same entity — matches OCaml's objet1 = objet2 check
            }
            let ent1 = get_entity(state, e1);
            let ent2 = get_entity(state, e2);
            if collision_entities(ent1, ent2, precis, globals.advanced_hitbox) {
                // advanced_hitbox stays top-level
                pairs.push((e1, e2));
            }
        }
    }
}

/// Apply physical collision consequences to all collected pairs.
fn apply_collision_pairs(
    pairs: &[(GridEntry, GridEntry)],
    state: &mut GameState,
    globals: &mut Globals,
) {
    for &(e1_ref, e2_ref) in pairs {
        let e1 = get_entity(state, e1_ref).clone();
        let e2 = get_entity(state, e2_ref).clone();
        let (new_e1, new_e2) = consequences_collision(e1, e2, globals);
        *get_entity_mut(state, e1_ref) = new_e1;
        *get_entity_mut(state, e2_ref) = new_e2;
    }
}

/// Run collision detection between two grids, applying consequences.
/// `extend=true`: also check adjacent cells (right, down, diagonal).
/// Matches OCaml calculate_collision_tables.
fn calculate_collision_tables(
    grid1: &CollisionGrid,
    grid2: &CollisionGrid,
    extend: bool,
    state: &mut GameState,
    globals: &mut Globals,
) {
    let w = WIDTH_COLLISION_TABLE as usize;
    let h = HEIGHT_COLLISION_TABLE as usize;
    let mut pairs: Vec<(GridEntry, GridEntry)> = Vec::new();

    // Same-cell pairs (always)
    for x in 0..w {
        for y in 0..h {
            let idx = x * h + y;
            collect_pairs_for_cell(&grid1[idx], &grid2[idx], state, true, globals, &mut pairs);
        }
    }

    // Adjacent-cell pairs (only when extend=true)
    if extend {
        for x in 0..w - 1 {
            for y in 0..h - 1 {
                let base = x * h + y;
                let right = base + h; // x+1, y
                let down = base + 1; // x, y+1
                let diag = base + h + 1; // x+1, y+1
                collect_pairs_for_cell(
                    &grid1[base],
                    &grid2[down],
                    state,
                    false,
                    globals,
                    &mut pairs,
                );
                collect_pairs_for_cell(
                    &grid1[base],
                    &grid2[right],
                    state,
                    false,
                    globals,
                    &mut pairs,
                );
                collect_pairs_for_cell(
                    &grid1[base],
                    &grid2[diag],
                    state,
                    false,
                    globals,
                    &mut pairs,
                );
            }
        }
    }

    apply_collision_pairs(&pairs, state, globals);
}

/// Repulse colliding fragment pairs.
/// Fragments NOT involved in any collision this frame are promoted to state.objects.
/// Matches OCaml: calculate_collisions_frags + promotion logic.
fn run_fragment_collisions(state: &mut GameState, globals: &Globals) {
    let n = state.fragments.len();
    let mut involved = vec![false; n];

    // Collect colliding pairs (by index)
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if collision_entities(
                &state.fragments[i],
                &state.fragments[j],
                false,
                globals.advanced_hitbox,
            ) {
                // advanced_hitbox stays top-level
                pairs.push((i, j));
                involved[i] = true;
                involved[j] = true;
            }
        }
    }

    // Apply repulsion to each pair
    for (i, j) in pairs {
        let f1 = state.fragments[i].clone();
        let f2 = state.fragments[j].clone();
        let (new_f1, new_f2) = consequences_collision_frags(f1, f2, globals);
        state.fragments[i] = new_f1;
        state.fragments[j] = new_f2;
    }

    // Promote non-colliding fragments to objects (they've "settled")
    let mut settled: Vec<Entity> = Vec::new();
    let mut still_colliding: Vec<Entity> = Vec::new();
    for (i, frag) in state.fragments.drain(..).enumerate() {
        if involved[i] {
            still_colliding.push(frag);
        } else {
            settled.push(frag);
        }
    }
    state.fragments = still_colliding;
    state.objects.extend(settled);
}

/// Main game update: movement, transfers, collisions, spawning, despawn.
/// Called each frame when not paused.
pub fn update_game(state: &mut GameState, globals: &mut Globals) {
    // Update observer proper time (for time dilation)
    globals.observer_proper_time = state.ship.proper_time; // observer_proper_time stays top-level

    // --- Smoke & chunk decay ---
    {
        let smoke_load = state.smoke.len() as f64 / PARTICLE_BUDGET_SMOKE as f64;
        let fade_multiplier = if smoke_load > PARTICLE_DEGRADATION_THRESHOLD {
            PARTICLE_DEGRADATION_FADE_MULTIPLIER
        } else {
            1.0
        };
        for s in state.smoke.iter_mut() {
            decay_smoke_multiplied(s, globals, fade_multiplier);
        }
        for s in state.smoke_oos.iter_mut() {
            decay_smoke_multiplied(s, globals, fade_multiplier);
        }
    }

    // --- Decay chunks (radius shrink) ---
    // OCaml formula: radius -= observer_proper_time * game_speed * decay_rate * dt / chunk.proper_time
    {
        let dt = globals.dt();
        let gs = globals.time.game_speed;
        let opt = globals.observer_proper_time;
        let chunk_load = state.chunks.len() as f64 / PARTICLE_BUDGET_CHUNKS as f64;
        let chunk_fade_multiplier = if chunk_load > PARTICLE_DEGRADATION_THRESHOLD {
            PARTICLE_DEGRADATION_FADE_MULTIPLIER
        } else {
            1.0
        };
        for c in state.chunks.iter_mut() {
            c.visuals.radius -=
                opt * gs * CHUNK_RADIUS_DECAY * dt * chunk_fade_multiplier / c.proper_time;
        }
        for c in state.chunks_oos.iter_mut() {
            c.visuals.radius -=
                opt * gs * CHUNK_RADIUS_DECAY * dt * chunk_fade_multiplier / c.proper_time;
        }
        for c in state.chunks_explo.iter_mut() {
            c.visuals.radius -= opt * gs * CHUNK_EXPLO_RADIUS_DECAY * dt / c.proper_time;
        }
    }

    // Remove dead/negative-radius smoke
    state
        .smoke
        .retain(|s| s.visuals.radius > 0.0 && s.hdr_exposure > 0.001);
    state
        .smoke_oos
        .retain(|s| s.visuals.radius > 0.0 && s.hdr_exposure > 0.001);

    // --- Spawn explosions from dead projectiles ---
    // Previous explosions → smoke (before overwriting)
    if globals.visual.smoke_enabled {
        state.smoke.append(&mut state.explosions);
    } else {
        state.explosions.clear();
    }

    // Spawn new explosions from dead projectiles (health < 0)
    let dead_projectile_explosions: Vec<Entity> = state
        .projectiles
        .iter()
        .filter(|p| p.health < 0.0)
        .map(|p| spawn_explosion(p, &mut state.rng))
        .collect();
    state.explosions.extend(dead_projectile_explosions);

    // Spawn explosions from dead asteroids/toosmall/fragments → add to smoke list
    {
        let dead_objects: Vec<Entity> = state
            .objects
            .iter()
            .chain(state.objects_oos.iter())
            .chain(state.toosmall.iter())
            .chain(state.toosmall_oos.iter())
            .chain(state.fragments.iter())
            .filter(|e| crate::objects::is_dead(e))
            .cloned()
            .collect();
        for obj in &dead_objects {
            let (explo, side_effects) = spawn_explosion_object(
                obj,
                globals.visual.flashes_enabled,
                globals.visual.variable_exposure,
                FLASHES_SATURATE,
                FLASHES_EXPLOSION,
                FLASHES_NORMAL_MASS,
                &mut state.rng,
            );
            if let Some(ac) = side_effects.add_color {
                globals.exposure.add_color = (
                    globals.exposure.add_color.0 + ac.0,
                    globals.exposure.add_color.1 + ac.1,
                    globals.exposure.add_color.2 + ac.2,
                );
            }
            if let Some(em) = side_effects.exposure_multiplier {
                globals.exposure.game_exposure *= em;
            }
            state.smoke.push(explo);
        }
    }

    // Chunk explosions (chunks_explo → explosions)
    if !globals.time.pause {
        let explo_chunks: Vec<Entity> = state
            .chunks_explo
            .iter()
            .map(|c| {
                let (explo, se) = spawn_chunk_explosion(
                    c,
                    globals.visual.flashes_enabled,
                    FLASHES_SATURATE,
                    FLASHES_EXPLOSION,
                    FLASHES_NORMAL_MASS,
                    &mut state.rng,
                );
                if let Some(ac) = se.add_color {
                    globals.exposure.add_color = (
                        globals.exposure.add_color.0 + ac.0,
                        globals.exposure.add_color.1 + ac.1,
                        globals.exposure.add_color.2 + ac.2,
                    );
                }
                if let Some(em) = se.exposure_multiplier {
                    globals.exposure.game_exposure *= em;
                }
                explo
            })
            .collect();
        state.explosions.extend(explo_chunks);
    }

    // game_speed slowdown per explosion
    let nb_explo = state.explosions.len();
    globals.time.game_speed *= RATIO_TIME_EXPLOSION.powi(nb_explo as i32);

    // --- Projectile inertia ---
    for p in state.projectiles.iter_mut() {
        apply_inertia(p, globals);
    }

    // --- Explosion inertia (one frame entities) ---
    for e in state.explosions.iter_mut() {
        apply_inertia(e, globals);
    }

    // --- Filter dead or OOS projectiles (projectiles don't wrap, they despawn) ---
    state.projectiles.retain(|p| {
        p.health >= 0.0 && {
            let x = p.position.x;
            let y = p.position.y;
            x >= -globals.render.phys_width
                && x <= 2.0 * globals.render.phys_width
                && y >= -globals.render.phys_height
                && y <= 2.0 * globals.render.phys_height
        }
    });

    // --- Cooldown tick ---
    if state.cooldown > 0.0 {
        state.cooldown -= globals.time.game_speed * globals.dt();
    }
    if state.cooldown_tp > 0.0 {
        state.cooldown_tp -= globals.time.game_speed * globals.dt();
    }

    // --- Inertia (position update) ---
    apply_inertia(&mut state.ship, globals);
    apply_inertia_all(&mut state.objects, globals);
    apply_inertia_all(&mut state.objects_oos, globals);
    apply_inertia_all(&mut state.toosmall, globals);
    apply_inertia_all(&mut state.toosmall_oos, globals);
    apply_inertia_all(&mut state.fragments, globals);
    apply_inertia_all(&mut state.chunks, globals);
    apply_inertia_all(&mut state.chunks_oos, globals);
    apply_inertia_all(&mut state.chunks_explo, globals);

    // --- Rotation (moment update) ---
    apply_angular_momentum(&mut state.ship, globals);
    apply_angular_momentum_all(&mut state.objects, globals);
    apply_angular_momentum_all(&mut state.objects_oos, globals);
    apply_angular_momentum_all(&mut state.toosmall, globals);
    apply_angular_momentum_all(&mut state.toosmall_oos, globals);
    apply_angular_momentum_all(&mut state.fragments, globals);

    // --- Size classification: move too-small asteroids ---
    let small_objs = state
        .objects
        .extract_if(.., |e| crate::update::too_small(e))
        .collect::<Vec<_>>();
    state.toosmall.extend(small_objs);
    let small_frags = state
        .fragments
        .extract_if(.., |e| crate::update::too_small(e))
        .collect::<Vec<_>>();
    state.toosmall.extend(small_frags);

    // --- OOS transfers ---
    transfer_oos(&mut state.objects, &mut state.objects_oos, globals);
    transfer_oos(&mut state.toosmall, &mut state.toosmall_oos, globals);
    transfer_oos(&mut state.chunks, &mut state.chunks_oos, globals);

    // === Collision grids ===
    let mut grid_objects = make_grid();
    let mut grid_toosmall = make_grid();
    let mut grid_other = make_grid();
    let mut grid_frag = make_grid();

    let mut entries_obj: Vec<(GridEntry, Vec2)> = state
        .objects
        .iter()
        .enumerate()
        .map(|(i, e)| (GridEntry::Object(i), e.position))
        .collect();
    entries_obj.extend(
        state
            .objects_oos
            .iter()
            .enumerate()
            .map(|(i, e)| (GridEntry::ObjectOos(i), e.position)),
    );

    let mut entries_small: Vec<(GridEntry, Vec2)> = state
        .toosmall
        .iter()
        .enumerate()
        .map(|(i, e)| (GridEntry::TooSmall(i), e.position))
        .collect();
    entries_small.extend(
        state
            .toosmall_oos
            .iter()
            .enumerate()
            .map(|(i, e)| (GridEntry::TooSmallOos(i), e.position)),
    );

    // Note: explosions live exactly one frame; we include them in grid_other for collision
    // but they're not tracked by GridEntry index (they can't be mutated via get_entity_mut).
    // OCaml: other_ref = ship :: explosions @ projectiles — explosions damage via mass.
    // We handle explosion→asteroid damage separately below after grid collision.
    let entries_other: Vec<(GridEntry, Vec2)> = vec![(GridEntry::Ship, state.ship.position)];

    let entries_frag: Vec<(GridEntry, Vec2)> = state
        .fragments
        .iter()
        .enumerate()
        .map(|(i, e)| (GridEntry::Fragment(i), e.position))
        .collect();

    insert_into_grid(&entries_obj, &mut grid_objects, globals);
    insert_into_grid(&entries_small, &mut grid_toosmall, globals);
    insert_into_grid(&entries_other, &mut grid_other, globals);
    insert_into_grid(&entries_frag, &mut grid_frag, globals);

    // === Collision detection ===
    // Asteroid vs asteroid (extend=true)
    calculate_collision_tables(&grid_objects, &grid_objects, true, state, globals);
    // Asteroid vs toosmall (extend=false)
    calculate_collision_tables(
        &grid_objects.clone(),
        &grid_toosmall.clone(),
        false,
        state,
        globals,
    );
    // Ship/other vs asteroid (extend=true)
    calculate_collision_tables(
        &grid_other.clone(),
        &grid_objects.clone(),
        true,
        state,
        globals,
    );
    // Ship/other vs toosmall (extend=true)
    calculate_collision_tables(
        &grid_other.clone(),
        &grid_toosmall.clone(),
        true,
        state,
        globals,
    );
    // Ship/other vs fragment (extend=true)
    calculate_collision_tables(
        &grid_other.clone(),
        &grid_frag.clone(),
        true,
        state,
        globals,
    );

    // === Explosion damage to asteroids ===
    // Explosions are one-frame entities; we do a simple O(n*m) check here.
    // Scale damage by dt*60 so total damage per second is framerate-independent
    // (baseline: 60fps → scale=1.0; 120fps → scale=0.5 per frame but 2× frames = same total).
    let explo_dt_scale = globals.dt() * 60.0;
    for explo in &state.explosions {
        let explo_pos = explo.position;
        let explo_rad = explo.hitbox.ext_radius;
        let explo_mass = explo.mass * explo_dt_scale;
        for obj in state
            .objects
            .iter_mut()
            .chain(state.objects_oos.iter_mut())
            .chain(state.toosmall.iter_mut())
            .chain(state.toosmall_oos.iter_mut())
        {
            if collision_circles(explo_pos, explo_rad, obj.position, obj.hitbox.int_radius) {
                damage(obj, explo_mass, globals);
            }
        }
    }

    // === Explosion shockwave push ===
    // One-shot velocity impulse: no dt scaling needed (explosions are one-frame).
    // Linear falloff within SHOCKWAVE_RANGE_MULTIPLIER × blast radius.
    // Two passes: physics objects (Newtonian: impulse/mass) and particles (fixed impulse for visual flair).
    for explo in &state.explosions {
        let explo_pos = explo.position;
        let blast_range = explo.hitbox.ext_radius * SHOCKWAVE_RANGE_MULTIPLIER;
        if blast_range < 1e-6 {
            continue;
        }
        let explo_impulse = explo.mass * SHOCKWAVE_IMPULSE_SCALE;

        // Physics objects: Newtonian impulse / mass
        for obj in state
            .objects
            .iter_mut()
            .chain(state.objects_oos.iter_mut())
            .chain(state.toosmall.iter_mut())
            .chain(state.toosmall_oos.iter_mut())
            .chain(state.chunks.iter_mut())
            .chain(state.chunks_oos.iter_mut())
            .chain(state.chunks_explo.iter_mut())
        {
            let diff = obj.position - explo_pos;
            let center_dist = diff.length();
            if center_dist < 1e-6 {
                continue;
            }
            let effective_dist = (center_dist - obj.hitbox.avg_radius).max(0.0);
            if effective_dist >= blast_range {
                continue;
            }
            let strength = 1.0 - (effective_dist / blast_range);
            let impulse = explo_impulse * strength / obj.mass;
            let direction = diff * (1.0 / center_dist);
            obj.velocity += direction * impulse;
        }

        // Particles (zero-mass): fixed impulse scaled by SHOCKWAVE_PARTICLE_PUSH
        let particle_impulse = explo_impulse * SHOCKWAVE_PARTICLE_PUSH;
        for p in state
            .smoke
            .iter_mut()
            .chain(state.smoke_oos.iter_mut())
            .chain(state.sparks.iter_mut())
        {
            let diff = p.position - explo_pos;
            let center_dist = diff.length();
            if center_dist < 1e-6 {
                continue;
            }
            if center_dist >= blast_range {
                continue; // Particles have no meaningful radius, use center distance directly
            }
            let strength = 1.0 - (center_dist / blast_range);
            let direction = diff * (1.0 / center_dist);
            p.velocity += direction * (particle_impulse * strength);
        }
    }

    // === Projectile damage to asteroids + self-kill on hit ===
    for proj in state.projectiles.iter_mut() {
        let proj_pos = proj.position;
        let proj_rad = proj.hitbox.ext_radius;
        let mut hit = false;
        for obj in state
            .objects
            .iter_mut()
            .chain(state.objects_oos.iter_mut())
            .chain(state.toosmall.iter_mut())
            .chain(state.toosmall_oos.iter_mut())
        {
            if collision_circles(proj_pos, proj_rad, obj.position, obj.hitbox.int_radius) {
                hit = true;
            }
        }
        if hit {
            proj.health = -1.0; // kill projectile
        }
    }

    // === Destroyed entity accounting (asteroids + fragments) ===
    let nb_destroyed = state.objects.iter().filter(|e| crate::objects::is_dead(e)).count()
        + state.objects_oos.iter().filter(|e| crate::objects::is_dead(e)).count()
        + state.toosmall.iter().filter(|e| crate::objects::is_dead(e)).count()
        + state.toosmall_oos.iter().filter(|e| crate::objects::is_dead(e)).count()
        + state.fragments.iter().filter(|e| crate::objects::is_dead(e)).count();
    globals.time.game_speed *= RATIO_TIME_DESTR_ASTEROID.powi(nb_destroyed as i32);
    state.score += nb_destroyed as i32;
    globals.screenshake.shake_score += nb_destroyed as f64;

    // === Fragment vs fragment repulsion + promotion ===
    run_fragment_collisions(state, globals);

    // --- Fragmentation (spawn fragments from dead entities) ---
    spawn_fragments(
        &state.objects.clone(),
        &mut state.fragments,
        FRAGMENT_NUMBER,
        &mut state.rng,
    );
    spawn_fragments(
        &state.toosmall.clone(),
        &mut state.fragments,
        FRAGMENT_NUMBER,
        &mut state.rng,
    );
    spawn_fragments(
        &state.fragments.clone(),
        &mut state.fragments,
        FRAGMENT_NUMBER,
        &mut state.rng,
    );

    // --- Move chunks out of fragments ---
    let new_chunks = state
        .fragments
        .extract_if(.., |e| ischunk(e))
        .collect::<Vec<_>>();
    state.chunks.extend(new_chunks);

    // --- Recenter (wrap positions) ---
    wrap_entities(&mut state.objects, globals);
    wrap_entities(&mut state.toosmall, globals);
    wrap_entities(&mut state.objects_oos, globals);
    wrap_entities(&mut state.toosmall_oos, globals);
    wrap_entities(&mut state.fragments, globals);

    // --- Spawning ---
    if globals.spawn.time_since_last_spawn > TIME_SPAWN_ASTEROID {
        globals.spawn.time_since_last_spawn = 0.0;

        let nb_asteroids_stage = ASTEROID_MIN_NB + ASTEROID_STAGE_NB * state.stage;
        if globals.spawn.current_stage_asteroids >= nb_asteroids_stage {
            // Advance to next stage
            state.stage += 1;
            globals.spawn.current_stage_asteroids = 0;

            // Pick new random stage colors (matches OCaml)
            let new_col = (
                rand_range(RAND_MIN_LUM, RAND_MAX_LUM, &mut state.rng),
                rand_range(RAND_MIN_LUM, RAND_MAX_LUM, &mut state.rng),
                rand_range(RAND_MIN_LUM, RAND_MAX_LUM, &mut state.rng),
            );
            let new_hdr = hdr(new_col);
            globals.exposure.mul_base = {
                let c = saturate(intensify(new_hdr, 1.0), FILTER_SATURATION);
                (c.r, c.g, c.b)
            };
            globals.visual.space_color_goal = {
                let c = saturate(intensify(new_hdr, 10.0), SPACE_SATURATION);
                (c.r, c.g, c.b)
            };
            globals.visual.star_color_goal = {
                let c = saturate(intensify(new_hdr, 100.0), STAR_SATURATION);
                (c.r, c.g, c.b)
            };
        }

        // Spawn one asteroid
        state.objects_oos.push(spawn_random_asteroid(
            state.stage,
            globals.render.phys_width,
            globals.render.phys_height,
            &mut state.rng,
        ));
        globals.spawn.current_stage_asteroids += 1;
    }

    let elapsed =
        (globals.time.time_current_frame - globals.time.time_last_frame) * globals.time.game_speed;
    globals.spawn.time_since_last_spawn += elapsed;

    // --- Despawn ---
    despawn(state, globals);

    // --- Ship auto-regeneration ---
    if AUTOREGEN && state.ship.health > 0.0 && state.ship.health < SHIP_MAX_HEALTH {
        state.ship.health += AUTOREGEN_HEALTH * globals.time.game_speed * globals.dt();
        state.ship.health = state.ship.health.min(SHIP_MAX_HEALTH);
    }

    // --- Lagged health (orange bar) ---
    // Exponential lag: last_health chases ship.health (matches OCaml affiche_hud line 769)
    {
        let target = state.ship.health.max(0.0);
        state.last_health = target
            + exp_decay(
                state.last_health - target,
                0.5,
                globals.observer_proper_time,
                globals.time.game_speed,
                globals.time.time_last_frame,
                globals.time.time_current_frame,
                state.ship.proper_time,
            );
    }

    // --- Ship death handling ---
    // Step 1: Detect death entry (first time health < 0, not already in mort() phase)
    if state.ship.health < 0.0 && !state.is_dead {
        state.is_dead = true;
        globals.time.time_of_death = globals.time.time_current_frame;
        state.lives -= 1;

        // Chunk explosion at death
        if globals.visual.chunks_enabled {
            let death_color = (1500.0, 400.0, 200.0);
            let new_chunks =
                spawn_n_chunks(&state.ship, NB_CHUNKS_EXPLO, death_color, &mut state.rng);
            state.chunks_explo.extend(new_chunks);
        }

        // Death VFX: screenshake + big red flash + game speed slowdown
        globals.screenshake.game_screenshake += SCREENSHAKE_DEATH;
        if globals.visual.flashes_enabled {
            let death_flash = intensify(HdrColor::new(1000.0, 0.0, 0.0), FLASHES_DEATH);
            globals.exposure.add_color = (
                globals.exposure.add_color.0 + death_flash.r,
                globals.exposure.add_color.1 + death_flash.g,
                globals.exposure.add_color.2 + death_flash.b,
            );
        }
        globals.time.game_speed *= RATIO_TIME_DEATH;
        globals.time.game_speed_target = GAME_SPEED_TARGET_DEATH;
        globals.exposure.game_exposure_target = GAME_EXPOSURE_TARGET_DEATH;

        // Clamp health so death entry does not re-trigger
        state.ship.health = -0.1;
    }

    // Step 2: Per-frame death fire — spawn burning explosion while in mort() phase
    if state.is_dead {
        let elapsed = (globals.time.time_current_frame - globals.time.time_last_frame)
            * globals.time.game_speed;
        let death_explo = spawn_explosion_death(&state.ship, elapsed, &mut state.rng);
        state.explosions.push(death_explo);
    }

    // Step 3: End the death phase when timer expires or early-exit condition met
    if state.is_dead {
        let t = globals.time.time_current_frame;
        let tod = globals.time.time_of_death;
        let timer_expired = t > tod + TIME_STAY_DEAD_MAX;
        let early_exit = t > tod + TIME_STAY_DEAD_MIN && state.ship.health < -100.0;

        if timer_expired || early_exit {
            state.is_dead = false;

            if state.lives <= 0 {
                // Game over: reset and pause
                *state = GameState::new(globals);
                globals.time.pause = true;
                globals.time.game_speed_target = GAME_SPEED_TARGET_BOUCLE;
                globals.exposure.game_exposure_target = GAME_EXPOSURE_TARGET_BOUCLE;
            } else {
                // Second chunk burst
                if globals.visual.chunks_enabled {
                    let death_color = (1500.0, 400.0, 200.0);
                    let new_chunks =
                        spawn_n_chunks(&state.ship, NB_CHUNKS_EXPLO, death_color, &mut state.rng);
                    state.chunks_explo.extend(new_chunks);
                }

                // Second screenshake + red flash
                globals.screenshake.game_screenshake += SCREENSHAKE_DEATH;
                if globals.visual.flashes_enabled {
                    let death_flash = intensify(HdrColor::new(1000.0, 0.0, 0.0), FLASHES_DEATH);
                    globals.exposure.add_color = (
                        globals.exposure.add_color.0 + death_flash.r,
                        globals.exposure.add_color.1 + death_flash.g,
                        globals.exposure.add_color.2 + death_flash.b,
                    );
                }
                globals.time.game_speed *= RATIO_TIME_DEATH;

                // Respawn ship and restore normal targets
                state.ship = spawn_ship();
                globals.time.game_speed_target = GAME_SPEED_TARGET_BOUCLE;
                globals.exposure.game_exposure_target = GAME_EXPOSURE_TARGET_BOUCLE;
            }
        }
    }

    // --- Visual aim smoothing ---
    let dt_game = globals.time.time_current_frame - globals.time.time_last_frame;
    update_visual_aim(&mut state.gamepad, state.ship.orientation, dt_game);

    // --- Particle budget enforcement ---
    enforce_particle_budgets(state);

    // --- Camera update: translate all entities to keep ship centred ---
    crate::camera::update_camera(state, globals);
}

/// Render a complete frame: background, stars, chunks, asteroids, ship
pub fn render_frame(
    state: &mut GameState,
    globals: &mut Globals,
    renderer: &mut Renderer2D,
    mouse_sx: f64,
    mouse_sy: f64,
    mouse_down: bool,
) {
    let (w, h) = (renderer.width as i32, renderer.height as i32);

    // Background
    // Emit HDR value (exposure baked in); GPU post-process applies add_color/mul_color/redirect
    let bg = intensify(
        hdr(globals.visual.space_color),
        globals.exposure.game_exposure,
    );
    let bg_color = [bg.r as f32, bg.g as f32, bg.b as f32, 1.0];
    renderer.fill_rect(0, 0, w, h, bg_color);

    // Stars
    for star in &state.stars {
        render_star_trail(star, renderer, globals, &mut state.rng);
    }

    // Smoke
    for s in &state.smoke {
        render_visuals(s, Vec2::ZERO, renderer, globals);
    }

    // Chunks
    for chunk in &state.chunks {
        render_chunk(chunk, renderer, globals);
    }

    // TODO: Sparkles (collision light-trails) — will be added when collision system creates them

    // Projectiles
    for p in &state.projectiles {
        render_projectile(p, renderer, globals, &mut state.rng);
    }

    // Fragments
    for entity in &state.fragments {
        render_visuals(entity, Vec2::ZERO, renderer, globals);
    }

    // Toosmall
    for entity in &state.toosmall {
        render_visuals(entity, Vec2::ZERO, renderer, globals);
    }

    // Asteroids
    for entity in &state.objects {
        render_visuals(entity, Vec2::ZERO, renderer, globals);
    }

    // Explosions — in front of asteroids, behind ship
    for e in &state.explosions {
        render_visuals(e, Vec2::ZERO, renderer, globals);
    }

    // Ship — topmost game object
    let true_aim = state.ship.orientation;
    state.ship.orientation = state.gamepad.visual_aim_angle;
    render_visuals(&state.ship, Vec2::ZERO, renderer, globals);
    state.ship.orientation = true_aim;

    // HUD overlay — skip when paused (matches OCaml behavior)
    if !globals.time.pause {
        render_hud(state, globals, renderer);
    }

    // Pause menu overlay
    if globals.time.pause {
        state.pause_menu.render(
            globals,
            renderer,
            &mut state.rng,
            mouse_sx,
            mouse_sy,
            mouse_down,
        );
    }
}

#[cfg(test)]
mod render_order_tests {
    #[test]
    fn render_layer_order_is_documented() {
        const BACKGROUND: u8 = 0;
        const STARS: u8 = 1;
        const SMOKE: u8 = 2;
        const CHUNKS: u8 = 3;
        const SPARKLES_PLACEHOLDER: u8 = 4;
        const PROJECTILES: u8 = 5;
        const FRAGMENTS: u8 = 6;
        const TOOSMALL: u8 = 7;
        const ASTEROIDS: u8 = 8;
        const EXPLOSIONS: u8 = 9;
        const SHIP: u8 = 10;
        assert!(BACKGROUND < STARS);
        assert!(STARS < SMOKE);
        assert!(SMOKE < CHUNKS);
        assert!(CHUNKS < SPARKLES_PLACEHOLDER);
        assert!(SPARKLES_PLACEHOLDER < PROJECTILES);
        assert!(PROJECTILES < FRAGMENTS);
        assert!(FRAGMENTS < TOOSMALL);
        assert!(TOOSMALL < ASTEROIDS);
        assert!(ASTEROIDS < EXPLOSIONS);
        assert!(EXPLOSIONS < SHIP);
    }
}
