//! Recover readable names for the FNV-hashed sound properties in `sounds.ron`.
//!
//! The SOUND LVLs store every property and sample id as an `fnv1a_lowercase`
//! hash and keep no strings (`strings -n 8` on SOUND/HOT.LVL is pure ADPCM
//! noise), so a name can only come back by hashing a candidate and matching.
//! A match is a **confirmed preimage**, not a guess — but with a large
//! candidate set some matches are collisions, so every entry records where its
//! candidate came from:
//!
//! * `Direct`  — the string appears verbatim on the discs: a class name or a
//!   string-valued class property in the baked `classes/*.ron` (ODF writes the
//!   sound name as the property VALUE, e.g. `FireSound =
//!   "all_weap_inf_rifle_fire"`), or a printable run inside an extracted
//!   `.luac`. These are the names the game itself looks up.
//! * `Derived` — built by recombining observed parts (swap the side prefix,
//!   append an observed suffix). Useful for the per-side variants the discs
//!   only spell out for one side, but a `Derived` hit has a real chance of
//!   being a hash collision rather than the true name.
//!
//! Collision budget: with `C` candidates and `T` target hashes the expected
//! number of false matches is `C * T / 2^32`. `write_names` reports both
//! counts so the number is checkable rather than assumed.
//!
//! Output: `<mapdir>/sounds/names.ron`. Nothing in the runtime depends on it;
//! it exists so `scripts/audition.sh` can lay the samples out under names a
//! human can read.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// Where a recovered name's candidate string came from.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameSource {
    /// Verbatim on the discs (baked class strings or extracted Lua).
    Direct,
    /// Recombined from observed parts — may be a hash collision.
    Derived,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NamesRon {
    /// Candidate strings hashed while building this file.
    pub candidates: usize,
    /// Hashes present in the map's `sounds.ron` that a name was found for.
    pub named: usize,
    /// Hashes present in the map's `sounds.ron` in total.
    pub total: usize,
    /// hash -> (name, where the candidate came from)
    pub names: Vec<(u32, String, NameSource)>,
}

/// Side/faction prefixes the discs use on sound and class names. Swapping
/// these over an observed stem is what recovers the per-side variants.
const SIDES: &[&str] = &[
    "all", "imp", "rep", "cis", "com", "des", "tur", "gam", "amb", "shell",
];

/// Suffixes observed on real sound-property names (mined from the `Direct`
/// hits on the shipped banks).
const SUFFIXES: &[&str] = &[
    "_fire",
    "_reload",
    "_hit",
    "_exp",
    "_explode",
    "_idle",
    "_move",
    "_die",
    "_hurt",
    "_shot",
    "_ricochet",
    "_lp",
    "_start",
    "_stop",
    "_end",
    "_distant",
    "_bounce",
    "_empty",
    "_in",
    "_out",
    "_open",
    "_close",
    "_loop",
    "_impact",
];

fn is_nameish(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 96
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Every string-valued token in a baked `classes/*.ron`: the class `name:`
/// and every string property value.
fn strings_from_classes(dir: &Path, out: &mut HashSet<String>) -> Result<()> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Ok(());
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("ron") {
            continue;
        }
        let txt = std::fs::read_to_string(&p).with_context(|| p.display().to_string())?;
        // Both shapes are `"..."`-quoted; scan quoted runs and keep the
        // name-shaped ones. Cheaper and more robust than parsing the RON.
        let mut rest = txt.as_str();
        while let Some(i) = rest.find('"') {
            rest = &rest[i + 1..];
            let Some(j) = rest.find('"') else { break };
            let s = &rest[..j];
            if is_nameish(s) {
                out.insert(s.to_string());
            }
            rest = &rest[j + 1..];
        }
    }
    Ok(())
}

/// Printable runs inside the extracted Lua bytecode — mission and common
/// scripts name sounds as string constants.
fn strings_from_lua(root: &Path, out: &mut HashSet<String>) {
    fn walk(dir: &Path, out: &mut HashSet<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("luac") {
                let Ok(buf) = std::fs::read(&p) else { continue };
                let mut cur = String::new();
                for &b in &buf {
                    if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                        cur.push(b as char);
                    } else {
                        if cur.len() >= 4 && is_nameish(&cur) {
                            out.insert(std::mem::take(&mut cur));
                        }
                        cur.clear();
                    }
                }
                if cur.len() >= 4 {
                    out.insert(cur);
                }
            }
        }
    }
    walk(root, out);
}

