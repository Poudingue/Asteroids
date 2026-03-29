# Phase 1: Rendering Pipeline Overhaul — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the single-pass CPU-driven rendering with a multi-pass GPU pipeline: offscreen HDR texture, SDF instanced entities, post-process tonemapping, and proper anti-aliasing.

**Architecture:** 4-pass pipeline (world polygons → SDF entities → post-process → HUD) with Rgba16Float offscreen texture, pre-allocated buffers, compile-time tonemapping variants, and toggleable MSAA + SDF AA.

**Tech Stack:** Rust, wgpu 24.x (Vulkan backend), WGSL shaders, SDL2

---

## Current Architecture Summary

The existing renderer (`Renderer2D` in `src/rendering/mod.rs`) is a single-pass CPU-driven pipeline:
- One `Vec<Vertex>` buffer (capacity 65536), re-created as a GPU buffer each frame
- One render pipeline targeting the swapchain surface directly
- Vertex format: `position: [f32; 2]`, `color: [f32; 4]` (converted from `[u8; 4]` internally)
- Color pipeline: `HdrColor` → `intensify(hdr_exposure)` → `rgb_of_hdr(add_color, mul_color, game_exposure)` → `[u8; 4]` → `push_vertex` divides by 255 to `[f32; 4]`
- Single shader (`shape.wgsl`): pixel-to-NDC vertex transform, passthrough fragment
- `fill_circle` = CPU fan triangulation (8–64 segments), `fill_poly` = CPU scanline fill
- All geometry (world + HUD + pause menu) goes into the same vertex buffer
- `begin_frame()` clears vertices, `end_frame()` creates buffer + submits single draw call

---

## Task 1: Offscreen Texture + Post-Process Pass

**Goal:** Render the existing pipeline into an `Rgba16Float` offscreen texture, then blit it to the swapchain via a fullscreen-quad post-process pass. Output must be visually identical to V1.

**Commit message:** `feat(render): offscreen Rgba16Float + post-process blit pass`

### Step 1.1: Create `src/shaders/world.wgsl`

- [ ] Create `src/shaders/world.wgsl` with the same vertex/fragment as current `shape.wgsl`, but targeting `Rgba16Float` output.

**File: `src/shaders/world.wgsl`** (new file)
```wgsl
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;

// Hardcoded to 1.0 for Phase 1. Phase 3 (Camera & Zoom) wires this to the camera system.
@group(0) @binding(1) var<uniform> zoom_factor: f32;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Convert pixel coords to NDC: x: [0, width] -> [-1, 1], y: [0, height] -> [-1, 1]
    out.position = vec4<f32>(
        (in.position.x / screen_size.x) * 2.0 - 1.0,
        (in.position.y / screen_size.y) * 2.0 - 1.0,
        0.0,
        1.0
    );
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
```

- [ ] Run `cargo check` to verify the project still compiles (shader is not yet loaded).

### Step 1.2: Create `src/shaders/postprocess.wgsl` (initial passthrough)

- [ ] Create `src/shaders/postprocess.wgsl` — a fullscreen triangle that samples the offscreen texture and outputs unchanged.

**File: `src/shaders/postprocess.wgsl`** (new file)
```wgsl
// Post-process pass: samples offscreen Rgba16Float texture and outputs to swapchain.
// Phase 1 Task 1: passthrough (no tonemapping yet).
// Phase 1 Task 2: adds faithful tonemapping.
// Phase 1 Task 3: adds variant selection.

// Tonemapping variant selection (compile-time const).
// 0 = passthrough (Task 1), 1 = faithful (Task 2), 2 = spectral_bleed, 3 = ACES, 4 = Reinhard
const TONEMAP_VARIANT: u32 = 0u;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle: 3 vertices that cover the entire screen.
// No vertex buffer needed — vertex_index generates positions.
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Triangle that covers [-1,1] clip space:
    // vertex 0: (-1, -1), vertex 1: (3, -1), vertex 2: (-1, 3)
    let x = f32(i32(vertex_index & 1u)) * 4.0 - 1.0;
    let y = f32(i32(vertex_index >> 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    // UV: map clip [-1,1] to texture [0,1], Y-flipped for wgpu convention
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@group(0) @binding(0) var offscreen_texture: texture_2d<f32>;
@group(0) @binding(1) var offscreen_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(offscreen_texture, offscreen_sampler, in.uv);
    // Task 1: passthrough — no tonemapping
    return hdr_color;
}
```

- [ ] Run `cargo check`.

### Step 1.3: Add offscreen texture and post-process pipeline to `Renderer2D`

- [ ] Modify `src/rendering/mod.rs`: add offscreen texture, texture view, sampler, post-process pipeline, and bind group fields to `Renderer2D`.

**Add to `Renderer2D` struct** (in `src/rendering/mod.rs`):
```rust
pub struct Renderer2D {
    // Existing fields
    pipeline: wgpu::RenderPipeline,           // renamed: world_pipeline
    screen_size_buffer: wgpu::Buffer,
    screen_size_bind_group: wgpu::BindGroup,
    vertices: Vec<Vertex>,
    pub width: u32,
    pub height: u32,

    // New fields for offscreen rendering
    offscreen_texture: wgpu::Texture,
    offscreen_view: wgpu::TextureView,
    postprocess_pipeline: wgpu::RenderPipeline,
    postprocess_bind_group: wgpu::BindGroup,
    postprocess_sampler: wgpu::Sampler,
    surface_format: wgpu::TextureFormat,
    // zoom_factor uniform (hardcoded 1.0 for Phase 1)
    zoom_factor_buffer: wgpu::Buffer,
}
```

**Add helper function** to create the offscreen texture (called from `new` and `resize`):
```rust
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
```

- [ ] Run `cargo check`.

### Step 1.4: Update `Renderer2D::new()` to create offscreen resources

- [ ] Modify `Renderer2D::new()` to:
  1. Rename the existing pipeline to `world_pipeline` internally (keep same behavior).
  2. Load `world.wgsl` instead of `shape.wgsl` for the world pipeline.
  3. Change the world pipeline's color target format from `surface_format` to `Rgba16Float`.
  4. Create the offscreen texture + view + sampler.
  5. Create a zoom_factor buffer (uniform, initialized to `1.0f32`).
  6. Update the world pipeline bind group layout to include both screen_size (binding 0) and zoom_factor (binding 1).
  7. Create the post-process shader module from `postprocess.wgsl`.
  8. Create the post-process bind group layout with texture (binding 0) + sampler (binding 1).
  9. Create the post-process pipeline targeting `surface_format`.
  10. Create the post-process bind group.

**World pipeline bind group layout update** (replaces current single-entry layout):
```rust
let world_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
```

**Post-process bind group layout:**
```rust
let postprocess_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Post-Process Bind Group Layout"),
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
    ],
});
```

**Post-process pipeline creation:**
```rust
let postprocess_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("Post-Process Shader"),
    source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/postprocess.wgsl").into()),
});

let postprocess_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("Post-Process Pipeline Layout"),
    bind_group_layouts: &[&postprocess_bind_group_layout],
    push_constant_ranges: &[],
});

let postprocess_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Post-Process Pipeline"),
    layout: Some(&postprocess_pipeline_layout),
    vertex: wgpu::VertexState {
        module: &postprocess_shader,
        entry_point: Some("vs_main"),
        buffers: &[],  // No vertex buffer — fullscreen triangle from vertex_index
        compilation_options: Default::default(),
    },
    fragment: Some(wgpu::FragmentState {
        module: &postprocess_shader,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
            format: surface_format,
            blend: None,  // Post-process: no blending
            write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: Default::default(),
    }),
    primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        ..Default::default()
    },
    depth_stencil: None,
    multisample: wgpu::MultisampleState::default(),
    multiview: None,
    cache: None,
});
```

