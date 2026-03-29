# Gamepad + World-Space Controls Design Spec

**Date**: 2026-03-28
**Status**: Draft
**Phase**: V2 — Pre-Phase 1 (input architecture)

---

## Overview

Replace the rotate+thrust control scheme with world-space twin-stick controls. Add SDL2 gamepad support. Rework teleport into an aim-directed cone-based targeting mechanic.

---

## Control Scheme

### Movement (world-space, decoupled from aim)

| Input | Keyboard | Gamepad |
|---|---|---|
| Move up | W | Left stick up |
| Move down | S | Left stick down |
| Move left | A | Left stick left |
| Move right | D | Left stick right |
| Aim | Mouse position | Right stick angle |
| Fire | Left click | Right trigger / A button |
| Teleport | F | Left trigger / B button |
| Pause | Esc | Start |

- **Keyboard**: WASD = full thrust in that world-space direction. Diagonal inputs (W+D) normalized to unit magnitude.
- **Gamepad left stick**: Analog thrust, magnitude proportional to stick deflection after dead zone processing.
- Ship `orientation` field repurposed: now means "aim direction" exclusively. Set by mouse angle or right stick angle.
- Ship visual (nose direction) follows aim. Movement is independent of facing.
- **Removed bindings**: Space (fire), P (pause), A/D rotation, Q/E strafing — no longer meaningful in world-space mode.

### Aim System

