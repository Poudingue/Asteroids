//! Parameters of the whole game
//!
//! This is a port of ml/parameters.ml to Rust.
//! Contains all game configuration constants and mutable global state.

use std::f64::consts::PI;

// ============================================================================
// Constants (Display Parameters)
// ============================================================================

// Effect of scanlines to imitate CRT monitors that projected the image line by line.
// Activating the animated_scanlines effect allows animation imitating interlaced videos,
// by activating one line out of two one image out of two, but it doesn't pass well
// because the image refresh cannot really be at exactly 60 with the OCaml engine.
// Test at your own risk.
pub const SCANLINES_PERIOD: i32 = 5;
pub const ANIMATED_SCANLINES: bool = true;

// Antialiasing jitter makes the render space "shake".
// This is a form of spatial dithering to compensate for the loss of precision
// due to rasterization when placing objects and drawing contours.
pub const DITHER_AA: bool = true;
// The power of jitter determines how much the rendering can shift.
// Set to 1 or less to avoid a blurring and visual fatigue effect.
pub const DITHER_POWER: f64 = 0.5; // As a ratio of pixel size
pub const DITHER_POWER_RADIUS: f64 = 0.5;

pub const FILTER_HALF_LIFE: f64 = 1.0;
pub const FILTER_SATURATION: f64 = 0.5;
pub const SPACE_HALF_LIFE: f64 = 1.0;

// ============================================================================
// Constants (Game Speed & Timing)
// ============================================================================

pub const GAME_SPEED_TARGET_PAUSE: f64 = 0.0; // Game speed when paused
pub const GAME_SPEED_TARGET_DEATH: f64 = 0.8; // Game speed after death
pub const GAME_SPEED_TARGET_BOUCLE: f64 = 1.0; // Default game speed

// The half_speed_change determines at what "speed" the game speed approaches game_speed_target (in half-life)
pub const HALF_SPEED_CHANGE: f64 = 0.1;

// Speed change ratios based on events
pub const RATIO_TIME_EXPLOSION: f64 = 0.99;
pub const RATIO_TIME_DESTR_ASTEROID: f64 = 0.95;
pub const RATIO_TIME_TP: f64 = 0.0;
pub const RATIO_TIME_DEATH: f64 = 0.5;

// Death timer
pub const TIME_STAY_DEAD_MIN: f64 = 1.0;
pub const TIME_STAY_DEAD_MAX: f64 = 5.0;

// Framerate limiting is available, but it seems that gettimeofday and the waiting of Unix.select
// are not precise enough for each frame to last just the right amount of time.
// My advice is not to activate it.
// The framerate requested in the assignment is 20.
// A higher framerate offers a better game experience: more reactive controls, better visual comfort,
// and more accurate physics. Of course, it's possible to change it below.
// (only has an effect with locked_framerate enabled)
pub const FRAMERATE_LIMIT: f64 = 300.0;
// The render framerate is used to determine the length of motion blur.
// Set it to your actual screen refresh rate,
// and shutter_speed controls the blur length like a real camera.
pub const FRAMERATE_RENDER: f64 = 60.0;

// Observer's proper time.
// In this case, we get that of the ship.
// This allows for Einsteinian relativity.
// TODO: Use it.
pub const OBSERVER_PROPER_TIME: f64 = 1.0; // As a ratio of "absolute" universe time

// ============================================================================
// Constants (Window Dimensions & Game Surface)
// ============================================================================

pub const WIDTH: i32 = 1920;
pub const HEIGHT: i32 = 1080;
pub const GAME_SURFACE: f64 = 30.0; // Determines the size of the game terrain.
pub const MAX_DIST: f64 = 20000.0;

// Dimensions of the physical space in which objects evolve.
// We ensure that the game surface is the same regardless of resolution.
// We maintain the ratio of the resolution for game dimensions.
// We have a game surface of 1,000,000 by default.
pub const PROJECTILE_NUMBER_DEFAULT: i32 = 1;

// Collision table dimensions
pub const WIDTH_COLLISION_TABLE: i32 = 15;
pub const HEIGHT_COLLISION_TABLE: i32 = 9;

// ============================================================================
// Constants (Advanced Graphics Parameters)
// ============================================================================

// Random colors per stage
pub const RAND_MIN_LUM: f64 = 0.5;
pub const RAND_MAX_LUM: f64 = 1.5;
pub const SPACE_SATURATION: f64 = 2.0;
pub const STAR_SATURATION: f64 = 8.0;