**Sampler creation:**
```rust
let postprocess_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
    label: Some("Post-Process Sampler"),
    mag_filter: wgpu::FilterMode::Linear,
    min_filter: wgpu::FilterMode::Linear,
    ..Default::default()
});
```

- [ ] Run `cargo check`.

### Step 1.5: Update `end_frame()` for two-pass rendering

- [ ] Modify `Renderer2D::end_frame()` to:
  1. **Pass 1:** Render world vertices into `offscreen_view` (clear to black, use world pipeline).
  2. **Pass 2:** Render fullscreen triangle to swapchain `view` (use postprocess pipeline + bind group, draw 3 vertices).

**Updated `end_frame` body** (replaces the current implementation):
```rust
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

    // === Pass 1: World geometry -> offscreen Rgba16Float ===
    if !self.vertices.is_empty() {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("World Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("World Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.offscreen_view,
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
    } else {
        // Still clear the offscreen texture
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Offscreen Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.offscreen_view,
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

    // === Pass 3: Post-process fullscreen quad -> swapchain ===
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post-Process Pass"),
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
        render_pass.draw(0..3, 0..1);  // Fullscreen triangle
    }

    queue.submit(std::iter::once(encoder.finish()));
}
```

- [ ] Run `cargo check`.

### Step 1.6: Update `resize()` to recreate offscreen texture

- [ ] Modify `Renderer2D::resize()` to destroy and recreate the offscreen texture + view + postprocess bind group at the new resolution. Signature changes — add `device` parameter.

```rust
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
    self.offscreen_view = self.offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Recreate post-process bind group (references new texture view)
    self.postprocess_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Post-Process Bind Group"),
        layout: &self.postprocess_pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.offscreen_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&self.postprocess_sampler),
            },
        ],
    });
}
```

**Update call site in `src/main.rs`:**
```rust
// Before:
renderer.resize(&queue, new_w, new_h);
// After:
renderer.resize(&device, &queue, new_w, new_h);
```

- [ ] Run `cargo check`.

### Step 1.7: Build and visual validation

- [ ] Run `cargo build` and launch the game to verify visual identity with V1.
- [ ] Verify window resize works (offscreen texture recreated).
- [ ] Run `cargo clippy` and fix any warnings.

---

## Task 2: Move Exposure/Color to GPU

**Goal:** Port `redirect_spectre_wide`, `game_exposure`, `add_color`, `mul_color` from CPU per-vertex to GPU post-process. Vertex colors become HDR `f32` values (no u8 clamping). Output must be visually identical to V1 (`TONEMAP_FAITHFUL`).

**Commit message:** `feat(render): GPU tonemapping - faithful port of redirect_spectre_wide`

### Step 2.1: Add post-process uniform buffer

- [ ] Add a `PostProcessUniforms` struct to `src/rendering/mod.rs`:

```rust
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
    pub _padding: f32,  // 32 bytes total, 16-byte aligned
}
```

- [ ] Add `postprocess_uniform_buffer: wgpu::Buffer` to `Renderer2D`.
- [ ] Create the buffer in `new()` with `BufferUsages::UNIFORM | BufferUsages::COPY_DST`, initialized to neutral values (`game_exposure=1.0`, `add_color=0,0,0`, `mul_color=1,1,1`).
- [ ] Update the postprocess bind group layout to include the uniform buffer at binding 2:

```rust
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
```

- [ ] Run `cargo check`.

### Step 2.2: Add `update_postprocess_uniforms()` method

- [ ] Add method to `Renderer2D`:

```rust
pub fn update_postprocess_uniforms(&self, queue: &wgpu::Queue, uniforms: &PostProcessUniforms) {
    queue.write_buffer(
        &self.postprocess_uniform_buffer,
        0,
        bytemuck::cast_slice(std::slice::from_ref(uniforms)),
    );
}
```

- [ ] Call it from `src/main.rs`, before `renderer.begin_frame()`, passing the current `globals.exposure` values:

```rust
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
```

- [ ] Run `cargo check`.

### Step 2.3: Port `redirect_spectre_wide` to WGSL

- [ ] Update `src/shaders/postprocess.wgsl` — change `TONEMAP_VARIANT` to `1u` and add the faithful tonemapping implementation:

```wgsl
const TONEMAP_VARIANT: u32 = 1u;  // 1 = faithful

struct PostProcessUniforms {
    game_exposure: f32,
    add_color_r: f32,
    add_color_g: f32,
    add_color_b: f32,
    mul_color_r: f32,
    mul_color_g: f32,
    mul_color_b: f32,
    _padding: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index & 1u)) * 4.0 - 1.0;
    let y = f32(i32(vertex_index >> 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@group(0) @binding(0) var offscreen_texture: texture_2d<f32>;
@group(0) @binding(1) var offscreen_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: PostProcessUniforms;

// Port of Rust redirect_spectre_wide: redistributes excess channel energy
// to spectral neighbors when a channel exceeds 255.0 (in HDR [0..inf] space).
fn redirect_spectre_wide(col: vec3<f32>) -> vec3<f32> {
    var r = col.r;
    var g = col.g;
    var b = col.b;

    // Red channel: receives from green overflow, and from blue double-overflow
    var r_out = r;
    if (b > 510.0) {
        if (g > 255.0) {
            r_out = r + g + b - 510.0 - 255.0;
        } else {
            r_out = r + b - 510.0;
        }
    } else {
        if (g > 255.0) {
            r_out = r + g - 255.0;
        }
    }

    // Green channel: receives from red and/or blue overflow
    var g_out = g;
    if (b > 255.0 && r > 255.0) {
        g_out = g + r + b - 510.0;
    } else if (r > 255.0) {
        g_out = g + r - 255.0;
    } else if (b > 255.0) {
        g_out = g + b - 255.0;
    }

    // Blue channel: receives from green overflow, and from red double-overflow
    var b_out = b;
    if (r > 510.0) {
        if (g > 255.0) {
            b_out = r + g + b - 510.0 - 255.0;
        } else {
            b_out = r + b - 510.0;
        }
    } else {
        if (g > 255.0) {
            b_out = g + b - 255.0;
        }
    }

    return vec3<f32>(r_out, g_out, b_out);
}

// Faithful tonemapping: 1:1 port of CPU rgb_of_hdr pipeline.
// hdr_color is in [0..inf] HDR space (values are raw HdrColor * exposure).
// Output is [0..1] for the swapchain.
fn tonemap_faithful(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);

    // Apply add_color (scaled by game_exposure) then mul_color, then redirect
    let with_add = hdr_color + add_color * uniforms.game_exposure;
    let with_mul = with_add * mul_color;
    let redirected = redirect_spectre_wide(with_mul);

    // Clamp to [0, 255] then normalize to [0, 1]
    return clamp(redirected, vec3<f32>(0.0), vec3<f32>(255.0)) / 255.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(offscreen_texture, offscreen_sampler, in.uv);

    if (TONEMAP_VARIANT == 0u) {
        // Passthrough (no tonemapping)
        return hdr_color;
    } else if (TONEMAP_VARIANT == 1u) {
        // Faithful: port of CPU redirect_spectre_wide pipeline
        let mapped = tonemap_faithful(hdr_color.rgb);
        return vec4<f32>(mapped, hdr_color.a);
    }

    return hdr_color;
}
```

