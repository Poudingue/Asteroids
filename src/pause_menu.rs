use rand::prelude::*;

use crate::game::GameState;
use crate::rendering::hud::render_string;
use crate::parameters::{GlobalToggle, Globals};
use crate::rendering::Renderer2D;

// ============================================================================
// Pause menu button system
// ============================================================================

/// An interactive toggle button for the pause screen.
/// Matches OCaml `button_bool` record.
pub struct ButtonBoolean {
    /// Bottom-left corner in physical coordinates.
    pub pos1: (f64, f64),
    /// Top-right corner in physical coordinates.
    pub pos2: (f64, f64),
    /// Label rendered inside the button.
    pub text: &'static str,
    /// Tooltip rendered near the mouse when hovering.
    pub text_over: &'static str,
    /// Which `Globals` boolean field this button controls.
    pub field: GlobalToggle,
    /// Left-mouse state from the previous frame (for rising-edge detection).
    pub last_mouse_state: bool,
}

// ============================================================================
// Pause button helpers
// ============================================================================

/// Build the full list of pause-screen buttons.
/// Positions are computed as fractions of the 16:9 safe zone on a 16×24 grid.
pub fn make_buttons(globals: &Globals) -> Vec<ButtonBoolean> {
    let sx = globals.render.safe_offset_x;
    let sy = globals.render.safe_offset_y;
    let w = globals.render.safe_phys_width;
    let h = globals.render.safe_phys_height;
    // Macro-style helper: build one ButtonBoolean from grid fractions
    macro_rules! btn {
        ($text:expr, $text_over:expr,
         $c1:expr, $r1:expr, $c2:expr, $r2:expr,
         $field:expr) => {
            ButtonBoolean {
                pos1: (sx + $c1 / 16.0 * w, sy + $r1 / 24.0 * h),
                pos2: (sx + $c2 / 16.0 * w, sy + $r2 / 24.0 * h),
                text: $text,
                text_over: $text_over,
                field: $field,
                last_mouse_state: false,
            }
        };
    }
    vec![
        btn!("quit",             "Quit the game and go outside",
             10.0, 20.0, 12.0, 22.0, GlobalToggle::Quit),
        btn!("resume",           "Resume current game",
             7.0,  20.0,  9.0, 22.0, GlobalToggle::Pause),
        btn!("New Game",         "Start a new game with current parameters",
             4.0,  20.0,  6.0, 22.0, GlobalToggle::Restart),
        btn!("scanlines",        "Imitates the look of old CRT monitors.\nLowers luminosity.",
             10.0, 12.0, 12.0, 14.0, GlobalToggle::Scanlines),
        btn!("retro visuals",    "White vectors on black background design",
             7.0,  12.0,  9.0, 14.0, GlobalToggle::Retro),
        btn!("Advanced hitbox",  "A more precise hitbox.",
             10.0,  9.0, 12.0, 11.0, GlobalToggle::AdvancedHitbox),
        btn!("smoke particles",  "Allows smoke. Disable for better performance.",
             7.0,   6.0,  9.0,  8.0, GlobalToggle::Smoke),
        btn!("screenshake",      "Feel the impacts and explosions.",
             4.0,   6.0,  6.0,  8.0, GlobalToggle::Screenshake),
        btn!("Light Flashes",    "Activates light flashes for events",
             10.0,  6.0, 12.0,  8.0, GlobalToggle::Flashes),
        btn!("chunk particles",  "Allows chunks. Disable for better performance.",
             7.0,   3.0,  9.0,  5.0, GlobalToggle::Chunks),
        btn!("Color Effects",    "Color changes and correction",
             10.0,  3.0, 12.0,  5.0, GlobalToggle::DynColor),
    ]
}

/// Convert screen Y (SDL2, Y-down) to physical Y (Y-up).
#[inline]
fn screen_to_phys_y(screen_y: f64, globals: &Globals) -> f64 {
    globals.render.phys_height - screen_y / globals.render.render_scale
}

