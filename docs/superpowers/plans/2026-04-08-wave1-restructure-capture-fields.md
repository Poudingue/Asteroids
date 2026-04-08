# Wave 1: Code Restructure + Capture Tooling + Field Groundwork

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the three largest files (game.rs, rendering/mod.rs, hud.rs), create capture tooling module, and lay distortion field type groundwork — all without changing any runtime behavior.

**Architecture:** Pure extraction refactors (A) run first, followed by new independent modules (D, F). Each workstream touches different files and can be executed in parallel on separate worktrees.

**Tech Stack:** Rust, wgpu 24, image 0.25 (PNG), chrono 0.4

---

## Stream A: Code Restructure

### Task A1: Extract `src/update.rs` from `game.rs`

**Files:**
- Create: `src/update.rs`
- Modify: `src/game.rs`
- Modify: `src/lib.rs`

game.rs (~1572 lines) contains entity update logic mixed with game orchestration. Extract all per-frame entity update functions.

**Move to `src/update.rs`:**
- Entity motion: `move_entity`, `apply_inertia`, `accelerate_entity`, `boost_entity`, `rotate_entity`, `turn_entity`, `apply_torque`, `boost_torque`, `apply_angular_momentum`, `translate_entity`
- Batch operations: `apply_inertia_all`, `apply_angular_momentum_all`, `wrap_entity`, `wrap_entities`
- Lifecycle: `checkspawn_objet`, `transfer_oos`, `despawn`, `move_star`, `update_frame`
- Particle management: `enforce_particle_budgets`, `decay_smoke`, `decay_smoke_multiplied`
- Visual: `update_visual_aim`
- Entity predicates: `is_alive`, `is_dead`, `ischunk`, `big_enough`, `too_small`, `positive_radius`

**Keep in `game.rs`:**
- `GamepadState`, `GameState` structs
- `update_game` (top-level orchestration)
- `render_frame`
- `get_entity`, `get_entity_mut`
- Collision functions: `collect_pairs_for_cell`, `apply_collision_pairs`, `calculate_collision_tables`, `run_fragment_collisions`
- Helper functions: `hdr`, `to_rgba`, `to_hdr_rgba`

- [ ] **Step 1: Create `src/update.rs`** — move all listed functions, add necessary `use` imports at the top (crate::math::Vec2, crate::objects::*, crate::parameters::*, crate::math_utils::*, etc.)
- [ ] **Step 2: Add `pub mod update;` to `src/lib.rs`**
- [ ] **Step 3: Update `game.rs`** — remove moved functions, add `use crate::update::*;`
- [ ] **Step 4: Run `cargo check`** — fix any missing imports or visibility issues
- [ ] **Step 5: Run `cargo test`** — ensure all existing tests pass
- [ ] **Step 6: Commit** — `git commit -m "refactor: extract update.rs from game.rs"`

---

### Task A2: Extract `src/spawning.rs` from `objects.rs`

**Files:**
- Create: `src/spawning.rs`
- Modify: `src/objects.rs`
- Modify: `src/lib.rs`

objects.rs (~910 lines) mixes entity type definitions with spawn/factory functions.

**Move to `src/spawning.rs`:**
- Ship: `spawn_ship`
- Projectiles: `spawn_projectile`, `spawn_n_projectiles`, `spawn_muzzle`
- Explosions: `spawn_explosion_chunk`, `spawn_n_chunks`, `spawn_explosion`, `spawn_explosion_object`, `spawn_explosion_death`, `spawn_chunk_explosion`
- Fire: `spawn_fire`
- Asteroids: `generate_asteroid_polygon`, `spawn_asteroid`, `spawn_random_asteroid`, `fragment_asteroid`, `spawn_fragments`
- Stars: `random_offscreen_position`, `spawn_random_star`, `spawn_stars`

**Keep in `objects.rs`:**
- `EntityKind` enum
- `Polygon`, `Hitbox`, `Visuals`, `Entity`, `Star`, `ExplosionObjectSideEffects` structs
- Entity predicates: `is_alive`, `is_dead`, `positive_radius`, `is_chunk`, `not_chunk`, `too_small`, `big_enough`, `close_enough`, `too_far`, `check_spawn`, `check_not_spawn`

**Note:** Some entity predicates exist in BOTH game.rs and objects.rs. Consolidate duplicates in objects.rs during this extraction. update.rs (from A1) should import from objects.rs.