- [ ] Run `cargo check`.

### Step 2.4: Change vertex colors from `[u8; 4]` to HDR `[f32; 4]`

- [ ] Modify `push_vertex` in `src/rendering/mod.rs` to accept `[f32; 4]` directly instead of `[u8; 4]`:

```rust
// Before:
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

// After:
fn push_vertex(&mut self, x: f32, y: f32, color: [f32; 4]) {
    self.vertices.push(Vertex {
        position: [x, y],
        color,
    });
}
```

- [ ] Update all public methods that take `[u8; 4]` to take `[f32; 4]`: `fill_rect`, `fill_circle`, `fill_ellipse`, `fill_poly`, `draw_poly`, `draw_line`, `plot`.

- [ ] Run `cargo check` (this will produce many errors in callers — expected).

### Step 2.5: Create `to_hdr_rgba()` replacing `to_rgba()` in `src/game.rs`

- [ ] In `src/game.rs`, add a new function that outputs HDR `[f32; 4]` without applying `game_exposure`/`add_color`/`mul_color` (those are now GPU-side):

```rust
/// Convert an HDR color (already intensified with per-entity exposure) to [f32; 4].
/// Does NOT apply game_exposure, add_color, or mul_color — those are GPU post-process.
pub(crate) fn to_hdr_rgba(color: HdrColor) -> [f32; 4] {
    [color.r as f32, color.g as f32, color.b as f32, 255.0]
}
```

- [ ] Replace all calls to `to_rgba(color, globals)` in `src/rendering/world.rs` with `to_hdr_rgba(color)`.
- [ ] For the background rect in `render_frame()`, compute the HDR space color without `rgb_of_hdr`:

```rust
// Before:
let bg_color = to_rgba(
    intensify(hdr(globals.visual.space_color), globals.exposure.game_exposure),
    globals,
);
renderer.fill_rect(0, 0, w, h, bg_color);

// After:
let bg = intensify(hdr(globals.visual.space_color), globals.exposure.game_exposure);
let bg_color = [bg.r as f32, bg.g as f32, bg.b as f32, 255.0];
renderer.fill_rect(0, 0, w, h, bg_color);
```

- [ ] Run `cargo check`.

### Step 2.6: Update HUD and pause menu color calls

- [ ] In `src/rendering/hud.rs`, all `[u8; 4]` color literals (e.g., `[255, 32, 32, 255]`) must become `[f32; 4]` in the HDR 0–255 range (e.g., `[255.0, 32.0, 32.0, 255.0]`). The post-process shader handles the /255 normalization.

- [ ] Update `render_hud`, `render_string`, `render_char`, `render_bar`, `draw_heart`, `draw_n_hearts`, `draw_bar_frame`, `render_scanlines` signatures from `color: [u8; 4]` to `color: [f32; 4]`.

- [ ] In `src/pause_menu.rs`, update all `[u8; 4]` color literals to `[f32; 4]`.

- [ ] Run `cargo check`.

### Step 2.7: Validation

- [ ] Run `cargo build`, launch the game, verify visuals match V1.
- [ ] Specifically verify: explosion flashes, exposure dimming, color shifts between stages, teleport blue flash.
- [ ] Run `cargo clippy` and fix warnings.

---

## Task 3: Add Tonemapping Variants

**Goal:** Add spectral bleed, ACES, and Reinhard tonemapping alongside faithful. Switchable via compile-time const.

**Commit message:** `feat(render): tonemapping variants (spectral bleed, ACES, Reinhard)`

### Step 3.1: Add tonemapping functions to `postprocess.wgsl`

- [ ] Add the following functions to `src/shaders/postprocess.wgsl`:

```wgsl
// Spectral bleed: smooth redistribution following spectral order.
// Excess in one channel bleeds to its spectral neighbor via smoothstep.
fn tonemap_spectral_bleed(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);

    let with_add = hdr_color + add_color * uniforms.game_exposure;
    let with_mul = with_add * mul_color;

    var r = with_mul.r;
    var g = with_mul.g;
    var b = with_mul.b;

    // Red excess bleeds to green (spectral neighbor: red -> orange -> yellow)
    let r_excess = max(r - 255.0, 0.0);
    let r_bleed = smoothstep(0.0, 255.0, r_excess) * r_excess;
    r = r - r_bleed;
    g = g + r_bleed * 0.7;  // 70% to green (orange tint)

    // Green excess bleeds equally to red and blue (spectral center)
    let g_excess = max(g - 255.0, 0.0);
    let g_bleed = smoothstep(0.0, 255.0, g_excess) * g_excess;
    g = g - g_bleed;
    r = r + g_bleed * 0.5;
    b = b + g_bleed * 0.5;

    // Blue excess bleeds to green (spectral neighbor: blue -> cyan -> green)
    let b_excess = max(b - 255.0, 0.0);
    let b_bleed = smoothstep(0.0, 255.0, b_excess) * b_excess;
    b = b - b_bleed;
    g = g + b_bleed * 0.7;

    return clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(255.0)) / 255.0;
}

// ACES filmic tonemapping (RRT + ODT fit by Stephen Hill).
// Input: linear HDR in [0..inf], output: [0..1].
fn aces_curve(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b_c = vec3<f32>(0.03);
    let c_c = vec3<f32>(2.43);
    let d = vec3<f32>(0.59);
    let e = vec3<f32>(0.14);
    return clamp((x * (a * x + b_c)) / (x * (c_c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn tonemap_aces(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);

    let with_add = hdr_color + add_color * uniforms.game_exposure;
    let with_mul = with_add * mul_color;

    // Normalize from [0..255] HDR space to [0..1] linear before ACES
    let linear = with_mul / 255.0;
    return aces_curve(linear);
}

// Reinhard luminance-based tonemapping.
fn tonemap_reinhard(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);

    let with_add = hdr_color + add_color * uniforms.game_exposure;
    let with_mul = with_add * mul_color;

    // Luminance-based Reinhard in [0..255] space
    let lum = dot(with_mul, vec3<f32>(0.2126, 0.7152, 0.0722));
    let mapped_lum = lum / (1.0 + lum / 255.0);
    let scale = select(mapped_lum / lum, 0.0, lum < 0.001);

    return clamp(with_mul * scale / 255.0, vec3<f32>(0.0), vec3<f32>(1.0));
}
```

### Step 3.2: Update fragment shader dispatch

