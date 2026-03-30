# Fixed-dt Deterministic Simulation Mode — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add fixed-timestep deterministic simulation with scripted scenarios, state snapshots, headless mode, and input recording for reproducible testing and replay.

**Architecture:** Replace `ThreadRng` with seedable `SmallRng`, add `SimulationMode` enum for fixed-dt stepping, `clap` CLI for mode selection, RON-based scenario files with actions/assertions, binary input recording with zstd, and conditional headless init that skips SDL2 video + wgpu.

**Tech Stack:** Rust, `clap` (CLI), `ron` + `serde` (scenarios), `zstd` (input compression), `rand::rngs::SmallRng` (deterministic RNG)

**Design spec:** `docs/superpowers/specs/2026-03-30-fixed-dt-deterministic-mode-design.md`

---

## File Structure

| File | Role |
|------|------|
| `src/parameters.rs` | Add `SimulationMode` enum to `TimeConfig` |
| `src/game.rs` | `SmallRng` instead of `ThreadRng`, `GameState` derives `Serialize`/`Clone` for snapshots |
| `src/scenario.rs` | **New** — `Scenario`, `Action`, `SetupAction`, `Assertion`, `ScenarioResult`, `StateSnapshot`, builder API, load/run |
| `src/recording.rs` | **New** — `InputFrame`, `.inputs` binary format, zstd compress/decompress |
| `src/time.rs` | **New** — `SimulationMode` enum, fixed-dt time source logic |
| `src/main.rs` | CLI parsing (clap), branch on mode, headless path, recording hooks |
| `src/lib.rs` | Export new modules |
| `Cargo.toml` | Add deps: `clap`, `ron`, `serde` (derive), `zstd`, `bincode` |

---

## Task 1: Replace `ThreadRng` with `SmallRng`

**Goal:** Make the RNG seedable for deterministic simulation. No behavioral change in normal gameplay — seed from entropy by default.

**Files:**
- Modify: `src/game.rs` — change `rng` field type, update constructor
- Modify: `Cargo.toml` — add `serde` with derive feature (needed later, add now)

### Step 1.1: Update `Cargo.toml`

- [ ] Add `serde` dependency with derive feature:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
```

Keep all existing deps unchanged.

### Step 1.2: Change `rng` field type in `GameState`

- [ ] In `src/game.rs`, add the import for `SmallRng` and `SeedableRng`:

```rust
use rand::rngs::SmallRng;
use rand::SeedableRng;
```

- [ ] Change the `rng` field type in `GameState` (line 93):

From:
```rust
pub rng: ThreadRng,
```
To:
```rust
pub rng: SmallRng,
```

### Step 1.3: Update `GameState::new()` constructor

- [ ] Change the RNG initialization in `GameState::new()`. Currently at line 106:

From:
```rust
let mut rng = thread_rng();
```
To:
```rust
let mut rng = SmallRng::from_entropy();
```

### Step 1.4: Add `GameState::new_with_seed()` constructor

- [ ] Add a new constructor for deterministic mode, right after `new()`:

```rust
/// Create a new GameState with a fixed seed for deterministic simulation.
pub fn new_with_seed(globals: &Globals, seed: u64) -> Self {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut ship = new_ship();
    ship.position = Vec2::new(
        globals.render.phys_width / 2.0,
        globals.render.phys_height / 2.0,
    );

    let mut state = Self {
        score: 0,
        lives: NB_LIVES,
        stage: 1,
        cooldown: 0.0,
        cooldown_tp: 0.0,
        last_health: SHIP_MAX_HEALTH,
        is_dead: false,
        time_of_death: 0.0,
        ship,
        objects: Vec::new(),
        objects_oos: Vec::new(),
        toosmall: Vec::new(),
        toosmall_oos: Vec::new(),
        fragments: Vec::new(),
        chunks: Vec::new(),
        chunks_oos: Vec::new(),
        chunks_explo: Vec::new(),
        projectiles: Vec::new(),
        explosions: Vec::new(),
        smoke: Vec::new(),
        smoke_oos: Vec::new(),
        sparks: Vec::new(),
        stars: Vec::new(),
        rng,
        buttons: Vec::new(),
        mouse_button_down: false,
        gamepad: GamepadState::new(),
    };

    // Spawn initial stars
    for _ in 0..NB_STARS {
        state.stars.push(spawn_random_star(
            globals.render.phys_width,
            globals.render.phys_height,
            &mut state.rng,
        ));
    }
    state.buttons = crate::pause_menu::make_buttons(globals);
    state
}
```

Note: This duplicates `new()` but with a seeded RNG. Check the actual `new()` body and replicate it exactly, just changing the `rng` init line.

### Step 1.5: Verify and commit

- [ ] Run `cargo check && cargo clippy`. The only expected warning is that `new_with_seed` is unused — suppress with `#[allow(dead_code)]` for now (will be used in Task 5).
- [ ] Run `cargo test` — ensure existing tests still pass (they use `impl Rng` so the type change is transparent).

```bash
git add src/game.rs Cargo.toml
git commit -m "refactor: replace ThreadRng with seedable SmallRng"
```

---

## Task 2: `SimulationMode` Enum and Fixed-dt Time Stepping

**Goal:** Add the `SimulationMode` enum and modify the game loop to support fixed-dt modes.

**Files:**
- Modify: `src/parameters.rs` — add `SimulationMode` to `TimeConfig`
- Modify: `src/main.rs` — fixed-dt time stepping logic

### Step 2.1: Add `SimulationMode` enum to `parameters.rs`

- [ ] Add the enum before `TimeConfig`:

