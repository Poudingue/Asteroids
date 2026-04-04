# Rendering Fix, HDR Pipeline & Pause Menu Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the B&W rendering bug, add HDR output with configurable tonemapping, and redesign the pause menu as a scrollable option list.

**Architecture:** Fix the alpha=255 blend bug first for immediate visual restoration. Then replace the hard spectral redirect with a soft 80%-shoulder tonemap. Redesign pause_menu.rs as a vertical scrollable list supporting toggles, cycles, and sliders. Add HDR surface format switching (sRGB ↔ Rgba16Float) with three configurable nit values. Pipeline recreation on format change.

**Tech Stack:** Rust, wgpu 24, SDL2, WGSL shaders

---

## Task 1: Fix Alpha Bug in to_hdr_rgba

**Files:**
- Modify: `src/game.rs:225` (to_hdr_rgba alpha)
- Modify: `src/game.rs:1479` (background color alpha)
- Test: `scenarios/test_visual.ron` (visual verification)

- [ ] **Step 1: Fix alpha in to_hdr_rgba**

In `src/game.rs`, change line 225:

```rust
// Before:
pub(crate) fn to_hdr_rgba(color: HdrColor) -> [f32; 4] {
    [color.r as f32, color.g as f32, color.b as f32, 255.0]
}

// After:
pub(crate) fn to_hdr_rgba(color: HdrColor) -> [f32; 4] {
    [color.r as f32, color.g as f32, color.b as f32, 1.0]
}
```

- [ ] **Step 2: Fix alpha in background color**

In `src/game.rs`, change line 1479:

```rust
// Before:
let bg_color = [bg.r as f32, bg.g as f32, bg.b as f32, 255.0];

// After:
let bg_color = [bg.r as f32, bg.g as f32, bg.b as f32, 1.0];
```

- [ ] **Step 3: Fix postprocess alpha output**

In `src/shaders/postprocess.wgsl`, change line 141:

```wgsl
// Before:
return vec4<f32>(mapped, hdr_color.a);

// After:
return vec4<f32>(mapped, 1.0);
```

- [ ] **Step 4: Build and verify**

Run: `rtk cargo build`
Expected: Compiles with no errors.

- [ ] **Step 5: Visual test**

Run: `rtk cargo run -- --scenario scenarios/test_visual.ron`
Expected: Window opens for 2 seconds showing colored asteroids and ship (no longer B&W/XOR). Verify colors are visible and shapes blend correctly.

- [ ] **Step 6: Run existing tests**

