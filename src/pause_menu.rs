use rand::Rng;

use crate::parameters::{GlobalToggle, Globals};
use crate::rendering::hud::render_string;
use crate::rendering::Renderer2D;

// ============================================================================
// Data model
// ============================================================================

/// What kind of interactive entry this is.
pub enum MenuEntryKind {
    /// An action that sets a GlobalToggle to true (e.g. Resume, Quit).
    Action(GlobalToggle),
    /// A boolean toggle stored in Globals.
    Toggle(GlobalToggle),
    /// A discrete cycle through named values.
    Cycle {
        labels: &'static [&'static str],
        /// Read current index from Globals.
        get: fn(&Globals) -> usize,
        /// Write new index to Globals.
        set: fn(&mut Globals, usize),
    },
    /// A continuous slider with min/max/step.
    Slider {
        min: f64,
        max: f64,
        step: f64,
        /// Read current value from Globals.
        get: fn(&Globals) -> f64,
        /// Write new value to Globals.
        set: fn(&mut Globals, f64),
    },
    /// A visual separator (non-interactive).
    Separator,
}

/// One row in the pause menu.
pub struct MenuEntry {
    pub label: &'static str,
    pub kind: MenuEntryKind,
}

/// Whether a separator should only be drawn when HDR is enabled.
pub struct PauseMenu {
    pub entries: Vec<MenuEntry>,
    /// Index of the currently selected (keyboard/gamepad) entry. Points to a
    /// selectable entry (not a separator).
    pub selected: usize,
    /// First visible row index (for scrolling).
    pub scroll_offset: usize,
    /// Number of entries that fit on screen at once.
    pub visible_rows: usize,
    /// Index of the entry being dragged (slider), or None.
    pub dragging_entry: Option<usize>,
    /// Last frame's mouse button state (for rising-edge detection).
    pub last_mouse_down: bool,
}

// ============================================================================
// MSAA cycle helpers
// ============================================================================

fn msaa_get(globals: &Globals) -> usize {
    match globals.hdr.msaa_sample_count {
        1 => 0,
        _ => 1, // 4 or anything else → x4
    }
}

fn msaa_set(globals: &mut Globals, idx: usize) {
    globals.hdr.msaa_sample_count = match idx {
        0 => 1,
        _ => 4,
    };
}

// ============================================================================
// PauseMenu construction
// ============================================================================

