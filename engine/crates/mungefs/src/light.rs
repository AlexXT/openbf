//! `lght` chunk: a map's authored lights (BF2 only — no BF1 map LVL carries
//! one; measured over every map LVL on both discs, 2026-08-31).
//!
//! `lght` is a munged config tree ([`crate::config`]), so the property names
//! are stored only as `fnv1a_lowercase` hashes. Every name below is a
//! **confirmed preimage** — the hash of the spelled name equals the stored
//! word exactly (pinned by `hash_preimages_are_confirmed`). Names that could
//! not be confirmed are kept as raw hashes and reported verbatim; nothing
//! here is guessed.
//!
//! Shape, byte-verified on BF2 TAT2/GEO1/HOT1/END1:
//! ```text
//! Light("sun")            // string arg; the block's name
//!   Rotation(a0, a1, a2, a3) // unit quaternion, see below
//!   Position(x, y, z)
//!   Type(1)               // 1 on every light named "sun"; 3 carries Cone+Range
//!   Color(r, g, b)        // 0..1
//!   CastShadow() CastSpecular(1) Static() PS2BlendMode(0) ...
//! GlobalLights()
//!   Light1("Sun")  Light2("bouncing")   // names of the two Light blocks used
//!   Top(r, g, b)  Bottom(r, g, b)       // 0..255 hemisphere ambient
//! ```
//!
//! **`Rotation` is read as the PS2 reads it** (BF2 `FUN_00596978`,
//! `analysis/re-sun-direction.md` §2 `[O]`): the stored a0..a3 become the
//! quaternion q = (x, y, z, w) = (−a3, a0, −a1, a2) — the editor-to-game
//! handedness change, applied at load, which also stores `Position` as
//! (x, y, −z). A directional light travels along row 2 of q's rotation
//! matrix (`FUN_0053ac20` -> `FUN_00595968`, normalised into the light's
//! +0x50) — [`Light::light_vector`]. The PS2's RAM equals it to ≤ 0.001° for
//! Light1 on all 17 BF2 maps with a PCSX2 state, and for Light2 on the five
//! that author one.
//!
//! Most `lght` chunks in a BF2 map are 12-byte stubs — a bare `NAME` whose u32
//! is `fnv1a_lowercase` of a `.LGT` layer name, with no DATA/SCOP. Fifteen of
//! the eighteen BF2 maps carry exactly ONE non-stub; **DEA1, FEL1 and UTA1
//! carry three**: a world layer with `GlobalLights` plus local point-light
//! sets (`jail_blue1`, `river11`, `macy16`...). Callers wanting the map's sun
//! and ambient must take the one with `GlobalLights`, not the first.
//!
//! All 23 hashes here are confirmed preimages; the full dictionary with
//! per-map values is `analysis/re-sky-lighting.md`.

use crate::config::{parse_tree, ConfigNode};
use crate::{Chunk, UcfbError};

/// Confirmed `fnv1a_lowercase` preimages (see `hash_preimages_are_confirmed`).
mod key {
    pub const LIGHT: u32 = 0xe29d_1e2f; // "Light"
    pub const GLOBAL_LIGHTS: u32 = 0xf754_8f33; // "GlobalLights"
    pub const LIGHT1: u32 = 0xdb56_693a; // "Light1"
    pub const LIGHT2: u32 = 0xda56_67a7; // "Light2"
    pub const TOP: u32 = 0xa710_dc3c; // "Top"
    pub const BOTTOM: u32 = 0x4ea7_6b2a; // "Bottom"
    pub const ROTATION: u32 = 0x21ac_415f; // "Rotation"
    pub const POSITION: u32 = 0x934f_4e0a; // "Position"
    pub const TYPE: u32 = 0x5127_f14d; // "Type"
    pub const COLOR: u32 = 0x3d7e_6258; // "Color"
    pub const RANGE: u32 = 0xfadc_0cd2; // "Range"
    pub const CONE: u32 = 0xdd40_4b6a; // "Cone"
    pub const CAST_SHADOW: u32 = 0xe7e9_0aee; // "CastShadow"
    pub const CAST_SPECULAR: u32 = 0x4891_62bb; // "CastSpecular"
    pub const STATIC: u32 = 0xd290_c23b; // "Static"
    pub const REGION: u32 = 0x031a_fe7d; // "Region"
    pub const TEXTURE: u32 = 0x3c64_68f4; // "Texture"
    pub const PS2_BLEND_MODE: u32 = 0x84e2_6526; // "PS2BlendMode"
    pub const BIDIRECTIONAL: u32 = 0x044c_2100; // "Bidirectional"
                                                // Directional-light projection controls.
    pub const TILE_UV: u32 = 0xbfe6_0da6; // "TileUV"
    pub const OFFSET_UV: u32 = 0x6ac6_27c1; // "OffsetUV"
    pub const PS2_OVERBRIGHT: u32 = 0x55b8_6d2c; // "PS2Overbright"
                                                 // Editor-only.
    pub const EDITOR_ICON_SIZE: u32 = 0x7755_626a; // "EditorGlobalDirIconSize"
}

