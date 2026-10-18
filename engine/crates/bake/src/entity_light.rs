//! BF2's EntityLights (card BPNT, `analysis/re-point-lights.md` §2, [O]): the
//! classes whose BASE is `Light` ([`light_class`]) and the omnis props hang
//! on their model's hardpoints ([`attach`]).
//!
//! Every shipped EntityLight comes from a carrier class's `AttachOdf` +
//! `AttachToHardpoint` (B2 `FUN_001d5260` → `FUN_001d4db8`); none is placed
//! by a `wrld`. The replay is `crate::leafpatch`'s: `AttachOdf` sets ONE
//! current class that carries over to the next `entc`, `AttachEffect` clears
//! it, and each `AttachToHardpoint` the model's skeleton has makes one
//! object at that hardpoint's bind matrix (the leaf patch's per-class object
//! cap is not applied here). A hardpoint the skeleton lacks: [`Miss`]. The light itself (ctor
//! `FUN_00227150`, registration `FUN_00227e48`): an omni of range
//! `OmniRadius` at the instance matrix's origin, registered only while that
//! range is > 0, coloured `Color` bytes / 255, never Overbright; the class's
//! `Static` is the render light's.
//!
//! Not baked: `FlickerType` / `FlickerPeriod` (read, §2 "Flicker", B2
//! `FUN_002278f8`, but not built: the light is steady here; a note on card
//! LFLR), and the flare and cone the same classes draw (card LFLR).

use manifest::{ManifestClass, ManifestClassLight};

use crate::hardpoints::Skeletons;
use crate::leafpatch::{ATTACH_EFFECT, ATTACH_ODF, ATTACH_TO_HARDPOINT};

/// `Color` (FNV of the property name).
pub const COLOR: u32 = 0x3d7e_6258;
/// `OmniRadius`.
pub const OMNI_RADIUS: u32 = 0xc2c1_813a;
/// `Static`.
pub const STATIC: u32 = 0xd290_c23b;

/// An `entc` BASE that makes an EntityLight class.
pub fn is_light(base: &str) -> bool {
    base.eq_ignore_ascii_case("light")
}

/// Whether a light class's property is one [`light_class`] reads.
pub fn is_light_prop(hash: u32) -> bool {
    matches!(hash, COLOR | OMNI_RADIUS | STATIC)
}

/// A light class: its name, colour bytes, `OmniRadius` and `Static`.
#[derive(Debug, Clone, PartialEq)]
pub struct LightClass {
    pub name: String,
    pub color: [u8; 3],
    pub radius: f32,
    pub is_static: bool,
}

/// A `Light` class from its ODF properties (`(fnv hash, value)` in PROP
/// order, a later one overriding an earlier), defaults from the class
/// constructor `FUN_002287e0` (B2 141043–141048: white, `OmniRadius` 10,
/// not Static).
pub fn light_class(name: &str, props: &[(u32, String)]) -> LightClass {
    let mut l = LightClass {
        name: name.to_owned(),
        color: [255; 3],
        radius: 10.0,
        is_static: false,
    };
    for (h, v) in props {
        let f: Vec<f32> = v
            .split_whitespace()
            .filter_map(|t| t.parse().ok())
            .collect();
        match *h {
            COLOR if f.len() >= 3 => {
                l.color = [0, 1, 2].map(|k| f[k].clamp(0.0, 255.0) as u8);
            }
            OMNI_RADIUS => l.radius = f.first().copied().unwrap_or(l.radius).max(0.0),
            STATIC => l.is_static = f.first().is_some_and(|&x| x != 0.0),
            _ => {}
        }
    }
    l
}

/// What [`attach`] does with an `AttachToHardpoint` whose hardpoint the
/// carrier's skeleton lacks (or whose carrier has no skeleton).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Miss {
    /// BF2: the light is made all the same and sits at the carrier's origin
    /// (card B1DK, `analysis/re-kamino-decks.md` §5 `[O]`: B2 `FUN_001d4db8`
    /// hands the factory the carrier's matrix and `FUN_00451e58` finds no
    /// bone; RAM 13 of 13 lights - Kamino's bunkers, POL1's cavern, DEA1's
    /// beam generator).
    Origin,
    /// BF1: a missing bone keeps nothing (§5, §6).
    Skip,
}

