use rand::prelude::*;

use crate::color::*;
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;
use crate::rendering::Renderer2D;
use crate::game::{hdr, to_rgba};

// ============================================================================
// Polygon rendering helpers
// ============================================================================

/// Render a single polar polygon at a position with rotation and color
pub fn render_poly(
    poly: &[(f64, f64)],
    pos: Vec2,
    rotat: f64,
    color: [u8; 4],
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    let affine = polygon_to_cartesian(poly, rotat, globals.render_scale);
    let displaced = translate_polygon(&affine, pos);
    let screen_points: Vec<(i32, i32)> = displaced
        .iter()
        .map(|&p| dither_vec(p, DITHER_AA, globals.current_jitter_double))
        .collect();
    if globals.retro {
        renderer.draw_poly(&screen_points, [255, 255, 255, 255], 1.0);
    } else {
        renderer.fill_poly(&screen_points, color);
    }
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
        let color = to_rgba(intensify(hdr(*col), exposure), globals);
        render_poly(poly, pos, rotat, color, renderer, globals);
    }
}

// ============================================================================
// Entity rendering
// ============================================================================

/// Render an entity: base circle (if not retro) + polygon shapes
pub fn render_visuals(
    entity: &Entity,
    offset: Vec2,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let visuals = &entity.visuals;
    let position = scale_vec(
        add_vec(
            add_vec(entity.position, globals.game_screenshake_pos),
            offset,
        ),
        globals.render_scale,
    );
    let exposure = globals.game_exposure * entity.hdr_exposure;

    // Base circle (not in retro mode)
    if visuals.radius > 0.0 && !globals.retro {
        let color = to_rgba(intensify(hdr(visuals.color), exposure), globals);
        let (x, y) = dither_vec(position, DITHER_AA, globals.current_jitter_double);
        let r = dither_radius(
            visuals.radius * globals.render_scale,
            DITHER_AA,
            DITHER_POWER_RADIUS,
            rng,
        );
        renderer.fill_circle(x as f64, y as f64, r.max(1) as f64, color);
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
pub fn render_chunk(
    entity: &Entity,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let pos = scale_vec(
        add_vec(entity.position, globals.game_screenshake_pos),
        globals.render_scale,
    );
    if globals.retro {
        let (x, y) = dither_vec(pos, DITHER_AA, globals.current_jitter_double);
        renderer.fill_circle(
            x as f64, y as f64,
            (0.25 * globals.render_scale * entity.visuals.radius).max(1.0),
            [128, 128, 128, 255],
        );
    } else {
        let intensity_chunk = 1.0;
        let color = to_rgba(
            intensify(hdr(entity.visuals.color), intensity_chunk * globals.game_exposure * entity.hdr_exposure),
            globals,
        );
        let (x, y) = dither_vec(pos, DITHER_AA, globals.current_jitter_double);
        let r = dither_radius(
            globals.render_scale * entity.visuals.radius,
            DITHER_AA, DITHER_POWER_RADIUS, rng,
        );
        renderer.fill_circle(x as f64, y as f64, r.max(1) as f64, color);
    }
}

/// Render a star with motion trail
pub fn render_star_trail(
    star: &Star,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let pos1 = scale_vec(
        add_vec(star.pos, globals.game_screenshake_pos),
        globals.render_scale,
    );
    let last_position = scale_vec(
        add_vec(star.last_pos, globals.game_screenshake_previous_pos),
        globals.render_scale,
    );
    let pos2 = lerp_vec(last_position, pos1, SHUTTER_SPEED);
    let (x1, y1) = dither_vec(pos1, DITHER_AA, globals.current_jitter_double);
    let (x2, y2) = dither_vec(pos2, DITHER_AA, globals.current_jitter_double);

    let lum = if globals.pause {
        star.lum + 0.5 * STAR_RAND_LUM
    } else {
        star.lum + rng.gen::<f64>() * STAR_RAND_LUM
    };

    let star_color_tmp = intensify(hdr(globals.star_color), lum * globals.game_exposure);

    if x1 == x2 && y1 == y2 {
        // Static star: render as a cross of pixels
        let center_color = to_rgba(
            intensify(
                hdr_add(
                    star_color_tmp,
                    hdr(globals.space_color),
                ),
                globals.game_exposure,
            ),
            globals,
        );
        renderer.plot(x1, y1, center_color);

        let arm_color = to_rgba(intensify(star_color_tmp, 0.25), globals);
        renderer.plot(x1 + 1, y1, arm_color);
        renderer.plot(x1 - 1, y1, arm_color);
        renderer.plot(x1, y1 + 1, arm_color);
        renderer.plot(x1, y1 - 1, arm_color);

        let diag_color = to_rgba(intensify(star_color_tmp, 0.125), globals);
        renderer.plot(x1 + 1, y1 + 1, diag_color);
        renderer.plot(x1 + 1, y1 - 1, diag_color);
        renderer.plot(x1 - 1, y1 + 1, diag_color);
        renderer.plot(x1 - 1, y1 - 1, diag_color);
    } else {
        // Moving star: render as a line trail
        let dist = magnitude(sub_vec(pos1, pos2));
        let trail_lum = (1.0 / (1.0 + dist)).sqrt();
        let trail_color = hdr_add(
            intensify(star_color_tmp, trail_lum),
            hdr_add(
                intensify(hdr(globals.space_color), globals.game_exposure),
                intensify(hdr(globals.add_color), globals.game_exposure),
            ),
        );
        let color = to_rgba(trail_color, globals);
        renderer.draw_line(x1, y1, x2, y2, color, 2.0);
    }
}

// ============================================================================
// Projectile rendering
// ============================================================================

/// Render a light trail (motion blur line) for a fast-moving entity.
/// Used for projectiles. Ported from OCaml render_light_trail.
pub fn render_light_trail(
    radius: f64,
    pos: Vec2,
    velocity: Vec2,
    hdr_color: HdrColor,
    proper_time: f64,
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    let pos1 = scale_vec(add_vec(pos, globals.game_screenshake_pos), globals.render_scale);
    let dt_game = globals.game_speed
        * (globals.time_current_frame - globals.time_last_frame)
            .max(1.0 / FRAMERATE_RENDER);
    let veloc = scale_vec(velocity, -(globals.observer_proper_time / proper_time) * dt_game);
    let last_pos = scale_vec(
        add_vec(sub_vec(pos, veloc), globals.game_screenshake_previous_pos),
        globals.render_scale,
    );
    let pos2 = lerp_vec(last_pos, pos1, SHUTTER_SPEED);
    let dist = magnitude(sub_vec(pos1, pos2));
    let trail_lum = 0.5 * (radius / (radius + dist)).sqrt();
    let color = to_rgba(intensify(hdr_color, trail_lum), globals);
    let (x1, y1) = dither_vec(pos1, DITHER_AA, globals.current_jitter_double);
    let (x2, y2) = dither_vec(pos2, DITHER_AA, globals.current_jitter_double);
    let line_width = dither_radius(2.0 * radius, DITHER_AA, DITHER_POWER_RADIUS, &mut rand::thread_rng());
    renderer.draw_line(x1, y1, x2, y2, color, line_width.max(1) as f32);
}

/// Render a projectile as four concentric light trails. Ported from OCaml render_projectile.
pub fn render_projectile(entity: &Entity, renderer: &mut Renderer2D, globals: &Globals, rng: &mut impl Rng) {
    let rad = globals.render_scale
        * rand_range(0.5, 1.0, rng)
        * entity.visuals.radius;
    if globals.retro {
        // Retro mode: simple white filled circle at projectile position
        let pos = scale_vec(entity.position, globals.render_scale);
        let (x, y) = dither_vec(pos, DITHER_AA, globals.current_jitter_double);
        renderer.fill_circle(x as f64, y as f64, rad.max(1.0), [255, 255, 255, 255]);
    } else {
        let pos = entity.position;
        let vel = entity.velocity;
        let col = intensify(hdr(entity.visuals.color), entity.hdr_exposure * globals.game_exposure);
        let pt = entity.proper_time;
        render_light_trail(rad,        pos, vel, intensify(col, 0.25), pt, renderer, globals);
        render_light_trail(rad * 0.75, pos, vel, intensify(col, 0.5),  pt, renderer, globals);
        render_light_trail(rad * 0.5,  pos, vel, col,                  pt, renderer, globals);
        render_light_trail(rad * 0.25, pos, vel, intensify(col, 2.0),  pt, renderer, globals);
    }
}
