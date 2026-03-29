# Gamepad + World-Space Controls — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace rotate+thrust controls with world-space twin-stick movement, add SDL2 gamepad support, and rework teleport into cone-based aim-directed targeting.

**Architecture:** World-space WASD/left-stick movement decoupled from aim. Mouse/right-stick sets `ship.orientation` (= aim direction). Visual smoothing on ship rotation (rendering only). Cone-based teleport selects biggest asteroid in a 15° cone along aim. `ShipControlConfig` and all rotation/strafe input removed.

**Tech Stack:** Rust, SDL2 (GameController API), wgpu, existing `input.rs`/`main.rs`/`parameters.rs`

**Design spec:** `docs/superpowers/specs/2026-03-28-gamepad-world-space-controls-design.md`

---

## File Structure

| File | Role |
|------|------|
| `src/parameters.rs` | Add `GamepadConfig`, `TeleportConfig`. Remove `ShipControlConfig`. Add constants. |
| `src/input.rs` | Rewrite: world-space thrust (keyboard + stick), dead zone processing, cone teleport. Remove rotation/strafe functions. |
| `src/main.rs` | SDL2 GameController init, axis/button event handling, new input dispatch. |
| `src/game.rs` | Add `GamepadState` to `GameState`. Wire visual smoothing update. |
| `src/rendering/world.rs` | Ship renders with `visual_aim_angle` instead of `ship.orientation`. |
| `src/camera.rs` | No changes — already uses `ship.orientation` for lookahead, which now = aim direction (correct). |
| `src/pause_menu.rs` | No changes — no ShipControlConfig toggles exist in the pause menu. |

---

## Task 1: Remove `ShipControlConfig` and Old Input Functions

**Goal:** Clean out the rotation/strafing/impulse control system. After this task, the game won't have working keyboard movement (restored in Task 2), but it will compile and run.

**Files:**
- Modify: `src/parameters.rs` — remove `ShipControlConfig` struct and field from `Globals`
- Modify: `src/input.rs` — remove `handle_left`, `handle_right`, `strafe_left`, `strafe_right`, `boost_forward`
- Modify: `src/main.rs` — remove all input dispatch that references removed functions
- Modify: `src/game.rs` — remove any references to `ship_control` if present

### Step 1.1: Remove `ShipControlConfig` from `parameters.rs`

- [ ] Delete the `ShipControlConfig` struct (lines 441–446):
```rust
// DELETE this entire struct
pub struct ShipControlConfig {
    pub ship_direct_pos: bool,
    pub ship_direct_rotat: bool,
    pub ship_impulse_pos: bool,
    pub ship_impulse_rotat: bool,
}
```

- [ ] Remove the `ship_control` field from the `Globals` struct (line 515):
```rust
// DELETE this line from Globals struct
pub ship_control: ShipControlConfig,
```

- [ ] Remove the `ShipControlConfig` initialization from `Globals::new()`. Search for `ship_control:` in the `Globals::new()` method and delete the block:
```rust
// DELETE this block from Globals::new()
ship_control: ShipControlConfig {
    ship_direct_pos: false,
    ship_direct_rotat: false,
    ship_impulse_pos: true,
    ship_impulse_rotat: true,
},
```

### Step 1.2: Remove old input functions from `input.rs`

- [ ] Delete these functions entirely from `input.rs`:
  - `handle_left` (lines 88–99)
  - `handle_right` (lines 103–115)
  - `strafe_left` (lines 118–121)
  - `strafe_right` (lines 124–127)
  - `boost_forward` (lines 44–54)

- [ ] Remove unused imports from `input.rs`. After deletion, these imports from `crate::game` are no longer needed: `apply_torque`, `boost_torque`, `rotate_entity`, `turn_entity`, `boost_entity`. Keep `accelerate_entity` and `GameState`.

### Step 1.3: Remove old input dispatch from `main.rs`

- [ ] Remove the entire W-key impulse/continuous block (lines 231–239):
```rust
// DELETE this entire block
let w_pressed = keyboard.is_scancode_pressed(Scancode::W);
if globals.ship_control.ship_impulse_pos {
    if w_pressed && !prev_w_pressed {
        input::boost_forward(&mut state, &globals);
    }
} else if w_pressed {
    input::acceleration(&mut state, &globals);
}
prev_w_pressed = w_pressed;
```

- [ ] Remove the `prev_w_pressed` variable declaration (line 89) and its initial value.

