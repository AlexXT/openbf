//! BF2's HUD layouts: the five `hud_` chunks of INGAME.LVL (`hudtransforms`,
//! `1playerhud` … `4playerhud`; `analysis/re-hud.md` §1.1). Each is a `NAME`
//! (the hash of the layout's name) then a [`crate::config`] DATA/SCOP run —
//! the same tree as a `.hud` source file: element records (`Group`,
//! `Bitmap`, `Text`, `Model3D`, …) whose SCOP holds their properties and
//! child elements. No BF1 LVL carries one (BF1 builds its HUD in code).
//!
//! The records store only FNV-1a hashes ([`crate::fnv1a_lowercase`]) of their
//! names. [`NAMES`] is the table recovered by dictionary and brute force
//! (`re-hud.md` §1.1, `lead_tmp/re/bhud/names.txt`); 11 hashes stay unnamed
//! and [`name`] returns `None` for them.

use crate::config::ConfigNode;
use crate::{Chunk, UcfbError};
use std::collections::HashMap;
use std::sync::OnceLock;

/// The recovered HUD record names, as written in the PC `.hud` sources where
/// the case is known (the hash ignores case).
pub const NAMES: &[&str] = &[
    "ai",
    "aitext",
    "allowminimap",
    "alpha",
    "alwaysscroll",
    "angleend",
    "anglestart",
    "attackeroffscreen",
    "AttackerOffScreenBehind",
    "attackeronscreen",
    "AttackerOnScreenBehind",
    "backdrop",
    "backdroplarge",
    "backdropsmall",
    "backdropspawn",
    "background",
    "barbitmap",
    "barsegmented",
    "Bitmap",
    "bitmaprect",
    "bitmapstyle",
    "blendmode",
    "border",
    "color",
    "colorchange",
    "colorchangerate",
    "colorfriendly",
    "colorpulserate",
    "displaytime",
    "empty",
    "emptytext",
    "eventbitmap",
    "eventcolor",
    "eventdisable",
    "eventenable",
    "eventinput",
    "eventmesh",
    "eventnumber",
    "eventoutput",
    "eventplayerindex",
    "eventposition",
    "eventposthide",
    "eventpulserate",
    "eventrefreshmarker",
    "eventrefreshpost",
    "eventrefreshtarget",
    "eventrotation",
    "eventscale",
    "eventtext",
    "eventvalue",
    "fadeholdtime",
    "fadeintime",
    "fadeouttime",
    "fadesustaintime",
    "fileinfo",
    "flagoffscreen",
    "FlagOffScreenBehind",
    "flagonscreen",
    "FlagOnScreenBehind",
    "flashyscale",
    "floatformat",
    "format",
    "Group",
    "HintFlagOffScreen",
    "HintFlagOffScreenBehind",
    "HintFlagOnScreen",
    "HintFlagOnScreenBehind",
    "hintoffscreen",
    "HintOffScreenBehind",
    "hintonscreen",
    "HintOnScreenBehind",
    "lighting",
    "lockedoffscreen",
    "LockedOffScreenBehind",
    "lockedonscreen",
    "LockedOnScreenBehind",
    "Map",
    "markerlarge",
    "markersmall",
    "markerspawn",
    "mask",
    "maxscale",
    "mesh",
    "meshinfo",
    "minscale",
    "model3d",
    "multilinetext",
    "namemesh",
    "northlarge",
    "northsmall",
    "northspawn",
    "numbercolor",
    "numbertotime",
    "numlines",
    "numsegments",
    "objectivelist",
    "objectiveoffscreen",
    "ObjectiveOffScreenBehind",
    "objectiveonscreen",
    "ObjectiveOnScreenBehind",
    "player",
    "PlayerDirectionLarge",
    "PlayerDirectionSmall",
    "PlayerDirectionSpawn",
    "PlayerFOVLarge",
    "PlayerFOVSmall",
    "PlayerFOVSpawn",
    "playertext",
    "Position",
    "positionlarge",
    "positionsmall",
    "positionspawn",
    "postlarge",
    "PostSelectLarge",
    "PostSelectSmall",
    "PostSelectSpawn",
    "postsmall",
    "postspawn",
    "PostTextLarge",
    "PostTextSmall",
    "PostTextSpawn",
    "propagatealpha",
    "Rect",
    "rotation",
    "Scale",
    "scaletexture",
    "segment",
    "self",
    "selftext",
    "Target",
    "targetlarge",
    "targetsmall",
    "targetspawn",
    "texcoords",
    "Text",
    "textalignment",
    "textbox",
    "textbreak",
    "textfont",
    "textscale",
    "textstyle",
    "transformnamemesh",
    "transformnumbercolor",
    "usechangecolor",
    "value",
    "vehicleseating",
    "Viewport",
    "viewports",
    "zorder",
    "TransformNumberVector3",
    "NumberVector3",
    "Circular",
    "BorderedBox",
    "TextCharacterSpacing",
];

