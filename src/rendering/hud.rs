use rand::prelude::*;

use crate::color::*;
use crate::game::GameState;
use crate::parameters::*;
use crate::rendering::Renderer2D;

// ============================================================================
// Vector font
// ============================================================================

/// Return the polygon (as list of relative coords in [0,1]x[0,1]) for a given character.
/// Each character is a single closed polyline matched exactly to the OCaml shape_char.
fn shape_char(c: char) -> Vec<(f64, f64)> {
    match c {
        '0' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.2),(0.25,0.2),(0.75,0.6),(0.75,0.8),(0.25,0.375),(0.25,0.8),(0.75,0.8),(0.75,0.2),(0.,0.2)],
        '1' => vec![(0.125,0.),(0.875,0.),(0.875,0.2),(0.625,0.2),(0.625,1.),(0.375,1.),(0.,0.75),(0.15,0.65),(0.375,0.8),(0.375,0.2),(0.125,0.2)],
        '2' => vec![(0.,0.),(1.,0.),(1.,0.2),(0.35,0.2),(1.,0.5),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.6),(0.25,0.6),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.,0.2)],
        '3' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.875,0.5),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.6),(0.25,0.6),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.5,0.6),(0.5,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.4),(0.,0.4),(0.,0.2)],
        '4' => vec![(0.5,0.),(0.75,0.),(0.75,1.),(0.5,1.),(0.,0.4),(0.,0.2),(1.,0.2),(1.,0.4),(0.25,0.4),(0.5,0.8)],
        '5' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.5),(0.25,0.7),(0.25,0.8),(1.,0.8),(1.,1.),(0.,1.),(0.,0.6),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.35),(0.,0.4),(0.,0.2),(0.25,0.)],
        '6' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.75,0.6),(0.25,0.6),(0.25,0.8),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.4),(0.,0.4),(0.,0.2)],
        '7' => vec![(0.25,0.),(0.5,0.),(1.,0.8),(1.,1.),(0.,1.),(0.,0.8),(0.75,0.8)],
        '8' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.875,0.5),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.25,0.6),(0.25,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,1.),(0.,0.8),(0.,0.6),(0.125,0.5),(0.,0.4),(0.,0.2)],
        '9' => vec![(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.6),(0.25,0.4),(0.75,0.4),(0.75,0.2),(0.,0.2),(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.6),(0.25,0.6),(0.25,0.8),(0.75,0.8),(0.75,0.6),(1.,0.6),(1.,0.8)],
        ' ' => vec![(0.,0.),(0.,0.),(0.,0.)],
        'A' => vec![(0.,0.),(0.25,0.),(0.25,0.4),(0.75,0.4),(0.75,0.4),(0.75,0.6),(0.25,0.6),(0.25,0.8),(0.75,0.8),(0.75,0.),(1.,0.),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8)],
        'B' => vec![(0.,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.875,0.5),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.25,0.6),(0.25,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.,1.)],
        'C' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.8),(0.75,0.8),(0.75,0.6),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.2)],
        'D' => vec![(0.,0.),(0.75,0.),(1.,0.2),(1.,0.8),(0.75,1.),(0.,1.),(0.,0.2),(0.25,0.2),(0.25,0.8),(0.75,0.8),(0.75,0.2),(0.,0.2)],
        'E' => vec![(0.,0.),(0.75,0.),(1.,0.2),(0.25,0.2),(0.25,0.4),(0.5,0.4),(0.5,0.6),(0.25,0.6),(0.25,0.8),(1.,0.8),(0.75,1.),(0.,1.)],
        'F' => vec![(0.,0.),(0.25,0.),(0.25,0.4),(0.5,0.4),(0.75,0.6),(0.25,0.6),(0.25,0.8),(1.,0.8),(1.,1.),(0.,1.)],
        'G' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.6),(0.5,0.6),(0.5,0.4),(0.75,0.4),(0.75,0.2),(0.25,0.2),(0.25,0.8),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.2)],
        'H' => vec![(0.,1.),(0.25,1.),(0.25,0.6),(0.75,0.6),(0.75,1.),(1.,1.),(1.,0.),(0.75,0.),(0.75,0.4),(0.25,0.4),(0.25,0.),(0.,0.)],
        'I' => vec![(0.125,0.),(0.875,0.),(0.875,0.2),(0.625,0.2),(0.625,0.8),(0.875,0.8),(0.875,1.),(0.125,1.),(0.125,0.8),(0.375,0.8),(0.375,0.2),(0.125,0.2)],
        'J' => vec![(0.25,1.),(0.5,1.),(0.75,0.8),(0.75,0.2),(1.,0.2),(1.,0.),(0.,0.),(0.,0.2),(0.25,0.2),(0.25,0.8),(0.,0.8)],
        'K' => vec![(0.,1.),(0.25,1.),(0.25,0.6),(0.75,1.),(1.,1.),(0.375,0.5),(1.,0.),(0.75,0.),(0.25,0.4),(0.25,0.),(0.,0.)],
        'L' => vec![(0.,0.),(0.,1.),(0.25,1.),(0.25,0.2),(1.,0.2),(1.,0.)],
        'M' => vec![(0.,1.),(0.25,1.),(0.5,0.6),(0.75,1.),(1.,1.),(1.,0.),(0.75,0.),(0.75,0.6),(0.5,0.2),(0.25,0.6),(0.25,0.),(0.,0.)],
        'N' => vec![(0.,1.),(0.25,1.),(0.75,0.4),(0.75,1.),(1.,1.),(1.,0.),(0.75,0.),(0.25,0.6),(0.25,0.),(0.,0.)],
        'O' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.2),(0.25,0.2),(0.25,0.8),(0.75,0.8),(0.75,0.2),(0.,0.2)],
        'P' => vec![(0.,0.),(0.25,0.),(0.25,0.5),(0.75,0.5),(1.,0.6),(1.,0.8),(0.75,1.),(0.25,1.),(0.,1.)],
        'Q' => vec![(0.25,1.),(0.75,1.),(1.,0.8),(1.,0.2),(0.75,0.),(0.25,0.),(0.,0.2),(0.,0.8),(0.25,0.8),(0.25,0.2),(0.75,0.2),(0.75,0.8),(0.,0.8),(0.5,0.6),(1.,1.)],
        'R' => vec![(0.,0.),(0.25,0.),(0.25,0.8),(0.75,0.8),(0.75,0.6),(0.25,0.6),(0.25,0.4),(0.75,0.),(1.,0.),(0.5,0.4),(0.75,0.4),(1.,0.6),(1.,0.8),(0.75,1.),(0.,1.)],
        'S' => vec![(0.25,0.),(0.75,0.),(1.,0.2),(1.,0.4),(0.75,0.6),(0.25,0.6),(0.25,0.8),(1.,0.8),(0.75,1.),(0.25,1.),(0.,0.8),(0.,0.6),(0.25,0.4),(0.75,0.4),(0.75,0.2),(0.,0.2)],
        'T' => vec![(0.385,0.),(0.625,0.),(0.625,0.8),(1.,0.8),(1.,1.),(0.,1.),(0.,0.8),(0.385,0.8)],
        'U' => vec![(0.,1.),(0.25,1.),(0.25,0.2),(0.75,0.2),(0.75,1.),(1.,1.),(1.,0.),(0.75,0.),(0.25,0.),(0.,0.)],
        'V' => vec![(0.,1.),(0.25,1.),(0.5,0.2),(0.75,1.),(1.,1.),(0.6,0.),(0.4,0.)],
        'W' => vec![(0.,1.),(0.2,0.),(0.4,0.),(0.5,0.2),(0.6,0.),(0.8,0.),(1.,1.),(0.6,0.4),(0.6,0.6),(0.4,0.6),(0.4,0.4),(0.2,1.)],
        'X' => vec![(0.,1.),(0.25,1.),(0.5,0.6),(0.75,1.),(1.,1.),(0.625,0.5),(1.,0.),(0.75,0.),(0.5,0.4),(0.25,0.),(0.,0.),(0.375,0.5)],
        'Y' => vec![(0.,0.),(0.25,0.),(0.5,0.4),(0.75,0.),(1.,0.),(0.625,0.6),(0.625,1.),(0.375,1.),(0.375,0.6)],
        'Z' => vec![(0.,1.),(1.,1.),(1.,0.8),(0.25,0.2),(1.,0.2),(0.75,0.),(0.,0.),(0.,0.2),(0.75,0.8),(0.,0.8)],
        ':' => vec![(0.3,0.8),(0.7,0.8),(0.7,0.6),(0.3,0.6),(0.3,0.4),(0.7,0.4),(0.7,0.2),(0.3,0.2),(0.3,0.4),(0.7,0.4)],
        '-' => vec![(0.1,0.6),(0.9,0.6),(0.9,0.4),(0.1,0.4)],
        '.' => vec![(0.3,1.),(0.7,1.),(0.7,0.8),(0.3,0.8)],
        '!' => vec![(0.35,1.),(0.65,1.),(0.65,0.8),(0.35,0.8),(0.35,0.65),(0.65,0.65),(0.65,0.),(0.35,0.)],
        _ => vec![(0.,0.),(1.,0.),(1.,1.),(0.,1.)],
    }
}