```rust
/// Simulation time stepping mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimulationMode {
    /// Variable dt from wall clock, capped at MAX_DT. Default gameplay mode.
    RealTime,
    /// Fixed dt = 1/target_fps. Sleeps if frame is faster. Playable + deterministic.
    FixedInteractive(u32),
    /// Fixed dt = 1/target_fps. No sleeping — runs as fast as possible.
    FixedFullSpeed(u32),
    /// Fixed dt, no window, no renderer. Pure simulation.
    Headless(u32),
}

impl SimulationMode {
    /// Returns the fixed dt if in a fixed mode, None for RealTime.
    pub fn fixed_dt(&self) -> Option<f64> {
        match self {
            SimulationMode::RealTime => None,
            SimulationMode::FixedInteractive(fps)
            | SimulationMode::FixedFullSpeed(fps)
            | SimulationMode::Headless(fps) => Some(1.0 / *fps as f64),
        }
    }

    /// Whether this mode requires a window and renderer.
    pub fn needs_window(&self) -> bool {
        !matches!(self, SimulationMode::Headless(_))
    }

    /// Whether this mode should sleep to maintain target framerate.
    pub fn should_sleep(&self) -> bool {
        matches!(self, SimulationMode::FixedInteractive(_))
    }
}
```

- [ ] Add `simulation_mode` field to `TimeConfig`:

```rust
pub struct TimeConfig {
    pub simulation_mode: SimulationMode,
    pub game_speed: f64,
    // ... rest unchanged
}
```

- [ ] Initialize it in `Globals::new()`:

```rust
time: TimeConfig {
    simulation_mode: SimulationMode::RealTime,
    game_speed: 1.0,
    // ... rest unchanged
},
```

### Step 2.2: Add frame counter to `TimeConfig`

- [ ] Add a frame counter field to `TimeConfig` (needed for scenario action timing):

```rust
pub frame_count: u64,
```

Initialize to `0` in `Globals::new()`.

### Step 2.3: Modify time stepping in `main.rs`

- [ ] In the game loop, replace the current time stepping block (lines 119–122):

From:
```rust
globals.time.time_last_frame = globals.time.time_current_frame;
let raw_elapsed = start_time.elapsed().as_secs_f64();
globals.time.time_current_frame =
    globals.time.time_last_frame + (raw_elapsed - globals.time.time_last_frame).min(MAX_DT);
```

To:
```rust
globals.time.time_last_frame = globals.time.time_current_frame;
match globals.time.simulation_mode.fixed_dt() {
    Some(dt) => {
        // Fixed-dt mode: advance by exactly 1/target_fps
        globals.time.time_current_frame += dt;
    }
    None => {
        // RealTime mode: wall-clock dt, capped at MAX_DT
        let raw_elapsed = start_time.elapsed().as_secs_f64();
        globals.time.time_current_frame = globals.time.time_last_frame
            + (raw_elapsed - globals.time.time_last_frame).min(MAX_DT);
    }
}
globals.time.frame_count += 1;
```

- [ ] At the end of the game loop (after `output.present()`), add frame pacing for `FixedInteractive`:

```rust
// Frame pacing for FixedInteractive mode
if globals.time.simulation_mode.should_sleep() {
    if let Some(target_dt) = globals.time.simulation_mode.fixed_dt() {
        let elapsed = frame_start.elapsed().as_secs_f64();
        if elapsed < target_dt {
            std::thread::sleep(std::time::Duration::from_secs_f64(target_dt - elapsed));
        }
    }
}
```

### Step 2.4: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Run the game normally — should behave identically (defaults to `RealTime`).

```bash
git add src/parameters.rs src/main.rs
git commit -m "feat: SimulationMode enum and fixed-dt time stepping"
```

---

## Task 3: CLI Parsing with `clap`

**Goal:** Add command-line argument parsing to select simulation mode, scenario file, seed, and fps.

**Files:**
- Modify: `Cargo.toml` — add `clap` dependency
- Modify: `src/main.rs` — parse CLI args, wire to `SimulationMode`

### Step 3.1: Add `clap` dependency

- [ ] Add to `Cargo.toml`:

```toml
clap = { version = "4", features = ["derive"] }
```

### Step 3.2: Define CLI args struct in `main.rs`

- [ ] Add at the top of `main.rs`, after imports:

```rust
use clap::Parser;

/// Asteroids — a space shooter with deterministic simulation support
#[derive(Parser, Debug)]
#[command(name = "asteroids")]
struct Cli {
    /// Path to a scenario file (.ron)
    #[arg(long)]
    scenario: Option<String>,

    /// Run headless (no window, no GPU)
    #[arg(long)]
    headless: bool,

    /// Run at full speed (no frame pacing)
    #[arg(long)]
    full_speed: bool,

    /// Record input to file
    #[arg(long)]
    record: Option<String>,

    /// RNG seed for deterministic mode
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Target FPS for fixed-dt modes
    #[arg(long, default_value_t = 60)]
    fps: u32,
}
```

### Step 3.3: Parse args and set `SimulationMode`

- [ ] At the start of `main()`, parse CLI args and determine mode:

```rust
fn main() {
    let cli = Cli::parse();

    // Determine simulation mode
    let simulation_mode = if cli.headless {
        SimulationMode::Headless(cli.fps)
    } else if cli.full_speed {
        SimulationMode::FixedFullSpeed(cli.fps)
    } else if cli.scenario.is_some() {
        SimulationMode::FixedInteractive(cli.fps)
    } else {
        SimulationMode::RealTime
    };
```

- [ ] After `Globals::new()`, set the mode:

```rust
globals.time.simulation_mode = simulation_mode;
```

- [ ] Use `new_with_seed` when in fixed-dt mode:

```rust
let mut state = if simulation_mode.fixed_dt().is_some() {
    game::GameState::new_with_seed(&globals, cli.seed)
} else {
    game::GameState::new(&globals)
};
```

### Step 3.4: Add `SimulationMode` import

- [ ] Add import at top of `main.rs`:

```rust
use parameters::SimulationMode;
```

### Step 3.5: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Test: `cargo run -- --help` should show the CLI options.
- [ ] Test: `cargo run` (no args) should work normally as before.
- [ ] Test: `cargo run -- --full-speed --fps 60 --seed 123` should run with fixed dt.

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: clap CLI for simulation mode selection"
```

---

## Task 4: Scenario Types and RON Loading

**Goal:** Define the scenario data types (actions, assertions, setup) and load them from `.ron` files.

**Files:**
- Create: `src/scenario.rs` — all scenario types and loading
- Modify: `src/lib.rs` — export scenario module
- Modify: `Cargo.toml` — add `ron` dependency

### Step 4.1: Add `ron` dependency

- [ ] Add to `Cargo.toml`:

```toml
ron = "0.8"
```

### Step 4.2: Create `src/scenario.rs` with types

- [ ] Create `src/scenario.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

// ============================================================================
// Scenario definition types (deserialized from .ron files)
// ============================================================================

/// Top-level scenario definition, loaded from a .ron file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDef {
    pub name: String,
    pub seed: u64,
    pub target_fps: u32,
    pub mode: ScenarioMode,
    #[serde(default)]
    pub setup: Vec<SetupAction>,
    #[serde(default)]
    pub actions: Vec<TimedAction>,
    pub run_until: u64,
    /// Optional path to a .inputs file for dense input replay
    #[serde(default)]
    pub input_file: Option<String>,
    /// Frames at which to capture full state snapshots
    #[serde(default)]
    pub snapshots_at: Vec<u64>,
    /// Log entity trajectories every N frames (0 = disabled)
    #[serde(default)]
    pub trajectory_interval: u64,
    /// Simple assertions to check after running
    #[serde(default)]
    pub assertions: Vec<TimedAssertion>,
}

/// Scenario execution mode (maps to SimulationMode but without RealTime).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScenarioMode {
    Interactive,
    FullSpeed,
    Headless,
}

/// An action applied during scenario setup (before simulation starts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SetupAction {
    SpawnAsteroid {
        pos: (f64, f64),
        radius: f64,
        #[serde(default)]
        velocity: (f64, f64),
    },
    SetShipPosition(f64, f64),
    SetShipVelocity(f64, f64),
    SetShipAim(f64),
}

/// A timed action: execute at a specific frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedAction {
    pub frame: u64,
    pub action: Action,
}

/// An input action during simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Set aim angle (radians)
    AimAt(f64),
    /// Fire weapons
    Fire,
    /// Set movement direction (world-space, like WASD). (0,0) = stop.
    MoveDirection(f64, f64),
    /// Trigger teleport
    Teleport,
    /// Stop all movement
    StopMoving,
    /// Set left stick axes directly
    LeftStick(f64, f64),
    /// Set right stick axes directly
    RightStick(f64, f64),
}

/// A timed assertion: check condition at a specific frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedAssertion {
    pub frame: u64,
    pub check: AssertionCheck,
}

/// Simple assertion checks (evaluated in the scenario runner).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssertionCheck {
    ObjectCountAtLeast(usize),
    ObjectCountAtMost(usize),
    ShipAlive,
    ShipDead,
    ScoreAbove(i32),
}

// ============================================================================
// Loading
// ============================================================================

impl ScenarioDef {
    /// Load a scenario from a .ron file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read scenario file: {}", e))?;
        ron::from_str(&content)
            .map_err(|e| format!("Failed to parse scenario file: {}", e))
    }
}
```

### Step 4.3: Export module in `lib.rs`

- [ ] Add to `src/lib.rs`:

```rust
pub mod scenario;
```

### Step 4.4: Create a test scenario file

- [ ] Create `scenarios/test_basic.ron`:

```ron
ScenarioDef(
    name: "basic_test",
    seed: 42,
    target_fps: 60,
    mode: FullSpeed,
    setup: [
        SpawnAsteroid(pos: (500.0, 300.0), radius: 80.0, velocity: (10.0, -5.0)),
        SpawnAsteroid(pos: (600.0, 350.0), radius: 60.0, velocity: (0.0, 0.0)),
        SetShipPosition(400.0, 300.0),
    ],
    actions: [
        (frame: 30, action: AimAt(0.52)),
        (frame: 60, action: Fire),
        (frame: 90, action: MoveDirection(1.0, 0.0)),
        (frame: 120, action: Teleport),
        (frame: 120, action: StopMoving),
    ],
    run_until: 300,
    snapshots_at: [60, 120, 180, 300],
    trajectory_interval: 10,
    assertions: [
        (frame: 300, check: ShipAlive),
    ],
)
```

### Step 4.5: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Write a quick unit test in `scenario.rs` to verify loading:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_scenario() {
        let scenario = ScenarioDef::load("scenarios/test_basic.ron")
            .expect("Failed to load test scenario");
        assert_eq!(scenario.name, "basic_test");
        assert_eq!(scenario.seed, 42);
        assert_eq!(scenario.target_fps, 60);
        assert_eq!(scenario.run_until, 300);
        assert_eq!(scenario.setup.len(), 3);
        assert_eq!(scenario.actions.len(), 5);
    }
}
```

- [ ] Run `cargo test -- test_load_scenario`.

```bash
git add Cargo.toml src/scenario.rs src/lib.rs scenarios/test_basic.ron
git commit -m "feat: scenario types and RON loading"
```

---

