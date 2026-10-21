//! The PS2 spawn screens' content (card SSELB, `analysis/re-spawn-flow.md`
//! §2-§4): what `ui::spawn_draw` draws for the player's slot - the side
//! select's rows and highlight, the spawn map's objective and legend, the class panel's
//! lines as `FUN_00396430` assembles them - with the disc's texts through
//! [`crate::event_feed::EventFeed`].

/// The PS2 spawn screen for `slot` as the HUD pass draws it (card SSELB,
/// `ui::spawn_draw`): its texts from the disc's table through `feed`, the
/// class panel's lines assembled as `FUN_00396430` does
/// (`analysis/re-spawn-flow.md` §4). `None` when the battle runs no spawn
/// screens or the slot is only waiting.
pub fn page(
    walk: &crate::walk::WalkSim,
    feed: &crate::event_feed::EventFeed,
    slot: usize,
) -> Option<ui::spawn_draw::SpawnPage> {
    use core::sim::{SideChoice, SpawnStage};
    use ui::spawn_draw::SpawnPage;
    if !walk.sim.spawn_screens {
        return None;
    }
    let me = &walk.sim.units[walk.player.index as usize];
    match walk.sim.spawn_stage.get(slot).copied().unwrap_or_default() {
        SpawnStage::Side => {
            let [a, b] = feed.team_names();
            Some(SpawnPage::Side {
                title: feed.screen_text("ifs.sideselect.chooseteam", "Select a team"),
                rows: vec![
                    a.to_owned(),
                    b.to_owned(),
                    feed.screen_text("ifs.sideselect.autoassign", "Auto Assign"),
                ],
                selected: match walk.sim.side_choice.get(slot).copied().unwrap_or_default() {
                    SideChoice::Team1 => 0,
                    SideChoice::Team2 => 1,
                    SideChoice::Auto => 2,
                },
                select: feed.screen_text("common.select", "Select"),
            })
        }
        // The objective is the conquest mission's `AddMissionObjective`
        // text (`game.modes.con`; the defenders' `con2` reads the same);
        // the port's battles are all conquest.
        SpawnStage::Map => Some(SpawnPage::Map {
            objective: feed.screen_text("game.modes.con", "Capture all the Command Posts"),
            legend: [
                feed.screen_text("game.spawndisplay.commandpost", "Command Post"),
                feed.screen_text("game.spawndisplay.cpchange", "Change Command Post"),
                feed.screen_text(
                    "game.spawndisplay.spawndisplay.promptNext",
                    "{btna} Select Command Post",
                ),
            ],
        }),
        SpawnStage::Class => {
            let team = me.team.min(1);
            let t = &walk.teams[usize::from(team)];
            let pick = walk
                .sim
                .spawn_choice
                .get(slot)
                .map_or(0, |c| usize::from(*c))
                .min(t.classes.len().saturating_sub(1));
            let class = t.classes.get(pick)?;
            let title = feed
                .class_names(team)
                .get(pick)
                .cloned()
                .unwrap_or_else(|| class.label.clone());
            // "Unit Count: %d": the team's members on the field in this
            // class (`FUN_0036f538`, B2:367976). The port has no unlocks,
            // so never the "Max" form nor a refusal line.
            let count = walk
                .sim
                .units
                .iter()
                .filter(|u| u.alive && u.team == team && usize::from(u.spec.class) == pick)
                .count();
            let mut text = feed
                .text_by_hash(0xecf8_496f, "Unit Count: %d\r\n\r\n")
                .replacen("%d", &count.to_string(), 1);
            text.push_str(&feed.text_by_hash(0xfe98_f5cf, "Equipment:"));
            text.push('\n');
            // The primary first, then the other slots in slot order.
            let mut held: Vec<&(u8, crate::walk::WeaponHud)> = class.held_hud.iter().collect();
            held.sort_by_key(|(slot, _)| *slot);
            for w in std::iter::once(&class.weapon_hud).chain(held.into_iter().map(|(_, w)| w)) {
                if w.class.is_empty() {
                    continue;
                }
                let name = feed
                    .weapon_name(&w.class)
                    .unwrap_or_else(|| w.class.clone());
                text.push_str(&name);
                text.push('\n');
            }
            let lines = text
                .trim_end()
                .split('\n')
                .map(|l| l.trim_end_matches('\r').to_owned())
                .collect();
            Some(SpawnPage::Class {
                title,
                lines,
                back: feed
                    .screen_text("game.spawndisplay.characterdisplay.promptprev", "Back")
                    .trim()
                    .to_owned(),
                accept: feed
                    .screen_text(
                        "game.spawndisplay.characterdisplay.promptnext",
                        "Select Character",
                    )
                    .trim()
                    .to_owned(),
                team,
                class: pick,
                classes: t.classes.len(),
            })
        }
        SpawnStage::Accepted => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_feed::EventFeed;
    use crate::walk::{ClassVisual, WalkSim, WeaponHud};
    use core::sim::{buttons, PadState};
    use glam::Vec3;
    use ui::spawn_draw::SpawnPage;

    fn pad(b: u32) -> PadState {
        PadState {
            buttons: b,
            ..PadState::default()
        }
    }

    /// The side select follows the sim's highlight; the class panel lists
    /// the count of the class's live team-mates, a blank line, Equipment:
    /// and the primary, then the other slots in slot order; a battle
    /// without screens has no page.
    #[test]
    fn the_pages_follow_the_sim() {
        let mut w = WalkSim::new(3, Vec3::ZERO, None);
        let feed = EventFeed::default();
        assert_eq!(page(&w, &feed, 0), None, "no screens");
        assert!(w.sim.open_spawn_screens(0, true));
        let side = |w: &WalkSim| match page(w, &feed, 0) {
            Some(SpawnPage::Side {
                rows,
                selected,
                title,
                select,
            }) => {
                assert_eq!(
                    (title.as_str(), select.as_str()),
                    ("Select a team", "Select")
                );
                assert_eq!(rows.len(), 3);
                assert_eq!(rows[2], "Auto Assign");
                selected
            }
            p => panic!("{p:?}"),
        };
        assert_eq!(side(&w), 0);
        w.step_tick(&[pad(buttons::SPAWN_POST_DOWN)]);
        w.step_tick(&[pad(0)]);
        assert_eq!(side(&w), 1);
        w.step_tick(&[pad(buttons::SPAWN_ACCEPT)]);
        w.step_tick(&[pad(0)]);
        assert_eq!(
            page(&w, &feed, 0),
            Some(SpawnPage::Map {
                objective: "Capture all the Command Posts".into(),
                legend: [
                    "Command Post".into(),
                    "Change Command Post".into(),
                    "{btna} Select Command Post".into(),
                ],
            })
        );
        w.sim.spawn_stage[0] = core::sim::SpawnStage::Class;
        let me = w.player.index as usize;
        let team = usize::from(w.sim.units[me].team.min(1));
        let hud = |class: &str| WeaponHud {
            class: class.into(),
            ..WeaponHud::default()
        };
        let c: &mut ClassVisual = &mut w.teams[team].classes[0];
        c.weapon_hud = hud("imp_weap_inf_rifle");
        c.held_hud = vec![
            (4, hud("imp_weap_inf_thermaldetonator")),
            (1, hud("imp_weap_inf_pistol")),
        ];
        let mates = w
            .sim
            .units
            .iter()
            .filter(|u| u.alive && usize::from(u.team) == team && u.spec.class == 0)
            .count();
        match page(&w, &feed, 0) {
            Some(SpawnPage::Class {
                lines,
                back,
                accept,
                ..
            }) => {
                assert_eq!(
                    lines,
                    [
                        format!("Unit Count: {mates}"),
                        String::new(),
                        "Equipment:".into(),
                        "imp_weap_inf_rifle".into(),
                        "imp_weap_inf_pistol".into(),
                        "imp_weap_inf_thermaldetonator".into(),
                    ]
                );
                assert_eq!(
                    (back.as_str(), accept.as_str()),
                    ("Back", "Select Character")
                );
            }
            p => panic!("{p:?}"),
        }
    }
}