- [ ] Remove A/D rotation and Q/E strafe blocks (lines 241–255):
```rust
// DELETE all of these
if keyboard.is_scancode_pressed(Scancode::A) {
    input::handle_left(&mut state.ship, &globals);
}
if keyboard.is_scancode_pressed(Scancode::D) {
    input::handle_right(&mut state.ship, &globals);
}
if keyboard.is_scancode_pressed(Scancode::Q) {
    input::strafe_left(&mut state.ship);
}
if keyboard.is_scancode_pressed(Scancode::E) {
    input::strafe_right(&mut state.ship);
}
```

- [ ] Remove Space = fire (lines 258–260). Fire will be re-added as left-click only (already works via mouse_state.left()) and gamepad trigger in later tasks:
```rust
// DELETE this
if keyboard.is_scancode_pressed(Scancode::Space) {
    input::fire(&mut state, &mut globals);
}
```

- [ ] Remove `P` from the pause keycode match. Change:
```rust
Event::KeyDown {
    keycode: Some(Keycode::P) | Some(Keycode::Escape),
    repeat: false,
    ..
} => globals.time.pause = !globals.time.pause,
```
to:
```rust
Event::KeyDown {
    keycode: Some(Keycode::Escape),
    repeat: false,
    ..
} => globals.time.pause = !globals.time.pause,
```

### Step 1.4: Verify compilation

- [ ] Run `cargo check`. Fix any remaining references to `ship_control`, `handle_left`, `handle_right`, `strafe_left`, `strafe_right`, `boost_forward`, or `prev_w_pressed`.

- [ ] Run `cargo clippy` — fix any warnings.

### Step 1.5: Commit

```bash
git add src/parameters.rs src/input.rs src/main.rs
git commit -m "refactor(input): remove ShipControlConfig and rotation/strafe controls"
```

---

## Task 2: World-Space Keyboard Movement

**Goal:** WASD moves the ship in world-space cardinal directions, decoupled from aim. Mouse click still accelerates in aim direction (will be changed to fire-only in Task 3).

**Files:**
- Modify: `src/input.rs` — add `world_space_thrust_keyboard`
- Modify: `src/main.rs` — wire WASD to new function

### Step 2.1: Add `world_space_thrust_keyboard` to `input.rs`

- [ ] Add this function to `input.rs` (after the `aim_at_mouse` function):

```rust
/// World-space keyboard thrust: WASD = cardinal directions, diagonal normalized.
/// Movement is decoupled from aim (ship.orientation).
pub fn world_space_thrust_keyboard(state: &mut GameState, globals: &Globals, keys_pressed: [bool; 4]) {
    let [w, a, s, d] = keys_pressed;
    let mut dir = Vec2::new(0.0, 0.0);
    if w { dir.y += 1.0; } // Y-up in physics space
    if s { dir.y -= 1.0; }
    if a { dir.x -= 1.0; }
    if d { dir.x += 1.0; }
    let mag = (dir.x * dir.x + dir.y * dir.y).sqrt();
    if mag > 0.0 {
        let normalized = Vec2::new(dir.x / mag, dir.y / mag);
        accelerate_entity(
            &mut state.ship,
            scale_vec(normalized, SHIP_MAX_ACCEL),
            globals,
        );
        // Engine fire while thrusting
        if state.ship.health > 0.0 && globals.visual.smoke_enabled {
            let fire = spawn_fire(&state.ship, &mut state.rng);
            state.smoke.push(fire);
        }
    }
}
```

Note: `spawn_fire` uses `ship.orientation + PI` for the fire ejection direction. Since `orientation` is now aim direction, fire will eject opposite to where the player is aiming. This is acceptable — the thruster visually fires from the back of the ship (which faces the aim direction). If this looks wrong in testing, it can be adjusted to eject opposite to movement direction instead.

### Step 2.2: Wire WASD in `main.rs`

- [ ] In the `if !globals.time.pause` block, after the mouse aim line, replace the deleted keyboard input section with:

```rust
// WASD world-space movement
let keyboard = event_pump.keyboard_state();
let keys_pressed = [
    keyboard.is_scancode_pressed(Scancode::W),
    keyboard.is_scancode_pressed(Scancode::A),
    keyboard.is_scancode_pressed(Scancode::S),
    keyboard.is_scancode_pressed(Scancode::D),
];
input::world_space_thrust_keyboard(&mut state, &globals, keys_pressed);
```

- [ ] Keep the `use sdl2::keyboard::Scancode;` import at the top of `main.rs` (already present).

### Step 2.3: Remove `acceleration` from mouse click

- [ ] Change the mouse-click block from acceleration to fire. Replace:
```rust
if mouse_state.left() {
    input::acceleration(&mut state, &globals);
}
```
with:
```rust
if mouse_state.left() {
    input::fire(&mut state, &mut globals);
}
```

Now left-click = fire (was Space before), and WASD = move (was W=forward, A/D=rotate).

### Step 2.4: Clean up `acceleration` function

