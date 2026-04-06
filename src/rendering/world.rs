use rand::prelude::*;

use crate::color::*;
use crate::game::{hdr, to_hdr_rgba};
use crate::math_utils::*;
use crate::objects::*;
use crate::parameters::*;
use crate::rendering::Renderer2D;

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
        let r = dither_radius(
            visuals.radius * globals.render.render_scale,
            DITHER_AA,
            DITHER_POWER_RADIUS,
            rng,
        );
        let falloff = if entity.kind == EntityKind::Smoke { 0.2 } else { 0.0 };
        renderer.push_circle_instance(x as f32, y as f32, r.max(1) as f32, color, falloff);
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
        add_vec(entity.position, globals.screenshake.game_screenshake_pos),
        globals.render.render_scale,
    );
    let intensity_chunk = 1.0;
    let color = to_hdr_rgba(intensify(
        hdr(entity.visuals.color),
        intensity_chunk * globals.exposure.game_exposure * entity.hdr_exposure,
    ));
    let (x, y) = dither_vec(pos, DITHER_AA, globals.render.current_jitter_double);
    let r = dither_radius(
        globals.render.render_scale * entity.visuals.radius,
        DITHER_AA,
        DITHER_POWER_RADIUS,
        rng,
    );
    renderer.push_circle_instance(x as f32, y as f32, r.max(1) as f32, color, 0.0);
}

/// Render a star with motion trail
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
        let dist = magnitude(sub_vec(pos1, pos2));
        let trail_lum = (1.0 / (1.0 + dist)).sqrt();
        let trail_color = hdr_add(
            intensify(star_color_tmp, trail_lum),
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
        );
        let color = to_hdr_rgba(trail_color);
        renderer.push_capsule_instance(x1 as f32, y1 as f32, x2 as f32, y2 as f32, 1.0, color);
    }
}

// ============================================================================
// Projectile rendering
// ============================================================================

/// Render a projectile as an SDF capsule (motion-blur trail). Ported from OCaml render_projectile.
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

    let dist = magnitude(sub_vec(pos1, pos2));
    let trail_lum = 0.5 * (rad / (rad + dist)).sqrt();
    let color = to_hdr_rgba(intensify(col, trail_lum));

    let (x1, y1) = dither_vec(pos1, DITHER_AA, globals.render.current_jitter_double);
    let (x2, y2) = dither_vec(pos2, DITHER_AA, globals.render.current_jitter_double);
    let radius = dither_radius(rad, DITHER_AA, DITHER_POWER_RADIUS, rng).max(1) as f32;

    renderer.push_capsule_instance(x1 as f32, y1 as f32, x2 as f32, y2 as f32, radius, color);
}
