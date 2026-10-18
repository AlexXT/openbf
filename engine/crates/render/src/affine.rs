//! The PS2's vertex-colour blend (card BPRL, `analysis/re-vertex-colour-interp.md`
//! §3-§5 `[O]`), CPU side: the constants `shader.wgsl`'s `affine_w` reads and
//! its twin, with the referee test.
//!
//! The GS blends a triangle's vertex colour linearly in SCREEN space and the
//! VU1 clipper cuts a triangle only beyond the guard band (7.994 half-widths,
//! 9.1345 half-heights of the PS2's 512 x 448 view) or the near plane. The
//! port's model draws carry `a * wc` beside `wc` in perspective and divide per
//! pixel; `wc` is the vertex's depth, no nearer than where its view ray meets
//! the guard band. A named deviation from perspective shading, switched off by
//! `AFFINE_COLOUR` in the shader.

use crate::DrawLayer;

/// The guard band in the PS2's half-screens: 2047 / 256 half-widths and
/// 2046 / 224 half-heights (VU[85], §3).
pub const GUARD: [f32; 2] = [7.994, 9.1345];

/// The PS2 view's width over its height (tan 0.63707 / tan 0.48838, the
/// stored poses'): the half-width [`GUARD`]'s x counts in.
pub const PS2_ASPECT: f32 = 1.3045;

/// The near plane's floor on `wc`, metres.
pub const NEAR: f32 = 0.1;

/// [`crate::mesh::Instance::light`]'s bit for a draw whose pass takes the
/// blend; the low byte is the [`crate::LightPath`].
pub const AFFINE_BIT: u32 = 0x100;

/// [`AFFINE_BIT`] for a draw on `layer`: the model passes take the blend, the
/// water, its speckle, the far sheet and the terrain's layers do not (§8).
pub fn layer_bit(layer: DrawLayer) -> u32 {
    match layer {
        DrawLayer::Opaque | DrawLayer::Mask | DrawLayer::Blend | DrawLayer::Spec => AFFINE_BIT,
        _ => 0,
    }
}

/// The frame's `guard_x` / `guard_y`: what one unit of clip x / y (half-screens
/// x w) is worth against the guard band. The band is the PS2's on the screen
/// at any zoom (it comes from the viewport, not the view angle); a window
/// wider than the PS2's keeps the band's width in half-HEIGHTS, so hor+ does
/// not widen it.
pub fn guard(aspect: f32) -> [f32; 2] {
    [aspect / (PS2_ASPECT * GUARD[0]), 1.0 / GUARD[1]]
}

