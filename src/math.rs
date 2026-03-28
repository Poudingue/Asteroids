/// Vec2 — a proper 2D vector struct replacing the old `(f64, f64)` tuple alias.
///
/// Provides arithmetic operators, geometric helpers, and conversions.
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Squared length (avoids sqrt).
    #[inline]
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Euclidean length.
    #[inline]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Squared distance to another point.
    #[inline]
    pub fn distance_squared(self, other: Vec2) -> f64 {
        (self - other).length_squared()
    }

    /// Euclidean distance to another point.
    #[inline]
    pub fn distance(self, other: Vec2) -> f64 {
        (self - other).length()
    }

    /// Linear interpolation: `self * t + other * (1 - t)`.
    ///
    /// Matches the OCaml `moytuple` semantics where ratio=1 returns self.
    #[inline]
    pub fn lerp(self, other: Vec2, t: f64) -> Vec2 {
        Vec2 {
            x: self.x * t + other.x * (1.0 - t),
            y: self.y * t + other.y * (1.0 - t),
        }
    }

    /// Element-wise (Hadamard) multiplication.
    #[inline]
    pub fn component_mul(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    /// Convert to polar coordinates `(angle, magnitude)`.
    ///
    /// Returns `(0.0, 0.0)` for the zero vector.
    /// Uses the same formula as the OCaml `affine_to_polar`.
    pub fn to_polar(self) -> (f64, f64) {
        let r = self.length();
        if r == 0.0 {
            (0.0, 0.0)
        } else {
            (2.0 * (self.y / (self.x + r)).atan(), r)
        }
    }

    /// Construct a Vec2 from polar coordinates `(angle, magnitude)`.
    #[inline]
    pub fn from_polar(angle: f64, magnitude: f64) -> Vec2 {
        Vec2 {
            x: magnitude * angle.cos(),
            y: magnitude * angle.sin(),
        }
    }

    /// Convert to `(i32, i32)` by truncation (as i32).
    #[inline]
    pub fn to_i32(self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }

    /// Construct from `(i32, i32)`.
    #[inline]
    pub fn from_i32(v: (i32, i32)) -> Vec2 {
        Vec2 {
            x: v.0 as f64,
            y: v.1 as f64,
        }
    }
}

// ─── Arithmetic operators ──────────────────────────────────────────────────

impl Add for Vec2 {
    type Output = Vec2;
    #[inline]
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: f64) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vec2> for f64 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    #[inline]
    fn neg(self) -> Vec2 {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign<f64> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

// ─── Conversions ───────────────────────────────────────────────────────────

impl From<(f64, f64)> for Vec2 {
    #[inline]
    fn from(t: (f64, f64)) -> Vec2 {
        Vec2 { x: t.0, y: t.1 }
    }
}

impl From<Vec2> for (f64, f64) {
    #[inline]
    fn from(v: Vec2) -> (f64, f64) {
        (v.x, v.y)
    }
}
