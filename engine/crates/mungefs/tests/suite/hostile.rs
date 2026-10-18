//! The no-panic contract on hostile bytes the shipped discs never hold: each
//! input here panicked, hung or allocated without bound before cleanse CLNM2
//! (2026-10-05). None needs disc data.
use mungefs::coll::CollisionMesh;
use mungefs::terrain::Terrain;

fn build_chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(8 + payload.len());
    v.extend(tag);
    v.extend((payload.len() as u32).to_le_bytes());
    v.extend(payload);
    while v.len() % 4 != 0 {
        v.push(0);
    }
    v
}

fn tern(patch_edge: u16, pchs: &[u8]) -> Vec<u8> {
    let mut info = vec![0u8; 28];
    info[0..4].copy_from_slice(&1f32.to_le_bytes());
    info[16..18].copy_from_slice(&16u16.to_le_bytes());
    info[18..20].copy_from_slice(&patch_edge.to_le_bytes());
    let mut body = build_chunk(b"NAME", b"t\0");
    body.extend(build_chunk(b"INFO", &info));
    body.extend(build_chunk(b"HGT8", &[0xFFu8, 0x00, 0xFF, 0x00]));
    body.extend(build_chunk(b"HEXP", &[0u8; 8]));
    body.extend(build_chunk(b"PCHS", pchs));
    build_chunk(b"tern", &body)
}

fn ptch_info(mask: u16, flag: u32) -> Vec<u8> {
    let mut v = mask.to_le_bytes().to_vec();
    v.extend([0u8; 8]);
    v.extend(flag.to_le_bytes());
    v
}

/// A flagged patch painting two layers with no vertex: `chunks_exact(0)`
/// panicked in `replacement_alpha`.
#[test]
fn a_replacement_patch_without_vertices_has_no_alpha() {
    let mut p = build_chunk(b"INFO", &ptch_info(0b11, 1));
    p.extend(build_chunk(b"POSI", &[]));
    p.extend(build_chunk(b"STRP", &[]));
    let buf = tern(8, &build_chunk(b"PTCH", &p));
    let c = mungefs::chunks(&buf).next().unwrap().unwrap();
    let t = Terrain::parse(&c).unwrap();
    assert!(t.patch_meshes.iter().flatten().all(|m| m.alpha.is_empty()));
}

/// INFO's patch edge sized `(P+1)²` bytes per 1-layer PTCH with nothing read
/// from a stream: eight PTCH at edge 4095 held 128 MiB. Past the cap the walk
/// gives no alpha, as a stream that does not walk does.
#[test]
fn a_patch_edge_past_the_cap_walks_no_alpha() {
    let mut pchs = Vec::new();
    for _ in 0..8 {
        pchs.extend(build_chunk(
            b"PTCH",
            &build_chunk(b"INFO", &ptch_info(1, 0)),
        ));
    }
    for edge in [4095, 0xFFFF] {
        let buf = tern(edge, &pchs);
        let c = mungefs::chunks(&buf).next().unwrap().unwrap();
        let t = Terrain::parse(&c).unwrap();
        assert_eq!(t.patch_alpha.len(), 8);
        assert_eq!(t.patch_alpha.iter().map(Vec::len).sum::<usize>(), 0);
    }
    // Under the cap the same file keeps its opaque nibbles.
    let buf = tern(8, &pchs);
    let c = mungefs::chunks(&buf).next().unwrap().unwrap();
    let t = Terrain::parse(&c).unwrap();
    assert_eq!(t.patch_alpha.iter().map(Vec::len).sum::<usize>(), 8 * 81);
}

fn coll(verts: &[[f32; 3]], leaves: &[Vec<u16>]) -> Vec<u8> {
    let mut info = Vec::new();
    let total: usize = leaves.iter().map(Vec::len).sum();
    for v in [verts.len() as u32, 0, leaves.len() as u32, total as u32] {
        info.extend(v.to_le_bytes());
    }
    info.extend([0u8; 24]);
    let mut posi = Vec::new();
    for v in verts {
        for c in v {
            posi.extend(c.to_le_bytes());
        }
    }
    let mut tree = Vec::new();
    for leaf in leaves {
        let mut p = vec![leaf.len() as u8];
        p.extend([0u8; 6]);
        for i in leaf {
            p.extend(i.to_le_bytes());
        }
        tree.extend(build_chunk(b"LEAF", &p));
    }
    let mut body = build_chunk(b"NAME", b"g\0");
    body.extend(build_chunk(b"INFO", &info));
    body.extend(build_chunk(b"POSI", &posi));
    body.extend(build_chunk(b"TREE", &tree));
    build_chunk(b"coll", &body)
}

/// n coplanar concave quads (darts) in a chain, each sharing one vertex
/// with the next; the chain runs against the leaf order.
fn chain(n: usize) -> (Vec<[f32; 3]>, Vec<Vec<u16>>) {
    let mut verts = Vec::new();
    // s_k at 3k, a_k at 3k+1, b_k at 3k+2; s_n closes the chain.
    for k in 0..n {
        let x = 10.0 * k as f32;
        verts.push([x, 0.0, 0.0]);
        verts.push([x + 5.0, 0.0, 5.0]);
        verts.push([x + 5.0, 0.0, 1.0]);
    }
    verts.push([10.0 * n as f32, 0.0, 0.0]);
    let mut leaves = vec![Vec::new(); n];
    for k in 0..n {
        let idx = if k == 0 { 0 } else { n - k };
        let b = (3 * k) as u16;
        leaves[idx] = vec![b, b + 1, b + 2, b + 3];
    }
    (verts, leaves)
}

/// The leaf grouping grew a group one leaf per pass over every leaf, and
/// again for every leaf of a group it could not repair: 200 chained leaves
/// (12 KB) took 15 s, 16 times more per doubling. One walk per seed now.
#[test]
fn a_long_chain_of_concave_leaves_is_grouped_in_one_walk() {
    let n = 400;
    let (v, l) = chain(n);
    let buf = coll(&v, &l);
    let c = mungefs::chunks(&buf).next().unwrap().unwrap();
    let m = CollisionMesh::parse(&c).unwrap();
    // The darts do not overlap, so nothing is repaired: two fan triangles each.
    assert!(m.repaired().is_empty());
    assert_eq!(m.triangles().len(), 2 * n);
}

/// A sample record with no frame count takes `size / 16 * 28`: in u32 that
/// overflowed (a panic with overflow checks) for a size past 2.4 GB.
#[test]
fn a_huge_sample_size_does_not_overflow_the_frame_count() {
    let h = mungefs::fnv1a_lowercase;
    let mut info = Vec::new();
    for (k, v) in [
        (h("name"), 1u32),
        (0, h("sample")),
        (h("id"), 7),
        (h("frequency"), 22050),
        (h("size"), 0xFFFF_FFF0),
    ] {
        info.extend(k.to_le_bytes());
        info.extend(v.to_le_bytes());
    }
    let mut bank = build_chunk(&h("info").to_le_bytes(), &info);
    bank.extend(build_chunk(&h("data").to_le_bytes(), &[0u8; 16]));
    let buf = build_chunk(&h("SampleBank").to_le_bytes(), &bank);
    let _ = mungefs::soundbank::find_banks(&buf);
}
