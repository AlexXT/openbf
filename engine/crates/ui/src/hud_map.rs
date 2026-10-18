//! BF2's minimap — the `Map("player1map")` element's Small view (card BMAP,
//! `analysis/re-hud-map.md`): [`MapModel`] is what the game feeds per frame,
//! and the draw lays it out inside [`crate::hud_draw`]'s frame like any other
//! element.
//!
//! The PS2's rule (§2 [V][O]): the map is centred on the camera's eye and
//! turned so the camera's forward is up; a world point (X, Z) sits at
//! (u, v) = ((X − eye.x), (Z − eye.z)) · 0.5 / R of a 0.22 × 448 px square,
//! R the controlled class's view radius; the group is scaled by the layout's
//! 0.9 and placed at its Position. Posts beyond the circle are pulled onto
//! its rim, units beyond it are hidden. The backdrop is the map's image over
//! the whole-map square, cut to the `map_mask` disc.
//!
//! Not drawn yet (the card's `resume:`): the Large and Spawn views, the
//! Lua markers, the PostSelect ring, the vehicle / turret direction mesh and
//! the Target element's world markers.
//!
//! `[ours]`: the mask is a 64-texel disc with a one-texel soft rim; the port
//! draws the disc as a fan with that rim instead of a second texture stage.

use crate::hud_draw::{
    anchor_shift, color, frame_scale, mesh_frame, Ctx, HudVertex, Quad, Run, Screen, OVERFILL,
};
use crate::hud_tree::{Elem, HAlign, Hud, Kind, FRAME_H, FRAME_W};
use glam::{Mat4, Vec3};

/// The Small backdrop's side in frame pixels (§1 [V]: 0.22 screen heights,
/// ELF 0x5ff8c0; RAM rect ± 49.28).
pub const BACKDROP: f32 = 0.22 * FRAME_H;
/// A post / unit / north sprite before its scale (§1 [V] 0x62fb20).
pub const ICON: f32 = 6.0;
/// `BackdropSmall`'s alpha, `PlayerFOVSmall`'s Scale and `NorthSmall`'s Scale
/// in the `1playerhud` layout (the Map's children are its own records; the
/// tree keeps only the element's Position, Scale, alpha and z).
const BACKDROP_ALPHA: f32 = 0.5;
const FOV_SCALE: f32 = 1.15;
const NORTH_SCALE: f32 = 2.0;
/// `[H]` the post number's font: the layout's `PostTextSmall` names none.
const NUMBER_FONT: &str = "gamefont_tiny";
const NUMBER_DROP: f32 = 4.0;
/// The disc's fan and the mask's soft rim as a share of the radius
/// (one texel of the 64-texel `map_mask`).
const FAN: usize = 64;
const RIM: f32 = 1.0 / 32.0;

/// The post colours by owner seen from the local team (§3 [V]: friend
/// 0xffd55601, enemy DAT_0076c228, neutral DAT_0076c208 — RAM values).
pub const FRIEND: [u8; 3] = [1, 86, 213];
pub const ENEMY: [u8; 3] = [200, 100, 100];
pub const NEUTRAL: [u8; 3] = [192, 192, 192];

/// One sprite on the map.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapIcon {
    /// World (x, z).
    pub pos: [f32; 2],
    /// The class's `MapTexture` sheet.
    pub texture: String,
    /// The class's `MapScale`.
    pub scale: f32,
    pub color: [u8; 3],
    /// [`heading`] of the thing's forward; `None` = kept upright (a fixed
    /// command post).
    pub heading: Option<f32>,
    /// A post's number, printed above it.
    pub number: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapModel {
    /// Where the map image's sheet lives (the map's own bake), for the
    /// loader.
    pub sheet_dir: Option<std::path::PathBuf>,
    /// `<world>_map`; `None` draws no backdrop.
    pub image: Option<String>,
    /// The whole-map square: centre (x, z) and its SIGNED half side (§2.3:
    /// TAN1's is negative and mirrors the image).
    pub centre: [f32; 2],
    pub extent: f32,
    /// The north pointer's own turn, radians: the mission's
    /// `SetMapNorthAngle` when its flag is set (§2.2), else 0.
    pub north: f32,
    /// The camera's eye (x, z) and [`heading`].
    pub eye: [f32; 2],
    pub heading: f32,
    /// The view radius in metres ([`view_radius`]).
    pub radius: f32,
    pub posts: Vec<MapIcon>,
    pub units: Vec<MapIcon>,
}

/// θ of a horizontal forward (x, z): the PS2's atan2(−back.x, back.z)
/// (§2.2 [V] D:400174). 0 looks down −Z, which is up on the map image.
pub fn heading(forward: [f32; 2]) -> f32 {
    forward[0].atan2(-forward[1])
}

