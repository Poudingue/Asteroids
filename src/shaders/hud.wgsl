struct HudUniforms {
    screen_width: f32,
    screen_height: f32,
    brightness_scale: f32,
    _padding: f32,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: HudUniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(
        (in.position.x / uniforms.screen_width) * 2.0 - 1.0,
        (in.position.y / uniforms.screen_height) * 2.0 - 1.0,
        0.0, 1.0
    );
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(
        clamp(in.color.r / 255.0 * uniforms.brightness_scale, 0.0, 1.0),
        clamp(in.color.g / 255.0 * uniforms.brightness_scale, 0.0, 1.0),
        clamp(in.color.b / 255.0 * uniforms.brightness_scale, 0.0, 1.0),
        clamp(in.color.a / 255.0, 0.0, 1.0),
    );
}
