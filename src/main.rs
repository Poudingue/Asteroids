use asteroids::*;

use std::time::Instant;

use clap::Parser;
use parameters::{Globals, SimulationMode, MAX_DT};
use rendering::Renderer2D;
use sdl2::controller::{Axis, Button};
use sdl2::keyboard::Scancode;

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

    // Headless mode: skip all SDL2/wgpu init, run scenario directly
    if !simulation_mode.needs_window() {
        run_headless(&cli);
        return;
    }

    // SDL2 init
    let sdl_context = sdl2::init().expect("Failed to init SDL2");
    let video_subsystem = sdl_context.video().expect("Failed to init video");
    let game_controller_subsystem = sdl_context
        .game_controller()
        .expect("Failed to init game controller");

    // Start borderless fullscreen at desktop resolution
    let mut window = video_subsystem
        .window("Asteroids", 0, 0)
        .fullscreen_desktop()
        .build()
        .expect("Failed to create window");

    // Ensure mouse cursor is visible (even in fullscreen)
    sdl_context.mouse().show_cursor(true);

    // Get the actual window dimensions after fullscreen
    let (width, height) = window.size();

    // wgpu init
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });

    // Safety: window handle is valid for the lifetime of the window
    let surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap())
            .expect("Failed to create surface")
    };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .expect("Failed to find adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .expect("Failed to create device");

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .find(|f| !f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let mut renderer = Renderer2D::new(&device, surface_format, width, height);

    // Game state init
    let mut globals = Globals::new();
    globals.recompute_for_resolution(width, height);
    globals.time.simulation_mode = simulation_mode;
    let mut state = if simulation_mode.fixed_dt().is_some() {
        game::GameState::new_with_seed(&globals, cli.seed)
    } else {
        game::GameState::new(&globals)
    };
    let start_time = Instant::now();

    // Event loop
    let mut event_pump = sdl_context.event_pump().expect("Failed to get event pump");
    let mut running = true;
    let mut is_fullscreen = true;
    let mut active_controller: Option<sdl2::controller::GameController> = None;

    // Open any controller already connected at startup
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

    while running {
        let frame_start = Instant::now();

        // Update time
        globals.time.time_last_frame = globals.time.time_current_frame;
        match globals.time.simulation_mode.fixed_dt() {
            Some(dt) => {
                // Fixed-dt mode: advance by exactly 1/target_fps
                globals.time.time_current_frame += dt;
            }
            None => {
                // RealTime mode: wall-clock dt, capped at MAX_DT to prevent
                // physics explosions on frame stalls (alt-tab, window drag).
                let raw_elapsed = start_time.elapsed().as_secs_f64();
                globals.time.time_current_frame = globals.time.time_last_frame
                    + (raw_elapsed - globals.time.time_last_frame).min(MAX_DT);
            }
        }
        globals.time.frame_count += 1;

        // Snapshot mouse position and button state before poll_iter
        let (mouse_x_snap, mouse_y_snap, mouse_left_snap) = {
            let ms = event_pump.mouse_state();
            (ms.x() as f64, ms.y() as f64, ms.left())
        };

        // Poll events (discrete actions: quit, pause)
        for event in event_pump.poll_iter() {
            use sdl2::event::{Event, WindowEvent};
            use sdl2::keyboard::Keycode;
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::K),
                    ..
                } => running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    repeat: false,
                    ..
                } => globals.time.pause = !globals.time.pause,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    repeat: false,
                    ..
                } => {
                    state = game::GameState::new(&globals);
                    globals.exposure.game_exposure = 0.0;
                    globals.time.pause = false;
                }
                Event::KeyDown {
                    scancode: Some(Scancode::F),
                    repeat: false,
                    ..
                } => {
                    input::teleport(&mut state, &mut globals);
                }
                // F11: toggle fullscreen
                Event::KeyDown {
                    keycode: Some(Keycode::F11),
                    repeat: false,
                    ..
                } => {
                    use sdl2::video::FullscreenType;
                    if is_fullscreen {
                        window
                            .set_fullscreen(FullscreenType::Off)
                            .unwrap_or_else(|e| eprintln!("Fullscreen toggle failed: {e}"));
                        is_fullscreen = false;
                    } else {
                        window
                            .set_fullscreen(FullscreenType::Desktop)
                            .unwrap_or_else(|e| eprintln!("Fullscreen toggle failed: {e}"));
                        is_fullscreen = true;
                    }
                }
                // Alt+Enter: toggle fullscreen
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    keymod,
                    repeat: false,
                    ..
                } if keymod.contains(sdl2::keyboard::Mod::LALTMOD)
                    || keymod.contains(sdl2::keyboard::Mod::RALTMOD) =>
                {
                    use sdl2::video::FullscreenType;
                    if is_fullscreen {
                        window
                            .set_fullscreen(FullscreenType::Off)
                            .unwrap_or_else(|e| eprintln!("Fullscreen toggle failed: {e}"));
                        is_fullscreen = false;
                    } else {
                        window
                            .set_fullscreen(FullscreenType::Desktop)
                            .unwrap_or_else(|e| eprintln!("Fullscreen toggle failed: {e}"));
                        is_fullscreen = true;
                    }
                }
                // Window resize: reconfigure wgpu surface, renderer, and physics dimensions
                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    let new_w = w.max(1) as u32;
                    let new_h = h.max(1) as u32;
                    config.width = new_w;
                    config.height = new_h;
                    surface.configure(&device, &config);
                    renderer.resize(&device, &queue, new_w, new_h);
                    globals.recompute_for_resolution(new_w, new_h);
                }
                // `which` here is the device index (not instance ID) — used to open the controller
                Event::ControllerDeviceAdded { which, .. } => {
                    if active_controller.is_none() {
                        match game_controller_subsystem.open(which) {
                            Ok(controller) => {
                                println!("Controller connected: {}", controller.name());
                                state.gamepad.connected = true;
                                state.gamepad.left_center_offset = math::Vec2::ZERO;
                                state.gamepad.right_center_offset = math::Vec2::ZERO;
                                active_controller = Some(controller);
                            }
                            Err(e) => eprintln!("Failed to open controller: {}", e),
                        }
                    }
                }
                // `which` here is the joystick instance ID (not device index) — matches c.instance_id()
                Event::ControllerDeviceRemoved { which, .. } => {
                    if let Some(ref c) = active_controller {
                        if c.instance_id() == which {
                            println!("Controller disconnected");
                            state.gamepad.connected = false;
                            state.gamepad.left_stick_raw = math::Vec2::ZERO;
                            state.gamepad.right_stick_raw = math::Vec2::ZERO;
                            active_controller = None;
                        }
                    }
                }
                Event::ControllerAxisMotion { axis, value, .. } => {
                    let normalized = value as f64 / 32767.0;
                    match axis {
                        Axis::LeftX => state.gamepad.left_stick_raw.x = normalized,
                        Axis::LeftY => state.gamepad.left_stick_raw.y = -normalized,
                        Axis::RightX => state.gamepad.right_stick_raw.x = normalized,
                        Axis::RightY => state.gamepad.right_stick_raw.y = -normalized,
                        Axis::TriggerLeft => {
                            let was_pressed = state.gamepad.left_trigger_pressed;
                            let is_pressed = normalized > 0.5;
                            if is_pressed && !was_pressed {
                                input::teleport(&mut state, &mut globals);
                            }
                            state.gamepad.left_trigger_pressed = is_pressed;
                        }
                        _ => {}
                    }
                }
                Event::ControllerButtonDown { button, .. } => match button {
                    Button::B => {
                        state.gamepad.any_button_pressed = true;
                        input::teleport(&mut state, &mut globals);
                    }
                    Button::Start => globals.time.pause = !globals.time.pause,
                    _ => state.gamepad.any_button_pressed = true,
                },
                Event::ControllerButtonUp { .. } => {
                    state.gamepad.any_button_pressed = false;
                }
                _ => {}
            }
        }

        // Handle flags set by pause menu buttons
        if globals.time.quit {
            running = false;
        }
        if globals.time.restart {
            globals.time.restart = false;
            globals.time.pause = false;
            state = game::GameState::new(&globals);
            globals.exposure.game_exposure = 0.0;
        }

        // Track mouse button state in GameState
        state.mouse_button_down = mouse_left_snap;

        if !globals.time.pause {
            // Mouse aim
            let mouse_state = event_pump.mouse_state();
            input::aim_at_mouse(&mut state.ship, mouse_state.x(), mouse_state.y(), &globals);

            // WASD world-space movement
            let keyboard = event_pump.keyboard_state();
            let keys_pressed = [
                keyboard.is_scancode_pressed(Scancode::W),
                keyboard.is_scancode_pressed(Scancode::A),
                keyboard.is_scancode_pressed(Scancode::S),
                keyboard.is_scancode_pressed(Scancode::D),
            ];
            input::world_space_thrust_keyboard(&mut state, &globals, keys_pressed);

            // Mouse left-click = fire
            if mouse_state.left() {
                input::fire(&mut state, &mut globals);
            }

            // Gamepad input processing
            if state.gamepad.connected {
                // Process left stick (movement) — copy raw values before &mut state borrow
                let left_raw = state.gamepad.left_stick_raw;
                let left_offset = state.gamepad.left_center_offset;
                let right_raw = state.gamepad.right_stick_raw;
                let right_offset = state.gamepad.right_center_offset;

                let left_x = input::process_stick_axis(left_raw.x, left_offset.x);
                let left_y = input::process_stick_axis(left_raw.y, left_offset.y);
                let left_processed = math::Vec2::new(left_x, left_y);
                input::world_space_thrust_stick(&mut state, &globals, left_processed);

                // Process right stick (aim)
                let right_x = input::process_stick_axis(right_raw.x, right_offset.x);
                let right_y = input::process_stick_axis(right_raw.y, right_offset.y);
                let right_processed = math::Vec2::new(right_x, right_y);
                input::aim_from_stick(&mut state.ship, right_processed);

                // Fire on A button held or right trigger
                if let Some(ref controller) = active_controller {
                    if controller.button(Button::A) {
                        input::fire(&mut state, &mut globals);
                    }
                    let rt = controller.axis(Axis::TriggerRight) as f64 / 32767.0;
                    if rt > 0.5 {
                        input::fire(&mut state, &mut globals);
                    }
                }

                // Drift compensation update
                let dt = globals.time.time_current_frame - globals.time.time_last_frame;
                let current_time = globals.time.time_current_frame;
                let any_pressed = state.gamepad.any_button_pressed;

                input::update_drift_compensation(
                    &mut state.gamepad.left_center_offset,
                    left_raw,
                    any_pressed,
                    &mut state.gamepad.last_idle_time,
                    current_time,
                    dt,
                );
                input::update_drift_compensation(
                    &mut state.gamepad.right_center_offset,
                    right_raw,
                    any_pressed,
                    &mut state.gamepad.last_idle_time,
                    current_time,
                    dt,
                );
            }

            // Update game state (physics, wrapping, asteroids, etc.)
            game::update_game(&mut state, &mut globals);
        }

        // Update per-frame globals (screenshake pos, jitter, exposure, game speed, etc.)
        // Must run AFTER update_game so screenshake from death/damage is sampled this frame
        game::update_frame(&mut globals, &mut state.rng);

        // Render
        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) => {
                surface.configure(&device, &config);
                continue;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                running = false;
                continue;
            }
            Err(e) => {
                eprintln!("Surface error: {:?}", e);
                continue;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        renderer.update_postprocess_uniforms(
            &queue,
            &rendering::PostProcessUniforms {
                game_exposure: globals.exposure.game_exposure as f32,
                add_color_r: globals.exposure.add_color.0 as f32,
                add_color_g: globals.exposure.add_color.1 as f32,
                add_color_b: globals.exposure.add_color.2 as f32,
                mul_color_r: globals.exposure.mul_color.0 as f32,
                mul_color_g: globals.exposure.mul_color.1 as f32,
                mul_color_b: globals.exposure.mul_color.2 as f32,
                _padding: 0.0,
            },
        );
        renderer.begin_frame();
        game::render_frame(
            &mut state,
            &mut globals,
            &mut renderer,
            mouse_x_snap,
            mouse_y_snap,
            mouse_left_snap,
        );
        renderer.end_frame(&device, &queue, &view, [0.0, 0.0, 0.0, 1.0]);
        globals.framerate.frame_compute_secs = frame_start.elapsed().as_secs_f64();
        output.present();

        // Frame pacing for FixedInteractive mode
        if globals.time.simulation_mode.should_sleep() {
            if let Some(target_dt) = globals.time.simulation_mode.fixed_dt() {
                let elapsed = frame_start.elapsed().as_secs_f64();
                if elapsed < target_dt {
                    std::thread::sleep(std::time::Duration::from_secs_f64(target_dt - elapsed));
                }
            }
        }
    }

    println!("Bye bye!");
}

fn run_headless(cli: &Cli) {
    let scenario_path = cli
        .scenario
        .as_ref()
        .expect("Headless mode requires --scenario");

    let scenario =
        asteroids::scenario::Scenario::load(scenario_path).expect("Failed to load scenario");

    println!(
        "Running headless: {} ({} frames at {} fps)",
        scenario.def.name, scenario.def.run_until, scenario.def.target_fps
    );

    let start = std::time::Instant::now();
    let result = scenario.run();
    let elapsed = start.elapsed();

    println!(
        "Completed in {:.2}s ({:.0} sim-fps)",
        elapsed.as_secs_f64(),
        scenario.def.run_until as f64 / elapsed.as_secs_f64()
    );

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
