mod color;
mod game;
mod math_utils;
mod objects;
mod parameters;
mod renderer;

use std::time::Instant;

use parameters::Globals;
use renderer::Renderer2D;

fn main() {
    // SDL2 init
    let sdl_context = sdl2::init().expect("Failed to init SDL2");
    let video_subsystem = sdl_context.video().expect("Failed to init video");

    let width: u32 = parameters::WIDTH as u32;
    let height: u32 = parameters::HEIGHT as u32;

    let window = video_subsystem
        .window("Asteroids", width, height)
        .position_centered()
        .build()
        .expect("Failed to create window");

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
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
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
    let mut state = game::GameState::new(&globals);
    let start_time = Instant::now();

    // Event loop
    let mut event_pump = sdl_context.event_pump().expect("Failed to get event pump");
    let mut running = true;

    while running {
        // Update time
        globals.time_last_frame = globals.time_current_frame;
        globals.time_current_frame = start_time.elapsed().as_secs_f64();

        // Update per-frame globals (jitter, exposure, game speed, etc.)
        game::update_frame(&mut globals, &mut state.rng);

        // Poll events (discrete actions: quit, pause)
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::K),
                    ..
                } => running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    repeat: false,
                    ..
                } => globals.pause = !globals.pause,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    repeat: false,
                    ..
                } => {
                    state = game::GameState::new(&globals);
                    globals.game_exposure = 0.0;
                    globals.pause = false;
                }
                _ => {}
            }
        }

        if !globals.pause {
            // Mouse aim
            let mouse_state = event_pump.mouse_state();
            game::aim_at_mouse(&mut state.ship, mouse_state.x(), mouse_state.y(), &globals);

            // Mouse click = accelerate forward
            if mouse_state.left() {
                game::acceleration(&mut state.ship, &globals);
            }

            // Keyboard input (scancodes = physical key positions)
            // AZERTY: Z=forward, Q=left, D=right, A=strafe-left, E=strafe-right
            // Physical: W=Z, A=Q, D=D, Q=A, E=E
            let keyboard = event_pump.keyboard_state();
            use sdl2::keyboard::Scancode;

            // Forward: W (physical) = Z on AZERTY
            if keyboard.is_scancode_pressed(Scancode::W) {
                if globals.ship_impulse_pos {
                    game::boost_forward(&mut state.ship);
                } else {
                    game::acceleration(&mut state.ship, &globals);
                }
            }
            // Rotate left: A (physical) = Q on AZERTY
            if keyboard.is_scancode_pressed(Scancode::A) {
                game::handle_left(&mut state.ship, &globals);
            }
            // Rotate right: D (physical) = D on both
            if keyboard.is_scancode_pressed(Scancode::D) {
                game::handle_right(&mut state.ship, &globals);
            }
            // Strafe left: Q (physical) = A on AZERTY
            if keyboard.is_scancode_pressed(Scancode::Q) {
                game::strafe_left(&mut state.ship);
            }
            // Strafe right: E (physical) = E on both
            if keyboard.is_scancode_pressed(Scancode::E) {
                game::strafe_right(&mut state.ship);
            }

            // Update game state (physics, wrapping, asteroids, etc.)
            game::update_game(&mut state, &mut globals);
        }

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

        renderer.begin_frame();
        game::render_frame(&mut state, &globals, &mut renderer);
        renderer.end_frame(&device, &queue, &view, [0.0, 0.0, 0.0, 1.0]);
        output.present();
    }

    println!("Bye bye!");
}