- [ ] Update the `fs_main` function in `postprocess.wgsl`:

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(offscreen_texture, offscreen_sampler, in.uv);

    var mapped: vec3<f32>;
    if (TONEMAP_VARIANT == 0u) {
        mapped = hdr_color.rgb;  // passthrough
    } else if (TONEMAP_VARIANT == 1u) {
        mapped = tonemap_faithful(hdr_color.rgb);
    } else if (TONEMAP_VARIANT == 2u) {
        mapped = tonemap_spectral_bleed(hdr_color.rgb);
    } else if (TONEMAP_VARIANT == 3u) {
        mapped = tonemap_aces(hdr_color.rgb);
    } else {
        mapped = tonemap_reinhard(hdr_color.rgb);
    }

    return vec4<f32>(mapped, hdr_color.a);
}
```

### Step 3.3: Validation

- [ ] Run `cargo build` with `TONEMAP_VARIANT = 1` (faithful) — verify identical to V1.
- [ ] Change const to `2`, rebuild, verify spectral bleed renders without artifacts.
- [ ] Change const to `3`, rebuild, verify ACES renders without artifacts.
- [ ] Change const to `4`, rebuild, verify Reinhard renders without artifacts.
- [ ] Set back to `1` (faithful) as default.
- [ ] Run `cargo clippy`.

---

## Task 4: Separate HUD Pass

**Goal:** HUD geometry goes into its own buffer and renders in Pass 4, directly to the swapchain (not affected by post-process tonemapping).

**Commit message:** `feat(render): separate HUD pass (not affected by tonemapping)`

### Step 4.1: Create `src/shaders/hud.wgsl`

- [ ] Create `src/shaders/hud.wgsl` — screen-space polygon shader with colors normalized from [0..255] to [0..1]:

**File: `src/shaders/hud.wgsl`** (new file)
```wgsl
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(
        (in.position.x / screen_size.x) * 2.0 - 1.0,
        (in.position.y / screen_size.y) * 2.0 - 1.0,
        0.0,
        1.0
    );
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // HUD colors are in [0..255] HDR space; normalize to [0..1] for swapchain output.
    return vec4<f32>(
        clamp(in.color.r / 255.0, 0.0, 1.0),
        clamp(in.color.g / 255.0, 0.0, 1.0),
        clamp(in.color.b / 255.0, 0.0, 1.0),
        clamp(in.color.a / 255.0, 0.0, 1.0),
    );
}
```

- [ ] Run `cargo check`.

### Step 4.2: Add HUD pipeline and vertex buffer to `Renderer2D`

- [ ] Add new fields to `Renderer2D`:

```rust
hud_pipeline: wgpu::RenderPipeline,
hud_vertices: Vec<Vertex>,
hud_bind_group: wgpu::BindGroup,
```

- [ ] Create the HUD pipeline in `new()` targeting `surface_format` with `ALPHA_BLENDING`, using `hud.wgsl`.
- [ ] The HUD bind group only needs screen_size (binding 0) — reuse the single-entry bind group layout from V1.
- [ ] Initialize `hud_vertices` with `Vec::with_capacity(16384)`.

### Step 4.3: Add HUD vertex methods

- [ ] Add methods to `Renderer2D` that mirror the world methods but write to `hud_vertices`. These use internal helpers that push to a target vec:

```rust
pub fn hud_fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: [f32; 4]) { ... }
pub fn hud_fill_poly(&mut self, points: &[(i32, i32)], color: [f32; 4]) { ... }
pub fn hud_draw_poly(&mut self, points: &[(i32, i32)], color: [f32; 4], line_width: f32) { ... }
pub fn hud_fill_circle(&mut self, cx: f64, cy: f64, radius: f64, color: [f32; 4]) { ... }
pub fn hud_fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: [f32; 4]) { ... }
pub fn hud_draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: [f32; 4], width: f32) { ... }
pub fn hud_plot(&mut self, x: i32, y: i32, color: [f32; 4]) { ... }
```

To avoid code duplication, extract the geometry logic (scanline fill, line quad, circle fan) into internal functions parameterized by the target `Vec<Vertex>`:

```rust
fn push_vertex_to(target: &mut Vec<Vertex>, x: f32, y: f32, color: [f32; 4]) {
    target.push(Vertex { position: [x, y], color });
}
```

- [ ] Update `begin_frame()` to also clear `hud_vertices`.
- [ ] Run `cargo check`.

### Step 4.4: Route HUD rendering to HUD methods

- [ ] In `src/rendering/hud.rs`, change all `renderer.fill_poly(...)` calls to `renderer.hud_fill_poly(...)`, `renderer.fill_rect(...)` to `renderer.hud_fill_rect(...)`, etc.
- [ ] In `src/rendering/hud.rs`, change `render_string`/`render_char` to use `renderer.hud_fill_poly(...)`.
- [ ] In `src/pause_menu.rs`, change all rendering calls to use `hud_*` variants.
- [ ] In `src/game.rs` `render_frame()`, the scanlines call (if still present at this point) should also use HUD methods.
- [ ] Run `cargo check`.

### Step 4.5: Add Pass 4 (HUD) to `end_frame()`

- [ ] Update `end_frame()` to add a fourth render pass after post-process:

```rust
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
                load: wgpu::LoadOp::Load,  // Preserve post-process output
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
```

- [ ] Run `cargo check`.

### Step 4.6: Validation

- [ ] Run `cargo build`, launch the game.
- [ ] Verify HUD elements (score, stage, health bar, hearts, debug stats) render correctly.
- [ ] Verify HUD remains readable when explosions cause exposure changes (key test: HUD should NOT dim/brighten).
- [ ] Verify pause menu renders correctly (buttons, title, tooltips).
- [ ] Run `cargo clippy`.

---

## Task 5: SDF Circles

**Goal:** Replace CPU `fill_circle` for smoke, fire, muzzle, explosions, and chunks with instanced SDF circles. Remove asteroid base circle. Convert ship base circle to polygon.

**Commit message:** `feat(render): SDF instanced circles for particles`

### Step 5.1: Create `src/shaders/sdf.wgsl`

- [ ] Create `src/shaders/sdf.wgsl`:

**File: `src/shaders/sdf.wgsl`** (new file)
```wgsl
// SDF rendering for circles and capsules via instanced quads.
// Each instance defines shape parameters; the fragment shader evaluates the SDF.

// Compile-time AA toggle
const SDF_AA_ENABLED: bool = true;

// Instance data for circles
struct CircleInstance {
    @location(2) center: vec2<f32>,
    @location(3) radius: f32,
    @location(4) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,          // Local quad UV relative to center
    @location(1) color: vec4<f32>,
    @location(2) radius_px: f32,         // Radius in pixels (for AA sizing)
};

@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;

// Quad vertices: 6 vertices forming 2 triangles covering [-1, 1] local space.
@vertex
fn vs_circle(
    @builtin(vertex_index) vertex_index: u32,
    instance: CircleInstance,
) -> VertexOutput {
    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    let local = quad_pos[vertex_index];

    // Expand quad to cover circle + 1px margin for AA
    let margin = 1.0;
    let pixel_pos = instance.center + local * (instance.radius + margin);

    var out: VertexOutput;
    out.position = vec4<f32>(
        (pixel_pos.x / screen_size.x) * 2.0 - 1.0,
        (pixel_pos.y / screen_size.y) * 2.0 - 1.0,
        0.0,
        1.0
    );
    out.uv = local * (instance.radius + margin);
    out.color = instance.color;
    out.radius_px = instance.radius;
    return out;
}

