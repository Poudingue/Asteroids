# Wave 2: Layered Compositing Renderer + i18n Groundwork

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the 2-pass renderer (polygon→SDF) with an ordered 7-layer compositing system, and create the i18n locale module skeleton.

**Architecture:** The renderer gains per-layer instance buffers. SDF layers draw directly to the offscreen target. Polygon layers (4, 6) use MSAA then resolve. Postprocess sits between layer 5 and HUD. An additive-blend SDF pipeline variant handles capsule trails and explosions.

**Tech Stack:** Rust, wgpu 24, sys-locale 0.3

**Depends on:** Wave 1 complete (code restructure — pipeline.rs, textures.rs, glyphs.rs extracted)

---

## Stream B: Layered Compositing Renderer

### Task B1: Add per-layer instance buffers to Renderer2D

**Files:**
- Modify: `src/rendering/mod.rs`

Replace the single `sdf_circle_instances` and `sdf_capsule_instances` with per-layer collections.

- [ ] **Step 1: Add new fields to `Renderer2D`:**

```rust
// Layer 1: star trails (additive blend capsules)
pub star_trail_capsules: Vec<CapsuleInstance>,
// Layer 2: bullet trails (additive blend capsules)
pub bullet_trail_capsules: Vec<CapsuleInstance>,
// Layer 3: smoke (alpha blend circles)
pub smoke_circles: Vec<CircleInstance>,
// Layer 4: polygons — track where entity polygons start in vertices buffer
pub polygon_vertex_start: usize,
// Layer 5: effects — explosions, sparkles (additive blend circles)
pub effect_circles: Vec<CircleInstance>,
```

- [ ] **Step 2: Add `clear_layer_buffers()` method:**

```rust
pub fn clear_layer_buffers(&mut self) {
    self.star_trail_capsules.clear();
    self.bullet_trail_capsules.clear();
    self.smoke_circles.clear();
    self.polygon_vertex_start = 0;
    self.effect_circles.clear();
}
```

- [ ] **Step 3: Call `clear_layer_buffers()` in `begin_frame()`** (alongside existing vertices/hud_vertices clear)

- [ ] **Step 4: Initialize new Vec fields in `Renderer2D::new()`** as `Vec::new()`

- [ ] **Step 5: Run `cargo check`**
- [ ] **Step 6: Commit** — `git commit -m "refactor: add per-layer instance buffers to Renderer2D"`

---

### Task B2: Create additive-blend SDF pipeline variants

**Files:**
- Modify: `src/rendering/pipeline.rs`
- Modify: `src/rendering/mod.rs`

- [ ] **Step 1: Add to `pipeline.rs` — two new pipeline creation functions:**

```rust
pub fn create_sdf_circle_additive_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Identical to create_sdf_circle_pipeline except blend state:
    // blend: Some(wgpu::BlendState {
    //     color: wgpu::BlendComponent {
    //         src_factor: wgpu::BlendFactor::One,
    //         dst_factor: wgpu::BlendFactor::One,
    //         operation: wgpu::BlendOperation::Add,
    //     },
    //     alpha: wgpu::BlendComponent {
    //         src_factor: wgpu::BlendFactor::One,
    //         dst_factor: wgpu::BlendFactor::One,
    //         operation: wgpu::BlendOperation::Add,
    //     },
    // })
    // Copy the existing create_sdf_circle_pipeline and change only the blend state.
}

pub fn create_sdf_capsule_additive_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Same as create_sdf_capsule_pipeline with additive blend state.
}
```

- [ ] **Step 2: Add pipeline fields to `Renderer2D`:**

```rust
pub sdf_circle_additive_pipeline: wgpu::RenderPipeline,
pub sdf_capsule_additive_pipeline: wgpu::RenderPipeline,
```

- [ ] **Step 3: Create them in `Renderer2D::new()`** alongside existing pipelines

- [ ] **Step 4: Run `cargo check`**
- [ ] **Step 5: Commit** — `git commit -m "feat: add additive-blend SDF pipeline variants"`

---

### Task B3: Rewrite `end_frame()` for layered rendering

**Files:**
- Modify: `src/rendering/mod.rs`

This is the core architectural change. Replace the current 4-pass `end_frame()` with the layered sequence.