// Button colors (stored as u32: (r << 16) | (g << 8) | b)
pub const TRUECOLOR: u32 = (0 << 16) | (128 << 8) | 0; // rgb 0 128 0
pub const FALSECOLOR: u32 = (128 << 16) | (0 << 8) | 0; // rgb 128 0 0
pub const SLIDERCOLOR: u32 = (128 << 16) | (128 << 8) | 128; // rgb 128 128 128
pub const BUTTONFRAME: u32 = (64 << 16) | (64 << 8) | 64; // rgb 64 64 64

// Motion blur parameters
// Implemented correctly for bullets and stars,
// draws streaks behind other types of objects,
// but in an erratic way, so disabled by default.
pub const SHUTTER_SPEED: f64 = 1.0;

// ============================================================================
// Constants (Game Parameters)
// ============================================================================

// Direct controls do not control speed and momentum but directly position and rotation.
// The default values are those requested in the assignment.
// TODO: Implement correctly all control methods.

// Physical collision repulsion
pub const MIN_REPULSION: f64 = 100.0;
pub const MIN_BOUNCE: f64 = 1000.0;

// ============================================================================
// Constants (Asteroid Parameters)
// ============================================================================

pub const ASTEROID_MAX_SPAWN_RADIUS: f64 = 650.0; // Max asteroid size at spawn.
pub const ASTEROID_MIN_SPAWN_RADIUS: f64 = 350.0; // Min spawn size
pub const ASTEROID_MAX_MOMENT: f64 = 1.0; // Max rotation of an asteroid at spawn (in random direction)
pub const ASTEROID_MAX_VELOCITY: f64 = 2000.0; // Max velocity at spawn
pub const ASTEROID_MIN_VELOCITY: f64 = 1500.0; // Min velocity at spawn
pub const ASTEROID_STAGE_VELOCITY: f64 = 500.0; // Allows asteroids of more advanced stages to go faster
pub const ASTEROID_DENSITY: f64 = 1.0; // Used to determine asteroid mass based on its surface area
pub const ASTEROID_MIN_HEALTH: f64 = 50.0; // Avoids asteroids too fragile due to too low mass. Added to calculation.
pub const ASTEROID_MASS_HEALTH: f64 = 0.01; // Used to determine asteroid health based on its mass

// Damage parameters: phys = physical damage, ratio = damage multiplier, res = damage resistance (subtraction)
pub const ASTEROID_DAM_RATIO: f64 = 1.0; // Sensitivity to explosion damage
pub const ASTEROID_DAM_RES: f64 = 0.0; // Resistance to explosion damage
pub const ASTEROID_PHYS_RATIO: f64 = 1.0; // Sensitivity to physical impacts
pub const ASTEROID_PHYS_RES: f64 = 100.0; // Resistance to physical impacts

// Parameters for asteroid colors at birth
pub const ASTEROID_MIN_LUM: f64 = 40.0;
pub const ASTEROID_MAX_LUM: f64 = 120.0;
pub const ASTEROID_MIN_SATUR: f64 = 0.4;
pub const ASTEROID_MAX_SATUR: f64 = 0.5;

// Hitbox and polygonal visual parameters
pub const ASTEROID_POLYGON_MIN_SIDES: i32 = 7; // Minimum number of sides an asteroid can have
pub const ASTEROID_POLYGON_SIZE_RATIO: f64 = 0.02; // Determines the number of sides an asteroid will have for its hitbox and rendering
pub const ASTEROID_POLYGON_MIN: f64 = 1.0; // As a ratio of radius
pub const ASTEROID_POLYGON_MAX: f64 = 1.3; // As a ratio of radius

// Control of the number of asteroids appearing in each wave
pub const ASTEROID_MIN_NB: i32 = 2;
pub const ASTEROID_STAGE_NB: i32 = 1;
pub const ASTEROID_MIN_SIZE: f64 = 100.0;
pub const TIME_SPAWN_ASTEROID: f64 = 2.0; // seconds

// ============================================================================
// Constants (Fragment Parameters)
// ============================================================================