- [ ] The `acceleration` function in `input.rs` is no longer called. Delete it:
```rust
// DELETE this entire function
pub fn acceleration(state: &mut GameState, globals: &Globals) { ... }
```

### Step 2.5: Verify and commit

- [ ] Run `cargo check && cargo clippy`. Fix any issues.
- [ ] Run the game — verify WASD moves in world-space directions, left-click fires, mouse aims.

```bash
git add src/input.rs src/main.rs
git commit -m "feat(input): world-space WASD movement + left-click fires"
```

---

## Task 3: Config Structs and `GamepadState`

**Goal:** Add `GamepadConfig`, `TeleportConfig` to `parameters.rs` and `GamepadState` to `game.rs`. No behavioral changes yet — just the data structures.

**Files:**
- Modify: `src/parameters.rs` — add config structs and constants
- Modify: `src/game.rs` — add `GamepadState` to `GameState`

### Step 3.1: Add constants to `parameters.rs`

- [ ] Add these constants near the other ship/gameplay constants:

```rust
// ============================================================================
// Gamepad constants
// ============================================================================

/// Inner dead zone threshold — stick deflection below this is treated as zero
pub const STICK_DEAD_ZONE_INNER: f64 = 0.15;
/// Outer dead zone threshold — stick deflection above this is treated as 1.0
pub const STICK_DEAD_ZONE_OUTER: f64 = 0.90;
/// Seconds of idle before drift recalibration starts
pub const DRIFT_RECENTER_DELAY: f64 = 2.0;
/// Lerp speed for drift compensation (per second)
pub const DRIFT_RECENTER_SPEED: f64 = 0.5;
/// Visual smoothing for ship rotation (0.0 = instant, higher = more lag)
pub const AIM_VISUAL_SMOOTHING: f64 = 8.0;

// ============================================================================
// Teleport constants
// ============================================================================

/// Half-angle of the teleport targeting cone (degrees)
pub const TELEPORT_CONE_HALF_ANGLE_DEG: f64 = 7.5;
```

### Step 3.2: Add `GamepadState` to `game.rs`

- [ ] Add this struct to `game.rs` (near the top, before `GameState`):

```rust
use crate::math::Vec2;

/// Runtime state for gamepad input processing.
pub struct GamepadState {
    /// Whether a gamepad is currently connected
    pub connected: bool,
    /// Drift compensation offset for left stick
    pub left_center_offset: Vec2,
    /// Drift compensation offset for right stick
    pub right_center_offset: Vec2,
    /// Timestamp when sticks last went idle (for drift recalibration)
    pub last_idle_time: f64,
    /// Smoothed visual aim angle for ship rendering (lags behind true orientation)
    pub visual_aim_angle: f64,
    /// Raw left stick axes after normalization [-1.0, 1.0], before dead zone
    pub left_stick_raw: Vec2,
    /// Raw right stick axes after normalization [-1.0, 1.0], before dead zone
    pub right_stick_raw: Vec2,
    /// Whether any gamepad button is currently pressed (for drift detection)
    pub any_button_pressed: bool,
    /// Whether left trigger is currently past the activation threshold (for edge detection)
    pub left_trigger_pressed: bool,
}

impl GamepadState {
    pub fn new() -> Self {
        Self {
            connected: false,
            left_center_offset: Vec2::ZERO,
            right_center_offset: Vec2::ZERO,
            last_idle_time: 0.0,
            visual_aim_angle: std::f64::consts::PI / 2.0, // Match ship's initial orientation
            left_stick_raw: Vec2::ZERO,
            right_stick_raw: Vec2::ZERO,
            any_button_pressed: false,
            left_trigger_pressed: false,
        }
    }
}
```

- [ ] Add `pub gamepad: GamepadState` field to the `GameState` struct.

- [ ] Initialize it in `GameState::new`: `gamepad: GamepadState::new(),`

### Step 3.3: Verify and commit

- [ ] Run `cargo check && cargo clippy`.

```bash
git add src/parameters.rs src/game.rs
git commit -m "feat(input): add GamepadState and gamepad/teleport constants"
```

---

## Task 4: Dead Zone Processing and Stick Input Functions

**Goal:** Add stick dead zone processing and world-space stick thrust to `input.rs`. Pure functions, no SDL2 dependency yet.

**Files:**
- Modify: `src/input.rs` — add dead zone + stick thrust functions

### Step 4.1: Add dead zone processing

- [ ] Add to `input.rs`:

```rust
/// Process a single stick axis: subtract drift offset, apply inner/outer dead zone, remap to [0, 1].
pub fn process_stick_axis(raw: f64, center_offset: f64) -> f64 {
    let adjusted = raw - center_offset;
    let abs_val = adjusted.abs();
    if abs_val < STICK_DEAD_ZONE_INNER {
        return 0.0;
    }
    if abs_val > STICK_DEAD_ZONE_OUTER {
        return adjusted.signum();
    }
    let remapped = (abs_val - STICK_DEAD_ZONE_INNER) / (STICK_DEAD_ZONE_OUTER - STICK_DEAD_ZONE_INNER);
    remapped * adjusted.signum()
}
```

- [ ] Add the import for the new constants at the top of `input.rs`:
```rust
use crate::parameters::{STICK_DEAD_ZONE_INNER, STICK_DEAD_ZONE_OUTER};
```
(or use the existing wildcard `use crate::parameters::*;` which is already there)

### Step 4.2: Add world-space stick thrust

- [ ] Add to `input.rs`:

```rust
/// World-space gamepad stick thrust: analog magnitude proportional to stick deflection.
pub fn world_space_thrust_stick(state: &mut GameState, globals: &Globals, stick: Vec2) {
    let mag = (stick.x * stick.x + stick.y * stick.y).sqrt();
    if mag > 0.0 {
        let clamped_mag = mag.min(1.0);
        let direction = Vec2::new(stick.x / mag, stick.y / mag);
        accelerate_entity(
            &mut state.ship,
            scale_vec(direction, SHIP_MAX_ACCEL * clamped_mag),
            globals,
        );
        // Engine fire while thrusting via stick
        if state.ship.health > 0.0 && globals.visual.smoke_enabled {
            let fire = spawn_fire(&state.ship, &mut state.rng);
            state.smoke.push(fire);
        }
    }
}
```

### Step 4.3: Add aim-from-stick function

- [ ] Add to `input.rs`:

```rust
/// Set ship aim direction from right stick. Only updates when stick magnitude exceeds dead zone
/// (keeps last aim direction when stick is released / in dead zone).
pub fn aim_from_stick(ship: &mut Entity, stick: Vec2) {
    let mag = (stick.x * stick.x + stick.y * stick.y).sqrt();
    if mag > 0.0 {
        ship.orientation = stick.y.atan2(stick.x);
    }
}
```

### Step 4.4: Add drift compensation update

- [ ] Add to `input.rs`:

```rust
/// Update adaptive drift compensation for a stick.
/// When no buttons are pressed and stick is stable for DRIFT_RECENTER_DELAY seconds,
/// slowly lerp the center offset toward the current raw reading.
pub fn update_drift_compensation(
    center_offset: &mut Vec2,
    raw: Vec2,
    any_button_pressed: bool,
    last_idle_time: &mut f64,
    current_time: f64,
    dt: f64,
) {
    if any_button_pressed || raw.x.abs() > 0.5 || raw.y.abs() > 0.5 {
        // Stick is actively in use — reset idle timer
        *last_idle_time = current_time;
        return;
    }
    let idle_duration = current_time - *last_idle_time;
    if idle_duration >= DRIFT_RECENTER_DELAY {
        let lerp_factor = (DRIFT_RECENTER_SPEED * dt).min(1.0);
        center_offset.x += (raw.x - center_offset.x) * lerp_factor;
        center_offset.y += (raw.y - center_offset.y) * lerp_factor;
    }
}
```

- [ ] Add imports for the drift constants (already covered by `use crate::parameters::*;`).

### Step 4.5: Verify and commit

- [ ] Run `cargo check && cargo clippy`.

```bash
git add src/input.rs
git commit -m "feat(input): dead zone processing, stick thrust, aim, and drift compensation"
```

---

## Task 5: SDL2 GameController Integration in `main.rs`

**Goal:** Initialize SDL2 GameController subsystem, handle connect/disconnect events, poll axis/button events, and route to input functions.

**Files:**
- Modify: `src/main.rs` — SDL2 controller init + event handling
- Modify: `src/game.rs` — process stick inputs each frame

### Step 5.1: Initialize GameController subsystem

- [ ] In `main.rs`, after the SDL2 video init, add controller subsystem init:

```rust
let sdl_context = sdl2::init().expect("Failed to init SDL2");
let video_subsystem = sdl_context.video().expect("Failed to init video");
let game_controller_subsystem = sdl_context.game_controller().expect("Failed to init game controller");
```

- [ ] Add a variable to hold the active controller (after `is_fullscreen`):

```rust
let mut active_controller: Option<sdl2::controller::GameController> = None;
```

- [ ] Open any controller that's already connected at startup (after the event pump creation):

```rust
// Open first available controller at startup
if let Ok(count) = game_controller_subsystem.num_joysticks() {
    for i in 0..count {
        if game_controller_subsystem.is_game_controller(i) {
            match game_controller_subsystem.open(i) {
                Ok(controller) => {
                    println!("Controller connected: {}", controller.name());
                    state.gamepad.connected = true;
                    active_controller = Some(controller);
                    break;
                }
                Err(e) => eprintln!("Failed to open controller {}: {}", i, e),
            }
        }
    }
}
```

