//! The terrain foliage: `tern -> FOLG` (a 4-bit LAYER mask per cell) and the
//! map's `prp_` chunk (the munged `.prp` foliage file that defines the four
//! layers and the path-prop lines). RE: `analysis/re-foliage.md` ([O]).
//!
//! **FOLG** payload = `u32 byteLength` + the nibbles; bit L of a cell's nibble
//! says "layer L grows here" (§0, §2). Byte `(z·D + x) >> 1`, the HIGH nibble
//! for an even x, 0 outside `0..D` — BF2 `FUN_0042b0c0`; BF1 `FUN_00344d98`
//! spells it `z·(D>>1) + (x>>1)`, the same byte for every even D (512, 64 and
//! 128 are the only ones shipped). D is the caller's: BF1 is always 512, BF2
//! half the terrain grid (§2) — see `bake::foliage`.
//!
//! **`prp_`** is a munged config tree ([`crate::config`]). A string argument
//! of every record read here is stored as its FNV-1a hash (§3), so names come
//! back as hashes and the bake resolves them against the LVL's models and
//! classes. What is read (hashes, ELF loaders B1 `FUN_00345980` /
//! `FUN_00346f50`, B2 `FUN_0042bc40` / `FUN_0042d1a0`):
//!
//! ```text
//! Layer(i) {                              0x07a640f6, i in 0..4
//!     SpreadFactor(v)                     0xdbe276e9
//!     Mesh() {                            0xa100bebc, at most 32 per layer
//!         File("model", d)                0xaaea5743  kind Model (unless a patch is given)
//!         GrassPatch("class", d)          0x156b70a1  kind GrassPatch
//!         LeafPatch("class", d)           0x6a6fb399  kind LeafPatch
//!         Frequency(f) Scale(s) Stiffness(k) ColorVariation(c)
//!         Lighting("dynamic"|"static")    0x827eb1c9
//!         Sound("name", min, range)       0x0e0d9594
//!         0xc28f0c96("name")              the walk-into sound
//!         0x08be1de3(n)                   the near-fade distance
//!         0x26edc40c()                    build the model's collision
//!     }
//! }
//! 0x40b06ff7() {                          the path block (§9)
//!     0x84874d36("path") {
//!         0x6850acc6("class")             at most 12
//!         Distance(m)                     0x2eb31462, default 18
//!         0xbad71c66()                    flip the facing
//!     }
//! }
//! ```
//!
//! A `Layer` nested inside a layer's scope is ignored (BF1 GEO1 authors one,
//! which the PS2 never loads).

use crate::config::{parse_tree, ConfigNode};
use crate::{le, Chunk, UcfbError};

const LAYER: u32 = 0x07a6_40f6;
const SPREAD_FACTOR: u32 = 0xdbe2_76e9;
const MESH: u32 = 0xa100_bebc;
const FILE: u32 = 0xaaea_5743;
const GRASS_PATCH: u32 = 0x156b_70a1;
const LEAF_PATCH: u32 = 0x6a6f_b399;
const FREQUENCY: u32 = 0x2fb3_1c01;
const SCALE: u32 = 0x8297_1c71;
const STIFFNESS: u32 = 0x495f_91ae;
const COLOR_VARIATION: u32 = 0xbbce_475d;
const LIGHTING: u32 = 0x827e_b1c9;
const SOUND: u32 = 0x0e0d_9594;
const BRUSH_SOUND: u32 = 0xc28f_0c96;
const NEAR_FADE: u32 = 0x08be_1de3;
const COLLISION: u32 = 0x26ed_c40c;
const PATH_BLOCK: u32 = 0x40b0_6ff7;
const PATH_LINE: u32 = 0x8487_4d36;
const PATH_CLASS: u32 = 0x6850_acc6;
const DISTANCE: u32 = 0x2eb3_1462;
const PATH_FLIP: u32 = 0xbad7_1c66;

/// Layer slots (B1 `DAT_00626de0 + i·0x94`, i < 4).
pub const LAYERS: usize = 4;
/// Mesh entries per layer the loader keeps.
const MAX_MESHES: usize = 32;
/// Classes per path line (B1 334040–334058).
const MAX_PATH_CLASSES: usize = 12;
/// A `File`/`GrassPatch`/`LeafPatch` distance is capped here, and it is the
/// default (B1 `FUN_00346f50` 332836–333034).
pub const MAX_DISTANCE: f32 = 1000.0;
/// `Distance` of a path line when it authors none (B1 334032).
pub const PATH_DEFAULT_DISTANCE: f32 = 18.0;