@fragment
fn fs_circle(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv) - in.radius_px;

    var alpha: f32;
    if (SDF_AA_ENABLED) {
        alpha = smoothstep(0.5, -0.5, dist);
    } else {
        alpha = select(0.0, 1.0, dist < 0.0);
    }

    if (alpha < 0.001) {
        discard;
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
```

- [ ] Run `cargo check`.

### Step 5.2: Define `CircleInstance` struct in Rust

- [ ] Add to `src/rendering/mod.rs`:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub center: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
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
```

**Alignment note:** `CircleInstance` is 28 bytes (`[f32;2]` + `f32` + `[f32;4]`). All fields are `f32`-aligned. wgpu vertex buffer attributes only require component alignment (4 bytes for f32), so 28 bytes is valid as `array_stride`. If wgpu validation complains, add a `_padding: f32` field to reach 32 bytes.

- [ ] Run `cargo check`.

### Step 5.3: Add SDF pipeline and instance buffer to `Renderer2D`

- [ ] Add fields:

```rust
sdf_circle_pipeline: wgpu::RenderPipeline,
sdf_circle_instances: Vec<CircleInstance>,
sdf_bind_group: wgpu::BindGroup,
```

- [ ] Create the SDF circle pipeline in `new()`:
  - Uses `sdf.wgsl` with `vs_circle` and `fs_circle` entry points.
  - Vertex buffers: one instance buffer (`CircleInstance::desc()`) — no per-vertex buffer (quad generated from `vertex_index`).
  - Targets `Rgba16Float` (offscreen texture) with `ALPHA_BLENDING`.
  - Bind group: screen_size uniform at binding 0 (reuse existing buffer).

```rust
let sdf_circle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("SDF Circle Pipeline"),
    layout: Some(&sdf_pipeline_layout),  // Uses screen_size bind group layout
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
        ..Default::default()
    },
    depth_stencil: None,
    multisample: wgpu::MultisampleState::default(),
    multiview: None,
    cache: None,
});
```

- [ ] Initialize `sdf_circle_instances` with `Vec::with_capacity(4096)`.
- [ ] Update `begin_frame()` to clear `sdf_circle_instances`.
- [ ] Run `cargo check`.

### Step 5.4: Add `push_circle_instance()` method

- [ ] Add to `Renderer2D`:

```rust
pub fn push_circle_instance(&mut self, cx: f32, cy: f32, radius: f32, color: [f32; 4]) {
    if radius <= 0.0 {
        return;
    }
    self.sdf_circle_instances.push(CircleInstance {
        center: [cx, cy],
        radius,
        color,
    });
}
```

### Step 5.5: Add SDF pass to `end_frame()` (Pass 2)

- [ ] Insert Pass 2 between world pass and post-process pass in `end_frame()`:

```rust
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
    // 6 vertices per quad (2 triangles), N instances
    render_pass.draw(0..6, 0..self.sdf_circle_instances.len() as u32);
}
```

- [ ] Run `cargo check`.

### Step 5.6: Replace `fill_circle` calls for particles in `world.rs`

- [ ] In `render_chunk()` (`src/rendering/world.rs`), replace `fill_circle` with `push_circle_instance`:

```rust
// Before:
renderer.fill_circle(x as f64, y as f64, r.max(1) as f64, color);

// After:
renderer.push_circle_instance(x as f32, y as f32, r.max(1) as f32, color);
```

- [ ] Run `cargo check`.

### Step 5.7: Remove asteroid base circle

- [ ] In `render_visuals()`, remove the base circle rendering block entirely (the `if visuals.radius > 0.0 && !globals.visual.retro` block that calls `fill_circle`). Asteroids are now solely their polygon shapes.

### Step 5.8: Convert ship base circle to polygon

- [ ] In `src/objects.rs` `spawn_ship()`, add a new shape entry at the beginning of the shapes vec that approximates the ship's base circle as a 16-sided polygon:

```rust
// Ship base circle as polygon (replaces fill_circle rendering)
{
    let n_sides = 16;
    let base_radius = SHIP_RADIUS * 0.9;  // matches visuals.radius
    let mut circle_poly = Vec::with_capacity(n_sides);
    for i in 0..n_sides {
        let angle = 2.0 * PI * i as f64 / n_sides as f64;
        circle_poly.push((angle, base_radius));
    }
    shapes.insert(0, ((1000.0, 100.0, 25.0), Polygon(circle_poly)));
}
```

- [ ] Run `cargo check`.

### Step 5.9: Update `render_visuals` for SDF circle entities

- [ ] Restructure `render_visuals` to handle circle-only entities (smoke, explosions) via SDF:

```rust
pub fn render_visuals(
    entity: &Entity,
    offset: Vec2,
    renderer: &mut Renderer2D,
    globals: &Globals,
    rng: &mut impl Rng,
) {
    let visuals = &entity.visuals;
    let position = scale_vec(
        add_vec(
            add_vec(entity.position, globals.screenshake.game_screenshake_pos),
            offset,
        ),
        globals.render.render_scale,
    );
    let exposure = globals.exposure.game_exposure * entity.hdr_exposure;

    // SDF circle for entities with radius but no polygon shapes (smoke, explosions)
    if visuals.radius > 0.0 && !globals.visual.retro && visuals.shapes.is_empty() {
        let color = to_hdr_rgba(intensify(hdr(visuals.color), exposure));
        let (x, y) = dither_vec(position, DITHER_AA, globals.render.current_jitter_double);
        let r = dither_radius(
            visuals.radius * globals.render.render_scale,
            DITHER_AA,
            DITHER_POWER_RADIUS,
            rng,
        );
        renderer.push_circle_instance(x as f32, y as f32, r.max(1) as f32, color);
    }

    // Polygon shapes on top
    render_shapes(
        &visuals.shapes,
        position,
        entity.orientation,
        exposure,
        renderer,
        globals,
    );
}
```

- [ ] Run `cargo check`.

### Step 5.10: Validation

- [ ] Run `cargo build`, launch the game.
- [ ] Verify smoke/fire/muzzle/explosions/chunks render as smooth SDF circles.
- [ ] Verify asteroids show only their polygon shape (no base circle underneath).
- [ ] Verify ship has its base circle rendered as a polygon (no visual gap).
- [ ] Run `cargo clippy`.

---

## Task 6: SDF Capsules

**Goal:** Replace projectile and star trail rendering with SDF capsule instances.

**Commit message:** `feat(render): SDF capsule instances for projectile and star trails`

### Step 6.1: Add capsule SDF to `sdf.wgsl`

- [ ] Append to `src/shaders/sdf.wgsl`:

```wgsl
// Trail implementation selection (compile-time const)
// true = capsule SDF (single quad), false = composite (not implemented in Phase 1)
const TRAIL_IMPL_CAPSULE: bool = true;

struct CapsuleInstance {
    @location(5) p0: vec2<f32>,
    @location(6) p1: vec2<f32>,
    @location(7) radius: f32,
    @location(8) color: vec4<f32>,
};

struct CapsuleVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) p0: vec2<f32>,
    @location(3) p1: vec2<f32>,
    @location(4) radius_px: f32,
};

// Distance from point to line segment (for capsule SDF)
fn dist_to_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let len_sq = dot(ab, ab);
    // Degenerate case: segment is a point
    if (len_sq < 0.0001) {
        return length(ap);
    }
    let t = clamp(dot(ap, ab) / len_sq, 0.0, 1.0);
    let closest = a + ab * t;
    return length(p - closest);
}

@vertex
fn vs_capsule(
    @builtin(vertex_index) vertex_index: u32,
    instance: CapsuleInstance,
) -> CapsuleVertexOutput {
    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    let local = quad_pos[vertex_index];

    // Compute bounding box of capsule + margin for AA
    let margin = 1.0;
    let center = (instance.p0 + instance.p1) * 0.5;
    let half_extent = abs(instance.p1 - instance.p0) * 0.5 + vec2<f32>(instance.radius + margin);

    let pixel_pos = center + local * half_extent;

    var out: CapsuleVertexOutput;
    out.position = vec4<f32>(
        (pixel_pos.x / screen_size.x) * 2.0 - 1.0,
        (pixel_pos.y / screen_size.y) * 2.0 - 1.0,
        0.0,
        1.0
    );
    out.world_pos = pixel_pos;
    out.color = instance.color;
    out.p0 = instance.p0;
    out.p1 = instance.p1;
    out.radius_px = instance.radius;
    return out;
}

@fragment
fn fs_capsule(in: CapsuleVertexOutput) -> @location(0) vec4<f32> {
    let dist = dist_to_segment(in.world_pos, in.p0, in.p1) - in.radius_px;

    var alpha: f32;
    if (SDF_AA_ENABLED) {
        alpha = smoothstep(0.5, -0.5, dist);
    } else {
        alpha = select(0.0, 1.0, dist < 0.0);
    }

    // Intensity falloff: exponential decay from capsule surface.
    // Replaces the 4 concentric draw calls of V1 render_light_trail.
    let surface_dist = dist_to_segment(in.world_pos, in.p0, in.p1);
    let falloff = exp(-surface_dist / max(in.radius_px, 0.1) * 2.0);

    if (alpha < 0.001) {
        discard;
    }

    return vec4<f32>(in.color.rgb * falloff, in.color.a * alpha);
}
```

- [ ] Run `cargo check`.

### Step 6.2: Define `CapsuleInstance` struct in Rust

- [ ] Add to `src/rendering/mod.rs`:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CapsuleInstance {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
}
// Size: 4*2 + 4*2 + 4 + 4*4 = 36 bytes, all f32-aligned.

impl CapsuleInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        5 => Float32x2,   // p0
        6 => Float32x2,   // p1
        7 => Float32,     // radius
        8 => Float32x4,   // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CapsuleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
```

### Step 6.3: Add capsule pipeline and buffer to `Renderer2D`

- [ ] Add fields:

```rust
sdf_capsule_pipeline: wgpu::RenderPipeline,
sdf_capsule_instances: Vec<CapsuleInstance>,
```

- [ ] Create the pipeline in `new()` using `vs_capsule`/`fs_capsule` entry points from `sdf.wgsl`, same bind group as circle pipeline (screen_size only), targeting `Rgba16Float` with `ALPHA_BLENDING`.
- [ ] Initialize `sdf_capsule_instances` with capacity 2048.
- [ ] Update `begin_frame()` to clear capsule instances.

### Step 6.4: Add `push_capsule_instance()` method

- [ ] Add to `Renderer2D`:

```rust
pub fn push_capsule_instance(
    &mut self,
    x0: f32, y0: f32,
    x1: f32, y1: f32,
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
    });
}
```

### Step 6.5: Add capsule draw to Pass 2 in `end_frame()`

- [ ] Extend the SDF render pass to also draw capsules. Structure the pass to issue multiple draw calls within the same render pass:

```rust
// === Pass 2: SDF entities -> offscreen Rgba16Float ===
let has_sdf = !self.sdf_circle_instances.is_empty() || !self.sdf_capsule_instances.is_empty();
if has_sdf {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("SDF Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &self.offscreen_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    // Draw circles
    if !self.sdf_circle_instances.is_empty() {
        let circle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SDF Circle Instance Buffer"),
            contents: bytemuck::cast_slice(&self.sdf_circle_instances),
            usage: wgpu::BufferUsages::VERTEX,
        });
        render_pass.set_pipeline(&self.sdf_circle_pipeline);
        render_pass.set_bind_group(0, &self.sdf_bind_group, &[]);
        render_pass.set_vertex_buffer(0, circle_buffer.slice(..));
        render_pass.draw(0..6, 0..self.sdf_circle_instances.len() as u32);
    }

    // Draw capsules
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
```

- [ ] Run `cargo check`.

### Step 6.6: Replace `render_projectile` with capsule instances

- [ ] In `src/rendering/world.rs`, replace `render_projectile()`:

```rust
pub fn render_projectile(entity: &Entity, renderer: &mut Renderer2D, globals: &Globals, rng: &mut impl Rng) {
    let rad = globals.render.render_scale
        * rand_range(0.5, 1.0, rng)
        * entity.visuals.radius;

    if globals.visual.retro {
        let pos = scale_vec(entity.position, globals.render.render_scale);
        let (x, y) = dither_vec(pos, DITHER_AA, globals.render.current_jitter_double);
        renderer.push_circle_instance(x as f32, y as f32, rad.max(1.0) as f32, [255.0, 255.0, 255.0, 255.0]);
    } else {
        let pos = entity.position;
        let col = intensify(hdr(entity.visuals.color), entity.hdr_exposure * globals.exposure.game_exposure);

        // Compute trail endpoint (motion blur)
        let pos1 = scale_vec(add_vec(pos, globals.screenshake.game_screenshake_pos), globals.render.render_scale);
        let dt_game = globals.time.game_speed
            * (globals.time.time_current_frame - globals.time.time_last_frame)
                .max(1.0 / FRAMERATE_RENDER);
        let veloc = scale_vec(entity.velocity, -(globals.observer_proper_time / entity.proper_time) * dt_game);
        let last_pos = scale_vec(
            add_vec(sub_vec(pos, veloc), globals.screenshake.game_screenshake_previous_pos),
            globals.render.render_scale,
        );
        let pos2 = lerp_vec(last_pos, pos1, SHUTTER_SPEED);

        let (x1, y1) = dither_vec(pos1, DITHER_AA, globals.render.current_jitter_double);
        let (x2, y2) = dither_vec(pos2, DITHER_AA, globals.render.current_jitter_double);

        let color = to_hdr_rgba(col);
        renderer.push_capsule_instance(
            x1 as f32, y1 as f32,
            x2 as f32, y2 as f32,
            rad.max(1.0) as f32,
            color,
        );
    }
}
```

### Step 6.7: Replace `render_star_trail` moving-star branch with capsule

- [ ] In `render_star_trail()`, replace the moving-star branch (`else` block where `x1 != x2 || y1 != y2`):

```rust
// Moving star: render as capsule trail
let dist = magnitude(sub_vec(pos1, pos2));
let trail_lum = (1.0 / (1.0 + dist)).sqrt();
let trail_color = hdr_add(
    intensify(star_color_tmp, trail_lum),
    hdr_add(
        intensify(hdr(globals.visual.space_color), globals.exposure.game_exposure),
        intensify(hdr(globals.exposure.add_color), globals.exposure.game_exposure),
    ),
);
let color = to_hdr_rgba(trail_color);
renderer.push_capsule_instance(
    x1 as f32, y1 as f32,
    x2 as f32, y2 as f32,
    1.0,  // 1px radius (thin trail)
    color,
);
```

- [ ] Remove the `render_light_trail()` function (no longer needed).

- [ ] Run `cargo check`.

### Step 6.8: Validation

- [ ] Run `cargo build`, launch the game.
- [ ] Verify projectile trails render as smooth capsules with intensity falloff.
- [ ] Verify star trails render as capsules when moving.
- [ ] Fire many projectiles (hold Space) to stress-test capsule instance count.
- [ ] Run `cargo clippy`.

---

## Task 7: MSAA

**Goal:** Add multisampled offscreen texture + resolve for polygon passes (world). Toggleable off/2x/4x.

**Commit message:** `feat(render): MSAA for polygon geometry (off/2x/4x)`

### Step 7.1: Add MSAA config to `parameters.rs`

- [ ] Add to `src/parameters.rs`:

```rust
// ============================================================================
// Anti-Aliasing Constants
// ============================================================================

