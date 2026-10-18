//! Minimal KTX2 writer: uncompressed RGBA8 with a full mip chain, no
//! supercompression. Validated by round-trip with the `ktx2` reader crate.
//!
//! **The chain is COMPLETED here** — the writer used to store exactly the
//! levels it was handed, and the discs are full of short chains: 1,989 of the
//! 2,288 baked textures stopped early and 102 of those are 512 px or more on
//! the long side. A large texture with no chain shimmers at distance because
//! the sampler has nothing to fall back to, and the PC-stock replacements are
//! the worst of it (they are disproportionately the 512²/1024² ones). Doing
//! it at the single write point covers every producer — map textures, detail
//! companions, soldier and weapon sheets — rather than each remembering.

/// VK_FORMAT_R8G8B8A8_SRGB.
const VK_FORMAT_R8G8B8A8_SRGB: u32 = 43;

/// Size of mip `level` along an axis whose level-0 size is `base`.
fn mip_extent(base: u32, level: u32) -> u32 {
    (base >> level).max(1)
}

/// Extend `mips` down to 1x1 by box-filtering the last level it has.
///
/// Box filter, offline, in the STORED (sRGB) encoding rather than in linear
/// light. That is deliberate and matches what the rest of the bake does with
/// texel arithmetic — the same reasoning as the `_dtl` detail modulation,
/// which a GPU test pinned to gamma space (RE_LOG "BF1 detail textures").
/// Alpha is averaged with the colour; on a cutout material that softens the
/// mask with distance, which is what a mip chain is for.
///
/// Returns the input untouched when it is already complete, when it is
/// empty, or when a level is shorter than its dimensions claim (a malformed
/// source — better to ship the short chain than to fabricate from garbage).
fn complete_chain(width: u32, height: u32, mips: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let full = 32 - width.max(height).max(1).leading_zeros();
    let mut out = mips.to_vec();
    if out.is_empty() {
        return out;
    }
    while (out.len() as u32) < full {
        let level = out.len() as u32;
        let (pw, ph) = (mip_extent(width, level - 1), mip_extent(height, level - 1));
        let (w, h) = (mip_extent(width, level), mip_extent(height, level));
        let src = out.last().expect("non-empty");
        if src.len() < (pw * ph * 4) as usize {
            break;
        }
        let mut dst = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                // Clamped 2x2 box, so an odd dimension still averages.
                let (x0, y0) = (2 * x, 2 * y);
                let (x1, y1) = ((x0 + 1).min(pw - 1), (y0 + 1).min(ph - 1));
                for c in 0..4 {
                    let at = |xx: u32, yy: u32| src[((yy * pw + xx) * 4 + c) as usize] as u32;
                    let sum = at(x0, y0) + at(x1, y0) + at(x0, y1) + at(x1, y1);
                    dst[((y * w + x) * 4 + c) as usize] = ((sum + 2) / 4) as u8;
                }
            }
        }
        out.push(dst);
    }
    out
}