/// The FOLG layer mask of one terrain.
#[derive(Debug, Clone, PartialEq)]
pub struct FolgMask {
    /// Cells per side.
    pub dim: u32,
    pub bytes: Vec<u8>,
}

impl FolgMask {
    /// `tern -> FOLG` at `dim` cells a side, or `None` when the terrain ships
    /// none or it is too short for `dim` (then the PS2 would read past it; no
    /// shipped terrain is).
    pub fn parse(tern: &Chunk<'_>, dim: u32) -> Option<FolgMask> {
        let p = tern.child(b"FOLG")?.payload;
        if p.len() < 4 || dim == 0 {
            return None;
        }
        let len = (le::u32_at(p, 0) as usize).min(p.len() - 4);
        let need = (dim as usize * dim as usize).div_ceil(2);
        (len >= need).then(|| FolgMask {
            dim,
            bytes: p[4..4 + len].to_vec(),
        })
    }

    /// The cells per side a payload of this terrain's length holds, if it is
    /// square: `sqrt(2 · byteLength)`.
    pub fn payload_dim(tern: &Chunk<'_>) -> Option<u32> {
        let p = tern.child(b"FOLG")?.payload;
        if p.len() < 4 {
            return None;
        }
        let n = 2 * le::u32_at(p, 0) as u64;
        let d = (n as f64).sqrt().round() as u64;
        (d * d == n).then_some(d as u32)
    }

    /// The layer bits of cell (x, z); 0 outside the grid.
    pub fn layers(&self, x: i32, z: i32) -> u8 {
        let d = self.dim as i32;
        if !(0..d).contains(&x) || !(0..d).contains(&z) {
            return 0;
        }
        let at = ((z as usize) * self.dim as usize + x as usize) >> 1;
        let b = self.bytes.get(at).copied().unwrap_or(0);
        if x % 2 == 0 {
            b >> 4
        } else {
            b & 15
        }
    }
}

/// What a mesh entry draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshKind {
    /// `File("model", d)`: a model, by the hash of its `modl` name.
    Model,
    /// `GrassPatch("class", d)`: a `grasspatch` class's sprite cluster.
    GrassPatch,
    /// `LeafPatch("class", d)`: a `leafpatch` class (queue LEAF).
    LeafPatch,
}

/// One `Mesh()` entry of a layer.
#[derive(Debug, Clone, PartialEq)]
pub struct FoliageMesh {
    /// `None` when the entry names neither a model nor a patch: the PS2
    /// still picks it (and spends the random numbers), and draws nothing.
    pub kind: Option<MeshKind>,
    /// FNV-1a of the model name (`Model`) or the class name (a patch).
    pub name_hash: u32,
    /// The `File` a patch entry also names — an editor placeholder the PS2
    /// never draws (§3).
    pub file_hash: Option<u32>,
    /// The fade distance, metres (§6/§7), at most [`MAX_DISTANCE`].
    pub distance: f32,
    /// The pick weight as authored (default 1.0, B1 `FUN_003460d8`).
    pub frequency: f32,
    /// The pick threshold: the running sum of the layer's frequencies over
    /// their total, in f32 as the ELF accumulates it (B1 332060–332082).
    pub cumulative: f32,
    /// Uniform scale, models only (default 1).
    pub scale: f32,
    /// 0 = the terrain normal, 1 = world up (default 0).
    pub stiffness: f32,
    /// Per-channel random darkening of the tint (default 0).
    pub color_variation: f32,
    /// Models closer than this fade toward 0.1² alpha (0 = none).
    pub near_fade: f32,
    /// `Lighting("dynamic")` — every shipped entry is static.
    pub dynamic_lighting: bool,
    /// `0x26edc40c`: the model's `CollisionMesh` is solid (§11).
    pub collision: bool,
    /// `Sound(name, min, range)`: (hash, min s, range s).
    pub sound: Option<(u32, f32, f32)>,
    /// The sound a soldier walking into it plays.
    pub brush_sound: Option<u32>,
}