/// Map a relative coordinate (relx, rely) inside a bounding quad to screen pixels.
/// Matches OCaml `displacement`: bilinear interpolation across the 4 bounding points,
/// then multiply by render_scale.
/// Points: [p0=bottom-left, p1=bottom-right, p2=top-right, p3=top-left] (physical coords)
///
/// OCaml formula: lerp_vec a b ratio = a*ratio + b*(1-ratio)
/// top = lerp_vec(p2, p1, rely) = p2*rely + p1*(1-rely)
/// bot = lerp_vec(p3, p0, rely) = p3*rely + p0*(1-rely)
/// result = lerp_vec(top, bot, relx) = top*relx + bot*(1-relx)
fn displacement(
    encadrement: &[(f64, f64); 4],
    rel: (f64, f64),
    render_scale: f64,
) -> (f64, f64) {
    let (relx, rely) = rel;
    let [p0, p1, p2, p3] = encadrement;
    // lerp_vec a b ratio = a*ratio + b*(1-ratio)
    let top = (
        p2.0 * rely + p1.0 * (1.0 - rely),
        p2.1 * rely + p1.1 * (1.0 - rely),
    );
    let bot = (
        p3.0 * rely + p0.0 * (1.0 - rely),
        p3.1 * rely + p0.1 * (1.0 - rely),
    );
    let interp = (
        top.0 * relx + bot.0 * (1.0 - relx),
        top.1 * relx + bot.1 * (1.0 - relx),
    );
    (interp.0 * render_scale, interp.1 * render_scale)
}

