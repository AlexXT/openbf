//! Census: two joins over every baked map's bodies.
//!
//! TABLE 1 — how the ENGINE's collision-layer rule
//! ([`game::collision_layers`], A.4) changes the soldier world against the
//! port's old session-7 NAME rule.
//!
//! TABLE 2 — how the SOLDIER and ORDNANCE subsets differ, which is what
//! `game::assets::Scene::collision_world` now builds: it holds the UNION of
//! the two with a per-body mask, so a body in one subset only is a body a
//! soldier walks through and a bolt stops at, or the reverse
//! (analysis/re-collision-layers.md §2, §4). Table 2 counts only the bodies
//! the runtime BUILDS — a `coll` mesh with a glb, and prims of kind
//! sphere/cylinder/box — where table 1 counts every body the manifest holds
//! (on the 32 baked maps that is the same set: every baked prim is of a kind
//! the runtime builds).
//!
//! ```text
//! cargo run --release -p game --example layer_census [assets_dir]
//! ```
//!
//! For every instance of every baked map it builds the model's body list the
//! way `Scene::collision_world` does — the `coll` mesh first, then the `prim`
//! primitives in manifest order — and marks each body twice:
//!
//! * NEW: `layer_bodies(.., Layer::Soldier)`, i.e. the class's
//!   `SoldierCollision` / `VehicleCollisionOnly` lines plus the BF2 `MASK`.
//! * OLD: the mesh always, and a prim unless its NAME contains `vehicle`,
//!   `flyer` or `ordnance` (`game::assets::prim_blocks_soldiers`, deleted by
//!   the same commit — reproduced here only so the two can be compared).
//!
//! Classes and collision data are merged with the map's `common_<game>`
//! bake exactly as `Scene::merge_common` does (vacant-only, so the map's own
//! copy wins). Only the BASE world's instances are counted: mode-layer
//! instances are reported separately, because which sublevel a mission loads
//! is the mission's business.
//!
//! `LAYER_CENSUS_NO_COMMON=1` leaves the common merge out (the join a
//! map-LVL-only census does); `LAYER_CENSUS_LETTER_MASK=1` reads the BF2 mask
//! from the `p_-so-…` NAME letters instead of the `MASK` byte, which is the
//! proxy a join that cannot read the chunk has to use — on the shipped discs
//! the two agree exactly, which is what that switch is for.
//!
//! Counts BODIES THE MANIFEST HOLDS: the bake drops primitives of an
//! undecoded kind (28 over the 32 maps), so a join made against the raw LVLs
//! will count a few more bodies than this does.
//!
//! Reading manifests only — no GLB, no GPU.