/// Build `hash -> (name, source)`. `Direct` strings win over `Derived` ones
/// for the same hash, so a real disc string is never displaced by a
/// recombination that happens to collide with it.
pub fn build_dictionary(
    classes_dir: &Path,
    lua_roots: &[&Path],
) -> Result<HashMap<u32, (String, NameSource)>> {
    let mut direct: HashSet<String> = HashSet::new();
    strings_from_classes(classes_dir, &mut direct)?;
    for r in lua_roots {
        strings_from_lua(r, &mut direct);
    }

    // Stems = observed names with a leading side token removed.
    let mut stems: HashSet<&str> = HashSet::new();
    for w in &direct {
        if let Some((head, tail)) = w.split_once('_') {
            if SIDES.contains(&head.to_ascii_lowercase().as_str()) && !tail.is_empty() {
                stems.insert(tail);
            }
        }
    }

    let mut map: HashMap<u32, (String, NameSource)> = HashMap::new();

    // Derived first, so the Direct pass can overwrite any collision.
    for side in SIDES {
        for stem in &stems {
            insert_candidate(&mut map, format!("{side}_{stem}"), NameSource::Derived);
        }
    }
    let derived_base: Vec<String> = map.values().map(|(s, _)| s.clone()).collect();
    for w in derived_base.iter().chain(direct.iter()) {
        for sfx in SUFFIXES {
            insert_candidate(&mut map, format!("{w}{sfx}"), NameSource::Derived);
        }
    }
    for w in &direct {
        insert_candidate(&mut map, w.clone(), NameSource::Direct);
    }
    Ok(map)
}

/// Offer one spelling to the dictionary. Two spellings that differ only in
/// case hash the SAME (the hash lowercases), so several candidates compete for
/// one slot and the winner must not depend on which one arrived first: the
/// candidates come out of `HashSet`s and a `HashMap`, whose order changes with
/// every process (bug M-B3 — all 32 `sounds/names.ron` differed between two
/// runs of one binary). The rule is a minimum over a total order, so the
/// result is the same whatever the order: a `Direct` spelling always beats a
/// `Derived` one, and among equals the lexicographically smallest wins.
fn insert_candidate(map: &mut HashMap<u32, (String, NameSource)>, s: String, src: NameSource) {
    let h = mungefs::fnv1a_lowercase(&s);
    let better = match map.get(&h) {
        None => true,
        Some((old, old_src)) => match (src, *old_src) {
            (NameSource::Direct, NameSource::Derived) => true,
            (NameSource::Derived, NameSource::Direct) => false,
            _ => s < *old,
        },
    };
    if better {
        map.insert(h, (s, src));
    }
}

