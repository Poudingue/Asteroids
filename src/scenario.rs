use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::game::{self, GameState};
use crate::input;
use crate::math::Vec2;
use crate::parameters::{Globals, SimulationMode};
use crate::spawning::spawn_asteroid;

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
        ron::from_str(&content).map_err(|e| format!("Failed to parse scenario file: {}", e))
    }
}

// ============================================================================
// State snapshots
// ============================================================================

/// A serialized snapshot of game state at a specific frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub frame: u64,
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

// ============================================================================
// Scenario runner
// ============================================================================

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
        globals.recompute_for_resolution(1920, 1080);
        globals.time.simulation_mode = match self.def.mode {
            ScenarioMode::FullSpeed => SimulationMode::FixedFullSpeed(self.def.target_fps),
            ScenarioMode::Interactive => SimulationMode::FixedInteractive(self.def.target_fps),
            ScenarioMode::Headless => SimulationMode::Headless(self.def.target_fps),
        };

        let mut state = game::GameState::new_with_seed(&globals, self.def.seed);

        // Apply setup actions
        for action in &self.def.setup {
            apply_setup_action(&mut state, action);
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
                        firing = false;
                    }
                    Action::LeftStick(x, y) => {
                        let stick = Vec2::new(*x, *y);
                        input::world_space_thrust_stick(&mut state, &globals, stick);
                    }
                    Action::RightStick(_x, _y) => {
                        // Right stick controls aim — handled via AimAt for now
                    }
                }
                action_idx += 1;
            }

            // Apply continuous inputs
            let mag = (move_dir.0 * move_dir.0 + move_dir.1 * move_dir.1).sqrt();
            if mag > 0.0 {
                let keys = [
                    move_dir.1 > 0.5,  // W
                    move_dir.0 < -0.5, // A
                    move_dir.1 < -0.5, // S
                    move_dir.0 > 0.5,  // D
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
            crate::update::update_frame(&mut globals, &mut state.rng);

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
pub fn apply_setup_action(state: &mut GameState, action: &SetupAction) {
    match action {
        SetupAction::SpawnAsteroid {
            pos,
            radius,
            velocity,
        } => {
            let asteroid = spawn_asteroid(
                Vec2::new(pos.0, pos.1),
                Vec2::new(velocity.0, velocity.1),
                *radius,
                &mut state.rng,
            );
            state.objects.push(asteroid);
        }
        SetupAction::SetShipPosition(x, y) => {
            state.ship.position = Vec2::new(*x, *y);
        }
        SetupAction::SetShipVelocity(vx, vy) => {
            state.ship.velocity = Vec2::new(*vx, *vy);
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
                Some(format!(
                    "Frame {}: expected at least {} objects, got {}",
                    frame,
                    n,
                    state.objects.len()
                ))
            } else {
                None
            }
        }
        AssertionCheck::ObjectCountAtMost(n) => {
            if state.objects.len() > *n {
                Some(format!(
                    "Frame {}: expected at most {} objects, got {}",
                    frame,
                    n,
                    state.objects.len()
                ))
            } else {
                None
            }
        }
        AssertionCheck::ShipAlive => {
            if state.ship.health <= 0.0 {
                Some(format!(
                    "Frame {}: expected ship alive, but health = {}",
                    frame, state.ship.health
                ))
            } else {
                None
            }
        }
        AssertionCheck::ShipDead => {
            if state.ship.health > 0.0 {
                Some(format!(
                    "Frame {}: expected ship dead, but health = {}",
                    frame, state.ship.health
                ))
            } else {
                None
            }
        }
        AssertionCheck::ScoreAbove(n) => {
            if state.score <= *n {
                Some(format!(
                    "Frame {}: expected score above {}, got {}",
                    frame, n, state.score
                ))
            } else {
                None
            }
        }
    }
}

// ============================================================================
// Builder API (for programmatic scenario creation in tests)
// ============================================================================

/// Builder for creating scenarios programmatically.
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

    pub fn spawn_asteroid_with_velocity(
        mut self,
        pos: (f64, f64),
        radius: f64,
        vel: (f64, f64),
    ) -> Self {
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

impl Default for ScenarioBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Scenario {
    pub fn builder() -> ScenarioBuilder {
        ScenarioBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_scenario() {
        let scenario =
            ScenarioDef::load("scenarios/test_basic.ron").expect("Failed to load test scenario");
        assert_eq!(scenario.name, "basic_test");
        assert_eq!(scenario.seed, 42);
        assert_eq!(scenario.target_fps, 60);
        assert_eq!(scenario.run_until, 300);
        assert_eq!(scenario.setup.len(), 3);
        assert_eq!(scenario.actions.len(), 5);
    }
}