- [ ] **Step 1: Create `src/spawning.rs`** — move all spawn functions, add imports
- [ ] **Step 2: Add `pub mod spawning;` to `src/lib.rs`**
- [ ] **Step 3: Update `objects.rs`** — remove moved functions
- [ ] **Step 4: Deduplicate predicates** — if `is_alive`, `is_dead`, etc. exist in both game.rs and objects.rs, keep them in objects.rs only. Update update.rs to import from objects.
- [ ] **Step 5: Update all call sites** — game.rs, update.rs → `use crate::spawning::*;`
- [ ] **Step 6: Run `cargo check && cargo test`**
- [ ] **Step 7: Commit** — `git commit -m "refactor: extract spawning.rs from objects.rs"`

---

### Task A3: Extract `src/rendering/pipeline.rs` from `rendering/mod.rs`

**Files:**
- Create: `src/rendering/pipeline.rs`
- Modify: `src/rendering/mod.rs`

rendering/mod.rs (~1464 lines) has ~450 lines of pipeline creation in `Renderer2D::new()`.

**Move to `pipeline.rs`:**
- All render pipeline creation (world, sdf_circle, sdf_capsule, postprocess, hud)
- Bind group layout creation
- Shader module loading
- Vertex descriptor implementations (`Vertex::desc()`, `CircleInstance::desc()`, `CapsuleInstance::desc()`)

**Create these public functions:**

```rust
pub fn create_world_pipeline(
    device: &wgpu::Device, format: wgpu::TextureFormat,
    msaa: u32, bind_group_layout: &wgpu::BindGroupLayout
) -> wgpu::RenderPipeline

pub fn create_sdf_circle_pipeline(
    device: &wgpu::Device, format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout
) -> wgpu::RenderPipeline

pub fn create_sdf_capsule_pipeline(
    device: &wgpu::Device, format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout
) -> wgpu::RenderPipeline

pub fn create_postprocess_pipeline(
    device: &wgpu::Device, format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout
) -> wgpu::RenderPipeline

pub fn create_hud_pipeline(
    device: &wgpu::Device, format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout
) -> wgpu::RenderPipeline

pub fn create_screen_size_bind_group_layout(
    device: &wgpu::Device
) -> wgpu::BindGroupLayout
```

**Deduplication fix:** The same bind group layout is created in both `new()` and `resize()`. Extract to `create_screen_size_bind_group_layout()` and call it from both places.

