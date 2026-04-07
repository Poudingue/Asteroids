//! Parameters of the whole game
//!
//! This is a port of ml/parameters.ml to Rust.
//! Contains all game configuration constants and mutable global state.

use crate::math::Vec2;

// ============================================================================
// Constants (Display Parameters)
// ============================================================================

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
// Maximum delta time to prevent physics explosions on frame stalls (alt-tab, window drag).
// Equivalent to a 20fps floor: physics never sees more than 50ms per frame.
pub const MAX_DT: f64 = 0.05;

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

/// MSAA sample count for polygon rendering. Valid values: 1 (off), 2, 4.
/// SDF entities use their own smoothstep AA (controlled by SDF_AA_ENABLED in sdf.wgsl).
// SDF anti-aliasing is controlled by compile-time const `SDF_AA_ENABLED` in src/shaders/sdf.wgsl.
// true = smoothstep AA (default), false = hard edges.
// This is independent of MSAA, which only affects polygon geometry.
pub const DEFAULT_MSAA_SAMPLE_COUNT: u32 = 4;

// Button colors (stored as u32: (r << 16) | (g << 8) | b)
pub const TRUECOLOR: u32 = 128 << 8; // rgb 0 128 0
pub const FALSECOLOR: u32 = 128 << 16; // rgb 128 0 0
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
pub const SHIP_HALF_STOP: f64 = 10.0; // Time needed to lose half inertia.
pub const SHIP_MAX_MOMENT: f64 = 0.5; // In radian/s²
pub const SHIP_HALF_STOP_ROTAT: f64 = 0.2; // Time needed to lose half angular momentum

// Ship visual colors (HDR range: values >255 use wider color gamut in HDR mode)
pub const SHIP_COLOR_PRIMARY: (f64, f64, f64) = (1400.0, 60.0, 20.0);
pub const SHIP_COLOR_FIN: (f64, f64, f64) = (180.0, 15.0, 10.0);
pub const SHIP_COLOR_FIN_HIGHLIGHT: (f64, f64, f64) = (280.0, 20.0, 15.0);
pub const SHIP_COLOR_FIN_SHADOW: (f64, f64, f64) = (100.0, 3.0, 3.0);
pub const SHIP_COLOR_SHADOW_DARK: (f64, f64, f64) = (8.0, 8.0, 8.0);
pub const SHIP_COLOR_SHADOW_MID: (f64, f64, f64) = (25.0, 25.0, 25.0);
pub const SHIP_COLOR_HIGHLIGHT: (f64, f64, f64) = (220.0, 190.0, 160.0);
pub const SHIP_COLOR_ACCENT: (f64, f64, f64) = (18.0, 28.0, 45.0);

// Minimum time between random teleportations
pub const COOLDOWN_TP: f64 = 5.0;
pub const TP_TIME_INVIC: f64 = 1.0; // Invincibility time after tp. TODO: Implement

// ============================================================================
// Gamepad constants
// ============================================================================

/// Inner dead zone threshold — stick deflection below this is treated as zero
pub const STICK_DEAD_ZONE_INNER: f64 = 0.15;
/// Outer dead zone threshold — stick deflection above this is treated as 1.0
pub const STICK_DEAD_ZONE_OUTER: f64 = 0.90;
/// Seconds of idle before drift recalibration starts
pub const DRIFT_RECENTER_DELAY: f64 = 2.0;
/// Lerp speed for drift compensation (per second)
pub const DRIFT_RECENTER_SPEED: f64 = 0.5;
/// Visual smoothing for ship rotation (0.0 = instant, higher = more lag)
pub const AIM_VISUAL_SMOOTHING: f64 = 8.0;

// ============================================================================
// Teleport constants
// ============================================================================

/// Half-angle of the teleport targeting cone (degrees)
pub const TELEPORT_CONE_HALF_ANGLE_DEG: f64 = 7.5;

// ============================================================================
// Constants (Projectile Parameters)
// ============================================================================

// Regular projectile values
pub const PROJECTILE_HERIT_SPEED: bool = true;
pub const PROJECTILE_HEALTH: f64 = 0.0; // We consider death when health drops below zero. We have certainty here that the projectile will destroy itself.
pub const PROJECTILE_RADIUS: f64 = 15.0;
pub const PROJECTILE_RADIUS_HITBOX: f64 = 20.0;
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

/// Scaling factor for shockwave velocity impulse. Higher = stronger push.
pub const SHOCKWAVE_IMPULSE_SCALE: f64 = 0.5;
/// Blast range as a multiplier of explosion ext_radius.
pub const SHOCKWAVE_RANGE_MULTIPLIER: f64 = 1.5;
/// Fixed impulse scale for zero-mass particles (smoke, sparks). Visual flair only.
pub const SHOCKWAVE_PARTICLE_PUSH: f64 = 0.3;

