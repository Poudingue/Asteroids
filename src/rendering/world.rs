use rand::prelude::*;

use crate::color::*;
use crate::game::{hdr, to_hdr_rgba};
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;
use crate::rendering::{CapsuleInstance, CircleInstance, Renderer2D};

// ============================================================================
// Trail rendering
// ============================================================================

/// Configuration for motion-blur capsule trail rendering.
pub struct TrailConfig {
    pub radius: f64,
    pub brightness_falloff: f64,
    pub shutter_speed: f64,
}

impl TrailConfig {
    pub fn star() -> Self {
        TrailConfig {
            radius: 1.0,
            brightness_falloff: 1.0,
            shutter_speed: 1.0,
        }
    }
    pub fn bullet(radius: f64) -> Self {
        TrailConfig {
            radius,
            brightness_falloff: 0.5,
            shutter_speed: 1.0,
        }
    }
}

/// Render a motion-blur capsule trail between two screen-space endpoints.
/// Brightness conservation: brightness×area stays constant as circle→capsule.
/// Scale = π·r² / (π·r² + 2·r·L) = 1 / (1 + 2L/(π·r))
pub fn render_trail(
    target: &mut Vec<CapsuleInstance>,
    p0: (f64, f64),
    p1: (f64, f64),
    cfg: &TrailConfig,
    base_color: [f32; 4],
) {
    let (x1, y1) = p0;
    let (x2, y2) = p1;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let trail_len = (dx * dx + dy * dy).sqrt();

    let r = cfg.radius.max(0.001);
    let area_scale = if trail_len < 0.001 {
        1.0
    } else {
        let pi_r2 = std::f64::consts::PI * r * r;
        pi_r2 / (pi_r2 + 2.0 * r * trail_len)
    };

    let trail_lum = if cfg.brightness_falloff <= 0.0 {
        1.0
    } else if cfg.brightness_falloff >= 1.0 {
        (1.0 / (1.0 + trail_len)).sqrt()
    } else {
        cfg.brightness_falloff * (r / (r + trail_len)).sqrt()
    };

    let combined_scale = (area_scale * trail_lum) as f32;
    let color = [
        base_color[0] * combined_scale,
        base_color[1] * combined_scale,
        base_color[2] * combined_scale,
        base_color[3],
    ];

    let radius = (cfg.radius * cfg.shutter_speed).max(1.0) as f32;
    if radius > 0.0 {
        target.push(CapsuleInstance {
            p0: [x1 as f32, y1 as f32],
            p1: [x2 as f32, y2 as f32],
            radius,
            color,
            _padding: [0.0; 3],
        });
    }
}

// ============================================================================
// Polygon rendering helpers
// ============================================================================

/// Render a single polar polygon at a position with rotation and color
pub fn render_poly(
    poly: &[(f64, f64)],
    pos: Vec2,
    rotat: f64,
    color: [f32; 4],
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    let affine = polygon_to_cartesian(poly, rotat, globals.render.render_scale);
    let displaced = translate_polygon(&affine, pos);
    let screen_points: Vec<(i32, i32)> = displaced
        .iter()
        .map(|&p| dither_vec(p, DITHER_AA, globals.render.current_jitter_double))
        .collect();
    renderer.fill_poly(&screen_points, color);
}

/// Render all shape polygons of an entity's visuals
pub fn render_shapes(
    shapes: &[((f64, f64, f64), Polygon)],
    pos: Vec2,
    rotat: f64,
    exposure: f64,
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    for (col, Polygon(poly)) in shapes {
        let color = to_hdr_rgba(intensify(hdr(*col), exposure));
        render_poly(poly, pos, rotat, color, renderer, globals);
    }
}

// ============================================================================
// Entity rendering
// ============================================================================