// Fragment characteristics. Mainly inherited from parents.
pub const FRAGMENT_MAX_VELOCITY: f64 = 2500.0; // Max velocity at spawn
pub const FRAGMENT_MIN_VELOCITY: f64 = 1500.0; // Min velocity at spawn
pub const FRAGMENT_MAX_SIZE: f64 = 0.7; // As a ratio of parent asteroid size
pub const FRAGMENT_MIN_SIZE: f64 = 0.4; // As a ratio of parent asteroid size
pub const FRAGMENT_MIN_EXPOSURE: f64 = 0.666;
pub const FRAGMENT_MAX_EXPOSURE: f64 = 1.5;
pub const FRAGMENT_NUMBER: i32 = 5;
pub const FRAGMENT_MIN_REPULSION: f64 = 100.0;
pub const FRAGMENT_MIN_BOUNCE: f64 = 1000.0;
pub const CHUNK_MAX_SIZE: f64 = 50.0;
pub const CHUNK_RADIUS_DECAY: f64 = 25.0; // For decay of particles without collisions

pub const NB_CHUNKS_EXPLO: i32 = 15;
pub const CHUNKS_EXPLO_MIN_RADIUS: f64 = 150.0;
pub const CHUNKS_EXPLO_MAX_RADIUS: f64 = 300.0;
pub const CHUNKS_EXPLO_MIN_SPEED: f64 = 10000.0;
pub const CHUNKS_EXPLO_MAX_SPEED: f64 = 20000.0;
pub const CHUNK_EXPLO_RADIUS_DECAY: f64 = 500.0;

// ============================================================================
// Constants (Ship Parameters)
// ============================================================================

// Auto-regeneration
pub const AUTOREGEN: bool = true;
pub const AUTOREGEN_HEALTH: f64 = 5.0; // Health regeneration per second

// Ship values
pub const SHIP_MAX_HEALTH: f64 = 100.0; // Health at spawn. Allows applying it to the physical model.
pub const SHIP_MAX_LIVES: i32 = 3; // Number of times the ship can respawn
pub const SHIP_DENSITY: f64 = 100.0; // For calculating ship mass, which impacts physics
pub const SHIP_RADIUS: f64 = 25.0; // For hitbox and rendering

// Damage and physical damage reduction
pub const SHIP_DAM_RATIO: f64 = 0.8;
pub const SHIP_DAM_RES: f64 = 10.0;
pub const SHIP_PHYS_RATIO: f64 = 0.005;
pub const SHIP_PHYS_RES: f64 = 0.0;
pub const SHIP_DEATH_MAX_MOMENTUM: f64 = 2.0;

// Movement controls
pub const SHIP_MAX_DEPL: f64 = 50.0; // In px/s. Useful if direct movement control.
pub const SHIP_MAX_ACCEL: f64 = 10000.0; // In px/s². Useful if acceleration control.
pub const SHIP_MAX_BOOST: f64 = 2000.0; // In px/s. Useful if boost control.
pub const SHIP_HALF_STOP: f64 = 10.0; // Time needed to lose half inertia.

// Rotation controls
pub const SHIP_MAX_TOURN: f64 = 4.0; // In radian/s
pub const SHIP_MAX_MOMENT: f64 = 0.5; // In radian/s²
pub const SHIP_MAX_TOURN_BOOST: f64 = 3.0; // In radian/s
pub const SHIP_MAX_ROTAT: f64 = PI / 6.0; // In radians
pub const SHIP_HALF_STOP_ROTAT: f64 = 0.2; // Time needed to lose half angular momentum

// Minimum time between random teleportations
pub const COOLDOWN_TP: f64 = 5.0;
pub const TP_TIME_INVIC: f64 = 1.0; // Invincibility time after tp. TODO: Implement

// ============================================================================
// Constants (Projectile Parameters)
// ============================================================================

// Regular projectile values
pub const PROJECTILE_HERIT_SPEED: bool = true;
pub const PROJECTILE_HEALTH: f64 = 0.0; // We consider death when health drops below zero. We have certainty here that the projectile will destroy itself.
pub const PROJECTILE_NUMBER_DEFAULT_VAL: i32 = 5;
pub const EXPLOSION_DAMAGES_PROJECTILE: f64 = 5000.0;

// Shotgun
pub const SHOTGUN_RECOIL: f64 = 1000.0;
pub const SHOTGUN_COOLDOWN: f64 = 0.3;
pub const SHOTGUN_MAX_SPEED: f64 = 15000.0;
pub const SHOTGUN_MIN_SPEED: f64 = 10000.0;
pub const SHOTGUN_DEVIATION: f64 = 0.3;
pub const SHOTGUN_RADIUS: f64 = 15.0;
pub const SHOTGUN_RADIUS_HITBOX: f64 = 50.0;
pub const SHOTGUN_NUMBER: i32 = 50;