// ============================================================================
// Constants (Muzzle Flash Parameters)
// ============================================================================

pub const MUZZLE_RATIO_RADIUS: f64 = 3.0;
pub const MUZZLE_RATIO_SPEED: f64 = 0.05;

// ============================================================================
// Constants (Fire/Thrust Parameters)
// ============================================================================

pub const FIRE_MAX_RANDOM: f64 = 300.0;
pub const FIRE_MIN_SPEED: f64 = 1000.0;
pub const FIRE_MAX_SPEED: f64 = 2000.0;
/// Ratio applied to ship speed when computing fire kick velocity.
/// >1.0 ensures fire always moves backward relative to ship even with scatter.
pub const FIRE_SPEED_RATIO: f64 = 1.2;
pub const FIRE_RATIO_RADIUS: f64 = 1.4;

// ============================================================================
// Particle Budget Constants
// ============================================================================

pub const PARTICLE_BUDGET_SMOKE: usize = 2048;
pub const PARTICLE_BUDGET_FIRE: usize = 512;
pub const PARTICLE_BUDGET_CHUNKS: usize = 512;
pub const PARTICLE_BUDGET_EXPLOSIONS: usize = 256;
pub const PARTICLE_BUDGET_PROJECTILES: usize = 256;
pub const PARTICLE_DEGRADATION_THRESHOLD: f64 = 0.9;
pub const PARTICLE_DEGRADATION_FADE_MULTIPLIER: f64 = 3.0;

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
pub const SCREENSHAKE_DAM_RATIO: f64 = 0.025; // bumped ×5 from OCaml's 0.005 for more impactful explosions
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
// Global Mutable State — Sub-structs
// ============================================================================

/// Time, speed, and game flow control.
/// Simulation time stepping mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimulationMode {
    /// Variable dt from wall clock, capped at MAX_DT. Default gameplay mode.
    RealTime,
    /// Fixed dt = 1/target_fps. Sleeps if frame is faster. Playable + deterministic.
    FixedInteractive(u32),
    /// Fixed dt = 1/target_fps. No sleeping — runs as fast as possible.
    FixedFullSpeed(u32),
    /// Fixed dt, no window, no renderer. Pure simulation.
    Headless(u32),
}

impl SimulationMode {
    /// Returns the fixed dt if in a fixed mode, None for RealTime.
    pub fn fixed_dt(&self) -> Option<f64> {
        match self {
            SimulationMode::RealTime => None,
            SimulationMode::FixedInteractive(fps)
            | SimulationMode::FixedFullSpeed(fps)
            | SimulationMode::Headless(fps) => Some(1.0 / *fps as f64),
        }
    }

    /// Whether this mode requires a window and renderer.
    pub fn needs_window(&self) -> bool {
        !matches!(self, SimulationMode::Headless(_))
    }

    /// Whether this mode should sleep to maintain target framerate.
    pub fn should_sleep(&self) -> bool {
        matches!(self, SimulationMode::FixedInteractive(_))
    }
}

pub struct TimeConfig {
    pub simulation_mode: SimulationMode,
    pub game_speed: f64,
    pub game_speed_target: f64,
    pub time_last_frame: f64,
    pub time_current_frame: f64,
    pub time_of_death: f64,
    pub frame_count: u64,
    pub pause: bool,
    pub restart: bool,
    pub quit: bool,
}

/// Exposure and color overlay state.
pub struct ExposureConfig {
    pub game_exposure: f64,
    pub game_exposure_target: f64,
    pub add_color: (f64, f64, f64),
    pub mul_color: (f64, f64, f64),
    pub mul_base: (f64, f64, f64),
}

/// HDR output and anti-aliasing configuration.
#[derive(Clone, Debug)]
pub struct HdrConfig {
    pub hdr_enabled: bool,
    pub hud_nits: f64,
    pub exposure: f64,
    pub max_brightness: f64,
    pub smaa_enabled: bool,
    pub msaa_sample_count: u32,
    /// Remembered exposure target for SDR mode.
    pub game_exposure_target_sdr: f64,
    /// Remembered exposure target for HDR mode.
    pub game_exposure_target_hdr: f64,
    /// Tonemap algorithm: 0 = Passthrough, 1 = Pseudo-Reinhard, 2 = Hard Redirect, 3 = Soft Redirect (default).
    pub tonemap_variant: u32,
}

