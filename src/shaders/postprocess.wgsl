struct PostProcessUniforms {
    game_exposure: f32,
    add_color_r: f32,
    add_color_g: f32,
    add_color_b: f32,
    mul_color_r: f32,
    mul_color_g: f32,
    mul_color_b: f32,
    hdr_enabled: f32,
    exposure: f32,
    max_brightness: f32,
    tonemap_variant: f32,
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

fn tonemap_pseudo_reinhard(color: vec3<f32>, max_val: f32) -> vec3<f32> {
    return color * max_val / (color + vec3(max_val));
}

fn hard_redirect(col: vec3<f32>, threshold: f32) -> vec3<f32> {
    let double = threshold * 2.0;

    var r = col.r;
    var g = col.g;
    var b = col.b;

    // Red channel redistribution
    var r_out = r;
    if b > double { r_out = r + b - double; }
    else if b > threshold && g > threshold { r_out = r + b + g - threshold * 2.0; }
    else if g > threshold { r_out = r + g - threshold; }

    // Green channel redistribution
    var g_out = g;
    if r > threshold && b > threshold { g_out = g + r + b - threshold * 2.0; }
    else if r > threshold { g_out = g + r - threshold; }
    else if b > threshold { g_out = g + b - threshold; }

    // Blue channel redistribution
    var b_out = b;
    if r > double { b_out = b + r - double; }
    else if r > threshold && g > threshold { b_out = b + r + g - threshold * 2.0; }
    else if g > threshold { b_out = b + g - threshold; }

    return clamp(vec3<f32>(r_out, g_out, b_out), vec3<f32>(0.0), vec3<f32>(threshold));
}

fn tonemap(hdr_color: vec3<f32>) -> vec3<f32> {
    let add_color = vec3<f32>(uniforms.add_color_r, uniforms.add_color_g, uniforms.add_color_b);
    let mul_color = vec3<f32>(uniforms.mul_color_r, uniforms.mul_color_g, uniforms.mul_color_b);
    let with_add = hdr_color + add_color;
    let with_mul = with_add * mul_color * uniforms.game_exposure * uniforms.exposure;
    let variant = u32(uniforms.tonemap_variant);

    if uniforms.hdr_enabled > 0.5 {
        // Convert to nits: game 255 = 200 nits reference white (fixed constant)
        let nits = with_mul * (200.0 / 255.0);
        // Tonemap in nits space, then convert to scRGB (1.0 = 80 nits)
        if variant == 0u { let r = clamp(nits, vec3(0.0), vec3(uniforms.max_brightness)); return r / 80.0; }
        if variant == 1u { let r = tonemap_pseudo_reinhard(nits, uniforms.max_brightness); return r / 80.0; }
        if variant == 2u { let r = hard_redirect(nits, uniforms.max_brightness); return r / 80.0; }
        let r = soft_redirect(nits, uniforms.max_brightness); return r / 80.0;
    } else {
        // SDR: tonemap with threshold=255, then normalize to [0,1]
        if variant == 0u { return clamp(with_mul, vec3(0.0), vec3(255.0)) / 255.0; }
        if variant == 1u { return tonemap_pseudo_reinhard(with_mul, 255.0) / 255.0; }
        if variant == 2u { return hard_redirect(with_mul, 255.0) / 255.0; }
        return soft_redirect(with_mul, 255.0) / 255.0;
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(offscreen_texture, offscreen_sampler, in.uv);
    let mapped = tonemap(hdr_color.rgb);
    return vec4<f32>(mapped, 1.0);
}