// Sniper
pub const SNIPER_RECOIL: f64 = 10000.0;
pub const SNIPER_COOLDOWN: f64 = 1.0;
pub const SNIPER_MAX_SPEED: f64 = 20000.0;
pub const SNIPER_MIN_SPEED: f64 = 15000.0;
pub const SNIPER_DEVIATION: f64 = 0.0;
pub const SNIPER_RADIUS: f64 = 25.0;
pub const SNIPER_RADIUS_HITBOX: f64 = 75.0;
pub const SNIPER_NUMBER: i32 = 1;

// Machine gun
pub const MACHINE_RECOIL: f64 = 10.0;
pub const MACHINE_COOLDOWN: f64 = 0.01;
pub const MACHINE_MAX_SPEED: f64 = 10000.0;
pub const MACHINE_MIN_SPEED: f64 = 8000.0;
pub const MACHINE_DEVIATION: f64 = 0.2;
pub const MACHINE_RADIUS: f64 = 10.0;
pub const MACHINE_RADIUS_HITBOX: f64 = 25.0;
pub const MACHINE_NUMBER: i32 = 1;

// ============================================================================
// Constants (Explosion Parameters)
// ============================================================================

pub const EXPLOSION_MAX_RADIUS: f64 = 250.0;
pub const EXPLOSION_MIN_RADIUS: f64 = 200.0;
pub const EXPLOSION_MIN_EXPOSURE: f64 = 0.4; // Determines max and min brightness of explosions at spawn
pub const EXPLOSION_MAX_EXPOSURE: f64 = 1.3;

pub const EXPLOSION_DAMAGES_OBJET: f64 = 50.0;
pub const EXPLOSION_DAMAGES_CHUNK: f64 = 150.0;
pub const EXPLOSION_DAMAGES_DEATH: f64 = 50.0; // per second

// For explosions inheriting from an object
pub const EXPLOSION_RATIO_RADIUS: f64 = 2.0;
pub const EXPLOSION_DEATH_MAX_RADIUS: f64 = 150.0;
pub const EXPLOSION_DEATH_MIN_RADIUS: f64 = 100.0;
pub const EXPLOSION_SATURATE: f64 = 10.0;
pub const EXPLOSION_MIN_EXPOSURE_HERITATE: f64 = 2.0;
pub const EXPLOSION_MAX_EXPOSURE_HERITATE: f64 = 6.0;

// ============================================================================
// Constants (Muzzle Flash Parameters)
// ============================================================================

pub const MUZZLE_RATIO_RADIUS: f64 = 3.0;
pub const MUZZLE_RATIO_SPEED: f64 = 0.05;

// ============================================================================
// Constants (Fire/Thrust Parameters)
// ============================================================================

pub const FIRE_MAX_RANDOM: f64 = 250.0;
pub const FIRE_MIN_SPEED: f64 = 800.0;
pub const FIRE_MAX_SPEED: f64 = 1500.0;
pub const FIRE_RATIO_RADIUS: f64 = 1.4;

// ============================================================================
// Constants (Smoke Parameters)
// ============================================================================

pub const SMOKE_HALF_COL: f64 = 0.3; // Speed of color decay
pub const SMOKE_HALF_RADIUS: f64 = 0.5; // Speed of radius decay
pub const SMOKE_RADIUS_DECAY: f64 = 5.0; // Reduction of smoke particle radius
pub const SMOKE_MAX_SPEED: f64 = 400.0; // Random speed in random direction of smoke

// ============================================================================
// Constants (Star Parameters)
// ============================================================================

pub const STAR_MIN_PROX: f64 = 0.3; // Min proximity of stars. 0 = star at infinity, appears immobile regardless of movement.
pub const STAR_MAX_PROX: f64 = 0.9; // Max proximity. 1 = same depth as the ship
pub const STAR_PROX_LUM: f64 = 5.0; // To add brightness to closer stars
pub const STAR_MIN_LUM: f64 = 0.0;
pub const STAR_MAX_LUM: f64 = 4.0;
pub const STAR_RAND_LUM: f64 = 2.0; // Twinkling effect of stars
pub const STARS_NB_DEFAULT: i32 = 100;

// ============================================================================
// Constants (Camera Parameters)
// ============================================================================