impl Default for HdrConfig {
    fn default() -> Self {
        Self {
            hdr_enabled: false,
            hud_nits: 155.0,
            exposure: 1.0,
            max_brightness: 1000.0,
            smaa_enabled: false,
            msaa_sample_count: 4,
            game_exposure_target_sdr: 2.0,
            game_exposure_target_hdr: 2.0,
            tonemap_variant: 3,
        }
    }
}

/// Visual/rendering toggles and color goals.
pub struct VisualConfig {
    pub motion_blur: bool,
    pub screenshake_enabled: bool,
    pub smoke_enabled: bool,
    pub chunks_enabled: bool,
    pub flashes_enabled: bool,
    pub dyn_color: bool,
    pub variable_exposure: bool,
    pub space_color: (f64, f64, f64),
    pub space_color_goal: (f64, f64, f64),
    pub star_color: (f64, f64, f64),
    pub star_color_goal: (f64, f64, f64),
}

/// Rendering scale and physics / safe-zone dimensions.
pub struct RenderState {
    pub render_scale: f64,
    pub phys_width: f64,
    pub phys_height: f64,
    /// 16:9 safe zone in physics coords (always fully visible)
    pub safe_phys_width: f64,
    pub safe_phys_height: f64,
    /// Offset from edge to safe zone in physics coords
    pub safe_offset_x: f64,
    pub safe_offset_y: f64,
    pub current_jitter_double: Vec2,
    pub current_jitter_coll_table: Vec2,
}

/// Screen-shake intensity and position state.
pub struct ScreenshakeState {
    pub game_screenshake: f64,
    pub game_screenshake_pos: Vec2,
    pub game_screenshake_previous_pos: Vec2,
    pub shake_score: f64,
}

/// Framerate measurement and control state.
pub struct FramerateState {
    /// Frame computation time excluding vsync
    pub frame_compute_secs: f64,
    pub locked_framerate: bool,
    pub time_last_count: f64,
    pub time_current_count: f64,
    pub last_count: i32,
    pub current_count: i32,
    pub even_frame: bool,
    pub evener_frame: bool,
}

/// Asteroid spawn progression state.
pub struct SpawnState {
    pub current_stage_asteroids: i32,
    pub time_since_last_spawn: f64,
    pub stars_nb: i32,
    pub stars_nb_previous: i32,
}

/// Active weapon parameters (current projectile type).
pub struct WeaponState {
    pub projectile_recoil: f64,
    pub projectile_cooldown: f64,
    pub projectile_max_speed: f64,
    pub projectile_min_speed: f64,
    pub projectile_deviation: f64,
    pub projectile_radius: f64,
    pub projectile_radius_hitbox: f64,
    pub projectile_number: i32,
    pub physics_damage_ratio: f64,
}

// ============================================================================
// Global Mutable State
// ============================================================================

/// Global game state that changes during execution.
/// This corresponds to all the `let ... = ref ...` bindings in the OCaml code.
pub struct Globals {
    pub time: TimeConfig,
    pub exposure: ExposureConfig,
    pub visual: VisualConfig,
    pub render: RenderState,
    pub screenshake: ScreenshakeState,
    pub framerate: FramerateState,
    pub spawn: SpawnState,
    pub weapon: WeaponState,
    pub advanced_hitbox: bool,
    pub observer_proper_time: f64,
    pub hdr: HdrConfig,
}