/// Write a KTX2 file from RGBA8 mip levels (level 0 first).
pub fn write_rgba8(width: u32, height: u32, mips: &[Vec<u8>]) -> Vec<u8> {
    let completed = complete_chain(width, height, mips);
    let mips: &[Vec<u8>] = &completed;
    let level_count = mips.len() as u32;

    // Basic DFD for RGBA8 (KHR descriptor: 24 B header + 16 B per sample).
    let sample_count = 4u32;
    let dfd_block_size = 24 + 16 * sample_count;
    let dfd_total = 4 + dfd_block_size;
    let mut dfd: Vec<u8> = Vec::new();
    dfd.extend_from_slice(&dfd_total.to_le_bytes());
    dfd.extend_from_slice(&0u32.to_le_bytes()); // vendorId 0 | descriptorType 0
    dfd.extend_from_slice(&2u16.to_le_bytes()); // versionNumber
    dfd.extend_from_slice(&(dfd_block_size as u16).to_le_bytes());
    dfd.push(1); // colorModel RGBSDA
    dfd.push(1); // colorPrimaries BT709
    dfd.push(2); // transferFunction SRGB
    dfd.push(0); // flags
    dfd.extend_from_slice(&[0, 0, 0, 0]); // texel dims (1x1x1x1 block -> 0s)
    dfd.extend_from_slice(&[4, 0, 0, 0, 0, 0, 0, 0]); // bytesPlane0=4
    for (i, channel) in [0u8, 1, 2, 15].iter().enumerate() {
        // channel: R,G,B,ALPHA
        let bit_offset = (i * 8) as u16;
        dfd.extend_from_slice(&bit_offset.to_le_bytes());
        dfd.push(7); // bitLength-1
        dfd.push(*channel);
        dfd.extend_from_slice(&[0, 0, 0, 0]); // samplePosition
        dfd.extend_from_slice(&0u32.to_le_bytes()); // sampleLower
        dfd.extend_from_slice(&255u32.to_le_bytes()); // sampleUpper
    }
    debug_assert_eq!(dfd.len(), dfd_total as usize);

    let header_len = 80usize;
    let index_len = level_count as usize * 24;
    let dfd_offset = header_len + index_len;
    let kvd_offset = dfd_offset + dfd.len();
    // levels stored last-to-smallest per spec recommendation; keep simple:
    // in file order after kvd (none), aligned to 4.
    let mut level_offsets = Vec::new();
    let mut data: Vec<u8> = Vec::new();
    let base = (kvd_offset + 3) & !3;
    for m in mips {
        while !(base + data.len()).is_multiple_of(4) {
            data.push(0);
        }
        level_offsets.push((base + data.len()) as u64);
        data.extend_from_slice(m);
    }

    let mut out = Vec::new();
    out.extend_from_slice(&[
        0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ]);
    out.extend_from_slice(&VK_FORMAT_R8G8B8A8_SRGB.to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes()); // typeSize
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // pixelDepth
    out.extend_from_slice(&0u32.to_le_bytes()); // layerCount
    out.extend_from_slice(&1u32.to_le_bytes()); // faceCount
    out.extend_from_slice(&level_count.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // supercompression: none
    out.extend_from_slice(&(dfd_offset as u32).to_le_bytes());
    out.extend_from_slice(&(dfd.len() as u32).to_le_bytes());
    out.extend_from_slice(&(kvd_offset as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // kvdByteLength
    out.extend_from_slice(&0u64.to_le_bytes()); // sgdByteOffset
    out.extend_from_slice(&0u64.to_le_bytes()); // sgdByteLength
    debug_assert_eq!(out.len(), header_len);
    for (off, m) in level_offsets.iter().zip(mips) {
        out.extend_from_slice(&off.to_le_bytes());
        out.extend_from_slice(&(m.len() as u64).to_le_bytes());
        out.extend_from_slice(&(m.len() as u64).to_le_bytes());
    }
    out.extend_from_slice(&dfd);
    while out.len() < base {
        out.push(0);
    }
    out.extend_from_slice(&data);
    out
}

#[cfg(test)]
mod tests {
    use super::{complete_chain, write_rgba8};

    /// A single level becomes a full chain down to 1x1: 64x64 -> 7 levels.
    #[test]
    fn a_lone_level_is_extended_to_one_by_one() {
        let mips = vec![vec![128u8; 64 * 64 * 4]];
        let chain = complete_chain(64, 64, &mips);
        assert_eq!(chain.len(), 7);
        for (i, m) in chain.iter().enumerate() {
            let e = (64u32 >> i).max(1) as usize;
            assert_eq!(m.len(), e * e * 4, "level {i}");
        }
        // A constant image stays constant through a box filter.
        assert!(chain.last().unwrap().iter().all(|&b| b == 128));
    }

    /// Non-square and non-power-of-two both terminate at 1x1 without
    /// panicking, and the axis that hits 1 first stays there.
    #[test]
    fn odd_and_oblong_sizes_terminate() {
        for (w, h) in [(64u32, 16u32), (12, 5), (1, 1), (3, 3)] {
            let chain = complete_chain(w, h, &[vec![7u8; (w * h * 4) as usize]]);
            let want = 32 - w.max(h).max(1).leading_zeros();
            assert_eq!(chain.len() as u32, want, "{w}x{h}");
            let last = chain.last().unwrap();
            assert_eq!(last.len(), 4, "{w}x{h} should end at one texel");
        }
    }

    /// The box filter averages, it does not pick: a 2x1 of 0 and 255 halves.
    #[test]
    fn the_filter_averages() {
        let src = vec![vec![0u8, 0, 0, 0, 255, 255, 255, 255]];
        let chain = complete_chain(2, 1, &src);
        assert_eq!(chain.len(), 2);
        // Both source texels land in the one destination texel.
        assert_eq!(chain[1], vec![128, 128, 128, 128]);
    }

    /// A chain that is already complete is left exactly as it is, and a
    /// malformed level stops the walk rather than fabricating from garbage.
    #[test]
    fn complete_and_malformed_inputs_are_left_alone() {
        let full = vec![vec![1u8; 2 * 2 * 4], vec![2u8; 4]];
        assert_eq!(complete_chain(2, 2, &full), full);
        assert!(complete_chain(8, 8, &[]).is_empty());
        let short = vec![vec![3u8; 4]]; // claims 8x8, holds one texel
        assert_eq!(complete_chain(8, 8, &short), short);
    }

    /// The writer's own KTX2 must read back through the `ktx2` crate with the
    /// dimensions and the bytes of every level it was handed — every bake
    /// path (PS2 textures, detail companions, `pc-tex`) depends on the levels
    /// that come IN surviving unchanged. The level COUNT may grow, because
    /// the writer completes a short chain.
    #[test]
    fn rgba8_round_trips_through_the_ktx2_reader() {
        let mips: Vec<Vec<u8>> = (0..3)
            .map(|l| {
                let (w, h) = (8u32 >> l, 8u32 >> l);
                (0..w * h)
                    .flat_map(|i| [i as u8, l as u8, 7, 255])
                    .collect()
            })
            .collect();
        let bytes = write_rgba8(8, 8, &mips);
        let r = ktx2::Reader::new(&bytes).expect("ktx2 parses");
        assert_eq!((r.header().pixel_width, r.header().pixel_height), (8, 8));
        // The three levels handed in are stored VERBATIM; the writer only
        // adds the 1x1 the source stopped short of (see `complete_chain`).
        assert_eq!(r.header().level_count, 4);
        let levels: Vec<_> = r.levels().collect();
        for (i, level) in levels.iter().take(3).enumerate() {
            assert_eq!(level.data, mips[i].as_slice(), "level {i}");
        }
        assert_eq!(levels[3].data.len(), 4, "the generated 1x1");
    }
}