**Current `end_frame()` (4 passes):**
1. World Pass: clear offscreen, draw `self.vertices` with MSAA → resolve to offscreen_view
2. SDF Pass: load offscreen_view, draw `sdf_circle_instances` + `sdf_capsule_instances`
3. Postprocess: tonemap offscreen → swapchain
4. HUD: draw `hud_vertices` → swapchain

**New `end_frame()` (9 steps):**

- [ ] **Step 1: Implement new layered render sequence:**

```rust
pub fn end_frame(&mut self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) {
    let frame = surface.get_current_texture().expect("Failed to get surface texture");
    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render") });

    // === Layer 0: Background rect → offscreen (clear, no MSAA) ===
    // Draw vertices[0..polygon_vertex_start] — just the background fill rect
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("layer0_background"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.offscreen_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(/*clear_color*/), store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.world_pipeline); // Note: world_pipeline targets Rgba16Float
        // But world_pipeline has MSAA — need a non-MSAA polygon pipeline for background
        // OR: just draw background into the MSAA pass too (simpler)
        // DECISION: Draw background as part of layer 4's polygon pass instead of separately.
        // Actually no — background must be BEHIND SDF layers 1-3.
        // SOLUTION: Create a simple non-MSAA polygon pipeline for background, OR
        //   use the SDF circle pipeline to draw a fullscreen rect, OR
        //   just clear the offscreen to the background color directly.
        // SIMPLEST: Use the clear color as the background. The background rect IS the clear color.
        // If the background rect has a computed color, pass it as the clear color to the first pass.
    }

    // === Layers 1-2: Star + bullet trail capsules → offscreen (additive blend) ===
    if !self.star_trail_capsules.is_empty() || !self.bullet_trail_capsules.is_empty() {
        // Upload combined capsule data (stars first, then bullets — both additive)
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("layer1_2_trails"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.offscreen_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.sdf_capsule_additive_pipeline);
        pass.set_bind_group(0, &self.sdf_bind_group, &[]);
        // Draw star_trail_capsules + bullet_trail_capsules as one batch
        // (both are additive, same pipeline — can be concatenated)
        let all_capsules: Vec<CapsuleInstance> = self.star_trail_capsules.iter()
            .chain(self.bullet_trail_capsules.iter())
            .cloned().collect();
        // Upload to GPU buffer and draw
    }

    // === Layer 3: Smoke circles → offscreen (alpha blend) ===
    if !self.smoke_circles.is_empty() {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("layer3_smoke"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.offscreen_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.sdf_circle_pipeline);
        pass.set_bind_group(0, &self.sdf_bind_group, &[]);
        // Draw smoke_circles
    }

    // === Layer 4: Polygon entities → MSAA → resolve to offscreen ===
    let polygon_count = self.vertices.len() - self.polygon_vertex_start;
    if polygon_count > 0 {
        if self.msaa_sample_count > 1 {
            // Draw to msaa_offscreen_texture, resolve to offscreen_view
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("layer4_polygons_msaa"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.msaa_offscreen_view.as_ref().unwrap(),
                    resolve_target: Some(&self.offscreen_view),
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.world_pipeline);
            pass.set_bind_group(0, &self.world_bind_group, &[]);
            // Draw vertices[polygon_vertex_start..]
        } else {
            // Draw directly to offscreen_view
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("layer4_polygons"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.offscreen_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            // Need a non-MSAA world pipeline for this case
            // OR: the existing world_pipeline already handles sample_count=1
            pass.set_pipeline(&self.world_pipeline);
            pass.set_bind_group(0, &self.world_bind_group, &[]);
        }
    }

    // === Layer 5: Effect circles (explosions) → offscreen (additive blend) ===
    if !self.effect_circles.is_empty() {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("layer5_effects"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.offscreen_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.sdf_circle_additive_pipeline);
        pass.set_bind_group(0, &self.sdf_bind_group, &[]);
        // Draw effect_circles
    }

    // === Postprocess: tonemap offscreen → swapchain ===
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("postprocess"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.postprocess_pipeline);
        pass.set_bind_group(0, &self.postprocess_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    // === Layer 6: HUD → swapchain (alpha blend, no MSAA) ===
    if !self.hud_vertices.is_empty() {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("layer6_hud"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.hud_pipeline);
        pass.set_bind_group(0, &self.hud_bind_group, &[]);
        // Draw hud_vertices
    }

    queue.submit(std::iter::once(encoder.finish()));
    frame.present();
}
```

