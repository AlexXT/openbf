//! Spawn screen: what a dead player looks at while their wave slot comes up.
//!
//! Pure presentation — `SpawnModel` is filled by the game layer from the sim's
//! command posts and `Sim.waves[team]`.
//!
//! Two selections, and they are NOT the same kind of thing:
//! - the CLASS row is real. `selected_class` is read straight out of
//!   `Sim::spawn_choice[slot]`, the player moves it with pad bits the input
//!   log carries (`buttons::SPAWN_CLASS_NEXT`/`PREV`), and the respawn is
//!   built from it — so it reproduces under replay like any other sim state.
//! - the POST cursor (`selected`) is still UI-side only: the sim picks the
//!   spawn node itself. Wiring that up the same way is a later item.

use egui::{Align2, Color32, FontId, Frame, RichText, Vec2};

use crate::hud::TEAM_COLORS;

#[derive(Debug, Clone, Default)]
pub struct SpawnPost {
    pub name: String,
    /// Enemies are neutralizing this post right now.
    pub contested: bool,
    /// Metres from where the player died (ordering aid, not a game rule).
    pub distance: f32,
}

#[derive(Debug, Clone, Default)]
pub struct SpawnModel {
    pub team_names: [String; 2],
    pub player_team: u8,
    pub tickets: [u32; 2],
    /// Posts this team can spawn at, in map order.
    pub posts: Vec<SpawnPost>,
    /// Cursor into `posts` (UI-side; clamped when drawing).
    pub selected: usize,
    /// The player's soldier classes, in the team's table order — display
    /// labels from `game::walk::TeamTemplate::classes`. Empty = the team has
    /// no class table and no row is drawn.
    pub classes: Vec<String>,
    /// `Sim::spawn_choice[slot]` — the class the next respawn takes (clamped
    /// when drawing).
    pub selected_class: usize,
    /// Seconds until this player's spawn slot opens; None = slot is open.
    pub countdown: Option<f32>,
}