### Step 5.2: Handle controller connect/disconnect events

- [ ] In the event poll loop (`for event in event_pump.poll_iter()`), add these match arms before the `_ => {}` fallthrough:

```rust
Event::ControllerDeviceAdded { which, .. } => {
    if active_controller.is_none() {
        match game_controller_subsystem.open(which) {
            Ok(controller) => {
                println!("Controller connected: {}", controller.name());
                state.gamepad.connected = true;
                state.gamepad.left_center_offset = Vec2::ZERO;
                state.gamepad.right_center_offset = Vec2::ZERO;
                active_controller = Some(controller);
            }
            Err(e) => eprintln!("Failed to open controller: {}", e),
        }
    }
}
Event::ControllerDeviceRemoved { which, .. } => {
    if let Some(ref c) = active_controller {
        if c.instance_id() == which {
            println!("Controller disconnected");
            state.gamepad.connected = false;
            state.gamepad.left_stick_raw = Vec2::ZERO;
            state.gamepad.right_stick_raw = Vec2::ZERO;
            active_controller = None;
        }
    }
}
```

### Step 5.3: Handle axis motion events

- [ ] Add axis handling in the event poll loop:

```rust
Event::ControllerAxisMotion { axis, value, .. } => {
    // SDL2 axes are i16 [-32768, 32767] → normalize to f64 [-1.0, 1.0]
    let normalized = value as f64 / 32767.0;
    use sdl2::controller::Axis;
    match axis {
        Axis::LeftX  => state.gamepad.left_stick_raw.x = normalized,
        Axis::LeftY  => state.gamepad.left_stick_raw.y = -normalized, // SDL Y-down → Y-up
        Axis::RightX => state.gamepad.right_stick_raw.x = normalized,
        Axis::RightY => state.gamepad.right_stick_raw.y = -normalized, // SDL Y-down → Y-up
        _ => {} // Triggers handled as buttons below
    }
}
```

### Step 5.4: Handle button events

- [ ] Add button handling in the event poll loop:

```rust
Event::ControllerButtonDown { button, .. } => {
    use sdl2::controller::Button;
    match button {
        Button::A => state.gamepad.any_button_pressed = true,
        Button::B => {
            state.gamepad.any_button_pressed = true;
            // Teleport on B press — signature changes in Task 7 (cone-based).
            // For now, skip wiring teleport here. Task 7 will add the final call.
        }
        Button::Start => globals.time.pause = !globals.time.pause,
        _ => state.gamepad.any_button_pressed = true,
    }
}
Event::ControllerButtonUp { button, .. } => {
    // We track "any pressed" loosely — set to false on all ups.
    // This is imperfect (multiple buttons) but good enough for drift detection.
    state.gamepad.any_button_pressed = false;
}
```

Note: teleport signature will change in Task 7 — for now, wire the old `teleport` or leave as a TODO comment.

### Step 5.5: Process gamepad sticks each frame

- [ ] In the `if !globals.time.pause` block in `main.rs`, after the WASD input, add gamepad processing:

```rust
// Gamepad input processing
if state.gamepad.connected {
    // Process left stick (movement)
    let left_x = input::process_stick_axis(
        state.gamepad.left_stick_raw.x,
        state.gamepad.left_center_offset.x,
    );
    let left_y = input::process_stick_axis(
        state.gamepad.left_stick_raw.y,
        state.gamepad.left_center_offset.y,
    );
    let left_processed = Vec2::new(left_x, left_y);
    input::world_space_thrust_stick(&mut state, &globals, left_processed);

    // Process right stick (aim)
    let right_x = input::process_stick_axis(
        state.gamepad.right_stick_raw.x,
        state.gamepad.right_center_offset.x,
    );
    let right_y = input::process_stick_axis(
        state.gamepad.right_stick_raw.y,
        state.gamepad.right_center_offset.y,
    );
    let right_processed = Vec2::new(right_x, right_y);
    input::aim_from_stick(&mut state.ship, right_processed);

    // Fire on A button held or right trigger
    // (A button state tracked by SDL2 — check if pressed)
    if let Some(ref controller) = active_controller {
        use sdl2::controller::Button;
        if controller.button(Button::A) {
            input::fire(&mut state, &mut globals);
        }
        // Right trigger as fire (analog → digital at 50% threshold)
        use sdl2::controller::Axis;
        let rt = controller.axis(Axis::TriggerRight) as f64 / 32767.0;
        if rt > 0.5 {
            input::fire(&mut state, &mut globals);
        }
        // Left trigger as teleport is handled edge-triggered in event loop (Task 7)
    }

    // Drift compensation update
    let dt = globals.time.time_current_frame - globals.time.time_last_frame;
    let current_time = globals.time.time_current_frame;
    let any_pressed = state.gamepad.any_button_pressed;

    let left_raw = state.gamepad.left_stick_raw;
    input::update_drift_compensation(
        &mut state.gamepad.left_center_offset,
        left_raw,
        any_pressed,
        &mut state.gamepad.last_idle_time,
        current_time,
        dt,
    );
    let right_raw = state.gamepad.right_stick_raw;
    input::update_drift_compensation(
        &mut state.gamepad.right_center_offset,
        right_raw,
        any_pressed,
        &mut state.gamepad.last_idle_time,
        current_time,
        dt,
    );
}
```

