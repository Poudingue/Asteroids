use asteroids::*;

use std::time::Instant;

use parameters::{Globals, MAX_DT};
use rendering::Renderer2D;
use sdl2::keyboard::Scancode;

fn main() {
    // SDL2 init
    let sdl_context = sdl2::init().expect("Failed to init SDL2");
    let video_subsystem = sdl_context.video().expect("Failed to init video");

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
    let mut state = game::GameState::new(&globals);
    let start_time = Instant::now();

    // Event loop
    let mut event_pump = sdl_context.event_pump().expect("Failed to get event pump");
    let mut running = true;
    let mut is_fullscreen = true;

    while running {
        let frame_start = Instant::now();

        // Update time, capping dt to MAX_DT to prevent physics explosions on
        // frame stalls (alt-tab, window drag, etc.). This is equivalent to a
        // 20fps floor: physics never sees more than 50ms per frame.
        globals.time.time_last_frame = globals.time.time_current_frame;
        let raw_elapsed = start_time.elapsed().as_secs_f64();
        globals.time.time_current_frame =
            globals.time.time_last_frame + (raw_elapsed - globals.time.time_last_frame).min(MAX_DT);

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
                    input::teleport(&mut state, &mut globals, mouse_x_snap, mouse_y_snap);
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

            // Mouse click = accelerate forward
            if mouse_state.left() {
                input::acceleration(&mut state, &globals);
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

        renderer.update_postprocess_uniforms(&queue, &rendering::PostProcessUniforms {
            game_exposure: globals.exposure.game_exposure as f32,
            add_color_r: globals.exposure.add_color.0 as f32,
            add_color_g: globals.exposure.add_color.1 as f32,
            add_color_b: globals.exposure.add_color.2 as f32,
            mul_color_r: globals.exposure.mul_color.0 as f32,
            mul_color_g: globals.exposure.mul_color.1 as f32,
            mul_color_b: globals.exposure.mul_color.2 as f32,
            _padding: 0.0,
        });
        renderer.begin_frame();
        game::render_frame(&mut state, &mut globals, &mut renderer, mouse_x_snap, mouse_y_snap, mouse_left_snap);
        renderer.end_frame(&device, &queue, &view, [0.0, 0.0, 0.0, 1.0]);
        globals.framerate.frame_compute_secs = frame_start.elapsed().as_secs_f64();
        output.present();
    }

    println!("Bye bye!");
}
