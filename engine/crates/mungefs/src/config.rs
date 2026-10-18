//! Munged config trees (PC-identical, LibSWBF2 `config/DATA`): a flat run of
//! `DATA` records where a `SCOP` chunk attaches to the DATA immediately before
//! it as its scope. Used for BF2 `path` chunks (whole .pth files), `lght`,
//! `mcfg`, and the CORE.LVL config blocks.
//!
//! DATA payload (byte-verified on BF2 TAT2.LVL path chunks, 2026-08-30):
//! `[u32 fnv1a(name)][u8 argCount][argCount × 4-byte values][u32 blobLen][blob]`.
//! Numeric values are f32 (e.g. `Nodes(9)` stores 9.0f).
//!
//! ERRATUM (2026-08-31): a string argument was described as "the u32 `4` in
//! its slot". That is the one-argument special case of the real rule, and it
//! silently loses every argument after the first string. A string slot holds
//! **`4 * argCount + the byte offset of that string inside the blob`**, and
//! the blob packs NUL-terminated strings back to back. Byte-verified on BF2
//! MUS1's `sky_`:
//!
//! ```text
//! argc = 3, args = [12, 38, 0x40C00000], blobLen = 35
//! blob = "mus_sfx_lavaspray_skydome\0hp_lava5\0"
//!   12 -> 12 - 12 =  0 -> "mus_sfx_lavaspray_skydome"
//!   38 -> 38 - 12 = 26 -> "hp_lava5"
//!   0x40C00000                        -> 6.0f
//! => Effect("mus_sfx_lavaspray_skydome", "hp_lava5", 6.0)
//! ```
//!
//! `4 * argCount` is exactly why a 1-argument string record stores `4` and why
//! `lght`'s `Light("sun")` (argc 2) stores `8`. A slot is read as a string
//! only when it addresses a NUL-terminated UTF-8 run inside the blob;
//! a real f32 could only collide with that range at ~1e-44, which no authored
//! value is.

use crate::{Chunk, UcfbError};

#[derive(Debug, Clone)]
pub struct ConfigNode {
    pub hash: u32,
    /// Raw 4-byte argument slots.
    pub values: Vec<u32>,
    /// Per-argument string, where that slot addresses the trailing blob (see
    /// the module doc). `strings[i]` and a numeric `f32(i)` are mutually
    /// exclusive.
    pub strings: Vec<Option<String>>,
    /// The first string argument — the common case (`Geometry("x")`,
    /// `Light("sun")`, `Path("CP1_SpawnPath")`).
    pub string: Option<String>,
    /// The SCOP that followed this DATA, parsed recursively.
    pub children: Vec<ConfigNode>,
}

impl ConfigNode {
    /// Argument `i` as an f32, or `None` if it is absent or is a string.
    pub fn f32(&self, i: usize) -> Option<f32> {
        if self.strings.get(i).is_some_and(Option::is_some) {
            return None;
        }
        self.values.get(i).map(|v| f32::from_bits(*v))
    }

    /// Argument `i` as a string, if that slot addresses the blob.
    pub fn str_arg(&self, i: usize) -> Option<&str> {
        self.strings.get(i)?.as_deref()
    }

    pub fn find(&self, hash: u32) -> Option<&ConfigNode> {
        self.children.iter().find(|c| c.hash == hash)
    }

    pub fn find_all(&self, hash: u32) -> impl Iterator<Item = &ConfigNode> {
        self.children.iter().filter(move |c| c.hash == hash)
    }

    /// Every argument slot read as an f32, string slots included — what the
    /// `unknown` lists in `light`/`sky` keep, which record what an unhandled
    /// key HELD rather than what it meant.
    pub fn floats(&self) -> Vec<f32> {
        self.values.iter().map(|v| f32::from_bits(*v)).collect()
    }

    /// The first three arguments; `None` if any is absent or is a string.
    pub fn f3(&self) -> Option<[f32; 3]> {
        Some([self.f32(0)?, self.f32(1)?, self.f32(2)?])
    }

    /// The first four, same rule.
    pub fn f4(&self) -> Option<[f32; 4]> {
        Some([self.f32(0)?, self.f32(1)?, self.f32(2)?, self.f32(3)?])
    }

    /// The first `N` arguments with an absent or string slot read as `0.0` —
    /// the form the authored pairs take (`FogRange`, `Angle`, `BackAngle`,
    /// `Cone`, `TileUV`, `OffsetUV`, `Height`) and the one 4-wide
    /// (`LowResTerrain FogColor`).
    pub fn floats_or_zero<const N: usize>(&self) -> [f32; N] {
        std::array::from_fn(|i| self.f32(i).unwrap_or(0.0))
    }
}

