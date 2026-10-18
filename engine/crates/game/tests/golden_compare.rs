//! `common::compare_golden` compares R, G and B only (card FALPH): the
//! goldens' alpha plane is all 255 since the frame became opaque, and an older
//! golden's low alpha — or a capture's — must neither fail nor pass a
//! comparison on its own. CPU only: writes two tiny PNGs and compares.

mod common;

/// A 4×4 golden of one colour and alpha, written under the test's tmp dir.
fn golden(name: &str, px: [u8; 4]) -> std::path::PathBuf {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(
        &path,
        render::encode_png(&px.repeat(16), 4, 4).expect("png"),
    )
    .expect("write golden");
    path
}

#[test]
fn compare_golden_reads_rgb_and_ignores_the_alpha_plane() {
    let g = golden("falph_ghost.png", [40, 80, 120, 31]);
    let capture = [40u8, 80, 120, 255].repeat(16);
    assert_eq!(
        common::compare_golden(&g, &capture, 4, 4),
        None,
        "alpha only"
    );

    let moved = [40u8, 80, 160, 31].repeat(16);
    assert!(
        common::compare_golden(&g, &moved, 4, 4).is_some(),
        "a blue move must still fail"
    );
}