## Task 5: Scenario Runner

**Goal:** Implement the core scenario execution loop — apply setup, run simulation with timed actions, collect results.

**Files:**
- Modify: `src/scenario.rs` — add `Scenario`, `ScenarioResult`, run logic
- Modify: `src/game.rs` — add `Serialize` derive to key types, add `apply_setup_action` helper

### Step 5.1: Add `serde` derives to game types

- [ ] In `src/game.rs`, add `Serialize` derive to `GameState`. This requires all field types to be serializable. `SmallRng` does NOT implement `Serialize`, so we need to skip it:

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct GameState {
    // ... all existing fields ...
    #[serde(skip)]
    pub rng: SmallRng,
    #[serde(skip)]
    pub buttons: Vec<ButtonBoolean>,
    // ... rest unchanged
}
```

Note: `ButtonBoolean` (from pause_menu) doesn't implement Serialize — skip it too. `Entity`, `Vec2`, `Star`, `GamepadState`, etc. will need `#[derive(Serialize)]` — check each type and add the derive. Most are simple structs with numeric fields.

- [ ] Add `Serialize` to `Vec2` in `src/math.rs`, `Entity` and related types in `src/objects.rs`, `GamepadState` in `src/game.rs`. For any type that contains non-serializable fields (like function pointers), use `#[serde(skip)]`.

### Step 5.2: Add state snapshot type to `scenario.rs`

- [ ] Add to `src/scenario.rs`:

```rust
use crate::game::GameState;
use crate::parameters::Globals;

/// A serialized snapshot of game state at a specific frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub frame: u64,
    /// RON-serialized GameState
    pub data: String,
}

impl StateSnapshot {
    /// Capture a snapshot of the current game state.
    pub fn capture(state: &GameState, frame: u64) -> Self {
        let data = ron::ser::to_string_pretty(state, ron::ser::PrettyConfig::default())
            .expect("Failed to serialize game state");
        Self { frame, data }
    }
}

/// A trajectory entry for one entity at one frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryEntry {
    pub frame: u64,
    pub entity_index: usize,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub health: f64,
    pub radius: f64,
}

/// Result of running a scenario.
pub struct ScenarioResult {
    pub final_state: GameState,
    pub snapshots: Vec<StateSnapshot>,
    pub trajectories: Vec<TrajectoryEntry>,
    pub assertion_failures: Vec<String>,
}
```

### Step 5.3: Implement scenario runner

- [ ] Add the `Scenario` struct and `run()` method to `scenario.rs`:

```rust
use crate::game;
use crate::input;
use crate::objects::*;
use crate::parameters::SimulationMode;

/// A loaded scenario ready to run.
pub struct Scenario {
    pub def: ScenarioDef,
}

impl Scenario {
    /// Load a scenario from a .ron file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let def = ScenarioDef::load(path)?;
        Ok(Self { def })
    }

    /// Run the scenario to completion, returning results.
    pub fn run(&self) -> ScenarioResult {
        let mut globals = Globals::new();
        // Use a reasonable default resolution for headless
        globals.recompute_for_resolution(1920, 1080);
        globals.time.simulation_mode = match self.def.mode {
            ScenarioMode::FullSpeed => SimulationMode::FixedFullSpeed(self.def.target_fps),
            ScenarioMode::Interactive => SimulationMode::FixedInteractive(self.def.target_fps),
            ScenarioMode::Headless => SimulationMode::Headless(self.def.target_fps),
        };

        let mut state = game::GameState::new_with_seed(&globals, self.def.seed);

        // Apply setup actions
        for action in &self.def.setup {
            apply_setup_action(&mut state, &mut globals, action);
        }

        let mut snapshots = Vec::new();
        let mut trajectories = Vec::new();
        let mut assertion_failures = Vec::new();

        // Sort actions by frame for efficient lookup
        let mut actions = self.def.actions.clone();
        actions.sort_by_key(|a| a.frame);
        let mut action_idx = 0;

        // Active input state (persists between frames)
        let mut move_dir: (f64, f64) = (0.0, 0.0);
        let mut firing = false;

        let dt = 1.0 / self.def.target_fps as f64;

        for frame in 0..self.def.run_until {
            // Apply actions for this frame
            while action_idx < actions.len() && actions[action_idx].frame == frame {
                match &actions[action_idx].action {
                    Action::AimAt(angle) => {
                        state.ship.orientation = *angle;
                    }
                    Action::Fire => {
                        firing = true;
                    }
                    Action::MoveDirection(x, y) => {
                        move_dir = (*x, *y);
                    }
                    Action::Teleport => {
                        input::teleport(&mut state, &mut globals);
                    }
                    Action::StopMoving => {
                        move_dir = (0.0, 0.0);
                    }
                    _ => {} // LeftStick/RightStick handled similarly
                }
                action_idx += 1;
            }

            // Apply continuous inputs
            let mag = (move_dir.0 * move_dir.0 + move_dir.1 * move_dir.1).sqrt();
            if mag > 0.0 {
                let keys = [
                    move_dir.1 > 0.5,   // W
                    move_dir.0 < -0.5,  // A
                    move_dir.1 < -0.5,  // S
                    move_dir.0 > 0.5,   // D
                ];
                input::world_space_thrust_keyboard(&mut state, &globals, keys);
            }
            if firing {
                input::fire(&mut state, &mut globals);
            }

            // Advance time
            globals.time.time_last_frame = globals.time.time_current_frame;
            globals.time.time_current_frame += dt;
            globals.time.frame_count = frame;

            // Run game update
            game::update_game(&mut state, &mut globals);
            game::update_frame(&mut globals, &mut state.rng);

            // Capture snapshots
            if self.def.snapshots_at.contains(&frame) {
                snapshots.push(StateSnapshot::capture(&state, frame));
            }

            // Capture trajectories
            if self.def.trajectory_interval > 0 && frame % self.def.trajectory_interval == 0 {
                for (i, obj) in state.objects.iter().enumerate() {
                    trajectories.push(TrajectoryEntry {
                        frame,
                        entity_index: i,
                        x: obj.position.x,
                        y: obj.position.y,
                        vx: obj.velocity.x,
                        vy: obj.velocity.y,
                        health: obj.health,
                        radius: obj.hitbox.int_radius,
                    });
                }
            }

            // Check assertions
            for assertion in &self.def.assertions {
                if assertion.frame == frame {
                    if let Some(failure) = check_assertion(&state, &assertion.check, frame) {
                        assertion_failures.push(failure);
                    }
                }
            }
        }

        ScenarioResult {
            final_state: state,
            snapshots,
            trajectories,
            assertion_failures,
        }
    }
}

/// Apply a setup action before simulation starts.
fn apply_setup_action(state: &mut GameState, globals: &mut Globals, action: &SetupAction) {
    match action {
        SetupAction::SpawnAsteroid { pos, radius, velocity } => {
            let mut asteroid = new_asteroid(*radius, &mut state.rng);
            asteroid.position = crate::math::Vec2::new(pos.0, pos.1);
            asteroid.velocity = crate::math::Vec2::new(velocity.0, velocity.1);
            state.objects.push(asteroid);
        }
        SetupAction::SetShipPosition(x, y) => {
            state.ship.position = crate::math::Vec2::new(*x, *y);
        }
        SetupAction::SetShipVelocity(vx, vy) => {
            state.ship.velocity = crate::math::Vec2::new(*vx, *vy);
        }
        SetupAction::SetShipAim(angle) => {
            state.ship.orientation = *angle;
        }
    }
}

/// Check an assertion against the current state. Returns None if passed, Some(error) if failed.
fn check_assertion(state: &GameState, check: &AssertionCheck, frame: u64) -> Option<String> {
    match check {
        AssertionCheck::ObjectCountAtLeast(n) => {
            if state.objects.len() < *n {
                Some(format!("Frame {}: expected at least {} objects, got {}", frame, n, state.objects.len()))
            } else {
                None
            }
        }
        AssertionCheck::ObjectCountAtMost(n) => {
            if state.objects.len() > *n {
                Some(format!("Frame {}: expected at most {} objects, got {}", frame, n, state.objects.len()))
            } else {
                None
            }
        }
        AssertionCheck::ShipAlive => {
            if state.ship.health <= 0.0 {
                Some(format!("Frame {}: expected ship alive, but health = {}", frame, state.ship.health))
            } else {
                None
            }
        }
        AssertionCheck::ShipDead => {
            if state.ship.health > 0.0 {
                Some(format!("Frame {}: expected ship dead, but health = {}", frame, state.ship.health))
            } else {
                None
            }
        }
        AssertionCheck::ScoreAbove(n) => {
            if state.score <= *n {
                Some(format!("Frame {}: expected score above {}, got {}", frame, n, state.score))
            } else {
                None
            }
        }
    }
}
```

### Step 5.4: Check that `new_asteroid` is accessible

- [ ] Verify that `new_asteroid` (or equivalent) exists in `src/objects.rs` and is public. If the function has a different name, find the correct asteroid spawning function and use it in `apply_setup_action`. Search for `pub fn new_asteroid` or `pub fn spawn_asteroid` in `objects.rs`.

### Step 5.5: Write a determinism test

- [ ] Add to the `tests` module in `scenario.rs`:

```rust
#[test]
fn test_determinism() {
    let scenario = Scenario::load("scenarios/test_basic.ron")
        .expect("Failed to load scenario");
    let result_a = scenario.run();
    let result_b = scenario.run();

    // Same seed + same actions = same snapshots
    assert_eq!(result_a.snapshots.len(), result_b.snapshots.len());
    for (a, b) in result_a.snapshots.iter().zip(result_b.snapshots.iter()) {
        assert_eq!(a.frame, b.frame);
        assert_eq!(a.data, b.data, "State diverged at frame {}", a.frame);
    }
}
```

### Step 5.6: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Run `cargo test -- test_determinism`.

```bash
git add src/scenario.rs src/game.rs src/objects.rs src/math.rs
git commit -m "feat: scenario runner with deterministic execution and state snapshots"
```

---

## Task 6: Headless Mode

**Goal:** When `--headless` is passed (or scenario mode is Headless), skip SDL2 video, wgpu, and rendering entirely. Run pure simulation loop.

**Files:**
- Modify: `src/main.rs` — conditional init, headless game loop branch

### Step 6.1: Restructure `main.rs` for conditional init

- [ ] Split `main()` into two paths based on `simulation_mode.needs_window()`. The headless path loads the scenario and runs it directly:

```rust
fn main() {
    let cli = Cli::parse();

    let simulation_mode = /* ... same as before ... */;

    // Headless mode: skip all SDL2/wgpu init, run scenario directly
    if !simulation_mode.needs_window() {
        run_headless(&cli, simulation_mode);
        return;
    }

    // Normal windowed mode: existing SDL2 + wgpu init below
    // ... rest of existing main() unchanged ...
}

fn run_headless(cli: &Cli, mode: SimulationMode) {
    let scenario_path = cli.scenario.as_ref()
        .expect("Headless mode requires --scenario");

    let scenario = asteroids::scenario::Scenario::load(scenario_path)
        .expect("Failed to load scenario");

    println!("Running headless: {} ({} frames at {} fps)",
        scenario.def.name, scenario.def.run_until, scenario.def.target_fps);

    let start = std::time::Instant::now();
    let result = scenario.run();
    let elapsed = start.elapsed();

    println!("Completed in {:.2}s ({:.0} sim-fps)",
        elapsed.as_secs_f64(),
        scenario.def.run_until as f64 / elapsed.as_secs_f64());

    // Report assertion results
    if result.assertion_failures.is_empty() {
        println!("All assertions passed.");
    } else {
        eprintln!("Assertion failures:");
        for failure in &result.assertion_failures {
            eprintln!("  - {}", failure);
        }
        std::process::exit(1);
    }

    // Write snapshots to disk
    for snapshot in &result.snapshots {
        let path = format!("{}.snapshot.{}", scenario_path, snapshot.frame);
        std::fs::write(&path, &snapshot.data)
            .unwrap_or_else(|e| eprintln!("Failed to write snapshot: {}", e));
        println!("Snapshot written: {}", path);
    }
}
```

### Step 6.2: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Test headless: `cargo run -- --headless --scenario scenarios/test_basic.ron`
  - Should print timing info and assertion results, no window appears.

```bash
git add src/main.rs
git commit -m "feat: headless simulation mode (no window, no GPU)"
```

---

## Task 7: Dense Input Recording

**Goal:** Record per-frame input state to a binary `.inputs` file with zstd compression. Support replay via scenario `input_file` reference.

**Files:**
- Create: `src/recording.rs` — InputFrame, write/read .inputs files
- Modify: `src/lib.rs` — export recording module
- Modify: `src/main.rs` — recording hooks in the game loop
- Modify: `Cargo.toml` — add `zstd` and `bincode` dependencies

### Step 7.1: Add dependencies

- [ ] Add to `Cargo.toml`:

```toml
zstd = "0.13"
bincode = "1"
```

### Step 7.2: Create `src/recording.rs`

- [ ] Create `src/recording.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;

/// Header for a .inputs recording file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingHeader {
    pub seed: u64,
    pub target_fps: u32,
    pub frame_count: u64,
}

/// Per-frame input state (dense recording).
/// 4 stick axes (f32 for space efficiency) + button bitfield.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InputFrame {
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    pub buttons: u16,
}

// Button bitfield constants
pub const BTN_FIRE: u16 = 1 << 0;
pub const BTN_TELEPORT: u16 = 1 << 1;
pub const BTN_PAUSE: u16 = 1 << 2;
pub const BTN_MOVE_W: u16 = 1 << 3;
pub const BTN_MOVE_A: u16 = 1 << 4;
pub const BTN_MOVE_S: u16 = 1 << 5;
pub const BTN_MOVE_D: u16 = 1 << 6;

impl InputFrame {
    pub fn new() -> Self {
        Self {
            left_stick_x: 0.0,
            left_stick_y: 0.0,
            right_stick_x: 0.0,
            right_stick_y: 0.0,
            buttons: 0,
        }
    }

    pub fn has_button(&self, btn: u16) -> bool {
        self.buttons & btn != 0
    }

    pub fn set_button(&mut self, btn: u16) {
        self.buttons |= btn;
    }
}

/// Writer for .inputs files (zstd-compressed bincode).
pub struct InputRecorder {
    frames: Vec<InputFrame>,
    header: RecordingHeader,
}

impl InputRecorder {
    pub fn new(seed: u64, target_fps: u32) -> Self {
        Self {
            frames: Vec::new(),
            header: RecordingHeader {
                seed,
                target_fps,
                frame_count: 0,
            },
        }
    }

    pub fn push_frame(&mut self, frame: InputFrame) {
        self.frames.push(frame);
        self.header.frame_count = self.frames.len() as u64;
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let file = std::fs::File::create(path.as_ref())
            .map_err(|e| format!("Failed to create recording file: {}", e))?;
        let mut encoder = zstd::Encoder::new(file, 3)
            .map_err(|e| format!("Failed to create zstd encoder: {}", e))?;

        // Write header
        let header_bytes = bincode::serialize(&self.header)
            .map_err(|e| format!("Failed to serialize header: {}", e))?;
        let header_len = header_bytes.len() as u32;
        encoder.write_all(&header_len.to_le_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        encoder.write_all(&header_bytes)
            .map_err(|e| format!("Write error: {}", e))?;

        // Write frames
        let frames_bytes = bincode::serialize(&self.frames)
            .map_err(|e| format!("Failed to serialize frames: {}", e))?;
        encoder.write_all(&frames_bytes)
            .map_err(|e| format!("Write error: {}", e))?;

        encoder.finish()
            .map_err(|e| format!("Failed to finish zstd stream: {}", e))?;

        Ok(())
    }
}

/// Reader for .inputs files.
pub struct InputPlayback {
    pub header: RecordingHeader,
    pub frames: Vec<InputFrame>,
}

impl InputPlayback {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| format!("Failed to open recording file: {}", e))?;
        let mut decoder = zstd::Decoder::new(file)
            .map_err(|e| format!("Failed to create zstd decoder: {}", e))?;

        // Read header
        let mut len_buf = [0u8; 4];
        decoder.read_exact(&mut len_buf)
            .map_err(|e| format!("Read error: {}", e))?;
        let header_len = u32::from_le_bytes(len_buf) as usize;
        let mut header_buf = vec![0u8; header_len];
        decoder.read_exact(&mut header_buf)
            .map_err(|e| format!("Read error: {}", e))?;
        let header: RecordingHeader = bincode::deserialize(&header_buf)
            .map_err(|e| format!("Failed to deserialize header: {}", e))?;

        // Read frames
        let mut frames_buf = Vec::new();
        decoder.read_to_end(&mut frames_buf)
            .map_err(|e| format!("Read error: {}", e))?;
        let frames: Vec<InputFrame> = bincode::deserialize(&frames_buf)
            .map_err(|e| format!("Failed to deserialize frames: {}", e))?;

        Ok(Self { header, frames })
    }

    pub fn frame(&self, index: u64) -> Option<&InputFrame> {
        self.frames.get(index as usize)
    }
}
```

