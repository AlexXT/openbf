//! BF2's local lights on models (card BPNT, `analysis/re-point-lights.md`
//! §5–§7, [O]): the frame's light list ([`Renderer::set_point_lights`]), the
//! per-view filing, and the per-draw gather that picks each lit draw's ≤ 4
//! lights ([`gather`]). The term itself is the vertex shader's
//! (`point_sum` in `shader.wgsl`).
//!
//! - **Filing** (B2 `FUN_00597918`): a light is in the view's list when its
//!   range + 10 m sphere meets the frustum. The port keeps at most
//!   [`MAX_POINT_LIGHTS`], the nearest to the eye (OURS; no shipped view
//!   comes near it).
//! - **Gather** (B2 `FUN_00597e98`), per model draw from the model's sphere
//!   (centre c, radius R): an omni is a candidate when |c − L|² < (r + R)²,
//!   scored lum·min(1, r²/4d²)/max(1, e²/3600) — lum = 0.3R + 0.45G + 0.25B
//!   of the light's colour floats, d = |c − L|, e = |L − eye| — and the top 4
//!   are kept, ties in list order. A spot passes the same sphere test, then
//!   its own cone test ([`SpotCone`]), and only the best ONE is kept. A
//!   prelit set skips every Static light.
//! - **Upload** (B2 `FUN_0057fd90`): the omnis fill the four slots in score
//!   order; the spot — a point light to the VU, no cone — takes the first
//!   free slot, else replaces slot 3.
//!
//! DEVIATION, PROVISIONAL: the PS2 gathers once per MODEL; the caller hands
//! each draw its model's sphere ([`LitSphere`]), so a model's primitives
//! agree, but the sphere is the port's (`game::point_light`), not the
//! disc's.

use glam::Vec3;

use crate::Renderer;

/// Lights one view can hold: the shader's table is 256 records, record 0
/// the "no light" one.
pub const MAX_POINT_LIGHTS: usize = 255;

/// A one-way spot's axis and the sine of half its outer cone, for the
/// gather's test (the spot's vtable +0x1c, 0x59c3a8, read from the asm): a
/// model whose centre is BEHIND the spot — e = −D·(c − V) > 0 with
/// e² ≥ |c − V|²·sin² — takes it only when its sphere holds the apex,
/// |c − V|² ≤ R². Every other model in range takes it: the routine builds
/// its expanded-cone test from c in place of V, which always passes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpotCone {
    pub dir: Vec3,
    pub sin_half_outer: f32,
}

/// One omni or spot light of the frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub pos: Vec3,
    /// The colour the VU adds, in GS units (0x80 = 1.0): the packed byte /
    /// 128 — the authored 0..1 colour, doubled by `PS2Overbright`.
    pub color: [f32; 3],
    /// The colour floats' 0.3R + 0.45G + 0.25B, the gather's score weight
    /// (not doubled by Overbright).
    pub lum: f32,
    pub range: f32,
    /// A prelit draw skips it.
    pub is_static: bool,
    /// A spot: a draw takes at most one.
    pub spot: bool,
    /// A one-way spot's cone test; `None` passes every model in range.
    pub cone: Option<SpotCone>,
}

/// The sphere a lit draw's lights are gathered for: its MODEL's, in world
/// space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LitSphere {
    pub centre: Vec3,
    pub radius: f32,
}

/// The shader's record: (x, y, z, 1/r²), (r, g, b, 0).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PointRecord {
    pos_inv_r2: [f32; 4],
    color: [f32; 4],
}

/// Bytes of the shader's light table.
pub(crate) const POINT_TABLE_BYTES: u64 =
    ((MAX_POINT_LIGHTS + 1) * std::mem::size_of::<PointRecord>()) as u64;