/// Convert shape relative coords to screen pixel coords for a given bounding quad.
fn displace_shape(
    encadrement: &[(f64, f64); 4],
    shape: &[(f64, f64)],
    render_scale: f64,
) -> Vec<(i32, i32)> {
    shape
        .iter()
        .map(|&pt| {
            let (x, y) = displacement(encadrement, pt, render_scale);
            (x.round() as i32, y.round() as i32)
        })
        .collect()
}

/// Render a single character at the given bounding quad (physical coords), filled.
fn render_char(
    encadrement: &[(f64, f64); 4],
    c: char,
    color: [u8; 4],
    renderer: &mut Renderer2D,
    render_scale: f64,
) {
    let shape = shape_char(c);
    let pts = displace_shape(encadrement, &shape, render_scale);
    renderer.fill_poly(&pts, color);
}

/// Render a string of characters, each in a bounding quad advancing left to right.
/// Matches OCaml `render_characs` / `render_string`.
/// - pos: bottom-left start in physical coords
/// - l_char: char width in physical units
/// - h_char: char height in physical units
/// - l_space: spacing between chars in physical units
/// - shake: random position shake amplitude
pub fn render_string(
    s: &str,
    pos: (f64, f64),
    l_char: f64,
    h_char: f64,
    l_space: f64,
    shake: f64,
    color: [u8; 4],
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    use crate::math_utils::rand_range;
    let mut x0 = pos.0;
    let y0 = pos.1;
    for c in s.chars() {
        let c = c.to_ascii_uppercase();
        let sx0 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sy0 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sx1 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sy1 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sx2 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sy2 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sx3 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        let sy3 = if shake > 0.0 { rand_range(-shake, shake, rng) } else { 0.0 };
        // Bounding quad: [bottom-left, bottom-right, top-right, top-left]
        let encadrement: [(f64, f64); 4] = [
            (x0 + sx0, y0 + sy0),
            (x0 + sx1 + l_char, y0 + sy1),
            (x0 + sx2 + l_char, y0 + sy2 + h_char),
            (x0 + sx3, y0 + sy3 + h_char),
        ];
        render_char(&encadrement, c, color, renderer, globals.render.render_scale);
        x0 += l_char + l_space;
    }
}

