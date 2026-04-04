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
    g = g + r_bleed * 0.7;

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
    let linear = with_mul / 255.0;
    return aces_curve(linear);
}

fn tonemap_reinhard(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);
    let with_add = hdr_color + add_color * uniforms.game_exposure;
    let with_mul = with_add * mul_color;
    let lum = dot(with_mul, vec3<f32>(0.2126, 0.7152, 0.0722));
    let mapped_lum = lum / (1.0 + lum / 255.0);
    let scale = select(mapped_lum / lum, 0.0, lum < 0.001);
    return clamp(with_mul * scale / 255.0, vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(offscreen_texture, offscreen_sampler, in.uv);
    var mapped: vec3<f32>;
    if (TONEMAP_VARIANT == 0u) { mapped = hdr_color.rgb; }
    else if (TONEMAP_VARIANT == 1u) { mapped = tonemap_faithful(hdr_color.rgb); }
    else if (TONEMAP_VARIANT == 2u) { mapped = tonemap_spectral_bleed(hdr_color.rgb); }
    else if (TONEMAP_VARIANT == 3u) { mapped = tonemap_aces(hdr_color.rgb); }
    else { mapped = tonemap_reinhard(hdr_color.rgb); }
    return vec4<f32>(mapped, 1.0);
}