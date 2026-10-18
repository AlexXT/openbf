//! Per-`modl` segment census for one LVL: what the disc declares against what
//! the parser produced. Written for queue item 8 (the BF1 BES1 "sky sliver"),
//! whose whole question was "did the bake drop geometry?".
//!
//! ```text
//! cargo run --release -p mungefs --example segdump -- <LVL> [modl name]
//! ```
//!
//! Without a name it prints one line per `modl`; with one it also prints every
//! segment (counts, material flags, textures, local bbox).
//!
//! The `idx` column is the sum of the segments' `segm INFO indexCount`; the
//! `@tail` column is the LAST u32 of `modl INFO` (offset 64 of BF1's 68-byte
//! INFO, 68 of BF2's 72-byte one). MEASURED 2026-09-08 over 445 models in
//! seven LVLs (BF1 BES1/TAT1/HOT1/KAS2 = 184, BF2 TAT2/COR1/HOT1 = 261):
//! the two are equal for EVERY model, so `@tail` is the model's total index
//! count and a `idx != @tail` line means the parse lost indices. Note this is
//! NOT the field `model::Model::face_count` reads (offset 60/64): that one is
//! 1..14 on models of up to 3,758 triangles, so whatever it is, it is not a
//! face count — see `analysis/a2-bake-formats.md` §Models `modl`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: segdump <LVL> [modl name]");
        std::process::exit(2);
    }
    let buf = std::fs::read(&args[1])?;
    let want = args.get(2).cloned().unwrap_or_default();
    let (mut models, mut mismatched) = (0usize, 0usize);
    for c in mungefs::content_chunks(mungefs::open(&buf)?)? {
        if &c.tag != b"modl" {
            continue;
        }
        let m = mungefs::model::Model::parse(&c)?;
        if !want.is_empty() && !m.name.eq_ignore_ascii_case(&want) {
            continue;
        }
        models += 1;
        let info = c.child(b"INFO").ok_or("modl with no INFO")?.payload;
        let w = |o: usize| u32::from_le_bytes(info[o..o + 4].try_into().unwrap());
        let tail = if info.len() >= 72 { w(68) } else { w(64) };
        let tris: usize = m.segments.iter().map(|s| s.triangles.len()).sum();
        let idx: u32 = m.segments.iter().map(|s| s.index_count).sum();
        if idx != tail {
            mismatched += 1;
        }
        println!(
            "modl {:34} info={} segs={} tris={:5} idx={:5} @tail={:5}{}",
            m.name,
            info.len(),
            m.segments.len(),
            tris,
            idx,
            tail,
            if idx == tail { "" } else { "   INDEX MISMATCH" }
        );
        if want.is_empty() {
            continue;
        }
        for (i, s) in m.segments.iter().enumerate() {
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for p in &s.positions {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            println!(
                "  seg{i:2}: vtx={:5} (parsed {:5}) idx={:5} tris={:5} flags=0x{:X} tex={:?} bbox=({:.1},{:.1},{:.1})..({:.1},{:.1},{:.1})",
                s.vertex_count, s.positions.len(), s.index_count, s.triangles.len(),
                s.material_flags, s.textures, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]
            );
        }
    }
    println!("{models} modl(s), {mismatched} with idx != @tail");
    Ok(())
}