/// `FileInfo("<layout name>")` — the first record of every `hud_` chunk.
pub const FILE_INFO: u32 = 0xd4e0_c797;

/// The name behind a record hash, if [`NAMES`] has it.
pub fn name(hash: u32) -> Option<&'static str> {
    static TABLE: OnceLock<HashMap<u32, &'static str>> = OnceLock::new();
    TABLE
        .get_or_init(|| {
            NAMES
                .iter()
                .map(|n| (crate::fnv1a_lowercase(n), *n))
                .collect()
        })
        .get(&hash)
        .copied()
}

/// One `hud_` chunk.
#[derive(Debug, Clone)]
pub struct HudLayout {
    /// The chunk's `NAME` payload: the hash of the layout's name.
    pub name_hash: u32,
    /// The top-level records, `FileInfo` first.
    pub nodes: Vec<ConfigNode>,
}

impl HudLayout {
    /// The layout's name, from its `FileInfo` record.
    pub fn file_name(&self) -> Option<&str> {
        self.nodes.iter().find(|n| n.hash == FILE_INFO)?.str_arg(0)
    }

    /// Every record in the tree, depth first.
    pub fn record_count(&self) -> usize {
        fn count(ns: &[ConfigNode]) -> usize {
            ns.iter().map(|n| 1 + count(&n.children)).sum()
        }
        count(&self.nodes)
    }
}

/// Parse a `hud_` chunk.
pub fn parse(chunk: &Chunk<'_>) -> Result<HudLayout, UcfbError> {
    let name = chunk
        .child(b"NAME")
        .ok_or(UcfbError::Truncated(chunk.offset))?;
    if name.payload.len() < 4 {
        return Err(UcfbError::Truncated(name.offset));
    }
    let name_hash = crate::le::u32_at(name.payload, 0);
    let nodes = crate::config::parse_tree(chunk)?;
    Ok(HudLayout { name_hash, nodes })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ingame() -> Option<Vec<u8>> {
        let p = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../extracted/bf2/lvl/DATA/_LVL_PS2/INGAME.LVL"
        );
        std::fs::read(p).ok()
    }

    #[test]
    fn the_table_holds_no_two_names_with_one_hash() {
        let mut seen = HashMap::new();
        for n in NAMES {
            if let Some(o) = seen.insert(crate::fnv1a_lowercase(n), n) {
                panic!("{n} and {o} share a hash");
            }
        }
        assert_eq!(name(FILE_INFO), Some("fileinfo"));
        assert_eq!(name(0x4654_4626), Some("Bitmap"));
    }

    /// Data-gated: the five layouts, their record and top-level element
    /// counts, and the 11 unnamed hashes (`re-hud.md` §1.1, §7).
    #[test]
    fn ingame_carries_the_five_layouts() {
        let Some(buf) = ingame() else {
            eprintln!("SKIP: extracted BF2 INGAME.LVL not present");
            return;
        };
        let inner = crate::open(&buf).unwrap();
        let layouts: Vec<HudLayout> = crate::content_chunks(inner)
            .unwrap()
            .iter()
            .filter(|c| &c.tag == b"hud_")
            .map(|c| parse(c).unwrap())
            .collect();
        let got: Vec<(&str, usize, usize)> = layouts
            .iter()
            .map(|l| {
                let elements = l.nodes.iter().filter(|n| n.hash != FILE_INFO).count();
                (l.file_name().unwrap(), l.record_count(), elements)
            })
            .collect();
        assert_eq!(
            got,
            [
                ("hudtransforms", 125, 1),
                ("1playerhud", 2343, 42),
                ("2playerhud", 4722, 4),
                ("3playerhud", 9, 1),
                ("4playerhud", 5271, 6),
            ]
        );
        let mut unnamed = std::collections::BTreeSet::new();
        fn walk(ns: &[ConfigNode], out: &mut std::collections::BTreeSet<u32>) {
            for n in ns {
                if name(n.hash).is_none() {
                    out.insert(n.hash);
                }
                walk(&n.children, out);
            }
        }
        for l in &layouts {
            walk(&l.nodes, &mut unnamed);
        }
        let want: std::collections::BTreeSet<u32> = [
            0x2b76_908f,
            0x2deb_a703,
            0x7a4e_6e71,
            0x233d_39ec,
            0x0d53_3bd4,
            0x92c1_cee6,
            0x9d34_215e,
            0x6703_febc,
            0x86e6_ef58,
            0x0977_1c74,
            0x4ee6_8dbe,
        ]
        .into();
        assert_eq!(unnamed, want);
    }
}
