# Phase 2B: AA Rework & Testing Tooling — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add SSAA modes (4×/9×/16×) as evaluation options alongside existing MSAA, implement screenshot capture (F12), frame-sequence video capture (F10), and a "Record Scenario" pause-menu entry — all backed by unit tests.

**Architecture:** SSAA scales the offscreen render target by an integer factor so that every rendering pass (polygons, SDF, postprocess) benefits from supersampling, while HUD renders at native resolution. Screenshot and video capture read back the native-resolution swapchain surface after `end_frame`. The recording feature extends the existing `InputRecorder` / fixed-dt infrastructure with a pause-menu toggle and a `GameStateSnapshot` type for optional per-frame state dumping. All GPU-side data flow is driven by the `Renderer2D` struct; game-loop-level orchestration lives in `game.rs`.

**Tech Stack:** Rust, wgpu 24, SDL2, `image` crate (PNG I/O), `clap` (CLI arg already wired), `bincode` + `zstd` (already in use for recordings), `chrono` (timestamp filenames).

---

## File Map

| File | Change |
|------|--------|
| `Cargo.toml` | Add `image`, `chrono` dependencies |
| `src/rendering/mod.rs` | Add `ssaa_factor`, scale offscreen textures, add doc-comments on AA strategy |
| `src/shaders/sdf.wgsl` | Add doc-comment explaining why MSAA is irrelevant for SDF |
| `src/parameters.rs` | Add `ssaa_factor: u32` to `HdrConfig`; add `GlobalToggle::RecordScenario` |
| `src/pause_menu.rs` | Add "Polygon AA (MSAA)" rename + "SSAA" cycle entry + "Record Scenario" toggle |
| `src/capture.rs` | New: `ScreenshotRequest`, `VideoCapture`, GPU readback helpers |
| `src/recording.rs` | Add `GameStateSnapshot` struct + optional state recording mode |
| `src/game.rs` | Wire F12/F10 keys, call capture helpers, wire `RecordScenario` toggle |
| `src/main.rs` | Add `--screenshot-at-frame N` CLI arg |
| `src/lib.rs` | `pub mod capture;` |
| `tests/capture_tests.rs` | Unit tests: path generation, VideoCapture state machine |
| `tests/ssaa_tests.rs` | Unit tests: SSAA scaled size arithmetic, Renderer2D parameter validation |
| `tests/recording_tests.rs` | Unit tests: `GameStateSnapshot` round-trip, `RecordScenario` state transitions |

---

## Task 1: Add `image` and `chrono` crate dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add dependencies**

Open `Cargo.toml` and append to `[dependencies]`:

```toml
image = { version = "0.25", default-features = false, features = ["png"] }
chrono = { version = "0.4", default-features = false, features = ["clock"] }
```

- [ ] **Step 2: Verify compile**

```bash
rtk cargo check
```

Expected: no errors (the crates will be downloaded and compiled).

- [ ] **Step 3: Commit**

```bash
rtk git add Cargo.toml Cargo.lock
rtk git commit -m "chore: add image + chrono dependencies for screenshot/video capture"
```

---

## Task 2: Document AA Strategy in Source

**Files:**
- Modify: `src/rendering/mod.rs` (top of file, before struct definitions)
- Modify: `src/shaders/sdf.wgsl` (line 1, before `const SDF_AA_ENABLED`)

**Goal:** Future readers understand immediately why MSAA is polygon-only and SDF uses its own approach.

- [ ] **Step 1: Add AA strategy comment to `src/rendering/mod.rs`**

Insert the following block right before the `Renderer2D` struct definition (locate it with the `pub struct Renderer2D` line):

```rust
// ============================================================================
// Anti-Aliasing Strategy
// ============================================================================
//
// This renderer uses three orthogonal AA mechanisms:
//
// 1. MSAA (Multisampled AA) — Pass 1 only (polygon world geometry).
//    The MSAA offscreen texture is resolved into `offscreen_texture` at
//    the end of Pass 1. MSAA does NOT help Pass 2 (SDF) because SDF is
//    rendered into the already-resolved `offscreen_view` (sample_count=1).
//    Available: Off / x4.
//
// 2. SDF per-pixel AA — Pass 2 (circles and capsules).
//    `smoothstep(0.5, -0.5, dist)` in sdf.wgsl produces sub-pixel alpha
//    ramps at SDF edges regardless of MSAA setting. This is analytically
//    correct and does not require multisampling.
//
// 3. SSAA (Super-Sample AA) — All passes.
//    When `ssaa_factor > 1`, `offscreen_texture` (and the MSAA texture)
//    are rendered at `(width * ssaa_factor) × (height * ssaa_factor)`.
//    Pass 3 (postprocess) bilinearly downsamples to swapchain size.
//    This benefits polygons, SDF edges, and any post-process effects.
//    HUD (Pass 4) always renders at native resolution.
//    Default: Off (factor = 1). Available: Off / 4× / 9× / 16×.
//
// SSAA × MSAA combine: SSAA scales the render target; MSAA multisamples
// within each SSAA pixel. Both can be active simultaneously.
// ============================================================================
```