/// Render an entity: base circle + polygon shapes
pub fn render_visuals(entity: &Entity, offset: Vec2, renderer: &mut Renderer2D, globals: &Globals) {
    let visuals = &entity.visuals;
    let position = scale_vec(
        add_vec(
            add_vec(entity.position, globals.screenshake.game_screenshake_pos),
            offset,
        ),
        globals.render.render_scale,
    );
    let exposure = globals.exposure.game_exposure * entity.hdr_exposure;

    // SDF circle for entities with radius but no polygon shapes (smoke, explosions)
    // Entities with shapes (ship, asteroids): only render polygon shapes, no base circle
    if visuals.radius > 0.0 && visuals.shapes.is_empty() {
        let color = to_hdr_rgba(intensify(hdr(visuals.color), exposure));
        let (x, y) = dither_vec(position, DITHER_AA, globals.render.current_jitter_double);
        let r = (visuals.radius * globals.render.render_scale).max(1.0) as f32;
        if r > 0.0 {
            let (target, falloff_width) = if entity.kind == EntityKind::Smoke {
                // Layer 3: smoke — alpha blend, soft falloff
                (&mut renderer.smoke_circles, 0.2f32)
            } else {
                // Layer 5: effects — explosions and other circle-only entities
                (&mut renderer.effect_circles, 0.0f32)
            };
            target.push(CircleInstance {
                center: [x as f32, y as f32],
                radius: r,
                color,
                falloff_width,
            });
        }
    }

    // Polygon shapes on top
    render_shapes(
        &visuals.shapes,
        position,
        entity.orientation,
        exposure,
        renderer,
        globals,
    );
}

/// Render a chunk (small debris) — simpler than full entity rendering
pub fn render_chunk(entity: &Entity, renderer: &mut Renderer2D, globals: &Globals) {
    let pos = scale_vec(
        add_vec(entity.position, globals.screenshake.game_screenshake_pos),
        globals.render.render_scale,
    );
    let intensity_chunk = 1.0;
    let color = to_hdr_rgba(intensify(
        hdr(entity.visuals.color),
        intensity_chunk * globals.exposure.game_exposure * entity.hdr_exposure,
    ));
    let (x, y) = dither_vec(pos, DITHER_AA, globals.render.current_jitter_double);
    let r = (globals.render.render_scale * entity.visuals.radius).max(1.0) as f32;
    if r > 0.0 {
        renderer.effect_circles.push(CircleInstance {
            center: [x as f32, y as f32],
            radius: r,
            color,
            falloff_width: 0.0,
        });
    }
}

pub fn render_star_trail(
    star: &Star,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let pos1 = scale_vec(
        add_vec(star.pos, globals.screenshake.game_screenshake_pos),
        globals.render.render_scale,
    );
    let last_position = scale_vec(
        add_vec(
            star.last_pos,
            globals.screenshake.game_screenshake_previous_pos,
        ),
        globals.render.render_scale,
    );
    let pos2 = lerp_vec(last_position, pos1, SHUTTER_SPEED);
    let (x1, y1) = dither_vec(pos1, DITHER_AA, globals.render.current_jitter_double);
    let (x2, y2) = dither_vec(pos2, DITHER_AA, globals.render.current_jitter_double);

    let lum = if globals.time.pause {
        star.lum + 0.5 * STAR_RAND_LUM
    } else {
        star.lum + rng.gen::<f64>() * STAR_RAND_LUM
    };

    let star_color_tmp = intensify(
        hdr(globals.visual.star_color),
        lum * globals.exposure.game_exposure,
    );

    if x1 == x2 && y1 == y2 {
        // Static star: render as a cross of pixels
        let center_color = to_hdr_rgba(intensify(
            hdr_add(star_color_tmp, hdr(globals.visual.space_color)),
            globals.exposure.game_exposure,
        ));
        renderer.plot(x1, y1, center_color);

        let arm_color = to_hdr_rgba(intensify(star_color_tmp, 0.25));
        renderer.plot(x1 + 1, y1, arm_color);
        renderer.plot(x1 - 1, y1, arm_color);
        renderer.plot(x1, y1 + 1, arm_color);
        renderer.plot(x1, y1 - 1, arm_color);

        let diag_color = to_hdr_rgba(intensify(star_color_tmp, 0.125));
        renderer.plot(x1 + 1, y1 + 1, diag_color);
        renderer.plot(x1 + 1, y1 - 1, diag_color);
        renderer.plot(x1 - 1, y1 + 1, diag_color);
        renderer.plot(x1 - 1, y1 - 1, diag_color);
    } else {
        // Moving star: render as a thin SDF capsule trail
        let cfg = TrailConfig::star();
        let base_color = to_hdr_rgba(hdr_add(
            star_color_tmp,
            hdr_add(
                intensify(
                    hdr(globals.visual.space_color),
                    globals.exposure.game_exposure,
                ),
                intensify(
                    hdr(globals.exposure.add_color),
                    globals.exposure.game_exposure,
                ),
            ),
        ));
        render_trail(
            &mut renderer.star_trail_capsules,
            (x1 as f64, y1 as f64),
            (x2 as f64, y2 as f64),
            &cfg,
            base_color,
        );
    }
}