impl Globals {
    /// Create a new Globals instance with default values.
    /// This mirrors the initialization in the OCaml code.
    pub fn new() -> Self {
        let width = WIDTH as f64;
        let height = HEIGHT as f64;
        let render_scale = ((width * height) / (GAME_SURFACE * 1000000.0)).sqrt();
        let phys_width = width / render_scale;
        let phys_height = height / render_scale;

        Self {
            time: TimeConfig {
                simulation_mode: SimulationMode::RealTime,
                game_speed: 1.0,
                game_speed_target: 1.0,
                time_last_frame: 0.0,
                time_current_frame: 0.0,
                time_of_death: 0.0,
                frame_count: 0,
                pause: false,
                restart: false,
                quit: false,
            },
            exposure: ExposureConfig {
                game_exposure: 0.0,
                game_exposure_target: 2.0,
                add_color: (0.0, 0.0, 0.0),
                mul_color: (1.0, 1.0, 1.0),
                mul_base: (1.0, 1.0, 1.0),
            },
            visual: VisualConfig {
                motion_blur: false,
                screenshake_enabled: true,
                smoke_enabled: true,
                chunks_enabled: true,
                flashes_enabled: true,
                dyn_color: true,
                variable_exposure: true,
                space_color: (0.0, 0.0, 0.0),
                space_color_goal: (0.0, 0.0, 0.0),
                star_color: (100.0, 100.0, 100.0),
                star_color_goal: (100.0, 100.0, 100.0),
            },
            render: RenderState {
                render_scale,
                phys_width,
                phys_height,
                safe_phys_width: phys_width, // updated by recompute_for_resolution
                safe_phys_height: phys_height,
                safe_offset_x: 0.0,
                safe_offset_y: 0.0,
                current_jitter_double: Vec2::ZERO,
                current_jitter_coll_table: Vec2::ZERO,
            },
            screenshake: ScreenshakeState {
                game_screenshake: 0.0,
                game_screenshake_pos: Vec2::ZERO,
                game_screenshake_previous_pos: Vec2::ZERO,
                shake_score: 0.0,
            },
            framerate: FramerateState {
                frame_compute_secs: 1.0 / 60.0,
                locked_framerate: false,
                time_last_count: 0.0,
                time_current_count: 10.0,
                last_count: 0,
                current_count: 0,
                even_frame: false,
                evener_frame: false,
            },
            spawn: SpawnState {
                current_stage_asteroids: 3,
                time_since_last_spawn: 9.5,
                stars_nb: 200,
                stars_nb_previous: 200,
            },
            weapon: WeaponState {
                projectile_recoil: 500.0,
                projectile_cooldown: 0.5,
                projectile_max_speed: 15000.0,
                projectile_min_speed: 8000.0,
                projectile_deviation: 0.3,
                projectile_radius: 15.0,
                projectile_radius_hitbox: 20.0,
                projectile_number: 50,
                physics_damage_ratio: 0.001,
            },
            advanced_hitbox: true,
            observer_proper_time: 1.0,
            hdr: HdrConfig::default(),
        }
    }
}

impl Default for Globals {
    fn default() -> Self {
        Self::new()
    }
}

impl Globals {
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

        self.render.render_scale = (safe_w * safe_h / (GAME_SURFACE * 1_000_000.0)).sqrt();
        self.render.phys_width = sw / self.render.render_scale;
        self.render.phys_height = sh / self.render.render_scale;
        self.render.safe_phys_width = safe_w / self.render.render_scale;
        self.render.safe_phys_height = safe_h / self.render.render_scale;
        self.render.safe_offset_x = (self.render.phys_width - self.render.safe_phys_width) / 2.0;
        self.render.safe_offset_y = (self.render.phys_height - self.render.safe_phys_height) / 2.0;
    }

    /// Get the delta time since the last frame.
    pub fn dt(&self) -> f64 {
        self.time.time_current_frame - self.time.time_last_frame
    }

    /// Get the value of a boolean global by toggle enum.
    pub fn get_toggle(&self, t: &GlobalToggle) -> bool {
        match t {
            GlobalToggle::Quit => self.time.quit,
            GlobalToggle::Pause => self.time.pause,
            GlobalToggle::Restart => self.time.restart,
            GlobalToggle::AdvancedHitbox => self.advanced_hitbox,
            GlobalToggle::Smoke => self.visual.smoke_enabled,
            GlobalToggle::Screenshake => self.visual.screenshake_enabled,
            GlobalToggle::Flashes => self.visual.flashes_enabled,
            GlobalToggle::Chunks => self.visual.chunks_enabled,
            GlobalToggle::DynColor => self.visual.dyn_color,
            GlobalToggle::Hdr => self.hdr.hdr_enabled,
            GlobalToggle::Smaa => self.hdr.smaa_enabled,
        }
    }

    /// Set the value of a boolean global by toggle enum.
    pub fn set_toggle(&mut self, t: &GlobalToggle, val: bool) {
        match t {
            GlobalToggle::Quit => self.time.quit = val,
            GlobalToggle::Pause => self.time.pause = val,
            GlobalToggle::Restart => self.time.restart = val,
            GlobalToggle::AdvancedHitbox => self.advanced_hitbox = val,
            GlobalToggle::Smoke => self.visual.smoke_enabled = val,
            GlobalToggle::Screenshake => self.visual.screenshake_enabled = val,
            GlobalToggle::Flashes => self.visual.flashes_enabled = val,
            GlobalToggle::Chunks => self.visual.chunks_enabled = val,
            GlobalToggle::DynColor => self.visual.dyn_color = val,
            GlobalToggle::Hdr => self.hdr.hdr_enabled = val,
            GlobalToggle::Smaa => self.hdr.smaa_enabled = val,
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
    AdvancedHitbox,
    Smoke,
    Screenshake,
    Flashes,
    Chunks,
    DynColor,
    Hdr,
    Smaa,
}
