//! The squad-command marker (brief D11): the icon over a bot the LOCAL player
//! has given an order to.
//!
//! Two textures, both from `common_bf1` — `squad_cmd_highlight` over a bot
//! that took the order and `squad_cmd_cross` over the point a `Hold` was
//! anchored at. They are the strings the design names (PLAN Log 2026-09-01
//! "PHASE D DESIGN" decision 3, the BF1 icon sheet); `common_bf2` ships
//! NEITHER, so a BF2 map draws the BF1 icon — `PROVISIONAL (ours)`, a
//! placeholder until a BF2 icon source is found, and nothing at all when the
//! bake has no texture (the caller passes `None`).
//!
//! Camera-facing quads, BLENDED by the icon's own alpha and not alpha-tested:
//! the PS2 draws every HUD bitmap source-over by its texture alpha (ALPHA 0x44)
//! with no alpha test and no Z (TEST 0x31001), and the icon's shape IS that
//! alpha on the disc (`analysis/re-hud-icons.md` §2 [V]). They ride
//! `render::DrawLayer::Blend`, which writes no depth — so two markers behind
//! one another composite instead of cutting each other out — but which still
//! depth-TESTS, so a wall in front of a bot hides its marker. That depth test
//! is a deviation from TEST 0x31001, kept because the placement is ours anyway
//! (see `squad_icons` in `main/battle.rs`: the PS2 draws these sheets as a 2D menu).
//!
//! **Off in normal play** (card SQMK; the maintainer, INBOX 2026-09-30): the PS2 draws
//! nothing in the world for an order (`analysis/re-squad-orders.md` §2), so
//! the marker and the cross are a dev-mode overlay — see [`markers_on`].

use core::squad::CommandKind;
use core::unit::capsule_height;
use glam::{Mat4, Vec3};
use render::{Camera, DrawCmd, DrawLayer, MeshHandle, TextureHandle};

/// How big the icon is, in metres, at any distance. PROVISIONAL (ours).
pub const MARKER_SIZE: f32 = 0.5;
/// How far above a bot's head the icon floats. PROVISIONAL (ours).
pub const MARKER_LIFT: f32 = 0.4;

/// Whether the marker and the Hold cross are drawn at all: only in dev mode
/// (`dev` — `OPENBF_DEV=1`), unless `OPENBF_SQUAD_MARKER` (`marker`) says
/// otherwise for this one overlay — `1` forces it on, `0` forces it off, any
/// other value leaves it to dev mode.
pub fn markers_on(dev: bool, marker: Option<&str>) -> bool {
    match marker {
        Some("1") => true,
        Some("0") => false,
        _ => dev,
    }
}

/// [`markers_on`] from the environment.
pub fn markers_on_env() -> bool {
    markers_on(
        crate::dev::on(),
        std::env::var("OPENBF_SQUAD_MARKER").ok().as_deref(),
    )
}

/// One camera-facing quad of `size` at `pos` — `fx::billboard`'s basis, which
/// keeps the third column non-degenerate so the winding stays as authored.
fn billboard(pos: Vec3, size: f32, camera: &Camera) -> Mat4 {
    let right = camera.right();
    let up = right.cross(camera.forward()).normalize_or_zero();
    let basis = glam::Mat3::from_cols(right * size, up * size, -camera.forward() * size);
    Mat4::from_translation(pos) * Mat4::from_mat3(basis)
}

/// Where a marker goes for every bot under an order from player `slot`: one
/// point over each bot's head, and — for a `Hold` — one on the ANCHOR it was
/// told to stand at, so the player can see where they sent them.
///
/// Split out from [`squad_draws`] because THIS is the part with a rule in it:
/// it needs no GPU, so a test can read the points straight off the sim.
pub fn marker_points(sim: &core::sim::Sim, slot: u8) -> Vec<(Vec3, Marker)> {
    let mut out = Vec::new();
    for (_, u) in sim.alive_units() {
        let Some(c) = u.command.filter(|c| c.issuer == slot) else {
            continue;
        };
        out.push((
            u.soldier.pos + Vec3::Y * (capsule_height(u.soldier.stance) + MARKER_LIFT),
            Marker::Bot,
        ));
        if let CommandKind::Hold(anchor) = c.kind {
            out.push((anchor + Vec3::Y * MARKER_LIFT, Marker::Anchor));
        }
    }
    out
}

/// Which icon a point asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// Over a commanded bot: `squad_cmd_highlight`.
    Bot,
    /// On a Hold's anchor: `squad_cmd_cross`.
    Anchor,
}

/// [`marker_points`] as camera-facing draws. `highlight` and `cross` are what
/// the caller managed to upload; a `None` texture draws nothing rather than an
/// untextured white square.
pub fn squad_draws(
    sim: &core::sim::Sim,
    slot: u8,
    quad: MeshHandle,
    highlight: Option<TextureHandle>,
    cross: Option<TextureHandle>,
    camera: &Camera,
) -> Vec<DrawCmd> {
    marker_points(sim, slot)
        .into_iter()
        .filter_map(|(p, m)| {
            let t = match m {
                Marker::Bot => highlight,
                Marker::Anchor => cross,
            }?;
            Some(DrawCmd {
                texture: Some(t),
                ..marker_base(quad, billboard(p, MARKER_SIZE, camera))
            })
        })
        .collect()
}

