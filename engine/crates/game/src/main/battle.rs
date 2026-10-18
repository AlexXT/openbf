//! The battle every mode builds and shows: a baked map loaded and uploaded
//! with its common assets ([`load_and_upload`]), the two teams from the class
//! data, the `WalkSim` set up from the CLI or the shell ([`make_walk`]), and
//! what a frame draws of it besides the scene — the marker, target, bolt and
//! post boxes ([`walk_draws`]), the squad icons, and the HUD and spawn-screen
//! models the `ui` crate paints.

use super::BattleSetup;
use anyhow::Result;
use game::viewer::{upload_scene, SceneGpu};
use game::walk::WalkSim;
use render::{DrawCmd, MeshHandle, Renderer};

/// Walk-mode soldier marker tint (orange — reads on sand and rock).
const MARKER_COLOR: [f32; 4] = [0.9, 0.45, 0.1, 1.0];

pub(super) fn load_and_upload(
    r: &mut Renderer,
    dir: &std::path::Path,
    mode: &str,
) -> Result<(SceneGpu, game::assets::Scene)> {
    let (scene, load) = game::assets::load_battle_scene(dir, mode)?;
    if let Some(layer) = &load.layer {
        println!("mode layer '{layer}' merged");
    }
    match &load.common {
        Some((cd, Ok(n))) => println!("common assets merged from {} (+{n} models)", cd.display()),
        Some((cd, Err(e))) => eprintln!("common assets at {} unreadable: {e:#}", cd.display()),
        None => {}
    }
    let gpu = upload_scene(r, &scene)?;
    println!(
        "map '{}': {} draws ({} instances, {} skipped classes)",
        scene.manifest.map,
        gpu.draws.len(),
        scene.manifest.instances.len(),
        gpu.skipped.len()
    );
    for (class, n) in &gpu.skipped {
        println!("  skipped {n:3}x {class} (no drawable model)");
    }
    Ok((gpu, scene))
}

/// The two squad-command icons (brief D11), both out of `common_bf1`: BF2
/// ships neither, so a BF2 map shows the BF1 icon — `PROVISIONAL (ours)` —
/// and a bake without them shows nothing at all.
///
/// **OFF in normal play** (card SQMK): loaded only when
/// `game::squad_vis::markers_on_env` says so — dev mode (on by default; `OPENBF_DEV=0` is off), or
/// `OPENBF_SQUAD_MARKER=1` for this overlay alone (`=0` hides it in dev mode).
/// The icon's shape is its disc alpha (`analysis/re-hud-icons.md` §2 [V] —
/// 1,594 of `squad_cmd_highlight`'s 4,096 texels at 0), which the bake keeps.
///
/// The PLACEMENT is the recorded PROVISIONAL deviation: the PS2 draws these
/// two sheets as its 2D D-pad command MENU at HUD (−120, −50) — the cross at
/// the centre, one highlight arm rotated four ways around it (§2 "Where") —
/// and never over a bot in the world. PLAN §DESIGN SPECS Phase D decision 3
/// named the sheets, not that placement; the in-world marker is ours.
pub(super) fn squad_icons(
    r: &mut render::Renderer,
    assets_dir: &std::path::Path,
) -> (Option<render::TextureHandle>, Option<render::TextureHandle>) {
    if !game::squad_vis::markers_on_env() {
        return (None, None);
    }
    let dir = assets_dir.join("common_bf1").join("textures");
    let load = |r: &mut render::Renderer, n: &str| {
        game::viewer::upload_texture_file(r, &dir, n).ok().flatten()
    };
    (load(r, "squad_cmd_highlight"), load(r, "squad_cmd_cross"))
}

fn marker_draw(cube: MeshHandle, walk: &WalkSim) -> DrawCmd {
    DrawCmd::flat(cube, MARKER_COLOR, walk.marker_transform())
}

/// Soldier marker + enemy boxes (magnet target highlighted red, health
/// darkens the box) + in-flight bolts (team-tinted).
pub fn walk_draws(cube: MeshHandle, walk: &WalkSim, draws: &mut Vec<DrawCmd>, models: [bool; 2]) {
    let player_team = walk.sim.units[walk.player.index as usize].team;
    if walk.player_on_foot()
        && !walk.player_body_hidden()
        && !models[usize::from(player_team.min(1))]
    {
        draws.push(marker_draw(cube, walk));
    }
    for t in walk.target_draws() {
        if models[usize::from(t.team.min(1))] {
            continue; // drawn as an animated model instead
        }
        let base = if t.magnet {
            [0.85, 0.15, 0.15]
        } else {
            [0.35, 0.35, 0.4]
        };
        let k = 0.4 + 0.6 * t.health_frac;
        draws.push(DrawCmd::flat(
            cube,
            [base[0] * k, base[1] * k, base[2] * k, 1.0],
            t.transform,
        ));
    }
    for (m, team) in walk.bolt_draws() {
        let color = if team == 0 {
            [1.0, 0.25, 0.1, 1.0]
        } else {
            [0.3, 1.0, 0.3, 1.0]
        };
        draws.push(DrawCmd::flat(cube, color, m));
    }
}