- **Mouse**: `atan2(mouse_y - ship_y, mouse_x - ship_x)` → `ship.orientation` (instant, same as current `aim_at_mouse`)
- **Right stick**: `atan2(ry, rx)` → `ship.orientation` (only when stick magnitude > dead zone). When stick is in dead zone, keep last aim direction (don't snap to zero).
- **Aim is always mechanically instant** — firing and teleport cone always use the true aim direction immediately.
- **Visual smoothing**: Ship sprite rotation can lag behind the true aim for visual polish. Configurable via `aim_visual_smoothing` constant (0.0 = instant, higher = more smoothing). The smoothed angle is used ONLY for rendering the ship — never for gameplay mechanics (firing, teleport).

---

## Dead Zones + Drift Compensation

### Per-axis dead zone with outer clamp

Processing pipeline per axis:

```
raw_axis → subtract center_offset → inner dead zone (< 15% → 0) → outer dead zone (> 90% → 1.0) → linear remap [0.15, 0.90] → [0.0, 1.0]
```

Applied independently to X and Y axes before combining into a Vec2.

Constants (all configurable):
- `STICK_DEAD_ZONE_INNER: f64 = 0.15` — below this, treat as zero
- `STICK_DEAD_ZONE_OUTER: f64 = 0.90` — above this, treat as 1.0

### Adaptive center (drift compensation)

- Track `center_offset: Vec2` per stick (left and right), initialized to (0, 0)
- When no gamepad buttons are pressed AND stick reads a stable value for ~2 seconds: smoothly lerp `center_offset` toward the current raw reading
- All stick reads subtract `center_offset` before dead zone processing
- On controller connect/reconnect: reset `center_offset` to (0, 0)

Constants:
- `DRIFT_RECENTER_DELAY: f64 = 2.0` — seconds of idle before recalibration starts
- `DRIFT_RECENTER_SPEED: f64 = 0.5` — lerp speed toward new center

---

## Teleport Targeting (Cone-Based)

Replaces the current "teleport to mouse cursor" mechanic. Unified behavior for mouse and gamepad — both use aim direction.

### Mechanic

1. Cast a 15° cone from ship position along aim direction
2. Filter candidates: all asteroids whose hitbox intersects the cone AND are within visible screen bounds
3. Pick the **biggest** by radius among candidates
4. Teleport ship to that asteroid's center
5. Same effect as current: temporary invulnerability + instant kill + blue explosion
6. If no asteroid in cone → teleport fails (nothing happens)

### Cone intersection test

For each asteroid:
1. Compute angle from ship to asteroid center
2. Check if this angle is within ±7.5° of aim direction
3. Account for asteroid radius: use `angle_to_center ± asin(radius/distance)` so large asteroids at the cone edge are still caught
4. Distance must be positive (asteroid is ahead of ship, not behind)
5. Asteroid must be within visible screen bounds

Constants:
- `TELEPORT_CONE_HALF_ANGLE: f64 = 7.5` — degrees, half of 15° total cone
- Screen bounds check uses the existing visible area from `globals.render`

### Goal

The design intent is to teleport INTO the biggest asteroid to destroy it from the inside, provoking chain reactions via the explosion fragments.

---

## Code Architecture

### Modified files

| File | Changes |
|---|---|
| `src/input.rs` | Rewrite: world-space movement functions, aim-from-mouse, aim-from-stick, cone-based teleport targeting. Remove rotation/strafing functions. |
| `src/main.rs` | Init SDL2 GameController subsystem, open controllers, poll ControllerAxisMotion/ControllerButtonDown events, route to new input functions |
| `src/parameters.rs` | New `GamepadConfig` sub-struct. New `TeleportConfig`. Remove `ShipControlConfig` entirely. |
| `src/game.rs` | Remove `handle_left`/`handle_right`/`strafe_left`/`strafe_right`. Fire/recoil already use `ship.orientation` (now = aim direction). Teleport function rewritten for cone targeting. |
| `src/objects.rs` | No structural changes — `orientation` repurposed semantically from "facing" to "aim direction" |
| `src/rendering/world.rs` | Ship visual rotation uses smoothed aim (visual_aim_angle), not raw ship.orientation |
| `src/pause_menu.rs` | Remove impulse/direct toggle buttons. Gamepad can navigate pause menu with D-pad/A/B. |

### New config structs in parameters.rs

```rust
pub struct GamepadConfig {
    pub stick_dead_zone_inner: f64,    // 0.15
    pub stick_dead_zone_outer: f64,    // 0.90
    pub drift_recenter_delay: f64,     // 2.0 seconds
    pub drift_recenter_speed: f64,     // 0.5
    pub aim_visual_smoothing: f64,     // 0.0 = instant
}

pub struct TeleportConfig {
    pub cone_half_angle_deg: f64,      // 7.5 degrees
}
```

### New runtime state

```rust
pub struct GamepadState {
    pub connected: bool,
    pub left_center_offset: Vec2,
    pub right_center_offset: Vec2,
    pub last_idle_time: f64,
    pub visual_aim_angle: f64,         // smoothed visual orientation for rendering
}
```

### Removed

- `ShipControlConfig` struct (ship_direct_pos, ship_direct_rotat, ship_impulse_pos, ship_impulse_rotat)
- All rotation input functions: `handle_left`, `handle_right`
- All strafe input functions: `strafe_left`, `strafe_right`
- Impulse/direct toggle buttons in pause menu
- `GlobalToggle::DirectPos`, `GlobalToggle::DirectRotat`, `GlobalToggle::ImpulsePos`, `GlobalToggle::ImpulseRotat` (if they exist)

### Dead zone processing function (in input.rs)

```rust
fn process_stick_axis(raw: f64, center_offset: f64, inner_dz: f64, outer_dz: f64) -> f64 {
    let adjusted = raw - center_offset;
    let abs_val = adjusted.abs();
    if abs_val < inner_dz { return 0.0; }
    if abs_val > outer_dz { return adjusted.signum(); }
    let remapped = (abs_val - inner_dz) / (outer_dz - inner_dz);
    remapped * adjusted.signum()
}
```

### SDL2 GameController integration (in main.rs)

- Initialize: `sdl_context.game_controller()?`
- On `Event::ControllerDeviceAdded`: open controller, reset drift offsets
- On `Event::ControllerDeviceRemoved`: mark disconnected
- On `Event::ControllerAxisMotion`: update raw axis values in GamepadState
- On `Event::ControllerButtonDown/Up`: handle fire (A/RT), teleport (B/LT), pause (Start)
- Axis mapping: `LeftX`/`LeftY` → movement, `RightX`/`RightY` → aim
- SDL2 axis values are i16 [-32768, 32767] — normalize to f64 [-1.0, 1.0]

### Movement processing (in input.rs)

```rust
fn world_space_thrust_keyboard(ship: &mut Entity, keys: &HashSet<Scancode>, globals: &Globals) {
    let mut dir = Vec2 { x: 0.0, y: 0.0 };
    if keys.contains(&Scancode::W) { dir.y -= 1.0; }
    if keys.contains(&Scancode::S) { dir.y += 1.0; }
    if keys.contains(&Scancode::A) { dir.x -= 1.0; }
    if keys.contains(&Scancode::D) { dir.x += 1.0; }
    let mag = (dir.x*dir.x + dir.y*dir.y).sqrt();
    if mag > 0.0 {
        let normalized = Vec2 { x: dir.x/mag, y: dir.y/mag };
        accelerate_entity(ship, normalized * SHIP_ACCEL, globals);
    }
}

fn world_space_thrust_stick(ship: &mut Entity, stick: Vec2, globals: &Globals) {
    let mag = (stick.x*stick.x + stick.y*stick.y).sqrt();
    if mag > 0.0 {
        let clamped_mag = mag.min(1.0);
        let direction = Vec2 { x: stick.x/mag, y: stick.y/mag };
        accelerate_entity(ship, direction * SHIP_ACCEL * clamped_mag, globals);
    }
}
```

---

## Dependencies on Later Phases

- Phase 3 (Camera & Zoom): gamepad controls work independently of camera changes
- Phase 5 (Weapons): weapon switching could map to gamepad bumpers/Y button
- Future: vibration/haptic feedback on teleport impact, aim assist for stick users