### Step 5.6: Add necessary imports to `main.rs`

- [ ] Add `use asteroids::math::Vec2;` at the top of `main.rs` (or wherever needed for `Vec2`).

- [ ] The `sdl2::controller::*` imports are used inline with `use` statements in the match arms.

### Step 5.7: Verify and commit

- [ ] Run `cargo check && cargo clippy`. Fix any borrow checker issues (especially around `state` borrows in the event loop vs. continuous input section).

- [ ] Run the game with a controller connected — verify stick movement works, buttons register.

```bash
git add src/main.rs src/game.rs
git commit -m "feat(input): SDL2 GameController integration with stick input processing"
```

---

## Task 6: Visual Aim Smoothing

**Goal:** Ship sprite rotation smoothly tracks the true aim direction. Smoothed angle used only for rendering — never for gameplay.

**Files:**
- Modify: `src/game.rs` — update `visual_aim_angle` each frame
- Modify: `src/rendering/world.rs` — use `visual_aim_angle` when rendering ship
- Modify: `src/game.rs` (`render_frame`) — pass `visual_aim_angle` to ship rendering

### Step 6.1: Add visual aim angle update to `game.rs`

- [ ] Add this function to `game.rs`:

```rust
/// Update the smoothed visual aim angle to track ship.orientation.
/// Uses exponential approach: visual angle lerps toward true aim at a rate
/// controlled by AIM_VISUAL_SMOOTHING. Handles angle wrapping correctly.
pub fn update_visual_aim(gamepad: &mut GamepadState, target: f64, dt: f64) {
    use std::f64::consts::PI;
    let mut diff = target - gamepad.visual_aim_angle;
    // Wrap to [-PI, PI]
    while diff > PI { diff -= 2.0 * PI; }
    while diff < -PI { diff += 2.0 * PI; }

    if AIM_VISUAL_SMOOTHING <= 0.0 {
        gamepad.visual_aim_angle = target;
    } else {
        let factor = (AIM_VISUAL_SMOOTHING * dt).min(1.0);
        gamepad.visual_aim_angle += diff * factor;
    }
}
```

- [ ] Import `AIM_VISUAL_SMOOTHING` (already covered by `use crate::parameters::*;` if present, otherwise add it).

### Step 6.2: Call visual aim update each frame

- [ ] In `update_game` in `game.rs`, near the end (before particle budgets), add:

```rust
// Update visual aim smoothing
let dt_game = globals.time.time_current_frame - globals.time.time_last_frame;
update_visual_aim(&mut state.gamepad, state.ship.orientation, dt_game);
```

Note: if `dt_game` is already computed earlier in `update_game`, reuse it.

### Step 6.3: Use `visual_aim_angle` for ship rendering

- [ ] In `render_frame` in `game.rs`, change the ship rendering call (line 1137). Currently:
```rust
render_visuals(&state.ship, Vec2::ZERO, renderer, globals, &mut state.rng);
```

The ship is rendered through `render_visuals` → `render_shapes`, which uses `entity.orientation`. To use the smoothed angle, we need to temporarily override `orientation` for rendering, or pass it differently.

**Approach:** Temporarily swap `ship.orientation` with `visual_aim_angle` for ship rendering only:

```rust
// Ship — render with smoothed visual aim angle
let true_aim = state.ship.orientation;
state.ship.orientation = state.gamepad.visual_aim_angle;
render_visuals(&state.ship, Vec2::ZERO, renderer, globals, &mut state.rng);
state.ship.orientation = true_aim;
```

This is a pragmatic approach that avoids threading a new parameter through the entire render chain. The swap is safe because rendering is single-threaded and `orientation` is restored immediately.

### Step 6.4: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Run the game — the ship nose should smoothly track the mouse. Rapid mouse movements should show visible lag in ship rotation while firing remains instant.

