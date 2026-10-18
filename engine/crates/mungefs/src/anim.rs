//! Animation banks (`zaa_` = NAME + BIN_): the clip directory.
//!
//! Verified layout (analysis/re-anim-banks.md §2.2, BF1 `FUN_004cdf28` /
//! `FUN_0043d590` / `FUN_00446c20`): `BIN_` = `SMNA` header (0x18 B: tag,
//! u32 0, u32 version = 8, u32 totalBoneTracks, u32 tadaBytes, u16 animCount)
//! followed by the sibling chunks `MINA` (animCount × 16 B: u64 boneFlagMask,
//! u32 nameCrc, u16 frames, u16 bones), `TNJA` (totalBoneTracks × 0x30 B
//! per-track records) and `TADA` (tadaBytes of packed keys). Clip and bank
//! names hash with the engine's CRC-32 (`crc32_lower`), not FNV.
//! Keyframes (analysis/re-anim-keys.md, lead-verified): TNJA record 0x30 =
//! `u32 boneCrc, u32 rotOff[4], u32 transOff[3], f32 base[3], f32 scale`;
//! MINA mask bit i ⇔ track i has translation streams (else p = base). TADA
//! stream = `i16 first` then per frame `0x80 nn` hold / `0x81 i16` absolute
//! / `i8` delta. Rotation = raw/2047 (xyzw, ABSOLUTE local-to-parent),
//! translation = base + scale·raw (metres, local). `zaf_` = the rig: BIN_ is
//! a reversed-tag "ZAFF" tree whose SKEL holds 0x2C-byte JONT records
//! (crc, rest quat xyzw, rest pos, parent/child/sibling crcs).

use crate::{le, Chunk, UcfbError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipInfo {
    /// `crc32_lower(full clip name)`, e.g. "human_rifle_diveforward".
    pub name_crc: u32,
    /// Frame count; BF1 duration = frames/30 s, BF2 = (frames−1)/30 s.
    pub frames: u16,
    /// Bone tracks in this clip.
    pub bones: u16,
    /// Per-bone flag bits (tested per track when the runtime record is built).
    pub bone_mask: u64,
}

#[derive(Debug, Clone)]
pub struct AnimBank<'a> {
    pub name: String,
    pub version: u32,
    pub total_tracks: u32,
    pub clips: Vec<ClipInfo>,
    /// Raw `TNJA` payload (total_tracks × 0x30 B).
    pub tnja: &'a [u8],
    /// Raw `TADA` payload.
    pub tada: &'a [u8],
}