/// Team templates from the baked class data (factions default to the map's
/// source game); falls back to the built-in BF1 trooper with a warning.
fn team_templates(
    scene: &game::assets::Scene,
    b: &BattleSetup,
    sim: &mut core::sim::Sim,
) -> [game::walk::TeamTemplate; 2] {
    let defaults = game::roster::default_factions(&scene.manifest.source_game);
    let factions: [&str; 2] = match &b.teams {
        Some(t) => [t[0].as_str(), t[1].as_str()],
        None => defaults,
    };
    match game::roster::FactionRoster::pair(
        &b.classes_dir,
        &scene.manifest.source_game,
        factions,
        b.mission.as_ref(),
    ) {
        Ok((rosters, warnings)) => {
            for w in &warnings {
                eprintln!("{w}");
            }
            // One `Sim::weapon_classes` index per weapon NAME (a Vec scan —
            // the sim never iterates a hash map), and the per-team class
            // TABLE the AI mix and the spawn-select both read.
            let mut names: Vec<(String, usize)> = Vec::new();
            let mut team = 0usize;
            rosters.map(|r| {
                let mut table = Vec::new();
                let mut visuals = Vec::new();
                for c in &r.classes {
                    let uc = c.register(sim, &mut names);
                    println!(
                        "team {} class {}: {} — min {} — {} HP, {:.1} m/s, {} ({} dmg, {} m/s bolts){}",
                        r.faction,
                        c.class,
                        c.label,
                        c.min,
                        c.max_health,
                        c.consts.max_speed,
                        c.weapon.name,
                        c.weapon.ordnance.max_damage,
                        c.weapon.ordnance.velocity,
                        if c.jet { "  [jet pack]" } else { "" }
                    );
                    table.push(uc);
                    visuals.push(game::walk::ClassVisual {
                        name: c.class.clone(),
                        label: c.label.clone(),
                        geometry: c.geometry.clone(),
                        weapon_geometry: c.weapon_geometry.clone(),
                        weapon_hud: c.weapon_hud.clone(),
                        held_hud: c.held.iter().map(|h| (h.slot, h.hud.clone())).collect(),
                        foley_class: c.foley_class,
                        cries: c.cries.clone(),
                        anim: c.anim.clone(),
                    });
                }
                sim.classes[team] = table;
                team += 1;
                let c = r.fill_class();
                game::walk::TeamTemplate {
                    faction: r.faction.clone(),
                    name: c.class.clone(),
                    consts: c.consts,
                    max_health: c.max_health,
                    health_type: c.health_type,
                    weapon_class: sim.classes[team - 1][0].weapon_class,
                    weapon_slot: sim.classes[team - 1][0].weapon_slot,
                    slots: sim.classes[team - 1][0].slots.clone(),
                    drops: sim.classes[team - 1][0].drops,
                    spare_clips: c.spare_clips,
                    geometry: c.geometry.clone(),
                    weapon_geometry: c.weapon_geometry.clone(),
                    classes: visuals,
                }
            })
        }
        Err(e) => {
            eprintln!("class data unavailable ({e:#}); using built-in BF1 troopers");
            let t = game::walk::TeamTemplate::default_trooper();
            [t.clone(), t]
        }
    }
}

