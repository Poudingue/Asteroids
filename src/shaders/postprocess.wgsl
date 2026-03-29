const TONEMAP_VARIANT: u32 = 1u;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

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

@group(0) @binding(0) var offscreen_texture: texture_2d<f32>;
@group(0) @binding(1) var offscreen_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: PostProcessUniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index & 1u)) * 4.0 - 1.0;
    let y = f32(i32(vertex_index >> 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

fn redirect_spectre_wide(col: vec3<f32>) -> vec3<f32> {
    var r = col.r;
    var g = col.g;
    var b = col.b;

    var r_out = r;
    if (b > 510.0) {
        if (g > 255.0) { r_out = r + g + b - 510.0 - 255.0; }
        else { r_out = r + b - 510.0; }
    } else {
        if (g > 255.0) { r_out = r + g - 255.0; }
    }

    var g_out = g;
    if (b > 255.0 && r > 255.0) { g_out = g + r + b - 510.0; }
    else if (r > 255.0) { g_out = g + r - 255.0; }
    else if (b > 255.0) { g_out = g + b - 255.0; }

    var b_out = b;
    if (r > 510.0) {
        if (g > 255.0) { b_out = r + g + b - 510.0 - 255.0; }
        else { b_out = r + b - 510.0; }
    } else {
        if (g > 255.0) { b_out = g + b - 255.0; }
    }

    return vec3<f32>(r_out, g_out, b_out);
}

fn tonemap_faithful(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);
    let with_add = hdr_color + add_color * uniforms.game_exposure;
    let with_mul = with_add * mul_color;
    let redirected = redirect_spectre_wide(with_mul);
    return clamp(redirected, vec3<f32>(0.0), vec3<f32>(255.0)) / 255.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(offscreen_texture, offscreen_sampler, in.uv);
    if (TONEMAP_VARIANT == 0u) { return hdr_color; }
    else if (TONEMAP_VARIANT == 1u) {
        let mapped = tonemap_faithful(hdr_color.rgb);
        return vec4<f32>(mapped, hdr_color.a);
    }
    return hdr_color;
}