impl Default for PauseMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl PauseMenu {
    pub fn new() -> Self {
        let entries = vec![
            MenuEntry {
                label: "Resume",
                kind: MenuEntryKind::Action(GlobalToggle::Pause),
            },
            MenuEntry {
                label: "New Game",
                kind: MenuEntryKind::Action(GlobalToggle::Restart),
            },
            MenuEntry {
                label: "Quit",
                kind: MenuEntryKind::Action(GlobalToggle::Quit),
            },
            MenuEntry {
                label: "",
                kind: MenuEntryKind::Separator,
            },
            MenuEntry {
                label: "Advanced Hitbox",
                kind: MenuEntryKind::Toggle(GlobalToggle::AdvancedHitbox),
            },
            MenuEntry {
                label: "Smoke Particles",
                kind: MenuEntryKind::Toggle(GlobalToggle::Smoke),
            },
            MenuEntry {
                label: "Screenshake",
                kind: MenuEntryKind::Toggle(GlobalToggle::Screenshake),
            },
            MenuEntry {
                label: "Light Flashes",
                kind: MenuEntryKind::Toggle(GlobalToggle::Flashes),
            },
            MenuEntry {
                label: "Chunk Particles",
                kind: MenuEntryKind::Toggle(GlobalToggle::Chunks),
            },
            MenuEntry {
                label: "Color Effects",
                kind: MenuEntryKind::Toggle(GlobalToggle::DynColor),
            },
            MenuEntry {
                label: "",
                kind: MenuEntryKind::Separator,
            },
            MenuEntry {
                label: "MSAA",
                kind: MenuEntryKind::Cycle {
                    labels: &["Off", "x4"],
                    get: msaa_get,
                    set: msaa_set,
                },
            },
            MenuEntry {
                label: "SMAA",
                kind: MenuEntryKind::Toggle(GlobalToggle::Smaa),
            },
            MenuEntry {
                label: "Tonemap",
                kind: MenuEntryKind::Cycle {
                    labels: &[
                        "Passthrough",
                        "Pseudo-Reinhard",
                        "Hard Redirect",
                        "Soft Redirect",
                    ],
                    get: |g| g.hdr.tonemap_variant as usize,
                    set: |g, idx| g.hdr.tonemap_variant = idx as u32,
                },
            },
            MenuEntry {
                label: "HDR",
                kind: MenuEntryKind::Toggle(GlobalToggle::Hdr),
            },
            MenuEntry {
                label: "",
                kind: MenuEntryKind::Separator,
            },
            MenuEntry {
                label: "HUD Nits",
                kind: MenuEntryKind::Slider {
                    min: 50.0,
                    max: 500.0,
                    step: 5.0,
                    get: |g| g.hdr.hud_nits,
                    set: |g, v| g.hdr.hud_nits = v,
                },
            },
            MenuEntry {
                label: "Exposure",
                kind: MenuEntryKind::Slider {
                    min: 0.1,
                    max: 4.0,
                    step: 0.1,
                    get: |g| g.hdr.exposure,
                    set: |g, v| g.hdr.exposure = v,
                },
            },
            MenuEntry {
                label: "Max Brightness",
                kind: MenuEntryKind::Slider {
                    min: 400.0,
                    max: 2000.0,
                    step: 10.0,
                    get: |g| g.hdr.max_brightness,
                    set: |g, v| g.hdr.max_brightness = v,
                },
            },
            MenuEntry {
                label: "",
                kind: MenuEntryKind::Separator,
            },
            MenuEntry {
                label: "Record Scenario",
                kind: MenuEntryKind::Toggle(GlobalToggle::RecordScenario),
            },
            MenuEntry {
                label: "",
                kind: MenuEntryKind::Separator,
            },
            MenuEntry {
                label: "Game Exposure",
                kind: MenuEntryKind::Slider {
                    min: 0.5,
                    max: 4.0,
                    step: 0.1,
                    get: |g| {
                        if g.hdr.hdr_enabled {
                            g.hdr.game_exposure_target_hdr
                        } else {
                            g.hdr.game_exposure_target_sdr
                        }
                    },
                    set: |g, v| {
                        if g.hdr.hdr_enabled {
                            g.hdr.game_exposure_target_hdr = v;
                        } else {
                            g.hdr.game_exposure_target_sdr = v;
                        }
                        g.exposure.game_exposure_target = v;
                        g.exposure.game_exposure = v;
                    },
                },
            },
        ];

        Self {
            entries,
            selected: 0,
            scroll_offset: 0,
            visible_rows: 12,
            dragging_entry: None,
            last_mouse_down: false,
        }
    }

    // -------------------------------------------------------------------------
    // Entry visibility helpers
    // -------------------------------------------------------------------------

    /// Returns true if entry at index `i` should be shown.
    fn is_entry_visible(&self, i: usize, globals: &Globals) -> bool {
        let entry = &self.entries[i];
        match entry.label {
            // HDR-specific: only visible when HDR is on
            "HUD Nits" | "Max Brightness" => globals.hdr.hdr_enabled,
            // Generic multipliers: always visible
            "Exposure" | "Game Exposure" => true,
            _ => true,
        }
    }

    /// Returns true if this entry can receive focus/interaction.
    fn is_selectable(entry: &MenuEntry) -> bool {
        !matches!(entry.kind, MenuEntryKind::Separator)
    }

    /// Collect visible entry indices.
    fn visible_indices(&self, globals: &Globals) -> Vec<usize> {
        (0..self.entries.len())
            .filter(|&i| self.is_entry_visible(i, globals))
            .collect()
    }

    /// Collect selectable visible entry indices.
    fn selectable_visible(&self, globals: &Globals) -> Vec<usize> {
        self.visible_indices(globals)
            .into_iter()
            .filter(|&i| Self::is_selectable(&self.entries[i]))
            .collect()
    }