// The predictive camera orients the camera towards where the ship is going,
// to keep it as much as possible in the center of the screen.
pub const CAMERA_PREDICTION: f64 = 2.5; // In seconds of ship movement into the future.
pub const CAMERA_HALF_DEPL: f64 = 1.5; // Time to move halfway to the camera target
pub const CAMERA_RATIO_OBJECTS: f64 = 0.4; // Camera moves to average position of objects, weighted by mass and distance squared
pub const CAMERA_RATIO_VISION: f64 = 0.25; // Camera moves to where the ship is looking, at distance = ratio × terrain width
pub const CAMERA_START_BOUND: f64 = 0.3; // As ratio of screen size: distance from edge where camera starts to recenter
pub const CAMERA_MAX_FORCE: f64 = 3.0; // As ratio of screen size: speed applied to camera to recenter if edge is reached

// ============================================================================
// Constants (Screen Shake Parameters)
// ============================================================================

// Screen shake adds trembling effects with intensity depending on events.
pub const SCREENSHAKE_SMOOTH: bool = true; // Allows smoother, more realistic screen shake. Sort of low-pass filter on movements.
pub const SCREENSHAKE_SMOOTHNESS: f64 = 0.8; // 0 = no change, 0.5 = average, 1 = infinite smoothing, screen shake suppressed.
pub const SCREENSHAKE_TIR_RATIO: f64 = 400.0;
pub const SCREENSHAKE_DEATH: f64 = 6000.0;
pub const SCREENSHAKE_DAM_RATIO: f64 = 0.005;
pub const SCREENSHAKE_PHYS_RATIO: f64 = 0.005;
pub const SCREENSHAKE_PHYS_MASS: f64 = 100000.0; // "Normal" screen shake mass. Lighter objects cause less, heavier objects cause more.
pub const SCREENSHAKE_HALF_LIFE: f64 = 0.1;

// Score shake using score increase to shake the numbers
pub const SHAKE_SCORE_RATIO: f64 = 0.2;
pub const SHAKE_STRENGTH: f64 = 0.01;
pub const SHAKE_SCORE_HALF_LIFE: f64 = 0.2;

// ============================================================================
// Constants (Variable Exposure Parameters)
// ============================================================================

// Variable exposure allows brightness variations based on events
pub const EXPOSURE_RATIO_DAMAGE: f64 = 0.995;
pub const EXPOSURE_TIR: f64 = 0.98;
pub const EXPOSURE_RATIO_EXPLOSIONS: f64 = 0.99;
pub const EXPOSURE_HALF_LIFE: f64 = 0.5;
pub const GAME_EXPOSURE_TARGET_DEATH: f64 = 1.5;
pub const GAME_EXPOSURE_TARGET_BOUCLE: f64 = 2.0;
pub const GAME_EXPOSURE_TP: f64 = 0.25;

// ============================================================================
// Constants (Flash Parameters)
// ============================================================================

// Bright flashes during events
pub const FLASHES_DAMAGE: f64 = 0.0;
pub const FLASHES_EXPLOSION: f64 = 0.02;
pub const FLASHES_SATURATE: f64 = 10.0;
pub const FLASHES_NORMAL_MASS: f64 = 100000.0;
pub const FLASHES_TIR: f64 = 1.0;
pub const FLASHES_TELEPORT: f64 = 100.0;
pub const FLASHES_DEATH: f64 = 200.0;
pub const FLASHES_HALF_LIFE: f64 = 0.01;

// ============================================================================
// Global Mutable State
// ============================================================================

/// Global game state that changes during execution.
/// This corresponds to all the `let ... = ref ...` bindings in the OCaml code.
pub struct Globals {
    // Time-related fields
    pub game_speed: f64,
    pub game_speed_target: f64,
    pub time_last_frame: f64,
    pub time_current_frame: f64,
    pub time_of_death: f64,
    pub pause: bool,
    pub restart: bool,
    pub quit: bool,

    // Visual fields
    pub game_exposure: f64,
    pub game_exposure_target: f64,
    pub add_color: (f64, f64, f64),
    pub mul_color: (f64, f64, f64),
    pub mul_base: (f64, f64, f64),
    pub space_color: (f64, f64, f64),
    pub space_color_goal: (f64, f64, f64),
    pub star_color: (f64, f64, f64),
    pub star_color_goal: (f64, f64, f64),