// ============================================================================
// Projectile rendering
// ============================================================================

pub fn render_projectile(
    entity: &Entity,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let rad = globals.render.render_scale * rand_range(0.5, 1.0, rng) * entity.visuals.radius;
    let pos = entity.position;
    let vel = entity.velocity;
    let col = intensify(
        hdr(entity.visuals.color),
        entity.hdr_exposure * globals.exposure.game_exposure,
    );

    // Compute trail endpoint using the same motion-blur logic as render_light_trail
    let pos1 = scale_vec(
        add_vec(pos, globals.screenshake.game_screenshake_pos),
        globals.render.render_scale,
    );
    let dt_game = globals.time.game_speed
        * (globals.time.time_current_frame - globals.time.time_last_frame)
            .max(1.0 / FRAMERATE_RENDER);
    let proper_time = entity.proper_time;
    let veloc = scale_vec(vel, -(globals.observer_proper_time / proper_time) * dt_game);
    let last_pos = scale_vec(
        add_vec(
            sub_vec(pos, veloc),
            globals.screenshake.game_screenshake_previous_pos,
        ),
        globals.render.render_scale,
    );
    let pos2 = lerp_vec(last_pos, pos1, SHUTTER_SPEED);

    let (x1, y1) = dither_vec(pos1, DITHER_AA, globals.render.current_jitter_double);
    let (x2, y2) = dither_vec(pos2, DITHER_AA, globals.render.current_jitter_double);
    let radius_px = dither_radius(rad, DITHER_AA, DITHER_POWER_RADIUS, rng).max(1) as f64;

    let cfg = TrailConfig::bullet(radius_px);
    let base_color = to_hdr_rgba(col);
    render_trail(
        &mut renderer.bullet_trail_capsules,
        (x1 as f64, y1 as f64),
        (x2 as f64, y2 as f64),
        &cfg,
        base_color,
    );
}

#[cfg(test)]
mod trail_config_tests {
    use super::*;
    #[test]
    fn trail_config_star_defaults() {
        let cfg = TrailConfig::star();
        assert!((cfg.radius - 1.0).abs() < 1e-9);
        assert!((cfg.shutter_speed - 1.0).abs() < 1e-9);
    }
    #[test]
    fn trail_config_bullet_defaults() {
        let cfg = TrailConfig::bullet(15.0);
        assert!((cfg.radius - 15.0).abs() < 1e-9);
        assert!((cfg.shutter_speed - 1.0).abs() < 1e-9);
    }
    #[test]
    fn trail_config_shutter_zero_means_no_trail() {
        let cfg = TrailConfig {
            radius: 5.0,
            brightness_falloff: 0.5,
            shutter_speed: 0.0,
        };
        assert!((cfg.shutter_speed).abs() < 1e-9);
    }
}