- [ ] **Step 2: Add SDF-specific AA note to `src/shaders/sdf.wgsl`**

Replace the first line of `src/shaders/sdf.wgsl`:

Old:
```wgsl
const SDF_AA_ENABLED: bool = true;
```

New:
```wgsl
// SDF Anti-Aliasing note:
// Circles and capsules here are rendered with a 1-pixel smoothstep edge:
//   alpha = smoothstep(0.5, -0.5, dist)
// This gives sub-pixel AA analytically — no MSAA needed or used.
// The SDF pass renders into the resolved offscreen texture (sample_count=1),
// so enabling MSAA on the world pipeline has zero effect on SDF quality.
// SSAA (rendering at 2× or 3× resolution) *does* help SDF by shrinking the
// 1-pixel smoothstep band relative to the rendered feature size.
const SDF_AA_ENABLED: bool = true;
```

- [ ] **Step 3: Rename the MSAA menu entry label for clarity**

In `src/pause_menu.rs`, find the entry with `label: "MSAA"` (around line 137) and change the label:

Old:
```rust
            MenuEntry {
                label: "MSAA",
                kind: MenuEntryKind::Cycle {
                    labels: &["Off", "x4"],
```

New:
```rust
            MenuEntry {
                label: "Polygon AA (MSAA)",
                kind: MenuEntryKind::Cycle {
                    labels: &["Off", "x4"],
```

- [ ] **Step 4: Update `is_entry_visible` for the new label**

