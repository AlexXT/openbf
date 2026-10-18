// ── The DomeInfo frame filter (`filter.rs`, card BFLT) ──────────────────────
// `analysis/re-glow.md` §3 [O]: over the finished 3D frame, per channel in
// display bytes, out = min(1, v + gain · max(0, lowpass(v) − threshold)).
// The frame's copy is read with `textureLoad` and encoded to display units
// by hand (the copy has the target's sRGB format), so the box averages the
// GS's bytes, not linear light.
struct FilterParams {
    // x = threshold (0..1 of a display byte 255), y = gain (Intensity/128),
    // z = the box's side in pixels, w unused.
    p: vec4<f32>,
};
@group(0) @binding(0) var t_frame: texture_2d<f32>;
@group(0) @binding(1) var<uniform> fp: FilterParams;

fn f_linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn f_srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

// One triangle over the whole target.
@vertex
fn vs_filter(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_filter(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let size = vec2<i32>(textureDimensions(t_frame));
    let px = vec2<i32>(pos.xy);
    // The PS2's 2:1 box at 256×224 is a box of height/224 output pixels
    // here, centred on the pixel (its sub-pixel blur is below one PS2 pixel).
    let n = i32(fp.p.z);
    let lo = -(n / 2);
    var sum = vec3<f32>(0.0);
    for (var y = 0; y < n; y++) {
        for (var x = 0; x < n; x++) {
            let q = clamp(px + vec2<i32>(lo + x, lo + y), vec2<i32>(0), size - 1);
            sum += f_linear_to_srgb(textureLoad(t_frame, q, 0).rgb);
        }
    }
    let low = sum / f32(n * n);
    let here = textureLoad(t_frame, px, 0);
    let v = f_linear_to_srgb(here.rgb);
    let add = fp.p.y * max(low - vec3<f32>(fp.p.x), vec3<f32>(0.0));
    // Below the threshold the pixel is left as it was, bit for bit.
    if all(add == vec3<f32>(0.0)) {
        return here;
    }
    let out = min(v + add, vec3<f32>(1.0));
    return vec4<f32>(f_srgb_to_linear(out), 1.0);
}
