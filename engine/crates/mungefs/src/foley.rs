//! `ffx_` foley tables (card FOLY, `analysis/re-foley.md` §1, `[O]`): the
//! soldier sound rows of a SOUND/<PLANET>.LVL, per `lvl_` sub-level
//! `<map><era>` (`nab1cw`, `uta1gcw`).
//!
//! An `ffx_` chunk is a config tree (`crate::config`): a `NAME`, then
//! top-level records, each with a `SCOP` of children. Every value is a
//! name HASH (no string blob). Two record kinds ship on the discs:
//!
//! ```text
//! FoleyFXSoldier { name(h)  Class(h)  FootstepLeftWalk(h) ... BodyFall(h) }
//! FoleyFXGroup   { name(h)  FoleyFX(h soldier record name) x N }
//! ```
//!
//! A group member whose record the sub-level does not define is skipped by
//! the PS2's loader (B1 355500-355503) and is dropped here.

use crate::config::{parse_tree, ConfigNode};
use crate::{chunks, fnv1a_lowercase, Chunk};

/// A `FoleyFXSoldier` record: one soldier family's sounds on one surface.
/// A slot the record does not set is 0 (BF2's records carry no `Prone` and
/// no `Stand`). The two `Footstep*Stop` slots are read by nothing on the
/// PS2 (§4.5) and are not kept.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoleySoldier {
    pub name: u32,
    /// The hash of a soldier class's `FoleyFXClass` value.
    pub class: u32,
    pub walk_left: u32,
    pub walk_right: u32,
    pub run_left: u32,
    pub run_right: u32,
    pub jump: u32,
    pub land: u32,
    pub roll: u32,
    pub prone: u32,
    pub squat: u32,
    pub stand: u32,
    pub body_fall: u32,
}

/// A `FoleyFXGroup`: a surface (`terrain_foley`, `metal_foley`, ...) and the
/// records it lists that the sub-level defines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoleyGroup {
    pub name: u32,
    pub rows: Vec<FoleySoldier>,
}

/// One `lvl_` sub-level's foley.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoleyLevel {
    /// The sub-level's name hash (`nab1cw`).
    pub name: u32,
    pub groups: Vec<FoleyGroup>,
    /// Soldier records defined, for the census (some are in no group).
    pub records: usize,
}

fn first(n: &ConfigNode) -> u32 {
    n.values.first().copied().unwrap_or(0)
}

fn soldier(n: &ConfigNode) -> FoleySoldier {
    let h = fnv1a_lowercase;
    let mut s = FoleySoldier::default();
    for c in &n.children {
        let v = first(c);
        let slot = match c.hash {
            k if k == h("name") => &mut s.name,
            k if k == h("Class") => &mut s.class,
            k if k == h("FootstepLeftWalk") => &mut s.walk_left,
            k if k == h("FootstepRightWalk") => &mut s.walk_right,
            k if k == h("FootstepLeftRun") => &mut s.run_left,
            k if k == h("FootstepRightRun") => &mut s.run_right,
            k if k == h("Jump") => &mut s.jump,
            k if k == h("Land") => &mut s.land,
            k if k == h("Roll") => &mut s.roll,
            k if k == h("Prone") => &mut s.prone,
            k if k == h("Squat") => &mut s.squat,
            k if k == h("Stand") => &mut s.stand,
            k if k == h("BodyFall") => &mut s.body_fall,
            _ => continue,
        };
        *slot = v;
    }
    s
}

/// The records of one sub-level's `ffx_` chunks, groups resolved.
fn level(name: u32, trees: &[Vec<ConfigNode>]) -> FoleyLevel {
    let h = fnv1a_lowercase;
    let nodes = || trees.iter().flatten();
    let records: Vec<FoleySoldier> = nodes()
        .filter(|n| n.hash == h("FoleyFXSoldier"))
        .map(soldier)
        .collect();
    let groups = nodes()
        .filter(|n| n.hash == h("FoleyFXGroup"))
        .map(|g| FoleyGroup {
            name: g.find(h("name")).map_or(0, first),
            rows: g
                .find_all(h("FoleyFX"))
                .filter_map(|m| records.iter().find(|r| r.name == first(m)).cloned())
                .collect(),
        })
        .collect();
    FoleyLevel {
        name,
        groups,
        records: records.len(),
    }
}