- [ ] **Step 1: Create `src/rendering/pipeline.rs`** with all pipeline creation functions
- [ ] **Step 2: Add `pub mod pipeline;` to `rendering/mod.rs`** (not lib.rs — it's a submodule of rendering)
- [ ] **Step 3: Update `Renderer2D::new()`** to call the new functions
- [ ] **Step 4: Fix bind group layout duplication** — use `create_screen_size_bind_group_layout()` in both `new()` and `resize()`
- [ ] **Step 5: Run `cargo check`**
- [ ] **Step 6: Commit** — `git commit -m "refactor: extract rendering/pipeline.rs, fix bind group layout duplication"`

---

### Task A4: Extract `src/rendering/textures.rs` from `rendering/mod.rs`

**Files:**
- Create: `src/rendering/textures.rs`
- Modify: `src/rendering/mod.rs`

**Move to `textures.rs`:**
- `create_offscreen_texture(device, width, height)` — already a free function
- `create_msaa_texture(device, width, height, sample_count)` — already a free function

- [ ] **Step 1: Create `src/rendering/textures.rs`** with both functions
- [ ] **Step 2: Add `pub mod textures;` to `rendering/mod.rs`**
- [ ] **Step 3: Update imports** in mod.rs: `use textures::{create_offscreen_texture, create_msaa_texture};`
- [ ] **Step 4: Run `cargo check`**
- [ ] **Step 5: Commit** — `git commit -m "refactor: extract rendering/textures.rs"`

---

### Task A5: Extract `src/glyphs.rs` from `rendering/hud.rs`

**Files:**
- Create: `src/glyphs.rs`
- Modify: `src/rendering/hud.rs`
- Modify: `src/lib.rs`

hud.rs (~1093 lines) has ~570 lines of hand-coded glyph polygons. Extract the font system to a top-level module (not under rendering/ — glyphs are data, not rendering).

**Move to `src/glyphs.rs`:**
- `shape_char` function (lines 11-580 — the entire match block of polygon coordinates)
- `displacement` function
- `displace_shape` function

**Add new entry point:**

```rust
/// Main glyph lookup. Returns polygon points for a character.
/// Falls back to filled unit square for unknown characters.
pub fn glyph(c: char) -> Vec<(f64, f64)> {
    let shape = shape_char(c);
    if shape.is_empty() {
        vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
    } else {
        shape
    }
}
```

- [ ] **Step 1: Create `src/glyphs.rs`** with shape_char, displacement, displace_shape, and glyph()
- [ ] **Step 2: Add `pub mod glyphs;` to `src/lib.rs`**
- [ ] **Step 3: Update `hud.rs`** — remove moved functions, add `use crate::glyphs::{glyph, displacement, displace_shape};`
- [ ] **Step 4: Replace `shape_char` calls in hud.rs** with `glyph` calls
- [ ] **Step 5: Run `cargo check && cargo test`**
- [ ] **Step 6: Commit** — `git commit -m "refactor: extract glyphs.rs from hud.rs"`

---

### Task A6: Final restructure verification

- [ ] **Step 1: Run `cargo check && cargo clippy && cargo test && cargo fmt`**
- [ ] **Step 2: Verify line counts** — game.rs should be ~300-400, rendering/mod.rs ~400-500, hud.rs ~400-500
- [ ] **Step 3: Commit any fmt fixes** — `git commit -m "style: cargo fmt after restructure"`

---

## Stream D: Capture Tooling

### Task D1: Add dependencies to Cargo.toml

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add under [dependencies]:**
```toml
image = { version = "0.25", default-features = false, features = ["png"] }
chrono = "0.4"
```
- [ ] **Step 2: Run `cargo check`**
- [ ] **Step 3: Commit** — `git commit -m "deps: add image (PNG) and chrono for capture tooling"`

---

### Task D2: Create `src/capture.rs` — path helpers and VideoCapture

**Files:**
- Create: `src/capture.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write tests in `src/capture.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshot_path_has_png_extension() {
        let path = screenshot_path();
        assert_eq!(path.extension().unwrap(), "png");
        assert!(path.to_str().unwrap().starts_with("screenshots/asteroids_"));
    }

    #[test]
    fn video_capture_starts_inactive() {
        let vc = VideoCapture::new();
        assert!(!vc.is_active());
    }

    #[test]
    fn video_capture_toggle() {
        let mut vc = VideoCapture::new();
        vc.start();
        assert!(vc.is_active());
        vc.stop();
        assert!(!vc.is_active());
    }

    #[test]
    fn frame_paths_increment() {
        let dir = std::path::PathBuf::from("/tmp/test_session");
        assert_eq!(frame_path(&dir, 0), dir.join("frame_00000.png"));
        assert_eq!(frame_path(&dir, 42), dir.join("frame_00042.png"));
    }
}
```

- [ ] **Step 2: Write implementation:**

```rust
use chrono::Local;
use std::path::PathBuf;

pub struct VideoCapture {
    session_dir: PathBuf,
    frame_count: u32,
    active: bool,
}

pub fn screenshot_path() -> PathBuf {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    PathBuf::from(format!("screenshots/asteroids_{}.png", timestamp))
}

pub fn capture_session_dir() -> PathBuf {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    PathBuf::from(format!("captures/session_{}", timestamp))
}

fn frame_path(session_dir: &std::path::Path, frame: u32) -> PathBuf {
    session_dir.join(format!("frame_{:05}.png", frame))
}

impl VideoCapture {
    pub fn new() -> Self {
        Self { session_dir: PathBuf::new(), frame_count: 0, active: false }
    }

    pub fn is_active(&self) -> bool { self.active }

    pub fn start(&mut self) {
        self.session_dir = capture_session_dir();
        std::fs::create_dir_all(&self.session_dir).expect("Failed to create capture directory");
        self.frame_count = 0;
        self.active = true;
    }

    pub fn stop(&mut self) { self.active = false; }

    pub fn toggle(&mut self) {
        if self.active { self.stop(); } else { self.start(); }
    }

    pub fn next_frame_path(&mut self) -> PathBuf {
        let path = frame_path(&self.session_dir, self.frame_count);
        self.frame_count += 1;
        path
    }
}
```

- [ ] **Step 3: Add `pub mod capture;` to lib.rs**
- [ ] **Step 4: Run `cargo test`** — all capture tests pass
- [ ] **Step 5: Commit** — `git commit -m "feat: add capture.rs with path helpers and VideoCapture"`

---

### Task D3: PNG save function

**Files:**
- Modify: `src/capture.rs`

- [ ] **Step 1: Add test:**

```rust
#[test]
fn save_png_creates_file() {
    let dir = std::env::temp_dir().join("claude_test_capture");
    let path = dir.join("test.png");
    let data = vec![255u8; 4 * 2 * 2]; // 2x2 white RGBA
    save_png(&path, &data, 2, 2);
    assert!(path.exists());
    std::fs::remove_dir_all(&dir).ok();
}
```

- [ ] **Step 2: Add implementation:**

```rust
pub fn save_png(path: &std::path::Path, data: &[u8], width: u32, height: u32) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create screenshot directory");
    }
    let img = image::RgbaImage::from_raw(width, height, data.to_vec())
        .expect("Failed to create image from pixel data");
    img.save(path).expect("Failed to save PNG");
}
```

- [ ] **Step 3: Run `cargo test`**
- [ ] **Step 4: Commit** — `git commit -m "feat: add save_png for capture tooling"`

---

### Task D4: Add GameStateSnapshot to recording.rs

**Files:**
- Modify: `src/recording.rs`

- [ ] **Step 1: Add structs:**

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjectSnapshot {
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub speed_x: f64,
    pub speed_y: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct GameStateSnapshot {
    pub frame: u64,
    pub entity_count: usize,
    pub objects: Vec<ObjectSnapshot>,
}
```