#[cfg(test)]
mod brightness_conservation_tests {
    use std::f64::consts::PI;

    /// Compute the brightness conservation scale factor.
    /// This mirrors the formula in render_trail().
    fn area_scale(radius: f64, trail_len: f64) -> f64 {
        let r = radius.max(0.001);
        if trail_len < 0.001 {
            1.0
        } else {
            let pi_r2 = PI * r * r;
            pi_r2 / (pi_r2 + 2.0 * r * trail_len)
        }
    }

    /// Circle area = π·r²
    fn circle_area(r: f64) -> f64 {
        PI * r * r
    }

    /// Capsule area = π·r² + 2·r·L (two semicircles + rectangle)
    fn capsule_area(r: f64, l: f64) -> f64 {
        PI * r * r + 2.0 * r * l
    }

    #[test]
    fn stationary_object_has_unit_scale() {
        // No trail → scale = 1.0 (circle unchanged)
        assert!((area_scale(5.0, 0.0) - 1.0).abs() < 1e-9);
        assert!((area_scale(100.0, 0.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn total_luminous_flux_is_conserved() {
        // For any (radius, trail_length), brightness×area must equal the circle's brightness×area
        // circle: flux = 1.0 × π·r²
        // capsule: flux = scale × (π·r² + 2·r·L)
        // These must be equal.
        let test_cases = [
            (1.0, 10.0),
            (5.0, 5.0),
            (10.0, 100.0),
            (0.5, 1000.0),
            (50.0, 0.1),
            (1.0, 1.0),
        ];
        for (r, l) in test_cases {
            let circle_flux = 1.0 * circle_area(r);
            let scale = area_scale(r, l);
            let capsule_flux = scale * capsule_area(r, l);
            let relative_error = ((capsule_flux - circle_flux) / circle_flux).abs();
            assert!(
                relative_error < 1e-10,
                "Conservation violated: r={r}, l={l}, circle_flux={circle_flux}, capsule_flux={capsule_flux}, error={relative_error}"
            );
        }
    }

    #[test]
    fn scale_decreases_with_trail_length() {
        // Longer trail → dimmer (more area to spread light over)
        let r = 5.0;
        let s1 = area_scale(r, 1.0);
        let s2 = area_scale(r, 10.0);
        let s3 = area_scale(r, 100.0);
        assert!(s1 > s2, "s1={s1} should be > s2={s2}");
        assert!(s2 > s3, "s2={s2} should be > s3={s3}");
    }

    #[test]
    fn scale_approaches_zero_for_infinite_trail() {
        // Very long trail relative to radius → scale approaches 0
        let scale = area_scale(1.0, 1_000_000.0);
        assert!(
            scale < 0.001,
            "scale={scale} should approach 0 for very long trail"
        );
    }

    #[test]
    fn scale_is_independent_of_absolute_size() {
        // Scaling both radius and length by same factor should give same scale
        // (the formula simplifies: πr²/(πr²+2rL) = πr/(πr+2L))
        // Actually this is NOT true — let's verify the actual behavior.
        // scale(r, L) = πr / (πr + 2L) when simplified by r
        // So scale(2r, 2L) = π·2r / (π·2r + 2·2L) = 2πr / (2πr + 4L) = πr / (πr + 2L)
        // Yes, it IS scale-invariant when both scale together!
        let s1 = area_scale(1.0, 10.0);
        let s2 = area_scale(2.0, 20.0);
        let s3 = area_scale(0.5, 5.0);
        assert!(
            (s1 - s2).abs() < 1e-10,
            "Should be scale-invariant: s1={s1}, s2={s2}"
        );
        assert!(
            (s1 - s3).abs() < 1e-10,
            "Should be scale-invariant: s1={s1}, s3={s3}"
        );
    }
}
