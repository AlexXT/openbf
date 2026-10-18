//! Tier-1/3: the BF1 loading beep decodes from CORE.LVL to the traced
//! reference values (RE: analysis/re-sound-banks.md). Data-gated.

#[test]
fn beep_decodes_to_traced_reference() {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../extracted/bf1/lvl/DATA/_LVL_PS2/CORE.LVL"
    );
    let Ok(core) = std::fs::read(p) else {
        eprintln!("SKIP: extracted CORE.LVL not present");
        return;
    };
    let out = std::env::temp_dir().join(format!("openbf-bake-test-beep-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let b = bake::beep::bake_beep(&core, out.to_str().unwrap()).expect("beep bakes");

    let (_, rate, pcm) = &b.outputs[0];
    // reference: 720 ADPCM bytes -> 1260 frames @ 3605 Hz
    assert_eq!(*rate, 3605);
    assert_eq!(pcm.len(), 1260);
    // peak near full scale (traced: -0.26 dBFS)
    let peak = pcm.iter().map(|s| s.unsigned_abs()).max().unwrap();
    assert!(peak > 30000, "peak={peak}");
    // dominant tone ~440.6 Hz (traced): Goertzel power at 440 must dwarf
    // neighbors at 300 and 600 Hz
    let p440 = bake::adpcm::goertzel(pcm, *rate as f64, 440.6);
    let p300 = bake::adpcm::goertzel(pcm, *rate as f64, 300.0);
    let p600 = bake::adpcm::goertzel(pcm, *rate as f64, 600.0);
    assert!(p440 > 50.0 * p300, "p440={p440:.1} p300={p300:.1}");
    assert!(p440 > 50.0 * p600, "p440={p440:.1} p600={p600:.1}");
    // as-played renders present with pitched rates
    assert_eq!(b.outputs[1].1, 7571); // 3605 * 2.1 rounded
    assert_eq!(b.outputs[2].1, 6489); // 3605 * 1.8
    assert!(out.join("ui_load_lp.wav").exists());

    // M-B4: the OGGs must be byte-identical between two bakes of the same
    // input. ffmpeg draws the Ogg stream serial at random; bake::beep pins it
    // to a hash of the file name and recomputes the page checksums, so a
    // rebake stops showing 24 changed bytes per file for no reason.
    if b.ogg_written {
        let out2 =
            std::env::temp_dir().join(format!("openbf-bake-test-beep-{}-b", std::process::id()));
        let _ = std::fs::remove_dir_all(&out2);
        bake::beep::bake_beep(&core, out2.to_str().unwrap()).expect("beep bakes twice");
        for (stem, _, _) in &b.outputs {
            let one = std::fs::read(out.join(format!("{stem}.ogg"))).unwrap();
            let two = std::fs::read(out2.join(format!("{stem}.ogg"))).unwrap();
            assert_eq!(one, two, "{stem}.ogg differs between two bakes");
            // The pinned serial is the same in every page of one file.
            assert_eq!(
                &one[14..18],
                &one[58 + 14..58 + 18],
                "{stem}.ogg page serials"
            );
        }
        let _ = std::fs::remove_dir_all(&out2);
    }
    let _ = std::fs::remove_dir_all(&out);
}
