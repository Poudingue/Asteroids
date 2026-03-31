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
        ron::from_str(&content).map_err(|e| format!("Failed to parse scenario file: {}", e))
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