/// Render and process one pause-screen button.
/// `mouse_sx`, `mouse_sy`: raw SDL2 screen coordinates (Y-down).
/// `mouse_down`: is the left mouse button currently pressed?
///
/// Returns `true` if the button was just clicked (rising edge).
pub fn apply_button(
    btn: &mut ButtonBoolean,
    globals: &mut Globals,
    renderer: &mut Renderer2D,
    rng: &mut impl Rng,
    mouse_sx: f64,
    mouse_sy: f64,
    mouse_down: bool,
) {
    let rr = globals.render.render_scale;

    // Physical mouse position (Y-flipped)
    let mx = mouse_sx / rr;
    let my = screen_to_phys_y(mouse_sy, globals);

    // Button bounds in physical coords
    let (x1, y1) = btn.pos1;
    let (x2, y2) = btn.pos2;

    let hovered = mx >= x1 && mx <= x2 && my >= y1 && my <= y2;

    // Current toggle value
    let on = globals.get_toggle(&btn.field);

    // ---- Rendering ----
    // Pixel coords in Y-up space (matching the vertex shader and fill_poly)
    let px1 = (x1 * rr).round() as i32;
    let px2 = (x2 * rr).round() as i32;
    let py1 = (y1 * rr).round() as i32;  // bottom in Y-up
    let py2 = (y2 * rr).round() as i32;  // top in Y-up

    // fill_poly rect: bottom-left, bottom-right, top-right, top-left
    let rect_pts = vec![(px1, py1), (px2, py1), (px2, py2), (px1, py2)];

    if globals.visual.retro {
        // Retro mode: white fill if ON, black fill if OFF
        let fill_col = if on { [255u8, 255, 255, 255] } else { [0u8, 0, 0, 255] };
        renderer.fill_poly(&rect_pts, fill_col);
        // White frame
        renderer.draw_poly(&rect_pts, [255, 255, 255, 255], 2.0 * rr as f32);
    } else {
        // Normal mode
        let fill_col: [u8; 4] = if on { [0, 128, 0, 255] } else { [128, 0, 0, 255] };
        renderer.fill_poly(&rect_pts, fill_col);

        // Border: dark grey, 10 * render_scale px wide
        let border_w = 10.0 * rr as f32;
        renderer.draw_poly(&rect_pts, [64, 64, 64, 255], border_w);
    }

    // ---- Centered text (both modes) ----
    // Uniform character size based on safe zone (like HUD text), not button dimensions
    let sh = globals.render.safe_phys_height;
    let char_h = 0.02 * sh;
    let char_w = char_h * 0.6;  // fixed aspect ratio
    let char_sp = char_w * 0.15;
    let text_total_w = btn.text.len() as f64 * (char_w + char_sp) - char_sp;
    // Center text in button
    let text_x = x1 + ((x2 - x1) - text_total_w) * 0.5;
    let text_y = y1 + ((y2 - y1) - char_h) * 0.5;
    let text_col = if globals.visual.retro {
        if on { [0u8, 0, 0, 255] } else { [255u8, 255, 255, 255] }
    } else {
        [255, 255, 255, 255]
    };
    if !globals.visual.retro {
        // Shadow: offset by -1 phys unit
        render_string(
            btn.text, (text_x - 1.0, text_y - 1.0),
            char_w, char_h, char_sp, 0.0,
            [0, 0, 0, 255], renderer, globals, rng,
        );
    }
    render_string(
        btn.text, (text_x, text_y),
        char_w, char_h, char_sp, 0.0,
        text_col, renderer, globals, rng,
    );

    // ---- Click detection (rising edge) ----
    if mouse_down && !btn.last_mouse_state && hovered {
        let new_val = !globals.get_toggle(&btn.field);
        globals.set_toggle(&btn.field, new_val);
    }
    btn.last_mouse_state = mouse_down;
}