```bash
git add src/game.rs src/rendering/world.rs
git commit -m "feat(render): visual aim smoothing for ship rotation"
```

---

## Task 7: Cone-Based Teleport

**Goal:** Replace teleport-to-mouse with cone-based targeting. Cast a 15° cone along aim direction, find the biggest asteroid in the cone, teleport to its center and destroy it.

**Files:**
- Modify: `src/input.rs` — rewrite `teleport` function
- Modify: `src/main.rs` — update teleport call sites (F key + B button)
- Modify: `src/game.rs` — add asteroid destruction on teleport arrival

### Step 7.1: Rewrite `teleport` in `input.rs`

- [ ] Replace the existing `teleport` function with:

```rust
/// Cone-based teleport: cast a 15° cone along aim direction, find the biggest
/// asteroid in the cone (within screen bounds), teleport to its center.
/// Returns the index of the targeted asteroid (if any) for destruction.
pub fn teleport(state: &mut GameState, globals: &mut Globals) -> Option<usize> {
    if state.cooldown_tp > 0.0 {
        return None;
    }

    let cone_half_angle = TELEPORT_CONE_HALF_ANGLE_DEG.to_radians();
    let aim_angle = state.ship.orientation;
    let ship_pos = state.ship.position;

    // Screen bounds for visibility check
    let phys_w = globals.render.phys_width;
    let phys_h = globals.render.phys_height;

    // Find the biggest asteroid (object) within the cone
    let mut best_idx: Option<usize> = None;
    let mut best_radius: f64 = 0.0;

    for (i, asteroid) in state.objects.iter().enumerate() {
        let delta = sub_vec(asteroid.position, ship_pos);
        let distance = (delta.x * delta.x + delta.y * delta.y).sqrt();
        if distance < 1.0 {
            continue; // Too close / overlapping
        }

        // Angle from ship to asteroid center
        let angle_to = delta.y.atan2(delta.x);

        // Angular difference (wrapped to [-PI, PI])
        let mut angle_diff = angle_to - aim_angle;
        while angle_diff > PI { angle_diff -= 2.0 * PI; }
        while angle_diff < -PI { angle_diff += 2.0 * PI; }

        // Effective cone: widen by the asteroid's angular radius
        let angular_radius = (asteroid.hitbox.int_radius / distance).asin().abs();
        let effective_diff = angle_diff.abs() - angular_radius;

        if effective_diff > cone_half_angle {
            continue; // Outside cone
        }

        // Asteroid must be ahead (not behind ship)
        // Check dot product with aim direction
        let aim_dir = from_polar(aim_angle, 1.0);
        let dot = delta.x * aim_dir.x + delta.y * aim_dir.y;
        if dot <= 0.0 {
            continue; // Behind ship
        }

        // Screen bounds check: asteroid center must be visible
        let pos = asteroid.position;
        if pos.x < 0.0 || pos.x > phys_w || pos.y < 0.0 || pos.y > phys_h {
            continue;
        }

        // Pick biggest
        let radius = asteroid.hitbox.int_radius;
        if radius > best_radius {
            best_radius = radius;
            best_idx = Some(i);
        }
    }

    if let Some(idx) = best_idx {
        let target_pos = state.objects[idx].position;

        // Kill the targeted asteroid — set health to 0 so normal fragmentation
        // in update_game (spawn_fragments) handles debris spawning.
        state.objects[idx].health = 0.0;

        // Teleport ship to asteroid center
        state.ship.position = target_pos;
        state.ship.velocity = Vec2::ZERO;

        // Visual flash (blue for teleport)
        if globals.visual.flashes_enabled {
            let flash = intensify(HdrColor { r: 0.0, g: 4.0, b: 40.0 }, 1.0);
            globals.exposure.add_color = (
                globals.exposure.add_color.0 + flash.r,
                globals.exposure.add_color.1 + flash.g,
                globals.exposure.add_color.2 + flash.b,
            );
        }
        globals.exposure.game_exposure *= GAME_EXPOSURE_TP;
        globals.time.game_speed *= RATIO_TIME_TP;

        // Spawn teleport explosion chunks (blue)
        let tp_color = (0.0, 1000.0, 10000.0);
        let new_chunks = spawn_n_chunks(&state.ship, NB_CHUNKS_EXPLO, tp_color, &mut state.rng);
        state.chunks_explo.extend(new_chunks);

        // Reset cooldown
        state.cooldown_tp += COOLDOWN_TP;

        Some(idx)
    } else {
        None // No valid target — teleport fails silently
    }
}
```

- [ ] Add `TELEPORT_CONE_HALF_ANGLE_DEG` import (already via wildcard `use crate::parameters::*;`).

### Step 7.2: Handle asteroid destruction on teleport

