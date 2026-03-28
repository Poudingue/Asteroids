use rand::Rng;

pub use crate::math::Vec2;

/// Generate a random float between min and max
pub fn randfloat(min: f64, max: f64, rng: &mut impl Rng) -> f64 {
    min + rng.gen::<f64>() * (max - min)
}

/// Square a value (useful for Pythagorean formulas)
pub fn carre(v: f64) -> f64 {
    v * v
}

/// Exponential decay based on in-game time
///
/// # Arguments
/// * `n` - Initial value
/// * `half_life` - Half-life duration
/// * `observer_proper_time` - Observer's proper time ratio
/// * `game_speed` - Current game speed multiplier
/// * `time_last_frame` - Previous frame time
/// * `time_current_frame` - Current frame time
/// * `proper_time` - Proper time constant
pub fn exp_decay(
    n: f64,
    half_life: f64,
    observer_proper_time: f64,
    game_speed: f64,
    time_last_frame: f64,
    time_current_frame: f64,
    proper_time: f64,
) -> f64 {
    n * 2.0_f64.powf(
        (observer_proper_time * game_speed * (time_last_frame - time_current_frame))
            / (proper_time * half_life),
    )
}

/// Exponential decay based on real time (not game time)
///
/// # Arguments
/// * `n` - Initial value
/// * `half_life` - Half-life duration
/// * `time_last_frame` - Previous frame time
/// * `time_current_frame` - Current frame time
pub fn abso_exp_decay(
    n: f64,
    half_life: f64,
    time_last_frame: f64,
    time_current_frame: f64,
) -> f64 {
    n * 2.0_f64.powf((time_last_frame - time_current_frame) / half_life)
}

/// Calculate the hypotenuse (magnitude) of a 2D vector
pub fn hypothenuse(v: Vec2) -> f64 {
    (carre(v.x) + carre(v.y)).sqrt()
}

/// Add two 2D vectors
pub fn addtuple(v1: Vec2, v2: Vec2) -> Vec2 {
    Vec2::new(v1.x + v2.x, v1.y + v2.y)
}

/// Subtract two 2D vectors
pub fn soustuple(v1: Vec2, v2: Vec2) -> Vec2 {
    Vec2::new(v1.x - v2.x, v1.y - v2.y)
}

/// Multiply a 2D vector by a scalar
pub fn multuple(v: Vec2, ratio: f64) -> Vec2 {
    Vec2::new(v.x * ratio, v.y * ratio)
}

/// Linear interpolation between two floats
///
/// # Arguments
/// * `val1` - First value
/// * `val2` - Second value
/// * `ratio` - Blend ratio (1.0 = val1, 0.0 = val2)
pub fn moyfloat(val1: f64, val2: f64, ratio: f64) -> f64 {
    val1 * ratio + val2 * (1.0 - ratio)
}

/// Linear interpolation between two 2D vectors
///
/// # Arguments
/// * `tuple1` - First vector
/// * `tuple2` - Second vector
/// * `ratio` - Blend ratio (1.0 = tuple1, 0.0 = tuple2)
pub fn moytuple(tuple1: Vec2, tuple2: Vec2, ratio: f64) -> Vec2 {
    Vec2::new(
        tuple1.x * ratio + tuple2.x * (1.0 - ratio),
        tuple1.y * ratio + tuple2.y * (1.0 - ratio),
    )
}

/// Element-wise multiplication of two 2D vectors
pub fn multuple_parallel(v1: Vec2, v2: Vec2) -> Vec2 {
    Vec2::new(v1.x * v2.x, v1.y * v2.y)
}

/// Check if a point is between two other points (axis-aligned bounding box check)
pub fn entretuple(p: Vec2, min: Vec2, max: Vec2) -> bool {
    p.x > min.x && p.x < max.x && p.y > min.y && p.y < max.y
}

/// Convert a 2D vector of floats to a 2D vector of ints
pub fn inttuple(v: Vec2) -> (i32, i32) {
    (v.x as i32, v.y as i32)
}

/// Convert a 2D vector of ints to a 2D vector of floats
pub fn floattuple(v: (i32, i32)) -> Vec2 {
    Vec2::new(v.0 as f64, v.1 as f64)
}