- [ ] **Step 2: Add tests:**

```rust
#[test]
fn game_state_snapshot_default() {
    let snap = GameStateSnapshot::default();
    assert_eq!(snap.frame, 0);
    assert_eq!(snap.entity_count, 0);
    assert!(snap.objects.is_empty());
}

#[test]
fn game_state_snapshot_roundtrip() {
    let snap = GameStateSnapshot {
        frame: 42,
        entity_count: 3,
        objects: vec![ObjectSnapshot {
            kind: "asteroid".to_string(),
            x: 1.0, y: 2.0, radius: 5.0,
            speed_x: 0.1, speed_y: -0.2,
        }],
    };
    let bytes = bincode::serialize(&snap).unwrap();
    let restored: GameStateSnapshot = bincode::deserialize(&bytes).unwrap();
    assert_eq!(restored.frame, 42);
    assert_eq!(restored.objects.len(), 1);
    assert_eq!(restored.objects[0].kind, "asteroid");
}
```

- [ ] **Step 3: Run `cargo test`**
- [ ] **Step 4: Commit** — `git commit -m "feat: add GameStateSnapshot/ObjectSnapshot to recording.rs"`

---

### Task D5: Wire capture flags in parameters.rs and pause_menu.rs

**Files:**
- Modify: `src/parameters.rs`
- Modify: `src/pause_menu.rs`

- [ ] **Step 1: Add to `GlobalToggle` enum:**
```rust
RecordScenario,
```

- [ ] **Step 2: Add fields to appropriate config struct** (TimeConfig or new section in Globals):
```rust
pub screenshot_requested: bool,  // set true on F12, consumed after capture
pub video_capture_active: bool,  // toggled by F10
```

- [ ] **Step 3: Add "Record Scenario" toggle to pause menu** (MenuEntryKind::Toggle)
- [ ] **Step 4: Run `cargo check`**
- [ ] **Step 5: Commit** — `git commit -m "feat: wire capture flags in parameters and pause menu"`

**Note:** F12/F10 key handling in main.rs event loop and GPU readback integration happen in Wave 2 after the layer renderer is in place.

---

## Stream F: Field Groundwork

### Task F1: Create `src/field.rs` with types

**Files:**
- Create: `src/field.rs`
- Modify: `src/lib.rs`
- Modify: `src/game.rs`

- [ ] **Step 1: Write tests:**

```rust
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
            kind: FieldSourceKind::GravityWell { strength: 100.0, radius: 50.0 },
            position: Vec2 { x: 10.0, y: 10.0 },
            age: 0.0,
            lifetime: 10.0,
        }];
        let sample = evaluate_field(pos, &sources);
        assert_eq!(sample.time_dilation, 1.0);
    }
}
```

- [ ] **Step 2: Write implementation:**

```rust
use crate::math::Vec2;

#[derive(Debug, Clone)]
pub enum FieldSourceKind {
    ShockwaveRing { speed: f64, width: f64, pressure: f64 },
    GravityWell { strength: f64, radius: f64 },
    Vortex { angular_speed: f64, radius: f64 },
    WindZone { direction: Vec2, strength: f64, radius: f64 },
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
```

- [ ] **Step 3: Add `pub mod field;` to lib.rs**
- [ ] **Step 4: Add `pub field_sources: Vec<crate::field::FieldSource>` to `GameState`** in game.rs, initialized as `Vec::new()`
- [ ] **Step 5: Run `cargo test`**
- [ ] **Step 6: Commit** — `git commit -m "feat: add field.rs with distortion field types (groundwork)"`

---

## Final Wave 1 Verification

- [ ] **Run `cargo check && cargo clippy && cargo test && cargo fmt`**
- [ ] **Verify no runtime behavior change** — game looks and plays identically
- [ ] **Commit any remaining fixes**