/// MSAA sample count for polygon rendering. Valid values: 1 (off), 2, 4.
/// SDF entities use their own smoothstep AA (controlled by SDF_AA_ENABLED in sdf.wgsl).
pub const MSAA_SAMPLE_COUNT: u32 = 4;
```

### Step 7.2: Create multisampled offscreen texture

- [ ] Add a `create_msaa_texture` helper to `src/rendering/mod.rs`:

```rust
fn create_msaa_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    sample_count: u32,
) -> Option<wgpu::Texture> {
    if sample_count <= 1 {
        return None;
    }
    Some(device.create_texture(&wgpu::TextureDescriptor {
        label: Some("MSAA Texture"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    }))
}
```

- [ ] Add fields to `Renderer2D`:

```rust
msaa_sample_count: u32,
msaa_offscreen_texture: Option<wgpu::Texture>,
msaa_offscreen_view: Option<wgpu::TextureView>,
```

- [ ] Create the MSAA texture in `new()` and `resize()`.

### Step 7.3: Update world pipeline for MSAA

- [ ] In the world pipeline creation, set `multisample`:

```rust
multisample: wgpu::MultisampleState {
    count: MSAA_SAMPLE_COUNT,
    mask: !0,
    alpha_to_coverage_enabled: false,
},
```

**Important:** When MSAA is enabled (`count > 1`), the blend state on the color target must still be `ALPHA_BLENDING` — wgpu supports blending with MSAA.

### Step 7.4: Update Pass 1 for MSAA resolve

- [ ] When MSAA is enabled, Pass 1 renders to the multisampled texture and resolves to the non-multisampled offscreen texture:

```rust
// Pass 1: World geometry -> MSAA offscreen -> resolve to offscreen
let (target_view, resolve) = if let Some(ref msaa_view) = self.msaa_offscreen_view {
    (msaa_view, Some(&self.offscreen_view))
} else {
    (&self.offscreen_view, None)
};

// ... in RenderPassColorAttachment:
wgpu::RenderPassColorAttachment {
    view: target_view,
    resolve_target: resolve,  // None when MSAA disabled, Some when enabled
    ops: wgpu::Operations {
        load: wgpu::LoadOp::Clear(/* ... */),
        store: wgpu::StoreOp::Store,
    },
}
```

- [ ] Run `cargo check`.

### Step 7.5: Update `resize()` to recreate MSAA textures

- [ ] In `resize()`, recreate `msaa_offscreen_texture` and `msaa_offscreen_view`:

```rust
self.msaa_offscreen_texture = create_msaa_texture(
    device, width, height,
    wgpu::TextureFormat::Rgba16Float,
    self.msaa_sample_count,
);
self.msaa_offscreen_view = self.msaa_offscreen_texture.as_ref()
    .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));
