//! PS2 RLE codec used by `tex_` BODY (element = 1 byte), `pal_` BODY
//! (element = 4 bytes, one RGBA color), terrain `HGT8` (1 byte) and the
//! terrain's `ALPH` / `COLR` streams ([`decode_prefix`] takes a counted
//! prefix of one stream and says where it stopped).
//!
//! Spec (audit A2 §4, verified 13/13 texture bodies + 107/107 palettes, and
//! independently re-verified by the lead on tat_skiff01_lowrez):
//!   control byte c:
//!     c < 0x80  → copy (c & 0x7F) + 1 literal elements from the stream
//!     c >= 0x80 → repeat the next single element (c & 0x7F) + 1 times
//! Matches swbf-unmunge `src/handle_texture_ps2.cpp` `decompress_body`.

use crate::UcfbError;

/// Cap on decoded output. The codec expands up to 64× (one control byte plus
/// one element yields 128 elements), so a BODY read off a disc is a
/// decompression bomb: every caller checks the decoded length only after the
/// whole allocation exists. The largest legitimate stream on either disc
/// decodes to 256 KiB (a 256×256 terrain HGT8, a 512×512 texture mip), so this
/// is 64× past anything real.
pub const MAX_OUTPUT: usize = 16 << 20;

/// Decode an RLE stream. `elem` is the element size in bytes (1 or 4).
/// Errors once the output would exceed [`MAX_OUTPUT`].
pub fn decode(src: &[u8], elem: usize) -> Result<Vec<u8>, UcfbError> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < src.len() {
        let c = src[i] as usize;
        i += 1;
        let n = (c & 0x7F) + 1;
        if out.len() + n * elem > MAX_OUTPUT {
            return Err(UcfbError::Malformed(format!(
                "rle: output past the {MAX_OUTPUT}-byte cap at input offset {i}"
            )));
        }
        if c < 0x80 {
            let len = n * elem;
            if i + len > src.len() {
                return Err(UcfbError::Truncated(i));
            }
            out.extend_from_slice(&src[i..i + len]);
            i += len;
        } else {
            if i + elem > src.len() {
                return Err(UcfbError::Truncated(i));
            }
            let e = &src[i..i + elem];
            for _ in 0..n {
                out.extend_from_slice(e);
            }
            i += elem;
        }
    }
    Ok(out)
}

/// Decode whole control units from `src` until at least `need` bytes are out,
/// the way the PS2 walks the terrain's `PCHS -> ALPH` stream: one patch's
/// share at a time from a running pointer (B2 `FUN_00572c98`, B1
/// `FUN_00488850`; analysis/re-terrain-layers.md §1). Returns the first
/// `need` bytes and how many input bytes were consumed. Errors on a stream
/// that ends before `need` bytes, and past [`MAX_OUTPUT`].
pub fn decode_prefix(src: &[u8], elem: usize, need: usize) -> Result<(Vec<u8>, usize), UcfbError> {
    if need > MAX_OUTPUT {
        return Err(UcfbError::Malformed(format!("rle: {need} bytes asked for")));
    }
    let mut out = Vec::with_capacity(need + 128 * elem);
    let mut i = 0usize;
    while out.len() < need {
        let Some(&c) = src.get(i) else {
            return Err(UcfbError::Truncated(i));
        };
        let c = c as usize;
        i += 1;
        let n = (c & 0x7F) + 1;
        let len = if c < 0x80 { n * elem } else { elem };
        if i + len > src.len() {
            return Err(UcfbError::Truncated(i));
        }
        if c < 0x80 {
            out.extend_from_slice(&src[i..i + len]);
        } else {
            for _ in 0..n {
                out.extend_from_slice(&src[i..i + elem]);
            }
        }
        i += len;
    }
    out.truncate(need);
    Ok((out, i))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A prefix decode stops after the control unit that reaches `need`, and
    /// says where the next one starts.
    #[test]
    fn prefix_decode_walks_a_running_stream() {
        let src = [0x01, 0x10, 0x20, 0x82, 0xAA, 0x80, 0xBB];
        assert_eq!(decode_prefix(&src, 1, 2).unwrap(), (vec![0x10, 0x20], 3));
        assert_eq!(decode_prefix(&src[3..], 1, 3).unwrap(), (vec![0xAA; 3], 2));
        assert_eq!(decode_prefix(&src[5..], 1, 1).unwrap(), (vec![0xBB], 2));
        assert!(decode_prefix(&src[5..], 1, 2).is_err());
    }

    #[test]
    fn literals_and_runs_elem1() {
        // 2 literals (0x01 => n=2), then run of 3 (0x82 => n=3) of 0xAA
        let src = [0x01, 0x10, 0x20, 0x82, 0xAA];
        assert_eq!(decode(&src, 1).unwrap(), vec![0x10, 0x20, 0xAA, 0xAA, 0xAA]);
    }

    #[test]
    fn runs_elem4() {
        // run of 2 (0x81) of the 4-byte color RGBA
        let src = [0x81, 1, 2, 3, 4];
        assert_eq!(decode(&src, 4).unwrap(), vec![1, 2, 3, 4, 1, 2, 3, 4]);
    }

    #[test]
    fn truncated_input_errors() {
        assert!(decode(&[0x03, 0x10], 1).is_err()); // promises 4 literals, has 1
        assert!(decode(&[0x81], 4).is_err()); // run with no element
    }

    /// The codec expands 64×, so a BODY off a disc is a decompression bomb:
    /// 600 KiB of maximal runs used to decode to 38 MB (and 15 MB to 1 GB),
    /// because callers only check the length after the allocation exists.
    #[test]
    fn output_cap_stops_a_decompression_bomb() {
        let src = [0xFFu8, 0xAA].repeat(300 * 1024);
        let e = decode(&src, 1).unwrap_err();
        assert!(matches!(e, UcfbError::Malformed(_)), "{e}");
    }

    /// The cap must clear anything real: the largest shipped stream is a
    /// 256×256 HGT8 (64 KiB out).
    #[test]
    fn real_sized_streams_are_under_the_cap() {
        let src = [0xFFu8, 0xAA].repeat(512); // 128 KiB out
        assert_eq!(decode(&src, 1).unwrap().len(), 512 * 128);
        const { assert!(MAX_OUTPUT >= 256 * 1024) };
    }
}