**Important implementation notes:**
- The background color must be passed as the `Clear` color for layer 0 (the first pass). Currently `render_frame` computes an HDR background color and draws a fill rect — instead, pass that color to `end_frame` as the clear color. Add a `pub clear_color: wgpu::Color` field to Renderer2D.
- For MSAA layer 4: the MSAA pass uses `Load` (not `Clear`) so it composites on top of what's already in offscreen. But MSAA resolve needs the MSAA texture to contain only the polygon content, then resolve adds it to offscreen. This means MSAA layer needs `Clear` on the MSAA texture and the resolve writes to offscreen. **Wait** — wgpu resolve replaces, not blends. So we need to: (a) copy offscreen to MSAA texture first, (b) draw polygons on top, (c) resolve back. OR: use `Load` on MSAA texture if it shares content with offscreen. **Actually**, the cleanest approach: for MSAA layer 4, clear the MSAA texture, draw polygons, resolve to a TEMP texture, then composite (blend) the temp onto offscreen. This is complex. **Simpler**: skip MSAA resolve for compositing and instead just draw polygons directly to offscreen with a higher sample count. But wgpu doesn't support that easily. **Simplest practical approach**: accept that MSAA resolve overwrites the target region. Since polygons (asteroids, ship) are opaque and fill their area, the resolve overwriting is acceptable — the SDF content underneath (smoke, stars) that overlaps polygon areas would be hidden by the polygon anyway. So `Clear` the MSAA texture, draw polygons, resolve to offscreen, accepting that polygon areas overwrite SDF content beneath them. This is the correct visual result for opaque polygons on top of SDF background layers.

- [ ] **Step 2: Handle background color** — add `pub clear_color: wgpu::Color` to Renderer2D. Set it in `render_frame` before rendering starts. Use it as the Clear color for the first render pass.

- [ ] **Step 3: Handle GPU buffer uploads** — each layer's instance data needs to be uploaded to a GPU buffer before the draw call. Use `device.create_buffer_init()` for each non-empty layer, same pattern as current SDF pass.

- [ ] **Step 4: Run `cargo check`**
- [ ] **Step 5: Commit** — `git commit -m "feat: layered compositing renderer — rewrite end_frame()"`

---

### Task B4: Update `render_frame()` to populate layer buffers

**Files:**
- Modify: `src/game.rs`
- Modify: `src/rendering/world.rs`

Current `render_frame()` pushes everything to `renderer.vertices` (polygons) and `renderer.sdf_circle_instances` / `renderer.sdf_capsule_instances` (SDF). Redirect to per-layer buffers.

- [ ] **Step 1: Modify world.rs render functions** to accept explicit target buffers:

Current signatures like:
```rust
pub fn render_star_trail(renderer: &mut Renderer2D, ...)
```

Change to accept the specific target:
```rust
pub fn render_star_trail(capsule_target: &mut Vec<CapsuleInstance>, ...)
// Instead of using renderer.sdf_capsule_instances, push to capsule_target
```

Similarly for:
- `render_trail` → takes `capsule_target: &mut Vec<CapsuleInstance>`
- `render_visuals` / `render_shapes` → takes `circle_target: &mut Vec<CircleInstance>` for SDF circles and still uses `renderer.vertices` for polygons
- `render_chunk` → takes `circle_target: &mut Vec<CircleInstance>`
- `render_projectile` → takes `capsule_target: &mut Vec<CapsuleInstance>`

- [ ] **Step 2: Update `render_frame()` in game.rs:**