impl Miss {
    /// The rule of the game a map is baked from (`"bf1"` / `"bf2"`).
    pub fn of(game: &str) -> Self {
        if game == "bf2" {
            Self::Origin
        } else {
            Self::Skip
        }
    }
}

/// Replay the attach properties of every class (`entcs`: `(name, props)` in
/// the walk's order) and fill [`ManifestClass::lights`] of each class in
/// `classes`: one row per hardpoint whose current class is a light of
/// `OmniRadius` > 0, at the hardpoint's bind position in the model `skels`
/// holds, or by `miss` where the skeleton has no such bone.
pub fn attach(
    entcs: &[(String, Vec<(u32, String)>)],
    lights: &[LightClass],
    skels: &Skeletons,
    miss: Miss,
    classes: &mut [ManifestClass],
) {
    let find = |n: &str| {
        lights
            .iter()
            .find(|l| l.name.eq_ignore_ascii_case(n.trim()))
    };
    let mut current: Option<&LightClass> = None;
    for (name, props) in entcs {
        let class = classes
            .iter_mut()
            .find(|k| k.name.eq_ignore_ascii_case(name));
        let skel = class
            .as_ref()
            .and_then(|k| k.geometry.as_deref())
            .and_then(|g| skels.find(g));
        let binds = skel.and_then(|s| s.bind_matrices().ok());
        let mut out: Vec<ManifestClassLight> = Vec::new();
        for (h, v) in props {
            match *h {
                ATTACH_ODF => current = find(v),
                ATTACH_EFFECT => current = None,
                ATTACH_TO_HARDPOINT => {
                    let Some(cur) = current else {
                        continue;
                    };
                    let bone = match (skel, &binds) {
                        (Some(s), Some(m)) => v
                            .split_whitespace()
                            .next()
                            .and_then(|hp| s.index_of(hp))
                            .map(|i| [m[i][3][0], m[i][3][1], m[i][3][2]]),
                        _ => None,
                    };
                    let pos = match (bone, miss) {
                        (Some(p), _) => p,
                        (None, Miss::Origin) => [0.0; 3],
                        (None, Miss::Skip) => continue,
                    };
                    if cur.radius > 0.0 {
                        out.push(ManifestClassLight {
                            class: cur.name.clone(),
                            pos,
                            color: cur.color,
                            radius: cur.radius,
                            is_static: cur.is_static,
                        });
                    }
                }
                _ => {}
            }
        }
        if let Some(k) = class {
            if !out.is_empty() {
                k.lights = out;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(p: &[(u32, &str)]) -> Vec<(u32, String)> {
        p.iter().map(|&(h, v)| (h, v.to_owned())).collect()
    }

    /// UTA1's three classes as the disc spells them (`Color=90 20 05`).
    #[test]
    fn a_light_class_reads_its_properties() {
        let l = light_class(
            "uta1_post_flare",
            &props(&[(COLOR, "90 20 05"), (OMNI_RADIUS, "1"), (STATIC, "1")]),
        );
        assert_eq!((l.color, l.radius, l.is_static), ([90, 20, 5], 1.0, true));
        let l = light_class(
            "uta1_bldg_tower_flicker",
            &props(&[(COLOR, "230 230 255"), (OMNI_RADIUS, "12.0"), (STATIC, "0")]),
        );
        assert_eq!(
            (l.color, l.radius, l.is_static),
            ([230, 230, 255], 12.0, false)
        );
        let l = light_class("d", &[]);
        assert_eq!((l.color, l.radius, l.is_static), ([255; 3], 10.0, false));
    }

    /// Card B1DK: a carrier whose model lacks the hardpoint (here: no
    /// skeleton at all, and an `AttachOdf` carried over from the class
    /// before) still gets its light on BF2, at its origin; on BF1 it gets
    /// none. A light of `OmniRadius` 0 makes none either way.
    #[test]
    fn a_missing_hardpoint_is_the_origin_on_bf2_and_nothing_on_bf1() {
        let lights = [
            light_class(
                "kamlight",
                &props(&[(COLOR, "181 172 163"), (OMNI_RADIUS, "20")]),
            ),
            light_class("dark", &props(&[(OMNI_RADIUS, "0")])),
        ];
        let entcs = vec![
            (
                "bunker".to_owned(),
                props(&[
                    (ATTACH_ODF, "kamlight"),
                    (ATTACH_TO_HARDPOINT, "hp_light_a"),
                    (ATTACH_TO_HARDPOINT, "hp_light_b"),
                    (ATTACH_ODF, "dark"),
                    (ATTACH_TO_HARDPOINT, "hp_light_c"),
                    (ATTACH_ODF, "kamlight"),
                ]),
            ),
            (
                "pod".to_owned(),
                props(&[
                    (ATTACH_TO_HARDPOINT, "hp_light_a"),
                    (ATTACH_EFFECT, "fx"),
                    (ATTACH_TO_HARDPOINT, "hp_fx"),
                ]),
            ),
        ];
        let class = |n: &str| {
            crate::map::class_from_entc(&mungefs::entity::EntityClass {
                name: n.to_owned(),
                base: "building".to_owned(),
                props: Vec::new(),
            })
        };
        let run = |miss| {
            let mut classes = vec![class("bunker"), class("pod")];
            attach(&entcs, &lights, &Skeletons::of(&[]), miss, &mut classes);
            classes
        };
        let bf2 = run(Miss::of("bf2"));
        assert_eq!(bf2[0].lights.len(), 2, "{:?}", bf2[0].lights);
        assert_eq!(bf2[1].lights.len(), 1, "{:?}", bf2[1].lights);
        for l in bf2.iter().flat_map(|k| &k.lights) {
            assert_eq!(
                (l.pos, l.color, l.radius, l.is_static),
                ([0.0; 3], [181, 172, 163], 20.0, false)
            );
        }
        let bf1 = run(Miss::of("bf1"));
        assert!(bf1.iter().all(|k| k.lights.is_empty()), "{bf1:?}");
    }

    /// Data-gated: UTA1's carriers as the PS2's RAM holds them
    /// (re-point-lights §2): each `uta1_prop_post` one flare omni (its spot
    /// class has `OmniRadius` 0 and makes none), the supporter four flicker
    /// omnis at y 24.7.
    #[test]
    fn the_replay_attaches_utapaus_lights() {
        let lvl = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../extracted/bf2/lvl/DATA/_LVL_PS2/UTA/UTA1.LVL"
        );
        let Ok(buf) = std::fs::read(lvl) else {
            eprintln!("SKIP: {lvl} not present");
            return;
        };
        let inner = mungefs::open(&buf).unwrap();
        let chunks = mungefs::content_chunks(inner).unwrap();
        let (mut classes, mut entcs, mut lights) = (Vec::new(), Vec::new(), Vec::new());
        for c in &chunks {
            if &c.tag != b"entc" {
                continue;
            }
            let e = mungefs::entity::EntityClass::parse(c).unwrap();
            let p: Vec<(u32, String)> = e
                .props
                .iter()
                .map(|p| (p.key_hash, p.value.clone()))
                .collect();
            if is_light(&e.base) {
                lights.push(light_class(&e.name, &p));
            }
            entcs.push((e.name.clone(), p));
            classes.push(crate::map::class_from_entc(&e));
        }
        attach(
            &entcs,
            &lights,
            &Skeletons::of(&chunks),
            Miss::Origin,
            &mut classes,
        );
        let of = |n: &str| {
            &classes
                .iter()
                .find(|k| k.name.eq_ignore_ascii_case(n))
                .unwrap_or_else(|| panic!("no class {n}"))
                .lights
        };
        let post = of("uta1_prop_post");
        assert_eq!(post.len(), 1, "{post:?}");
        assert_eq!((post[0].color, post[0].radius), ([90, 20, 5], 1.0));
        assert!(post[0].is_static);
        let tower = of("uta1_bldg_supporter");
        assert_eq!(tower.len(), 4, "{tower:?}");
        for l in tower {
            assert_eq!(
                (l.color, l.radius, l.is_static),
                ([230, 230, 255], 12.0, false)
            );
        }
    }
}