/// Build the walk/battle sim for a loaded scene per the CLI flags.
pub(super) fn make_walk(scene: &game::assets::Scene, b: &BattleSetup) -> WalkSim {
    let mut sim = core::sim::Sim::empty(b.seed);
    // The tier is set BEFORE `setup_conquest` spawns anyone: every bot takes
    // its skill at spawn (D2).
    sim.difficulty = b.difficulty;
    // BF1 has no 10 m omni in the view cone (RE §9.1); BF2 does.
    sim.omni_radius = core::sim::omni_radius_for(&scene.manifest.source_game);
    sim.water_level = scene.manifest.water.as_ref().map(|w| w.height);
    sim.ground_rules = core::soldier::GroundRules::for_game(&scene.manifest.source_game);
    sim.item_pool = game::roster::item_pool(&scene.manifest.source_game, &scene.manifest.map);
    sim.cheats = b.cheats;
    sim.battle = core::battle::Battle::new(match &b.mission {
        _ if b.tickets_set => [b.tickets, b.tickets],
        Some(m) => m
            .tickets()
            .map(|n| core::sim::scale_reinforcements(n, b.ticket_pct)),
        None => [core::sim::scale_reinforcements(b.tickets, b.ticket_pct); 2],
    });
    sim.time_limit = (b.time_limit > 0).then(|| b.time_limit * 60);
    if let Some(m) = &b.mission {
        sim.ai_alive_cap = m.ai_alive_cap;
        m.apply_death_drains(&mut sim.battle);
    }
    // --ai-cap wins over the mission, and applies with or without one.
    if let Some(cap) = b.ai_cap {
        sim.ai_alive_cap = cap;
    }
    let teams = team_templates(scene, b, &mut sim);
    let seat = usize::from(b.player_team.min(1));
    let mut walk = WalkSim::with_teams_seat(
        sim,
        scene.heightfield(),
        glam::Vec3::new(b.spawn.0, 0.0, b.spawn.1),
        teams,
        game::layout::nav_graph(&scene.manifest, &scene.dir),
        b.player_team,
    );
    // Model collision: the scene has already had its mode layer applied (see
    // `Scene::collision_world` — building it earlier misses every BF2 conquest
    // instance).
    scene.install_collision(&mut walk.sim);
    walk.camera_blockers = game::camera_collide::CameraBlockers::from_scene(scene);
    walk.fov_y_ps2 = game::walk::ps2_fov_y(&scene.manifest.source_game);
    if b.targets {
        walk.spawn_dummy_targets();
    }
    // --mission implies a conquest setup: AI counts from the mission unless
    // --ai overrides them.
    let ai_counts: Option<[u32; 2]> = match (b.ai, &b.mission) {
        (Some(n), _) => Some([n, n]),
        (None, Some(m)) => Some([m.initial_ai(0, seat == 0), m.initial_ai(1, seat == 1)]),
        (None, None) => None,
    };
    let mut vehicle_classes: Vec<String> = Vec::new();
    let mut layout = game::layout::MapLayout::from_manifest(&scene.manifest);
    // The mission owns the command-post roster, the posts it kills, the
    // ownership it re-authors and its death regions (game::layout).
    if let Some(m) = &b.mission {
        let a = layout.apply_mission(m);
        walk.sim.death_regions = a.death_regions.clone();
        let blocked = m.apply_arc_blocks(walk.sim.nav.as_mut());
        if blocked != 0 {
            println!("mission: route groups {blocked:#b} blocked (plnp)");
        }
        let switched = m.apply_barriers(walk.sim.nav.as_mut());
        if switched != 0 {
            println!("mission: {switched} barriers switched (Enable/DisableBarriers)");
        }
        println!(
            "mission: {} posts (roster dropped {}, killed {}), {} re-owned, {} death regions{}",
            a.kept,
            a.dropped_by_roster,
            a.killed,
            a.reassigned,
            a.death_regions.len(),
            if a.missing_death_regions.is_empty() {
                String::new()
            } else {
                format!(" ({} missing)", a.missing_death_regions.len())
            }
        );
    }
    if let Some(counts) = ai_counts {
        walk.add_posts(&layout);
        // The vehicle spawn pads follow the posts just added (card VPAD) and
        // go in before the opening wave, which spawns at the command walkers
        // they are about to build (card CWSP).
        let factions = [walk.teams[0].faction.clone(), walk.teams[1].faction.clone()];
        match game::vehicle_pads::install(
            &mut walk.sim,
            &scene.manifest,
            &scene.dir,
            &b.classes_dir,
            &factions,
        ) {
            Ok(p) => {
                println!(
                    "vehicle pads: {} ({} classes){}",
                    p.pads.len(),
                    p.classes.len(),
                    if p.absent.is_empty() {
                        String::new()
                    } else {
                        format!(", no class for {:?}", p.absent)
                    }
                );
                walk.vehicle_geometry = p.geometry;
                walk.vehicle_seats = p.seats;
                walk.vehicle_riders = p.riders;
                walk.vehicle_engines = p.engines;
                walk.vehicle_labels = p.labels;
                vehicle_classes = p.classes;
            }
            Err(e) => eprintln!("vehicle pads unavailable ({e:#})"),
        }
        walk.opening_wave(counts, !b.spawn_set);
        // The commander speaks the mission's lines over the posts and the
        // battle just set up.
        if let Some(m) = &b.mission {
            walk.sound_levels = m.sound_levels();
            walk.voice = Some(game::announce::Commander::for_battle(m, &layout, &walk));
            // ...and the music is the script's too, by the reinforcements
            // the teams start with.
            let tickets = [0, 1].map(|t| walk.sim.battle.teams[t].tickets());
            let bf2 = m.source_game == "bf2";
            walk.music = Some(game::music::BattleMusic::new(&m.music, bf2, tickets));
        }
        println!(
            "conquest: {} posts, the player on team {} ({}), AI {}/{}, tickets {}/{}, alive-AI cap {}, difficulty {}, omni {:.0} m",
            layout.cps.len(),
            seat + 1,
            walk.teams[seat].faction,
            counts[0],
            counts[1],
            walk.sim.battle.teams[0].tickets(),
            walk.sim.battle.teams[1].tickets(),
            walk.sim.ai_alive_cap,
            walk.sim.difficulty.label(),
            walk.sim.omni_radius,
        );
    } else if !b.spawn_set {
        // No conquest setup and no pinned spawn: the player is standing at
        // the authored default, which is off the walkable surface on seven
        // BF2 maps (`WalkSim::rescue_spawn`).
        if let Some((post, y)) = walk.rescue_spawn(&layout) {
            println!(
                "spawn: no ground at the authored point — moved to command post {post} (y {y:.2})"
            );
        }
    }
    walk.minimap = Some(game::minimap::MapStatic::load(
        scene,
        &b.classes_dir,
        &walk,
        &vehicle_classes,
        b.mission.as_ref().and_then(|m| m.era.as_deref()),
    ));
    walk
}