/// Render only the hover tooltip for a button (second-pass, always on top).
pub fn render_button_tooltip(
    btn: &ButtonBoolean,
    globals: &mut Globals,
    renderer: &mut Renderer2D,
    rng: &mut impl Rng,
    mouse_sx: f64,
    mouse_sy: f64,
) {
    let rr = globals.render.render_scale;
    let mx = mouse_sx / rr;
    let my = screen_to_phys_y(mouse_sy, globals);

    let (x1, y1) = btn.pos1;
    let (x2, y2) = btn.pos2;
    let hovered = mx >= x1 && mx <= x2 && my >= y1 && my <= y2;

    if hovered {
        let sw = globals.render.safe_phys_width;
        let sh = globals.render.safe_phys_height;
        let tip_x = mx + 0.5;
        let tip_y = my + 0.5;
        let tip_char_w = 0.009 * sw;
        let tip_char_h = 0.018 * sh;
        let tip_sp     = 0.002 * sw;
        render_string(
            btn.text_over, (tip_x - 1.0, tip_y - 1.0),
            tip_char_w, tip_char_h, tip_sp, 0.0,
            [0, 0, 0, 255], renderer, globals, rng,
        );
        render_string(
            btn.text_over, (tip_x, tip_y),
            tip_char_w, tip_char_h, tip_sp, 0.0,
            [255, 255, 255, 255], renderer, globals, rng,
        );
    }
}

/// Render the pause screen title "ASTEROIDS" and process all pause buttons.
/// Matches OCaml `affiche_hud` pause block.
/// `mouse_sx`, `mouse_sy`: raw SDL2 screen coordinates (Y-down).
/// `mouse_down`: left mouse button state.
pub fn render_pause_title(
    state: &mut GameState,
    globals: &mut Globals,
    renderer: &mut Renderer2D,
    mouse_sx: f64,
    mouse_sy: f64,
    mouse_down: bool,
) {
    // Safe zone for pause menu positioning
    let sx = globals.render.safe_offset_x;
    let sy = globals.render.safe_offset_y;
    let sw = globals.render.safe_phys_width;
    let sh = globals.render.safe_phys_height;

    // Shadow (black, slightly offset)
    let shadow_col = [0u8, 0, 0, 255];
    // We need to split borrow: rng from state, but buttons also in state.
    // Render title shadow first (no buttons yet).
    {
        let rng = &mut state.rng as *mut _;
        render_string(
            "ASTEROIDS",
            (sx + (2.1/16.0) * sw, sy + (14.7/24.0) * sh),
            (1.0/16.0) * sw,
            (4.0/24.0) * sh,
            (1.0/40.0) * sw,
            0.0,
            shadow_col,
            renderer,
            globals,
            unsafe { &mut *rng },
        );
    }

    // Phase 1: render all button backgrounds + text + click detection
    let btn_count = state.buttons.len();
    for i in 0..btn_count {
        let rng = &mut state.rng as *mut _;
        let btn = &mut state.buttons[i] as *mut ButtonBoolean;
        apply_button(
            unsafe { &mut *btn },
            globals,
            renderer,
            unsafe { &mut *rng },
            mouse_sx,
            mouse_sy,
            mouse_down,
        );
    }

    // Phase 2: render tooltips on top of all buttons
    for i in 0..btn_count {
        let rng = &mut state.rng as *mut _;
        let btn = &state.buttons[i] as *const ButtonBoolean;
        render_button_tooltip(
            unsafe { &*btn },
            globals,
            renderer,
            unsafe { &mut *rng },
            mouse_sx,
            mouse_sy,
        );
    }

    // White title on top of everything
    {
        let rng = &mut state.rng as *mut _;
        render_string(
            "ASTEROIDS",
            (sx + (2.0/16.0) * sw, sy + (15.0/24.0) * sh),
            (1.0/16.0) * sw,
            (4.0/24.0) * sh,
            (1.0/40.0) * sw,
            0.0,
            [255, 255, 255, 255],
            renderer,
            globals,
            unsafe { &mut *rng },
        );
    }
}