/// One authored light. Fields absent from the chunk stay `None`; nothing is
/// defaulted here, so a consumer can tell "not authored" from "authored zero".
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Light {
    /// Block name as stored ("sun", "bouncing", "arden11", "Light 1"...).
    pub name: String,
    /// `Type`: 1 directional, 2 omni, 3 spot (BF2 `FUN_00596978`,
    /// `analysis/re-point-lights.md` §1 `[O]`).
    pub kind: Option<f32>,
    /// The quaternion as stored, a0..a3 — read it through
    /// [`Light::light_vector`] (the module doc).
    pub rotation: Option<[f32; 4]>,
    pub position: Option<[f32; 3]>,
    /// 0..1 (measured range over every `lght` on the BF2 disc).
    pub color: Option<[f32; 3]>,
    pub range: Option<f32>,
    /// `Cone(inner, outer)` in radians as stored (0.698 = 40°, 1.047 = 60°).
    pub cone: Option<[f32; 2]>,
    pub cast_shadow: bool,
    pub cast_specular: Option<f32>,
    /// `Static` present: the light's word-0 bit 0x100. A prelit model and
    /// the terrain skip every Static light (`re-point-lights.md` §8 `[O]`).
    pub is_static: bool,
    /// `PS2Overbright` present (bit 0x1000): the colour is packed ×255
    /// instead of ×128 for the VU, i.e. twice as bright on models
    /// (`re-point-lights.md` §1 `[O]`).
    pub overbright: bool,
    pub region: Option<String>,
    pub texture: Option<String>,
    pub ps2_blend_mode: Option<f32>,
    pub bidirectional: Option<f32>,
    /// Projection controls that appear only on `Type(1)` lights:
    /// `TileUV` (always u == v on the discs) and `OffsetUV` (always 0, 0).
    pub tile_uv: Option<[f32; 2]>,
    pub offset_uv: Option<[f32; 2]>,
}

impl Light {
    /// The unit direction a directional light TRAVELS, as BF2 builds it
    /// (the module doc): row 2 of the matrix of q = (−a3, a0, −a1, a2).
    /// `None` without a `Rotation` or for a degenerate one.
    pub fn light_vector(&self) -> Option<[f32; 3]> {
        let [a0, a1, a2, a3] = self.rotation?;
        let (x, y, z, w) = (-a3, a0, -a1, a2);
        let r = [
            2.0 * (x * z + w * y),
            2.0 * (y * z - w * x),
            1.0 - 2.0 * (x * x + y * y),
        ];
        let n = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        (n > 1e-6).then(|| r.map(|c| c / n))
    }
}

/// `GlobalLights`: which two authored lights light the world, plus the
/// hemisphere ambient (`Top` = from above, `Bottom` = from below), 0..255.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GlobalLights {
    pub light1: Option<String>,
    pub light2: Option<String>,
    pub top: Option<[f32; 3]>,
    pub bottom: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Lights {
    pub lights: Vec<Light>,
    pub global: Option<GlobalLights>,
    /// Hashes seen at the top level or inside a `Light`/`GlobalLights` block
    /// whose name we could not confirm, with the values as stored. Reported,
    /// never guessed at.
    pub unknown: Vec<(u32, Vec<f32>)>,
}

impl Lights {
    /// Parse a `lght` chunk. A stub `lght` (BF2 ships one per mode sublevel)
    /// yields an empty `Lights`.
    pub fn parse(lght: &Chunk<'_>) -> Result<Lights, UcfbError> {
        let mut out = Lights::default();
        for node in parse_tree(lght)? {
            match node.hash {
                key::LIGHT => out.lights.push(parse_light(&node, &mut out.unknown)),
                key::GLOBAL_LIGHTS => {
                    out.global = Some(parse_global(&node, &mut out.unknown));
                }
                h => out.unknown.push((h, node.floats())),
            }
        }
        Ok(out)
    }

    /// The light `GlobalLights.Light1` names, matched case-insensitively
    /// (the disc stores `Light1("Sun")` against a block named `"sun"`).
    pub fn key_light(&self) -> Option<&Light> {
        let want = self.global.as_ref()?.light1.as_deref()?;
        self.lights
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case(want))
    }

    /// The light `GlobalLights.Light2` names, if any (often empty).
    pub fn fill_light(&self) -> Option<&Light> {
        let want = self.global.as_ref()?.light2.as_deref()?;
        if want.is_empty() {
            return None;
        }
        self.lights
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case(want))
    }
}