```rust
pub fn render_frame(state: &GameState, renderer: &mut Renderer2D, globals: &Globals) {
    // Background: set clear color instead of drawing a rect
    let bg = hdr(globals.visual.bg_r, globals.visual.bg_g, globals.visual.bg_b, globals);
    renderer.clear_color = wgpu::Color {
        r: bg.r as f64, g: bg.g as f64, b: bg.b as f64, a: 1.0,
    };

    // Mark where entity polygons start (after background, which is now clear color)
    renderer.polygon_vertex_start = renderer.vertices.len();
    // Actually, if background is now handled by clear, polygon_vertex_start = 0
    // No background vertices needed at all.

    // Layer 1: Star trails
    for star in &state.stars {
        render_star_trail(&mut renderer.star_trail_capsules, star, globals);
    }

    // Layer 2: Bullet trails
    for entity in &state.entities {
        if entity.kind == EntityKind::Projectile {
            render_projectile(&mut renderer.bullet_trail_capsules, renderer, entity, globals);
        }
    }

    // Layer 3: Smoke
    for entity in &state.entities {
        if is_smoke(entity) {
            render_visuals(&mut renderer.smoke_circles, renderer, entity, globals);
        }
    }

    // Layer 4: Polygons (asteroids, fragments, ship)
    renderer.polygon_vertex_start = renderer.vertices.len();
    // Draw chunks, fragments, asteroids, ship into renderer.vertices
    for entity in &state.entities {
        match entity.kind {
            EntityKind::Asteroid | EntityKind::Fragment | EntityKind::Ship => {
                render_visuals_polygon_only(renderer, entity, globals);
            }
            EntityKind::Chunk => {
                render_chunk_polygon_only(renderer, entity, globals);
            }
            _ => {}
        }
    }

    // Layer 5: Effects (explosions, chunk circles)
    for entity in &state.entities {
        if is_explosion(entity) || is_chunk(entity) {
            render_visuals_circles_only(&mut renderer.effect_circles, entity, globals);
        }
    }

    // HUD (layer 6) — unchanged, still uses hud_vertices
    if !globals.time.pause {
        render_hud(renderer, state, globals);
    }
    if globals.time.pause {
        state.pause_menu.render(renderer, globals);
    }
}
```

**Note:** The exact mapping depends on which entities produce circles vs polygons vs both. `render_visuals` currently pushes BOTH polygons and circles for entities that have both (e.g., asteroids with polygon shape + circle hitbox visualization). The split needs to separate these concerns. Some entities may need two render calls — one for their polygon layer and one for their circle layer.

- [ ] **Step 3: Remove old `sdf_circle_instances` and `sdf_capsule_instances`** from Renderer2D (replaced by per-layer buffers)

- [ ] **Step 4: Run `cargo check`**
- [ ] **Step 5: Visual test** — stars behind asteroids, smoke behind ship, explosions on top
- [ ] **Step 6: Commit** — `git commit -m "feat: populate per-layer buffers in render_frame()"`

---

### Task B5: Clean up old 2-pass code

**Files:**
- Modify: `src/rendering/mod.rs`

- [ ] **Step 1: Run `cargo clippy`** to find dead code
- [ ] **Step 2: Remove** unused fields (`sdf_circle_instances`, `sdf_capsule_instances`), methods (`push_circle_instance`, `push_capsule_instance`), and imports
- [ ] **Step 3: Run `cargo test`**
- [ ] **Step 4: Commit** — `git commit -m "refactor: remove old 2-pass rendering code"`

---

### Task B6: Layer renderer verification

- [ ] **Step 1: Run `cargo check && cargo clippy && cargo test && cargo fmt`**
- [ ] **Step 2: Visual test checklist:**
  - Stars render BEHIND asteroids
  - Smoke renders BEHIND ship
  - Explosions render ON TOP of asteroids
  - Bullet trails render BEHIND asteroids (explosion circles show impact on top)
  - Ship renders ON TOP of everything in layer 4
  - HUD renders on top of everything
  - MSAA toggle still works (polygon edges only)
  - Additive blend visible on capsule trails (bright glow where trails overlap)
- [ ] **Step 3: Commit any fixes**

---

## Stream G: i18n Groundwork

### Task G1: Add sys-locale dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add under [dependencies]:**
```toml
sys-locale = "0.3"
```

- [ ] **Step 2: Run `cargo check`**
- [ ] **Step 3: Commit** — `git commit -m "deps: add sys-locale for i18n groundwork"`

---

### Task G2: Create `src/locale.rs` skeleton