Run: `rtk cargo test`
Expected: All existing tests pass (determinism tests should be unaffected since they don't check pixel output).

- [ ] **Step 7: Commit**

```bash
rtk git add src/game.rs src/shaders/postprocess.wgsl
rtk git commit -m "fix: alpha 255→1 in to_hdr_rgba and postprocess output

Fixes B&W/XOR rendering artifacts caused by alpha=255.0 in Rgba16Float
offscreen texture with ALPHA_BLENDING. GPU blend equation was computing
src.rgb*255 - dst.rgb*254 instead of standard alpha compositing.
Also fixes SDF anti-aliasing which was non-functional due to same bug."
```

---

## Task 2: Switch to Soft Redirect Tonemap with 80% Shoulder

**Files:**
- Modify: `src/shaders/postprocess.wgsl` (replace tonemap logic)

- [ ] **Step 1: Replace tonemap with soft redirect and 80% shoulder**

Rewrite `src/shaders/postprocess.wgsl`. Replace the `TONEMAP_VARIANT` switch and all tonemap functions with a single soft-redirect tonemap. The key principle: passthrough below 80% of threshold (255 for now, will become max_brightness later), smoothstep bleed from 80% to 100%.

Replace the full shader content with:

```wgsl
struct PostProcessUniforms {
    game_exposure: f32,
    add_color_r: f32,
    add_color_g: f32,
    add_color_b: f32,
    mul_color_r: f32,
    mul_color_g: f32,
    mul_color_b: f32,
    _padding: f32,
}

@group(0) @binding(0) var offscreen_texture: texture_2d<f32>;
@group(0) @binding(1) var offscreen_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: PostProcessUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) / 2.0, (1.0 - y) / 2.0);
    return out;
}

// Soft spectral redirect with 80% shoulder.
// Below soft_start: passthrough (preserves full saturation and WCG).
// Between soft_start and threshold: smoothstep bleed into spectral neighbors.
// Above threshold: full redistribution.
fn soft_redirect(col: vec3<f32>, threshold: f32) -> vec3<f32> {
    let soft_start = threshold * 0.8;

    // Per-channel excess and blend factor
    let excess_r = max(col.r - soft_start, 0.0);
    let excess_g = max(col.g - soft_start, 0.0);
    let excess_b = max(col.b - soft_start, 0.0);

    let blend_r = smoothstep(soft_start, threshold, col.r);
    let blend_g = smoothstep(soft_start, threshold, col.g);
    let blend_b = smoothstep(soft_start, threshold, col.b);

    // Bleed into spectral neighbors (r↔g↔b circular)
    // Primary neighbor gets 60% of excess, secondary gets 30%
    var r_out = col.r;
    var g_out = col.g;
    var b_out = col.b;

    // Red excess → green (primary), blue (secondary)
    let r_bleed = excess_r * blend_r;
    r_out = r_out - r_bleed * 0.9;
    g_out = g_out + r_bleed * 0.6;
    b_out = b_out + r_bleed * 0.3;

    // Green excess → red (primary), blue (primary — green is between)
    let g_bleed = excess_g * blend_g;
    g_out = g_out - g_bleed * 0.9;
    r_out = r_out + g_bleed * 0.45;
    b_out = b_out + g_bleed * 0.45;

    // Blue excess → green (primary), red (secondary)
    let b_bleed = excess_b * blend_b;
    b_out = b_out - b_bleed * 0.9;
    g_out = g_out + b_bleed * 0.6;
    r_out = r_out + b_bleed * 0.3;

    return clamp(vec3<f32>(r_out, g_out, b_out), vec3<f32>(0.0), vec3<f32>(threshold));
}

fn tonemap(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);

    let with_add = hdr_color + add_color * uniforms.game_exposure;
    let with_mul = with_add * mul_color;

    let threshold = 255.0;  // Will become max_brightness uniform in Task 6
    let redirected = soft_redirect(with_mul, threshold);

    return redirected / threshold;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(offscreen_texture, offscreen_sampler, in.uv);
    let mapped = tonemap(hdr_color.rgb);
    return vec4<f32>(mapped, 1.0);
}
```

- [ ] **Step 2: Build and verify**

Run: `rtk cargo build`
Expected: Compiles with no errors.

- [ ] **Step 3: Visual test**

Run: `rtk cargo run -- --scenario scenarios/test_visual.ron`
Expected: Colors should look more vivid than before — bright objects gradually bleed instead of hard clipping. No white-crush on bright objects.

- [ ] **Step 4: Run full game to verify flash/explosion effects**

Run: `rtk cargo run`
Expected: Play for a few seconds, fire weapons, cause explosions. Flashes and bright effects should blend smoothly without hard white-out. Kill the game manually.

- [ ] **Step 5: Run tests**

Run: `rtk cargo test`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
rtk git add src/shaders/postprocess.wgsl
rtk git commit -m "feat: soft redirect tonemap with 80% shoulder

Replaces hard spectral redistribution (redirect_spectre_wide) with
smoothstep-gated soft bleed. Below 80% of threshold: full passthrough
preserving saturation. 80-100%: gradual spectral neighbor bleed.
Eliminates white-crush on bright HDR colors."
```

---

## Task 3: Remove Dead zoom_factor Uniform

**Files:**
- Modify: `src/shaders/world.wgsl:12` (remove binding)
- Modify: `src/rendering/mod.rs:188-231` (remove buffer and bind group entry)

- [ ] **Step 1: Remove zoom_factor from world.wgsl**

In `src/shaders/world.wgsl`, remove line 12:

```wgsl
// Before (line 12):
@group(0) @binding(1) var<uniform> zoom_factor: f32;

// After: line removed entirely
```

- [ ] **Step 2: Remove zoom_factor_buffer from Renderer2D**

In `src/rendering/mod.rs`, find the `zoom_factor_buffer` field in `Renderer2D` struct (around line 94-130) and remove it. It has `#[allow(dead_code)]`.

Remove the buffer creation (around line 188):
```rust
// Remove this block:
let zoom_factor_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Zoom Factor Buffer"),
    contents: bytemuck::cast_slice(&[1.0_f32]),
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
});
```

- [ ] **Step 3: Update world bind group layout**

In `src/rendering/mod.rs`, find the world bind group layout (around line 194-220). Remove the second entry (binding 1 for zoom_factor). The layout should have only one entry for screen_size:

```rust
let world_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("World Bind Group Layout"),
    entries: &[
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Remove the binding 1 entry entirely
    ],
});
```

Also update the bind group creation to remove the zoom_factor entry:

```rust
let world_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("World Bind Group"),
    layout: &world_bind_group_layout,
    entries: &[
        wgpu::BindGroupEntry {
            binding: 0,
            resource: screen_size_buffer.as_entire_binding(),
        },
        // Remove the binding 1 entry entirely
    ],
});
```

- [ ] **Step 4: Remove zoom_factor_buffer from struct field**

Find the struct field `zoom_factor_buffer` in `Renderer2D` (has `#[allow(dead_code)]`) and remove it. Also remove it from the struct initialization in `new()`.

- [ ] **Step 5: Build and verify**

Run: `rtk cargo build`
Expected: Compiles. No warnings about dead_code for zoom_factor.

- [ ] **Step 6: Run tests**

Run: `rtk cargo test`
Expected: All pass.

- [ ] **Step 7: Commit**

```bash
rtk git add src/shaders/world.wgsl src/rendering/mod.rs
rtk git commit -m "cleanup: remove dead zoom_factor uniform and buffer"
```

---

## Task 4: Add HDR Config to Parameters

**Files:**
- Modify: `src/parameters.rs` (add HdrConfig, new GlobalToggle variants, make MSAA_SAMPLE_COUNT non-const)

- [ ] **Step 1: Add HdrConfig struct**

In `src/parameters.rs`, add after `ExposureConfig` (around line 490):

```rust
#[derive(Clone, Debug)]
pub struct HdrConfig {
    pub hdr_enabled: bool,
    pub hud_nits: f64,
    pub paper_white: f64,
    pub max_brightness: f64,
    pub smaa_enabled: bool,
    pub msaa_sample_count: u32,
}

impl Default for HdrConfig {
    fn default() -> Self {
        Self {
            hdr_enabled: false,
            hud_nits: 155.0,
            paper_white: 200.0,
            max_brightness: 1000.0,
            smaa_enabled: false,
            msaa_sample_count: 4,
        }
    }
}
```

- [ ] **Step 2: Add HdrConfig to Globals**

In `src/parameters.rs`, add a field to `Globals` struct (around line 562):

```rust
pub struct Globals {
    // ... existing fields ...
    pub hdr: HdrConfig,
}
```

Initialize it in `Globals::new()` or wherever Globals is constructed:

```rust
hdr: HdrConfig::default(),
```

- [ ] **Step 3: Add new GlobalToggle variants**

In `src/parameters.rs`, extend the `GlobalToggle` enum (around line 742):

```rust
pub enum GlobalToggle {
    Quit,
    Pause,
    Restart,
    AdvancedHitbox,
    Smoke,
    Screenshake,
    Flashes,
    Chunks,
    DynColor,
    // New:
    Hdr,
    Smaa,
}
```

Update the `set_toggle` and `get_toggle` methods to handle the new variants:

```rust
GlobalToggle::Hdr => &mut self.hdr.hdr_enabled,
GlobalToggle::Smaa => &mut self.hdr.smaa_enabled,
```

- [ ] **Step 4: Make MSAA_SAMPLE_COUNT use HdrConfig**

Change `MSAA_SAMPLE_COUNT` from a const to a reference through `globals.hdr.msaa_sample_count`. At line 102:

```rust
// Before:
pub const MSAA_SAMPLE_COUNT: u32 = 4;

// After: Remove the const. MSAA is now in HdrConfig.
// All references to MSAA_SAMPLE_COUNT must be updated to use globals.hdr.msaa_sample_count
// or the renderer's stored value.
```

Note: The renderer stores its own `msaa_sample_count` field already. The const is used at pipeline creation time. For now, keep the const as a default and add a note that Task 8 will make it runtime-configurable.

Actually, keep the const for now as `DEFAULT_MSAA_SAMPLE_COUNT`:

```rust
pub const DEFAULT_MSAA_SAMPLE_COUNT: u32 = 4;
```

- [ ] **Step 5: Build and fix compilation**

Run: `rtk cargo build`
Fix any compilation errors from the renamed const or new struct. Expected references to `MSAA_SAMPLE_COUNT` in `rendering/mod.rs` — update them to `DEFAULT_MSAA_SAMPLE_COUNT`.

- [ ] **Step 6: Run tests**

Run: `rtk cargo test`
Expected: All pass.

- [ ] **Step 7: Commit**

```bash
rtk git add src/parameters.rs src/rendering/mod.rs
rtk git commit -m "feat: add HdrConfig struct with nit values and MSAA/SMAA toggles

Adds hdr_enabled, hud_nits (155), paper_white (200), max_brightness (1000),
smaa_enabled, and msaa_sample_count to Globals. Adds Hdr and Smaa to
GlobalToggle enum. Renames MSAA_SAMPLE_COUNT to DEFAULT_MSAA_SAMPLE_COUNT."
```

---

## Task 5: Redesign Pause Menu as Vertical Scrollable List

**Files:**
- Rewrite: `src/pause_menu.rs` (complete rewrite)

- [ ] **Step 1: Define menu entry types**

Rewrite `src/pause_menu.rs` with the new data model. Replace all existing content:

```rust
use crate::parameters::{GlobalToggle, Globals};
use crate::rendering::Renderer2D;

/// Type of menu entry
#[derive(Clone)]
pub enum MenuEntryKind {
    /// Action button (Resume, New Game, Quit)
    Action(GlobalToggle),
    /// Boolean toggle (on/off)
    Toggle(GlobalToggle),
    /// Cycle through discrete values
    Cycle {
        values: Vec<String>,
        current: usize,
        on_change: fn(&mut Globals, usize),
    },
    /// Numeric slider
    Slider {
        min: f64,
        max: f64,
        step: f64,
        get: fn(&Globals) -> f64,
        set: fn(&mut Globals, f64),
    },
    /// Visual separator
    Separator,
}

pub struct MenuEntry {
    pub label: String,
    pub kind: MenuEntryKind,
    pub visible: fn(&Globals) -> bool,
}

pub struct PauseMenu {
    entries: Vec<MenuEntry>,
    scroll_offset: usize,
    selected: usize,
    dragging_slider: bool,
    last_mouse_down: bool,
}
```

- [ ] **Step 2: Build the entries list**

Add the constructor that creates all menu entries:

```rust
impl PauseMenu {
    pub fn new() -> Self {
        let always_visible: fn(&Globals) -> bool = |_| true;
        let hdr_visible: fn(&Globals) -> bool = |g| g.hdr.hdr_enabled;

        let entries = vec![
            // Action buttons
            MenuEntry {
                label: "Resume".into(),
                kind: MenuEntryKind::Action(GlobalToggle::Pause),
                visible: always_visible,
            },
            MenuEntry {
                label: "New Game".into(),
                kind: MenuEntryKind::Action(GlobalToggle::Restart),
                visible: always_visible,
            },
            MenuEntry {
                label: "Quit".into(),
                kind: MenuEntryKind::Action(GlobalToggle::Quit),
                visible: always_visible,
            },
            MenuEntry { label: String::new(), kind: MenuEntryKind::Separator, visible: always_visible },

            // Gameplay toggles
            MenuEntry {
                label: "Advanced Hitbox".into(),
                kind: MenuEntryKind::Toggle(GlobalToggle::AdvancedHitbox),
                visible: always_visible,
            },
            MenuEntry {
                label: "Smoke Particles".into(),
                kind: MenuEntryKind::Toggle(GlobalToggle::Smoke),
                visible: always_visible,
            },
            MenuEntry {
                label: "Screenshake".into(),
                kind: MenuEntryKind::Toggle(GlobalToggle::Screenshake),
                visible: always_visible,
            },
            MenuEntry {
                label: "Light Flashes".into(),
                kind: MenuEntryKind::Toggle(GlobalToggle::Flashes),
                visible: always_visible,
            },
            MenuEntry {
                label: "Chunk Particles".into(),
                kind: MenuEntryKind::Toggle(GlobalToggle::Chunks),
                visible: always_visible,
            },
            MenuEntry {
                label: "Color Effects".into(),
                kind: MenuEntryKind::Toggle(GlobalToggle::DynColor),
                visible: always_visible,
            },
            MenuEntry { label: String::new(), kind: MenuEntryKind::Separator, visible: always_visible },

            // Rendering options
            MenuEntry {
                label: "MSAA".into(),
                kind: MenuEntryKind::Cycle {
                    values: vec!["Off".into(), "x2".into(), "x4".into()],
                    current: 2, // default x4
                    on_change: |g, idx| {
                        g.hdr.msaa_sample_count = match idx {
                            0 => 1,
                            1 => 2,
                            _ => 4,
                        };
                    },
                },
                visible: always_visible,
            },
            MenuEntry {
                label: "SMAA".into(),
                kind: MenuEntryKind::Toggle(GlobalToggle::Smaa),
                visible: always_visible,
            },
            MenuEntry {
                label: "HDR".into(),
                kind: MenuEntryKind::Toggle(GlobalToggle::Hdr),
                visible: always_visible,
            },
            MenuEntry { label: String::new(), kind: MenuEntryKind::Separator, visible: hdr_visible },

            // HDR config (visible only when HDR enabled)
            MenuEntry {
                label: "HUD Nits".into(),
                kind: MenuEntryKind::Slider {
                    min: 50.0, max: 500.0, step: 5.0,
                    get: |g| g.hdr.hud_nits,
                    set: |g, v| g.hdr.hud_nits = v,
                },
                visible: hdr_visible,
            },
            MenuEntry {
                label: "Paper White".into(),
                kind: MenuEntryKind::Slider {
                    min: 80.0, max: 500.0, step: 5.0,
                    get: |g| g.hdr.paper_white,
                    set: |g, v| g.hdr.paper_white = v,
                },
                visible: hdr_visible,
            },
            MenuEntry {
                label: "Max Brightness".into(),
                kind: MenuEntryKind::Slider {
                    min: 400.0, max: 2000.0, step: 10.0,
                    get: |g| g.hdr.max_brightness,
                    set: |g, v| g.hdr.max_brightness = v,
                },
                visible: hdr_visible,
            },
        ];

        Self {
            entries,
            scroll_offset: 0,
            selected: 0,
            dragging_slider: false,
            last_mouse_down: false,
        }
    }
}
```

- [ ] **Step 3: Implement visible entries helper and scroll logic**

```rust
impl PauseMenu {
    /// Returns indices of currently visible entries
    fn visible_indices(&self, globals: &Globals) -> Vec<usize> {
        self.entries.iter().enumerate()
            .filter(|(_, e)| (e.visible)(globals))
            .map(|(i, _)| i)
            .collect()
    }

    /// Maximum number of entries visible on screen
    fn max_visible_rows(&self, safe_height: f64, entry_height: f64) -> usize {
        ((safe_height * 0.75) / entry_height).floor() as usize  // 75% of safe zone for list
    }

    /// Scroll to keep selection visible
    fn ensure_selected_visible(&mut self, visible_count: usize, max_rows: usize) {
        if visible_count == 0 { return; }
        // Find position of selected in visible list
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + max_rows {
            self.scroll_offset = self.selected - max_rows + 1;
        }
    }

    /// Handle scroll input. Returns true if selection changed.
    pub fn handle_input(
        &mut self,
        globals: &mut Globals,
        up: bool,
        down: bool,
        left: bool,
        right: bool,
        click: bool,
        mouse_down: bool,
        mouse_x: f64,
        mouse_y: f64,
        scroll_delta: i32,
        safe_x: f64,
        safe_y: f64,
        safe_w: f64,
        safe_h: f64,
    ) {
        let visible = self.visible_indices(globals);
        if visible.is_empty() { return; }

        let entry_height = safe_h * 0.04;  // each entry is 4% of safe height
        let max_rows = self.max_visible_rows(safe_h, entry_height);
        let list_top = safe_y + safe_h * 0.2;  // 20% from top for title

        // Keyboard / scroll navigation
        if up || scroll_delta > 0 {
            // Move selection up, skip separators
            let mut pos = visible.iter().position(|&i| i == visible[self.selected.min(visible.len()-1)]).unwrap_or(0);
            loop {
                if pos == 0 { break; }  // hard stop at top
                pos -= 1;
                if !matches!(self.entries[visible[pos]].kind, MenuEntryKind::Separator) {
                    self.selected = pos;
                    break;
                }
            }
        }
        if down || scroll_delta < 0 {
            let mut pos = visible.iter().position(|&i| i == visible[self.selected.min(visible.len()-1)]).unwrap_or(0);
            loop {
                if pos >= visible.len() - 1 { break; }  // hard stop at bottom
                pos += 1;
                if !matches!(self.entries[visible[pos]].kind, MenuEntryKind::Separator) {
                    self.selected = pos;
                    break;
                }
            }
        }

        self.ensure_selected_visible(visible.len(), max_rows);

        // Mouse hover detection
        let rel_y = mouse_y - list_top;
        if mouse_x >= safe_x && mouse_x <= safe_x + safe_w && rel_y >= 0.0 {
            let hovered_row = (rel_y / entry_height) as usize + self.scroll_offset;
            if hovered_row < visible.len() {
                if !matches!(self.entries[visible[hovered_row]].kind, MenuEntryKind::Separator) {
                    self.selected = hovered_row;
                }
            }
        }

        // Click/interaction on selected entry
        let sel_idx = visible[self.selected.min(visible.len()-1)];
        let rising_edge = mouse_down && !self.last_mouse_down;

        match &mut self.entries[sel_idx].kind {
            MenuEntryKind::Action(toggle) => {
                if rising_edge || click {
                    globals.set_toggle(*toggle);
                }
            }
            MenuEntryKind::Toggle(toggle) => {
                if rising_edge || click {
                    globals.set_toggle(*toggle);
                }
            }
            MenuEntryKind::Cycle { values, current, on_change } => {
                if rising_edge || click {
                    *current = (*current + 1) % values.len();
                    on_change(globals, *current);
                }
            }
            MenuEntryKind::Slider { min, max, step, get, set } => {
                if mouse_down {
                    // Slider drag: map mouse_x within entry bounds to value
                    let slider_left = safe_x + safe_w * 0.5;  // right half is slider track
                    let slider_right = safe_x + safe_w * 0.95;
                    let t = ((mouse_x - slider_left) / (slider_right - slider_left)).clamp(0.0, 1.0);
                    let value = *min + t * (*max - *min);
                    // Snap to step
                    let snapped = (*min + ((value - *min) / *step).round() * *step).clamp(*min, *max);
                    set(globals, snapped);
                }
                let current_val = get(globals);
                if left {
                    set(globals, (current_val - *step).max(*min));
                }
                if right {
                    set(globals, (current_val + *step).min(*max));
                }
            }
            MenuEntryKind::Separator => {}
        }

        self.last_mouse_down = mouse_down;
    }
}
```

- [ ] **Step 4: Implement rendering**

Add the render method to PauseMenu:

```rust
impl PauseMenu {
    pub fn render(
        &self,
        globals: &Globals,
        renderer: &mut Renderer2D,
        phys_width: f64,
        phys_height: f64,
        render_scale: f64,
    ) {
        let safe_w = phys_width * 0.6;
        let safe_h = phys_height * 0.9;
        let safe_x = (phys_width - safe_w) / 2.0;
        let safe_y = (phys_height - safe_h) / 2.0;

        let entry_height = safe_h * 0.04;
        let entry_width = safe_w * 0.9;
        let entry_x = safe_x + safe_w * 0.05;
        let max_rows = self.max_visible_rows(safe_h, entry_height);
        let list_top = safe_y + safe_h * 0.2;

        let visible = self.visible_indices(globals);
        let char_h = entry_height * 0.6;
        let char_w = char_h * 0.6;

        // Title
        let title_h = safe_h * 0.12;
        let title_w = title_h * 0.6;
        let title_x = safe_x + (safe_w - title_w * 9.0) / 2.0;  // "ASTEROIDS" = 9 chars
        let title_y = safe_y + safe_h * 0.05;

        // Render title "ASTEROIDS" with shadow
        renderer.draw_text("ASTEROIDS", title_x + 2.0, title_y - 2.0, title_w, title_h, [0.0, 0.0, 0.0, 255.0]);
        renderer.draw_text("ASTEROIDS", title_x, title_y, title_w, title_h, [255.0, 255.0, 255.0, 255.0]);

        // Render visible entries
        for (row, &idx) in visible.iter().enumerate().skip(self.scroll_offset).take(max_rows) {
            let entry = &self.entries[idx];
            let y = list_top + (row - self.scroll_offset) as f64 * entry_height * 1.2;
            let is_selected = row == self.selected;

            match &entry.kind {
                MenuEntryKind::Separator => {
                    // Draw thin horizontal line
                    let sep_y = y + entry_height * 0.5;
                    renderer.fill_rect(
                        entry_x, sep_y,
                        entry_x + entry_width, sep_y + 2.0,
                        [64.0, 64.0, 64.0, 255.0],
                    );
                }
                MenuEntryKind::Action(_) => {
                    let bg = if is_selected { [80.0, 80.0, 120.0, 255.0] } else { [40.0, 40.0, 60.0, 255.0] };
                    renderer.fill_rect(entry_x, y, entry_x + entry_width, y + entry_height, bg);
                    let text_x = entry_x + (entry_width - char_w * entry.label.len() as f64) / 2.0;
                    let text_y = y + (entry_height - char_h) / 2.0;
                    renderer.draw_text(&entry.label, text_x, text_y, char_w, char_h, [255.0, 255.0, 255.0, 255.0]);
                }
                MenuEntryKind::Toggle(toggle) => {
                    let on = globals.get_toggle(*toggle);
                    let bg = if on {
                        if is_selected { [0.0, 160.0, 0.0, 255.0] } else { [0.0, 100.0, 0.0, 255.0] }
                    } else {
                        if is_selected { [160.0, 0.0, 0.0, 255.0] } else { [100.0, 0.0, 0.0, 255.0] }
                    };
                    renderer.fill_rect(entry_x, y, entry_x + entry_width, y + entry_height, bg);
                    let text_y = y + (entry_height - char_h) / 2.0;
                    renderer.draw_text(&entry.label, entry_x + char_w, text_y, char_w, char_h, [255.0, 255.0, 255.0, 255.0]);
                    let state_text = if on { "ON" } else { "OFF" };
                    let state_x = entry_x + entry_width - char_w * 4.0;
                    renderer.draw_text(state_text, state_x, text_y, char_w, char_h, [255.0, 255.0, 255.0, 255.0]);
                }
                MenuEntryKind::Cycle { values, current, .. } => {
                    let bg = if is_selected { [80.0, 80.0, 80.0, 255.0] } else { [50.0, 50.0, 50.0, 255.0] };
                    renderer.fill_rect(entry_x, y, entry_x + entry_width, y + entry_height, bg);
                    let text_y = y + (entry_height - char_h) / 2.0;
                    renderer.draw_text(&entry.label, entry_x + char_w, text_y, char_w, char_h, [255.0, 255.0, 255.0, 255.0]);
                    let val_text = &values[*current];
                    let val_x = entry_x + entry_width - char_w * (val_text.len() as f64 + 1.0);
                    renderer.draw_text(val_text, val_x, text_y, char_w, char_h, [200.0, 200.0, 255.0, 255.0]);
                }
                MenuEntryKind::Slider { min, max, get, .. } => {
                    let bg = if is_selected { [80.0, 80.0, 80.0, 255.0] } else { [50.0, 50.0, 50.0, 255.0] };
                    renderer.fill_rect(entry_x, y, entry_x + entry_width, y + entry_height, bg);
                    let text_y = y + (entry_height - char_h) / 2.0;
                    // Label on left
                    renderer.draw_text(&entry.label, entry_x + char_w, text_y, char_w, char_h, [255.0, 255.0, 255.0, 255.0]);
                    // Slider track on right half
                    let track_left = entry_x + entry_width * 0.5;
                    let track_right = entry_x + entry_width * 0.9;
                    let track_y = y + entry_height * 0.4;
                    let track_h = entry_height * 0.2;
                    renderer.fill_rect(track_left, track_y, track_right, track_y + track_h, [100.0, 100.0, 100.0, 255.0]);
                    // Thumb position
                    let val = get(globals);
                    let t = (val - min) / (max - min);
                    let thumb_x = track_left + t * (track_right - track_left);
                    let thumb_w = entry_height * 0.3;
                    renderer.fill_rect(thumb_x - thumb_w / 2.0, y + entry_height * 0.15, thumb_x + thumb_w / 2.0, y + entry_height * 0.85, [255.0, 255.0, 255.0, 255.0]);
                    // Value text
                    let val_str = format!("{:.0}", val);
                    let val_x = entry_x + entry_width * 0.92;
                    renderer.draw_text(&val_str, val_x, text_y, char_w, char_h, [200.0, 200.0, 200.0, 255.0]);
                }
            }
        }

        // Scroll indicators
        if self.scroll_offset > 0 {
            renderer.draw_text("^", entry_x + entry_width / 2.0, list_top - entry_height, char_w, char_h, [200.0, 200.0, 200.0, 255.0]);
        }
        if self.scroll_offset + max_rows < visible.len() {
            let bottom_y = list_top + max_rows as f64 * entry_height * 1.2;
            renderer.draw_text("v", entry_x + entry_width / 2.0, bottom_y, char_w, char_h, [200.0, 200.0, 200.0, 255.0]);
        }
    }
}
```

- [ ] **Step 5: Update game.rs to use new PauseMenu**

The existing `render_pause_title` function in `src/game.rs` (around line 1463 in `render_frame`) calls into `pause_menu::render_pause_title`. Replace this call with the new `PauseMenu::render()` and `PauseMenu::handle_input()`.

Add `PauseMenu` to `GameState` or pass it alongside. The menu should be constructed once (at game init) and stored.

In `src/main.rs`, add alongside GameState:

```rust
let mut pause_menu = PauseMenu::new();
```

The render_frame call where pause menu was invoked should become:

```rust
if globals.is_paused() {
    pause_menu.render(globals, renderer, phys_width, phys_height, render_scale);
}
```

The input handling (in the main loop in main.rs) should call:

```rust
if globals.is_paused() {
    let up = keys_pressed.contains(&Keycode::W) || keys_pressed.contains(&Keycode::Up);
    let down = keys_pressed.contains(&Keycode::S) || keys_pressed.contains(&Keycode::Down);
    let left = keys_pressed.contains(&Keycode::A) || keys_pressed.contains(&Keycode::Left);
    let right = keys_pressed.contains(&Keycode::D) || keys_pressed.contains(&Keycode::Right);
    pause_menu.handle_input(
        globals, up, down, left, right,
        false, mouse_down, mouse_x, mouse_y,
        scroll_delta,
        safe_x, safe_y, safe_w, safe_h,
    );
}
```

Note: `scroll_delta` must be captured from SDL2 events. In the SDL2 event loop (in main.rs), add handling for `Event::MouseWheel { y, .. }` to capture scroll delta:

```rust
Event::MouseWheel { y, .. } => {
    scroll_delta = y;  // positive = scroll up, negative = scroll down
}
```

- [ ] **Step 6: Verify the renderer has the needed drawing primitives**

Check that `Renderer2D` has `fill_rect` and `draw_text` methods. The existing pause_menu code uses `renderer.fill_rect(...)` and text rendering through glyph functions. The new code should use the same primitives. If method signatures differ, adapt the render code to match.

- [ ] **Step 7: Build and fix compilation**

Run: `rtk cargo build`
Expected: May need adjustments for method signatures, borrow checker issues with closures, and SDL2 event handling. Fix iteratively.

- [ ] **Step 8: Visual test**

Run: `rtk cargo run`
Expected: Press Escape to pause. See vertical list of options. Navigate with W/S, mouse wheel. Click toggles. HDR options hidden until HDR is toggled on.

- [ ] **Step 9: Run tests**

Run: `rtk cargo test`
Expected: All pass.

- [ ] **Step 10: Commit**

```bash
rtk git add src/pause_menu.rs src/game.rs src/main.rs
rtk git commit -m "feat: redesign pause menu as vertical scrollable list

Replaces grid layout with vertical stack. Supports three entry types:
toggle (on/off), cycle (discrete values), slider (continuous range).
Navigation via mouse wheel, W/S, arrow keys. Hard stop at edges.
HDR config sliders visible only when HDR is enabled.
MSAA cycle: Off/x2/x4. SMAA and HDR toggles added."
```

---

## Task 6: Add HDR Uniforms to Postprocess Shader

**Files:**
- Modify: `src/rendering/mod.rs` (PostProcessUniforms, update function)
- Modify: `src/shaders/postprocess.wgsl` (add uniforms, HDR tonemap path)

- [ ] **Step 1: Extend PostProcessUniforms**

In `src/rendering/mod.rs`, update the struct (around line 7):

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PostProcessUniforms {
    pub game_exposure: f32,
    pub add_color_r: f32,
    pub add_color_g: f32,
    pub add_color_b: f32,
    pub mul_color_r: f32,
    pub mul_color_g: f32,
    pub mul_color_b: f32,
    pub hdr_enabled: f32,      // 0.0 = SDR, 1.0 = HDR (use f32 for alignment)
    pub paper_white: f32,
    pub max_brightness: f32,
    pub _padding: [f32; 2],    // pad to 48 bytes (multiple of 16)
}
```

- [ ] **Step 2: Update uniform upload**

In `src/main.rs` where PostProcessUniforms is constructed (around line 532-544), add the new fields:

```rust
let uniforms = PostProcessUniforms {
    game_exposure: globals.exposure.game_exposure as f32,
    add_color_r: globals.exposure.add_color.0 as f32,
    add_color_g: globals.exposure.add_color.1 as f32,
    add_color_b: globals.exposure.add_color.2 as f32,
    mul_color_r: globals.exposure.mul_color.0 as f32,
    mul_color_g: globals.exposure.mul_color.1 as f32,
    mul_color_b: globals.exposure.mul_color.2 as f32,
    hdr_enabled: if globals.hdr.hdr_enabled { 1.0 } else { 0.0 },
    paper_white: globals.hdr.paper_white as f32,
    max_brightness: globals.hdr.max_brightness as f32,
    _padding: [0.0; 2],
};
```

- [ ] **Step 3: Update postprocess.wgsl to use HDR uniforms**

Update the shader's PostProcessUniforms struct and tonemap function:

```wgsl
struct PostProcessUniforms {
    game_exposure: f32,
    add_color_r: f32,
    add_color_g: f32,
    add_color_b: f32,
    mul_color_r: f32,
    mul_color_g: f32,
    mul_color_b: f32,
    hdr_enabled: f32,
    paper_white: f32,
    max_brightness: f32,
    _padding0: f32,
    _padding1: f32,
}

fn tonemap(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);

    let with_add = hdr_color + add_color * uniforms.game_exposure;
    let with_mul = with_add * mul_color;

    if uniforms.hdr_enabled > 0.5 {
        // HDR path: scale to nits, passthrough below max, soft redirect above
        let nits = with_mul * (uniforms.paper_white / 255.0);
        let redirected = soft_redirect(nits, uniforms.max_brightness);
        return redirected / uniforms.max_brightness;
    } else {
        // SDR path: soft redirect at 255
        let redirected = soft_redirect(with_mul, 255.0);
        return redirected / 255.0;
    }
}
```

- [ ] **Step 4: Build and verify**

Run: `rtk cargo build`
Expected: Compiles. Shader and Rust struct must have matching layout.

- [ ] **Step 5: Visual test in SDR mode**

Run: `rtk cargo run -- --scenario scenarios/test_visual.ron`
Expected: Same as before (HDR defaults to off). No regression.

- [ ] **Step 6: Run tests**

Run: `rtk cargo test`
Expected: All pass.

- [ ] **Step 7: Commit**

```bash
rtk git add src/rendering/mod.rs src/main.rs src/shaders/postprocess.wgsl
rtk git commit -m "feat: add HDR uniforms to postprocess shader

