//! Dump mip 0 of PC (`_LVL_PC`) `tex_` chunks to PNG, for eyeballing the
//! [`mungefs::texture_pc`] decoder.
//!
//! ```text
//! pc_tex_dump <lvl-file|-> <out-dir> [label] [texture-name ...]
//! ```
//!
//! `-` reads the LVL from stdin, which is how the Harrisonfog mod LVLs are
//! inspected without unpacking the archives:
//!
//! ```text
//! unzip -p BS1_extract_in_addon_folder.2.zip BS1/data/_LVL_PC/SIDE/BES.lvl \
//!   | cargo run -p mungefs --example pc_tex_dump -- - /tmp/w1 bes
//! ```
//!
//! With no name filter it writes one texture per distinct pixel format it
//! meets, as `<label>_<texture>_<format>.png`. PNGs are written with a
//! stored-deflate encoder inlined below so `mungefs` keeps its single
//! dependency.

use std::collections::BTreeSet;
use std::io::Read;

use mungefs::texture_pc::find_textures;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: pc_tex_dump <lvl-file|-> <out-dir> [label] [texture-name ...]");
        std::process::exit(2);
    }
    let (src, out_dir) = (&args[0], std::path::Path::new(&args[1]));
    let label = args.get(2).cloned().unwrap_or_else(|| {
        std::path::Path::new(src)
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "lvl".into())
    });
    let wanted: BTreeSet<String> = args[3..].iter().map(|s| s.to_lowercase()).collect();

    let buf = if src == "-" {
        let mut v = Vec::new();
        std::io::stdin().read_to_end(&mut v).expect("read stdin");
        v
    } else {
        std::fs::read(src).unwrap_or_else(|e| panic!("{src}: {e}"))
    };
    let inner = mungefs::open(&buf).expect("ucfb");
    std::fs::create_dir_all(out_dir).expect("create out dir");

    let mut seen: BTreeSet<&'static str> = BTreeSet::new();
    for tex in find_textures(inner) {
        if !wanted.is_empty() && !wanted.contains(&tex.name.to_lowercase()) {
            continue;
        }
        if !tex.unsupported.is_empty() {
            eprintln!("{}: undecodable formats {:?}", tex.name, tex.unsupported);
        }
        for surface in &tex.surfaces {
            let fmt = surface.format;
            // Default mode: one sample per format, and skip the giants.
            if wanted.is_empty() && (!seen.insert(fmt.name()) || surface.width > 1024) {
                continue;
            }
            let Some(mip) = surface.mip0() else { continue };
            let path = out_dir.join(format!("{label}_{}_{}.png", tex.name, fmt.name()));
            std::fs::write(&path, encode_png(&mip.rgba, mip.width, mip.height))
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            println!(
                "{:<40} {:<9} {:>4}x{:<4} mips={:<2} faces={} -> {}",
                tex.name,
                fmt.name(),
                surface.width,
                surface.height,
                surface.mip_count,
                surface.faces.len(),
                path.display()
            );
        }
    }
    if wanted.is_empty() {
        println!("formats dumped: {:?}", seen);
    }
}

/// Minimal RGBA8 PNG writer: one uncompressed-deflate zlib stream, so it
/// needs no compression library. Output is large but valid.
fn encode_png(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut raw = Vec::with_capacity(rgba.len() + height as usize);
    for row in rgba.chunks_exact(width as usize * 4) {
        raw.push(0); // filter type 0 (None)
        raw.extend_from_slice(row);
    }

    let mut z = vec![0x78, 0x01]; // zlib header, deflate/32K window
    for (i, block) in raw.chunks(0xFFFF).enumerate() {
        let last = (i + 1) * 0xFFFF >= raw.len();
        z.push(u8::from(last));
        z.extend((block.len() as u16).to_le_bytes());
        z.extend((!(block.len() as u16)).to_le_bytes());
        z.extend_from_slice(block);
    }
    z.extend(adler32(&raw).to_be_bytes());

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend(width.to_be_bytes());
    ihdr.extend(height.to_be_bytes());
    ihdr.extend([8, 6, 0, 0, 0]); // 8-bit, truecolour+alpha, no interlace

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    push_chunk(&mut png, b"IHDR", &ihdr);
    push_chunk(&mut png, b"IDAT", &z);
    push_chunk(&mut png, b"IEND", &[]);
    png
}

fn push_chunk(out: &mut Vec<u8>, tag: &[u8; 4], payload: &[u8]) {
    out.extend((payload.len() as u32).to_be_bytes());
    out.extend(tag);
    out.extend(payload);
    let mut crc = crc32(0xFFFF_FFFF, tag);
    crc = crc32(crc, payload);
    out.extend((!crc).to_be_bytes());
}

fn crc32(mut crc: u32, data: &[u8]) -> u32 {
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    b << 16 | a
}
