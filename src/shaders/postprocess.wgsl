struct PostProcessUniforms {
    game_exposure: f32,
    add_color_r: f32,
    add_color_g: f32,
    add_color_b: f32,
    mul_color_r: f32,
    mul_color_g: f32,
    mul_color_b: f32,
    _padding: f32,
}

@group(0) @binding(0) var offscreen_texture: texture_2d<f32>;
@group(0) @binding(1) var offscreen_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: PostProcessUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) / 2.0, (1.0 - y) / 2.0);
    return out;
}

// Soft spectral redirect with 80% shoulder.
// Below soft_start: passthrough (preserves full saturation and WCG).
// Between soft_start and threshold: smoothstep bleed into spectral neighbors.
// Above threshold: full redistribution.
fn soft_redirect(col: vec3<f32>, threshold: f32) -> vec3<f32> {
    let soft_start = threshold * 0.8;

    // Per-channel excess and blend factor
    let excess_r = max(col.r - soft_start, 0.0);
    let excess_g = max(col.g - soft_start, 0.0);
    let excess_b = max(col.b - soft_start, 0.0);

    let blend_r = smoothstep(soft_start, threshold, col.r);
    let blend_g = smoothstep(soft_start, threshold, col.g);
    let blend_b = smoothstep(soft_start, threshold, col.b);

    // Bleed into spectral neighbors (r<->g<->b circular)
    // Primary neighbor gets 60% of excess, secondary gets 30%
    var r_out = col.r;
    var g_out = col.g;
    var b_out = col.b;

    // Red excess -> green (primary), blue (secondary)
    let r_bleed = excess_r * blend_r;
    r_out = r_out - r_bleed * 0.9;
    g_out = g_out + r_bleed * 0.6;
    b_out = b_out + r_bleed * 0.3;

    // Green excess -> red (primary), blue (primary -- green is spectrally between)
    let g_bleed = excess_g * blend_g;
    g_out = g_out - g_bleed * 0.9;
    r_out = r_out + g_bleed * 0.45;
    b_out = b_out + g_bleed * 0.45;

    // Blue excess -> green (primary), red (secondary)
    let b_bleed = excess_b * blend_b;
    b_out = b_out - b_bleed * 0.9;
    g_out = g_out + b_bleed * 0.6;
    r_out = r_out + b_bleed * 0.3;

    return clamp(vec3<f32>(r_out, g_out, b_out), vec3<f32>(0.0), vec3<f32>(threshold));
}

fn tonemap(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);

    let with_add = hdr_color + add_color * uniforms.game_exposure;
    let with_mul = with_add * mul_color;

    let threshold = 255.0;
    let redirected = soft_redirect(with_mul, threshold);

    return redirected / threshold;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(offscreen_texture, offscreen_sampler, in.uv);
    let mapped = tonemap(hdr_color.rgb);
    return vec4<f32>(mapped, 1.0);
}