```

- [ ] Run `cargo check`.

### Step 7.6: Validation

- [ ] Build and run with `MSAA_SAMPLE_COUNT = 4`. Verify polygon edges (ship, asteroids) are smoother.
- [ ] Change to `MSAA_SAMPLE_COUNT = 2`. Rebuild, verify.
- [ ] Change to `MSAA_SAMPLE_COUNT = 1`. Rebuild, verify no MSAA (fallback to direct render).
- [ ] Verify SDF entities are unaffected by MSAA setting (they use their own AA).
- [ ] Run `cargo clippy`.

---

## Task 8: SDF AA Toggle

**Goal:** The `SDF_AA_ENABLED` const in `sdf.wgsl` is already implemented (Task 5). Verify it works and document the toggle.

**Commit message:** `feat(render): verify SDF AA toggle (smoothstep vs hard edge)`

### Step 8.1: Test SDF AA on

- [ ] Ensure `sdf.wgsl` has `const SDF_AA_ENABLED: bool = true;`.
- [ ] Build and run. Verify SDF circles and capsules have smooth edges.

### Step 8.2: Test SDF AA off

- [ ] Change `sdf.wgsl` to `const SDF_AA_ENABLED: bool = false;`.
- [ ] Build and run. Verify SDF shapes have hard pixel edges.
- [ ] Revert to `true`.

### Step 8.3: Document the toggle

- [ ] Add a comment in `src/parameters.rs`:

```rust
// SDF anti-aliasing is controlled by compile-time const `SDF_AA_ENABLED` in src/shaders/sdf.wgsl.
// true = smoothstep AA (default), false = hard edges.
// This is independent of MSAA, which only affects polygon geometry.
```

- [ ] Run `cargo clippy`.

---

## Task 9: Delete Retro/Scanline Code

**Goal:** Remove all retro mode and scanline code paths. The SDF AA toggle provides a clean-edge option for those who want a crisper look.

**Commit message:** `refactor: remove retro/scanline code paths`

### Step 9.1: Remove `VisualConfig` retro/scanline fields

- [ ] In `src/parameters.rs`, remove from `VisualConfig`:
  - `retro: bool`
  - `oldschool: bool`
  - `scanlines: bool`
  - `scanlines_offset: i32`

- [ ] Remove constants `SCANLINES_PERIOD` and `ANIMATED_SCANLINES`.

- [ ] Remove `GlobalToggle::Scanlines` and `GlobalToggle::Retro` variants and their match arms in `get_toggle`/`set_toggle`.

- [ ] Update `Globals::new()` to remove the deleted fields from `VisualConfig` initialization.

### Step 9.2: Remove retro branches from rendering code

- [ ] In `src/rendering/world.rs`:
  - `render_poly()`: remove the `if globals.visual.retro` branch that draws white outlines. Always use `fill_poly`.
  - `render_chunk()`: remove the retro branch (entire `if globals.visual.retro { ... } else { ... }` becomes just the non-retro body).
  - `render_projectile()`: remove the retro branch.

- [ ] In `src/rendering/hud.rs`:
  - Remove `render_scanlines()` function entirely.
  - In `render_hud()`, remove the `if globals.visual.retro { return; }` early-return.

- [ ] In `src/game.rs` `render_frame()`:
  - Remove the retro background branch (`if globals.visual.retro { ... } else { ... }` becomes just the non-retro body).
  - Remove the `if !globals.visual.retro` guard around star rendering.
  - Remove the scanlines block at the end (`if globals.visual.scanlines { ... }`).

- [ ] In `src/pause_menu.rs`:
  - Remove all `globals.visual.retro` branches in `apply_button()` and elsewhere. Use only the normal (non-retro) rendering path.

### Step 9.3: Remove retro/scanline buttons from pause menu

- [ ] In `src/pause_menu.rs` `make_buttons()`, remove the "scanlines" and "retro visuals" button entries.

### Step 9.4: Clean up

- [ ] Run `cargo check` — fix all compilation errors from removed fields.
- [ ] Run `cargo clippy` — fix all dead code warnings.
- [ ] Grep for remaining references: `retro`, `scanlines`, `SCANLINES_PERIOD`, `ANIMATED_SCANLINES`, `oldschool`. Remove any stragglers.

### Step 9.5: Delete old shader

- [ ] Delete `src/shaders/shape.wgsl` (replaced by `world.wgsl`, `sdf.wgsl`, `postprocess.wgsl`, `hud.wgsl`).
- [ ] Verify no `include_str!("../shaders/shape.wgsl")` references remain.
- [ ] Run `cargo build`.

### Step 9.6: Validation

- [ ] Build and run. Game should work without any retro or scanline functionality.
- [ ] Run `cargo clippy` — should be clean, no dead code warnings related to retro/scanlines.

---

## Task 10: Particle Budgets

**Goal:** Add caps and graceful despawn logic for particles to prevent unbounded growth.

**Commit message:** `feat(render): particle budgets with graceful degradation`

### Step 10.1: Add particle budget constants to `parameters.rs`

- [ ] Add to `src/parameters.rs`:

```rust
// ============================================================================
// Particle Budget Constants
// ============================================================================