    // -------------------------------------------------------------------------
    // Input handling
    // -------------------------------------------------------------------------

    /// Handle mouse wheel scroll. `delta` is +1 for scroll up, -1 for scroll down.
    pub fn handle_scroll(&mut self, delta: i32, globals: &Globals) {
        let vis = self.visible_indices(globals);
        let max_scroll = vis.len().saturating_sub(self.visible_rows);
        if delta > 0 {
            // scroll up → decrease offset
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        } else if delta < 0 {
            // scroll down → increase offset
            self.scroll_offset = (self.scroll_offset + 1).min(max_scroll);
        }
    }

    /// Handle keyboard navigation. Returns true if input was consumed.
    pub fn handle_key_nav(&mut self, up: bool, globals: &Globals) {
        let selectables = self.selectable_visible(globals);
        if selectables.is_empty() {
            return;
        }
        // Find current position in selectables
        let pos = selectables
            .iter()
            .position(|&i| i == self.selected)
            .unwrap_or(0);
        let new_pos = if up {
            pos.saturating_sub(1)
        } else {
            (pos + 1).min(selectables.len() - 1)
        };
        self.selected = selectables[new_pos];
        self.ensure_selected_visible(globals);
    }

    /// Handle A/Left or D/Right on sliders.
    pub fn handle_slider_step(&mut self, decrease: bool, globals: &mut Globals) {
        let entry = &self.entries[self.selected];
        if let MenuEntryKind::Slider {
            min,
            max,
            step,
            get,
            set,
        } = &entry.kind
        {
            let cur = get(globals);
            let new_val = if decrease {
                (cur - step).max(*min)
            } else {
                (cur + step).min(*max)
            };
            set(globals, snap_to_step(new_val, *min, *step));
        }
    }

    /// Ensure the selected entry is in the visible scroll window.
    fn ensure_selected_visible(&mut self, globals: &Globals) {
        let vis = self.visible_indices(globals);
        let idx_in_vis = vis.iter().position(|&i| i == self.selected);
        if let Some(pos) = idx_in_vis {
            if pos < self.scroll_offset {
                self.scroll_offset = pos;
            } else if pos >= self.scroll_offset + self.visible_rows {
                self.scroll_offset = pos + 1 - self.visible_rows;
            }
        }
    }

    // -------------------------------------------------------------------------
    // Rendering + mouse input
    // -------------------------------------------------------------------------