- [ ] In `main.rs`, the F key event now calls the new `teleport` (no mouse coords). The asteroid is killed inside `teleport` (health set to 0) — normal fragmentation in `update_game` handles debris. Update the call site:

```rust
Event::KeyDown {
    scancode: Some(Scancode::F),
    repeat: false,
    ..
} => {
    input::teleport(&mut state, &mut globals);
}
```

- [ ] Similarly update the gamepad B button event handler (from Task 5.4):

```rust
Button::B => {
    state.gamepad.any_button_pressed = true;
    input::teleport(&mut state, &mut globals);
}
```

- [ ] Handle Left Trigger as teleport (edge-triggered). Add to the `ControllerAxisMotion` handler:

```rust
Axis::TriggerLeft => {
    // Edge-trigger teleport at 50% threshold
    let was_pressed = state.gamepad.left_trigger_pressed;
    let is_pressed = normalized > 0.5;
    if is_pressed && !was_pressed {
        input::teleport(&mut state, &mut globals);
    }
    state.gamepad.left_trigger_pressed = is_pressed;
}
```

- [ ] `left_trigger_pressed: bool` was already added to `GamepadState` in Task 3 (init `false`).

### Step 7.4: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Run the game — aim at an asteroid, press F. Verify:
  - Ship teleports to the asteroid's center
  - Asteroid is destroyed with fragments
  - Blue explosion chunks spawn
  - No teleport happens if no asteroid is in the cone
  - Cooldown works

```bash
git add src/input.rs src/main.rs src/game.rs
git commit -m "feat(input): cone-based teleport targeting with asteroid destruction"
```

---

## Task 8: Final Integration and Cleanup

**Goal:** Clean up any remaining dead code, verify all input paths work together, and do a final pass.

**Files:**
- Modify: `src/input.rs` — remove any unused imports/functions
- Modify: `src/main.rs` — final cleanup

### Step 8.1: Audit unused code

- [ ] Run `cargo clippy` — fix all warnings about unused imports, dead code, etc.

- [ ] Check that these functions are still used and correct:
  - `aim_at_mouse` — still called for mouse input ✓
  - `fire` — called on left-click and gamepad A/RT ✓
  - `world_space_thrust_keyboard` — called for WASD ✓
  - `world_space_thrust_stick` — called for left stick ✓
  - `aim_from_stick` — called for right stick ✓
  - `process_stick_axis` — called for all stick processing ✓
  - `update_drift_compensation` — called each frame for connected controllers ✓
  - `teleport` — called on F key and B button/LT ✓

### Step 8.2: Verify engine fire direction

- [ ] With the new system, `spawn_fire` (in `objects.rs`) uses `ship.orientation + PI` for the fire position and velocity. Since `orientation` = aim direction now, fire ejects from the opposite side of the aim. This means:
  - If aiming right and moving up, fire comes from the left side of the ship
  - This is visually correct (thruster opposes aim/nose direction)

  If during testing this looks wrong, consider making fire eject opposite to the **movement** direction instead:
  ```rust
  // Alternative: fire opposite to velocity (if needed)
  let move_angle = ship.velocity.y.atan2(ship.velocity.x);
  ```
  But try the current behavior first — it may look fine.

### Step 8.3: Run `cargo fmt`

- [ ] Run `cargo fmt` to ensure consistent formatting.

### Step 8.4: Final commit

```bash
git add -A
git commit -m "chore(input): cleanup and verify gamepad + world-space controls"
```

---

## Summary of Removed Code

| What | Where |
|------|-------|
| `ShipControlConfig` struct | `parameters.rs` |
| `ship_control` field in `Globals` | `parameters.rs` |
| `handle_left`, `handle_right` | `input.rs` |
| `strafe_left`, `strafe_right` | `input.rs` |
| `boost_forward` | `input.rs` |
| `acceleration` | `input.rs` |
| W impulse/continuous block | `main.rs` |
| A/D rotation, Q/E strafe blocks | `main.rs` |
| Space = fire binding | `main.rs` |
| P = pause binding | `main.rs` |
| `prev_w_pressed` variable | `main.rs` |

## Summary of New Code

| What | Where |
|------|-------|
| `GamepadState` struct | `game.rs` |
| Gamepad/teleport constants | `parameters.rs` |
| `world_space_thrust_keyboard` | `input.rs` |
| `world_space_thrust_stick` | `input.rs` |
| `process_stick_axis` | `input.rs` |
| `aim_from_stick` | `input.rs` |
| `update_drift_compensation` | `input.rs` |
| `update_visual_aim` | `game.rs` |
| Cone-based `teleport` | `input.rs` |
| SDL2 GameController init + events | `main.rs` |
| Visual aim swap in `render_frame` | `game.rs` |