/// One TNJA record (a bone track of one clip).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Track {
    pub bone_crc: u32,
    pub rot_off: [u32; 4],
    pub trans_off: [u32; 3],
    pub base: [f32; 3],
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedTrack {
    pub bone_crc: u32,
    /// Unit quaternion xyzw per frame, local-to-parent (absolute).
    pub rotations: Vec<[f32; 4]>,
    /// Local translation per frame (metres); constant `base` when the clip's
    /// mask bit is clear.
    pub translations: Vec<[f32; 3]>,
    pub translated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedClip {
    pub name_crc: u32,
    pub frames: u16,
    pub tracks: Vec<DecodedTrack>,
}

/// Engine rotation quantum (`0.0004885198` in both ELFs = 1/2047).
pub const ROT_QUANTUM: f32 = 0.000_488_519_8;

/// Decode one TADA stream of `frames` values starting at `off`.
pub fn decode_stream(tada: &[u8], off: usize, frames: usize) -> Result<Vec<i16>, UcfbError> {
    let bad = || UcfbError::Truncated(off);
    let mut out = Vec::with_capacity(frames);
    if frames == 0 {
        return Ok(out);
    }
    let mut at = off;
    let mut v = i16::from_le_bytes([
        *tada.get(at).ok_or_else(bad)?,
        *tada.get(at + 1).ok_or_else(bad)?,
    ]);
    at += 2;
    out.push(v);
    while out.len() < frames {
        let op = *tada.get(at).ok_or_else(bad)?;
        at += 1;
        match op {
            0x80 => {
                let n = usize::from(*tada.get(at).ok_or_else(bad)?);
                at += 1;
                for _ in 0..n {
                    if out.len() < frames {
                        out.push(v);
                    }
                }
            }
            0x81 => {
                v = i16::from_le_bytes([
                    *tada.get(at).ok_or_else(bad)?,
                    *tada.get(at + 1).ok_or_else(bad)?,
                ]);
                at += 2;
                out.push(v);
            }
            b => {
                v = v.wrapping_add(i16::from(b as i8));
                out.push(v);
            }
        }
    }
    Ok(out)
}

impl AnimBank<'_> {
    pub fn clip(&self, name: &str) -> Option<&ClipInfo> {
        let crc = crc32_lower(name);
        self.clips.iter().find(|c| c.name_crc == crc)
    }

    pub fn clip_index(&self, name: &str) -> Option<usize> {
        let crc = crc32_lower(name);
        self.clips.iter().position(|c| c.name_crc == crc)
    }

    /// All TNJA records in bank order (clip k's tracks start at Σ bones of
    /// clips 0..k).
    pub fn tracks(&self) -> Vec<Track> {
        let u32_at = |at: usize| le::u32_at(self.tnja, at);
        let f32_at = |at: usize| le::f32_at(self.tnja, at);
        (0..self.total_tracks as usize)
            .map(|i| {
                let b = i * 0x30;
                Track {
                    bone_crc: u32_at(b),
                    rot_off: [u32_at(b + 4), u32_at(b + 8), u32_at(b + 12), u32_at(b + 16)],
                    trans_off: [u32_at(b + 20), u32_at(b + 24), u32_at(b + 28)],
                    base: [f32_at(b + 32), f32_at(b + 36), f32_at(b + 40)],
                    scale: f32_at(b + 44),
                }
            })
            .collect()
    }

    /// Index of clip `k`'s first track.
    pub fn first_track(&self, k: usize) -> usize {
        self.clips[..k].iter().map(|c| usize::from(c.bones)).sum()
    }

    /// Decode every frame of clip `k`. Zero-norm rotation keys (4 exist in
    /// the shipped BF1 data: a lerp through a sign flip) take the previous
    /// frame's key, the first frame the next non-degenerate one.
    pub fn decode_clip(&self, k: usize) -> Result<DecodedClip, UcfbError> {
        let info = *self.clips.get(k).ok_or(UcfbError::Truncated(k))?;
        let frames = usize::from(info.frames);
        let tracks = self.tracks();
        let first = self.first_track(k);
        let mut out = Vec::with_capacity(usize::from(info.bones));
        for i in 0..usize::from(info.bones) {
            let t = *tracks
                .get(first + i)
                .ok_or(UcfbError::Truncated(first + i))?;
            let mut chans = Vec::with_capacity(4);
            for off in t.rot_off {
                chans.push(decode_stream(self.tada, off as usize, frames)?);
            }
            let mut rotations: Vec<[f32; 4]> = (0..frames)
                .map(|f| {
                    let q = [
                        f32::from(chans[0][f]) * ROT_QUANTUM,
                        f32::from(chans[1][f]) * ROT_QUANTUM,
                        f32::from(chans[2][f]) * ROT_QUANTUM,
                        f32::from(chans[3][f]) * ROT_QUANTUM,
                    ];
                    let n2: f32 = q.iter().map(|x| x * x).sum();
                    if n2 > 0.25 {
                        let n = n2.sqrt();
                        [q[0] / n, q[1] / n, q[2] / n, q[3] / n]
                    } else {
                        [0.0; 4] // degenerate: patched below
                    }
                })
                .collect();
            for f in 0..frames {
                if rotations[f] == [0.0; 4] {
                    let fix = (0..f)
                        .rev()
                        .chain(f + 1..frames)
                        .find(|&g| rotations[g] != [0.0; 4]);
                    rotations[f] = fix.map_or([0.0, 0.0, 0.0, 1.0], |g| rotations[g]);
                }
            }
            let translated = i < 64 && info.bone_mask & (1u64 << i) != 0;
            let translations = if translated {
                let mut tc = Vec::with_capacity(3);
                for off in t.trans_off {
                    tc.push(decode_stream(self.tada, off as usize, frames)?);
                }
                (0..frames)
                    .map(|f| {
                        [
                            t.base[0] + t.scale * f32::from(tc[0][f]),
                            t.base[1] + t.scale * f32::from(tc[1][f]),
                            t.base[2] + t.scale * f32::from(tc[2][f]),
                        ]
                    })
                    .collect()
            } else {
                vec![t.base; frames]
            };
            out.push(DecodedTrack {
                bone_crc: t.bone_crc,
                rotations,
                translations,
                translated,
            });
        }
        Ok(DecodedClip {
            name_crc: info.name_crc,
            frames: info.frames,
            tracks: out,
        })
    }
}