/// Everything about one marker draw except its texture — i.e. exactly
/// `DrawCmd::textured_blended` before the texture is bound: the blend pass
/// (the PS2's ALPHA 0x44 bitmap state, `analysis/re-hud-icons.md` §2) and an
/// element alpha of 1.0 (the PS2 dims the unselected arms of its menu to 0x7f;
/// a world marker has no "unselected"). Split out so a test can check it
/// without a GPU to make a texture handle on.
fn marker_base(quad: MeshHandle, model: Mat4) -> DrawCmd {
    DrawCmd {
        texture: None,
        layer: DrawLayer::Blend,
        ..DrawCmd::flat(quad, [1.0; 4], model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sim::{PadState, Sim};
    use core::unit::{Controller, UnitSpec};

    fn sim_with_bots() -> Sim {
        let mut sim = Sim::empty(3);
        sim.heightfield = Some(core::heightfield::Heightfield {
            size: 2,
            unit: 400.0,
            offset_x: 200.0,
            offset_z: 200.0,
            heights: vec![0.0; 4],
        });
        sim.spawn(&UnitSpec::player_in_slot(0, 0, Vec3::ZERO))
            .expect("player");
        for k in 0..3 {
            sim.spawn(&UnitSpec {
                controller: Controller::Ai,
                ..UnitSpec::trooper(0, Vec3::new(2.0 + k as f32, 0.0, 0.0))
            })
            .expect("bot");
        }
        sim
    }

    /// Card SQMK: no marker and no cross in normal play; dev mode brings them
    /// back; `OPENBF_SQUAD_MARKER` overrides either way.
    #[test]
    fn the_markers_are_off_unless_dev_mode_or_the_override_asks() {
        assert_eq!(
            markers_on(crate::dev::flag(None), None),
            crate::dev::DEFAULT_ON,
            "no env: the default"
        );
        assert!(markers_on(crate::dev::flag(Some("1")), None), "dev: on");
        assert!(!markers_on(crate::dev::flag(Some("0")), None));
        assert!(markers_on(false, Some("1")), "forced on without dev");
        assert!(!markers_on(true, Some("0")), "forced off in dev");
        assert!(
            markers_on(true, Some("")),
            "an empty override decides nothing"
        );
        assert!(!markers_on(false, Some("")));
    }

    /// D11: one marker per COMMANDED bot and none for anyone else, a second one
    /// on a Hold's anchor, and another slot's orders are not mine.
    #[test]
    fn a_marker_is_placed_over_every_bot_under_my_order() {
        let mut sim = sim_with_bots();
        assert!(marker_points(&sim, 0).is_empty(), "nobody is commanded yet");

        let leader = sim.players[0].expect("player");
        // A FOLLOW is taken by two (card SQACK).
        assert_eq!(sim.issue_command(0, CommandKind::Follow(leader)), 2);
        let p = marker_points(&sim, 0);
        assert_eq!(p.len(), 2, "one per bot, no anchor for a Follow: {p:?}");
        let bot = &sim.units[1];
        assert_eq!(
            p[0],
            (
                bot.soldier.pos + Vec3::Y * (capsule_height(bot.soldier.stance) + MARKER_LIFT),
                Marker::Bot
            ),
            "over the bot's head, above the capsule"
        );
        assert!(p.iter().all(|(_, m)| *m == Marker::Bot));

        // Hold: the anchor is the bot's OWN position (D3), so each bot gets a
        // cross on the spot as well as an icon over its head.
        assert_eq!(sim.issue_command(0, CommandKind::Hold(Vec3::ZERO)), 3);
        let p = marker_points(&sim, 0);
        assert_eq!(p.len(), 6);
        assert_eq!(p.iter().filter(|(_, m)| *m == Marker::Anchor).count(), 3);
        assert_eq!(
            p[1].0,
            sim.units[1].soldier.pos + Vec3::Y * MARKER_LIFT,
            "the cross is on the anchor, not over the head"
        );

        // Another slot's orders are not mine, and the sim keeps stepping.
        assert!(marker_points(&sim, 1).is_empty());
        sim.step(&[PadState::default()]);
        assert_eq!(marker_points(&sim, 0).len(), 6, "still theirs next tick");
    }

    /// Queue item SM: the marker composites by its texture's alpha
    /// (`re-hud-icons.md` §2 — ALPHA 0x44, no alpha test) rather than being
    /// alpha-TESTED, at an element alpha of 1.0, and it is never far-culled.
    #[test]
    fn the_marker_draws_in_the_blend_pass() {
        let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let d = marker_base(MeshHandle::default(), m);
        assert_eq!(d.layer, DrawLayer::Blend, "blended, not Mask");
        assert_eq!(d.color, [1.0; 4], "element alpha 1.0, untinted");
        assert_eq!(d.cull, None);
        assert_eq!(d.model, m);
    }
}