/// BF2's HUD from the bake at `assets_dir`, painted into `r`'s frames;
/// `None` — said once — when the bake holds none.
pub(super) fn load_hud_view(
    assets_dir: &std::path::Path,
    r: &render::Renderer,
) -> Option<game::hud_feed::HudView> {
    match game::hud_feed::HudView::for_renderer(assets_dir, r) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("hud: no BF2 HUD in the bake ({e:#}); bars and counts not drawn");
            None
        }
    }
}

/// HUD model from the battle state. Takes `feed`'s new event lines: each
/// model must reach the HUD (`HudView::update`) or its lines are lost.
pub(super) fn hud_model(
    walk: &WalkSim,
    feed: &mut game::event_feed::EventFeed,
) -> ui::hud::HudModel {
    let me = &walk.sim.units[walk.player.index as usize];
    let b = &walk.sim.battle;
    let on_foot = me.alive && me.mount.is_none();
    let weapon = on_foot.then(|| {
        let class = walk.sim.weapon_classes.get(me.weapon_class);
        let rounds = class.map_or(0, |c| c.rounds_per_clip);
        // The weapon in the hands: the primary, or the channel-0 slot the
        // player switched to (GREN).
        let hud = walk.teams[usize::from(me.team.min(1))]
            .class(me.spec.class)
            .hud_of(me.weapon_slot);
        ui::hud::HudWeapon {
            key: hud.key.clone(),
            class: hud.class.clone(),
            icon: hud.icon.clone(),
            // The localised weapon name waits for the Locl tables.
            name: String::new(),
            total: (rounds > 0).then_some(me.weapon.clip + me.weapon.spare),
            clip_fraction: if rounds > 0 {
                me.weapon.clip as f32 / rounds as f32
            } else {
                1.0
            },
        }
    });
    // The secondary channel's current weapon: the grenade row (GREN).
    let weapon2 = on_foot
        .then(|| me.cur1.and_then(|k| me.held.get(usize::from(k))))
        .flatten()
        .and_then(|w| {
            let class = walk.sim.weapon_classes.get(w.class)?;
            let hud = walk.teams[usize::from(me.team.min(1))]
                .class(me.spec.class)
                .hud_of(w.slot);
            Some(ui::hud::HudWeapon {
                key: hud.key.clone(),
                class: hud.class.clone(),
                icon: hud.icon.clone(),
                name: String::new(),
                total: w.state.rounds(class),
                clip_fraction: if class.rounds_per_clip > 0 {
                    w.state.clip as f32 / class.rounds_per_clip as f32
                } else {
                    1.0
                },
            })
        });
    let reticle = on_foot.then(|| {
        walk.sim
            .reticle_pick
            .first()
            .copied()
            .flatten()
            .map_or(0, |(_, rel)| rel)
    });
    let posts = |t: u8| walk.sim.cps.iter().filter(|c| c.owner == Some(t)).count();
    ui::hud::HudModel {
        team_names: [walk.teams[0].faction.clone(), walk.teams[1].faction.clone()],
        tickets: [b.teams[0].tickets(), b.teams[1].tickets()],
        bleed: [b.teams[0].bleed_per_sec, b.teams[1].bleed_per_sec],
        posts: [posts(0), posts(1)],
        hp: me.health.cur,
        hp_max: me.health.max,
        energy: me.soldier.energy,
        energy_max: me.consts.energy_bar,
        clip: me.weapon.clip,
        spare: me.weapon.spare,
        reloading: me.weapon.reloading(),
        weapon,
        weapon2,
        reticle,
        reticle_y: walk.reticle_y(),
        dead: !me.alive,
        respawn_in: (!me.alive).then(|| {
            let timer = me.respawn_at.map_or(0.0, |t| {
                t.saturating_sub(walk.sim.tick) as f32 / core::TICK_HZ as f32
            });
            let wave = me.spawn_slot.map_or(0.0, |s| {
                walk.sim.waves[usize::from(me.team).min(1)].countdown(s)
            });
            timer.max(wave)
        }),
        outcome: walk.sim.outcome,
        time_left: walk.sim.time_limit.map(|l| {
            (u64::from(l) * u64::from(core::TICK_HZ)).saturating_sub(walk.sim.tick) as f32
                / core::TICK_HZ as f32
        }),
        messages: feed.take(),
        stats: Some(feed.stats(&walk.sim, walk.player)),
        player_team: me.team,
        map: walk.minimap.as_ref().map(|m| m.model(walk)),
        squad: feed.squad(walk.sim.tick),
        capture: game::capture_feed::for_unit(
            &walk.sim,
            walk.player.index as usize,
            game::capture_feed::Rules::PORT,
        ),
    }
}

