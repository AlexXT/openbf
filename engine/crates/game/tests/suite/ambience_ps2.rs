//! Card AMBS's referees: the world's sound emitters of each showcase map
//! against the PS2's own sound entities in RAM, and Utapau's wind's level
//! by distance through the baked bank.
//!
//! The PS2 list (`tests/data/ambience_ps2.txt`) is every entity of the sound
//! entity list in one reference save state per map: the `soundstream`
//! regions, the `com_snd_amb_static` objects and, on BF1, the prop emitters
//! (BF2 keeps a prop's sound on the prop, not in the list, so its props are
//! printed and not compared). The port's list is `game::ambience::emitters`
//! over the baked manifest with the conquest layer merged, as a battle
//! builds it. Name, position and the three distances must agree to 0.01 m;
//! an entity whose flags are 0 takes the property's own distances (`None`).
//! Skips a map that is not baked.

use game::ambience::{Ambience, Emitter, Kind};
use game::sounds::SoundLib;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    game::assets::baked_root()
}

/// The battle's manifest: the map's, with its conquest layer merged
/// (`Scene::apply_mode_layer`).
fn battle_manifest(dir: &Path) -> Option<manifest::Manifest> {
    let txt = std::fs::read_to_string(dir.join("manifest.ron")).ok()?;
    let mut m: manifest::Manifest = ron::from_str(&txt).expect("manifest.ron");
    if let Some(i) = game::assets::mode_layer_index(&m, "con") {
        let l = m.layers.remove(i);
        m.regions.extend(l.regions);
        m.instances.extend(l.instances);
    }
    Some(m)
}

fn ambience(dir: &Path) -> Option<(manifest::Manifest, Ambience)> {
    let m = battle_manifest(dir)?;
    let a = Ambience::load(&m, dir, &root().join("classes"));
    Some((m, a))
}

/// One row as the PS2 file spells it: kind name x y z min max mute flags.
fn ps2_rows() -> Vec<(String, Vec<String>)> {
    let txt = include_str!("../data/ambience_ps2.txt");
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for l in txt
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let w: Vec<&str> = l.split_whitespace().collect();
        if w[0] == "map" {
            out.push((w[1].to_owned(), Vec::new()));
        } else {
            out.last_mut()
                .expect("a map line first")
                .1
                .push(l.to_owned());
        }
    }
    out
}

/// The port's emitter as a PS2 row (BF2 props have none).
fn row(e: &Emitter, bf2: bool) -> Option<String> {
    let kind = match e.kind {
        Kind::Stream => "stream",
        Kind::Static | Kind::Object => "static",
        Kind::Prop if !bf2 => "static",
        Kind::Prop => return None,
    };
    let flags = u8::from(e.min.is_some())
        | u8::from(e.max.is_some()) << 1
        | u8::from(e.mute.is_some()) << 2;
    // The PS2's constructor defaults stand where a flag is clear.
    let d = |v: Option<f32>, def: f32| v.unwrap_or(def);
    Some(format!(
        "{kind} {} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} {flags}",
        e.sound.to_ascii_lowercase(),
        e.pos.x,
        e.pos.y,
        e.pos.z,
        d(e.min, 1.0),
        d(e.max, 10.0),
        d(e.mute, 10.0),
    ))
}

/// Two rows agree when the words match and every number is within 0.01
/// (the file holds two decimals).
fn same(a: &str, b: &str) -> bool {
    let (a, b): (Vec<&str>, Vec<&str>) = (
        a.split_whitespace().collect(),
        b.split_whitespace().collect(),
    );
    a.len() == b.len()
        && a.iter()
            .zip(&b)
            .all(|(x, y)| match (x.parse::<f32>(), y.parse::<f32>()) {
                (Ok(x), Ok(y)) => (x - y).abs() <= 0.0101,
                _ => x == y,
            })
}