fn parse_data(payload: &[u8]) -> Option<ConfigNode> {
    if payload.len() < 5 {
        return None;
    }
    let hash = u32::from_le_bytes(payload[0..4].try_into().unwrap());
    let count = payload[4] as usize;
    let mut values = Vec::with_capacity(count);
    let mut off = 5;
    for _ in 0..count {
        if off + 4 > payload.len() {
            break;
        }
        values.push(u32::from_le_bytes(
            payload[off..off + 4].try_into().unwrap(),
        ));
        off += 4;
    }
    // The trailing blob, if present. `len` is its byte count including the
    // NUL of the last string.
    let mut blob: &[u8] = &[];
    if off + 4 <= payload.len() {
        let len = u32::from_le_bytes(payload[off..off + 4].try_into().unwrap()) as usize;
        if len > 0 && off + 4 + len <= payload.len() {
            blob = &payload[off + 4..off + 4 + len];
        }
    }
    // A slot pointing into the blob is a string argument; see the module doc.
    let base = 4 * values.len() as u32;
    let strings: Vec<Option<String>> = values
        .iter()
        .map(|&v| {
            let at = v.checked_sub(base)? as usize;
            let rest = blob.get(at..)?;
            let end = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
            std::str::from_utf8(&rest[..end]).ok().map(str::to_owned)
        })
        .collect();
    let string = strings.iter().flatten().next().cloned();
    Some(ConfigNode {
        hash,
        values,
        strings,
        string,
        children: Vec::new(),
    })
}

/// Cap on SCOP nesting. Every level is another `parse_tree` frame, so a chunk
/// made of nothing but nested SCOPs overflows the stack — an abort no caller
/// can catch, reachable straight off a disc LVL (`path`, `lght`, `snd_`).
/// Shipped configs nest a handful of levels; 64 is far past any of them.
const MAX_DEPTH: usize = 64;

/// Parse the DATA/SCOP run inside `container` (a `path`, `lght`, SCOP...).
pub fn parse_tree(container: &Chunk<'_>) -> Result<Vec<ConfigNode>, UcfbError> {
    parse_tree_at(container, 0)
}

fn parse_tree_at(container: &Chunk<'_>, depth: usize) -> Result<Vec<ConfigNode>, UcfbError> {
    if depth > MAX_DEPTH {
        return Err(UcfbError::Malformed(format!(
            "config SCOP nested deeper than {MAX_DEPTH}"
        )));
    }
    let mut out: Vec<ConfigNode> = Vec::new();
    for c in container.children() {
        let c = c?;
        match &c.tag {
            b"DATA" => {
                if let Some(n) = parse_data(c.payload) {
                    out.push(n);
                }
            }
            b"SCOP" => {
                let kids = parse_tree_at(&c, depth + 1)?;
                if let Some(last) = out.last_mut() {
                    last.children = kids;
                } else {
                    out.extend(kids);
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_chunk;

    /// The string-blob erratum, on the record that found it: BF2 MUS1's
    /// `Effect("mus_sfx_lavaspray_skydome", "hp_lava5", 6.0)`. The old
    /// reading returned only the first string and read the second slot as a
    /// denormal float.
    #[test]
    fn a_record_can_carry_several_string_arguments() {
        let mut payload = Vec::new();
        payload.extend(0x6e6e_8d54u32.to_le_bytes()); // "Effect"
        payload.push(3);
        payload.extend(12u32.to_le_bytes()); // 4*3 + 0
        payload.extend(38u32.to_le_bytes()); // 4*3 + 26
        payload.extend(6.0f32.to_bits().to_le_bytes());
        let blob = b"mus_sfx_lavaspray_skydome\0hp_lava5\0";
        payload.extend((blob.len() as u32).to_le_bytes());
        payload.extend(blob);
        let buf = build_chunk(b"sky_", &build_chunk(b"DATA", &payload));
        let c = crate::chunks(&buf).next().unwrap().unwrap();
        let n = &parse_tree(&c).unwrap()[0];
        assert_eq!(n.str_arg(0), Some("mus_sfx_lavaspray_skydome"));
        assert_eq!(n.str_arg(1), Some("hp_lava5"));
        assert_eq!(n.str_arg(2), None);
        assert_eq!(n.f32(2), Some(6.0));
        // A string slot is not a number, however small its bit pattern.
        assert_eq!(n.f32(0), None);
        assert_eq!(n.string.as_deref(), Some("mus_sfx_lavaspray_skydome"));
    }

    /// A `path`/`lght`/`snd_` chunk of nothing but nested SCOPs used to
    /// recurse once per level and overflow the stack — a process abort, not
    /// an error a caller could catch.
    #[test]
    fn runaway_scop_nesting_errors() {
        let mut inner = Vec::new();
        for _ in 0..50_000 {
            inner = build_chunk(b"SCOP", &inner);
        }
        let buf = build_chunk(b"path", &inner);
        let c = crate::chunks(&buf).next().unwrap().unwrap();
        let e = parse_tree(&c).unwrap_err();
        assert!(matches!(e, UcfbError::Malformed(_)), "{e}");
    }

    /// Nesting up to the cap still parses (the fix must not clip real trees).
    #[test]
    fn nesting_within_the_cap_parses() {
        let mut inner = build_chunk(b"DATA", &[7, 0, 0, 0, 0]);
        for _ in 0..MAX_DEPTH - 1 {
            let mut lvl = build_chunk(b"DATA", &[1, 0, 0, 0, 0]);
            lvl.extend(build_chunk(b"SCOP", &inner));
            inner = lvl;
        }
        let buf = build_chunk(b"path", &inner);
        let c = crate::chunks(&buf).next().unwrap().unwrap();
        let mut node = &parse_tree(&c).unwrap()[0];
        let mut depth = 1;
        while let Some(k) = node.children.first() {
            node = k;
            depth += 1;
        }
        assert_eq!(depth, MAX_DEPTH);
        assert_eq!(node.hash, 7);
    }
}