/// Spawn screen model while the player is dead and the round is live; None
/// otherwise. `selected` is the UI cursor — see `ui::spawn` on why it does not
/// steer the respawn yet.
pub(super) fn spawn_model(walk: &WalkSim, selected: usize) -> Option<ui::spawn::SpawnModel> {
    let me = &walk.sim.units[walk.player.index as usize];
    if me.alive || walk.sim.outcome.is_some() {
        return None;
    }
    let b = &walk.sim.battle;
    // The team's posts, then its live command walkers — each owns a post
    // the PS2's spawn map lists like any other, at the walker's live position
    // (card CWSP, re-command-walker-post §5); named by the class `Label`.
    let walkers = walk
        .sim
        .command_walkers(me.team)
        .map(|(_, v)| ui::spawn::SpawnPost {
            name: walk
                .vehicle_labels
                .get(v.class as usize)
                .and_then(Clone::clone)
                .unwrap_or_else(|| "Command walker".to_owned()),
            contested: false,
            distance: (v.pos - me.soldier.pos).length(),
        });
    let posts = walk
        .sim
        .cps
        .iter()
        .filter(|c| c.spawnable_by(me.team))
        .map(|c| ui::spawn::SpawnPost {
            name: c.def.name.clone(),
            // Enemies are neutralizing it right now.
            contested: c.neutral > 0.0,
            distance: (c.def.pos - me.soldier.pos).length(),
        })
        .chain(walkers)
        .collect();
    let countdown = {
        let timer = me.respawn_at.map_or(0.0, |t| {
            t.saturating_sub(walk.sim.tick) as f32 / core::TICK_HZ as f32
        });
        let wave = me.spawn_slot.map_or(0.0, |s| {
            walk.sim.waves[usize::from(me.team).min(1)].countdown(s)
        });
        let secs = timer.max(wave);
        (secs > 0.0).then_some(secs)
    };
    // The class row comes from the SIM: the labels are the team's table and
    // the selection is `Sim::spawn_choice` for this player's slot, which the
    // pad moves and the respawn reads (`ui::spawn`).
    let team = &walk.teams[usize::from(me.team.min(1))];
    let slot = me.spec.player_slot.map_or(0, usize::from);
    let has_table = !walk.sim.classes[usize::from(me.team.min(1))].is_empty();
    Some(ui::spawn::SpawnModel {
        team_names: [walk.teams[0].faction.clone(), walk.teams[1].faction.clone()],
        player_team: me.team,
        tickets: [b.teams[0].tickets(), b.teams[1].tickets()],
        posts,
        selected,
        classes: if has_table {
            team.classes.iter().map(|c| c.label.clone()).collect()
        } else {
            Vec::new()
        },
        selected_class: walk
            .sim
            .spawn_choice
            .get(slot)
            .map_or(0, |c| usize::from(*c)),
        countdown,
    })
}
