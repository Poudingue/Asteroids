pub mod hud;
pub mod world;

use wgpu::util::DeviceExt;

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

pub struct Renderer2D {
    pipeline: wgpu::RenderPipeline,
    screen_size_buffer: wgpu::Buffer,
    screen_size_bind_group: wgpu::BindGroup,
    vertices: Vec<Vertex>,
    pub width: u32,
    pub height: u32,
}


impl Renderer2D {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shape Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shape.wgsl").into()),
        });

        let screen_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Size Buffer"),
            contents: bytemuck::cast_slice(&[width as f32, height as f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Screen Size Bind Group Layout"),
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

        let screen_size_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Screen Size Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
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
                cull_mode: None, // No culling for 2D
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
            pipeline,
            screen_size_buffer,
            screen_size_bind_group,
            vertices: Vec::with_capacity(65536),
            width,
            height,
        }
    }

    /// Update the screen size uniform buffer after a window resize.
    pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        queue.write_buffer(
            &self.screen_size_buffer,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );
    }

    pub fn begin_frame(&mut self) {
        self.vertices.clear();
    }

    fn push_vertex(&mut self, x: f32, y: f32, color: [u8; 4]) {
        self.vertices.push(Vertex {
            position: [x, y],
            color: [
                color[0] as f32 / 255.0,
                color[1] as f32 / 255.0,
                color[2] as f32 / 255.0,
                color[3] as f32 / 255.0,
            ],
        });
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [u8; 4]) {
        let (x0, y0) = (x as f32, y as f32);
        let (x1, y1) = ((x + w) as f32, (y + h) as f32);
        // Two triangles
        self.push_vertex(x0, y0, color);
        self.push_vertex(x1, y0, color);
        self.push_vertex(x1, y1, color);
        self.push_vertex(x0, y0, color);
        self.push_vertex(x1, y1, color);
        self.push_vertex(x0, y1, color);
    }

    pub fn fill_circle(&mut self, cx: f64, cy: f64, radius: f64, color: [u8; 4]) {
        if radius <= 0.0 {
            return;
        }
        let segments = (radius as i32).max(8).min(64) as usize;
        let cx = cx as f32;
        let cy = cy as f32;
        let r = radius as f32;

        for i in 0..segments {
            let angle1 = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
            let angle2 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / (segments as f32);
            self.push_vertex(cx, cy, color);
            self.push_vertex(cx + r * angle1.cos(), cy + r * angle1.sin(), color);
            self.push_vertex(cx + r * angle2.cos(), cy + r * angle2.sin(), color);
        }
    }

    pub fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: [u8; 4]) {
        let segments = (rx.max(ry) as usize).max(8).min(64);
        let cx = cx as f32;
        let cy = cy as f32;
        let rx = rx as f32;
        let ry = ry as f32;

        for i in 0..segments {
            let angle1 = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
            let angle2 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / (segments as f32);
            self.push_vertex(cx, cy, color);
            self.push_vertex(cx + rx * angle1.cos(), cy + ry * angle1.sin(), color);
            self.push_vertex(cx + rx * angle2.cos(), cy + ry * angle2.sin(), color);
        }
    }

    /// Fill a polygon from a list of (i32, i32) points.
    /// Uses ear-clipping triangulation — works for concave and convex polygons.
    pub fn fill_poly(&mut self, points: &[(i32, i32)], color: [u8; 4]) {
        if points.len() < 3 { return; }

        // Find bounding box
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        for p in points {
            min_y = min_y.min(p.1);
            max_y = max_y.max(p.1);
        }

        let n = points.len();

        // Scanline fill with even-odd rule — correct for self-intersecting polygons
        for y in min_y..=max_y {
            let scan_y = y as f32 + 0.5; // center of pixel row
            let mut intersections: Vec<f32> = Vec::new();

            for i in 0..n {
                let j = (i + 1) % n;
                let (x0, y0) = (points[i].0 as f32, points[i].1 as f32);
                let (x1, y1) = (points[j].0 as f32, points[j].1 as f32);

                // Check if edge crosses this scanline
                if (y0 <= scan_y && y1 > scan_y) || (y1 <= scan_y && y0 > scan_y) {
                    // Compute X intersection
                    let t = (scan_y - y0) / (y1 - y0);
                    intersections.push(x0 + t * (x1 - x0));
                }
            }

            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // Fill between pairs (even-odd rule)
            for pair in intersections.chunks(2) {
                if pair.len() == 2 {
                    let x_left = pair[0];
                    let x_right = pair[1];
                    if x_right > x_left {
                        // Draw a horizontal span as two triangles
                        let y_top = y as f32;
                        let y_bot = (y + 1) as f32;
                        self.push_vertex(x_left,  y_top, color);
                        self.push_vertex(x_right, y_top, color);
                        self.push_vertex(x_right, y_bot, color);
                        self.push_vertex(x_left,  y_top, color);
                        self.push_vertex(x_right, y_bot, color);
                        self.push_vertex(x_left,  y_bot, color);
                    }
                }
            }
        }
    }

    /// Draw a polygon outline. Each edge is a thick quad.
    pub fn draw_poly(&mut self, points: &[(i32, i32)], color: [u8; 4], line_width: f32) {
        if points.len() < 2 {
            return;
        }
        let half_w = line_width / 2.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            let (x1, y1) = (points[i].0 as f32, points[i].1 as f32);
            let (x2, y2) = (points[j].0 as f32, points[j].1 as f32);
            self.draw_line_f32(x1, y1, x2, y2, color, half_w);
        }
    }

    /// Draw a thick line between two points.
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: [u8; 4], width: f32) {
        self.draw_line_f32(
            x1 as f32,
            y1 as f32,
            x2 as f32,
            y2 as f32,
            color,
            width / 2.0,
        );
    }

    fn draw_line_f32(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [u8; 4], half_w: f32) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }
        // Perpendicular direction
        let nx = -dy / len * half_w;
        let ny = dx / len * half_w;

        // Quad as two triangles
        self.push_vertex(x1 + nx, y1 + ny, color);
        self.push_vertex(x1 - nx, y1 - ny, color);
        self.push_vertex(x2 - nx, y2 - ny, color);
        self.push_vertex(x1 + nx, y1 + ny, color);
        self.push_vertex(x2 - nx, y2 - ny, color);
        self.push_vertex(x2 + nx, y2 + ny, color);
    }

    /// Plot a single pixel as a 1x1 rect.
    pub fn plot(&mut self, x: i32, y: i32, color: [u8; 4]) {
        self.fill_rect(x, y, 1, 1, color);
    }

    /// Draw a string using SDL2-style bitmap font would go here.
    /// For now, this is a no-op placeholder; the vector font in game.rs handles important text.
    pub fn draw_string(&mut self, _text: &str, _x: i32, _y: i32, _color: [u8; 4]) {
        // Debug text will use the custom vector font or be skipped initially
    }

    pub fn end_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        clear_color: [f64; 4],
    ) {
        if self.vertices.is_empty() {
            // Still need to do the clear
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Clear Encoder"),
            });
            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: clear_color[0],
                                g: clear_color[1],
                                b: clear_color[2],
                                a: clear_color[3],
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }
            queue.submit(std::iter::once(encoder.finish()));
            return;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color[0],
                            g: clear_color[1],
                            b: clear_color[2],
                            a: clear_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.screen_size_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..self.vertices.len() as u32, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