    // Settings fields
    pub retro: bool,
    pub oldschool: bool,
    pub scanlines: bool,
    pub scanlines_offset: i32,
    pub motion_blur: bool,
    pub screenshake_enabled: bool,
    pub smoke_enabled: bool,
    pub chunks_enabled: bool,
    pub flashes_enabled: bool,
    pub advanced_hitbox: bool,
    pub dyn_color: bool,
    pub variable_exposure: bool,
    pub ship_direct_pos: bool,
    pub ship_direct_rotat: bool,
    pub ship_impulse_pos: bool,
    pub ship_impulse_rotat: bool,

    // Rendering fields
    pub ratio_rendu: f64,
    pub phys_width: f64,
    pub phys_height: f64,
    /// 16:9 safe zone in physics coords (always fully visible)
    pub safe_phys_width: f64,
    pub safe_phys_height: f64,
    /// Offset from edge to safe zone in physics coords
    pub safe_offset_x: f64,
    pub safe_offset_y: f64,
    pub current_jitter_double: (f64, f64),
    pub current_jitter_coll_table: (f64, f64),

    // Game event fields
    pub game_screenshake: f64,
    pub game_screenshake_pos: (f64, f64),
    pub game_screenshake_previous_pos: (f64, f64),
    pub shake_score: f64,

    // Framerate fields
    pub frame_compute_secs: f64,  // frame computation time excluding vsync
    pub locked_framerate: bool,
    pub time_last_count: f64,
    pub time_current_count: f64,
    pub last_count: i32,
    pub current_count: i32,
    pub even_frame: bool,
    pub evener_frame: bool,

    // Spawning fields
    pub current_stage_asteroids: i32,
    pub time_since_last_spawn: f64,
    pub stars_nb: i32,
    pub stars_nb_previous: i32,

    // Projectile fields
    pub projectile_recoil: f64,
    pub projectile_cooldown: f64,
    pub projectile_max_speed: f64,
    pub projectile_min_speed: f64,
    pub projectile_deviation: f64,
    pub projectile_radius: f64,
    pub projectile_radius_hitbox: f64,
    pub projectile_number: i32,
    pub ratio_phys_deg: f64,
    pub observer_proper_time: f64,
}

impl Globals {
    /// Create a new Globals instance with default values.
    /// This mirrors the initialization in the OCaml code.
    pub fn new() -> Self {
        let width = WIDTH as f64;
        let height = HEIGHT as f64;
        let ratio_rendu = ((width * height) / (GAME_SURFACE * 1000000.0)).sqrt();
        let phys_width = width / ratio_rendu;
        let phys_height = height / ratio_rendu;

        Self {
            // Time-related fields
            game_speed: 1.0,
            game_speed_target: 1.0,
            time_last_frame: 0.0,
            time_current_frame: 0.0,
            time_of_death: 0.0,
            pause: false,
            restart: false,
            quit: false,

            // Visual fields
            game_exposure: 0.0,
            game_exposure_target: 2.0,
            add_color: (0.0, 0.0, 0.0),
            mul_color: (1.0, 1.0, 1.0),
            mul_base: (1.0, 1.0, 1.0),
            space_color: (0.0, 0.0, 0.0),
            space_color_goal: (0.0, 0.0, 0.0),
            star_color: (100.0, 100.0, 100.0),
            star_color_goal: (100.0, 100.0, 100.0),

            // Settings fields
            retro: false,
            oldschool: false,
            scanlines: false,
            scanlines_offset: 0,
            motion_blur: false,
            screenshake_enabled: true,
            smoke_enabled: true,
            chunks_enabled: true,
            flashes_enabled: true,
            advanced_hitbox: true,
            dyn_color: true,
            variable_exposure: true,
            ship_direct_pos: false,
            ship_direct_rotat: false,
            ship_impulse_pos: true,
            ship_impulse_rotat: true,

            // Rendering fields
            ratio_rendu,
            phys_width,
            phys_height,
            safe_phys_width: phys_width,   // updated by recompute_for_resolution
            safe_phys_height: phys_height,
            safe_offset_x: 0.0,
            safe_offset_y: 0.0,
            current_jitter_double: (0.0, 0.0),
            current_jitter_coll_table: (0.0, 0.0),

            // Game event fields
            game_screenshake: 0.0,
            game_screenshake_pos: (0.0, 0.0),
            game_screenshake_previous_pos: (0.0, 0.0),
            shake_score: 0.0,

            // Framerate fields
            frame_compute_secs: 1.0 / 60.0,
            locked_framerate: false,
            time_last_count: 0.0,
            time_current_count: 10.0,
            last_count: 0,
            current_count: 0,
            even_frame: false,
            evener_frame: false,

            // Spawning fields
            current_stage_asteroids: 3,
            time_since_last_spawn: 9.5,
            stars_nb: 200,
            stars_nb_previous: 200,

            // Projectile fields
            projectile_recoil: 500.0,
            projectile_cooldown: 0.5,
            projectile_max_speed: 15000.0,
            projectile_min_speed: 8000.0,
            projectile_deviation: 0.3,
            projectile_radius: 15.0,
            projectile_radius_hitbox: 20.0,
            projectile_number: 50,
            ratio_phys_deg: 0.001,
            observer_proper_time: 1.0,
        }
    }