#[test]
fn emitters_are_the_ps2s_sound_entities() {
    let mut checked = 0;
    let mut bad = Vec::new();
    for (map, ps2) in ps2_rows() {
        let dir = root().join(&map);
        let Some((m, amb)) = ambience(&dir) else {
            println!("SKIP {map}: not baked");
            continue;
        };
        let bf2 = m.source_game.eq_ignore_ascii_case("bf2");
        let mut port: Vec<String> = amb.emitters.iter().filter_map(|e| row(e, bf2)).collect();
        port.sort();
        let mut ps2 = ps2.clone();
        ps2.sort();
        println!("{map}: PS2 {} entities, port {}", ps2.len(), port.len());
        let mut left: Vec<&String> = port.iter().collect();
        for r in &ps2 {
            match left.iter().position(|p| same(p, r)) {
                Some(i) => {
                    println!("  both  {r}");
                    left.remove(i);
                }
                None => {
                    println!("  PS2   {r}");
                    bad.push(format!("{map}: PS2 only: {r}"));
                }
            }
        }
        for p in left {
            println!("  port  {p}");
            bad.push(format!("{map}: port only: {p}"));
        }
        for e in amb.emitters.iter().filter(|e| bf2 && e.kind == Kind::Prop) {
            println!(
                "  (BF2 prop, not in the PS2 list) {} at {:.2?}",
                e.sound, e.pos
            );
        }
        checked += 1;
    }
    println!("{checked} maps checked");
    assert!(
        bad.is_empty(),
        "{} differences:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

#[test]
fn utapau_wind_level_through_the_bank() {
    let dir = root().join("bf2_uta1");
    let (Some((_, amb)), Some(lib)) = (ambience(&dir), SoundLib::load(&dir)) else {
        println!("SKIP: bf2_uta1 not baked");
        return;
    };
    let i = amb
        .emitters
        .iter()
        .position(|e| e.sound == "uta_amb_wind")
        .expect("Utapau's wind region");
    let c = amb.emitters[i].pos;
    // RollOff 0: 1 inside 235.70 m, linear to 0 at 471.40 m.
    for (d, want) in [
        (100.0, 1.0),
        (235.0, 1.0),
        (353.55, 0.5),
        (412.475, 0.25),
        (471.5, 0.0),
    ] {
        let ears = c + glam::Vec3::new(d, 0.0, 0.0);
        let h = lib.ambient_heard(&amb, ears);
        let h = h
            .iter()
            .find(|h| h.index == i)
            .expect("the wind is in the bank");
        println!(
            "uta_amb_wind at {d:7.2} m: level {:.4} (want {want}), gain {:.4}",
            h.level, h.gain
        );
        assert!((h.level - want).abs() < 2e-3, "level {} at {d} m", h.level);
    }
}

/// Every emitter of the six showcase maps has a take the voices can play:
/// a stream's place in a disc LVL that is there, or a baked file (a
/// property missing from the bank would be silent). Skips a map not baked;
/// a disc stream is checked only when the disc's SOUND LVLs are extracted.
#[test]
fn every_showcase_emitter_has_a_take() {
    let mut bad = Vec::new();
    for map in [
        "bf1_nab1", "tat1", "bf1_yav2", "bf2_uta1", "bf2_myg1", "bf2_hot1",
    ] {
        let dir = root().join(map);
        let (Some((_, amb)), Some(mut lib)) = (ambience(&dir), SoundLib::load(&dir)) else {
            println!("SKIP {map}: not baked");
            continue;
        };
        let (mut streams, mut files) = (0, 0);
        for e in &amb.emitters {
            match lib.ambient_take(e) {
                Some(game::sounds::AmbientTake::Stream(t)) => {
                    streams += 1;
                    if t.size == 0
                        || (t.path.parent().is_some_and(Path::exists) && !t.path.exists())
                    {
                        bad.push(format!(
                            "{map}: {} streams from {}",
                            e.sound,
                            t.path.display()
                        ));
                    }
                }
                Some(game::sounds::AmbientTake::File(p)) => {
                    files += 1;
                    if !p.exists() {
                        bad.push(format!("{map}: {} names {}", e.sound, p.display()));
                    }
                }
                None => bad.push(format!(
                    "{map}: {} ({}) has no take",
                    e.sound,
                    e.kind.word()
                )),
            }
        }
        println!(
            "{map}: {} emitters, {streams} disc streams, {files} files",
            amb.emitters.len()
        );
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