impl FoliageMesh {
    fn parse(node: &ConfigNode) -> FoliageMesh {
        let mut m = FoliageMesh {
            kind: None,
            name_hash: 0,
            file_hash: None,
            distance: MAX_DISTANCE,
            frequency: 1.0,
            cumulative: 0.0,
            scale: 1.0,
            stiffness: 0.0,
            color_variation: 0.0,
            near_fade: 0.0,
            dynamic_lighting: false,
            collision: false,
            sound: None,
            brush_sound: None,
        };
        let raw = |c: &ConfigNode, i: usize| c.values.get(i).copied();
        let num = |c: &ConfigNode, i: usize| c.values.get(i).map(|v| f32::from_bits(*v));
        let dist = |c: &ConfigNode| num(c, 1).unwrap_or(MAX_DISTANCE).min(MAX_DISTANCE);
        for c in &node.children {
            match c.hash {
                GRASS_PATCH | LEAF_PATCH => {
                    m.kind = Some(if c.hash == GRASS_PATCH {
                        MeshKind::GrassPatch
                    } else {
                        MeshKind::LeafPatch
                    });
                    m.name_hash = raw(c, 0).unwrap_or(0);
                    m.distance = dist(c);
                }
                FILE => {
                    let h = raw(c, 0).unwrap_or(0);
                    m.file_hash = Some(h);
                    // A File counts only while no patch was given (0x347118).
                    if !matches!(m.kind, Some(MeshKind::GrassPatch | MeshKind::LeafPatch)) {
                        m.kind = Some(MeshKind::Model);
                        m.name_hash = h;
                        m.distance = dist(c);
                    }
                }
                FREQUENCY => m.frequency = num(c, 0).unwrap_or(m.frequency),
                SCALE => m.scale = num(c, 0).unwrap_or(m.scale),
                STIFFNESS => m.stiffness = num(c, 0).unwrap_or(m.stiffness),
                COLOR_VARIATION => m.color_variation = num(c, 0).unwrap_or(0.0),
                NEAR_FADE => m.near_fade = num(c, 0).unwrap_or(0.0),
                COLLISION => m.collision = true,
                // A case-insensitive prefix test against "dynamic" (verifier
                // B5); BF2 FEL1's `Lighting(1.0)` is a float, so static.
                LIGHTING => {
                    m.dynamic_lighting = c.str_arg(0).is_some_and(|s| {
                        s.get(..7)
                            .is_some_and(|p| p.eq_ignore_ascii_case("dynamic"))
                    })
                }
                SOUND => {
                    m.sound =
                        raw(c, 0).map(|h| (h, num(c, 1).unwrap_or(0.0), num(c, 2).unwrap_or(0.0)))
                }
                BRUSH_SOUND => m.brush_sound = raw(c, 0),
                _ => {}
            }
        }
        m
    }
}

/// One of the four layers.
#[derive(Debug, Clone, PartialEq)]
pub struct FoliageLayer {
    /// `SpreadFactor` ≥ 1: one instance per `spread × spread` cells.
    pub spread: u32,
    /// `SpreadFactor` < 1: `trunc(1 / v)` instances per cell.
    pub count: u32,
    pub meshes: Vec<FoliageMesh>,
}

impl FoliageLayer {
    pub fn empty() -> FoliageLayer {
        FoliageLayer {
            spread: 1,
            count: 1,
            meshes: Vec::new(),
        }
    }

    fn load(&mut self, node: &ConfigNode) {
        for c in &node.children {
            match c.hash {
                SPREAD_FACTOR => {
                    let v = c.f32(0).unwrap_or(0.0);
                    // EE `cvt.w.s` truncates; `div.s` of 1.0 (B1 332035–332040).
                    let s = v as i32;
                    if s == 0 {
                        self.spread = 1;
                        if v > 0.0 {
                            self.count = ((1.0 / v) as i32).max(0) as u32;
                        }
                    } else {
                        self.spread = s.max(1) as u32;
                    }
                }
                MESH if self.meshes.len() < MAX_MESHES => self.meshes.push(FoliageMesh::parse(c)),
                _ => {} // a nested Layer included
            }
        }
        let total: f32 = self.meshes.iter().map(|m| m.frequency).sum();
        let inv = if total == 0.0 { 1.0 } else { 1.0 / total };
        let mut acc = 0.0f32;
        for m in &mut self.meshes {
            acc += m.frequency * inv;
            m.cumulative = acc;
        }
    }
}