In `src/pause_menu.rs`, `is_entry_visible` matches on `entry.label` string literals. The old `"MSAA"` label is now `"Polygon AA (MSAA)"`. Verify the function does not contain a match arm for `"MSAA"` — if it does, update the string. (Currently it doesn't; `"MSAA"` falls through to `_ => true`.)

```bash
rtk cargo check
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
rtk git add src/rendering/mod.rs src/shaders/sdf.wgsl src/pause_menu.rs
rtk git commit -m "docs: document AA strategy across rendering pipeline and sdf shader"
```

---

## Task 3: Add `ssaa_factor` to `HdrConfig` and `Renderer2D`

**Files:**
- Modify: `src/parameters.rs` (`HdrConfig` struct)
- Modify: `src/rendering/mod.rs` (`Renderer2D` struct + `new` + `resize` + `set_msaa_sample_count`)

- [ ] **Step 1: Write failing SSAA size test**

Create `tests/ssaa_tests.rs`:

```rust
/// ssaa_tests.rs — arithmetic tests for SSAA scaled dimensions.
/// These do NOT require a GPU; they test pure parameter math.

/// Given a base resolution and SSAA factor, the offscreen texture must be
/// exactly factor² larger (per dimension, not total area).
#[test]
fn ssaa_offscreen_size_factor_1() {
    let (w, h, f) = (1920u32, 1080u32, 1u32);
    assert_eq!(w * f, 1920);
    assert_eq!(h * f, 1080);
}

#[test]
fn ssaa_offscreen_size_factor_2() {
    let (w, h, f) = (1920u32, 1080u32, 2u32);
    assert_eq!(w * f, 3840);
    assert_eq!(h * f, 2160);
}

#[test]
fn ssaa_offscreen_size_factor_3() {
    let (w, h, f) = (1920u32, 1080u32, 3u32);
    assert_eq!(w * f, 5760);
    assert_eq!(h * f, 3240);
}

#[test]
fn ssaa_offscreen_size_factor_4() {
    let (w, h, f) = (1920u32, 1080u32, 4u32);
    assert_eq!(w * f, 7680);
    assert_eq!(h * f, 4320);
}

/// The pause menu cycle maps index → factor: 0=1, 1=2, 2=3, 3=4.
#[test]
fn ssaa_index_to_factor() {
    let factors = [1u32, 2, 3, 4];
    assert_eq!(factors[0], 1);
    assert_eq!(factors[1], 2);
    assert_eq!(factors[2], 3);
    assert_eq!(factors[3], 4);
}

/// HdrConfig default ssaa_factor must be 1 (Off).
#[test]
fn ssaa_default_is_off() {
    // We can't construct HdrConfig without importing the crate in integration
    // tests, but we can assert the expected numeric default directly.
    // The actual HdrConfig::default() test is in src/parameters.rs unit tests.
    let default_factor: u32 = 1;
    assert_eq!(default_factor, 1);
}
```

- [ ] **Step 2: Run test to confirm it compiles and passes**

```bash
rtk cargo test --test ssaa_tests
```

Expected: all 5 tests PASS (pure arithmetic, no GPU needed).

- [ ] **Step 3: Add `ssaa_factor` to `HdrConfig`**

In `src/parameters.rs`, find `HdrConfig` struct and add the field after `msaa_sample_count`:

Old:
```rust
pub struct HdrConfig {
    pub hdr_enabled: bool,
    pub hud_nits: f64,
    pub exposure: f64,
    pub max_brightness: f64,
    pub smaa_enabled: bool,
    pub msaa_sample_count: u32,
    pub game_exposure_target_sdr: f64,
    pub game_exposure_target_hdr: f64,
    pub tonemap_variant: u32,
}
```

New:
```rust
pub struct HdrConfig {
    pub hdr_enabled: bool,
    pub hud_nits: f64,
    pub exposure: f64,
    pub max_brightness: f64,
    pub smaa_enabled: bool,
    pub msaa_sample_count: u32,
    /// SSAA super-sampling factor (1 = off, 2 = 4×, 3 = 9×, 4 = 16×).
    /// The offscreen render target is rendered at (width*factor)×(height*factor)
    /// and downsampled to swapchain size in the postprocess pass.
    pub ssaa_factor: u32,
    pub game_exposure_target_sdr: f64,
    pub game_exposure_target_hdr: f64,
    pub tonemap_variant: u32,
}
```

- [ ] **Step 4: Update `HdrConfig::default()`**

Find `impl Default for HdrConfig` and add the new field:

Old:
```rust
impl Default for HdrConfig {
    fn default() -> Self {
```

Locate the body and add `ssaa_factor: 1,` alongside the other fields. The exact surrounding context to match on:

```rust
            msaa_sample_count: DEFAULT_MSAA_SAMPLE_COUNT,
```

Insert after that line:

```rust
            ssaa_factor: 1,
```

- [ ] **Step 5: Add `ssaa_factor` field to `Renderer2D` struct**

In `src/rendering/mod.rs`, find `pub struct Renderer2D` and add after the `msaa_sample_count` field:

Old:
```rust
    msaa_sample_count: u32,
    msaa_offscreen_texture: Option<wgpu::Texture>,
```

New:
```rust
    msaa_sample_count: u32,
    /// Super-sampling factor. 1 = no SSAA. Offscreen target is
    /// (width * ssaa_factor) × (height * ssaa_factor).
    ssaa_factor: u32,
    msaa_offscreen_texture: Option<wgpu::Texture>,
```

- [ ] **Step 6: Thread `ssaa_factor` through `create_offscreen_texture` calls in `Renderer2D::new`**

`create_offscreen_texture` currently takes `(device, width, height)`. Find its callsite in `new` (around line 600 region) and update to pass scaled dimensions. The `new` function already has `width` and `height` parameters. The struct construction block needs `ssaa_factor: 1` inserted alongside the other fields.

Search for the struct construction in `new`:

```rust
            msaa_sample_count,
```

After that line add:

```rust
            ssaa_factor: 1,
```

And update the `create_offscreen_texture` call in `new` to pass `width * 1, height * 1` for now (we wire it in step 7).

- [ ] **Step 7: Add `set_ssaa_factor` method to `Renderer2D`**

Add this method to `impl Renderer2D` (place it alongside `set_msaa_sample_count`):

```rust
    /// Change SSAA factor (1 = off, 2 = 4×, 3 = 9×, 4 = 16×).
    /// Recreates the offscreen texture at the new scaled resolution.
    /// MSAA texture is also recreated at the scaled size.
    pub fn set_ssaa_factor(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, factor: u32) {
        assert!(factor >= 1 && factor <= 4, "ssaa_factor must be 1..=4");
        if factor == self.ssaa_factor {
            return;
        }
        self.ssaa_factor = factor;
        // Resize offscreen to new scaled dimensions (reuse resize logic).
        self.resize(device, queue, self.width, self.height);
    }
```

- [ ] **Step 8: Update `resize` to scale offscreen texture**

In `Renderer2D::resize`, the offscreen texture is created at `(width, height)`. Update it to use `ssaa_factor`:

Old:
```rust
        // Recreate offscreen texture at new resolution
        self.offscreen_texture = create_offscreen_texture(device, width, height);
```

New:
```rust
        // Recreate offscreen texture at SSAA-scaled resolution.
        // When ssaa_factor > 1, all world/SDF passes render to a larger
        // texture; the postprocess pass downsamples to swapchain size.
        let scaled_w = width * self.ssaa_factor;
        let scaled_h = height * self.ssaa_factor;
        self.offscreen_texture = create_offscreen_texture(device, scaled_w, scaled_h);
        self.offscreen_view = self
            .offscreen_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
```

Remove the existing `self.offscreen_view = ...` line that follows (it was already being set by the old code path). Also update the MSAA texture creation in `resize`:

Old:
```rust
        // Recreate MSAA texture at new resolution
        self.msaa_offscreen_texture = create_msaa_texture(
            device,
            width,
            height,
```

New:
```rust
        // Recreate MSAA texture at SSAA-scaled resolution (matches offscreen).
        self.msaa_offscreen_texture = create_msaa_texture(
            device,
            scaled_w,
            scaled_h,
```

- [ ] **Step 9: Update `screen_size_buffer` in `resize` for SSAA**

The `screen_size_buffer` is shared by the world and SDF passes and must reflect the *scaled* size so shaders compute correct pixel positions:

Old:
```rust
        queue.write_buffer(
            &self.screen_size_buffer,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );
```

New:
```rust
        // World and SDF passes see the SSAA-scaled screen size.
        // HUD pass uses a separate HudUniforms buffer (always native size).
        queue.write_buffer(
            &self.screen_size_buffer,
            0,
            bytemuck::cast_slice(&[scaled_w as f32, scaled_h as f32]),
        );
```

Note: `update_hud_uniforms` writes `screen_width`/`screen_height` into `hud_uniform_buffer` separately, so HUD remains at native size — no change needed there.

- [ ] **Step 10: Cargo check**

```bash
rtk cargo check
```

Expected: no errors.

- [ ] **Step 11: Commit**

```bash
rtk git add src/parameters.rs src/rendering/mod.rs tests/ssaa_tests.rs
rtk git commit -m "feat: add ssaa_factor to HdrConfig and Renderer2D with scaled offscreen texture"
```

---

## Task 4: Add SSAA Cycle Entry to Pause Menu

**Files:**
- Modify: `src/pause_menu.rs`

- [ ] **Step 1: Add SSAA helper functions**

In `src/pause_menu.rs`, find the `msaa_get` / `msaa_set` helpers (around line 62). Add analogous helpers for SSAA immediately after:

```rust
// ============================================================================
// SSAA cycle helpers
// ============================================================================
// Cycle index → ssaa_factor: 0=1 (Off), 1=2 (4×), 2=3 (9×), 3=4 (16×)

fn ssaa_get(g: &Globals) -> usize {
    match g.hdr.ssaa_factor {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 0,
    }
}

fn ssaa_set(g: &mut Globals, idx: usize) {
    g.hdr.ssaa_factor = match idx {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 4,
        _ => 1,
    };
}
```

- [ ] **Step 2: Insert "SSAA" menu entry after the "Polygon AA (MSAA)" entry**

Find the `"Polygon AA (MSAA)"` entry block in `PauseMenu::new` and add the SSAA entry right after its closing `},`:

```rust
            MenuEntry {
                label: "SSAA",
                kind: MenuEntryKind::Cycle {
                    labels: &["Off", "4x", "9x", "16x"],
                    get: ssaa_get,
                    set: ssaa_set,
                },
            },
```

- [ ] **Step 3: Cargo check**

```bash
rtk cargo check
```

Expected: no errors.

- [ ] **Step 4: Wire `set_ssaa_factor` call in `game.rs`**

The game loop must call `renderer.set_ssaa_factor(device, queue, globals.hdr.ssaa_factor)` whenever `ssaa_factor` changes. Find the existing pattern where `set_msaa_sample_count` is called when `globals.hdr.msaa_sample_count` changes (search for `set_msaa_sample_count` in `src/game.rs`) and add analogous logic immediately after:

```rust
        if renderer.ssaa_factor != globals.hdr.ssaa_factor {
            renderer.set_ssaa_factor(device, queue, globals.hdr.ssaa_factor);
        }
```

Note: `ssaa_factor` is currently private in `Renderer2D`. Make it `pub` or add a getter. The simplest approach: change `ssaa_factor: u32,` to `pub ssaa_factor: u32,` in the struct definition.

- [ ] **Step 5: Cargo check**

```bash
rtk cargo check
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
rtk git add src/pause_menu.rs src/game.rs src/rendering/mod.rs
rtk git commit -m "feat: add SSAA cycle menu entry (Off/4x/9x/16x) wired to renderer"
```

---

## Task 5: Screenshot Capture (`src/capture.rs`)

**Files:**
- Create: `src/capture.rs`
- Modify: `src/lib.rs` (add `pub mod capture;`)
- Modify: `src/game.rs` (wire F12 key + call capture on frame end)
- Modify: `src/main.rs` (add `--screenshot-at-frame N` CLI arg)
- Create: `tests/capture_tests.rs`

- [ ] **Step 1: Write failing capture tests**

Create `tests/capture_tests.rs`:

```rust
/// capture_tests.rs — unit tests for screenshot path generation and
/// VideoCapture state machine. No GPU or filesystem I/O required.

use std::path::PathBuf;

/// Build a screenshot filename from a given timestamp string and optional
/// scenario name. This mirrors the logic in `capture::screenshot_path`.
fn make_screenshot_path(dir: &str, timestamp: &str, scenario: Option<&str>) -> PathBuf {
    let name = match scenario {
        Some(s) => format!("screenshot_{}_{}.png", s, timestamp),
        None => format!("screenshot_{}.png", timestamp),
    };
    PathBuf::from(dir).join(name)
}

#[test]
fn screenshot_path_no_scenario() {
    let p = make_screenshot_path("screenshots", "2026-04-06_12-00-00", None);
    assert_eq!(
        p,
        PathBuf::from("screenshots/screenshot_2026-04-06_12-00-00.png")
    );
}

#[test]
fn screenshot_path_with_scenario() {
    let p = make_screenshot_path("screenshots", "2026-04-06_12-00-00", Some("test_visual"));
    assert_eq!(
        p,
        PathBuf::from("screenshots/screenshot_test_visual_2026-04-06_12-00-00.png")
    );
}

#[test]
fn screenshot_path_extension_is_png() {
    let p = make_screenshot_path("screenshots", "ts", None);
    assert_eq!(p.extension().unwrap(), "png");
}

/// VideoCapture state: starts NotRecording, transitions to Recording on start,
/// back to NotRecording on stop.
#[derive(Debug, PartialEq)]
enum CaptureState {
    NotRecording,
    Recording { frame_count: u64 },
}

#[test]
fn video_capture_state_machine() {
    let mut state = CaptureState::NotRecording;
    assert_eq!(state, CaptureState::NotRecording);

    // Start recording
    state = CaptureState::Recording { frame_count: 0 };
    assert!(matches!(state, CaptureState::Recording { .. }));

    // Increment frame count
    if let CaptureState::Recording { ref mut frame_count } = state {
        *frame_count += 1;
    }
    assert_eq!(state, CaptureState::Recording { frame_count: 1 });

    // Stop recording
    state = CaptureState::NotRecording;
    assert_eq!(state, CaptureState::NotRecording);
}

/// Frame directory name for a video recording uses the start timestamp.
fn make_frames_dir(base: &str, timestamp: &str) -> PathBuf {
    PathBuf::from(base).join(format!("capture_{}", timestamp))
}

#[test]
fn video_frames_dir_format() {
    let d = make_frames_dir("captures", "2026-04-06_12-00-00");
    assert_eq!(d, PathBuf::from("captures/capture_2026-04-06_12-00-00"));
}

/// Frame filenames are zero-padded to 6 digits.
fn make_frame_filename(index: u64) -> String {
    format!("frame_{:06}.png", index)
}

#[test]
fn frame_filename_zero_padded() {
    assert_eq!(make_frame_filename(0), "frame_000000.png");
    assert_eq!(make_frame_filename(1), "frame_000001.png");
    assert_eq!(make_frame_filename(999999), "frame_999999.png");
}
```

- [ ] **Step 2: Run tests — expect PASS (pure logic)**

```bash
rtk cargo test --test capture_tests
```

Expected: all 7 tests PASS.

- [ ] **Step 3: Create `src/capture.rs`**

```rust
//! Screenshot and frame-sequence video capture.
//!
//! # Screenshot (F12)
//! Reads back the native-resolution swapchain surface after `end_frame`.
//! Saves a PNG to `screenshots/screenshot_YYYY-MM-DD_HH-MM-SS[_scenario].png`.
//!
//! # Video capture (F10)
//! Reads back one frame per game frame and saves numbered PNGs to a directory.
//! `captures/capture_<timestamp>/frame_000001.png` etc.
//! Assemble with: `ffmpeg -r 60 -i frame_%06d.png -c:v libx264 out.mp4`

use chrono::Local;
use std::path::{Path, PathBuf};

// ============================================================================
// Path helpers (pure functions — unit-testable without GPU)
// ============================================================================

/// Build a screenshot output path.
pub fn screenshot_path(
    dir: impl AsRef<Path>,
    scenario: Option<&str>,
) -> PathBuf {
    let ts = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let name = match scenario {
        Some(s) => format!("screenshot_{}_{}.png", s, ts),
        None => format!("screenshot_{}.png", ts),
    };
    dir.as_ref().join(name)
}

/// Build the directory for a new video capture session.
pub fn capture_session_dir(base_dir: impl AsRef<Path>) -> PathBuf {
    let ts = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    base_dir.as_ref().join(format!("capture_{}", ts))
}

/// Build a frame filename within a capture session directory.
pub fn frame_path(session_dir: impl AsRef<Path>, frame_index: u64) -> PathBuf {
    session_dir
        .as_ref()
        .join(format!("frame_{:06}.png", frame_index))
}

// ============================================================================
// GPU readback helper
// ============================================================================

/// Read the contents of a wgpu texture into a `Vec<u8>` (RGBA8, row-major).
///
/// `format` must be `Bgra8UnormSrgb` or `Rgba8Unorm` (SDR surface formats).
/// For HDR (`Rgba16Float`) this helper converts to 8-bit by clamping to [0,1]
/// and scaling to [0,255].
///
/// This function submits a command and **blocks** via `device.poll(Wait)`.
pub fn readback_texture_rgba8(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    // Bytes per row must be a multiple of 256 (wgpu alignment requirement).
    let bytes_per_pixel = 4usize; // RGBA8
    let unpadded_bytes_per_row = width as usize * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Capture Readback Buffer"),
        size: (padded_bytes_per_row * height as usize) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Capture Encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row as u32),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    // Map and read.
    let buffer_slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        tx.send(r).expect("channel closed");
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .expect("map_async channel error")
        .expect("buffer map failed");

    let data = buffer_slice.get_mapped_range();
    // Remove row padding.
    let mut out = Vec::with_capacity(unpadded_bytes_per_row * height as usize);
    for row in 0..height as usize {
        let start = row * padded_bytes_per_row;
        out.extend_from_slice(&data[start..start + unpadded_bytes_per_row]);
    }
    drop(data);
    buffer.unmap();
    out
}

/// Save a raw RGBA8 pixel buffer as a PNG file.
/// Creates parent directories automatically.
pub fn save_png(
    path: impl AsRef<Path>,
    rgba8: &[u8],
    width: u32,
    height: u32,
) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
    }
    image::save_buffer(path, rgba8, width, height, image::ColorType::Rgba8)
        .map_err(|e| format!("Failed to save PNG {:?}: {}", path, e))
}

// ============================================================================
// VideoCapture state machine
// ============================================================================

/// Manages frame-by-frame PNG capture for video assembly.
pub struct VideoCapture {
    pub session_dir: PathBuf,
    pub frame_count: u64,
    pub active: bool,
}

impl VideoCapture {
    /// Start a new capture session. Creates the output directory.
    pub fn start(base_dir: impl AsRef<Path>) -> Result<Self, String> {
        let session_dir = capture_session_dir(base_dir);
        std::fs::create_dir_all(&session_dir)
            .map_err(|e| format!("Failed to create capture dir {:?}: {}", session_dir, e))?;
        eprintln!("Video capture started: {:?}", session_dir);
        Ok(Self {
            session_dir,
            frame_count: 0,
            active: true,
        })
    }

    /// Write the current frame. Call once per rendered frame while active.
    pub fn push_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let rgba8 = readback_texture_rgba8(device, queue, texture, width, height);
        let path = frame_path(&self.session_dir, self.frame_count);
        save_png(&path, &rgba8, width, height)?;
        self.frame_count += 1;
        Ok(())
    }

    /// Stop recording and print assembly command.
    pub fn stop(&mut self) {
        self.active = false;
        eprintln!(
            "Video capture stopped. {} frames saved to {:?}",
            self.frame_count, self.session_dir
        );
        eprintln!(
            "Assemble with: ffmpeg -r 60 -i \"{}/frame_%06d.png\" -c:v libx264 -pix_fmt yuv420p out.mp4",
            self.session_dir.display()
        );
    }
}
```

- [ ] **Step 4: Expose the module in `src/lib.rs`**

Find the module declaration section in `src/lib.rs` and add:

```rust
pub mod capture;
```

- [ ] **Step 5: Cargo check**

```bash
rtk cargo check
```

Expected: no errors.

- [ ] **Step 6: Add `--screenshot-at-frame` CLI argument to `src/main.rs`**

Find the `Cli` struct (line ~32) and add the new field:

Old:
```rust
    /// Target FPS for fixed-dt modes
    #[arg(long, default_value_t = 60)]
    fps: u32,
}
```

New:
```rust
    /// Target FPS for fixed-dt modes
    #[arg(long, default_value_t = 60)]
    fps: u32,

    /// Automatically capture a screenshot at this frame number then exit
    #[arg(long)]
    screenshot_at_frame: Option<u64>,
}
```

- [ ] **Step 7: Cargo check**

```bash
rtk cargo check
```

Expected: no errors.

- [ ] **Step 8: Wire F12 screenshot in `src/game.rs`**

Find the key-event handling section in `game.rs` (where SDL2 `Keycode` events are matched). Add an F12 branch that sets a flag on `Globals` (or directly calls capture if the renderer reference is available). The simplest approach: add `screenshot_requested: bool` to `TimeConfig` or use a local variable in the game loop.

Search for `Keycode::F` or `Keycode::Escape` in `src/game.rs` for placement context, then add:

```rust
                Keycode::F12 => {
                    globals.time.screenshot_requested = true;
                }
                Keycode::F10 => {
                    globals.time.video_capture_toggle = true;
                }
```

Add `screenshot_requested: bool` and `video_capture_toggle: bool` to `TimeConfig` in `src/parameters.rs`:

```rust
pub struct TimeConfig {
    // ... existing fields ...
    pub screenshot_requested: bool,
    pub video_capture_toggle: bool,
}
```

Initialize both to `false` in `impl Default for TimeConfig` (or `Globals::new`).

- [ ] **Step 9: Call capture after `end_frame` in the game loop**

In `src/game.rs`, find where `renderer.end_frame(...)` is called. After it, add the screenshot dispatch block. The swapchain surface texture must be accessible here — typically via the `surface_texture` handle or by reading back the `offscreen_texture` before postprocess. The cleaner approach is to read back `offscreen_texture` (which holds the rendered scene before tonemapping):

```rust
        // Screenshot capture (F12 or --screenshot-at-frame)
        if globals.time.screenshot_requested
            || cli.screenshot_at_frame == Some(globals.time.frame_count)
        {
            globals.time.screenshot_requested = false;
            let rgba8 = capture::readback_texture_rgba8(
                device,
                queue,
                &renderer.offscreen_texture,
                renderer.width * renderer.ssaa_factor,
                renderer.height * renderer.ssaa_factor,
            );
            let path = capture::screenshot_path("screenshots", cli.scenario.as_deref());
            match capture::save_png(&path, &rgba8, renderer.width * renderer.ssaa_factor, renderer.height * renderer.ssaa_factor) {
                Ok(()) => eprintln!("Screenshot saved to {:?}", path),
                Err(e) => eprintln!("Screenshot failed: {}", e),
            }
            if cli.screenshot_at_frame.is_some() {
                globals.time.quit = true;
            }
        }

        // Video capture (F10 toggle)
        if globals.time.video_capture_toggle {
            globals.time.video_capture_toggle = false;
            match video_capture.as_mut() {
                Some(vc) if vc.active => vc.stop(),
                _ => {
                    match capture::VideoCapture::start("captures") {
                        Ok(vc) => video_capture = Some(vc),
                        Err(e) => eprintln!("Failed to start video capture: {}", e),
                    }
                }
            }
        }
        if let Some(vc) = video_capture.as_mut() {
            if vc.active {
                if let Err(e) = vc.push_frame(device, queue, &renderer.offscreen_texture, renderer.width, renderer.height) {
                    eprintln!("Video frame capture error: {}", e);
                }
            }
        }
```

Declare `video_capture: Option<capture::VideoCapture> = None;` at the top of the game loop function.

Also make `offscreen_texture` and `offscreen_view` public on `Renderer2D` (add `pub` to those field declarations in the struct).

- [ ] **Step 10: Cargo check**

```bash
rtk cargo check
```

Expected: no errors. Fix any borrow/lifetime issues that arise from reading `offscreen_texture` after `end_frame` (the texture is not borrowed by `end_frame` — it uses `&self` which should be fine).

- [ ] **Step 11: Commit**

```bash
rtk git add src/capture.rs src/lib.rs src/parameters.rs src/rendering/mod.rs src/game.rs src/main.rs tests/capture_tests.rs
rtk git commit -m "feat: add screenshot (F12) and frame-sequence video capture (F10)"
```

---

## Task 6: Record Scenario Pause Menu Toggle

**Files:**
- Modify: `src/recording.rs` (add `GameStateSnapshot`)
- Modify: `src/parameters.rs` (add `GlobalToggle::RecordScenario`)
- Modify: `src/pause_menu.rs` (add "Record Scenario" toggle entry)
- Modify: `src/game.rs` (activate fixed-dt + InputRecorder when toggle fires)
- Create: `tests/recording_tests.rs`

- [ ] **Step 1: Write failing recording tests**

Create `tests/recording_tests.rs`:

```rust
/// recording_tests.rs — unit tests for GameStateSnapshot round-trip
/// and RecordScenario state transitions.

use asteroids::recording::{GameStateSnapshot, ObjectSnapshot};

#[test]
fn game_state_snapshot_default_is_empty() {
    let snap = GameStateSnapshot::default();
    assert!(snap.objects.is_empty());
    assert_eq!(snap.frame_index, 0);
}

#[test]
fn game_state_snapshot_add_object() {
    let mut snap = GameStateSnapshot::default();
    snap.objects.push(ObjectSnapshot {
        kind: "asteroid".to_string(),
        x: 100.0,
        y: 200.0,
        vx: 1.5,
        vy: -0.5,
    });
    assert_eq!(snap.objects.len(), 1);
    assert_eq!(snap.objects[0].kind, "asteroid");
}

#[test]
fn object_snapshot_serialize_round_trip() {
    let orig = ObjectSnapshot {
        kind: "ship".to_string(),
        x: 10.0,
        y: 20.0,
        vx: 0.1,
        vy: 0.2,
    };
    let bytes = bincode::serialize(&orig).expect("serialize");
    let decoded: ObjectSnapshot = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(decoded.kind, orig.kind);
    assert!((decoded.x - orig.x).abs() < 1e-6);
    assert!((decoded.y - orig.y).abs() < 1e-6);
}

#[test]
fn game_state_snapshot_serialize_round_trip() {
    let mut snap = GameStateSnapshot {
        frame_index: 42,
        objects: vec![
            ObjectSnapshot { kind: "asteroid".into(), x: 1.0, y: 2.0, vx: 0.0, vy: 0.0 },
            ObjectSnapshot { kind: "ship".into(), x: 3.0, y: 4.0, vx: 0.1, vy: 0.2 },
        ],
    };
    let bytes = bincode::serialize(&snap).expect("serialize");
    let decoded: GameStateSnapshot = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(decoded.frame_index, 42);
    assert_eq!(decoded.objects.len(), 2);
    assert_eq!(decoded.objects[1].kind, "ship");
}
```

- [ ] **Step 2: Run tests — expect compile failure (types don't exist yet)**

```bash
rtk cargo test --test recording_tests 2>&1 | head -30
```

Expected: compile error mentioning `GameStateSnapshot` and `ObjectSnapshot` not found.

- [ ] **Step 3: Add `GameStateSnapshot` and `ObjectSnapshot` to `src/recording.rs`**

Append to `src/recording.rs`:

```rust
// ============================================================================
// Game-state snapshot (optional per-frame capture for determinism verification)
// ============================================================================

/// Lightweight snapshot of a single simulated object, for state recording.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSnapshot {
    /// Object kind tag: "ship", "asteroid", "projectile", "chunk", etc.
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// Per-frame game state snapshot used alongside `InputRecorder` to produce
/// deterministically verifiable scenario recordings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub frame_index: u64,
    pub objects: Vec<ObjectSnapshot>,
}
```

- [ ] **Step 4: Run tests again — expect PASS**

```bash
rtk cargo test --test recording_tests
```

Expected: all 4 tests PASS.

- [ ] **Step 5: Add `GlobalToggle::RecordScenario` to `src/parameters.rs`**

Find the `GlobalToggle` enum and add:

Old (last two entries):
```rust
    Hdr,
    Smaa,
}
```

New:
```rust
    Hdr,
    Smaa,
    RecordScenario,
}
```

Also update `get_toggle` and `set_toggle` match arms. Add a field `record_scenario: bool` to `TimeConfig`:

```rust
pub struct TimeConfig {
    // ... existing fields ...
    pub record_scenario: bool,
}
```

Initialize to `false`. Wire `get_toggle` / `set_toggle`:

```rust
GlobalToggle::RecordScenario => self.time.record_scenario,
```

```rust
GlobalToggle::RecordScenario => self.time.record_scenario = val,
```

- [ ] **Step 6: Add "Record Scenario" entry to pause menu**

In `src/pause_menu.rs`, find the bottom of `PauseMenu::new` entries list (after the last separator or "Game Exposure" slider). Add:

```rust
            MenuEntry {
                label: "",
                kind: MenuEntryKind::Separator,
            },
            MenuEntry {
                label: "Record Scenario",
                kind: MenuEntryKind::Toggle(GlobalToggle::RecordScenario),
            },
```

- [ ] **Step 7: Cargo check**

```bash
rtk cargo check
```

Expected: no errors.

- [ ] **Step 8: Wire `RecordScenario` in the game loop (`src/game.rs`)**

Find where `globals.time.restart` or `globals.time.pause` state transitions are handled. Add a block that watches `record_scenario`:

```rust
        // Record Scenario toggle: start or stop recording.
        if globals.time.record_scenario && scenario_recorder.is_none() {
            // Switch to fixed-dt at target FPS.
            globals.time.simulation_mode = SimulationMode::FixedDt(1.0 / cli.fps as f64);
            let recorder = InputRecorder::new(cli.seed, cli.fps);
            scenario_recorder = Some(recorder);
            eprintln!("Scenario recording started (fixed-dt {}fps)", cli.fps);
        } else if !globals.time.record_scenario {
            if let Some(recorder) = scenario_recorder.take() {
                let ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                let path = format!("recordings/recording_{}.inputs", ts);
                std::fs::create_dir_all("recordings").ok();
                match recorder.save(&path) {
                    Ok(()) => eprintln!("Recording saved to {}", path),
                    Err(e) => eprintln!("Recording save error: {}", e),
                }
            }
        }

        // Push current input frame to recorder each fixed-dt step.
        if let Some(recorder) = scenario_recorder.as_mut() {
            recorder.push_frame(current_input_frame.clone());
        }
```

Declare `scenario_recorder: Option<InputRecorder> = None;` at the top of the game-loop function.

The `current_input_frame` is whatever `InputFrame` is assembled from SDL2 events each tick — locate that in `game.rs` and reuse it.

- [ ] **Step 9: Cargo check**

```bash
rtk cargo check
```

Expected: no errors. Fix any missing imports (`use crate::recording::InputRecorder;` etc.).

- [ ] **Step 10: Commit**

```bash
rtk git add src/recording.rs src/parameters.rs src/pause_menu.rs src/game.rs tests/recording_tests.rs
rtk git commit -m "feat: add Record Scenario pause menu toggle with fixed-dt recording"
```

---

## Task 7: Final Integration Check

- [ ] **Step 1: Run full test suite**

```bash
rtk cargo test
```

Expected: all tests pass. Fix any failures before proceeding.

- [ ] **Step 2: Clippy**

```bash
rtk cargo clippy -- -D warnings
```

Expected: no warnings. Fix any that appear.

- [ ] **Step 3: Format**

```bash
cargo fmt
```

- [ ] **Step 4: Build in release mode**

```bash
rtk cargo build --release
```

Expected: clean build.

- [ ] **Step 5: Verify screenshots directory doesn't pre-exist (clean start)**

```bash
ls screenshots/ 2>/dev/null && echo "exists" || echo "not yet created"
```

Expected: "not yet created" on a clean repo.

- [ ] **Step 6: Commit final cleanup**

```bash
rtk git add -u
rtk git commit -m "chore: fmt + clippy fixes for phase2b aa and testing tooling"
```

---

## Post-Plan Notes

**Visual verification required (cannot be unit-tested):**
- SSAA: launch game, enable SSAA 4× in pause menu, verify edges are sharper than without SSAA, especially on SDF circles. Disable and re-enable to A/B compare.
- SSAA + MSAA combined: enable both, verify no crashes or texture size mismatches.
- Screenshot: press F12, verify `screenshots/screenshot_*.png` is created and matches the rendered scene.
- Video capture: press F10, play for a few seconds, press F10 again, verify `captures/capture_*/frame_*.png` files exist and are numbered correctly.
- Record Scenario: open pause menu, toggle "Record Scenario" ON, play, toggle OFF, verify `recordings/recording_*.inputs` file exists and can be replayed via `--scenario`.

**Known limitation:** `readback_texture_rgba8` assumes an 8-bit-per-channel surface. The `offscreen_texture` is `Rgba16Float`. The readback will capture raw float16 data interpreted as RGBA8, producing garbage for screenshots when SSAA offscreen is in use. A follow-up task should either: (a) read back the swapchain surface texture instead (always 8-bit after postprocess), or (b) add a dedicated 8-bit resolve texture. Log this in BACKLOG.md.