### Step 7.3: Export module

- [ ] Add to `src/lib.rs`:

```rust
pub mod recording;
```

### Step 7.4: Add recording hooks to `main.rs`

- [ ] When `--record` is passed, create an `InputRecorder` before the game loop, capture input each frame, and save on exit:

```rust
// Before game loop:
let mut recorder = cli.record.as_ref().map(|_| {
    recording::InputRecorder::new(cli.seed, cli.fps)
});

// Inside game loop, after input polling:
if let Some(ref mut rec) = recorder {
    let mut frame = recording::InputFrame::new();
    // Capture keyboard state
    let keyboard = event_pump.keyboard_state();
    if keyboard.is_scancode_pressed(Scancode::W) { frame.set_button(recording::BTN_MOVE_W); }
    if keyboard.is_scancode_pressed(Scancode::A) { frame.set_button(recording::BTN_MOVE_A); }
    if keyboard.is_scancode_pressed(Scancode::S) { frame.set_button(recording::BTN_MOVE_S); }
    if keyboard.is_scancode_pressed(Scancode::D) { frame.set_button(recording::BTN_MOVE_D); }
    if mouse_left_snap { frame.set_button(recording::BTN_FIRE); }
    // Capture gamepad state
    frame.left_stick_x = state.gamepad.left_stick_raw.x as f32;
    frame.left_stick_y = state.gamepad.left_stick_raw.y as f32;
    frame.right_stick_x = state.gamepad.right_stick_raw.x as f32;
    frame.right_stick_y = state.gamepad.right_stick_raw.y as f32;
    rec.push_frame(frame);
}

// After game loop ends:
if let Some(rec) = recorder {
    if let Some(path) = &cli.record {
        rec.save(path).expect("Failed to save recording");
        println!("Recording saved: {} ({} frames)", path, rec.header.frame_count);
    }
}
```

### Step 7.5: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Test recording: `cargo run -- --record test_session.inputs --seed 42 --fps 60` — play briefly, quit with K. Verify file is created.

```bash
git add Cargo.toml src/recording.rs src/lib.rs src/main.rs
git commit -m "feat: dense input recording with zstd compression"
```

---

## Task 8: Builder API and Integration Tests

**Goal:** Add programmatic scenario builder for Rust tests. Write integration tests for determinism.

**Files:**
- Modify: `src/scenario.rs` — add builder pattern
- Create: `tests/scenario_tests.rs` — integration tests

### Step 8.1: Add builder to `scenario.rs`

- [ ] Add after the `Scenario` impl:

```rust
/// Builder for creating scenarios programmatically (used in tests).
pub struct ScenarioBuilder {
    def: ScenarioDef,
}

impl ScenarioBuilder {
    pub fn new() -> Self {
        Self {
            def: ScenarioDef {
                name: "programmatic".to_string(),
                seed: 42,
                target_fps: 60,
                mode: ScenarioMode::Headless,
                setup: Vec::new(),
                actions: Vec::new(),
                run_until: 60,
                input_file: None,
                snapshots_at: Vec::new(),
                trajectory_interval: 0,
                assertions: Vec::new(),
            },
        }
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.def.seed = seed;
        self
    }

    pub fn fps(mut self, fps: u32) -> Self {
        self.def.target_fps = fps;
        self
    }

    pub fn spawn_asteroid(mut self, pos: (f64, f64), radius: f64) -> Self {
        self.def.setup.push(SetupAction::SpawnAsteroid {
            pos,
            radius,
            velocity: (0.0, 0.0),
        });
        self
    }

    pub fn spawn_asteroid_with_velocity(mut self, pos: (f64, f64), radius: f64, vel: (f64, f64)) -> Self {
        self.def.setup.push(SetupAction::SpawnAsteroid {
            pos,
            radius,
            velocity: vel,
        });
        self
    }

    pub fn ship_at(mut self, x: f64, y: f64) -> Self {
        self.def.setup.push(SetupAction::SetShipPosition(x, y));
        self
    }

    pub fn at_frame(mut self, frame: u64, action: Action) -> Self {
        self.def.actions.push(TimedAction { frame, action });
        self
    }

    pub fn snapshot_at(mut self, frame: u64) -> Self {
        self.def.snapshots_at.push(frame);
        self
    }

    pub fn run_until(mut self, frame: u64) -> Self {
        self.def.run_until = frame;
        self
    }

    pub fn build(self) -> Scenario {
        Scenario { def: self.def }
    }

    /// Build and run immediately, returning results.
    pub fn run(self) -> ScenarioResult {
        self.build().run()
    }
}

impl Scenario {
    pub fn builder() -> ScenarioBuilder {
        ScenarioBuilder::new()
    }
}
```

### Step 8.2: Create integration tests

- [ ] Create `tests/scenario_tests.rs`:

```rust
use asteroids::scenario::{Action, Scenario, ScenarioBuilder};

#[test]
fn test_determinism_file_based() {
    let scenario = Scenario::load("scenarios/test_basic.ron")
        .expect("Failed to load scenario");
    let result_a = scenario.run();
    let result_b = scenario.run();

    assert_eq!(result_a.snapshots.len(), result_b.snapshots.len());
    for (a, b) in result_a.snapshots.iter().zip(result_b.snapshots.iter()) {
        assert_eq!(a.frame, b.frame);
        assert_eq!(a.data, b.data, "State diverged at frame {}", a.frame);
    }
}

#[test]
fn test_determinism_builder() {
    let run = || {
        Scenario::builder()
            .seed(12345)
            .fps(60)
            .spawn_asteroid((500.0, 300.0), 80.0)
            .spawn_asteroid((600.0, 350.0), 60.0)
            .ship_at(400.0, 300.0)
            .at_frame(30, Action::AimAt(0.5))
            .at_frame(60, Action::Fire)
            .snapshot_at(60)
            .snapshot_at(120)
            .run_until(120)
            .run()
    };

    let result_a = run();
    let result_b = run();

    for (a, b) in result_a.snapshots.iter().zip(result_b.snapshots.iter()) {
        assert_eq!(a.data, b.data, "State diverged at frame {}", a.frame);
    }
}

#[test]
fn test_different_seeds_diverge() {
    let result_a = Scenario::builder()
        .seed(1)
        .fps(60)
        .spawn_asteroid((500.0, 300.0), 80.0)
        .snapshot_at(60)
        .run_until(60)
        .run();

    let result_b = Scenario::builder()
        .seed(2)
        .fps(60)
        .spawn_asteroid((500.0, 300.0), 80.0)
        .snapshot_at(60)
        .run_until(60)
        .run();

    // Different seeds should produce different states
    // (asteroids get random properties from the seed)
    assert_ne!(result_a.snapshots[0].data, result_b.snapshots[0].data);
}

#[test]
fn test_assertions_pass() {
    let result = Scenario::builder()
        .seed(42)
        .fps(60)
        .run_until(60)
        .run();

    assert!(result.assertion_failures.is_empty());
    assert!(result.final_state.ship.health > 0.0);
}
```

### Step 8.3: Verify and commit

- [ ] Run `cargo test -- scenario_tests`.
- [ ] Run `cargo clippy`.

```bash
git add src/scenario.rs tests/scenario_tests.rs
git commit -m "feat: scenario builder API and determinism integration tests"
```

---

## Task 9: Wire Scenario Loading in Windowed Mode

**Goal:** When `--scenario` is passed without `--headless`, load the scenario and run it in the windowed game loop with fixed-dt.

**Files:**
- Modify: `src/main.rs` — load scenario, apply setup, run actions during game loop

### Step 9.1: Load scenario in windowed mode

- [ ] After creating `state` and `globals`, if a scenario file is provided:

```rust
let scenario_def = cli.scenario.as_ref().map(|path| {
    let def = asteroids::scenario::ScenarioDef::load(path)
        .expect("Failed to load scenario");
    println!("Loaded scenario: {} ({} frames)", def.name, def.run_until);
    def
});

// Apply setup actions if scenario loaded
if let Some(ref def) = scenario_def {
    for action in &def.setup {
        asteroids::scenario::apply_setup_action(&mut state, &mut globals, action);
    }
}
```

Note: Make `apply_setup_action` public in `scenario.rs` if it isn't already.

### Step 9.2: Apply actions during game loop

- [ ] Track action index and apply actions at the right frame:

```rust
let mut scenario_action_idx = 0;
let mut scenario_move_dir: (f64, f64) = (0.0, 0.0);
let mut scenario_firing = false;

// Inside game loop, before update_game:
if let Some(ref def) = scenario_def {
    while scenario_action_idx < def.actions.len()
        && def.actions[scenario_action_idx].frame == globals.time.frame_count
    {
        // ... apply action (same logic as scenario runner)
        scenario_action_idx += 1;
    }

    // Auto-quit when scenario ends
    if globals.time.frame_count >= def.run_until {
        running = false;
    }
}
```

### Step 9.3: Verify and commit

- [ ] Run `cargo check && cargo clippy`.
- [ ] Test: `cargo run -- --scenario scenarios/test_basic.ron` — should run the scenario in a window with fixed-dt, auto-quit at run_until.

```bash
git add src/main.rs src/scenario.rs
git commit -m "feat: scenario loading in windowed mode"
```

---

## Task 10: Final Cleanup

**Goal:** Clean up warnings, verify all paths work, update documentation.

**Files:**
- Various — clippy fixes, remove dead code warnings

### Step 10.1: Remove `#[allow(dead_code)]` annotations

- [ ] Remove any `#[allow(dead_code)]` added during earlier tasks — all functions should now have callers.

### Step 10.2: Run full test suite

- [ ] Run `cargo test` — all tests should pass.
- [ ] Run `cargo clippy` — fix any warnings.
- [ ] Run `cargo fmt`.

### Step 10.3: Update BACKLOG.md

- [ ] Mark the fixed-dt deterministic mode task as complete in BACKLOG.md.
- [ ] Add entry to DONE.md.

### Step 10.4: Final commit

```bash
git add -A
git commit -m "chore: cleanup and verify fixed-dt deterministic mode"
```

---

## Summary

| Task | What | Files |
|------|------|-------|
| 1 | SmallRng replacement | game.rs, Cargo.toml |
| 2 | SimulationMode + fixed-dt | parameters.rs, main.rs |
| 3 | CLI with clap | Cargo.toml, main.rs |
| 4 | Scenario types + RON | scenario.rs, lib.rs, Cargo.toml |
| 5 | Scenario runner | scenario.rs, game.rs, objects.rs |
| 6 | Headless mode | main.rs |
| 7 | Dense input recording | recording.rs, lib.rs, main.rs, Cargo.toml |
| 8 | Builder API + tests | scenario.rs, tests/ |
| 9 | Windowed scenario | main.rs |
| 10 | Cleanup | various |