fn parse_light(node: &ConfigNode, unknown: &mut Vec<(u32, Vec<f32>)>) -> Light {
    let mut l = Light {
        name: node.string.clone().unwrap_or_default(),
        ..Light::default()
    };
    for c in &node.children {
        match c.hash {
            key::ROTATION => l.rotation = c.f4(),
            key::POSITION => l.position = c.f3(),
            key::TYPE => l.kind = c.f32(0),
            key::COLOR => l.color = c.f3(),
            key::RANGE => l.range = c.f32(0),
            key::CONE => l.cone = Some(c.floats_or_zero()),
            key::CAST_SHADOW => l.cast_shadow = true,
            key::CAST_SPECULAR => l.cast_specular = c.f32(0),
            key::STATIC => l.is_static = true,
            key::REGION => l.region = c.string.clone(),
            key::TEXTURE => l.texture = c.string.clone(),
            key::PS2_BLEND_MODE => l.ps2_blend_mode = c.f32(0),
            key::BIDIRECTIONAL => l.bidirectional = c.f32(0),
            key::TILE_UV => l.tile_uv = Some(c.floats_or_zero()),
            key::OFFSET_UV => l.offset_uv = Some(c.floats_or_zero()),
            key::PS2_OVERBRIGHT => l.overbright = true,
            key::EDITOR_ICON_SIZE => {}
            h => unknown.push((h, c.floats())),
        }
    }
    l
}

fn parse_global(node: &ConfigNode, unknown: &mut Vec<(u32, Vec<f32>)>) -> GlobalLights {
    let mut g = GlobalLights::default();
    for c in &node.children {
        match c.hash {
            key::LIGHT1 => g.light1 = c.string.clone(),
            key::LIGHT2 => g.light2 = c.string.clone(),
            key::TOP => g.top = c.f3(),
            key::BOTTOM => g.bottom = c.f3(),
            key::EDITOR_ICON_SIZE => {}
            h => unknown.push((h, c.floats())),
        }
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fnv1a_lowercase;

    /// Every name this module claims is a CONFIRMED preimage: the spelled
    /// name hashes to the constant. A wrong spelling fails here, so the
    /// module doc's names cannot drift from the data.
    #[test]
    fn hash_preimages_are_confirmed() {
        for (name, h) in [
            ("Light", key::LIGHT),
            ("GlobalLights", key::GLOBAL_LIGHTS),
            ("Light1", key::LIGHT1),
            ("Light2", key::LIGHT2),
            ("Top", key::TOP),
            ("Bottom", key::BOTTOM),
            ("Rotation", key::ROTATION),
            ("Position", key::POSITION),
            ("Type", key::TYPE),
            ("Color", key::COLOR),
            ("Range", key::RANGE),
            ("Cone", key::CONE),
            ("CastShadow", key::CAST_SHADOW),
            ("CastSpecular", key::CAST_SPECULAR),
            ("Static", key::STATIC),
            ("Region", key::REGION),
            ("Texture", key::TEXTURE),
            ("PS2BlendMode", key::PS2_BLEND_MODE),
            ("Bidirectional", key::BIDIRECTIONAL),
            ("TileUV", key::TILE_UV),
            ("OffsetUV", key::OFFSET_UV),
            ("PS2Overbright", key::PS2_OVERBRIGHT),
            ("EditorGlobalDirIconSize", key::EDITOR_ICON_SIZE),
        ] {
            assert_eq!(fnv1a_lowercase(name), h, "{name}");
        }
    }

    /// Row 2 of (−a3, a0, −a1, a2) against the PS2's RAM (+0x50 of the
    /// light DAT_0060c6e0 / DAT_0060c6e4 point at;
    /// `analysis/re-sun-direction.md` §6.2).
    #[test]
    fn light_vector_matches_the_ps2_ram() {
        for (rot, ram) in [
            // GEO1 Light1.
            (
                [-0.409_077, -0.872_693, 0.262_547, -0.046_18],
                [-0.1342, -0.7382, 0.6610],
            ),
            // COR1 Light2.
            (
                [0.803_575, 0.239_57, -0.480_029, 0.257_893],
                [-0.6479, -0.6326, -0.4244],
            ),
        ] {
            let l = Light {
                rotation: Some(rot),
                ..Light::default()
            }
            .light_vector()
            .unwrap();
            for i in 0..3 {
                assert!((l[i] - ram[i]).abs() < 2e-4, "{rot:?}: {l:?} vs {ram:?}");
            }
        }
    }

    #[test]
    fn stub_lght_parses_empty() {
        let buf = crate::build_chunk(b"lght", &crate::build_chunk(b"NAME", b"x\0\0\0"));
        let c = crate::chunks(&buf).next().unwrap().unwrap();
        let l = Lights::parse(&c).unwrap();
        assert!(l.lights.is_empty() && l.global.is_none());
    }
}
