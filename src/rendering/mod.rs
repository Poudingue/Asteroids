pub mod hud;
pub mod world;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PostProcessUniforms {
    pub game_exposure: f32,
    pub add_color_r: f32,
    pub add_color_g: f32,
    pub add_color_b: f32,
    pub mul_color_r: f32,
    pub mul_color_g: f32,
    pub mul_color_b: f32,
    pub _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub center: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
    pub _padding: f32,
}

impl CircleInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        2 => Float32x2,   // center
        3 => Float32,     // radius
        4 => Float32x4,   // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CircleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct Renderer2D {
    world_pipeline: wgpu::RenderPipeline,
    screen_size_buffer: wgpu::Buffer,
    // Kept for future zoom control (Phase 1.2+)
    #[allow(dead_code)]
    zoom_factor_buffer: wgpu::Buffer,
    world_bind_group: wgpu::BindGroup,
    offscreen_texture: wgpu::Texture,
    offscreen_view: wgpu::TextureView,
    postprocess_pipeline: wgpu::RenderPipeline,
    postprocess_bind_group: wgpu::BindGroup,
    postprocess_sampler: wgpu::Sampler,
    postprocess_uniform_buffer: wgpu::Buffer,
    // Kept for use in resize() when rebuilding pipelines
    #[allow(dead_code)]
    surface_format: wgpu::TextureFormat,
    vertices: Vec<Vertex>,
    hud_pipeline: wgpu::RenderPipeline,
    hud_vertices: Vec<Vertex>,
    hud_bind_group: wgpu::BindGroup,
    sdf_circle_pipeline: wgpu::RenderPipeline,
    sdf_circle_instances: Vec<CircleInstance>,
    sdf_bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
}

fn create_offscreen_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Offscreen HDR Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

