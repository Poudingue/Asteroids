pub mod hud;
pub mod pipeline;
pub mod textures;
pub mod world;

use textures::{create_msaa_texture, create_offscreen_texture};

use crate::parameters::DEFAULT_MSAA_SAMPLE_COUNT;
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
    pub hdr_enabled: f32, // 0.0 = SDR, 1.0 = HDR
    pub exposure: f32,
    pub max_brightness: f32,
    pub tonemap_variant: f32, // 0=Passthrough, 1=Pseudo-Reinhard, 2=Hard Redirect, 3=Soft Redirect
    pub _padding: f32,        // pad to 48 bytes (3 × vec4)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudUniforms {
    pub screen_width: f32,
    pub screen_height: f32,
    pub brightness_scale: f32,
    pub hdr_enabled: f32,
    pub max_brightness: f32,
    pub tonemap_variant: f32,
    pub exposure: f32,
    pub _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub center: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
    pub falloff_width: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CapsuleInstance {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
    pub _padding: [f32; 3],
}

pub struct Renderer2D {
    world_pipeline: wgpu::RenderPipeline,
    screen_size_buffer: wgpu::Buffer,
    world_bind_group: wgpu::BindGroup,
    offscreen_texture: wgpu::Texture,
    offscreen_view: wgpu::TextureView,
    postprocess_pipeline: wgpu::RenderPipeline,
    postprocess_bind_group: wgpu::BindGroup,
    postprocess_sampler: wgpu::Sampler,
    postprocess_uniform_buffer: wgpu::Buffer,
    surface_format: wgpu::TextureFormat,
    vertices: Vec<Vertex>,
    hud_pipeline: wgpu::RenderPipeline,
    hud_vertices: Vec<Vertex>,
    hud_bind_group: wgpu::BindGroup,
    hud_uniform_buffer: wgpu::Buffer,
    sdf_circle_pipeline: wgpu::RenderPipeline,
    sdf_circle_instances: Vec<CircleInstance>,
    sdf_capsule_pipeline: wgpu::RenderPipeline,
    sdf_capsule_instances: Vec<CapsuleInstance>,
    sdf_bind_group: wgpu::BindGroup,
    msaa_sample_count: u32,
    msaa_offscreen_texture: Option<wgpu::Texture>,
    msaa_offscreen_view: Option<wgpu::TextureView>,
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
        // --- World pipeline (renders into Rgba16Float offscreen texture) ---
        let screen_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Size Buffer"),
            contents: bytemuck::cast_slice(&[width as f32, height as f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let world_bind_group_layout = pipeline::create_screen_size_bind_group_layout(device);

        let world_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("World Bind Group"),
            layout: &world_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding(),
            }],
        });

        let world_pipeline = pipeline::create_world_pipeline(
            device,
            &world_bind_group_layout,
            DEFAULT_MSAA_SAMPLE_COUNT,
        );

        // --- Offscreen texture ---
        let offscreen_texture = create_offscreen_texture(device, width, height);
        let offscreen_view = offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // --- MSAA texture (None when DEFAULT_MSAA_SAMPLE_COUNT == 1) ---
        let msaa_offscreen_texture = create_msaa_texture(
            device,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
            DEFAULT_MSAA_SAMPLE_COUNT,
        );
        let msaa_offscreen_view = msaa_offscreen_texture
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        // --- Postprocess pipeline (renders fullscreen triangle to swapchain) ---
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
                    hdr_enabled: 0.0,
                    exposure: 1.0,
                    max_brightness: 1000.0,
                    tonemap_variant: 0.0,
                    _padding: 0.0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let postprocess_bind_group_layout = pipeline::create_postprocess_bind_group_layout(device);

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

        let postprocess_pipeline = pipeline::create_postprocess_pipeline(
            device,
            surface_format,
            &postprocess_bind_group_layout,
        );

        // --- SDF circle + capsule pipelines (render into Rgba16Float offscreen texture) ---
        let sdf_bind_group_layout = pipeline::create_screen_size_bind_group_layout(device);

        let sdf_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SDF Bind Group"),
            layout: &sdf_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding(),
            }],
        });

        let sdf_circle_pipeline =
            pipeline::create_sdf_circle_pipeline(device, &sdf_bind_group_layout);
        let sdf_capsule_pipeline =
            pipeline::create_sdf_capsule_pipeline(device, &sdf_bind_group_layout);

        // --- HUD pipeline (renders directly to swapchain, alpha blending, no tonemapping) ---
        let hud_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("HUD Uniform Buffer"),
            contents: bytemuck::cast_slice(&[HudUniforms {
                screen_width: width as f32,
                screen_height: height as f32,
                brightness_scale: 1.0,
                hdr_enabled: 0.0,
                max_brightness: 1000.0,
                tonemap_variant: 3.0,
                exposure: 1.0,
                _padding: 0.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let hud_bind_group_layout = pipeline::create_hud_bind_group_layout(device);

        let hud_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("HUD Bind Group"),
            layout: &hud_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: hud_uniform_buffer.as_entire_binding(),
            }],
        });

        let hud_pipeline =
            pipeline::create_hud_pipeline(device, surface_format, &hud_bind_group_layout);

        Self {
            world_pipeline,
            screen_size_buffer,
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
            hud_uniform_buffer,
            sdf_circle_pipeline,
            sdf_circle_instances: Vec::with_capacity(4096),
            sdf_capsule_pipeline,
            sdf_capsule_instances: Vec::with_capacity(2048),
            sdf_bind_group,
            msaa_sample_count: DEFAULT_MSAA_SAMPLE_COUNT,
            msaa_offscreen_texture,
            msaa_offscreen_view,
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

        // Recreate MSAA texture at new resolution
        self.msaa_offscreen_texture = create_msaa_texture(
            device,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
            self.msaa_sample_count,
        );
        self.msaa_offscreen_view = self
            .msaa_offscreen_texture
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        // Recreate postprocess bind group (it references the old view)
        let postprocess_bind_group_layout = pipeline::create_postprocess_bind_group_layout(device);

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

    /// Recreate the world pipeline and MSAA texture with a new sample count.
    /// Call this when the user changes MSAA from the pause menu.
    pub fn set_msaa_sample_count(&mut self, device: &wgpu::Device, sample_count: u32) {
        if sample_count == self.msaa_sample_count {
            return;
        }
        self.msaa_sample_count = sample_count;

        // Recreate MSAA texture (None when sample_count == 1)
        self.msaa_offscreen_texture = create_msaa_texture(
            device,
            self.width,
            self.height,
            wgpu::TextureFormat::Rgba16Float,
            sample_count,
        );
        self.msaa_offscreen_view = self
            .msaa_offscreen_texture
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        // Recreate world pipeline with new sample count
        let world_bind_group_layout = pipeline::create_screen_size_bind_group_layout(device);
        self.world_pipeline =
            pipeline::create_world_pipeline(device, &world_bind_group_layout, sample_count);
    }

    /// Recreate the postprocess and HUD pipelines with a new swapchain format.
    /// Call this when the surface format changes (e.g. HDR toggle).
    pub fn recreate_surface_pipelines(
        &mut self,
        device: &wgpu::Device,
        new_format: wgpu::TextureFormat,
    ) {
        self.surface_format = new_format;

        // Recreate postprocess pipeline
        let postprocess_bind_group_layout = pipeline::create_postprocess_bind_group_layout(device);
        self.postprocess_pipeline = pipeline::create_postprocess_pipeline(
            device,
            new_format,
            &postprocess_bind_group_layout,
        );

        // Recreate HUD pipeline
        let hud_bind_group_layout = pipeline::create_hud_bind_group_layout(device);
        self.hud_pipeline =
            pipeline::create_hud_pipeline(device, new_format, &hud_bind_group_layout);

        // Rebind HUD bind group with the new layout (uses dedicated hud_uniform_buffer)
        self.hud_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("HUD Bind Group"),
            layout: &hud_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.hud_uniform_buffer.as_entire_binding(),
            }],
        });
    }

    pub fn update_postprocess_uniforms(&self, queue: &wgpu::Queue, uniforms: &PostProcessUniforms) {
        queue.write_buffer(
            &self.postprocess_uniform_buffer,
            0,
            bytemuck::cast_slice(std::slice::from_ref(uniforms)),
        );
    }

    pub fn update_hud_uniforms(&self, queue: &wgpu::Queue, uniforms: &HudUniforms) {
        queue.write_buffer(
            &self.hud_uniform_buffer,
            0,
            bytemuck::cast_slice(std::slice::from_ref(uniforms)),
        );
    }

    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.hud_vertices.clear();
        self.sdf_circle_instances.clear();
        self.sdf_capsule_instances.clear();
    }

    pub fn push_circle_instance(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        color: [f32; 4],
        falloff_width: f32,
    ) {
        if radius <= 0.0 {
            return;
        }
        self.sdf_circle_instances.push(CircleInstance {
            center: [cx, cy],
            radius,
            color,
            falloff_width,
        });
    }

    pub fn push_capsule_instance(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        radius: f32,
        color: [f32; 4],
    ) {
        if radius <= 0.0 {
            return;
        }
        self.sdf_capsule_instances.push(CapsuleInstance {
            p0: [x0, y0],
            p1: [x1, y1],
            radius,
            color,
            _padding: [0.0; 3],
        });
    }

    // ---- Internal geometry helpers (write to an arbitrary target Vec<Vertex>) ----

    fn push_vertex_to(target: &mut Vec<Vertex>, x: f32, y: f32, color: [f32; 4]) {
        target.push(Vertex {
            position: [x, y],
            color,
        });
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
        if radius <= 0.0 {
            return;
        }
        let segments = (radius as i32).clamp(8, 64) as usize;
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

    fn geo_fill_ellipse(
        target: &mut Vec<Vertex>,
        cx: i32,
        cy: i32,
        rx: i32,
        ry: i32,
        color: [f32; 4],
    ) {
        let segments = (rx.max(ry) as usize).clamp(8, 64);
        let cx = cx as f32;
        let cy = cy as f32;
        let rx = rx as f32;
        let ry = ry as f32;
        for i in 0..segments {
            let angle1 = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
            let angle2 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / (segments as f32);
            Self::push_vertex_to(target, cx, cy, color);
            Self::push_vertex_to(
                target,
                cx + rx * angle1.cos(),
                cy + ry * angle1.sin(),
                color,
            );
            Self::push_vertex_to(
                target,
                cx + rx * angle2.cos(),
                cy + ry * angle2.sin(),
                color,
            );
        }
    }

    fn geo_fill_poly(target: &mut Vec<Vertex>, points: &[(i32, i32)], color: [f32; 4]) {
        if points.len() < 3 {
            return;
        }
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
                        Self::push_vertex_to(target, x_left, y_top, color);
                        Self::push_vertex_to(target, x_right, y_top, color);
                        Self::push_vertex_to(target, x_right, y_bot, color);
                        Self::push_vertex_to(target, x_left, y_top, color);
                        Self::push_vertex_to(target, x_right, y_bot, color);
                        Self::push_vertex_to(target, x_left, y_bot, color);
                    }
                }
            }
        }
    }

    fn geo_draw_line_f32(
        target: &mut Vec<Vertex>,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        half_w: f32,
    ) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }
        let nx = -dy / len * half_w;
        let ny = dx / len * half_w;
        Self::push_vertex_to(target, x1 + nx, y1 + ny, color);
        Self::push_vertex_to(target, x1 - nx, y1 - ny, color);
        Self::push_vertex_to(target, x2 - nx, y2 - ny, color);
        Self::push_vertex_to(target, x1 + nx, y1 + ny, color);
        Self::push_vertex_to(target, x2 - nx, y2 - ny, color);
        Self::push_vertex_to(target, x2 + nx, y2 + ny, color);
    }

    fn geo_draw_poly(
        target: &mut Vec<Vertex>,
        points: &[(i32, i32)],
        color: [f32; 4],
        line_width: f32,
    ) {
        if points.len() < 2 {
            return;
        }
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
        Self::geo_draw_line_f32(
            &mut self.vertices,
            x1 as f32,
            y1 as f32,
            x2 as f32,
            y2 as f32,
            color,
            width / 2.0,
        );
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

    pub fn hud_draw_line(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: [f32; 4],
        width: f32,
    ) {
        Self::geo_draw_line_f32(
            &mut self.hud_vertices,
            x1 as f32,
            y1 as f32,
            x2 as f32,
            y2 as f32,
            color,
            width / 2.0,
        );
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
        // When MSAA is enabled, render into the multisampled texture and resolve to offscreen.
        // When MSAA is off (sample_count==1), render directly to offscreen.
        {
            let load_op = wgpu::LoadOp::Clear(wgpu::Color {
                r: clear_color[0],
                g: clear_color[1],
                b: clear_color[2],
                a: clear_color[3],
            });

            let (target_view, resolve_target) =
                if let Some(ref msaa_view) = self.msaa_offscreen_view {
                    (msaa_view, Some(&self.offscreen_view))
                } else {
                    (&self.offscreen_view, None)
                };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("World Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target,
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
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
        let has_sdf =
            !self.sdf_circle_instances.is_empty() || !self.sdf_capsule_instances.is_empty();
        if has_sdf {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SDF Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.offscreen_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve world geometry
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !self.sdf_circle_instances.is_empty() {
                let instance_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("SDF Circle Instance Buffer"),
                        contents: bytemuck::cast_slice(&self.sdf_circle_instances),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                render_pass.set_pipeline(&self.sdf_circle_pipeline);
                render_pass.set_bind_group(0, &self.sdf_bind_group, &[]);
                render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
                render_pass.draw(0..6, 0..self.sdf_circle_instances.len() as u32);
            }

            if !self.sdf_capsule_instances.is_empty() {
                let capsule_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("SDF Capsule Instance Buffer"),
                    contents: bytemuck::cast_slice(&self.sdf_capsule_instances),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                render_pass.set_pipeline(&self.sdf_capsule_pipeline);
                render_pass.set_bind_group(0, &self.sdf_bind_group, &[]);
                render_pass.set_vertex_buffer(0, capsule_buffer.slice(..));
                render_pass.draw(0..6, 0..self.sdf_capsule_instances.len() as u32);
            }
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

#[cfg(test)]
mod hud_uniforms_tests {
    use super::*;
    #[test]
    fn hud_uniforms_size_is_32_bytes() {
        assert_eq!(std::mem::size_of::<HudUniforms>(), 32);
    }
}

#[cfg(test)]
mod circle_instance_tests {
    use super::*;
    #[test]
    fn circle_instance_size_unchanged() {
        assert_eq!(std::mem::size_of::<CircleInstance>(), 32);
    }
    #[test]
    fn circle_instance_falloff_stored() {
        let c = CircleInstance {
            center: [10.0, 20.0],
            radius: 5.0,
            color: [1.0, 0.5, 0.0, 1.0],
            falloff_width: 0.2,
        };
        assert!((c.falloff_width - 0.2).abs() < 1e-6);
    }
}