/// One joint of a `zaf_` rig (JONT record).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigJoint {
    pub crc: u32,
    /// Rest rotation xyzw, local-to-parent.
    pub rest_rot: [f32; 4],
    pub rest_pos: [f32; 3],
    pub parent_crc: u32,
    pub first_child_crc: u32,
    pub next_sibling_crc: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rig {
    pub name: String,
    pub joints: Vec<RigJoint>,
}

impl Rig {
    pub fn joint(&self, crc: u32) -> Option<&RigJoint> {
        self.joints.iter().find(|j| j.crc == crc)
    }
    pub fn parent_index(&self, i: usize) -> Option<usize> {
        let p = self.joints[i].parent_crc;
        if p == 0 {
            None
        } else {
            self.joints.iter().position(|j| j.crc == p)
        }
    }
}

/// Walk a reversed-tag (`FFAZ` = "ZAFF") chunk tree, calling `f(tag, payload)`.
fn walk_reversed(buf: &[u8], f: &mut dyn FnMut(&[u8; 4], &[u8])) {
    let mut at = 0;
    while at + 8 <= buf.len() {
        let mut tag = [buf[at + 3], buf[at + 2], buf[at + 1], buf[at]];
        if !tag
            .iter()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || *b == b'_')
        {
            tag = [0; 4];
        }
        let len = u32::from_le_bytes(buf[at + 4..at + 8].try_into().unwrap()) as usize;
        let end = (at + 8 + len).min(buf.len());
        if tag == [0; 4] {
            break;
        }
        f(&tag, &buf[at + 8..end]);
        at = (end + 3) & !3;
    }
}

/// Parse a `zaf_` chunk into its rig.
pub fn parse_zaf<'a>(zaf: &Chunk<'a>) -> Result<Rig, UcfbError> {
    let bad = || UcfbError::Truncated(zaf.offset);
    let name = zaf.child_string(b"NAME").ok_or_else(bad)?;
    let bin = zaf.child(b"BIN_").ok_or_else(bad)?.payload;
    let mut joints = Vec::new();
    walk_reversed(bin, &mut |tag, zaff| {
        if tag != b"ZAFF" {
            return;
        }
        walk_reversed(zaff, &mut |tag, skel| {
            if tag != b"SKEL" {
                return;
            }
            walk_reversed(skel, &mut |tag, jont| {
                if tag != b"JONT" || jont.len() < 0x2C {
                    return;
                }
                let u = |at: usize| le::u32_at(jont, at);
                let f = |at: usize| le::f32_at(jont, at);
                joints.push(RigJoint {
                    crc: u(0),
                    rest_rot: [f(4), f(8), f(12), f(16)],
                    rest_pos: [f(20), f(24), f(28)],
                    parent_crc: u(32),
                    first_child_crc: u(36),
                    next_sibling_crc: u(40),
                });
            });
        });
    });
    if joints.is_empty() {
        return Err(bad());
    }
    Ok(Rig { name, joints })
}

/// Every `zaf_` rig in an LVL.
pub fn find_rigs(inner: &[u8]) -> Result<Vec<Rig>, UcfbError> {
    crate::content_chunks(inner)?
        .iter()
        .filter(|c| &c.tag == b"zaf_")
        .map(parse_zaf)
        .collect()
}

/// Engine CRC-32 for animation/bank names (BF1 `FUN_00444bb8`, table
/// `DAT_00521ca8`): MSB-first, polynomial 0x04C11DB7, init 0xFFFFFFFF, final
/// complement, ASCII `A-Z` lowercased first.
pub fn crc32_lower(name: &str) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for b in name.bytes() {
        let b = b.to_ascii_lowercase();
        crc ^= u32::from(b) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ 0x04C1_1DB7
            } else {
                crc << 1
            };
        }
    }
    !crc
}

