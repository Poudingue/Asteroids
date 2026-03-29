const SDF_AA_ENABLED: bool = true;

struct CircleInstance {
    @location(2) center: vec2<f32>,
    @location(3) radius: f32,
    @location(4) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) radius_px: f32,
};

@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;

@vertex
fn vs_circle(
    @builtin(vertex_index) vertex_index: u32,
    instance: CircleInstance,
) -> VertexOutput {
    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );
    let local = quad_pos[vertex_index];
    let margin = 1.0;
    let pixel_pos = instance.center + local * (instance.radius + margin);

    var out: VertexOutput;
    out.position = vec4<f32>(
        (pixel_pos.x / screen_size.x) * 2.0 - 1.0,
        (pixel_pos.y / screen_size.y) * 2.0 - 1.0,
        0.0, 1.0
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
    if (SDF_AA_ENABLED) { alpha = smoothstep(0.5, -0.5, dist); }
    else { alpha = select(0.0, 1.0, dist < 0.0); }
    if (alpha < 0.001) { discard; }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