// ============================================================================
// Bar and health display helpers
// ============================================================================

/// Fill a quadrilateral bar from 0 (empty) to ratio (full).
/// The quad is given as [p0, p1, p2, p3] in physical ratio coords [0,1],
/// where p0,p1 are the "zero" side and p2,p3 are the "full" side.
/// Matches OCaml `render_bar`.
fn render_bar(
    ratio: f64,
    quad: &[(f64, f64); 4],
    color: [u8; 4],
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    // relative_poly converts [0,1] coords to pixels: multiply by (width, height)
    // p0=quad[0], p1=quad[1], p2=quad[2], p3=quad[3]
    // For bar: use points p0, p1, lerp_vec(p2,p1,ratio), lerp_vec(p3,p0,ratio)
    let p0 = (quad[0].0 * globals.render.phys_width * globals.render.render_scale,
              quad[0].1 * globals.render.phys_height * globals.render.render_scale);
    let p1 = (quad[1].0 * globals.render.phys_width * globals.render.render_scale,
              quad[1].1 * globals.render.phys_height * globals.render.render_scale);
    let p2_full = (quad[2].0 * globals.render.phys_width * globals.render.render_scale,
                   quad[2].1 * globals.render.phys_height * globals.render.render_scale);
    let p3_full = (quad[3].0 * globals.render.phys_width * globals.render.render_scale,
                   quad[3].1 * globals.render.phys_height * globals.render.render_scale);

    // OCaml: lerp_vec p2 p1 ratio = p2*ratio + p1*(1-ratio)
    // ratio=1 → p2_full (full side) → full bar
    // ratio=0 → p1/p0 (zero side) → empty bar
    let p2 = (
        p2_full.0 * ratio + p1.0 * (1.0 - ratio),
        p2_full.1 * ratio + p1.1 * (1.0 - ratio),
    );
    let p3 = (
        p3_full.0 * ratio + p0.0 * (1.0 - ratio),
        p3_full.1 * ratio + p0.1 * (1.0 - ratio),
    );

    let pts: Vec<(i32, i32)> = vec![
        (p0.0.round() as i32, p0.1.round() as i32),
        (p1.0.round() as i32, p1.1.round() as i32),
        (p2.0.round() as i32, p2.1.round() as i32),
        (p3.0.round() as i32, p3.1.round() as i32),
    ];
    renderer.fill_poly(&pts, color);
}

