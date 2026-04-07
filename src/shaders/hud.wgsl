struct HudUniforms {
    screen_width: f32,
    screen_height: f32,
    brightness_scale: f32,
    hdr_enabled: f32,
    max_brightness: f32,
    tonemap_variant: f32,
    exposure: f32,
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

fn hud_tonemap_pseudo_reinhard(color: vec3<f32>, max_val: f32) -> vec3<f32> {
    return color * max_val / (color + vec3(max_val));
}

fn hud_soft_redirect(col: vec3<f32>, threshold: f32) -> vec3<f32> {
    let soft_start = threshold * 0.8;
    let excess_r = max(col.r - soft_start, 0.0);
    let excess_g = max(col.g - soft_start, 0.0);
    let excess_b = max(col.b - soft_start, 0.0);
    let blend_r = smoothstep(soft_start, threshold, col.r);
    let blend_g = smoothstep(soft_start, threshold, col.g);
    let blend_b = smoothstep(soft_start, threshold, col.b);
    var r_out = col.r; var g_out = col.g; var b_out = col.b;
    let r_bleed = excess_r * blend_r;
    r_out = r_out - r_bleed * 0.9; g_out = g_out + r_bleed * 0.6; b_out = b_out + r_bleed * 0.3;
    let g_bleed = excess_g * blend_g;
    g_out = g_out - g_bleed * 0.9; r_out = r_out + g_bleed * 0.45; b_out = b_out + g_bleed * 0.45;
    let b_bleed = excess_b * blend_b;
    b_out = b_out - b_bleed * 0.9; g_out = g_out + b_bleed * 0.6; r_out = r_out + b_bleed * 0.3;
    return clamp(vec3<f32>(r_out, g_out, b_out), vec3<f32>(0.0), vec3<f32>(threshold));
}

fn hud_hard_redirect(col: vec3<f32>, threshold: f32) -> vec3<f32> {
    let double = threshold * 2.0;
    var r = col.r; var g = col.g; var b = col.b;
    var r_out = r;
    if b > double { r_out = r + b - double; }
    else if b > threshold && g > threshold { r_out = r + b + g - threshold * 2.0; }
    else if g > threshold { r_out = r + g - threshold; }
    var g_out = g;
    if r > threshold && b > threshold { g_out = g + r + b - threshold * 2.0; }
    else if r > threshold { g_out = g + r - threshold; }
    else if b > threshold { g_out = g + b - threshold; }
    var b_out = b;
    if r > double { b_out = b + r - double; }
    else if r > threshold && g > threshold { b_out = b + r + g - threshold * 2.0; }
    else if g > threshold { b_out = b + g - threshold; }
    return clamp(vec3<f32>(r_out, g_out, b_out), vec3<f32>(0.0), vec3<f32>(threshold));
}

fn hud_apply_tonemap(raw: vec3<f32>) -> vec3<f32> {
    let variant = u32(uniforms.tonemap_variant);
    if uniforms.hdr_enabled > 0.5 {
        let nits = raw * 80.0;
        let max_b = uniforms.max_brightness;
        var tonemapped: vec3<f32>;
        if variant == 0u { tonemapped = clamp(nits, vec3(0.0), vec3(max_b)); }
        else if variant == 1u { tonemapped = hud_tonemap_pseudo_reinhard(nits, max_b); }
        else if variant == 2u { tonemapped = hud_hard_redirect(nits, max_b); }
        else { tonemapped = hud_soft_redirect(nits, max_b); }
        return tonemapped / 80.0;
    } else {
        if variant == 0u { return clamp(raw, vec3(0.0), vec3(1.0)); }
        else if variant == 1u { return hud_tonemap_pseudo_reinhard(raw, 1.0); }
        else if variant == 2u { return hud_hard_redirect(raw, 1.0); }
        else { return hud_soft_redirect(raw, 1.0); }
    }
}

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
    let raw = vec3<f32>(
        in.color.r / 255.0 * uniforms.brightness_scale,
        in.color.g / 255.0 * uniforms.brightness_scale,
        in.color.b / 255.0 * uniforms.brightness_scale,
    );
    let alpha = clamp(in.color.a / 255.0, 0.0, 1.0);
    let mapped = hud_apply_tonemap(raw);
    return vec4<f32>(mapped, alpha);
}
