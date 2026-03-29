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

// ============================================================================
// Capsule SDF
// ============================================================================

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

fn dist_to_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let len_sq = dot(ab, ab);
    if (len_sq < 0.0001) { return length(ap); }
    let t = clamp(dot(ap, ab) / len_sq, 0.0, 1.0);
    return length(p - (a + ab * t));
}

@vertex
fn vs_capsule(@builtin(vertex_index) vertex_index: u32, instance: CapsuleInstance) -> CapsuleVertexOutput {
    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );
    let local = quad_pos[vertex_index];
    let margin = 1.0;
    let center = (instance.p0 + instance.p1) * 0.5;
    let half_extent = abs(instance.p1 - instance.p0) * 0.5 + vec2<f32>(instance.radius + margin);
    let pixel_pos = center + local * half_extent;

    var out: CapsuleVertexOutput;
    out.position = vec4<f32>(
        (pixel_pos.x / screen_size.x) * 2.0 - 1.0,
        (pixel_pos.y / screen_size.y) * 2.0 - 1.0,
        0.0, 1.0
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
    if (SDF_AA_ENABLED) { alpha = smoothstep(0.5, -0.5, dist); }
    else { alpha = select(0.0, 1.0, dist < 0.0); }

    let surface_dist = dist_to_segment(in.world_pos, in.p0, in.p1);
    let falloff = exp(-surface_dist / max(in.radius_px, 0.1) * 2.0);

    if (alpha < 0.001) { discard; }
    return vec4<f32>(in.color.rgb * falloff, in.color.a * alpha);
}