/// Draw a heart shape: two ellipses + a diamond polygon.
/// Matches OCaml `draw_heart`.
/// pos0, pos1 are bounding box corners in physical coords.
fn draw_heart(
    pos0: (f64, f64),
    pos1: (f64, f64),
    color: [u8; 4],
    renderer: &mut Renderer2D,
    render_scale: f64,
) {
    // Scale to pixels
    let x0 = pos0.0 * render_scale;
    let y0 = pos0.1 * render_scale;
    let x1 = pos1.0 * render_scale;
    let y1 = pos1.1 * render_scale;

    let quartx = (x1 - x0) / 4.0;
    let tiery  = (y1 - y0) / 3.0;

    // Left ellipse center: (x0 + quartx, y1 - tiery)
    let lcx = (x0 + quartx + 0.5) as i32;
    let lcy = (y1 - tiery) as i32;
    let rx  = (quartx + 0.5) as i32;
    let ry  = (tiery  + 0.5) as i32;
    renderer.fill_ellipse(lcx, lcy, rx, ry, color);

    // Right ellipse center: (x1 - quartx, y1 - tiery)
    let rcx = (x1 - quartx + 0.5) as i32;
    renderer.fill_ellipse(rcx, lcy, rx, ry, color);

    // Diamond bottom polygon (matches OCaml fill_poly)
    let decal = 1.0 - (1.0 / 2.0_f64.sqrt());
    let pts: Vec<(i32, i32)> = vec![
        ((x0 + 2.0*quartx) as i32,                          y0 as i32),
        ((x0 + decal*quartx + 0.5) as i32,       (y0 + (1.0+decal)*tiery) as i32),
        ((x0 + 2.0*quartx) as i32,               (y1 - tiery) as i32),
        ((x1 - decal*quartx - 0.5) as i32,       (y0 + (1.0+decal)*tiery) as i32),
    ];
    renderer.fill_poly(&pts, color);
}

/// Render `n` hearts for the lives display. Matches OCaml `draw_n_hearts`.
fn draw_n_hearts(n: i32, color: [u8; 4], renderer: &mut Renderer2D, globals: &Globals) {
    let sx = globals.render.safe_offset_x;
    let sy = globals.render.safe_offset_y;
    let sw = globals.render.safe_phys_width;
    let sh = globals.render.safe_phys_height;
    let mut lastx = sx + 0.95 * sw;
    for _ in 0..n {
        draw_heart(
            (lastx - 0.03 * sw, sy + 0.75 * sh),
            (lastx,              sy + 0.80 * sh),
            color,
            renderer,
            globals.render.render_scale,
        );
        lastx -= 0.05 * sw;
    }
}

/// Draw a bar outline (quadrilateral frame) using draw_poly.
fn draw_bar_frame(
    quad: &[(f64, f64); 4],
    color: [u8; 4],
    line_width: f32,
    renderer: &mut Renderer2D,
    globals: &Globals,
) {
    let pts: Vec<(i32, i32)> = quad.iter().map(|&(rx, ry)| {
        (
            (rx * globals.render.phys_width  * globals.render.render_scale).round() as i32,
            (ry * globals.render.phys_height * globals.render.render_scale).round() as i32,
        )
    }).collect();
    renderer.draw_poly(&pts, color, line_width);
}

// ============================================================================
// HUD
// ============================================================================

/// Render scanlines effect: draw horizontal black lines every SCANLINES_PERIOD pixels
/// starting at `offset`, across the full screen width.
/// Imitates old CRT monitors that projected the image line by line.
pub fn render_scanlines(offset: i32, height: i32, renderer: &mut Renderer2D) {
    let width = renderer.width as i32;
    let mut y = offset;
    while y < height {
        renderer.fill_rect(0, y, width, 1, [0, 0, 0, 255]);
        y += SCANLINES_PERIOD;
    }
}