impl Renderer2D {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        // --- World shader + pipeline (renders into Rgba16Float offscreen texture) ---
        let world_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("World Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/world.wgsl").into()),
        });

        let screen_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Size Buffer"),
            contents: bytemuck::cast_slice(&[width as f32, height as f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let zoom_factor_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Zoom Factor Buffer"),
            contents: bytemuck::cast_slice(&[1.0f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let world_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("World Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let world_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("World Bind Group"),
            layout: &world_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: screen_size_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: zoom_factor_buffer.as_entire_binding(),
                },
            ],
        });

        let world_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("World Pipeline Layout"),
                bind_group_layouts: &[&world_bind_group_layout],
                push_constant_ranges: &[],
            });

        let world_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("World Pipeline"),
            layout: Some(&world_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &world_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &world_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // --- Offscreen texture ---
        let offscreen_texture = create_offscreen_texture(device, width, height);
        let offscreen_view =
            offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // --- Postprocess shader + pipeline (renders fullscreen triangle to swapchain) ---
        let postprocess_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Postprocess Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/postprocess.wgsl").into()),
        });

        let postprocess_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Postprocess Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Neutral postprocess uniforms: exposure=1.0, add=0, mul=1
        let postprocess_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Postprocess Uniform Buffer"),
                contents: bytemuck::cast_slice(&[PostProcessUniforms {
                    game_exposure: 1.0,
                    add_color_r: 0.0,
                    add_color_g: 0.0,
                    add_color_b: 0.0,
                    mul_color_r: 1.0,
                    mul_color_g: 1.0,
                    mul_color_b: 1.0,
                    _padding: 0.0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let postprocess_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Postprocess Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let postprocess_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Postprocess Bind Group"),
            layout: &postprocess_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&postprocess_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: postprocess_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let postprocess_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Postprocess Pipeline Layout"),
                bind_group_layouts: &[&postprocess_bind_group_layout],
                push_constant_ranges: &[],
            });

        let postprocess_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Postprocess Pipeline"),
                layout: Some(&postprocess_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &postprocess_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[], // fullscreen triangle from vertex_index
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &postprocess_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        // --- SDF circle shader + pipeline (renders into Rgba16Float offscreen texture) ---
        let sdf_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SDF Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sdf.wgsl").into()),
        });

        let sdf_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("SDF Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let sdf_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SDF Bind Group"),
            layout: &sdf_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding(),
            }],
        });

        let sdf_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("SDF Pipeline Layout"),
                bind_group_layouts: &[&sdf_bind_group_layout],
                push_constant_ranges: &[],
            });

        let sdf_circle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SDF Circle Pipeline"),
            layout: Some(&sdf_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sdf_shader,
                entry_point: Some("vs_circle"),
                buffers: &[CircleInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sdf_shader,
                entry_point: Some("fs_circle"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // --- HUD shader + pipeline (renders directly to swapchain, alpha blending, no tonemapping) ---
        let hud_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("HUD Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/hud.wgsl").into()),
        });

        let hud_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("HUD Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let hud_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("HUD Bind Group"),
            layout: &hud_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding(),
            }],
        });

        let hud_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("HUD Pipeline Layout"),
                bind_group_layouts: &[&hud_bind_group_layout],
                push_constant_ranges: &[],
            });

        let hud_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("HUD Pipeline"),
            layout: Some(&hud_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &hud_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &hud_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            world_pipeline,
            screen_size_buffer,
            zoom_factor_buffer,
            world_bind_group,
            offscreen_texture,
            offscreen_view,
            postprocess_pipeline,
            postprocess_bind_group,
            postprocess_sampler,
            postprocess_uniform_buffer,
            surface_format,
            vertices: Vec::with_capacity(65536),
            hud_pipeline,
            hud_vertices: Vec::with_capacity(16384),
            hud_bind_group,
            sdf_circle_pipeline,
            sdf_circle_instances: Vec::with_capacity(4096),
            sdf_bind_group,
            width,
            height,
        }
    }

    /// Update the screen size uniform buffer and recreate offscreen texture after a window resize.
    pub fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        queue.write_buffer(
            &self.screen_size_buffer,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );

        // Recreate offscreen texture at new resolution
        self.offscreen_texture = create_offscreen_texture(device, width, height);
        self.offscreen_view = self
            .offscreen_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Recreate postprocess bind group (it references the old view)
        let postprocess_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Postprocess Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        self.postprocess_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Postprocess Bind Group"),
            layout: &postprocess_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.postprocess_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.postprocess_uniform_buffer.as_entire_binding(),
                },
            ],
        });
    }

    pub fn update_postprocess_uniforms(&self, queue: &wgpu::Queue, uniforms: &PostProcessUniforms) {
        queue.write_buffer(
            &self.postprocess_uniform_buffer,
            0,
            bytemuck::cast_slice(std::slice::from_ref(uniforms)),
        );
    }

    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.hud_vertices.clear();
        self.sdf_circle_instances.clear();
    }

    pub fn push_circle_instance(&mut self, cx: f32, cy: f32, radius: f32, color: [f32; 4]) {
        if radius <= 0.0 { return; }
        self.sdf_circle_instances.push(CircleInstance { center: [cx, cy], radius, color, _padding: 0.0 });
    }

    // ---- Internal geometry helpers (write to an arbitrary target Vec<Vertex>) ----

    fn push_vertex_to(target: &mut Vec<Vertex>, x: f32, y: f32, color: [f32; 4]) {
        target.push(Vertex { position: [x, y], color });
    }

    fn geo_fill_rect(target: &mut Vec<Vertex>, x: i32, y: i32, w: i32, h: i32, color: [f32; 4]) {
        let (x0, y0) = (x as f32, y as f32);
        let (x1, y1) = ((x + w) as f32, (y + h) as f32);
        Self::push_vertex_to(target, x0, y0, color);
        Self::push_vertex_to(target, x1, y0, color);
        Self::push_vertex_to(target, x1, y1, color);
        Self::push_vertex_to(target, x0, y0, color);
        Self::push_vertex_to(target, x1, y1, color);
        Self::push_vertex_to(target, x0, y1, color);
    }

    fn geo_fill_circle(target: &mut Vec<Vertex>, cx: f64, cy: f64, radius: f64, color: [f32; 4]) {
        if radius <= 0.0 { return; }
        let segments = (radius as i32).max(8).min(64) as usize;
        let cx = cx as f32;
        let cy = cy as f32;
        let r = radius as f32;
        for i in 0..segments {
            let angle1 = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
            let angle2 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / (segments as f32);
            Self::push_vertex_to(target, cx, cy, color);
            Self::push_vertex_to(target, cx + r * angle1.cos(), cy + r * angle1.sin(), color);
            Self::push_vertex_to(target, cx + r * angle2.cos(), cy + r * angle2.sin(), color);
        }
    }

    fn geo_fill_ellipse(target: &mut Vec<Vertex>, cx: i32, cy: i32, rx: i32, ry: i32, color: [f32; 4]) {
        let segments = (rx.max(ry) as usize).max(8).min(64);
        let cx = cx as f32;
        let cy = cy as f32;
        let rx = rx as f32;
        let ry = ry as f32;
        for i in 0..segments {
            let angle1 = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
            let angle2 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / (segments as f32);
            Self::push_vertex_to(target, cx, cy, color);
            Self::push_vertex_to(target, cx + rx * angle1.cos(), cy + ry * angle1.sin(), color);
            Self::push_vertex_to(target, cx + rx * angle2.cos(), cy + ry * angle2.sin(), color);
        }
    }

    fn geo_fill_poly(target: &mut Vec<Vertex>, points: &[(i32, i32)], color: [f32; 4]) {
        if points.len() < 3 { return; }
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        for p in points {
            min_y = min_y.min(p.1);
            max_y = max_y.max(p.1);
        }
        let n = points.len();
        for y in min_y..=max_y {
            let scan_y = y as f32 + 0.5;
            let mut intersections: Vec<f32> = Vec::new();
            for i in 0..n {
                let j = (i + 1) % n;
                let (x0, y0) = (points[i].0 as f32, points[i].1 as f32);
                let (x1, y1) = (points[j].0 as f32, points[j].1 as f32);
                if (y0 <= scan_y && y1 > scan_y) || (y1 <= scan_y && y0 > scan_y) {
                    let t = (scan_y - y0) / (y1 - y0);
                    intersections.push(x0 + t * (x1 - x0));
                }
            }
            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for pair in intersections.chunks(2) {
                if pair.len() == 2 {
                    let x_left = pair[0];
                    let x_right = pair[1];
                    if x_right > x_left {
                        let y_top = y as f32;
                        let y_bot = (y + 1) as f32;
                        Self::push_vertex_to(target, x_left,  y_top, color);
                        Self::push_vertex_to(target, x_right, y_top, color);
                        Self::push_vertex_to(target, x_right, y_bot, color);
                        Self::push_vertex_to(target, x_left,  y_top, color);
                        Self::push_vertex_to(target, x_right, y_bot, color);
                        Self::push_vertex_to(target, x_left,  y_bot, color);
                    }
                }
            }
        }
    }

    fn geo_draw_line_f32(target: &mut Vec<Vertex>, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 4], half_w: f32) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 { return; }
        let nx = -dy / len * half_w;
        let ny = dx / len * half_w;
        Self::push_vertex_to(target, x1 + nx, y1 + ny, color);
        Self::push_vertex_to(target, x1 - nx, y1 - ny, color);
        Self::push_vertex_to(target, x2 - nx, y2 - ny, color);
        Self::push_vertex_to(target, x1 + nx, y1 + ny, color);
        Self::push_vertex_to(target, x2 - nx, y2 - ny, color);
        Self::push_vertex_to(target, x2 + nx, y2 + ny, color);
    }

    fn geo_draw_poly(target: &mut Vec<Vertex>, points: &[(i32, i32)], color: [f32; 4], line_width: f32) {
        if points.len() < 2 { return; }
        let half_w = line_width / 2.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            let (x1, y1) = (points[i].0 as f32, points[i].1 as f32);
            let (x2, y2) = (points[j].0 as f32, points[j].1 as f32);
            Self::geo_draw_line_f32(target, x1, y1, x2, y2, color, half_w);
        }
    }

    // ---- World rendering methods (write to self.vertices → HDR offscreen) ----

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [f32; 4]) {
        Self::geo_fill_rect(&mut self.vertices, x, y, w, h, color);
    }

    pub fn fill_circle(&mut self, cx: f64, cy: f64, radius: f64, color: [f32; 4]) {
        Self::geo_fill_circle(&mut self.vertices, cx, cy, radius, color);
    }

    pub fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: [f32; 4]) {
        Self::geo_fill_ellipse(&mut self.vertices, cx, cy, rx, ry, color);
    }

    /// Fill a polygon from a list of (i32, i32) points.
    /// Uses scanline fill with even-odd rule — correct for self-intersecting polygons.
    pub fn fill_poly(&mut self, points: &[(i32, i32)], color: [f32; 4]) {
        Self::geo_fill_poly(&mut self.vertices, points, color);
    }

    /// Draw a polygon outline. Each edge is a thick quad.
    pub fn draw_poly(&mut self, points: &[(i32, i32)], color: [f32; 4], line_width: f32) {
        Self::geo_draw_poly(&mut self.vertices, points, color, line_width);
    }

    /// Draw a thick line between two points.
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: [f32; 4], width: f32) {
        Self::geo_draw_line_f32(&mut self.vertices, x1 as f32, y1 as f32, x2 as f32, y2 as f32, color, width / 2.0);
    }

    /// Plot a single pixel as a 1x1 rect.
    pub fn plot(&mut self, x: i32, y: i32, color: [f32; 4]) {
        self.fill_rect(x, y, 1, 1, color);
    }

    /// Draw a string using SDL2-style bitmap font would go here.
    /// For now, this is a no-op placeholder; the vector font in game.rs handles important text.
    pub fn draw_string(&mut self, _text: &str, _x: i32, _y: i32, _color: [f32; 4]) {
        // Debug text will use the custom vector font or be skipped initially
    }

    // ---- HUD rendering methods (write to self.hud_vertices → swapchain, no tonemapping) ----

    pub fn hud_fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [f32; 4]) {
        Self::geo_fill_rect(&mut self.hud_vertices, x, y, w, h, color);
    }

    pub fn hud_fill_poly(&mut self, points: &[(i32, i32)], color: [f32; 4]) {
        Self::geo_fill_poly(&mut self.hud_vertices, points, color);
    }

    pub fn hud_draw_poly(&mut self, points: &[(i32, i32)], color: [f32; 4], line_width: f32) {
        Self::geo_draw_poly(&mut self.hud_vertices, points, color, line_width);
    }

    pub fn hud_fill_circle(&mut self, cx: f64, cy: f64, radius: f64, color: [f32; 4]) {
        Self::geo_fill_circle(&mut self.hud_vertices, cx, cy, radius, color);
    }

    pub fn hud_fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: [f32; 4]) {
        Self::geo_fill_ellipse(&mut self.hud_vertices, cx, cy, rx, ry, color);
    }

    pub fn hud_draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: [f32; 4], width: f32) {
        Self::geo_draw_line_f32(&mut self.hud_vertices, x1 as f32, y1 as f32, x2 as f32, y2 as f32, color, width / 2.0);
    }

    pub fn hud_plot(&mut self, x: i32, y: i32, color: [f32; 4]) {
        self.hud_fill_rect(x, y, 1, 1, color);
    }

    pub fn end_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        clear_color: [f64; 4],
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // --- Pass 1: Render world geometry into offscreen Rgba16Float texture ---
        {
            let load_op = wgpu::LoadOp::Clear(wgpu::Color {
                r: clear_color[0],
                g: clear_color[1],
                b: clear_color[2],
                a: clear_color[3],
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("World Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.offscreen_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !self.vertices.is_empty() {
                let vertex_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&self.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

                render_pass.set_pipeline(&self.world_pipeline);
                render_pass.set_bind_group(0, &self.world_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..self.vertices.len() as u32, 0..1);
            }
        }

        // === Pass 2: SDF entities -> offscreen Rgba16Float (no clear, load existing) ===
        if !self.sdf_circle_instances.is_empty() {
            let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("SDF Circle Instance Buffer"),
                contents: bytemuck::cast_slice(&self.sdf_circle_instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SDF Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.offscreen_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,  // Preserve world geometry
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.sdf_circle_pipeline);
            render_pass.set_bind_group(0, &self.sdf_bind_group, &[]);
            render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
            render_pass.draw(0..6, 0..self.sdf_circle_instances.len() as u32);
        }

        // --- Pass 3: Blit offscreen texture to swapchain via fullscreen triangle ---
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Postprocess Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.postprocess_pipeline);
            render_pass.set_bind_group(0, &self.postprocess_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // === Pass 4: HUD -> swapchain (no tonemapping, alpha blending) ===
        if !self.hud_vertices.is_empty() {
            let hud_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("HUD Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.hud_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("HUD Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve post-process output
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.hud_pipeline);
            render_pass.set_bind_group(0, &self.hud_bind_group, &[]);
            render_pass.set_vertex_buffer(0, hud_vertex_buffer.slice(..));
            render_pass.draw(0..self.hud_vertices.len() as u32, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
