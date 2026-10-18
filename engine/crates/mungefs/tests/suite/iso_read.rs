//! Tier-1: ISO9660 reader vs the 7z-extracted tree (data-gated).

use std::path::Path;

#[test]
fn iso_matches_extracted_bytes() {
    let iso_p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../iso/SWBF1.iso");
    let ext_p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../extracted/bf1/lvl/DATA/_LVL_PS2/MISSION.LVL"
    );
    if !Path::new(iso_p).exists() || !Path::new(ext_p).exists() {
        eprintln!("SKIP: iso or extracted data not present");
        return;
    }
    let f = std::fs::File::open(iso_p).unwrap();
    let mut iso = mungefs::iso::Iso::open(std::io::BufReader::new(f)).unwrap();
    assert!(iso.files.len() > 200, "BF1 disc has 223 files");
    let ix = iso
        .find("DATA/_LVL_PS2/MISSION.LVL")
        .expect("MISSION.LVL on disc");
    let entry_size = iso.files[ix].size;
    let from_iso = {
        let entry = mungefs::iso::IsoFile {
            path: iso.files[ix].path.clone(),
            lba: iso.files[ix].lba,
            size: entry_size,
        };
        iso.read_file(&entry).unwrap()
    };
    let from_fs = std::fs::read(ext_p).unwrap();
    assert_eq!(from_iso.len(), from_fs.len());
    assert_eq!(from_iso, from_fs, "byte-identical to 7z extraction");
}