/// One path line of the path block (§9): a prop of `classes` every
/// `distance` metres along the path.
#[derive(Debug, Clone, PartialEq)]
pub struct PathLine {
    /// FNV-1a of the path's name.
    pub path_hash: u32,
    /// FNV-1a of each class name, in authored order.
    pub classes: Vec<u32>,
    pub distance: f32,
    pub flip: bool,
}

/// A map's whole `prp_`.
#[derive(Debug, Clone, PartialEq)]
pub struct FoliageFile {
    pub layers: [FoliageLayer; LAYERS],
    pub paths: Vec<PathLine>,
}

impl FoliageFile {
    pub fn parse(prp: &Chunk<'_>) -> Result<FoliageFile, UcfbError> {
        let mut out = FoliageFile {
            layers: std::array::from_fn(|_| FoliageLayer::empty()),
            paths: Vec::new(),
        };
        for n in parse_tree(prp)? {
            match n.hash {
                LAYER => {
                    let i = n.f32(0).unwrap_or(-1.0) as i32;
                    if let Some(l) = usize::try_from(i).ok().and_then(|i| out.layers.get_mut(i)) {
                        l.load(&n);
                    }
                }
                PATH_BLOCK => {
                    for p in n.find_all(PATH_LINE) {
                        let mut line = PathLine {
                            path_hash: p.values.first().copied().unwrap_or(0),
                            classes: Vec::new(),
                            distance: PATH_DEFAULT_DISTANCE,
                            flip: false,
                        };
                        for c in &p.children {
                            match c.hash {
                                PATH_CLASS if line.classes.len() < MAX_PATH_CLASSES => {
                                    line.classes.extend(c.values.first())
                                }
                                DISTANCE => line.distance = c.f32(0).unwrap_or(line.distance),
                                PATH_FLIP => line.flip = true,
                                _ => {}
                            }
                        }
                        out.paths.push(line);
                    }
                }
                _ => {}
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fnv1a_lowercase as fnv;

    #[test]
    fn the_property_hashes_are_their_names() {
        for (h, n) in [
            (LAYER, "Layer"),
            (SPREAD_FACTOR, "SpreadFactor"),
            (MESH, "Mesh"),
            (FILE, "File"),
            (GRASS_PATCH, "GrassPatch"),
            (LEAF_PATCH, "LeafPatch"),
            (FREQUENCY, "Frequency"),
            (SCALE, "Scale"),
            (STIFFNESS, "Stiffness"),
            (COLOR_VARIATION, "ColorVariation"),
            (LIGHTING, "Lighting"),
            (SOUND, "Sound"),
            (DISTANCE, "Distance"),
        ] {
            assert_eq!(fnv(n), h, "{n}");
        }
    }

    fn disc(rel: &str) -> Option<Vec<u8>> {
        let path = format!("{}/../../../extracted/{rel}", env!("CARGO_MANIFEST_DIR"));
        if !std::path::Path::new(&path).exists() {
            eprintln!("SKIP: {path} not present");
            return None;
        }
        Some(std::fs::read(path).unwrap())
    }

    /// The first `prp_` and `tern` of an LVL, top level or one `lvl_` deep.
    fn chunks_of(inner: &[u8]) -> (Option<Chunk<'_>>, Option<Chunk<'_>>) {
        let mut prp = None;
        let mut tern = None;
        for c in crate::content_chunks(inner).unwrap() {
            if &c.tag == b"prp_" && prp.is_none() {
                prp = Some(c);
            } else if &c.tag == b"tern" && tern.is_none() && c.child(b"FOLG").is_some() {
                tern = Some(c);
            }
        }
        (prp, tern)
    }

    /// BF1 END1 against the RE census (re-foliage §1): layer 0 two
    /// GrassPatches at 60 m (SpreadFactor 0.35 → 2 per cell), layer 1 a fern
    /// and a tree clump (0.8 → 1 per cell), layer 2 three foliage models at
    /// 80/70/50 m, one path line of `end_prop_treebill` every 28 m along
    /// `jungle1`; FOLG 512² with 2951/1956/1293/0 cells per layer.
    #[test]
    fn bf1_end1_matches_the_census() {
        let Some(buf) = disc("bf1/lvl/DATA/_LVL_PS2/END/END1.LVL") else {
            return;
        };
        let inner = crate::open(&buf).unwrap();
        let (prp, tern) = chunks_of(inner);
        let f = FoliageFile::parse(&prp.expect("END1 has a prp_")).unwrap();
        let l = &f.layers;
        assert_eq!((l[0].spread, l[0].count, l[0].meshes.len()), (1, 2, 2));
        assert_eq!((l[1].spread, l[1].count, l[1].meshes.len()), (1, 1, 2));
        assert_eq!(l[2].meshes.len(), 3);
        assert!(l[3].meshes.is_empty());
        let m = &l[0].meshes[0];
        assert_eq!(
            (m.kind, m.name_hash, m.distance),
            (Some(MeshKind::GrassPatch), fnv("end_fern"), 60.0)
        );
        assert_eq!(l[0].meshes[1].name_hash, fnv("end_prop_grass"));
        assert_eq!(l[1].meshes[0].kind, Some(MeshKind::Model));
        assert_eq!(l[1].meshes[0].name_hash, fnv("end_prop_fern5"));
        let d: Vec<f32> = l[2].meshes.iter().map(|m| m.distance).collect();
        assert_eq!(d, [80.0, 70.0, 50.0]);
        assert_eq!(l[2].meshes.last().unwrap().cumulative, 1.0);
        assert_eq!(f.paths.len(), 1);
        let p = &f.paths[0];
        assert_eq!(
            (p.path_hash, p.distance, p.flip),
            (fnv("jungle1"), 28.0, false)
        );
        assert_eq!(p.classes, [fnv("end_prop_treebill")]);

        let tern = tern.expect("END1 has a FOLG");
        assert_eq!(FolgMask::payload_dim(&tern), Some(512));
        let mask = FolgMask::parse(&tern, 512).unwrap();
        let mut n = [0usize; 4];
        for z in 0..512 {
            for x in 0..512 {
                let b = mask.layers(x, z);
                for (k, c) in n.iter_mut().enumerate() {
                    *c += (b >> k & 1) as usize;
                }
            }
        }
        assert_eq!(n, [2951, 1956, 1293, 0]);
        assert_eq!(mask.layers(-1, 0), 0);
        assert_eq!(mask.layers(0, 512), 0);
    }

    /// BF2 FEL1 (re-foliage §1): FOLG 64² (grid 128), four layers — three
    /// models at 100 m (the pitcher bush with collision) and a GrassPatch at
    /// 95 m; the hellbush line every 20 m.
    #[test]
    fn bf2_fel1_matches_the_census() {
        let Some(buf) = disc("bf2/lvl/DATA/_LVL_PS2/FEL/FEL1.LVL") else {
            return;
        };
        let inner = crate::open(&buf).unwrap();
        let (prp, tern) = chunks_of(inner);
        let f = FoliageFile::parse(&prp.expect("FEL1 has a prp_")).unwrap();
        let k: Vec<_> = f
            .layers
            .iter()
            .map(|l| l.meshes.first().and_then(|m| m.kind))
            .collect();
        use MeshKind::*;
        assert_eq!(k, [Some(Model), Some(Model), Some(Model), Some(GrassPatch)]);
        let pitcher = &f.layers[1].meshes[0];
        assert_eq!(pitcher.name_hash, fnv("fel1_bldg_pitcher_bush"));
        assert!(pitcher.collision);
        assert_eq!(pitcher.color_variation, 0.4);
        assert!(!pitcher.dynamic_lighting);
        assert_eq!(f.layers[3].meshes[0].distance, 95.0);
        assert!(f
            .paths
            .iter()
            .any(|p| p.classes.contains(&fnv("fel1_prop_hellbush")) && p.distance == 20.0));
        let tern = tern.expect("FEL1 has a FOLG");
        assert_eq!(FolgMask::payload_dim(&tern), Some(64));
    }
}