    /// Recompute rendering scale and physics dimensions for a given screen resolution.
    ///
    /// Uses a 16:9 safe zone inscribed in the actual screen, preserving GAME_SURFACE density.
    /// On wider-than-16:9 screens, extra world is visible on the sides.
    /// On taller-than-16:9 screens, extra world is visible above/below.
    pub fn recompute_for_resolution(&mut self, screen_w: u32, screen_h: u32) {
        let sw = screen_w as f64;
        let sh = screen_h as f64;
        let safe_aspect = 16.0 / 9.0;
        let screen_aspect = sw / sh;

        let (safe_w, safe_h) = if screen_aspect >= safe_aspect {
            // Wider than 16:9 — height-constrained
            (sh * safe_aspect, sh)
        } else {
            // Taller than 16:9 — width-constrained
            (sw, sw / safe_aspect)
        };

        self.ratio_rendu = (safe_w * safe_h / (GAME_SURFACE * 1_000_000.0)).sqrt();
        self.phys_width  = sw / self.ratio_rendu;
        self.phys_height = sh / self.ratio_rendu;
        self.safe_phys_width  = safe_w / self.ratio_rendu;
        self.safe_phys_height = safe_h / self.ratio_rendu;
        self.safe_offset_x = (self.phys_width  - self.safe_phys_width)  / 2.0;
        self.safe_offset_y = (self.phys_height - self.safe_phys_height) / 2.0;
    }

    /// Get the delta time since the last frame.
    pub fn dt(&self) -> f64 {
        self.time_current_frame - self.time_last_frame
    }

    /// Get the value of a boolean global by toggle enum.
    pub fn get_toggle(&self, t: &GlobalToggle) -> bool {
        match t {
            GlobalToggle::Quit           => self.quit,
            GlobalToggle::Pause          => self.pause,
            GlobalToggle::Restart        => self.restart,
            GlobalToggle::Scanlines      => self.scanlines,
            GlobalToggle::Retro          => self.retro,
            GlobalToggle::AdvancedHitbox => self.advanced_hitbox,
            GlobalToggle::Smoke          => self.smoke_enabled,
            GlobalToggle::Screenshake    => self.screenshake_enabled,
            GlobalToggle::Flashes        => self.flashes_enabled,
            GlobalToggle::Chunks         => self.chunks_enabled,
            GlobalToggle::DynColor       => self.dyn_color,
        }
    }

    /// Set the value of a boolean global by toggle enum.
    pub fn set_toggle(&mut self, t: &GlobalToggle, val: bool) {
        match t {
            GlobalToggle::Quit           => self.quit            = val,
            GlobalToggle::Pause          => self.pause           = val,
            GlobalToggle::Restart        => self.restart         = val,
            GlobalToggle::Scanlines      => self.scanlines       = val,
            GlobalToggle::Retro          => self.retro           = val,
            GlobalToggle::AdvancedHitbox => self.advanced_hitbox = val,
            GlobalToggle::Smoke          => self.smoke_enabled   = val,
            GlobalToggle::Screenshake    => self.screenshake_enabled = val,
            GlobalToggle::Flashes        => self.flashes_enabled = val,
            GlobalToggle::Chunks         => self.chunks_enabled  = val,
            GlobalToggle::DynColor       => self.dyn_color       = val,
        }
    }
}

/// Identifies which boolean field of `Globals` a pause button toggles.
/// Used to avoid storing mutable references in `ButtonBoolean`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GlobalToggle {
    Quit,
    Pause,
    Restart,
    Scanlines,
    Retro,
    AdvancedHitbox,
    Smoke,
    Screenshake,
    Flashes,
    Chunks,
    DynColor,
}
