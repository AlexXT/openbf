//! Card SNDE's referees: a battle's sounds are its mission's own sub-level's.
//!
//! A planet's SOUND LVL holds one `lvl_` sub-level per map and era and the
//! PS2 loads the one the mission script names; the port bakes the whole LVL
//! and `SoundLib` resolves a name in the battle's sub-level first. Two
//! checks over every generated mission, through the game's own loader
//! (`SoundLib::load_for`):
//!
//! - against the DISC: every name the mission's sub-level defines resolves
//!   to that sub-level's stream, takes, samples and layers, read here from
//!   the LVL itself (not from the bake);
//! - the commander's lines of the mission's two armies come from the
//!   mission's own sub-level, and on BF1 never from the other era's voice
//!   stream.
//!
//! `OPENBF_SOUND_ROOT=<dir>` reads a scratch bake (`<dir>/<map>/sounds`
//! beside the map's manifest). Skips when nothing is baked, the discs are
//! not extracted, or the bake predates the card (no sub-level on a record).

use game::classdb::fnv1a_lowercase;
use game::layout::MapLayout;
use game::mission::MissionDef;
use game::sounds::SoundLib;
use mungefs::sound_props::SoundProp;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    std::env::var_os("OPENBF_SOUND_ROOT").map_or_else(game::assets::baked_root, PathBuf::from)
}

fn missions() -> Vec<(String, MissionDef)> {
    let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../missions"));
    // None in a public checkout until scripts/pack.py generates them: the tests skip.
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = rd
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "ron"))
        .collect();
    paths.sort();
    let name = |p: &Path| p.file_stem().unwrap().to_string_lossy().into_owned();
    paths
        .iter()
        .map(|p| (name(p), MissionDef::load(p).expect("mission")))
        .collect()
}

/// The battle's library, when the map is baked with sub-levels.
fn library(mission: &MissionDef) -> Option<SoundLib> {
    let lib = SoundLib::load_for(&root().join(&mission.map), mission)?;
    lib.has_levels().then_some(lib)
}

fn disc(game: &str, rel: &str) -> PathBuf {
    let e = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../extracted");
    PathBuf::from(format!("{e}/{game}/{rel}"))
}

fn audible(p: &SoundProp) -> bool {
    !p.samples.is_empty() || !p.layers.is_empty() || !p.segments.is_empty()
}

type Audio = (Option<u32>, Vec<u32>, Vec<u32>, Vec<u32>);

/// One sub-level of a SOUND LVL as the PS2 holds it: per name, what it
/// plays - the layers its parser keeps, and for a name defined twice the
/// last definition that can make a noise (the port's own rule inside one
/// sub-level).
fn disc_sublevel(game: &str, lvl: &str, sub: &str) -> Option<HashMap<u32, Audio>> {
    let read = |p: PathBuf| std::fs::read(p).ok();
    let lvl = lvl.to_uppercase();
    let buf = read(disc(game, &format!("sound/DATA/_LVL_PS2/SOUND/{lvl}.LVL")))?;
    let core = read(disc(game, "lvl/DATA/_LVL_PS2/CORE.LVL"))?;
    let kind = fnv1a_lowercase("soundproperties");
    let core: HashSet<u32> = mungefs::sound_props::find_all(mungefs::open(&core).ok()?)
        .iter()
        .filter(|p| p.kind == kind)
        .map(|p| p.name_hash)
        .collect();
    let subs = mungefs::sound_props::find_named_sublevels(mungefs::open(&buf).ok()?);
    let (_, mut records) = subs.into_iter().find(|(n, _)| *n == fnv1a_lowercase(sub))?;
    mungefs::sound_props::drop_unresolved_layers(&mut records, &core);
    let mut kept: HashMap<u32, SoundProp> = HashMap::new();
    for p in records {
        if kept
            .get(&p.name_hash)
            .is_none_or(|prev| audible(&p) || !audible(prev))
        {
            kept.insert(p.name_hash, p);
        }
    }
    let ids = |v: &[(u32, f32)]| v.iter().map(|(id, _)| *id).collect();
    let audio = |p: &SoundProp| -> Audio {
        let layers = p.layers.iter().map(|l| l.sound).collect();
        (p.stream, ids(&p.segments), ids(&p.samples), layers)
    };
    Some(kept.iter().map(|(n, p)| (*n, audio(p))).collect())
}

/// The census against the game's loader (acceptance 4 and 5): per mission,
/// the names of its sub-level whose audio the library takes from elsewhere
/// (0), and the names it has only by falling through to another sub-level.
#[test]
fn a_battle_plays_its_missions_own_sub_level() {
    let (mut checked, mut names, mut off) = (0, 0, Vec::new());
    println!("mission            sub-level  names  other audio  fall-through names");
    for (file, mission) in missions() {
        let Some(lib) = library(&mission) else {
            continue;
        };
        assert_eq!(mission.sound.len(), 1, "{file}: one sound sub-level");
        let (lvl, sub) = &mission.sound[0];
        let Some(own) = disc_sublevel(&mission.source_game, lvl, sub) else {
            continue;
        };
        let mut wrong = 0;
        for (name, audio) in &own {
            if lib.audio_of(*name).as_ref() != Some(audio) {
                wrong += 1;
                off.push(format!("{file}: {name:08x}"));
            }
        }
        let fall = lib.map_names().filter(|n| !own.contains_key(n)).count();
        println!(
            "{file:<18} {sub:<10} {:>5}  {wrong:>11}  {fall:>18}",
            own.len()
        );
        checked += 1;
        names += own.len();
    }
    if checked == 0 {
        eprintln!(
            "SKIP: no map baked with sub-levels under {}",
            root().display()
        );
        return;
    }
    println!(
        "{checked} missions, {names} names, {} of other audio",
        off.len()
    );
    assert!(
        off.is_empty(),
        "{} names: {:?}",
        off.len(),
        &off[..off.len().min(20)]
    );
}