/// The table for `filed`: record 0 zero, record i + 1 = `filed[i]`.
pub(crate) fn table(filed: &[PointLight]) -> Vec<PointRecord> {
    let mut out = vec![PointRecord::zeroed_record(); MAX_POINT_LIGHTS + 1];
    for (k, l) in filed.iter().take(MAX_POINT_LIGHTS).enumerate() {
        out[k + 1] = PointRecord {
            pos_inv_r2: [l.pos.x, l.pos.y, l.pos.z, 1.0 / (l.range * l.range)],
            color: [l.color[0], l.color[1], l.color[2], 0.0],
        };
    }
    out
}

impl PointRecord {
    fn zeroed_record() -> Self {
        bytemuck::Zeroable::zeroed()
    }
}

/// The view's list: every light with a range whose range + 10 m sphere
/// `in_view` accepts, the non-Static ones first (the PS2 files them at the
/// list's head), at most [`MAX_POINT_LIGHTS`] — the nearest to `eye`.
pub(crate) fn file(
    lights: &[PointLight],
    eye: Vec3,
    in_view: impl Fn(Vec3, f32) -> bool,
) -> Vec<PointLight> {
    let mut out: Vec<PointLight> = lights
        .iter()
        .filter(|l| l.range > 0.0 && in_view(l.pos, l.range + 10.0))
        .copied()
        .collect();
    if out.len() > MAX_POINT_LIGHTS {
        out.sort_by(|a, b| {
            a.pos
                .distance_squared(eye)
                .total_cmp(&b.pos.distance_squared(eye))
        });
        out.truncate(MAX_POINT_LIGHTS);
    }
    out.sort_by_key(|l| l.is_static);
    out
}

/// The light's score for a model `d2` (squared) metres from it, the light
/// `e2` (squared) from the eye.
fn score(l: &PointLight, d2: f32, e2: f32) -> f32 {
    let r2 = l.range * l.range;
    let mut s = l.lum;
    let mut div = 1.0;
    if r2 < 4.0 * d2 {
        s *= r2;
        div = 4.0 * d2;
    }
    let far = e2 / 3600.0;
    if far > 1.0 {
        div *= far;
    }
    s / div
}

/// Whether a spot's cone test passes the sphere (see [`SpotCone`]).
fn cone_takes(l: &PointLight, s: LitSphere) -> bool {
    let Some(c) = l.cone else {
        return true;
    };
    let d = s.centre - l.pos;
    let e = -c.dir.dot(d);
    let d2 = d.length_squared();
    if e > 0.0 && d2 * c.sin_half_outer * c.sin_half_outer <= e * e {
        return d2 <= s.radius * s.radius;
    }
    true
}

/// The four table records (0 = none) a draw of sphere `s` takes from the
/// view's list `filed` — the module doc's gather and upload.
pub(crate) fn gather(filed: &[PointLight], s: LitSphere, eye: Vec3, prelit: bool) -> [u16; 4] {
    let mut omnis: Vec<(f32, usize)> = Vec::new();
    let mut spot: Option<(f32, usize)> = None;
    for (k, l) in filed.iter().enumerate() {
        if prelit && l.is_static {
            continue;
        }
        let d2 = l.pos.distance_squared(s.centre);
        let reach = l.range + s.radius;
        if d2 >= reach * reach {
            continue;
        }
        let sc = score(l, d2, l.pos.distance_squared(eye));
        if !l.spot {
            omnis.push((sc, k));
        } else if cone_takes(l, s) && spot.is_none_or(|(best, _)| sc > best) {
            spot = Some((sc, k));
        }
    }
    // Best first; the stable sort keeps the list's order among equals.
    omnis.sort_by(|a, b| b.0.total_cmp(&a.0));
    omnis.truncate(4);
    let mut out = [0u16; 4];
    for (slot, (_, k)) in omnis.iter().enumerate() {
        out[slot] = *k as u16 + 1;
    }
    if let Some((_, k)) = spot {
        out[omnis.len().min(3)] = k as u16 + 1;
    }
    out
}