use game::assets::{CommonManifest, Manifest, ManifestClass, ManifestCollision};
use game::collision_layers::{layer_bodies, BodyRef, Layer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The PC exporter's `p_-so-…` LETTERS, read as a MASK byte — the proxy a
/// name-only join has to use when it cannot read the `MASK` chunk. Kept for
/// `LAYER_CENSUS_LETTER_MASK=1`, which reproduces such a join for comparison;
/// the real byte (`ManifestPrim::mask`) is what the runtime uses.
fn letter_mask(name: &str) -> u8 {
    let Some(letters) = name
        .to_ascii_lowercase()
        .strip_prefix("p_-")
        .and_then(|r| r.split_once('-').map(|(l, _)| l.to_owned()))
    else {
        return 0;
    };
    let mut m = 0u8;
    for c in letters.bytes() {
        m |= match c {
            b's' => 0x01,
            b'v' => 0x02,
            b'b' => 0x04,
            b't' => 0x08,
            b'o' => 0x10,
            _ => return 0,
        };
    }
    m
}

/// The port's session-7 rule, kept here (and nowhere else) so the census can
/// measure what it cost. PROVISIONAL when it was written; the PS2 engines
/// never parse a primitive's name (analysis/re-collision-layers.md §0.1).
fn old_prim_blocks_soldiers(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    !(n.contains("vehicle") || n.contains("flyer") || n.contains("ordnance"))
}

/// TABLE 2's per-map counts: the soldier subset against the ordnance one,
/// over the bodies the runtime BUILDS.
#[derive(Default, Clone, Copy)]
struct Split {
    /// Bodies the runtime builds: the mesh when the geometry has a glb, plus
    /// every prim of a kind `Scene::collision_world` turns into a shape.
    built: usize,
    soldier: usize,
    ordnance: usize,
    both: usize,
    s_only: usize,
    o_only: usize,
    neither: usize,
    /// Of `s_only` / `o_only`: split by WHY. A BF2 `MASK` byte that is
    /// non-zero and lacks the other layer's bit did it; otherwise the class's
    /// collision properties did.
    s_only_mask: usize,
    o_only_mask: usize,
}

impl Split {
    fn add(&mut self, o: Split) {
        self.built += o.built;
        self.soldier += o.soldier;
        self.ordnance += o.ordnance;
        self.both += o.both;
        self.s_only += o.s_only;
        self.o_only += o.o_only;
        self.neither += o.neither;
        self.s_only_mask += o.s_only_mask;
        self.o_only_mask += o.o_only_mask;
    }
}

/// A prim kind `Scene::collision_world` builds a shape for. Anything else is
/// skipped there and is not a body at runtime.
fn is_built_kind(kind: &str) -> bool {
    matches!(kind, "sphere" | "cylinder" | "box")
}

/// TABLE 2's join for one instance list. `rows` collects the distinct
/// `(class, body, why)` with an instance count.
fn split_census(
    classes: &[ManifestClass],
    collision: &[ManifestCollision],
    instances: &[game::assets::ManifestInstance],
    rows: &mut HashMap<(String, String, &'static str), usize>,
) -> Split {
    let by_geometry: HashMap<String, &ManifestCollision> = collision
        .iter()
        .map(|c| (c.geometry.to_ascii_lowercase(), c))
        .collect();
    let by_class: HashMap<String, &ManifestClass> = classes
        .iter()
        .map(|c| (c.name.to_ascii_lowercase(), c))
        .collect();
    let mut n = Split::default();
    for inst in instances {
        let Some(class) = by_class.get(&inst.class.to_ascii_lowercase()) else {
            continue;
        };
        let Some(geom) = class.geometry.as_deref() else {
            continue;
        };
        let Some(cg) = by_geometry.get(&geom.to_ascii_lowercase()) else {
            continue;
        };
        // The model's body list, exactly as `Scene::collision_world` builds
        // it: the mesh first when there is a glb, then every prim (a prim of
        // an unbuilt kind is still a body the PROPERTIES can name, so it is in
        // the list and only the COUNTS leave it out).
        let mut refs = Vec::with_capacity(cg.prims.len() + 1);
        let mut built = Vec::with_capacity(cg.prims.len() + 1);
        if cg.glb.is_some() {
            refs.push(BodyRef::mesh());
            built.push(true);
        }
        for p in &cg.prims {
            refs.push(BodyRef::prim(&p.name, p.mask));
            built.push(is_built_kind(&p.kind));
        }
        let s = layer_bodies(&refs, &class.collision, Layer::Soldier);
        let o = layer_bodies(&refs, &class.collision, Layer::Ordnance);
        for (i, r) in refs.iter().enumerate() {
            if !built[i] {
                continue;
            }
            n.built += 1;
            if s[i] {
                n.soldier += 1;
            }
            if o[i] {
                n.ordnance += 1;
            }
            let why = match (s[i], o[i]) {
                (true, true) => {
                    n.both += 1;
                    continue;
                }
                (false, false) => {
                    n.neither += 1;
                    continue;
                }
                (true, false) => {
                    n.s_only += 1;
                    // The MASK decided iff the byte is set and lacks the
                    // OTHER layer's bit; else a class property did.
                    if r.mask != 0 && r.mask & Layer::Ordnance.mask_bit() == 0 {
                        n.s_only_mask += 1;
                        "soldier-only (MASK)"
                    } else {
                        "soldier-only (class prop)"
                    }
                }
                (false, true) => {
                    n.o_only += 1;
                    if r.mask != 0 && r.mask & Layer::Soldier.mask_bit() == 0 {
                        n.o_only_mask += 1;
                        "ordnance-only (MASK)"
                    } else {
                        "ordnance-only (class prop)"
                    }
                }
            };
            *rows
                .entry((class.name.clone(), r.name.clone(), why))
                .or_default() += 1;
        }
    }
    n
}

#[derive(Default, Clone, Copy)]
struct Counts {
    bodies: usize,
    new_only: usize,
    old_only: usize,
    both: usize,
    /// Of `old_only`: dropped by a MASK byte this prim AUTHORED, by one it
    /// INHERITED through the BF2 loader's carry-over quirk, and by the
    /// class's own collision properties. The three are disjoint and sum to
    /// `old_only` — the split matters because a name/letter-based join
    /// cannot see the inherited ones.
    by_own_mask: usize,
    by_inherited_mask: usize,
    by_class_prop: usize,
    /// Of `old_only`: how many are the `coll` MESH body rather than a prim.
    /// The RE's join says none on any map (analysis/re-collision-layers.md).
    mesh_drops: usize,
}

impl Counts {
    fn add(&mut self, o: Counts) {
        self.bodies += o.bodies;
        self.new_only += o.new_only;
        self.old_only += o.old_only;
        self.both += o.both;
        self.by_own_mask += o.by_own_mask;
        self.by_inherited_mask += o.by_inherited_mask;
        self.by_class_prop += o.by_class_prop;
        self.mesh_drops += o.mesh_drops;
    }
}

/// One map's join. `examples` collects the first differing bodies for the
/// report.
#[allow(clippy::too_many_arguments)]
fn census(
    classes: &[ManifestClass],
    collision: &[ManifestCollision],
    instances: &[game::assets::ManifestInstance],
    examples: &mut Vec<String>,
    drops: &mut Vec<String>,
    by_class_drops: &mut HashMap<String, usize>,
    map: &str,
    letters: bool,
) -> Counts {
    let by_geometry: HashMap<String, &ManifestCollision> = collision
        .iter()
        .map(|c| (c.geometry.to_ascii_lowercase(), c))
        .collect();
    let by_class: HashMap<String, &ManifestClass> = classes
        .iter()
        .map(|c| (c.name.to_ascii_lowercase(), c))
        .collect();
    let mut n = Counts::default();
    for inst in instances {
        let Some(class) = by_class.get(&inst.class.to_ascii_lowercase()) else {
            continue;
        };
        let Some(geom) = class.geometry.as_deref() else {
            continue;
        };
        let Some(cg) = by_geometry.get(&geom.to_ascii_lowercase()) else {
            continue;
        };
        let mut refs = Vec::with_capacity(cg.prims.len() + 1);
        let mut old = Vec::with_capacity(cg.prims.len() + 1);
        // (mask, authored-here?) per body, for the drop-reason split.
        let mut masks: Vec<(u8, bool)> = Vec::with_capacity(cg.prims.len() + 1);
        if cg.glb.is_some() {
            refs.push(BodyRef::mesh());
            old.push(true);
            masks.push((0, false));
        }
        for p in &cg.prims {
            let mask = if letters {
                letter_mask(&p.name)
            } else {
                p.mask
            };
            refs.push(BodyRef::prim(&p.name, mask));
            old.push(old_prim_blocks_soldiers(&p.name));
            masks.push((mask, p.mask_own));
        }
        let new = layer_bodies(&refs, &class.collision, Layer::Soldier);
        n.bodies += refs.len();
        for (i, r) in refs.iter().enumerate() {
            match (new[i], old[i]) {
                (true, true) => n.both += 1,
                (true, false) => {
                    n.new_only += 1;
                    if examples.len() < 10 {
                        examples.push(format!(
                            "  + {map:10} {:24} {:22} kept by the class rule",
                            inst.class, r.name
                        ));
                    }
                }
                (false, true) => {
                    n.old_only += 1;
                    *by_class_drops.entry(class.name.clone()).or_default() += 1;
                    if drops.len() < 10 {
                        let (mask, own) = masks[i];
                        drops.push(format!(
                            "  - {map:10} {:24} {:22} mask 0x{mask:02x}{}",
                            inst.class,
                            r.name,
                            if mask != 0 && !own {
                                " (inherited)"
                            } else {
                                ""
                            }
                        ));
                    }
                    if r.name == game::collision_layers::MESH_BODY {
                        n.mesh_drops += 1;
                    }
                    let (mask, own) = masks[i];
                    // The seed decides first; only if the MASK left the body
                    // in the layer can a property have removed it.
                    if mask != 0 && mask & Layer::Soldier.mask_bit() == 0 {
                        if own {
                            n.by_own_mask += 1;
                        } else {
                            n.by_inherited_mask += 1;
                        }
                    } else {
                        n.by_class_prop += 1;
                    }
                }
                (false, false) => {}
            }
        }
    }
    n
}

fn common_for(assets: &Path, game_name: &str) -> Option<CommonManifest> {
    let path = assets
        .join(format!("common_{game_name}"))
        .join("common.ron");
    ron::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

fn main() {
    let assets = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(game::assets::baked_root);
    // The runtime always merges common; `LAYER_CENSUS_NO_COMMON=1` leaves it
    // out, which is the join a map-LVL-only census does.
    let no_common = std::env::var("LAYER_CENSUS_NO_COMMON").is_ok();
    let letters = std::env::var("LAYER_CENSUS_LETTER_MASK").is_ok();
    let maps = game::assets::discover_maps(&assets);
    let mut commons: HashMap<String, CommonManifest> = HashMap::new();
    for g in ["bf1", "bf2"] {
        if let Some(c) = common_for(&assets, g) {
            commons.insert(g.to_owned(), c);
        }
    }

    println!("Soldier-world census: the class's SoldierCollision subset (NEW) vs the");
    println!("session-7 prim NAME rule (OLD), over every baked map's BASE world.");
    println!(
        "Bodies = coll mesh + prims per instance, common_<game> merged as the game merges it.\n"
    );
    println!(
        "{:12} {:>9} {:>9} {:>9} {:>9}  {:>9}",
        "map", "bodies", "both", "new only", "old only", "layer_ins"
    );
    let mut total = Counts::default();
    let mut layer_total = Counts::default();
    let mut examples = Vec::new();
    let mut drops = Vec::new();
    let mut by_class_drops: HashMap<String, usize> = HashMap::new();
    let mut rows = 0usize;
    // TABLE 2's accumulators.
    let mut splits: Vec<(String, Split, Split)> = Vec::new();
    let mut split_total = Split::default();
    let mut split_layer_total = Split::default();
    let mut split_rows: HashMap<(String, String, &'static str), usize> = HashMap::new();
    for (key, game_name, dir) in &maps {
        let Ok(text) = std::fs::read_to_string(dir.join("manifest.ron")) else {
            continue;
        };
        let Ok(m) = ron::from_str::<Manifest>(&text) else {
            eprintln!("{key}: manifest did not parse");
            continue;
        };
        // merge_common: vacant-only, the map's own copy always wins.
        let mut classes = m.classes.clone();
        let mut collision = m.collision.clone();
        if let Some(c) = commons.get(game_name).filter(|_| !no_common) {
            for cl in &c.classes {
                if !classes
                    .iter()
                    .any(|x| x.name.eq_ignore_ascii_case(&cl.name))
                {
                    classes.push(cl.clone());
                }
            }
            for co in &c.collision {
                if !collision
                    .iter()
                    .any(|x| x.geometry.eq_ignore_ascii_case(&co.geometry))
                {
                    collision.push(co.clone());
                }
            }
        }
        let n = census(
            &classes,
            &collision,
            &m.instances,
            &mut examples,
            &mut drops,
            &mut by_class_drops,
            key,
            letters,
        );
        let mut layer_instances = Vec::new();
        for l in &m.layers {
            layer_instances.extend(l.instances.iter().cloned());
        }
        let ln = census(
            &classes,
            &collision,
            &layer_instances,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut HashMap::new(),
            key,
            letters,
        );
        let sn = split_census(&classes, &collision, &m.instances, &mut split_rows);
        let sl = split_census(&classes, &collision, &layer_instances, &mut split_rows);
        split_total.add(sn);
        split_layer_total.add(sl);
        splits.push((key.clone(), sn, sl));

        total.add(n);
        layer_total.add(ln);
        rows += 1;
        println!(
            "{key:12} {:9} {:9} {:9} {:9}  {:9}",
            n.bodies, n.both, n.new_only, n.old_only, ln.bodies
        );
    }
    println!(
        "\n{rows} maps: {} instance-bodies; the class rule DROPS {} that the name rule kept, \
         and KEEPS {} that the name rule dropped.",
        total.bodies, total.old_only, total.new_only
    );
    println!(
        "of the {} drops: {} by an authored BF2 MASK, {} by an INHERITED one (the \
         loader's carry-over quirk), {} by the class's collision properties.",
        total.old_only, total.by_own_mask, total.by_inherited_mask, total.by_class_prop
    );
    println!(
        "coll MESH bodies among the drops: {} (common merged: {}, MASK source: {})",
        total.mesh_drops,
        !no_common,
        if letters {
            "name letters"
        } else {
            "the MASK byte"
        }
    );
    println!(
        "mode-layer instances (not counted above): {} bodies, drops {}, keeps {}",
        layer_total.bodies, layer_total.old_only, layer_total.new_only
    );
    let mut top: Vec<(&String, &usize)> = by_class_drops.iter().collect();
    top.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("\nclasses that lose the most instance-bodies:");
    for (name, n) in top.iter().take(15) {
        println!("  {n:6}  {name}");
    }
    if !drops.is_empty() {
        println!("\nfirst bodies the class rule drops and the name rule kept:");
        for d in &drops {
            println!("{d}");
        }
    }
    if !examples.is_empty() {
        println!("\nfirst bodies the class rule keeps and the name rule dropped:");
        for e in &examples {
            println!("{e}");
        }
    }

    // ---- TABLE 2: the SOLDIER subset against the ORDNANCE subset ----
    println!("\n\nTABLE 2 — soldier subset vs ordnance subset, over the bodies the runtime");
    println!("BUILDS (mesh with a glb; prims of kind sphere/cylinder/box). The world is the");
    println!("UNION: `built` minus `neither` is what Scene::collision_world uploads.");
    println!("`base` = the map's base world, `layer` = its mode-layer instances.\n");
    println!(
        "{:12} {:>7} {:>7} {:>8} {:>6} {:>6} {:>7} | {:>6} {:>6} {:>8} {:>6} {:>6} {:>7}",
        "map",
        "built",
        "soldier",
        "ordnance",
        "s-only",
        "o-only",
        "neither",
        "built",
        "soldr",
        "ordnance",
        "s-only",
        "o-only",
        "neither",
    );
    for (key, b, l) in &splits {
        println!(
            "{key:12} {:7} {:7} {:8} {:6} {:6} {:7} | {:6} {:6} {:8} {:6} {:6} {:7}",
            b.built,
            b.soldier,
            b.ordnance,
            b.s_only,
            b.o_only,
            b.neither,
            l.built,
            l.soldier,
            l.ordnance,
            l.s_only,
            l.o_only,
            l.neither,
        );
    }
    let tot = |t: &Split, what: &str| {
        println!(
            "{what}: {} built, soldier {}, ordnance {}, union {}, in both {}, \
             soldier-only {} ({} by a MASK, {} by class properties), \
             ordnance-only {} ({} by a MASK, {} by class properties), in neither {}.",
            t.built,
            t.soldier,
            t.ordnance,
            t.built - t.neither,
            t.both,
            t.s_only,
            t.s_only_mask,
            t.s_only - t.s_only_mask,
            t.o_only,
            t.o_only_mask,
            t.o_only - t.o_only_mask,
            t.neither,
        );
    };
    println!();
    tot(&split_total, &format!("{rows} maps, BASE world"));
    tot(&split_layer_total, &format!("{rows} maps, MODE LAYERS"));

    let mut distinct: Vec<((String, String, &str), usize)> = split_rows.into_iter().collect();
    distinct.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then(a.0 .0.cmp(&b.0 .0))
            .then(a.0 .1.cmp(&b.0 .1))
    });
    println!(
        "\ndistinct (class, body, why) in exactly ONE subset, with instance counts \
         ({} rows):",
        distinct.len()
    );
    for ((class, body, why), n) in &distinct {
        println!("  {n:5}  {class:34} {body:26} {why}");
    }
}
