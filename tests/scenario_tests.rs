use asteroids::scenario::{Action, Scenario};

#[test]
fn test_determinism_file_based() {
    let scenario = Scenario::load("scenarios/test_basic.ron").expect("Failed to load scenario");
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
            .snapshot_at(119)
            .run_until(120)
            .run()
    };

    let result_a = run();
    let result_b = run();

    assert_eq!(result_a.snapshots.len(), result_b.snapshots.len());
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
        .snapshot_at(59)
        .run_until(60)
        .run();

    let result_b = Scenario::builder()
        .seed(2)
        .fps(60)
        .spawn_asteroid((500.0, 300.0), 80.0)
        .snapshot_at(59)
        .run_until(60)
        .run();

    // Different seeds should produce different states
    // (asteroids get random properties from the seed)
    assert_ne!(result_a.snapshots[0].data, result_b.snapshots[0].data);
}

#[test]
fn test_assertions_pass() {
    let result = Scenario::builder().seed(42).fps(60).run_until(60).run();

    assert!(result.assertion_failures.is_empty());
    assert!(result.final_state.ship.health > 0.0);
}