impl Renderer {
    /// This frame's local lights (BF2; none on BF1). Kept until set again;
    /// every draw carrying a [`LitSphere`] gathers from them.
    pub fn set_point_lights(&mut self, lights: Vec<PointLight>) {
        self.point_lights = lights;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn omni(x: f32, lum: f32, range: f32) -> PointLight {
        PointLight {
            pos: Vec3::new(x, 0.0, 0.0),
            color: [lum; 3],
            lum,
            range,
            is_static: false,
            spot: false,
            cone: None,
        }
    }

    const AT_ORIGIN: LitSphere = LitSphere {
        centre: Vec3::ZERO,
        radius: 1.0,
    };

    /// The sphere test, the score's order, the cap of four, and the
    /// Static skip of a prelit draw.
    #[test]
    fn the_gather_keeps_the_four_best_omnis_in_reach() {
        let mut lights: Vec<PointLight> = (0..6).map(|k| omni(k as f32, 1.0, 4.0)).collect();
        // Index 5 is 5 m away with reach 4 + 1: out. Index 4 is at d = 4:
        // r²/4d² = 0.25. Indices 0..=2 are within r/2 (score 1), 3 is 16/36.
        assert_eq!(gather(&lights, AT_ORIGIN, Vec3::ZERO, false), [1, 2, 3, 4]);
        lights[0].lum = 0.1;
        assert_eq!(gather(&lights, AT_ORIGIN, Vec3::ZERO, false), [2, 3, 4, 5]);
        lights[1].is_static = true;
        assert_eq!(gather(&lights, AT_ORIGIN, Vec3::ZERO, true), [3, 4, 5, 1]);
        // 120 m from the eye a light's score is a quarter; past r/2 from
        // the model it falls by r²/4d².
        let l = omni(0.0, 0.8, 4.0);
        assert_eq!(score(&l, 0.0, 0.0), 0.8);
        assert_eq!(score(&l, 0.0, 120.0 * 120.0), 0.2);
        assert_eq!(score(&l, 16.0, 0.0), 0.2);
    }

    /// One spot at most; it takes the first free slot, else slot 3; a model
    /// behind a one-way spot does not take it.
    #[test]
    fn the_gather_takes_one_spot_as_a_point() {
        let spot = |x: f32, lum: f32| PointLight {
            spot: true,
            ..omni(x, lum, 4.0)
        };
        let mut lights = vec![omni(0.0, 1.0, 4.0), spot(1.0, 0.5), spot(2.0, 0.9)];
        assert_eq!(gather(&lights, AT_ORIGIN, Vec3::ZERO, false), [1, 3, 0, 0]);
        lights.extend((0..4).map(|_| omni(0.5, 1.0, 4.0)));
        assert_eq!(gather(&lights, AT_ORIGIN, Vec3::ZERO, false), [1, 4, 5, 3]);
        // A spot 3 m up pointing up: the model is behind it.
        let up = PointLight {
            pos: Vec3::new(0.0, 3.0, 0.0),
            cone: Some(SpotCone {
                dir: Vec3::Y,
                sin_half_outer: 0.5,
            }),
            ..spot(0.0, 1.0)
        };
        assert_eq!(gather(&[up], AT_ORIGIN, Vec3::ZERO, false), [0; 4]);
        let down = PointLight {
            cone: Some(SpotCone {
                dir: Vec3::NEG_Y,
                sin_half_outer: 0.5,
            }),
            ..up
        };
        assert_eq!(gather(&[down], AT_ORIGIN, Vec3::ZERO, false), [1, 0, 0, 0]);
    }

    /// The filing drops what the view does not hold and puts the non-Static
    /// lights first.
    #[test]
    fn the_filing_orders_dynamic_lights_first() {
        let mut lights = vec![
            omni(0.0, 1.0, 4.0),
            omni(1.0, 1.0, 4.0),
            omni(50.0, 1.0, 4.0),
        ];
        lights[0].is_static = true;
        let f = file(&lights, Vec3::ZERO, |p, r| p.x.abs() < 20.0 + r);
        assert_eq!(f.len(), 2);
        assert!(!f[0].is_static && f[1].is_static);
        assert_eq!(table(&f)[1].pos_inv_r2, [1.0, 0.0, 0.0, 1.0 / 16.0]);
        assert_eq!(table(&f)[0].color, [0.0; 4]);
    }
}
