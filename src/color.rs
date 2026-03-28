/// HDR Color type with no arbitrary limits on brightness
/// Negative values are accepted and represent black
#[derive(Clone, Copy, Debug)]
pub struct HdrColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl HdrColor {
    /// Create a new HdrColor with the given components
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        HdrColor { r, g, b }
    }

    /// Create a black color (0, 0, 0)
    pub fn zero() -> Self {
        HdrColor {
            r: 0.,
            g: 0.,
            b: 0.,
        }
    }

    /// Create a white color (1, 1, 1) for use as multiplication base
    pub fn one() -> Self {
        HdrColor {
            r: 1.,
            g: 1.,
            b: 1.,
        }
    }

    /// Default space color
    pub fn space_color() -> Self {
        HdrColor::new(0., 0., 0.)
    }

    /// Default star color
    pub fn star_color() -> Self {
        HdrColor::new(0., 0., 0.)
    }
}

impl Default for HdrColor {
    fn default() -> Self {
        HdrColor::zero()
    }
}

/// Add two HDR colors
pub fn hdr_add(col1: HdrColor, col2: HdrColor) -> HdrColor {
    HdrColor {
        r: col1.r + col2.r,
        g: col1.g + col2.g,
        b: col1.b + col2.b,
    }
}

/// Subtract col2 from col1
pub fn hdr_sous(col1: HdrColor, col2: HdrColor) -> HdrColor {
    HdrColor {
        r: col1.r - col2.r,
        g: col1.g - col2.g,
        b: col1.b - col2.b,
    }
}

/// Multiply two HDR colors component-wise
pub fn hdr_mul(col1: HdrColor, col2: HdrColor) -> HdrColor {
    HdrColor {
        r: col1.r * col2.r,
        g: col1.g * col2.g,
        b: col1.b * col2.b,
    }
}

/// Adjust color intensity by a scalar factor
pub fn intensify(hdr_in: HdrColor, i: f64) -> HdrColor {
    HdrColor {
        r: i * hdr_in.r,
        g: i * hdr_in.g,
        b: i * hdr_in.b,
    }
}

/// Helper function for exponential decay calculation
fn abso_exp_decay(n: f64, half_life: f64, dt: f64) -> f64 {
    n * 2.0_f64.powf(-dt / half_life)
}

/// Interpolate between two colors using exponential decay
/// Uses the difference between colors and applies exponential decay over dt
pub fn half_color(col1: HdrColor, col2: HdrColor, half_life: f64, dt: f64) -> HdrColor {
    hdr_add(
        col2,
        HdrColor {
            r: abso_exp_decay(col1.r - col2.r, half_life, dt),
            g: abso_exp_decay(col1.g - col2.g, half_life, dt),
            b: abso_exp_decay(col1.b - col2.b, half_life, dt),
        },
    )
}

/// Redirect saturation of a color towards neighboring colors
/// When a channel exceeds 255, redistribute the excess to neighboring channels
pub fn redirect_spectre(col: HdrColor) -> HdrColor {
    HdrColor {
        r: if col.g > 255. {
            col.r + col.g - 255.
        } else {
            col.r
        },
        g: if col.b > 255. && col.r > 255. {
            col.g + col.r + col.b - 510.
        } else if col.r > 255. {
            col.g + col.r - 255.
        } else if col.b > 255. {
            col.g + col.b - 255.
        } else {
            col.g
        },
        b: if col.g > 255. {
            col.b + col.g - 255.
        } else {
            col.b
        },
    }
}

/// Redirect saturation with more aggressive redistribution for extreme saturation
pub fn redirect_spectre_wide(col: HdrColor) -> HdrColor {
    HdrColor {
        r: if col.b > 510. {
            if col.g > 255. {
                col.r + col.g + col.b - 510. - 255.
            } else {
                col.r + col.b - 510.
            }
        } else {
            if col.g > 255. {
                col.r + col.g - 255.
            } else {
                col.r
            }
        },
        g: if col.b > 255. && col.r > 255. {
            col.g + col.r + col.b - 510.
        } else if col.r > 255. {
            col.g + col.r - 255.
        } else if col.b > 255. {
            col.g + col.b - 255.
        } else {
            col.g
        },
        b: if col.r > 510. {
            if col.g > 255. {
                col.r + col.g + col.b - 510. - 255.
            } else {
                col.r + col.b - 510.
            }
        } else {
            if col.g > 255. {
                col.g + col.b - 255.
            } else {
                col.b
            }
        },
    }
}

/// Convert HDR color to RGBA with tone mapping
/// Takes the HDR color and applies additive and multiplicative adjustments,
/// then converts to 8-bit RGBA values
pub fn rgb_of_hdr(
    hdr: HdrColor,
    add_color: &HdrColor,
    mul_color: &HdrColor,
    game_exposure: f64,
) -> [u8; 4] {
    let hdr_mod = redirect_spectre_wide(hdr_mul(
        hdr_add(hdr, intensify(*add_color, game_exposure)),
        *mul_color,
    ));

    let normal_color = |fl: f64| -> u8 {
        let clamped = fl.max(0.0).min(255.0);
        clamped as u8
    };

    [
        normal_color(hdr_mod.r),
        normal_color(hdr_mod.g),
        normal_color(hdr_mod.b),
        255,
    ]
}

/// Saturate a color between grayscale and full saturation
/// i ranges from 0 (grayscale) to any positive value (full saturation and beyond)
/// i = 1.0 leaves the color unchanged
pub fn saturate(hdr_in: HdrColor, i: f64) -> HdrColor {
    let value = (hdr_in.r + hdr_in.g + hdr_in.b) / 3.;
    HdrColor {
        r: i * hdr_in.r + (1. - i) * value,
        g: i * hdr_in.g + (1. - i) * value,
        b: i * hdr_in.b + (1. - i) * value,
    }
}

impl std::ops::Add for HdrColor {
    type Output = HdrColor;
    fn add(self, rhs: HdrColor) -> HdrColor {
        HdrColor { r: self.r + rhs.r, g: self.g + rhs.g, b: self.b + rhs.b }
    }
}

impl std::ops::Sub for HdrColor {
    type Output = HdrColor;
    fn sub(self, rhs: HdrColor) -> HdrColor {
        HdrColor { r: self.r - rhs.r, g: self.g - rhs.g, b: self.b - rhs.b }
    }
}

impl std::ops::Mul for HdrColor {
    type Output = HdrColor;
    fn mul(self, rhs: HdrColor) -> HdrColor {
        HdrColor { r: self.r * rhs.r, g: self.g * rhs.g, b: self.b * rhs.b }
    }
}

impl std::ops::Mul<f64> for HdrColor {
    type Output = HdrColor;
    fn mul(self, rhs: f64) -> HdrColor {
        HdrColor { r: self.r * rhs, g: self.g * rhs, b: self.b * rhs }
    }
}

impl std::ops::Mul<HdrColor> for f64 {
    type Output = HdrColor;
    fn mul(self, rhs: HdrColor) -> HdrColor {
        HdrColor { r: self * rhs.r, g: self * rhs.g, b: self * rhs.b }
    }
}

impl std::ops::AddAssign for HdrColor {
    fn add_assign(&mut self, rhs: HdrColor) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}
