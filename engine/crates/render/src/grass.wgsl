
// ── The grass patches (`grass.rs`, queue FOLGG) and the leaf patches
// (`leaf.rs`, card LEAF), which ride the same pass ─────────────────────────
// Appended to `shader.wgsl` as one module, so it reads the same `globals`,
// `t_color` / `s_color`, fog and far fade. A vertex is a corner of the
// class's per-view sprite buffer in the PATCH's frame; the instance is the
// patch: its matrix and its draw colour (the FOLG tint, alpha × the 20 m
// fade). The PS2's GS MODULATE: texture × the sprite's grey × the patch
// colour, unlit (re-foliage §7). The multiply happens on the PS2's gamma
// bytes, so the colour factors go to linear before they scale a linear
// texel (the terrain path's rule).

struct GrassIn {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) up: f32,
    @location(4) m0: vec4<f32>,
    @location(5) m1: vec4<f32>,
    @location(6) m2: vec4<f32>,
    @location(7) m3: vec4<f32>,
    @location(8) inst_color: vec4<f32>,
    @location(9) params: vec4<f32>,
};

struct GrassOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) view_dist: f32,
    @location(3) fog_depth: f32,
    @location(4) light: vec3<f32>,
    @location(5) @interpolate(flat) params: vec4<f32>,
};

@vertex
fn vs_grass(in: GrassIn) -> GrassOut {
    let model = mat4x4<f32>(in.m0, in.m1, in.m2, in.m3);
    let world = model * vec4<f32>(in.position, 1.0);
    var out: GrassOut;
    out.clip_pos = globals.view_proj * world;
    out.uv = in.uv;
    out.color = in.color * in.inst_color;
    out.view_dist = distance(world.xyz, globals.camera_pos);
    out.fog_depth = out.clip_pos.w;
    out.params = in.params;
    out.light = vec3<f32>(1.0);
    // A BF2 leaf (`leaf.rs`, PROVISIONAL): the static `Lit` set on the
    // quad's ±up normal through its object's matrix, per vertex as the VU.
    if in.params.x > 0.5 {
        let n = normalize((model * vec4<f32>(0.0, in.up, 0.0, 0.0)).xyz);
        let ambient = mix(globals.ambient_ground, globals.ambient_sky, n.y * 0.5 + 0.5);
        let sun = globals.sun_color * max(dot(n, -globals.sun_dir), 0.0);
        let light2 = globals.light2_color * max(dot(n, -normalize(globals.light2_dir)), 0.0);
        out.light = ambient + sun;
        if any(globals.light2_color != vec3<f32>(0.0)) {
            out.light = ambient + sun + light2;
        }
    }
    return out;
}

@fragment
fn fs_grass(in: GrassOut) -> @location(0) vec4<f32> {
    let texel = textureSample(t_color, s_color, in.uv);
    var a = texel.a * in.color.a * far_fade(in.view_dist);
    if in.params.y > 0.5 {
        a = texel.a * in.color.a;
    }
    if a <= 0.0 {
        discard;
    }
    var rgb = min(texel.rgb * srgb_to_linear(min(in.color.rgb, vec3<f32>(1.0))), vec3<f32>(1.0));
    if in.params.x > 0.5 {
        rgb = rgb * in.light;
    }
    return vec4<f32>(apply_fog(rgb, in.fog_depth), a);
}