/// Parse one `zaa_` chunk.
pub fn parse_zaa<'a>(zaa: &Chunk<'a>) -> Result<AnimBank<'a>, UcfbError> {
    let bad = || UcfbError::Truncated(zaa.offset);
    let name = zaa.child_string(b"NAME").ok_or_else(bad)?;
    let bin = zaa.child(b"BIN_").ok_or_else(bad)?.payload;
    if bin.len() < 0x18 || &bin[0..4] != b"SMNA" {
        return Err(bad());
    }
    let version = le::u32_at(bin, 8);
    let total_tracks = le::u32_at(bin, 0xc);
    let tada_bytes = le::u32_at(bin, 0x10) as usize;
    let anim_count = usize::from(le::u16_at(bin, 0x14));

    // Sibling chunks after the header: [tag][u32 len][payload], 4-aligned.
    let mut at = 0x18;
    let (mut mina, mut tnja, mut tada) = (None, None, None);
    while at + 8 <= bin.len() {
        let tag = &bin[at..at + 4];
        if !matches!(tag, b"MINA" | b"TNJA" | b"TADA") {
            break; // trailing bytes after the last section
        }
        let len = le::u32_at(bin, at + 4) as usize;
        // Section lengths are rounded up to 4 but the enclosing BIN_ length is
        // not (BF2 INGAME.LVL human_1: TADA 28844 vs 28841 real bytes, BIN_
        // ends 1 byte short of the padding) — clamp to the parent.
        let end = (at + 8 + len).min(bin.len());
        let payload = bin.get(at + 8..end).ok_or_else(bad)?;
        match tag {
            b"MINA" => mina = Some(payload),
            b"TNJA" => tnja = Some(payload),
            _ => tada = Some(payload),
        }
        at = (at + 8 + len + 3) & !3;
    }
    let (mina, tnja, tada) = (
        mina.ok_or_else(bad)?,
        tnja.ok_or_else(bad)?,
        tada.ok_or_else(bad)?,
    );
    if mina.len() < anim_count * 16
        || tnja.len() < total_tracks as usize * 0x30
        || tada.len() < tada_bytes
    {
        return Err(bad());
    }
    let clips = (0..anim_count)
        .map(|i| {
            let e = i * 16;
            ClipInfo {
                bone_mask: le::u64_at(mina, e),
                name_crc: le::u32_at(mina, e + 8),
                frames: le::u16_at(mina, e + 12),
                bones: le::u16_at(mina, e + 14),
            }
        })
        .collect();
    Ok(AnimBank {
        name,
        version,
        total_tracks,
        clips,
        tnja: &tnja[..total_tracks as usize * 0x30],
        tada: &tada[..tada_bytes], // the chunk payload is padded to 4
    })
}