/// Write `<mapdir>/sounds/names.ron` for one baked map. Returns (named, total).
pub fn write_names(
    mapdir: &str,
    dict: &HashMap<u32, (String, NameSource)>,
) -> Result<(usize, usize)> {
    let sounds = Path::new(mapdir).join("sounds");
    let txt = std::fs::read_to_string(sounds.join("sounds.ron"))
        .with_context(|| format!("{mapdir}: no sounds.ron (bake sounds first)"))?;
    let ron: crate::sounds::SoundsRon = ron::from_str(&txt).context("sounds.ron")?;

    // Every hash a human might want a name for: properties and sample ids.
    let mut want: HashSet<u32> = HashSet::new();
    want.extend(ron.props.iter().map(|p| p.name_hash));
    want.extend(ron.samples.iter().map(|(h, _)| *h));
    want.extend(ron.banks.iter().map(|(h, _)| *h));

    let mut found: BTreeMap<u32, (String, NameSource)> = BTreeMap::new();
    for h in &want {
        if let Some(v) = dict.get(h) {
            found.insert(*h, v.clone());
        }
    }
    let out = NamesRon {
        candidates: dict.len(),
        named: found.len(),
        total: want.len(),
        names: found.into_iter().map(|(h, (n, s))| (h, n, s)).collect(),
    };
    crate::write_ron(&sounds.join("names.ron"), &out)?;
    Ok((out.named, out.total))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hash must agree with the engine's, or every name is wrong: the
    /// literal is the sample id the BF1 TAT bank stores for the rifle's
    /// FireSound (a confirmed preimage — see the bake::sounds test), so a
    /// change to the FNV basis or prime fails here rather than silently
    /// renaming every sound.
    #[test]
    fn known_sound_name_hashes() {
        assert_eq!(
            mungefs::fnv1a_lowercase("all_weap_inf_rifle_fire"),
            0x2729_7082
        );
        assert_eq!(
            mungefs::fnv1a_lowercase("ALL_WEAP_INF_RIFLE_FIRE"),
            0x2729_7082
        );
    }

    /// M-B3: the dictionary must not depend on the order the candidates
    /// arrive in. Every permutation of a set of equal-hash spellings has to
    /// end at the same entry — the smallest `Direct` one here.
    #[test]
    fn the_winner_does_not_depend_on_the_order() {
        // These four differ only in case, so they share one hash slot.
        let spellings = [
            ("all_weap_inf_rifle_fire", NameSource::Direct),
            ("ALL_WEAP_INF_RIFLE_FIRE", NameSource::Direct),
            ("All_Weap_Inf_Rifle_Fire", NameSource::Derived),
            ("all_weap_INF_rifle_fire", NameSource::Derived),
        ];
        let h = mungefs::fnv1a_lowercase(spellings[0].0);
        let mut seen: Option<(String, NameSource)> = None;
        for order in permutations(spellings.len()) {
            let mut m: HashMap<u32, (String, NameSource)> = HashMap::new();
            for i in order {
                insert_candidate(&mut m, spellings[i].0.to_string(), spellings[i].1);
            }
            assert_eq!(m.len(), 1, "the four spellings share one hash");
            let got = m[&h].clone();
            match &seen {
                None => seen = Some(got),
                Some(first) => assert_eq!(&got, first, "order changed the winner"),
            }
        }
        // Direct wins over Derived, and among the Direct pair the
        // lexicographically smallest spelling is the upper-case one.
        assert_eq!(
            seen.unwrap(),
            ("ALL_WEAP_INF_RIFLE_FIRE".to_string(), NameSource::Direct)
        );
    }

    /// All permutations of `0..n`, smallest first (n is 4 here).
    fn permutations(n: usize) -> Vec<Vec<usize>> {
        if n == 0 {
            return vec![vec![]];
        }
        let mut out = Vec::new();
        for tail in permutations(n - 1) {
            for pos in 0..n {
                let mut v = tail.clone();
                v.insert(pos, n - 1);
                out.push(v);
            }
        }
        out
    }

    /// Data-gated: over the shipped discs the dictionary names a useful slice
    /// of TAT1's sound properties, and the gameplay names resolve Direct.
    #[test]
    fn names_bf1_tat_properties() {
        let classes = format!("{}/classes", crate::BAKED_ROOT);
        let lua = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../extracted/bf1/lua");
        let mapdir = format!("{}/tat1", crate::BAKED_ROOT);
        if !Path::new(&classes).exists() || !Path::new(&mapdir).join("sounds").exists() {
            eprintln!("SKIP: baked assets not present");
            return;
        }
        let dict = build_dictionary(Path::new(&classes), &[Path::new(lua)]).unwrap();
        // Bug M-B2: `write_names` writes `<mapdir>/sounds/names.ron`, so
        // running it on the SHIPPED tree rewrote `assets_baked/tat1/sounds/
        // names.ron` on every `./ci.sh` — with a BF1-only dictionary, unlike
        // `rebake.sh`, which passes both discs' lua roots. The test copies the
        // one file `write_names` reads into a per-process scratch map dir and
        // writes there; the baked tree is `rebake.sh`'s alone.
        let scratch =
            std::env::temp_dir().join(format!("openbf-sound-names-tat1-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(scratch.join("sounds")).unwrap();
        std::fs::copy(
            Path::new(&mapdir).join("sounds/sounds.ron"),
            scratch.join("sounds/sounds.ron"),
        )
        .unwrap();
        let (named, total) = write_names(scratch.to_str().unwrap(), &dict).unwrap();
        assert!(named > 200, "only {named}/{total} named");
        assert!(
            scratch.join("sounds/names.ron").is_file(),
            "names.ron lands in the scratch dir, not in the baked tree"
        );
        // The weapon sound the runtime plays must come back Direct.
        let h = mungefs::fnv1a_lowercase("all_weap_inf_rifle_fire");
        assert_eq!(
            dict.get(&h).map(|(n, s)| (n.as_str(), *s)),
            Some(("all_weap_inf_rifle_fire", NameSource::Direct))
        );
        let _ = std::fs::remove_dir_all(&scratch);
    }
}