/// Render the full HUD. Matches OCaml `affiche_hud`.
/// Called at the END of render_frame (on top of everything).
pub fn render_hud(
    state: &GameState,
    globals: &Globals,
    renderer: &mut Renderer2D,
    rng: &mut impl Rng,
) {
    // Skip HUD in retro mode
    if globals.visual.retro {
        return;
    }

    // ----- Colors -----
    let red   : [u8; 4] = [255,  32,  32, 255];
    let orange: [u8; 4] = [255, 128,   0, 255];
    let dark_red: [u8; 4] = [32, 0, 0, 255];
    let cyan  : [u8; 4] = [  0, 192, 255, 255];
    let dark_blue: [u8; 4] = [0, 0, 32, 255];
    let yellow: [u8; 4] = [255, 220,  50, 255];
    let dark_yellow: [u8; 4] = [32, 16, 0, 255];
    let white : [u8; 4] = [255, 255, 255, 255];
    let frame_color: [u8; 4] = [64, 64, 64, 255];
    let frame_width: f32 = 10.0 * globals.render.render_scale as f32;

    // ----- Safe zone for HUD placement -----
    let sx = globals.render.safe_offset_x;
    let sy = globals.render.safe_offset_y;
    let sw = globals.render.safe_phys_width;
    let sh = globals.render.safe_phys_height;
    let pw = globals.render.phys_width;
    let ph = globals.render.phys_height;

    // Helper: convert safe-zone-relative fraction to full-screen fraction for render_bar
    // render_bar quads use fractions of phys_width/phys_height
    let fx = |frac: f64| -> f64 { (sx + frac * sw) / pw };
    let fy = |frac: f64| -> f64 { (sy + frac * sh) / ph };

    // ----- Hearts (lives) -----
    draw_n_hearts(state.lives, red, renderer, globals);

    // ----- Health bar -----
    // last_health tracks delayed (smooth) health
    let health_quad: [(f64, f64); 4] = [
        (fx(0.95), fy(0.9)),  (fx(0.95), fy(0.85)),
        (fx(0.6),  fy(0.85)), (fx(0.55), fy(0.9)),
    ];
    render_bar(1.0, &health_quad, dark_red,  renderer, globals);
    render_bar(
        (state.last_health / SHIP_MAX_HEALTH).min(1.0).max(0.0),
        &health_quad, orange, renderer, globals,
    );
    render_bar(
        (state.ship.health / SHIP_MAX_HEALTH).min(1.0).max(0.0),
        &health_quad, red, renderer, globals,
    );
    draw_bar_frame(&health_quad, frame_color, frame_width, renderer, globals);

    // ----- Teleport cooldown bar -----
    let tp_quad: [(f64, f64); 4] = [
        (fx(0.95), fy(0.7)),  (fx(0.95), fy(0.65)),
        (fx(0.8),  fy(0.65)), (fx(0.75), fy(0.7)),
    ];
    let tp_ratio = ((COOLDOWN_TP - state.cooldown_tp.max(0.0)) / COOLDOWN_TP).min(1.0).max(0.0);
    render_bar(1.0, &tp_quad, dark_blue, renderer, globals);
    render_bar(tp_ratio, &tp_quad, cyan, renderer, globals);
    draw_bar_frame(&tp_quad, frame_color, frame_width, renderer, globals);

    // Render 'F' indicator when teleport ready
    if state.cooldown_tp <= 0.0 {
        let encadrement: [(f64, f64); 4] = [
            (sx + 0.7  * sw, sy + 0.65 * sh),
            (sx + 0.72 * sw, sy + 0.65 * sh),
            (sx + 0.72 * sw, sy + 0.7  * sh),
            (sx + 0.7  * sw, sy + 0.7  * sh),
        ];
        render_char(&encadrement, 'F', cyan, renderer, globals.render.render_scale);
    }

    // ----- Weapon cooldown bar -----
    let weapon_quad: [(f64, f64); 4] = [
        (fx(0.95), fy(0.6)),  (fx(0.95), fy(0.55)),
        (fx(0.9),  fy(0.55)), (fx(0.85), fy(0.6)),
    ];
    let weapon_ratio = ((globals.weapon.projectile_cooldown - state.cooldown.max(0.0)) / globals.weapon.projectile_cooldown)
        .min(1.0)
        .max(0.0);
    render_bar(1.0, &weapon_quad, dark_yellow, renderer, globals);
    render_bar(weapon_ratio, &weapon_quad, yellow, renderer, globals);
    draw_bar_frame(&weapon_quad, frame_color, frame_width, renderer, globals);

    // ----- Score -----
    // Color: warm amber/orange, dimmed by shake_score
    let score_intensity = 1.0 / (1.0 + 10.0 * globals.screenshake.shake_score);
    let score_col = rgb_of_hdr(
        intensify(HdrColor::new(50000.0, 1000.0, 300.0), score_intensity),
        &HdrColor::zero(),
        &HdrColor::one(),
        1.0,
    );
    let score_str = format!("SCORE {}", state.score);
    let shake = globals.screenshake.shake_score * 7.0;
    let base_l_char = (1.0 + 0.05 * globals.screenshake.shake_score) * 0.03 * sw;
    let base_h_char = (1.0 + 0.05 * globals.screenshake.shake_score) * 0.08 * sh;
    let base_l_space = (1.0 + 0.05 * globals.screenshake.shake_score) * 0.01 * sw;
    let score_y = sy + 0.82 * sh * (1.0 - 0.05 * globals.screenshake.shake_score * 0.08);
    render_string(
        &score_str,
        (sx + 0.02 * sw, score_y),
        base_l_char,
        base_h_char,
        base_l_space,
        shake,
        score_col,
        renderer,
        globals,
        rng,
    );

    // ----- Stage -----
    let stage_str = format!("STAGE {}", state.stage);
    render_string(
        &stage_str,
        (sx + 0.02 * sw, sy + 0.7 * sh),
        0.02 * sw,
        0.05 * sh,
        0.01 * sw,
        0.0,
        white,
        renderer,
        globals,
        rng,
    );

    // ----- Death countdown -----
    // Show countdown when ship health <= 0
    if state.ship.health <= 0.0 {
        let time_until_explo = globals.time.time_of_death + TIME_STAY_DEAD_MAX - globals.time.time_current_frame;
        if time_until_explo > 0.0 {
            // Flash: show the integer countdown, alternating on/off at 0.5s boundary
            let frac = time_until_explo - time_until_explo.floor();
            if frac > 0.5 {
                let count_str = format!("{}", (time_until_explo + 1.0) as i32);
                render_string(
                    &count_str,
                    (sx + 0.42 * sw, sy + 0.3 * sh),
                    0.16 * sw,
                    0.4  * sh,
                    0.01 * sw,
                    0.0,
                    white,
                    renderer,
                    globals,
                    rng,
                );
            }
        }
    }

    // ----- Debug stats (half size) -----
    let debug_x = sx + 0.01 * sw;
    let debug_l = 0.006 * sw;
    let debug_h = 0.0125 * sh;
    let debug_sp = 0.0015 * sw;
    let debug_color = white;

    let nb_objets   = state.objects.len()   + state.objects_oos.len();
    let nb_toosmall = state.toosmall.len()  + state.toosmall_oos.len();
    let nb_frags    = state.fragments.len();
    let nb_projs    = state.projectiles.len();
    let nb_explos   = state.explosions.len();
    let nb_smoke    = state.smoke.len()     + state.smoke_oos.len();
    let nb_chunks   = state.chunks.len()    + state.chunks_oos.len();
    let nb_chunks_e = state.chunks_explo.len();

    let fps = if globals.framerate.time_current_count - globals.framerate.time_last_count > 0.0 {
        (globals.framerate.last_count as f64).round() as i32
    } else {
        0
    };

    let peak_fps = if globals.framerate.frame_compute_secs > 0.0 {
        (1.0 / globals.framerate.frame_compute_secs).round() as i32
    } else {
        0
    };

    let debug_lines = [
        format!("FPS        : {}", fps),
        format!("Peak FPS   : {}", peak_fps),
        format!("Objets     : {}", nb_objets),
        format!("TooSmall   : {}", nb_toosmall),
        format!("Frags      : {}", nb_frags),
        format!("Projectiles: {}", nb_projs),
        format!("Explosions : {}", nb_explos),
        format!("Smoke      : {}", nb_smoke),
        format!("Chunks     : {}", nb_chunks),
        format!("ChunksExplo: {}", nb_chunks_e),
    ];

    for (i, line) in debug_lines.iter().enumerate() {
        let y = (debug_h + debug_sp) * i as f64 + debug_sp;
        render_string(
            line,
            (debug_x, y),
            debug_l,
            debug_h,
            debug_sp * 0.5,
            0.0,
            debug_color,
            renderer,
            globals,
            rng,
        );
    }
}