Extends PostProcessUniforms with hdr_enabled, paper_white, max_brightness.
Shader now has two paths: SDR (soft redirect at 255) and HDR (scale to nits,
passthrough below max_brightness, soft redirect above). WCG: colors below
peak pass through untouched for maximum saturation."
```

---

## Task 7: Surface Format Switching (SDR ↔ HDR)

**Files:**
- Modify: `src/main.rs` (surface format selection, format change handling)
- Modify: `src/rendering/mod.rs` (pipeline recreation method)

- [ ] **Step 1: Add format-aware surface configuration**

In `src/main.rs`, replace the surface format selection (lines 111-117) with a function that selects based on HDR state:

```rust
fn select_surface_format(caps: &wgpu::SurfaceCapabilities, hdr: bool) -> wgpu::TextureFormat {
    if hdr {
        // Prefer Rgba16Float for HDR/WCG
        caps.formats.iter()
            .find(|f| **f == wgpu::TextureFormat::Rgba16Float)
            .copied()
            .unwrap_or_else(|| {
                // Fallback: any non-sRGB linear format
                caps.formats.iter()
                    .find(|f| !f.is_srgb())
                    .copied()
                    .unwrap_or(caps.formats[0])
            })
    } else {
        // Prefer sRGB for SDR
        caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0])
    }
}
```

- [ ] **Step 2: Add pipeline recreation to Renderer2D**

In `src/rendering/mod.rs`, add a method to recreate format-dependent pipelines:

```rust
impl Renderer2D {
    pub fn recreate_surface_pipelines(
        &mut self,
        device: &wgpu::Device,
        new_format: wgpu::TextureFormat,
    ) {
        // Recreate postprocess pipeline with new target format
        // Recreate HUD pipeline with new target format
        // Copy the pipeline creation code from new() but with new_format
        // Store the new format
    }
}
```

This method should duplicate the pipeline creation for postprocess (lines 390-422) and HUD (lines 564-596) from `Renderer2D::new()`, using the new format. Extract the pipeline creation into helper functions to avoid code duplication.

- [ ] **Step 3: Detect HDR toggle change in main loop**

In `src/main.rs`, in the main loop, after input handling, check if HDR state changed:

```rust
// Track previous HDR state
let mut prev_hdr_enabled = globals.hdr.hdr_enabled;

