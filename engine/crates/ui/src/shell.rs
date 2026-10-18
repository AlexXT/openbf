//! Shell v0 — BF2-style Instant Action setup (PLAN §Presentation: BF2 shell
//! flow): pick map, mode, the two factions (era mix = any pairing), AI per
//! team, the reinforcement percentage and the time limit, then Start. Pure egui over a [`ShellModel`]; the game
//! owns the map list and applies [`ShellAction`]s. Controller-navigated like
//! the pause menu (D-pad, Cross, ◀▶ to cycle values).

use egui::{Align2, Color32, RichText, Vec2};

pub const FACTIONS: [&str; 4] = ["all", "imp", "rep", "cis"];

#[derive(Debug, Clone, Default)]
pub struct ShellModel {
    /// (map name, source game) for every baked map found.
    pub maps: Vec<(String, String)>,
    pub map_idx: usize,
    pub mode: String,
    pub teams: [usize; 2],
    pub ai_per_team: u32,
    /// The map's own reinforcement counts, team 1 / team 2 (the mission's,
    /// else the 250 default) — what [`ShellModel::ticket_pct`] scales.
    pub base_tickets: [u32; 2],
    /// The disc's `con_mult` option: 10..500 % in steps of 10, default 100
    /// (`analysis/re-time-limit.md` §5 [V][O]; queue TLO replaced Shell v0's
    /// absolute count, which seeded both teams from team 1's).
    pub ticket_pct: u32,
    /// The disc's `con_timer` option in minutes: 0 = Off (the power-on
    /// value), else 5..60 in steps of 5 (`analysis/re-time-limit.md` §1
    /// [V][O]).
    pub time_limit: u32,
    pub cursor: usize,
    /// Career line for the title block.
    pub career_line: String,
    /// Whether a mission file exists for the current map+mode.
    pub mission_found: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShellAction {
    Start,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Row {
    Map,
    Mode,
    TeamA,
    TeamB,
    Ai,
    Tickets,
    TimeLimit,
    Start,
    Quit,
}

pub const ROWS: [Row; 9] = [
    Row::Map,
    Row::Mode,
    Row::TeamA,
    Row::TeamB,
    Row::Ai,
    Row::Tickets,
    Row::TimeLimit,
    Row::Start,
    Row::Quit,
];

fn cycle(i: usize, n: usize, right: bool) -> usize {
    if n == 0 {
        0
    } else if right {
        (i + 1) % n
    } else {
        (i + n - 1) % n
    }
}

/// Apply controller edges; mutates the model's values and returns actions.
pub fn navigate(m: &mut ShellModel, input: crate::menu::MenuInput) -> Vec<ShellAction> {
    let n = ROWS.len();
    let mut actions = Vec::new();
    if input.up {
        m.cursor = (m.cursor + n - 1) % n;
    }
    if input.down {
        m.cursor = (m.cursor + 1) % n;
    }
    let lr = input.left || input.right;
    match ROWS[m.cursor] {
        Row::Map if lr => m.map_idx = cycle(m.map_idx, m.maps.len(), input.right),
        Row::TeamA if lr => m.teams[0] = cycle(m.teams[0], FACTIONS.len(), input.right),
        Row::TeamB if lr => m.teams[1] = cycle(m.teams[1], FACTIONS.len(), input.right),
        Row::Ai if lr => {
            m.ai_per_team = if input.right {
                (m.ai_per_team + 2).min(32)
            } else {
                m.ai_per_team.saturating_sub(2)
            }
        }
        // The disc's clamps: `Clamp(iCONMult + 10·dir, 10, 500)` and
        // `Clamp(iCONTimer + 5·dir, 0, 60)` (`analysis/re-time-limit.md`).
        Row::Tickets if lr => {
            m.ticket_pct = if input.right {
                (m.ticket_pct + 10).min(500)
            } else {
                m.ticket_pct.saturating_sub(10).max(10)
            }
        }
        Row::TimeLimit if lr => {
            m.time_limit = if input.right {
                (m.time_limit + 5).min(60)
            } else {
                m.time_limit.saturating_sub(5)
            }
        }
        Row::Start if input.confirm => actions.push(ShellAction::Start),
        Row::Quit if input.confirm => actions.push(ShellAction::Quit),
        _ => {}
    }
    if input.confirm
        && matches!(
            ROWS[m.cursor],
            Row::Map | Row::Mode | Row::TeamA | Row::TeamB
        )
    {
        // Confirm on a value row cycles it too (feels right on a pad).
        let _ = navigate(
            m,
            crate::menu::MenuInput {
                right: true,
                ..Default::default()
            },
        );
    }
    actions
}

fn row(ui: &mut egui::Ui, selected: bool, label: &str, value: String) -> bool {
    let bg = if selected {
        Color32::from_rgba_premultiplied(60, 90, 140, 200)
    } else {
        Color32::TRANSPARENT
    };
    let mut clicked = false;
    egui::Frame::new()
        .fill(bg)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .corner_radius(3.0)
        .show(ui, |ui| {
            ui.set_min_width(440.0);
            ui.horizontal(|ui| {
                let r = ui.label(RichText::new(label).size(18.0).color(if selected {
                    Color32::WHITE
                } else {
                    Color32::LIGHT_GRAY
                }));
                clicked |= r.clicked();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let r = ui.label(
                        RichText::new(value)
                            .size(18.0)
                            .color(Color32::WHITE)
                            .strong(),
                    );
                    clicked |= r.clicked();
                });
            });
        });
    clicked
}

