//! BF2's `sanm` chunk (`SoldierAnimation`): the per-animation blend times,
//! loop flag, aim type and sync group of the soldier animator (card ANIMS,
//! `analysis/re-soldier-anim-layers.md` §6.4 `[O]`). One chunk on the disc, a
//! top-level chunk of `INGAME.LVL`: a `NAME` hash and a config tree of 192
//! `Animation("name") { ... }` scopes. A name is an animation of BF2's set
//! (`stand_runforward`, `jump`, `stand_shoot`) with an optional `_upper` or
//! `_lower` for one body layer; a bare name configures both. BF1 has no such
//! chunk: its times are code constants (§2.1).
//!
//! The PS2 stores the two times as bytes of `(int)(seconds * 100.0f)` in
//! single precision, chopped: 0.02 is kept as 1 and 0.01 as 0
//! ([`stored_hundredths`]; RAM holds the file's values in 258 of 258 records).

use crate::config::{parse_tree, ConfigNode};
use crate::{chunks, fnv1a_lowercase, Chunk, UcfbError};

/// What the animation's upper-body player aims (`AimType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AimType {
    #[default]
    None,
    Head,
    Torso,
    FullBody,
}

/// The body layer an `Animation` scope names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Upper,
    Lower,
}

/// One `Animation("name") { ... }` scope.
#[derive(Debug, Clone, PartialEq)]
pub struct SoldierAnimation {
    /// As written, suffix included (`stand_shoot_upper`).
    pub name: String,
    pub looping: bool,
    pub aim: AimType,
    /// `SyncType`: animations of one group keep their phase across a change.
    pub sync: Option<String>,
    /// `BlendInTime` / `BlendOutTime`, seconds as the file has them.
    pub blend_in: f32,
    pub blend_out: f32,
    /// `BlendTimeTo(other, seconds)`: the blend into that animation instead
    /// of `blend_out`.
    pub blend_to: Vec<(String, f32)>,
}

impl SoldierAnimation {
    /// The animation without its layer suffix, and the layer it names (`None`
    /// = both).
    pub fn base(&self) -> (&str, Option<Layer>) {
        if let Some(b) = self.name.strip_suffix("_upper") {
            (b, Some(Layer::Upper))
        } else if let Some(b) = self.name.strip_suffix("_lower") {
            (b, Some(Layer::Lower))
        } else {
            (&self.name, None)
        }
    }
}

/// The byte the PS2 keeps for a blend time: `seconds * 100.0f` in single
/// precision, chopped toward zero (0.02 -> 1, 0.01 -> 0, 0.15 -> 15). The
/// EE's FPU rounds the product toward zero where the host rounds to nearest
/// (0.02f * 100 is 1.99999996: 2.0 here, 1.9999999 there), so the product is
/// taken exactly and stepped down.
pub fn stored_hundredths(seconds: f32) -> u8 {
    let exact = f64::from(seconds) * 100.0;
    let mut r = exact as f32;
    if f64::from(r) > exact {
        r = f32::from_bits(r.to_bits() - 1);
    }
    r as u8
}

/// The whole chunk.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SoldierAnimations {
    pub animations: Vec<SoldierAnimation>,
}

impl SoldierAnimations {
    /// The scope that configures `layer` of animation `base`: the one named
    /// for the layer, else the bare name.
    pub fn get(&self, base: &str, layer: Layer) -> Option<&SoldierAnimation> {
        let suffixed = match layer {
            Layer::Upper => format!("{base}_upper"),
            Layer::Lower => format!("{base}_lower"),
        };
        self.animations
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(&suffixed))
            .or_else(|| {
                self.animations
                    .iter()
                    .find(|a| a.name.eq_ignore_ascii_case(base))
            })
    }
}

fn animation(n: &ConfigNode) -> Option<SoldierAnimation> {
    let h = fnv1a_lowercase;
    let name = n.str_arg(0)?.to_string();
    let f = |key: &str| n.find(h(key)).and_then(|c| c.f32(0));
    let aim = match n.find(h("AimType")).and_then(|c| c.str_arg(0)) {
        Some(s) if s.eq_ignore_ascii_case("Head") => AimType::Head,
        Some(s) if s.eq_ignore_ascii_case("Torso") => AimType::Torso,
        Some(s) if s.eq_ignore_ascii_case("FullBody") => AimType::FullBody,
        _ => AimType::None,
    };
    Some(SoldierAnimation {
        name,
        looping: f("Loop").is_some_and(|v| v != 0.0),
        aim,
        sync: n
            .find(h("SyncType"))
            .and_then(|c| c.str_arg(0))
            .map(str::to_string),
        blend_in: f("BlendInTime").unwrap_or(0.0),
        blend_out: f("BlendOutTime").unwrap_or(0.0),
        blend_to: n
            .find_all(h("BlendTimeTo"))
            .filter_map(|c| Some((c.str_arg(0)?.to_string(), c.f32(1)?)))
            .collect(),
    })
}

/// Parse one `sanm` chunk.
pub fn parse_sanm(c: &Chunk<'_>) -> Result<SoldierAnimations, UcfbError> {
    let key = fnv1a_lowercase("Animation");
    Ok(SoldierAnimations {
        animations: parse_tree(c)?
            .iter()
            .filter(|n| n.hash == key)
            .filter_map(animation)
            .collect(),
    })
}

/// The `sanm` chunk of an opened LVL payload (BF2 `INGAME.LVL`), if it has one.
pub fn find_sanm(inner: &[u8]) -> Result<Option<SoldierAnimations>, UcfbError> {
    for c in chunks(inner) {
        let c = c?;
        if &c.tag == b"sanm" {
            return parse_sanm(&c).map(Some);
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ps2_chops_a_blend_time_to_hundredths() {
        assert_eq!(stored_hundredths(0.02), 1);
        assert_eq!(stored_hundredths(0.01), 0);
        assert_eq!(stored_hundredths(0.15), 15);
        assert_eq!(stored_hundredths(0.25), 25);
        assert_eq!(stored_hundredths(1.0), 100);
    }

    #[test]
    fn a_name_splits_into_animation_and_layer() {
        let a = |n: &str| SoldierAnimation {
            name: n.into(),
            looping: false,
            aim: AimType::None,
            sync: None,
            blend_in: 0.0,
            blend_out: 0.0,
            blend_to: vec![],
        };
        assert_eq!(
            a("stand_shoot_upper").base(),
            ("stand_shoot", Some(Layer::Upper))
        );
        assert_eq!(
            a("stand_runforward_lower").base(),
            ("stand_runforward", Some(Layer::Lower))
        );
        assert_eq!(a("sprint").base(), ("sprint", None));
        let all = SoldierAnimations {
            animations: vec![a("jump"), a("stand_shoot_lower")],
        };
        assert_eq!(all.get("jump", Layer::Upper).unwrap().name, "jump");
        assert_eq!(
            all.get("stand_shoot", Layer::Lower).unwrap().name,
            "stand_shoot_lower"
        );
        assert!(all.get("stand_shoot", Layer::Upper).is_none());
    }
}