**Files:**
- Create: `src/locale.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detect_locale_returns_en() {
        assert_eq!(detect_system_locale(), "en");
    }

    #[test]
    fn locale_get_returns_value() {
        let mut strings = HashMap::new();
        strings.insert("greeting".to_string(), "Hello".to_string());
        let locale = Locale { strings, fallback: None };
        assert_eq!(locale.get("greeting"), "Hello");
    }

    #[test]
    fn locale_get_returns_key_on_miss() {
        let locale = Locale { strings: HashMap::new(), fallback: None };
        assert_eq!(locale.get("missing_key"), "missing_key");
    }

    #[test]
    fn locale_fallback_chain() {
        let mut en_strings = HashMap::new();
        en_strings.insert("greeting".to_string(), "Hello".to_string());
        en_strings.insert("farewell".to_string(), "Goodbye".to_string());
        let en = Locale { strings: en_strings, fallback: None };

        let mut fr_strings = HashMap::new();
        fr_strings.insert("greeting".to_string(), "Bonjour".to_string());
        let fr = Locale { strings: fr_strings, fallback: None }
            .with_fallback(en);

        assert_eq!(fr.get("greeting"), "Bonjour");
        assert_eq!(fr.get("farewell"), "Goodbye");
        assert_eq!(fr.get("unknown"), "unknown");
    }

    #[test]
    fn locale_load_from_ron_file() {
        let dir = std::env::temp_dir().join("claude_test_locale");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ron");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, r#"{{"greeting": "Hi", "quit": "Exit"}}"#).unwrap();

        let locale = Locale::load(&path).unwrap();
        assert_eq!(locale.get("greeting"), "Hi");
        assert_eq!(locale.get("quit"), "Exit");
        std::fs::remove_dir_all(&dir).ok();
    }
}
```

- [ ] **Step 2: Write implementation:**

```rust
use std::collections::HashMap;
use std::path::Path;

pub struct Locale {
    pub strings: HashMap<String, String>,
    pub fallback: Option<Box<Locale>>,
}

impl Locale {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read locale file {:?}: {}", path, e))?;
        let strings: HashMap<String, String> = ron::from_str(&content)
            .map_err(|e| format!("Failed to parse locale file {:?}: {}", path, e))?;
        Ok(Self { strings, fallback: None })
    }

    pub fn with_fallback(mut self, fallback: Locale) -> Self {
        self.fallback = Some(Box::new(fallback));
        self
    }

    pub fn get(&self, key: &str) -> &str {
        if let Some(value) = self.strings.get(key) {
            value
        } else if let Some(ref fallback) = self.fallback {
            fallback.get(key)
        } else {
            key
        }
    }
}

pub fn detect_system_locale() -> String {
    "en".to_string()
}
```

- [ ] **Step 3: Add `pub mod locale;` to lib.rs**
- [ ] **Step 4: Run `cargo test`**
- [ ] **Step 5: Commit** — `git commit -m "feat: add locale.rs skeleton with RON loading and fallback chain"`

---

### Task G3: Create minimal `locales/en.ron`

**Files:**
- Create: `locales/en.ron`

- [ ] **Step 1: Create `locales/` directory and `en.ron`:**

```ron
{
    "pause_title": "PAUSED",
    "resume": "RESUME",
}
```

- [ ] **Step 2: Add integration test** in locale.rs:

```rust
#[test]
fn load_english_locale() {
    let locale = Locale::load(std::path::Path::new("locales/en.ron")).unwrap();
    assert_eq!(locale.get("pause_title"), "PAUSED");
}
```

- [ ] **Step 3: Run `cargo test`**
- [ ] **Step 4: Commit** — `git commit -m "feat: add minimal English locale file"`

---

### Task G4: Update `glyphs.rs` entry point for i18n readiness

**Files:**
- Modify: `src/glyphs.rs`
- Modify: `src/rendering/hud.rs`

Wave 1 extracted `shape_char` and added `glyph()`. Now enhance for i18n:

- [ ] **Step 1: Remove `to_ascii_uppercase()` coercion** in `render_string` (hud.rs). Currently, all input is uppercased before rendering — remove this so lowercase chars can be rendered once glyphs exist.

- [ ] **Step 2: Verify `glyph()` returns unit-square fallback** for lowercase chars (they'll show as filled squares until lowercase glyphs are added in the i18n phase).

- [ ] **Step 3: Run `cargo check && cargo test`**
- [ ] **Step 4: Commit** — `git commit -m "feat: remove uppercase coercion in render_string for i18n readiness"`

---

### Task G5: Stream G verification

- [ ] **Step 1: Run `cargo check && cargo clippy && cargo test && cargo fmt`**
- [ ] **Step 2: Visual check** — HUD text renders identically (all existing strings are uppercase anyway)
- [ ] **Step 3: Commit any fixes**

---

## Final Wave 2 Verification

- [ ] **Run full test suite**
- [ ] **Visual test:** layer ordering correct, additive blending visible, HUD unchanged
- [ ] **Commit any remaining fixes**