pub fn draw_shell(ctx: &egui::Context, m: &ShellModel) -> Vec<ShellAction> {
    let mut actions = Vec::new();
    egui::Area::new(egui::Id::new("shell"))
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Color32::from_black_alpha(210))
                .inner_margin(egui::Margin::same(22))
                .corner_radius(6.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("openbf")
                                .size(34.0)
                                .color(Color32::WHITE)
                                .strong(),
                        );
                        ui.label(
                            RichText::new("INSTANT ACTION")
                                .size(16.0)
                                .color(Color32::from_rgb(242, 160, 60)),
                        );
                        ui.label(
                            RichText::new(&m.career_line)
                                .size(13.0)
                                .color(Color32::GRAY),
                        );
                    });
                    ui.add_space(10.0);
                    // Baked map keys are `<game>_<lvl>` (bf2_hot1); showing
                    // that next to the game tag reads as "bf2_hot1 (bf2)", so
                    // strip the prefix the tag already carries.
                    let map = m
                        .maps
                        .get(m.map_idx)
                        .map(|(n, g)| {
                            let short = n.strip_prefix(&format!("{g}_")).unwrap_or(n);
                            format!("{}  ({g})", short.to_uppercase())
                        })
                        .unwrap_or_else(|| "no baked maps found".into());
                    let mission = if m.mission_found {
                        ""
                    } else {
                        "  (no mission file: defaults)"
                    };
                    for (i, r) in ROWS.iter().enumerate() {
                        let sel = i == m.cursor;
                        let clicked = match r {
                            Row::Map => row(ui, sel, "Map  ◀ ▶", map.clone()),
                            Row::Mode => row(ui, sel, "Mode", format!("{}{mission}", m.mode)),
                            Row::TeamA => {
                                row(ui, sel, "Team 1  ◀ ▶", FACTIONS[m.teams[0]].to_uppercase())
                            }
                            Row::TeamB => {
                                row(ui, sel, "Team 2  ◀ ▶", FACTIONS[m.teams[1]].to_uppercase())
                            }
                            Row::Ai => row(ui, sel, "AI per team  ◀ ▶", m.ai_per_team.to_string()),
                            Row::Tickets => {
                                // `core::sim::scale_reinforcements`'s rule
                                // (ui does not depend on core).
                                let [a, b] = m
                                    .base_tickets
                                    .map(|n| (u64::from(n) * u64::from(m.ticket_pct) / 100) as u32);
                                row(
                                    ui,
                                    sel,
                                    "Reinforcements  ◀ ▶",
                                    format!("{} %  ({a} / {b})", m.ticket_pct),
                                )
                            }
                            Row::TimeLimit => row(
                                ui,
                                sel,
                                "Time limit  ◀ ▶",
                                match m.time_limit {
                                    0 => "Off".to_string(),
                                    n => format!("{n} min"),
                                },
                            ),
                            Row::Start => row(ui, sel, "START BATTLE", String::new()),
                            Row::Quit => row(ui, sel, "Quit", String::new()),
                        };
                        if clicked {
                            match r {
                                Row::Start => actions.push(ShellAction::Start),
                                Row::Quit => actions.push(ShellAction::Quit),
                                _ => {}
                            }
                        }
                    }
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("D-pad move · ◀▶ change · ✕ select")
                            .size(12.0)
                            .color(Color32::GRAY),
                    );
                });
        });
    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::MenuInput;

    #[test]
    fn shell_navigation_cycles_values_and_starts() {
        let mut m = ShellModel {
            maps: vec![("tat1".into(), "bf1".into()), ("tat2".into(), "bf2".into())],
            mode: "con".into(),
            teams: [0, 1],
            ai_per_team: 8,
            base_tickets: [150, 220],
            ticket_pct: 100,
            ..Default::default()
        };
        // Map row: right cycles, wraps.
        let right = MenuInput {
            right: true,
            ..Default::default()
        };
        assert!(navigate(&mut m, right).is_empty());
        assert_eq!(m.map_idx, 1);
        navigate(&mut m, right);
        assert_eq!(m.map_idx, 0);
        // Team B → cis via three rights from the TeamB row.
        m.cursor = 3;
        for _ in 0..2 {
            navigate(&mut m, right);
        }
        assert_eq!(FACTIONS[m.teams[1]], "cis");
        // AI, the reinforcement % and the time limit clamp.
        m.cursor = 4;
        for _ in 0..20 {
            navigate(&mut m, right);
        }
        assert_eq!(m.ai_per_team, 32);
        m.cursor = 5;
        let left = MenuInput {
            left: true,
            ..Default::default()
        };
        for _ in 0..20 {
            navigate(&mut m, left);
        }
        assert_eq!(m.ticket_pct, 10);
        for _ in 0..60 {
            navigate(&mut m, right);
        }
        assert_eq!(m.ticket_pct, 500);
        m.cursor = 6;
        assert_eq!(m.time_limit, 0, "Off by default");
        navigate(&mut m, right);
        assert_eq!(m.time_limit, 5);
        for _ in 0..20 {
            navigate(&mut m, right);
        }
        assert_eq!(m.time_limit, 60);
        for _ in 0..20 {
            navigate(&mut m, left);
        }
        assert_eq!(m.time_limit, 0);
        // Start row confirms.
        m.cursor = 7;
        let a = navigate(
            &mut m,
            MenuInput {
                confirm: true,
                ..Default::default()
            },
        );
        assert_eq!(a, vec![ShellAction::Start]);
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            draw_shell(ctx, &m);
        });
    }
}