    /// Main entry point: render the pause menu and process mouse input.
    /// `mouse_sx`, `mouse_sy`: raw SDL2 screen coords (Y-down).
    /// `mouse_down`: left mouse button state.
    pub fn render(
        &mut self,
        globals: &mut Globals,
        renderer: &mut Renderer2D,
        rng: &mut impl Rng,
        mouse_sx: f64,
        mouse_sy: f64,
        mouse_down: bool,
    ) {
        let rr = globals.render.render_scale;
        let sx = globals.render.safe_offset_x;
        let sy = globals.render.safe_offset_y;
        let sw = globals.render.safe_phys_width;
        let sh = globals.render.safe_phys_height;

        // Mouse in physical coords (Y-up)
        let mx = mouse_sx / rr;
        let my = globals.render.phys_height - mouse_sy / rr;

        let click_rising = mouse_down && !self.last_mouse_down;

        // ---- Title "ASTEROIDS" ----
        let title_char_w = (1.0 / 16.0) * sw;
        let title_char_h = (2.5 / 24.0) * sh;
        let title_sp = (1.0 / 40.0) * sw;
        let title_x = sx + (2.0 / 16.0) * sw;
        let title_y = sy + (20.5 / 24.0) * sh;

        // Shadow
        render_string(
            "ASTEROIDS",
            (title_x - 1.0, title_y - 1.0),
            title_char_w,
            title_char_h,
            title_sp,
            0.0,
            [0.0, 0.0, 0.0, 255.0],
            renderer,
            globals,
            rng,
        );
        render_string(
            "ASTEROIDS",
            (title_x, title_y),
            title_char_w,
            title_char_h,
            title_sp,
            0.0,
            [255.0, 255.0, 255.0, 255.0],
            renderer,
            globals,
            rng,
        );

        // ---- Layout ----
        // Menu list occupies the center column of the safe zone.
        let menu_left = sx + (3.0 / 16.0) * sw;
        let menu_right = sx + (13.0 / 16.0) * sw;
        let menu_width = menu_right - menu_left;

        // Top of menu (below title)
        let menu_top = sy + (19.5 / 24.0) * sh;
        let row_height = (1.2 / 24.0) * sh;
        let row_pad = (0.05 / 24.0) * sh;
        let separator_h = (0.3 / 24.0) * sh;

        // Collect visible entries
        let vis_indices = self.visible_indices(globals);
        let total_visible = vis_indices.len();
        let max_scroll = total_visible.saturating_sub(self.visible_rows);
        self.scroll_offset = self.scroll_offset.min(max_scroll);

        let show_from = self.scroll_offset;
        let show_to = (self.scroll_offset + self.visible_rows).min(total_visible);

        // ---- Scroll indicators ----
        let char_h_small = 0.6 * row_height;
        let char_w_small = char_h_small * 0.6;
        let sp_small = char_w_small * 0.15;

        if show_from > 0 {
            // "^" scroll up indicator
            let ind_x = menu_left + menu_width * 0.5 - char_w_small * 0.5;
            let ind_y = menu_top + (0.1 / 24.0) * sh;
            render_string(
                "^",
                (ind_x, ind_y),
                char_w_small,
                char_h_small,
                sp_small,
                0.0,
                [200.0, 200.0, 200.0, 255.0],
                renderer,
                globals,
                rng,
            );
        }

        // ---- Render rows ----
        let mut row_y = menu_top - row_height; // we draw downward (Y-up so subtract)

        let mut hovered_entry: Option<usize> = None;

        for &entry_idx in &vis_indices[show_from..show_to] {
            let entry = &self.entries[entry_idx];

            let is_sep = matches!(entry.kind, MenuEntryKind::Separator);
            let row_h = if is_sep { separator_h } else { row_height };

            let row_bottom = row_y;
            let row_top = row_y + row_h;

            if is_sep {
                // Thin horizontal line
                let line_y = (row_bottom + row_top) * 0.5;
                let px1 = (menu_left * rr) as i32;
                let px2 = (menu_right * rr) as i32;
                let py = (line_y * rr) as i32;
                renderer.hud_draw_line(
                    px1,
                    py,
                    px2,
                    py,
                    [80.0, 80.0, 80.0, 255.0],
                    2.0 * rr as f32,
                );
            } else {
                // Determine hover
                let hovered =
                    mx >= menu_left && mx <= menu_right && my >= row_bottom && my <= row_top;

                if hovered {
                    hovered_entry = Some(entry_idx);
                }

                let is_selected = self.selected == entry_idx;

                // Background rect
                let bg_col: [f32; 4] = if is_selected && !hovered {
                    [40.0, 40.0, 70.0, 220.0]
                } else if hovered {
                    [60.0, 60.0, 90.0, 240.0]
                } else {
                    [20.0, 20.0, 30.0, 180.0]
                };

                let px1 = ((menu_left - 2.0) * rr).round() as i32;
                let px2 = ((menu_right + 2.0) * rr).round() as i32;
                let py1 = (row_bottom * rr).round() as i32;
                let py2 = (row_top * rr).round() as i32;
                let rect_pts = [(px1, py1), (px2, py1), (px2, py2), (px1, py2)];
                renderer.hud_fill_poly(&rect_pts, bg_col);

                // Text sizing
                let char_h = row_h * 0.65;
                let char_w = char_h * 0.6;
                let char_sp = char_w * 0.15;

                // Vertical center
                let text_y = row_bottom + (row_h - char_h) * 0.5 + row_pad;

                // Label on left
                let label_x = menu_left + (0.5 / 16.0) * sw;

                // Shadow + label
                render_string(
                    entry.label,
                    (label_x - 0.5, text_y - 0.5),
                    char_w,
                    char_h,
                    char_sp,
                    0.0,
                    [0.0, 0.0, 0.0, 200.0],
                    renderer,
                    globals,
                    rng,
                );
                render_string(
                    entry.label,
                    (label_x, text_y),
                    char_w,
                    char_h,
                    char_sp,
                    0.0,
                    [255.0, 255.0, 255.0, 255.0],
                    renderer,
                    globals,
                    rng,
                );

                // Right-side content
                let right_x = menu_left + menu_width * 0.6;

                match &entry.kind {
                    MenuEntryKind::Action(_) => {
                        // No right-side content for action entries
                    }
                    MenuEntryKind::Toggle(toggle) => {
                        let on = globals.get_toggle(toggle);
                        let (status_str, status_col): (&str, [f32; 4]) = if on {
                            ("ON", [0.0, 200.0, 0.0, 255.0])
                        } else {
                            ("OFF", [200.0, 0.0, 0.0, 255.0])
                        };

                        // Small colored indicator box
                        let ind_px1 = (right_x * rr) as i32;
                        let ind_px2 = ((right_x + char_h * 1.5) * rr) as i32;
                        let ind_py1 = ((row_bottom + row_h * 0.2) * rr) as i32;
                        let ind_py2 = ((row_bottom + row_h * 0.8) * rr) as i32;
                        let ind_pts = [
                            (ind_px1, ind_py1),
                            (ind_px2, ind_py1),
                            (ind_px2, ind_py2),
                            (ind_px1, ind_py2),
                        ];
                        renderer.hud_fill_poly(&ind_pts, status_col);

                        // ON/OFF text inside box
                        let txt_x = right_x + char_h * 0.1;
                        let txt_y = row_bottom + (row_h - char_h * 0.7) * 0.5;
                        let s_char_h = char_h * 0.7;
                        let s_char_w = s_char_h * 0.6;
                        let s_sp = s_char_w * 0.15;
                        render_string(
                            status_str,
                            (txt_x, txt_y),
                            s_char_w,
                            s_char_h,
                            s_sp,
                            0.0,
                            [255.0, 255.0, 255.0, 255.0],
                            renderer,
                            globals,
                            rng,
                        );

                        // Handle click
                        if (hovered || is_selected) && click_rising {
                            let new_val = !globals.get_toggle(toggle);
                            globals.set_toggle(toggle, new_val);
                        }
                    }
                    MenuEntryKind::Cycle { labels, get, set } => {
                        let cur_idx = get(globals);
                        let cur_label = labels[cur_idx.min(labels.len() - 1)];

                        // Cycle through label with chevrons
                        let display = cur_label;
                        let lbl_char_h = char_h * 0.85;
                        let lbl_char_w = lbl_char_h * 0.6;
                        let lbl_sp = lbl_char_w * 0.15;
                        let lbl_y = row_bottom + (row_h - lbl_char_h) * 0.5 + row_pad;

                        render_string(
                            display,
                            (right_x, lbl_y),
                            lbl_char_w,
                            lbl_char_h,
                            lbl_sp,
                            0.0,
                            [255.0, 220.0, 100.0, 255.0],
                            renderer,
                            globals,
                            rng,
                        );

                        if (hovered || is_selected) && click_rising {
                            let new_idx = (cur_idx + 1) % labels.len();
                            set(globals, new_idx);
                        }
                    }
                    MenuEntryKind::Slider {
                        min,
                        max,
                        step,
                        get,
                        set,
                    } => {
                        let cur_val = get(globals);
                        let ratio = ((cur_val - min) / (max - min)).clamp(0.0, 1.0);

                        let track_left = right_x;
                        let track_right = menu_right - (0.3 / 16.0) * sw;
                        let track_width = track_right - track_left;
                        let track_mid_y = row_bottom + row_h * 0.5;
                        let track_h = row_h * 0.12;

                        let track_px1 = (track_left * rr) as i32;
                        let track_px2 = (track_right * rr) as i32;
                        let track_py1 = ((track_mid_y - track_h) * rr) as i32;
                        let track_py2 = ((track_mid_y + track_h) * rr) as i32;

                        // Track background
                        let track_bg = [
                            (track_px1, track_py1),
                            (track_px2, track_py1),
                            (track_px2, track_py2),
                            (track_px1, track_py2),
                        ];
                        renderer.hud_fill_poly(&track_bg, [60.0, 60.0, 60.0, 255.0]);

                        // Filled portion
                        let fill_px2 = ((track_left + ratio * track_width) * rr) as i32;
                        if fill_px2 > track_px1 {
                            let fill_pts = [
                                (track_px1, track_py1),
                                (fill_px2, track_py1),
                                (fill_px2, track_py2),
                                (track_px1, track_py2),
                            ];
                            renderer.hud_fill_poly(&fill_pts, [100.0, 180.0, 255.0, 255.0]);
                        }

                        // Thumb
                        let thumb_cx = track_left + ratio * track_width;
                        let thumb_r = row_h * 0.3;
                        let thumb_px = (thumb_cx * rr) as i32;
                        let thumb_py1 = ((track_mid_y - thumb_r) * rr) as i32;
                        let thumb_py2 = ((track_mid_y + thumb_r) * rr) as i32;
                        let thumb_pts = [
                            (thumb_px - (thumb_r * rr) as i32, thumb_py1),
                            (thumb_px + (thumb_r * rr) as i32, thumb_py1),
                            (thumb_px + (thumb_r * rr) as i32, thumb_py2),
                            (thumb_px - (thumb_r * rr) as i32, thumb_py2),
                        ];
                        renderer.hud_fill_poly(&thumb_pts, [220.0, 220.0, 255.0, 255.0]);

                        // Numeric value on right
                        let val_str = format_value(cur_val, *step);
                        let val_char_h = char_h * 0.7;
                        let val_char_w = val_char_h * 0.6;
                        let val_sp = val_char_w * 0.15;
                        let val_x = track_right + (0.1 / 16.0) * sw;
                        let val_y = row_bottom + (row_h - val_char_h) * 0.5 + row_pad;
                        render_string(
                            &val_str,
                            (val_x, val_y),
                            val_char_w,
                            val_char_h,
                            val_sp,
                            0.0,
                            [200.0, 200.0, 200.0, 255.0],
                            renderer,
                            globals,
                            rng,
                        );

                        // Drag: if mouse is inside this row and button is held, map mx to value
                        if mouse_down && hovered && mx >= track_left && mx <= track_right {
                            self.dragging_entry = Some(entry_idx);
                        }
                        if self.dragging_entry == Some(entry_idx)
                            && mouse_down
                            && mx >= track_left
                            && mx <= track_right
                        {
                            let new_ratio = ((mx - track_left) / track_width).clamp(0.0, 1.0);
                            let raw_val = min + new_ratio * (max - min);
                            set(globals, snap_to_step(raw_val, *min, *step));
                        }
                        if !mouse_down {
                            self.dragging_entry = None;
                        }
                    }
                    MenuEntryKind::Separator => unreachable!(),
                }

                // Handle Action click
                if let MenuEntryKind::Action(toggle) = &entry.kind {
                    if (hovered || is_selected) && click_rising {
                        globals.set_toggle(toggle, true);
                    }
                }
            }

            // Move to next row (Y-up, so subtract)
            row_y -= row_h;
        }

        // Scroll-down indicator
        if show_to < total_visible {
            let ind_y = row_y + row_height * 0.1;
            let ind_x = menu_left + menu_width * 0.5 - char_w_small * 0.5;
            render_string(
                "V",
                (ind_x, ind_y),
                char_w_small,
                char_h_small,
                sp_small,
                0.0,
                [200.0, 200.0, 200.0, 255.0],
                renderer,
                globals,
                rng,
            );
        }

        // Update selection from hover (mouse hover sets selection)
        if let Some(hov_idx) = hovered_entry {
            self.selected = hov_idx;
        }

        self.last_mouse_down = mouse_down;
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Snap `val` to nearest step starting from `min`.
fn snap_to_step(val: f64, min: f64, step: f64) -> f64 {
    let steps = ((val - min) / step).round();
    min + steps * step
}

/// Format a slider value for display. Use integer display when step >= 1.
fn format_value(val: f64, step: f64) -> String {
    if step >= 1.0 {
        format!("{}", val.round() as i64)
    } else {
        format!("{:.2}", val)
    }
}