/// Apply dithering to a float value before converting to int
///
/// # Arguments
/// * `fl` - Float value to convert
/// * `dither_aa` - Whether anti-aliasing dithering is enabled
/// * `dither_power` - Dithering strength
pub fn dither(fl: f64, dither_aa: bool, dither_power: f64, rng: &mut impl Rng) -> i32 {
    if dither_aa {
        (fl + rng.gen::<f64>() * dither_power) as i32
    } else {
        fl as i32
    }
}

/// Apply dithering to a radius value before converting to int
///
/// # Arguments
/// * `fl` - Float value to convert
/// * `dither_aa` - Whether anti-aliasing dithering is enabled
/// * `dither_power_radius` - Radius-specific dithering strength
pub fn dither_radius(
    fl: f64,
    dither_aa: bool,
    dither_power_radius: f64,
    rng: &mut impl Rng,
) -> i32 {
    if dither_aa {
        (fl - 0.5 + rng.gen::<f64>() * dither_power_radius) as i32
    } else {
        fl as i32
    }
}

/// Apply dithering to a 2D vector with jitter
///
/// # Arguments
/// * `v` - 2D vector to convert
/// * `dither_aa` - Whether anti-aliasing dithering is enabled
/// * `jitter` - Current jitter offset to apply
pub fn dither_tuple(v: Vec2, dither_aa: bool, jitter: Vec2) -> (i32, i32) {
    if dither_aa {
        inttuple(addtuple(jitter, v))
    } else {
        inttuple(v)
    }
}

/// Project tuple2 onto tuple1 with a ratio
///
/// # Arguments
/// * `tuple1` - Base vector
/// * `tuple2` - Vector to add
/// * `ratio` - Scaling ratio for tuple2
pub fn proj(tuple1: Vec2, tuple2: Vec2, ratio: f64) -> Vec2 {
    addtuple(tuple1, multuple(tuple2, ratio))
}

/// Convert polar coordinates (angle, magnitude) to Cartesian coordinates (x, y)
pub fn polar_to_affine(angle: f64, valeur: f64) -> Vec2 {
    Vec2::new(valeur * angle.cos(), valeur * angle.sin())
}

/// Convert polar coordinates tuple to Cartesian coordinates
pub fn polar_to_affine_tuple(polar: (f64, f64)) -> Vec2 {
    polar_to_affine(polar.0, polar.1)
}


/// Convert Cartesian coordinates (x, y) to polar coordinates (angle, magnitude)
pub fn affine_to_polar(v: Vec2) -> Vec2 {
    let r = hypothenuse(v);
    if r == 0.0 {
        Vec2::ZERO
    } else {
        Vec2::new(2.0 * (v.y / (v.x + r)).atan(), r)
    }
}

/// Calculate squared distance between two points
/// More efficient than distance since it avoids the square root
pub fn distancecarre(p1: Vec2, p2: Vec2) -> f64 {
    carre(p2.x - p1.x) + carre(p2.y - p1.y)
}

/// Modulo operation for floats (wrapping)
pub fn modulo_float(value: f64, modulo: f64) -> f64 {
    if value < 0.0 {
        value + modulo
    } else if value >= modulo {
        value - modulo
    } else {
        value
    }
}

/// Modulo operation for resolution-based wrapping
///
/// # Arguments
/// * `v` - Position vector
/// * `phys_width` - Physical width of game space
/// * `phys_height` - Physical height of game space
pub fn modulo_reso(v: Vec2, phys_width: f64, phys_height: f64) -> Vec2 {
    Vec2::new(
        modulo_float(v.x, phys_width),
        modulo_float(v.y, phys_height),
    )
}

/// Modulo operation for 3x3 resolution wrapping (for objects wrapping around edges)
/// Considers the play surface as 3x3 times the actual play area
///
/// # Arguments
/// * `v` - Position vector
/// * `phys_width` - Physical width of game space
/// * `phys_height` - Physical height of game space
pub fn modulo_3reso(v: Vec2, phys_width: f64, phys_height: f64) -> Vec2 {
    Vec2::new(
        modulo_float(v.x + phys_width, phys_width * 3.0) - phys_width,
        modulo_float(v.y + phys_height, phys_height * 3.0) - phys_height,
    )
}

/// Set difference: elements in l1 that are not in l2
pub fn diff<T: PartialEq + Clone>(l1: &[T], l2: &[T]) -> Vec<T> {
    l1.iter().filter(|x| !l2.contains(x)).cloned().collect()
}