/// Draw the spawn screen over the frame. Call instead of the normal death
/// banner while the player is dead and the round is still running.
pub fn draw_spawn(ctx: &egui::Context, m: &SpawnModel) {
    let team = usize::from(m.player_team.min(1));
    let accent = TEAM_COLORS[team];

    // Dim the battle behind the screen.
    egui::Area::new(egui::Id::new("spawn_dim"))
        .anchor(Align2::LEFT_TOP, Vec2::ZERO)
        .show(ctx, |ui| {
            let r = ctx.content_rect();
            ui.painter()
                .rect_filled(r, 0.0, Color32::from_black_alpha(150));
            ui.allocate_space(r.size());
        });

    egui::Area::new(egui::Id::new("spawn_panel"))
        .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
        .show(ctx, |ui| {
            Frame::new()
                .fill(Color32::from_black_alpha(215))
                .inner_margin(egui::Margin::symmetric(28, 20))
                .corner_radius(6.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("SELECT SPAWN POINT")
                                .font(FontId::proportional(26.0))
                                .color(accent)
                                .strong(),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(format!(
                                "{}  ·  {} reinforcements",
                                m.team_names[team].to_uppercase(),
                                m.tickets[team]
                            ))
                            .size(13.0)
                            .color(Color32::LIGHT_GRAY),
                        );
                    });
                    ui.add_space(12.0);

                    // The class row: the one selection on this screen that
                    // really does reach the sim.
                    if !m.classes.is_empty() {
                        let pick = m.selected_class.min(m.classes.len() - 1);
                        ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            for (i, c) in m.classes.iter().enumerate() {
                                let picked = i == pick;
                                ui.label(
                                    RichText::new(c.to_uppercase())
                                        .size(if picked { 16.0 } else { 14.0 })
                                        .color(if picked {
                                            accent
                                        } else {
                                            Color32::from_gray(150)
                                        })
                                        .strong(),
                                );
                                if i + 1 < m.classes.len() {
                                    ui.label(RichText::new("·").size(14.0).color(Color32::DARK_GRAY));
                                }
                            }
                        });
                        ui.label(
                            RichText::new("R1 / Select to change class")
                                .size(12.0)
                                .color(Color32::GRAY),
                        );
                        });
                        ui.add_space(10.0);
                    }

                    if m.posts.is_empty() {
                        ui.label(
                            RichText::new("no friendly command post — spawning at the fallback")
                                .size(15.0)
                                .color(Color32::from_rgb(230, 180, 80)),
                        );
                    }
                    let sel = m.selected.min(m.posts.len().saturating_sub(1));
                    for (i, p) in m.posts.iter().enumerate() {
                        let picked = i == sel;
                        let (mark, name_color) = if picked {
                            ("▶", accent)
                        } else {
                            ("  ", Color32::from_gray(210))
                        };
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(mark).size(16.0).color(accent));
                            ui.label(
                                RichText::new(p.name.to_uppercase())
                                    .size(17.0)
                                    .color(name_color)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(format!("{:.0} m", p.distance))
                                    .size(13.0)
                                    .color(Color32::GRAY),
                            );
                            if p.contested {
                                ui.label(
                                    RichText::new("UNDER ATTACK")
                                        .size(13.0)
                                        .color(Color32::from_rgb(240, 90, 60))
                                        .strong(),
                                );
                            }
                        });
                    }

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| match m.countdown {
                        Some(s) => ui.label(
                            RichText::new(format!("SPAWNING IN {}", s.ceil().max(1.0) as u32))
                                .font(FontId::proportional(22.0))
                                .color(Color32::WHITE)
                                .strong(),
                        ),
                        None => ui.label(
                            RichText::new("SPAWNING…")
                                .font(FontId::proportional(22.0))
                                .color(accent)
                                .strong(),
                        ),
                    });
                    ui.add_space(4.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("D-pad up / down: select post")
                                .size(12.0)
                                .color(Color32::GRAY),
                        );
                        // Honest label: the CLASS choice reaches the sim now
                        // (it is `Sim::spawn_choice`), the POST cursor still
                        // does not — the maintainer should not have to guess which is
                        // which from a dev build.
                        ui.label(
                            RichText::new(
                                "the class choice reaches the sim — the post cursor is still a preview",
                            )
                            .size(11.0)
                            .color(Color32::from_gray(120)),
                        );
                    });
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> SpawnModel {
        SpawnModel {
            team_names: ["all".into(), "imp".into()],
            player_team: 0,
            tickets: [147, 152],
            posts: vec![
                SpawnPost {
                    name: "cp1".into(),
                    contested: false,
                    distance: 42.0,
                },
                SpawnPost {
                    name: "cp3".into(),
                    contested: true,
                    distance: 118.0,
                },
            ],
            selected: 1,
            classes: vec![
                "Trooper".into(),
                "Assault Gunner".into(),
                "Pilot".into(),
                "Rebel Scout".into(),
                "Repair".into(),
            ],
            selected_class: 2,
            countdown: Some(3.4),
        }
    }

    /// Lays out headless at two aspects, in every state the game can hand it.
    #[test]
    fn spawn_screen_lays_out_headless() {
        for (w, h) in [(1280.0, 720.0), (2560.0, 1080.0)] {
            let ctx = egui::Context::default();
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(w, h))),
                ..Default::default()
            };
            // Anchored areas size on frame 1 and lay out on frame 2.
            let _ = ctx.run(input.clone(), |ctx| draw_spawn(ctx, &model()));
            let out = ctx.run(input, |ctx| draw_spawn(ctx, &model()));
            let prims = ctx.tessellate(out.shapes, out.pixels_per_point);
            let indices: usize = prims
                .iter()
                .map(|p| match &p.primitive {
                    egui::epaint::Primitive::Mesh(m) => m.indices.len(),
                    _ => 0,
                })
                .sum();
            assert!(indices > 300, "{w}x{h}: {indices} indices");
        }
    }

    #[test]
    fn degenerate_models_do_not_panic() {
        let ctx = egui::Context::default();
        // No posts, no countdown, selection past the end.
        let m = SpawnModel {
            posts: Vec::new(),
            selected: 7,
            countdown: None,
            ..model()
        };
        let _ = ctx.run(egui::RawInput::default(), |ctx| draw_spawn(ctx, &m));
        // Selection past the end with posts present clamps to the last.
        let m = SpawnModel {
            selected: 99,
            ..model()
        };
        let _ = ctx.run(egui::RawInput::default(), |ctx| draw_spawn(ctx, &m));
    }

    /// The class row draws for every table size the game can hand it — none
    /// (a team with no class table: no row at all), one, six, and a choice
    /// past the end (it clamps).
    #[test]
    fn the_class_row_renders_at_every_table_size() {
        let count = |m: &SpawnModel| {
            let ctx = egui::Context::default();
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1280.0, 720.0),
                )),
                ..Default::default()
            };
            let _ = ctx.run(input.clone(), |ctx| draw_spawn(ctx, m));
            let out = ctx.run(input, |ctx| draw_spawn(ctx, m));
            ctx.tessellate(out.shapes, out.pixels_per_point)
                .iter()
                .map(|p| match &p.primitive {
                    egui::epaint::Primitive::Mesh(m) => m.indices.len(),
                    _ => 0,
                })
                .sum::<usize>()
        };
        let none = count(&SpawnModel {
            classes: Vec::new(),
            selected_class: 0,
            ..model()
        });
        assert!(none > 300, "{none} indices with no class row");
        let six = count(&SpawnModel {
            classes: (0..6).map(|i| format!("class {i}")).collect(),
            selected_class: 5,
            ..model()
        });
        assert!(six > none, "the class row draws something: {none} -> {six}");
        // One class, and a choice past the end of the list.
        let one = count(&SpawnModel {
            classes: vec!["Trooper".into()],
            selected_class: 9,
            ..model()
        });
        assert!(one > none);
    }
}