/// Maximum smoke particles (oldest-first despawn).
pub const PARTICLE_BUDGET_SMOKE: usize = 2048;
/// Maximum fire/muzzle particles (oldest-first despawn).
pub const PARTICLE_BUDGET_FIRE: usize = 512;
/// Maximum chunk particles (lowest-opacity-first despawn).
pub const PARTICLE_BUDGET_CHUNKS: usize = 512;
/// Maximum explosion particles (oldest-first despawn).
pub const PARTICLE_BUDGET_EXPLOSIONS: usize = 256;
/// Maximum projectiles (never culled — gameplay-critical).
pub const PARTICLE_BUDGET_PROJECTILES: usize = 256;
/// At this fraction of capacity, begin accelerated fade for lowest-priority particles.
pub const PARTICLE_DEGRADATION_THRESHOLD: f64 = 0.9;
/// Fade acceleration factor when over degradation threshold.
pub const PARTICLE_DEGRADATION_FADE_MULTIPLIER: f64 = 3.0;
```

### Step 10.2: Add budget enforcement function to `game.rs`

- [ ] In `src/game.rs`, add `enforce_particle_budgets()`:

```rust
fn enforce_particle_budgets(state: &mut GameState) {
    // Smoke: oldest-first (front of Vec is oldest since we push_back)
    if state.smoke.len() > PARTICLE_BUDGET_SMOKE {
        let excess = state.smoke.len() - PARTICLE_BUDGET_SMOKE;
        state.smoke.drain(0..excess);
    }

    // OOS smoke
    if state.smoke_oos.len() > PARTICLE_BUDGET_SMOKE {
        let excess = state.smoke_oos.len() - PARTICLE_BUDGET_SMOKE;
        state.smoke_oos.drain(0..excess);
    }

    // Chunks: lowest-radius first (proxy for oldest/most-faded)
    if state.chunks.len() > PARTICLE_BUDGET_CHUNKS {
        state.chunks.sort_by(|a, b| {
            a.visuals.radius.partial_cmp(&b.visuals.radius).unwrap_or(std::cmp::Ordering::Equal)
        });
        let excess = state.chunks.len() - PARTICLE_BUDGET_CHUNKS;
        state.chunks.drain(0..excess);
    }

    // Explosion chunks
    if state.chunks_explo.len() > PARTICLE_BUDGET_CHUNKS {
        state.chunks_explo.sort_by(|a, b| {
            a.visuals.radius.partial_cmp(&b.visuals.radius).unwrap_or(std::cmp::Ordering::Equal)
        });
        let excess = state.chunks_explo.len() - PARTICLE_BUDGET_CHUNKS;
        state.chunks_explo.drain(0..excess);
    }

    // Explosions: oldest-first
    if state.explosions.len() > PARTICLE_BUDGET_EXPLOSIONS {
        let excess = state.explosions.len() - PARTICLE_BUDGET_EXPLOSIONS;
        state.explosions.drain(0..excess);
    }
}
```

### Step 10.3: Add graceful degradation (accelerated fade near budget)

- [ ] In `update_game()`, modify the smoke decay loop to apply a fade multiplier when near capacity:

```rust
// Graceful degradation: accelerate fade when near budget
let smoke_load = state.smoke.len() as f64 / PARTICLE_BUDGET_SMOKE as f64;
let fade_multiplier = if smoke_load > PARTICLE_DEGRADATION_THRESHOLD {
    PARTICLE_DEGRADATION_FADE_MULTIPLIER
} else {
    1.0
};

for s in state.smoke.iter_mut() {
    decay_smoke_with_multiplier(s, globals, fade_multiplier);
}
for s in state.smoke_oos.iter_mut() {
    decay_smoke_with_multiplier(s, globals, fade_multiplier);
}
```

- [ ] Create `decay_smoke_with_multiplier()` alongside existing `decay_smoke()`:

```rust
/// Decay smoke with an optional fade acceleration multiplier.
/// multiplier=1.0 is normal decay; multiplier>1.0 accelerates fade for budget management.
pub fn decay_smoke_with_multiplier(smoke: &mut Entity, globals: &Globals, multiplier: f64) {
    let dt_game = globals.time.game_speed * globals.dt() * multiplier;
    let half_r = SMOKE_HALF_RADIUS * smoke.proper_time;
    let half_c = SMOKE_HALF_COL * smoke.proper_time;
    smoke.visuals.radius = smoke.visuals.radius * (2.0_f64).powf(-dt_game / half_r)
        - SMOKE_RADIUS_DECAY * dt_game * globals.observer_proper_time / smoke.proper_time;
    if smoke.hdr_exposure > 0.001 {
        smoke.hdr_exposure *= (2.0_f64).powf(-dt_game / half_c);
    }
}
```

- [ ] Apply the same pattern to chunks: accelerate radius decay when chunk count exceeds `PARTICLE_DEGRADATION_THRESHOLD * PARTICLE_BUDGET_CHUNKS`.

### Step 10.4: Add budget enforcement call

- [ ] At the end of `update_game()`, after `despawn(state, globals);`, add:

```rust
enforce_particle_budgets(state);
```

### Step 10.5: Validation

- [ ] Build and run. Play aggressively (many explosions, rapid fire, sustained combat) to stress-test budgets.
- [ ] Monitor debug HUD particle counts — verify they stay within budgets.
- [ ] Verify no crashes, no stuttering at particle caps.
- [ ] Verify degradation is smooth — particles fade faster rather than popping.
- [ ] Run `cargo clippy`.

---

## Files Modified/Created Summary

### New Files
| File | Task |
|------|------|
| `src/shaders/world.wgsl` | Task 1 |
| `src/shaders/postprocess.wgsl` | Tasks 1, 2, 3 |
| `src/shaders/sdf.wgsl` | Tasks 5, 6 |
| `src/shaders/hud.wgsl` | Task 4 |

### Modified Files
| File | Tasks |
|------|-------|
| `src/rendering/mod.rs` | 1, 2, 4, 5, 6, 7 |
| `src/rendering/world.rs` | 2, 5, 6, 9 |
| `src/rendering/hud.rs` | 2, 4, 9 |
| `src/game.rs` | 2, 4, 9, 10 |
| `src/main.rs` | 1, 2 |
| `src/color.rs` | 2 (CPU `game_exposure`/`add_color`/`mul_color` no longer applied per-vertex) |
| `src/parameters.rs` | 7, 8, 9, 10 |
| `src/objects.rs` | 5 (ship base circle -> polygon) |
| `src/pause_menu.rs` | 2, 4, 9 |

### Deleted Files
| File | Task |
|------|------|
| `src/shaders/shape.wgsl` | Task 9 |

### Deleted Code
| Item | Task |
|------|------|
| `globals.visual.retro`, `scanlines`, `scanlines_offset`, `oldschool` | Task 9 |
| `render_scanlines()` | Task 9 |
| All `retro` branches in rendering code | Task 9 |
| `render_light_trail()` function | Task 6 |
| `GlobalToggle::Scanlines`, `GlobalToggle::Retro` | Task 9 |
| `SCANLINES_PERIOD`, `ANIMATED_SCANLINES` constants | Task 9 |

### Kept As-Is
| Item | Reason |
|------|--------|
| `fill_poly`, `draw_poly`, `draw_line` | Polygon CPU triangulation stays for world + HUD |
| `fill_rect` | Background quad, trivial |
| `fill_circle`, `fill_ellipse` | Kept on `Renderer2D` for HUD use (hearts) |
| `dither_radius`, `dither_vec` | Still used by polygon rendering + SDF positioning |
| `rgb_of_hdr` in `color.rs` | Kept as CPU reference implementation; no longer called in render path |