// ... in loop ...

if globals.hdr.hdr_enabled != prev_hdr_enabled {
    let new_format = select_surface_format(&surface_caps, globals.hdr.hdr_enabled);
    config.format = new_format;
    surface.configure(&device, &config);
    renderer.recreate_surface_pipelines(&device, new_format);
    prev_hdr_enabled = globals.hdr.hdr_enabled;
}
```

- [ ] **Step 4: Build and verify**

Run: `rtk cargo build`
Expected: Compiles. Pipeline recreation may need careful handling of bind group layouts.

- [ ] **Step 5: Test HDR toggle**

Run: `rtk cargo run`
Expected: Toggle HDR in pause menu. Surface format should switch. No crash. Visual appearance may change (sRGB vs linear gamma). If the monitor doesn't support HDR, the Rgba16Float format may or may not be available — the fallback should handle this gracefully.

- [ ] **Step 6: Run tests**

Run: `rtk cargo test`
Expected: All pass.

- [ ] **Step 7: Commit**

```bash
rtk git add src/main.rs src/rendering/mod.rs
rtk git commit -m "feat: runtime surface format switching for HDR toggle

HDR on: Rgba16Float (scRGB/WCG). HDR off: Bgra8UnormSrgb (sRGB).
Recreates postprocess and HUD pipelines on format change. Graceful
fallback if Rgba16Float not available."
```

---

## Task 8: HUD Brightness Scaling

**Files:**
- Modify: `src/shaders/hud.wgsl` (add brightness uniform)
- Modify: `src/rendering/mod.rs` (HUD uniform buffer and bind group)
- Modify: `src/main.rs` (upload HUD brightness per frame)

- [ ] **Step 1: Add HUD uniform to shader**

In `src/shaders/hud.wgsl`, add a uniform for brightness scaling:

```wgsl
struct HudUniforms {
    screen_size: vec2<f32>,
    brightness_scale: f32,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: HudUniforms;
```

Update the vertex shader to use `uniforms.screen_size` instead of `screen_size`:

```wgsl
let x = (in.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
let y = (in.position.y / uniforms.screen_size.y) * 2.0 - 1.0;
```

Update the fragment shader:

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(
        clamp(in.color.r / 255.0 * uniforms.brightness_scale, 0.0, 1.0),
        clamp(in.color.g / 255.0 * uniforms.brightness_scale, 0.0, 1.0),
        clamp(in.color.b / 255.0 * uniforms.brightness_scale, 0.0, 1.0),
        clamp(in.color.a / 255.0, 0.0, 1.0),
    );
}
```

- [ ] **Step 2: Update HUD bind group and buffer**

In `src/rendering/mod.rs`, the HUD currently shares `screen_size_buffer` for its bind group. Create a dedicated HUD uniform buffer that includes the brightness scale:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudUniforms {
    pub screen_width: f32,
    pub screen_height: f32,
    pub brightness_scale: f32,
    pub _padding: f32,
}
```

Create the buffer in `Renderer2D::new()` and update the HUD bind group to use it.

- [ ] **Step 3: Upload HUD brightness per frame**

In `src/main.rs`, before rendering, upload HUD uniforms:

```rust
let hud_brightness = if globals.hdr.hdr_enabled {
    (globals.hdr.hud_nits / globals.hdr.max_brightness) as f32
} else {
    1.0
};
let hud_uniforms = HudUniforms {
    screen_width: width as f32,
    screen_height: height as f32,
    brightness_scale: hud_brightness,
    _padding: 0.0,
};
renderer.update_hud_uniforms(&queue, &hud_uniforms);
```

- [ ] **Step 4: Build and verify**

Run: `rtk cargo build`
Expected: Compiles.

- [ ] **Step 5: Test HUD brightness**

Run: `rtk cargo run`
Expected: In SDR mode, HUD looks the same as before (brightness_scale=1.0). In HDR mode, HUD brightness is `hud_nits / max_brightness` = `155/1000` = 0.155 of full bright. Verify HUD is visible but not blinding in HDR.

- [ ] **Step 6: Commit**

```bash
rtk git add src/shaders/hud.wgsl src/rendering/mod.rs src/main.rs
rtk git commit -m "feat: HUD brightness scaling for HDR mode

Adds brightness_scale uniform to HUD shader. SDR: scale=1.0 (unchanged).
HDR: scale=hud_nits/max_brightness for correct nit output."
```

---

## Task 9: MSAA Configurability

**Files:**
- Modify: `src/rendering/mod.rs` (accept MSAA count, recreate pipelines)
- Modify: `src/main.rs` (detect MSAA change, trigger rebuild)

- [ ] **Step 1: Add MSAA rebuild method to Renderer2D**

In `src/rendering/mod.rs`, add a method to rebuild MSAA-dependent resources:

```rust
impl Renderer2D {
    pub fn set_msaa_sample_count(
        &mut self,
        device: &wgpu::Device,
        sample_count: u32,
    ) {
        if sample_count == self.msaa_sample_count {
            return;
        }
        self.msaa_sample_count = sample_count;
        // Recreate:
        // 1. MSAA offscreen texture (if count > 1)
        // 2. World pipeline (uses MSAA count in MultisampleState)
        // The SDF pipelines currently use MSAA=1 — keep them at 1 for now
        // (they render after MSAA resolve)
    }
}
```

- [ ] **Step 2: Detect MSAA change in main loop**

In `src/main.rs`:

```rust
let mut prev_msaa = globals.hdr.msaa_sample_count;

// ... in loop ...

if globals.hdr.msaa_sample_count != prev_msaa {
    renderer.set_msaa_sample_count(&device, globals.hdr.msaa_sample_count);
    prev_msaa = globals.hdr.msaa_sample_count;
}
```

- [ ] **Step 3: Build and verify**

Run: `rtk cargo build`
Expected: Compiles.

- [ ] **Step 4: Test MSAA cycling**

Run: `rtk cargo run`
Expected: Open pause menu, cycle MSAA (Off/x2/x4). Polygon edges should change quality. No crash on format change.

- [ ] **Step 5: Commit**

```bash
rtk git add src/rendering/mod.rs src/main.rs
rtk git commit -m "feat: runtime MSAA configurability from pause menu

Cycle MSAA Off/x2/x4 in pause menu. Recreates world pipeline and
MSAA texture on change. SDF pipelines remain at sample_count=1
(they use SDF anti-aliasing instead)."
```

---

## Task 10: Final Cleanup and Integration Test

**Files:**
- Various (minor fixes)

- [ ] **Step 1: Remove Rust-side redirect_spectre_wide if unused**

Check if `color.rs:redirect_spectre_wide` is still called anywhere. If it's only used in `rgb_of_hdr` (the old CPU tonemapping path), and `rgb_of_hdr` is dead code, remove both.

Run: `rtk cargo build` to verify no compilation errors.

- [ ] **Step 2: Run clippy**

Run: `rtk cargo clippy`
Expected: No new warnings. Fix any that appear.

- [ ] **Step 3: Run fmt**

Run: `rtk cargo fmt`

- [ ] **Step 4: Run full test suite**

Run: `rtk cargo test`
Expected: All pass.

- [ ] **Step 5: Full visual playthrough**

Run: `rtk cargo run`
Expected: Play through a full game cycle:
- Colors are correct (no B&W, no XOR)
- Pause menu shows vertical list
- Toggle options work
- MSAA cycling works
- HDR toggle works (if display supports it)
- HDR sliders adjust values
- Explosions and flashes look smooth
- SDF circles have anti-aliased edges

- [ ] **Step 6: Commit any remaining fixes**

```bash
rtk git add -A
rtk git commit -m "cleanup: remove dead code, clippy fixes, integration verified"
```

---

## Dependency Graph

```
Task 1 (alpha fix) ──────────────────────────────────────┐
Task 2 (soft redirect) ─────── depends on Task 1 ───────┤
Task 3 (zoom_factor cleanup) ── independent ─────────────┤
Task 4 (HdrConfig params) ──── independent ──────────────┤
Task 5 (pause menu) ────────── depends on Task 4 ───────┤
Task 6 (HDR uniforms) ──────── depends on Task 2, 4 ────┤
Task 7 (surface format) ────── depends on Task 6 ───────┤
Task 8 (HUD brightness) ────── depends on Task 7 ───────┤
Task 9 (MSAA config) ──────── depends on Task 5, 7 ────┤
Task 10 (cleanup) ──────────── depends on all ───────────┘
```

Tasks 1, 3, 4 can run in parallel.
Tasks 2 and 5 can run in parallel (after their deps).
Tasks 6-9 are sequential.