/// Acceptance 3: each commander line a battle of the mission's own two
/// armies can ask for - the script's bleeding and low-reinforcement lines
/// and its posts' capture and loss lines - is the mission's own sub-level's
/// definition; on BF1, where the two eras' post lines share their names, it
/// never plays the other era's voice stream.
#[test]
fn the_commander_speaks_from_the_missions_sub_level() {
    let (mut checked, mut lines, mut off) = (0, 0, Vec::new());
    for (file, mission) in missions() {
        let dir = root().join(&mission.map);
        let (Some(lib), Ok(txt)) = (
            library(&mission),
            std::fs::read_to_string(dir.join("manifest.ron")),
        ) else {
            continue;
        };
        let manifest: game::assets::Manifest = ron::from_str(&txt).expect("manifest");
        let mut layout = MapLayout::from_manifest(&manifest);
        layout.apply_mission(&mission);
        let factions: Vec<usize> = mission
            .teams
            .iter()
            .filter_map(|t| core::announcer::faction_index(&t.faction))
            .collect();
        let mut asked = std::collections::BTreeSet::new();
        asked.extend(mission.voice.bleeding.iter().map(|b| b.2.clone()));
        asked.extend(mission.voice.low_reinforcements.iter().map(|b| b.2.clone()));
        for c in &layout.cps {
            for (l, s, e, line) in c.vo.iter() {
                let heard = matches!(
                    e,
                    core::announcer::PostEvent::Capture | core::announcer::PostEvent::Lost
                );
                if heard && factions.contains(&l) && factions.contains(&s) {
                    asked.insert(line.to_owned());
                }
            }
        }
        let sub = fnv1a_lowercase(&mission.sound[0].1);
        let other = match mission.era.as_deref() {
            Some("gcw") => "cw_vo",
            _ => "gcw_vo",
        };
        checked += 1;
        for line in asked {
            // A name that is no sound at all (BF1 Yavin's `jungle2`) is
            // `every_mission_line_has_a_take`'s.
            let Some(level) = lib.level_of(&line) else {
                continue;
            };
            lines += 1;
            let stream = lib.audio_of(fnv1a_lowercase(&line)).and_then(|a| a.0);
            let foreign = mission.source_game == "bf1" && stream == Some(fnv1a_lowercase(other));
            if level != sub || foreign {
                off.push(format!("{file}: {line} (sub-level {level:08x})"));
            }
        }
    }
    if checked == 0 {
        eprintln!(
            "SKIP: no map baked with sub-levels under {}",
            root().display()
        );
        return;
    }
    println!(
        "{checked} missions, {lines} commander lines, {} off",
        off.len()
    );
    assert!(off.is_empty(), "{off:#?}");
}

/// The port's era mix (card ERAM) on BF1: an army the mission does not
/// field hears ITS commander at the posts - its own era's voice stream -
/// on every map whose LVL has a sub-level of that era. (The mission's own
/// sub-level names those lines too, as aliases of its own era's takes.)
#[test]
fn an_army_of_the_other_era_keeps_its_own_commander() {
    let (mut checked, mut lines, mut off) = (0, 0, Vec::new());
    for (file, mission) in missions() {
        let Some(mut lib) = library(&mission).filter(|_| mission.source_game == "bf1") else {
            continue;
        };
        let (guests, stream) = match mission.era.as_deref() {
            Some("gcw") => (["rep", "cis"], "cw_vo"),
            _ => (["all", "imp"], "gcw_vo"),
        };
        let home = mission.factions();
        let dir = root().join(&mission.map);
        checked += 1;
        for guest in guests {
            lib.for_battle(&dir, [guest, home[1].as_str()], &mission.sound_levels());
            for role in ["captured", "lost"] {
                let line = format!("{guest}_off_com_report_{role}_commandpost");
                let other = mission.sound[0].1.replace("gcw", "cw");
                let other = if mission.era.as_deref() == Some("gcw") {
                    other
                } else {
                    mission.sound[0].1.replace("cw", "gcw")
                };
                // A single-era planet has no sub-level to take them from.
                if !lib.defines(&other, &line) {
                    continue;
                }
                lines += 1;
                let got = lib.audio_of_name(&line).and_then(|a| a.0);
                if got != Some(fnv1a_lowercase(stream)) {
                    off.push(format!("{file}: {line} stream {got:08x?}"));
                }
            }
        }
    }
    if checked == 0 {
        eprintln!(
            "SKIP: no map baked with sub-levels under {}",
            root().display()
        );
        return;
    }
    println!(
        "{checked} BF1 missions, {lines} guest lines, {} off",
        off.len()
    );
    assert!(lines > 0 && off.is_empty(), "{off:#?}");
}