/// `affine_w`: a vertex's weight depth from its clip x, y and w.
pub fn guard_w(x: f32, y: f32, w: f32, guard: [f32; 2]) -> f32 {
    w.max(NEAR).max(x.abs() * guard[0]).max(y.abs() * guard[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    // UTA1 `walk/cp3_con_cp1_b000`, the PS2's pose (tan 0.637070 x 0.488380):
    // the one triangle of `uta1_bldg_lizardpen`'s floor fan that covers both
    // boxes below - its middle vertex 0.35 m under the post's light, under
    // the eye and 16 mm behind it. Per vertex: clip x, y (half-screens x w), w, and the light's
    // gain (`point_sum`'s scalar). From the disc through
    // `lead_tmp/bprl/export_vectors.py`.
    const TRI: [[f32; 4]; 3] = [
        [-15.1341, -3.6183, 7.9416, 0.0],
        [-0.1116, -3.7997, -0.0157, 0.97643],
        [-1.5258, -3.5927, 9.0655, 0.0],
    ];
    // (box on the 640 x 480 frame; the mean gain through `gsraster.py`'s PS2
    // model - the VU1 clip and the GS's affine blend - which predicts the
    // PS2's measured +1.2/+2.9 and +1.9/+4.8 G/B there (§4); the blue levels
    // one unit of gain is worth in the box.)
    const BOXES: [([u32; 4], f32, f32); 2] = [
        ([12, 384, 224, 470], 0.03911, 77.13),
        ([110, 436, 224, 470], 0.05121, 88.24),
    ];

    /// The box's mean gain with each vertex weighted by `wc(x, y, w)`:
    /// homogeneous barycentrics per pixel centre (no clip needed), value =
    /// sum(b wc a) / sum(b wc).
    fn mean_gain(b: [u32; 4], wc: impl Fn(f32, f32, f32) -> f32) -> f32 {
        let m = glam::Mat3::from_cols(
            glam::Vec3::new(TRI[0][0], TRI[0][1], TRI[0][2]),
            glam::Vec3::new(TRI[1][0], TRI[1][1], TRI[1][2]),
            glam::Vec3::new(TRI[2][0], TRI[2][1], TRI[2][2]),
        )
        .inverse();
        let w: Vec<f32> = TRI.iter().map(|v| wc(v[0], v[1], v[2])).collect();
        let (mut sum, mut n) = (0.0f64, 0u32);
        for py in b[1]..b[3] {
            for px in b[0]..b[2] {
                let ndc = glam::Vec3::new(
                    (px as f32 + 0.5) / 640.0 * 2.0 - 1.0,
                    1.0 - (py as f32 + 0.5) / 480.0 * 2.0,
                    1.0,
                );
                let beta = m * ndc;
                assert!(
                    beta.min_element() >= 0.0,
                    "pixel {px},{py} off the triangle"
                );
                let den = beta.x * w[0] + beta.y * w[1] + beta.z * w[2];
                let num = beta.x * w[0] * TRI[0][3]
                    + beta.y * w[1] * TRI[1][3]
                    + beta.z * w[2] * TRI[2][3];
                sum += f64::from(num / den);
                n += 1;
            }
        }
        (sum / f64::from(n)) as f32
    }

    #[test]
    fn the_pen_floor_takes_the_ps2s_share_of_the_posts_light() {
        let g = guard(PS2_ASPECT);
        for (b, ps2, blue) in BOXES {
            let port = mean_gain(b, |x, y, w| guard_w(x, y, w, g));
            assert!(
                ((port - ps2) * blue).abs() < 0.5,
                "box {b:?}: {port} against the PS2 model's {ps2}"
            );
            // The mutation: a perspective blend (equal weights) is tens of
            // levels over.
            let persp = mean_gain(b, |_, _, _| 1.0);
            assert!(
                (persp - ps2) * blue > 20.0,
                "box {b:?}: perspective {persp}"
            );
        }
    }

    #[test]
    fn a_vertex_in_view_weighs_its_depth_and_one_off_the_band_its_crossing() {
        let g = guard(16.0 / 9.0);
        // In view: |x|, |y| <= w.
        assert_eq!(guard_w(3.0, -2.0, 5.0, g), 5.0);
        // Far off to the side at no depth: where its ray meets the band.
        let x = 100.0;
        assert!((guard_w(x, 0.0, 0.01, g) - x * g[0]).abs() < 1e-6);
        // Behind the eye: never under the near plane, never negative.
        assert_eq!(guard_w(0.0, 0.0, -4.0, g), NEAR);
        // A wider window keeps the band where it is in the world.
        let wide = guard(21.0 / 9.0);
        assert!((wide[0] / g[0] - (21.0 / 16.0)).abs() < 1e-6);
        assert_eq!(wide[1], g[1]);
    }

    #[test]
    fn the_shader_reads_the_same_constants() {
        let wgsl = include_str!("shader.wgsl");
        assert!(wgsl.contains(&format!("const AFFINE_BIT: u32 = {AFFINE_BIT:#x}u;")));
        assert!(wgsl.contains(&format!("const AFFINE_NEAR: f32 = {NEAR};")));
        assert!(wgsl.contains("const AFFINE_COLOUR: bool = true;"));
    }
}
