//! Which FILE each commander line of a battle plays (card SNDE, from audit
//! ANNC2's probe): per generated mission, the four armies' lines through the
//! mission's own sound library (`SoundLib::load_for`), each take's path with
//! a hash of its bytes and the sub-level whose definition it is. An army the
//! mission does not field (the port's era mix) shows where its lines fall
//! through to.
//!
//! audit_takes [mission file stem...]   default: every mission
//! rows: mission, army, role, take, sub-level hash, path, bytes, byte hash
//! `OPENBF_SOUND_ROOT=<dir>` reads a scratch bake.

use std::path::PathBuf;

const ARMIES: [&str; 4] = ["all", "imp", "rep", "cis"];
const ROLES: [&str; 7] = [
    "off_com_report_us_overwhelmed",
    "off_com_report_enemy_losing",
    "off_defeat_im",
    "off_victory_im",
    "off_com_report_captured_commandpost",
    "off_com_report_lost_commandpost",
    "off_com_report_enemyCaptured_commandpost",
];

fn fnv64(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn main() {
    let only: Vec<String> = std::env::args().skip(1).collect();
    let root =
        std::env::var_os("OPENBF_SOUND_ROOT").map_or_else(game::assets::baked_root, PathBuf::from);
    let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../missions"));
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("missions dir")
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "ron"))
        .collect();
    paths.sort();
    for path in paths {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        if !only.is_empty() && !only.contains(&name) {
            continue;
        }
        let mission = game::mission::MissionDef::load(&path).expect("mission");
        let Some(sl) = game::sounds::SoundLib::load_for(&root.join(&mission.map), &mission) else {
            println!("{name}\tNO SOUND LIBRARY");
            continue;
        };
        for army in ARMIES {
            for role in ROLES {
                let line = format!("{army}_{role}");
                let level = sl.level_of(&line).unwrap_or(0);
                let mut nth = 0;
                while let Some(path) = sl.take_path(&line, nth) {
                    let (len, hash) = match std::fs::read(&path) {
                        Ok(b) => (b.len(), fnv64(&b)),
                        Err(_) => (0, 0),
                    };
                    let rel = path.strip_prefix(&root).unwrap_or(&path);
                    println!(
                        "{name}\t{army}\t{role}\t{nth}\t{level:08x}\t{}\t{len}\t{hash:016x}",
                        rel.display()
                    );
                    nth += 1;
                }
                if nth == 0 {
                    println!(
                        "{name}\t{army}\t{role}\t-\t{level:08x}\t{}\t0\t-",
                        if sl.has(&line) {
                            "(property, no take)"
                        } else {
                            "(no property)"
                        }
                    );
                }
            }
        }
    }
}