/// Every sub-level of a SOUND LVL payload that carries `ffx_` chunks, in
/// file order. A `lvl_`'s payload is `[name hash][size]` + its chunks, which
/// the chunk walker reads as one child tagged with the name hash.
pub fn find_all(inner: &[u8]) -> Vec<FoleyLevel> {
    let mut out = Vec::new();
    for top in chunks(inner).flatten() {
        if &top.tag != b"lvl_" {
            continue;
        }
        for sub in top.children().flatten() {
            let mut trees = Vec::new();
            crate::soundbank::walk_sound_wrappers(sub.payload, 3, &mut |c: &Chunk<'_>| {
                if &c.tag != b"ffx_" {
                    return false;
                }
                if let Ok(t) = parse_tree(c) {
                    trees.push(t);
                }
                true
            });
            if !trees.is_empty() {
                out.push(level(sub.tag_u32(), &trees));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_chunk;

    fn data(key: &str, vals: &[u32]) -> Vec<u8> {
        let mut p = fnv1a_lowercase(key).to_le_bytes().to_vec();
        p.push(vals.len() as u8);
        for v in vals {
            p.extend(v.to_le_bytes());
        }
        build_chunk(b"DATA", &p)
    }

    fn record(kind: &str, kids: &[(&str, u32)]) -> Vec<u8> {
        let mut v = data(kind, &[]);
        let scop: Vec<u8> = kids.iter().flat_map(|(k, x)| data(k, &[*x])).collect();
        v.extend(build_chunk(b"SCOP", &scop));
        v
    }

    /// A group keeps the members its sub-level defines, in the group's
    /// order, and drops the rest; the Stop slots are not kept.
    #[test]
    fn a_group_lists_only_the_records_its_sub_level_defines() {
        let mut ffx = record(
            "FoleyFXSoldier",
            &[
                ("name", 11),
                ("Class", 100),
                ("FootstepLeftRun", 5),
                ("FootstepLeftStop", 9),
                ("Land", 6),
            ],
        );
        ffx.extend(record("FoleyFXSoldier", &[("name", 12), ("Class", 200)]));
        ffx.extend(record(
            "FoleyFXGroup",
            &[
                ("name", 77),
                ("FoleyFX", 12),
                ("FoleyFX", 99),
                ("FoleyFX", 11),
            ],
        ));
        let sub = build_chunk(&0xAABB_CCDDu32.to_le_bytes(), &build_chunk(b"ffx_", &ffx));
        let buf = build_chunk(b"lvl_", &sub);
        let levels = find_all(&buf);
        assert_eq!(levels.len(), 1);
        let l = &levels[0];
        assert_eq!((l.name, l.records, l.groups.len()), (0xAABB_CCDD, 2, 1));
        let g = &l.groups[0];
        assert_eq!(g.name, 77);
        let classes: Vec<u32> = g.rows.iter().map(|r| r.class).collect();
        assert_eq!(classes, vec![200, 100]);
        assert_eq!(
            g.rows[1],
            FoleySoldier {
                name: 11,
                class: 100,
                run_left: 5,
                land: 6,
                ..FoleySoldier::default()
            }
        );
    }

    /// The PS2's RAM (re-foley §8.3), from the discs: BF1 nab1cw holds 4
    /// groups x 3 rows, BF2 uta1cw 3 x 5. Data-gated.
    #[test]
    fn nab1cw_and_uta1cw_match_the_ps2s_ram() {
        let e = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../extracted");
        let h = fnv1a_lowercase;
        for (lvl, sub, groups, classes, run_left) in [
            (
                "bf1/sound/DATA/_LVL_PS2/SOUND/NAB.LVL",
                "nab1cw",
                vec!["terrain_foley", "grass_foley", "stone_foley", "metal_foley"],
                vec!["cis_inf_droid", "rep_inf_trooper", "wok_inf_soldier"],
                "cis_inf_droid_run_grass_left",
            ),
            (
                "bf2/sound/DATA/_LVL_PS2/SOUND/UTA.LVL",
                "uta1cw",
                vec!["terrain_foley", "stone_foley", "metal_foley"],
                vec![
                    "cis_inf_droid",
                    "cis_inf_sbdroid",
                    "rep_inf_trooper",
                    "cis_inf_soldier",
                    "rep_inf_soldier",
                ],
                "cis_inf_droid_run_stone_left",
            ),
        ] {
            let Ok(buf) = std::fs::read(format!("{e}/{lvl}")) else {
                eprintln!("SKIP: {lvl} not present");
                continue;
            };
            let levels = find_all(crate::open(&buf).unwrap());
            let l = levels.iter().find(|l| l.name == h(sub)).expect(sub);
            let mut got: Vec<u32> = l.groups.iter().map(|g| g.name).collect();
            let mut want: Vec<u32> = groups.iter().map(|g| h(g)).collect();
            got.sort_unstable();
            want.sort_unstable();
            assert_eq!(got, want, "{sub} groups");
            for g in &l.groups {
                let mut got: Vec<u32> = g.rows.iter().map(|r| r.class).collect();
                let mut want: Vec<u32> = classes.iter().map(|c| h(c)).collect();
                got.sort_unstable();
                want.sort_unstable();
                assert_eq!(got, want, "{sub} group {:08x}", g.name);
            }
            let terrain = l
                .groups
                .iter()
                .find(|g| g.name == h("terrain_foley"))
                .unwrap();
            let droid = terrain
                .rows
                .iter()
                .find(|r| r.class == h("cis_inf_droid"))
                .unwrap();
            assert_eq!(droid.run_left, h(run_left), "{sub}");
            // BF1's rows carry Prone and Stand, BF2's neither (§1.1).
            let bf1 = lvl.starts_with("bf1");
            assert_eq!((droid.prone != 0, droid.stand != 0), (bf1, bf1), "{sub}");
            assert!(droid.jump != 0 && droid.land != 0 && droid.body_fall != 0);
        }
    }
}