/// Every `zaa_` bank in an LVL (via `content_chunks`).
pub fn find_banks<'a>(inner: &'a [u8]) -> Result<Vec<AnimBank<'a>>, UcfbError> {
    crate::content_chunks(inner)?
        .iter()
        .filter(|c| &c.tag == b"zaa_")
        .map(parse_zaa)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_lower_matches_the_engine_table() {
        // re-anim-banks.md §2.5: crc32("human_rifle_diveforward") = 0xDC2308EB.
        assert_eq!(crc32_lower("human_rifle_diveforward"), 0xDC23_08EB);
        assert_eq!(crc32_lower("HUMAN_RIFLE_DIVEFORWARD"), 0xDC23_08EB);
        assert_ne!(crc32_lower("human_rifle_diveforwarx"), 0xDC23_08EB);
    }

    fn banks_of(path: &str) -> Option<Vec<u8>> {
        std::fs::read(path).ok()
    }

    #[test]
    fn bf1_common_lvl_human_banks() {
        let p = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../extracted/bf1/lvl/DATA/_LVL_PS2/COMMON/ENG/COMMON.LVL"
        );
        let Some(buf) = banks_of(p) else {
            eprintln!("SKIP: COMMON.LVL not present");
            return;
        };
        let inner = crate::open(&buf).unwrap();
        let banks = find_banks(inner).unwrap();
        let names: Vec<&str> = banks.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"human_animation_0"), "{names:?}");
        let b2 = banks
            .iter()
            .find(|b| b.name == "human_animation_2")
            .unwrap();
        // §2.5 worked example: 55 anims, 1430 tracks, 61151 TADA bytes.
        assert_eq!(b2.version, 8);
        assert_eq!(b2.clips.len(), 55);
        assert_eq!(b2.total_tracks, 1430);
        assert_eq!(b2.tada.len(), 61151);
        assert_eq!(b2.tnja.len(), 1430 * 0x30);
        let dive = b2.clip("human_rifle_diveforward").unwrap();
        assert_eq!((dive.frames, dive.bones, dive.bone_mask), (35, 26, 0x5));
        // §3.1 timings table.
        let b4 = banks
            .iter()
            .find(|b| b.name == "human_animation_4")
            .unwrap();
        assert_eq!(b4.clip("human_rifle_stand_getupfront").unwrap().frames, 58);
        assert_eq!(b4.clip("human_rifle_prone_getup_stand").unwrap().frames, 45);
    }

    #[test]
    fn bf2_ingame_lvl_human_banks() {
        let p = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../extracted/bf2/lvl/DATA/_LVL_PS2/INGAME.LVL"
        );
        let Some(buf) = banks_of(p) else {
            eprintln!("SKIP: INGAME.LVL not present");
            return;
        };
        let inner = crate::open(&buf).unwrap();
        let banks = find_banks(inner).unwrap();
        let b1 = banks.iter().find(|b| b.name == "human_1").unwrap();
        let dive = b1.clip("human_rifle_diveforward").unwrap();
        assert_eq!((dive.frames, dive.bones), (35, 26));
        assert_eq!(b1.clip("human_rifle_jump").unwrap().frames, 31);
        let b0 = banks.iter().find(|b| b.name == "human_0").unwrap();
        assert_eq!(b0.clip("human_rifle_stand_getupfront").unwrap().frames, 58);
    }

    fn close4(a: [f32; 4], b: [f32; 4], tol: f32) -> bool {
        (0..4).all(|i| (a[i] - b[i]).abs() < tol)
    }
    fn close3(a: [f32; 3], b: [f32; 3], tol: f32) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < tol)
    }

    #[test]
    fn stream_decoder_ops() {
        // i16 first, hold 3, absolute, deltas.
        let tada = [0x10u8, 0x00, 0x80, 0x03, 0x81, 0xff, 0x7f, 0xfe, 0x02];
        assert_eq!(
            decode_stream(&tada, 0, 7).unwrap(),
            [16, 16, 16, 16, 32767, 32765, 32767]
        );
        assert!(decode_stream(&tada, 0, 20).is_err());
    }

    /// re-anim-keys.md §7: human_animation_2 diveforward tracks + keys.
    #[test]
    fn bf1_diveforward_keys() {
        let p = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../extracted/bf1/lvl/DATA/_LVL_PS2/COMMON/ENG/COMMON.LVL"
        );
        let Some(buf) = banks_of(p) else {
            eprintln!("SKIP: COMMON.LVL not present");
            return;
        };
        let inner = crate::open(&buf).unwrap();
        let banks = find_banks(inner).unwrap();
        let b2 = banks
            .iter()
            .find(|b| b.name == "human_animation_2")
            .unwrap();
        let k = b2.clip_index("human_rifle_diveforward").unwrap();
        assert_eq!(k, 9);
        assert_eq!(b2.first_track(k), 234);
        let tracks = b2.tracks();
        let t0 = tracks[234];
        assert_eq!(t0.bone_crc, 0x4446_E9B8);
        assert_eq!(t0.rot_off, [11451, 11455, 11459, 11463]);
        assert_eq!(t0.trans_off, [11467, 11471, 11475]);
        assert!(close3(t0.base, [0.0, 0.0, 2.70217], 1e-4));
        assert!((t0.scale - 1.32006e-3).abs() < 1e-7);
        let t2 = tracks[236];
        assert_eq!(t2.bone_crc, 0x16E2_7226);
        assert!(close3(t2.base, [-0.067772, 0.499355, -0.041133], 1e-5));
        assert!((t2.scale - 2.20555e-4).abs() < 1e-8);
        assert_eq!(tracks[242].bone_crc, 0x8E51_239C);
        assert_eq!(tracks[242].rot_off, [12675, 12679, 12683, 12737]);

        let clip = b2.decode_clip(k).unwrap();
        assert_eq!((clip.frames, clip.tracks.len()), (35, 26));
        let dr = &clip.tracks[0];
        assert!(dr.translated);
        assert!(close4(dr.rotations[0], [0.0, 0.0, 0.0, 1.0], 1e-4));
        assert!(
            close3(dr.translations[0], [0.0, 0.0, 0.0], 1e-3),
            "{:?}",
            dr.translations[0]
        );
        assert!(close3(dr.translations[10], [0.0, 0.0, 2.01441], 1e-4));
        assert!(close3(dr.translations[34], [0.0, 0.0, 5.40433], 1e-4));
        let root = &clip.tracks[2];
        assert!(close3(
            root.translations[0],
            [0.02243, 0.70028, -0.07863],
            1e-4
        ));
        assert!(close3(
            root.translations[10],
            [-0.03138, 0.89746, -0.10157],
            1e-4
        ));
        let fa = &clip.tracks[8];
        assert!(!fa.translated);
        assert!(close3(fa.translations[0], [0.24016, 0.0, 0.0], 1e-4));
        assert!(
            close4(fa.rotations[0], [0.0, 0.0, -0.82609, 0.56375], 1e-3),
            "{:?}",
            fa.rotations[0]
        );
        assert!(close4(
            fa.rotations[10],
            [0.0, 0.0, -0.51685, 0.85589],
            1e-3
        ));
        assert!(close4(
            fa.rotations[34],
            [0.0, 0.0, -0.59111, 0.80655],
            1e-3
        ));
        assert!(close4(
            clip.tracks[16].rotations[0],
            [0.43478, -0.18319, -0.41524, 0.77821],
            1e-3
        ));
        assert!(close4(
            clip.tracks[23].rotations[0],
            [0.0, 0.0, 0.81094, 0.58476],
            1e-3
        ));

        // The rig: 31 joints, bone_root under the identity helper, bone_l_calf.
        let rigs = find_rigs(inner).unwrap();
        let rig = rigs.iter().find(|r| r.name == "human_animation_0").unwrap();
        assert_eq!(rig.joints.len(), 31);
        let root = rig.joint(0x16E2_7226).unwrap();
        assert!(close3(root.rest_pos, [0.0, 1.1594, 0.0], 1e-4));
        assert_eq!(root.parent_crc, 0xB420_9B59);
        assert_eq!(root.first_child_crc, 0x5DE5_2389);
        let calf = rig.joint(0x8701_9234).unwrap();
        assert!(close4(
            calf.rest_rot,
            [-3.0e-4, 1.1e-4, 0.13078, 0.99141],
            1e-4
        ));
        assert!(close3(calf.rest_pos, [0.48, 0.0, 0.0], 1e-4));
        assert_eq!(calf.parent_crc, 0xC496_176D);
        assert_eq!(rig.joints[0].crc, 0x4446_E9B8);
        assert_eq!(rig.parent_index(0), None);

        // Every clip in every human bank decodes (stream ends line up).
        for b in &banks {
            for k in 0..b.clips.len() {
                let c = b.decode_clip(k).unwrap();
                assert_eq!(c.tracks.len(), usize::from(b.clips[k].bones));
            }
        }
    }

    /// The BF2 copy of diveforward decodes bit-identically to BF1's.
    #[test]
    fn bf1_bf2_diveforward_identical() {
        let p1 = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../extracted/bf1/lvl/DATA/_LVL_PS2/COMMON/ENG/COMMON.LVL"
        );
        let p2 = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../extracted/bf2/lvl/DATA/_LVL_PS2/INGAME.LVL"
        );
        let (Some(b1), Some(b2)) = (banks_of(p1), banks_of(p2)) else {
            eprintln!("SKIP: COMMON.LVL / INGAME.LVL not present");
            return;
        };
        let (i1, i2) = (crate::open(&b1).unwrap(), crate::open(&b2).unwrap());
        let (banks1, banks2) = (find_banks(i1).unwrap(), find_banks(i2).unwrap());
        let a = banks1
            .iter()
            .find(|b| b.name == "human_animation_2")
            .unwrap();
        let b = banks2.iter().find(|b| b.name == "human_1").unwrap();
        let ca = a
            .decode_clip(a.clip_index("human_rifle_diveforward").unwrap())
            .unwrap();
        let cb = b
            .decode_clip(b.clip_index("human_rifle_diveforward").unwrap())
            .unwrap();
        assert_eq!(ca, cb);
        let r1 = find_rigs(i1).unwrap();
        let r2 = find_rigs(i2).unwrap();
        let h1 = r1.iter().find(|r| r.name == "human_animation_0").unwrap();
        let h2 = r2.iter().find(|r| r.name == "human_0").unwrap();
        assert_eq!(h1.joints, h2.joints);
    }
}