/// R from a class's `[MapViewMin, MapViewMax, MapSpeedMin, MapSpeedMax]` and
/// the horizontal speed (§2.1 [V][O] D:400131–400154).
pub fn view_radius(view: [f32; 4], speed: f32) -> f32 {
    let [min, max, s0, s1] = view;
    let t = if s1 > s0 {
        ((speed - s0) / (s1 - s0)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    min + (max - min) * t
}

/// The entity-class defaults of the four view keys (§2.1 [V] ctor
/// `FUN_001e22a8`).
pub const VIEW_DEFAULT: [f32; 4] = [125.0, 250.0, 10.0, 100.0];

/// A post's colour while `capturing` differs from its owner (§3 [V]
/// D:401520–401545): a 1 Hz blink toward the capturing team's colour, up in
/// the first eighth of the second and down in the next. `clock` is seconds.
pub fn post_color(owner: [u8; 3], capturing: Option<[u8; 3]>, clock: f32) -> [u8; 3] {
    let Some(to) = capturing else {
        return owner;
    };
    let f = clock.rem_euclid(1.0);
    let w = if f < 0.125 {
        8.0 * f
    } else if f < 0.25 {
        8.0 * (0.25 - f)
    } else {
        0.0
    };
    [0, 1, 2].map(|c| (f32::from(owner[c]) + (f32::from(to[c]) - f32::from(owner[c])) * w) as u8)
}

impl MapModel {
    /// (u, v) of a world point in the unit square and its squared distance
    /// from the centre BEFORE the clamp (`FUN_003a5228` §2.1 [V][O]); beyond
    /// the circle the point is pulled onto the rim (`clamp`) or `None`.
    pub fn place(&self, p: [f32; 2], clamp: bool) -> Option<([f32; 2], f32)> {
        let k = 0.5 / self.radius;
        let (mut u, mut v) = ((p[0] - self.eye[0]) * k, (p[1] - self.eye[1]) * k);
        let d2 = u * u + v * v;
        if d2 > 0.25 {
            if !clamp {
                return None;
            }
            let s = 0.5 / d2.sqrt();
            u *= s;
            v *= s;
        }
        Some(([u, v], d2))
    }

    /// The map image's texture coordinate at (u, v) of the unit square
    /// (`FUN_003a5438` §2.3 [V][O]).
    pub fn image_uv(&self, uv: [f32; 2]) -> [f32; 2] {
        let k = 0.5 / self.extent;
        [0, 1].map(|a| (self.eye[a] + uv[a] * 2.0 * self.radius - self.centre[a]) * k + 0.5)
    }

    /// The element's group matrix under `base` (its Position and Scale): the
    /// rotation that puts the camera's forward up (§2.2 [V][O]).
    pub fn group(&self, base: &Mat4) -> Mat4 {
        *base * Mat4::from_rotation_z(-self.heading)
    }

    /// Where a post's icon centre lands in the group (frame pixels before
    /// the group matrix).
    pub fn local(&self, uv: [f32; 2]) -> Vec3 {
        Vec3::new(uv[0] * BACKDROP, uv[1] * BACKDROP, 0.0)
    }
}

/// The window pixel a world point's icon centre is drawn at in a `size`
/// window (posts `clamp` to the rim; a unit beyond it is `None`) — the same
/// mapping [`crate::hud_draw::draw`] lays the Map element out by.
pub fn window_point(hud: &Hud, world: [f32; 2], clamp: bool, size: [f32; 2]) -> Option<[f32; 2]> {
    let map = hud.map.as_ref()?;
    let e = hud
        .roots
        .iter()
        .map(|r| &hud.elems[*r])
        .find(|e| e.kind == Kind::Map)?;
    let (uv, _) = map.place(world, clamp)?;
    let p = map.group(&e.local()).transform_point3(map.local(uv));
    let sc = frame_scale(size);
    Some([
        OVERFILL[0].0 * 4.0 / 3.0 * size[1] + anchor_shift(e.pos.x / FRAME_W, size) + sc[0] * p.x,
        OVERFILL[1].0 * size[1] + sc[1] * p.y,
    ])
}

impl Ctx<'_> {
    /// The Map element (`i`, `e`) under `parent`.
    pub(crate) fn map(&mut self, i: usize, e: &Elem, parent: &Mat4, s: &Screen, alpha: f32) {
        let hud = self.hud;
        let Some(map) = hud.map.as_ref() else {
            return;
        };
        if map.radius <= 0.0 {
            return;
        }
        let base = *parent * e.local();
        let group = map.group(&base);

        // Back to front: the backdrop (z 255), the FOV mesh, posts (128),
        // north and numbers (127), units (126). The layout gives the mesh z 1
        // — in front of everything — but the PS2 frames show a post at the
        // eye undimmed by the mesh's dark part ([M] HOT1 / TAN1 walkabout
        // cp frames), so it goes under the sprites.
        if let (Some(image), true) = (map.image.as_deref(), map.extent != 0.0) {
            self.disc(i, map, &group, s, image, alpha * BACKDROP_ALPHA);
        }
        // The wedge: at the eye, up (§3 [V] `FUN_003a5650`). `hud_map_fov`
        // lies in its own XZ plane, +Z the view's way: stood up so +Z is the
        // frame's up ([D] the mesh's bounds; the PS2 frames show the dark
        // part below and beside the wedge).
        let fov = base
            * Mat4::from_scale(Vec3::splat(FOV_SCALE))
            * mesh_frame()
            * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        self.mesh(i, "hud_map_fov", &fov, s, color([255.0; 3], alpha), false);
        for p in &map.posts {
            let Some((uv, _)) = map.place(p.pos, true) else {
                continue;
            };
            // A fixed post stays upright on screen; a walker's turns.
            let turn = p.heading.unwrap_or(map.heading);
            self.icon_at(i, &group, s, map.local(uv), turn, p, alpha);
        }
        // The pointer sits on the backdrop's top edge — the image's −Z side —
        // and turns with the group.
        let north = group
            * Mat4::from_rotation_z(map.north)
            * Mat4::from_translation(Vec3::new(0.0, -BACKDROP * 0.5, 0.0));
        let h = ICON * NORTH_SCALE * 0.5;
        self.quad(
            i,
            &north,
            s,
            &Quad {
                rect: [-h, -h, h, h],
                uv: [0.0, 0.0, 1.0, 1.0],
                color: color([255.0; 3], alpha),
                texture: "hud_map_northpointer",
                additive: false,
                shadow: true,
                boxed: false,
            },
        );
        if let Some(font) = self.art.fonts.get(NUMBER_FONT) {
            for p in &map.posts {
                let (Some(n), Some((uv, d2))) = (p.number, map.place(p.pos, true)) else {
                    continue;
                };
                // Fades out toward the rim (§3 [V] D:401671).
                let a = alpha * (1.0 - 4.0 * d2);
                if a <= 0.0 {
                    continue;
                }
                let at = group.transform_point3(map.local(uv));
                let m = Mat4::from_translation(at);
                // Bottom-aligned on the icon's top edge; the PS2's digit
                // touches the ring ([M] HOT1 cp1 frame: 4 px lower than
                // the font's line box puts it — PROVISIONAL).
                let top =
                    -ICON * p.scale * 0.5 * e.scale.y - f32::from(font.line_height) + NUMBER_DROP;
                self.run(
                    i,
                    &m,
                    s,
                    font,
                    &Run {
                        text: &n.to_string(),
                        at: (0.0, top),
                        h_align: HAlign::Center,
                        scale: [1.0, 1.0],
                        color: color([255.0; 3], a),
                        additive: false,
                        shadow: false,
                    },
                );
            }
        }
        for u in &map.units {
            let Some((uv, _)) = map.place(u.pos, false) else {
                continue;
            };
            let turn = u.heading.unwrap_or(map.heading);
            self.icon_at(i, &group, s, map.local(uv), turn, u, alpha);
        }
    }

    /// One sprite at `at` of the group, turned `turn` in the map's plane.
    #[allow(clippy::too_many_arguments)]
    fn icon_at(
        &mut self,
        i: usize,
        group: &Mat4,
        s: &Screen,
        at: Vec3,
        turn: f32,
        icon: &MapIcon,
        alpha: f32,
    ) {
        let m = *group * Mat4::from_translation(at) * Mat4::from_rotation_z(turn);
        let h = ICON * icon.scale * 0.5;
        self.quad(
            i,
            &m,
            s,
            &Quad {
                rect: [-h, -h, h, h],
                uv: [0.0, 0.0, 1.0, 1.0],
                color: color(icon.color.map(f32::from), alpha),
                texture: &icon.texture,
                additive: false,
                shadow: true,
                boxed: false,
            },
        );
    }

    /// The backdrop: the map image inside the mask's disc.
    fn disc(
        &mut self,
        i: usize,
        map: &MapModel,
        group: &Mat4,
        s: &Screen,
        image: &str,
        alpha: f32,
    ) {
        let first = self.out.indices.len() as u32;
        let base = self.out.verts.len() as u32;
        let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        let mut push = |ctx: &mut Self, uv: [f32; 2], a: f32| {
            let w = s.map(group.transform_point3(map.local(uv)));
            b = [
                b[0].min(w[0]),
                b[1].min(w[1]),
                b[2].max(w[0]),
                b[3].max(w[1]),
            ];
            ctx.out.verts.push(HudVertex {
                pos: [w[0], w[1], 0.0],
                uv: map.image_uv(uv),
                color: [1.0, 1.0, 1.0, a.clamp(0.0, 1.0)],
            });
        };
        push(self, [0.0, 0.0], alpha);
        for k in 0..FAN {
            let (sn, cs) = (k as f32 / FAN as f32 * std::f32::consts::TAU).sin_cos();
            push(
                self,
                [0.5 * (1.0 - RIM) * cs, 0.5 * (1.0 - RIM) * sn],
                alpha,
            );
            push(self, [0.5 * cs, 0.5 * sn], 0.0);
        }
        for k in 0..FAN as u32 {
            let n = (k + 1) % FAN as u32;
            let (i0, o0, i1, o1) = (1 + 2 * k, 2 + 2 * k, 1 + 2 * n, 2 + 2 * n);
            self.out
                .indices
                .extend([0, i0, i1, i0, o0, o1, i0, o1, i1].map(|v| base + v));
        }
        self.out.batch(image, false, false, first);
        self.out.boxes.push((i, b));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `re-hud-map.md` §7: the PS2's own post icons at HOT1 postA spawn
    /// (minimap centre = eye (1.484, −10.097), back (−0.4834, ·, −0.8413)),
    /// from the posts' RAM positions, in the 640 × 480 frame.
    #[test]
    fn hot1_posts_land_where_the_ps2_frame_has_them() {
        let map = MapModel {
            eye: [1.484, -10.097],
            heading: heading([0.4834, 0.8413]),
            radius: 50.0,
            ..Default::default()
        };
        // The layout's Position × Scale.
        let base = Mat4::from_translation(Vec3::new(0.856385 * 512.0, 0.186248 * 448.0, 0.0))
            * Mat4::from_scale(Vec3::splat(0.9));
        let group = map.group(&base);
        let o = crate::hud_draw::OVERFILL;
        for (world, want) in [
            ([16.3, -15.8], [534.4, 79.8]),
            ([-213.0, 192.0], [606.7, 70.7]),
            ([-423.5, 204.9], [608.0, 84.9]),
        ] {
            let (uv, _) = map.place(world, true).unwrap();
            let p = group.transform_point3(map.local(uv));
            let got = [
                (o[0].0 + o[0].1 * p.x / 512.0) * 640.0,
                (o[1].0 + o[1].1 * p.y / 448.0) * 480.0,
            ];
            for a in 0..2 {
                assert!(
                    (got[a] - want[a]).abs() <= 3.0,
                    "{world:?}: {got:?}, the PS2 frame has {want:?}"
                );
            }
        }
    }

    #[test]
    fn units_hide_beyond_the_circle_and_posts_clamp() {
        let map = MapModel {
            radius: 50.0,
            ..Default::default()
        };
        assert!(map.place([60.0, 0.0], false).is_none());
        let (uv, d2) = map.place([60.0, 0.0], true).unwrap();
        assert!((uv[0] - 0.5).abs() < 1.0e-6 && d2 > 0.25);
        assert_eq!(map.place([25.0, 0.0], false).unwrap().0, [0.25, 0.0]);
    }

    /// TAN1's negative half side mirrors the image (§2.3).
    #[test]
    fn a_negative_extent_mirrors_the_image() {
        let mut map = MapModel {
            centre: [0.0, 0.0],
            extent: 100.0,
            radius: 50.0,
            ..Default::default()
        };
        assert_eq!(map.image_uv([0.5, 0.0]), [0.75, 0.5]);
        map.extent = -100.0;
        assert_eq!(map.image_uv([0.5, 0.0]), [0.25, 0.5]);
    }

    #[test]
    fn the_radius_follows_the_speed_and_the_blink_the_clock() {
        assert_eq!(view_radius([50.0, 50.0, 0.0, 100.0], 30.0), 50.0);
        assert_eq!(view_radius(VIEW_DEFAULT, 55.0), 187.5);
        assert_eq!(view_radius(VIEW_DEFAULT, 0.0), 125.0);
        assert_eq!(post_color(ENEMY, Some(FRIEND), 0.125), FRIEND);
        assert_eq!(post_color(ENEMY, Some(FRIEND), 0.5), ENEMY);
        assert_eq!(post_color(ENEMY, None, 0.125), ENEMY);
    }
}
