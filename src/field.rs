use crate::math::Vec2;

#[derive(Debug, Clone)]
pub enum FieldSourceKind {
    ShockwaveRing {
        speed: f64,
        width: f64,
        pressure: f64,
    },
    GravityWell {
        strength: f64,
        radius: f64,
    },
    Vortex {
        angular_speed: f64,
        radius: f64,
    },
    WindZone {
        direction: Vec2,
        strength: f64,
        radius: f64,
    },
}

#[derive(Debug, Clone)]
pub struct FieldSource {
    pub kind: FieldSourceKind,
    pub position: Vec2,
    pub age: f64,
    pub lifetime: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldSample {
    pub wind: Vec2,
    pub gravity: Vec2,
    pub time_dilation: f64,
}

impl FieldSample {
    pub fn neutral() -> Self {
        Self {
            wind: Vec2 { x: 0.0, y: 0.0 },
            gravity: Vec2 { x: 0.0, y: 0.0 },
            time_dilation: 1.0,
        }
    }
}

/// Evaluate all field sources at a position. Stub — returns neutral.
pub fn evaluate_field(_position: Vec2, _sources: &[FieldSource]) -> FieldSample {
    FieldSample::neutral()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;

    #[test]
    fn neutral_sample_has_no_effect() {
        let sample = FieldSample::neutral();
        assert_eq!(sample.wind.x, 0.0);
        assert_eq!(sample.wind.y, 0.0);
        assert_eq!(sample.gravity.x, 0.0);
        assert_eq!(sample.gravity.y, 0.0);
        assert_eq!(sample.time_dilation, 1.0);
    }

    #[test]
    fn evaluate_empty_sources_returns_neutral() {
        let pos = Vec2 { x: 100.0, y: 200.0 };
        let sample = evaluate_field(pos, &[]);
        assert_eq!(sample.time_dilation, 1.0);
    }

    #[test]
    fn evaluate_with_sources_returns_neutral_stub() {
        let pos = Vec2 { x: 0.0, y: 0.0 };
        let sources = vec![FieldSource {
            kind: FieldSourceKind::GravityWell {
                strength: 100.0,
                radius: 50.0,
            },
            position: Vec2 { x: 10.0, y: 10.0 },
            age: 0.0,
            lifetime: 10.0,
        }];
        let sample = evaluate_field(pos, &sources);
        assert_eq!(sample.time_dilation, 1.0);
    }
}
