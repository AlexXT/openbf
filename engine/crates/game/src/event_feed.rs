//! The PS2's event messages (card EVMSG, `analysis/re-event-messages.md`):
//! the sim's events turned into the lines of BF2's `player1systemMessages`
//! and the numbers of its statistics pop-up, for ONE viewer — the local
//! player. `ui::hud::feed` sends them into the HUD tree.
//!
//! What makes a line (§1, §1.1 [V][O]) — neither game has a kill feed:
//!
//! - a soldier's death, only when the victim is a human: "%s killed %s",
//!   "%s killed %s - [TK]", "%s killed himself", and with no killer
//!   "%s died". An AI's death makes no line; the player's own kill shows as
//!   his statistics. (The PS2's hero clauses — a hero victim, a human or hero
//!   killer — have no unit behind them here: the sim has no heroes yet.)
//! - `[ours]` a post captured or lost, on every map: "%s captured the %s" /
//!   "%s lost the %s" — BF1's lines in BF2's own unused strings (§7.3:
//!   The maintainer's ask; the BF2 PS2 shows none — [`POST_LINES`] reverses it).
//!
//! The local player's squad order prints no line (the PS2 prints none): the
//! feed keeps the press for BF1's D-pad cross — its arm, the PS2's 2 s timer
//! on the sim's clock and `[ours]` the count of units the order changed
//! (card SQHUD, [`EventFeed::squad`], drawn by `ui::hud_squad`).
//!
//! Colours are the viewer's (§1.2 [V][M]): a kill line takes the VICTIM's
//! team as the viewer sees it, a post line the capturing / losing team's, a
//! death with no killer nobody's grey.
//!
//! The statistics (§5, §9): kills less two per team kill, and BF2's points
//! from its own table (PS2 RAM [M]) — see [`points`].
//!
//! Two rules carried over from the placeholder box (card SQMK):
//!
//! - **A tick's events are read once.** `Sim::events` holds the LAST stepped
//!   tick's events until the next step; a render frame that steps no tick
//!   must not read them again.
//! - **An identical line is one line** while the first is still up.

use core::sim::{Event, Sim};
use core::unit::UnitHandle;
use manifest::hud::HudLocl;
use std::path::Path;
use ui::hud::{message_color, HudMessage, HudStats};
use ui::hud_squad::{self, SquadCross};

/// `[ours]` Post lines on every map. The BF2 PS2 shows none (its events 4
/// and 5 only speak, §1 [V][O]); BF1 shows both. `false` is the BF2 disc.
pub const POST_LINES: bool = true;

/// How long an identical line is held back: a line's life on screen (0.5 s
/// in, 3 s held, 0.5 s out).
const SAME_LINE_TICKS: u64 = 4 * core::TICK_HZ as u64;

/// BF2's points per event (§9: the table at 0x5fedf0, filled at run time by
/// `ScriptCB_SetPlayerStatsPoints`, read from two HOT1 save states [M]; the
/// events by their call sites [V]). Not built: the 2-point kills beside a
/// post (events 0x17, 0x18, 0x1b, 0x1d, 0x1e) and the hero's 5.
pub mod points {
    /// Event 0: killing an AI soldier (`FUN_00362268`).
    pub const KILL_AI: i32 = 1;
    /// Event 1: killing a human.
    pub const KILL_HUMAN: i32 = 2;
    /// Event 3: killing oneself (`FUN_00362c60`), or dying with no killer
    /// and no killing team (`FUN_0034f650`).
    pub const SUICIDE: i32 = -1;
    /// Event 4: killing a teammate.
    pub const TEAM_KILL: i32 = -2;
    /// Event 0x15: capturing a post as the unit longest in its capture
    /// region (`FUN_00357440` → `FUN_00363a30`).
    pub const CAPTURE: i32 = 5;
    /// Event 0x16: standing in the capture region when another captures it.
    pub const CAPTURE_ASSIST: i32 = 2;
}

/// The English lines the feed is built from, by BF2's own Locl hashes (§1's
/// table), with the disc's text as the fallback when no bake is at hand.
#[derive(Debug, Clone)]
pub struct Texts {
    /// BF2's table: the lines, the team names, the soldier names.
    bf2: Option<HudLocl>,
    /// The map's own game's table: its post names.
    map: Option<HudLocl>,
    /// The map is a BF1 map: its names are upper case on the disc.
    bf1_map: bool,
}

const KILLED: (u32, &str) = (0xacc1_eef7, "%s killed %s");
const TEAM_KILLED: (u32, &str) = (0xc57a_e533, "%s killed %s - [TK]");
const SUICIDE: (u32, &str) = (0x6163_33ea, "%s killed himself");
const DIED: (u32, &str) = (0x0a9f_4178, "%s died");
const CAPTURED: (u32, &str) = (0xdd27_9cfc, "%s captured the %s");
const LOST: (u32, &str) = (0xae1e_c790, "%s lost the %s");
const COMMAND_POST: (u32, &str) = (0x7e64_bc51, "Command Post");

fn read_locl(dir: &Path) -> Option<HudLocl> {
    let text = pak::read_to_string(dir.join("locl/english.ron")).ok()?;
    ron::from_str(&text).ok()
}

/// `%s` filled left to right.
fn fill(template: &str, args: &[&str]) -> String {
    let mut out = String::new();
    let mut args = args.iter();
    let mut rest = template;
    while let Some(at) = rest.find("%s") {
        out.push_str(&rest[..at]);
        out.push_str(args.next().copied().unwrap_or(""));
        rest = &rest[at + 2..];
    }
    out.push_str(rest);
    out
}

/// "PIER ONE" → "Pier One": a BF1 name in BF2's case. Words of one or two
/// letters keep theirs ("LZ").
fn title_case(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) if w.chars().count() > 2 => {
                    first.to_uppercase().collect::<String>() + &c.as_str().to_lowercase()
                }
                _ => w.to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl Texts {
    /// The tables `bake locl` wrote under `assets_dir`, for a map of
    /// `source_game`. Missing tables fall back to the built-in lines.
    pub fn load(assets_dir: &Path, source_game: &str) -> Self {
        let bf1_map = source_game.eq_ignore_ascii_case("bf1");
        let bf2 = read_locl(&assets_dir.join("common_bf2"));
        if bf2.is_none() {
            eprintln!("hud: no baked text table (bake common); names fall back");
        }
        let map = if bf1_map {
            read_locl(&assets_dir.join("common_bf1"))
        } else {
            bf2.clone()
        };
        Self { bf2, map, bf1_map }
    }

    /// Only the built-in lines (tests, a bake with no text).
    pub fn builtin() -> Self {
        Self {
            bf2: None,
            map: None,
            bf1_map: false,
        }
    }

    /// An order's name on the cross: BF1's own table on a BF1 map, else
    /// BF2's, else the disc's English.
    fn order_label(&self, (key, fallback): (&str, &str)) -> String {
        let hash = manifest::hud::locl_key(key);
        [&self.map, &self.bf2]
            .into_iter()
            .flatten()
            .find_map(|l| l.get(hash))
            .unwrap_or(fallback)
            .to_owned()
    }

    fn line(&self, (hash, fallback): (u32, &str)) -> String {
        self.bf2
            .as_ref()
            .and_then(|l| l.get(hash))
            .unwrap_or(fallback)
            .to_owned()
    }

    /// A team's name: `common.sides.<faction>.name` ("Rebels", "Empire").
    pub fn team_name(&self, faction: &str) -> String {
        self.bf2
            .as_ref()
            .and_then(|l| l.text(&format!("common.sides.{faction}.name")))
            .filter(|t| !t.trim().is_empty())
            .map_or_else(|| faction.to_uppercase(), str::to_owned)
    }

    /// A post's name by its key path (`level.<map>.<label>`: [D] the path
    /// that names 227 of the discs' 248 posts, and BF1 KAS2's CP5 "PIER ONE"
    /// as the PS2 printed it), else "Command Post" (the PS2's own fallback,
    /// §1 [V]).
    pub fn post_name(&self, key: &str) -> String {
        let named = self
            .map
            .as_ref()
            .and_then(|l| l.text(key))
            .map(|t| t.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|t| !t.is_empty());
        match named {
            Some(n) if self.bf1_map => title_case(&n),
            Some(n) => n,
            None => self.line(COMMAND_POST),
        }
    }

    /// The side's name lists, `<side>.firstnames.01` … and `.lastnames.01` …
    /// (§6: up to 64 each).
    fn name_lists(&self, faction: &str) -> (Vec<String>, Vec<String>) {
        let list = |kind: &str| -> Vec<String> {
            let Some(l) = &self.bf2 else {
                return Vec::new();
            };
            (1..=64)
                .map_while(|n| l.text(&format!("common.sides.{faction}.{kind}.{n:02}")))
                .map(str::to_owned)
                .collect()
        };
        (list("firstnames"), list("lastnames"))
    }
}

/// A small fixed hash: the name draws must not touch the sim's RNG.
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// An AI soldier's name, as the PS2 composes them (§6, PS2 RAM [M], BF2
/// HOT1's 37 AI records and BF1 KAS2's lines): a side with first and last
/// names "Ben Pulastra"; with first names only "Unit 688"; with neither two
/// letters and a number, "DX 178". PROVISIONAL (ours): which name a unit
/// draws — the PS2 draws from its own RNG — and a first-name-only side with
/// more than one name (the Republic's "Alpha" …, not seen on the PS2).
fn ai_name(first: &[String], last: &[String], unit: u32) -> String {
    let h = mix(u64::from(unit));
    let pick = |list: &[String], salt: u64| -> String {
        list[(mix(h ^ salt) % list.len() as u64) as usize].clone()
    };
    let number = (h >> 20) % 1000;
    match (first.is_empty(), last.is_empty()) {
        (false, false) => format!("{} {}", pick(first, 1), pick(last, 2)),
        (false, true) => format!("{} {number:03}", pick(first, 1)),
        _ => {
            let letter = |salt: u64| (b'A' + (mix(h ^ salt) % 26) as u8) as char;
            format!("{}{} {number:03}", letter(3), letter(4))
        }
    }
}

/// The four field order names of BF1's D-pad cross by their Locl keys, with
/// the disc's English as the fallback: up, down, right, left
/// (`re-squad-orders.md` §2).
const ORDER_LABELS: [(&str, &str); 4] = [
    ("game.squadcommand.field.up", "MOVE OUT"),
    ("game.squadcommand.field.down", "HOLD POSITION"),
    ("game.squadcommand.field.right", "FOLLOW ME"),
    ("game.squadcommand.field.left", "AT EASE"),
];

/// The cross's arm for `Event::Command`'s `kind` byte (0 follow, 1 hold,
/// 2 at ease, 3 move out).
pub fn order_arm(kind: u8) -> u8 {
    match kind {
        0 => hud_squad::RIGHT,
        1 => hud_squad::DOWN,
        2 => hud_squad::LEFT,
        _ => hud_squad::UP,
    }
}

/// The local player's last order, for the cross.
#[derive(Debug, Clone, Copy)]
struct Press {
    tick: u64,
    kind: u8,
    /// `[ours]` the units it changed so far (`Event::OrderTaken`'s `did`).
    took: u32,
}

/// One viewer's event lines and statistics.
#[derive(Debug)]
pub struct EventFeed {
    texts: Texts,
    /// Per team: its name and its soldiers' name lists.
    teams: [(String, Vec<String>, Vec<String>); 2],
    /// Per post (`Sim::cps` order): its name.
    posts: Vec<String>,
    /// The lines made since the last [`EventFeed::take`], oldest first.
    out: Vec<HudMessage>,
    /// What was said lately, and when (the one-line rule).
    recent: Vec<(String, u64)>,
    /// The tick whose events were already read.
    seen: Option<u64>,
    /// Per post: its owner when the last tick was read (who loses it).
    owners: Vec<Option<u8>>,
    /// Per unit: the post whose capture region it stands in, and since when.
    inside: Vec<Option<(u32, u64)>>,
    points: i32,
    kills: i32,
    captures: i32,
    /// The four order names, up / down / right / left.
    order_labels: [String; 4],
    order: Option<Press>,
}

impl Default for EventFeed {
    fn default() -> Self {
        Self::new(Texts::builtin(), ["", ""], &[])
    }
}

impl EventFeed {
    /// A feed for a battle of `factions` over posts with the key paths
    /// `post_keys` (`Sim::cps` order).
    pub fn new(texts: Texts, factions: [&str; 2], post_keys: &[String]) -> Self {
        let team = |f: &str| {
            let (first, last) = texts.name_lists(f);
            (texts.team_name(f), first, last)
        };
        Self {
            teams: [team(factions[0]), team(factions[1])],
            posts: post_keys.iter().map(|k| texts.post_name(k)).collect(),
            order_labels: ORDER_LABELS.map(|l| texts.order_label(l)),
            texts,
            out: Vec::new(),
            recent: Vec::new(),
            seen: None,
            owners: Vec::new(),
            inside: Vec::new(),
            points: 0,
            kills: 0,
            captures: 0,
            order: None,
        }
    }

    /// The feed of `walk`'s battle, with the text baked under `assets_dir`.
    pub fn for_walk(assets_dir: &Path, source_game: &str, walk: &crate::walk::WalkSim) -> Self {
        Self::new(
            Texts::load(assets_dir, source_game),
            [&walk.teams[0].faction, &walk.teams[1].faction],
            &walk.post_keys,
        )
    }

    /// The two teams' names as the lines print them.
    pub fn team_names(&self) -> [&str; 2] {
        [&self.teams[0].0, &self.teams[1].0]
    }

    /// The posts' names as the lines print them, in `Sim::cps` order.
    pub fn post_names(&self) -> &[String] {
        &self.posts
    }

    /// The name a line gives unit `h`: "Player 1" for a human (the PS2's
    /// `Player %d`), a composed one for an AI.
    pub fn unit_name(&self, sim: &Sim, h: UnitHandle) -> String {
        let u = &sim.units[h.index as usize];
        match u.spec.player_slot {
            Some(slot) => format!("Player {}", u32::from(slot) + 1),
            None => {
                let (_, first, last) = &self.teams[usize::from(u.team.min(1))];
                ai_name(first, last, h.index)
            }
        }
    }

    fn post_name(&self, cp: u32, sim: &Sim) -> String {
        self.posts
            .get(cp as usize)
            .cloned()
            .unwrap_or_else(|| sim.cps[cp as usize].def.name.clone())
    }

    /// The squad order cross at sim tick `tick`: the last order's arm, the
    /// PS2's timer on the sim's clock and `[ours]` the count of units the
    /// order changed (none while that is 0); `None` once it has faded.
    pub fn squad(&self, tick: u64) -> Option<SquadCross> {
        let o = self.order?;
        let seconds = tick.saturating_sub(o.tick) as f32 / core::TICK_HZ as f32;
        Some(SquadCross {
            labels: self.order_labels.clone(),
            arm: order_arm(o.kind),
            count: (o.took > 0).then_some(o.took),
            timer: SquadCross::timer_at(seconds)?,
        })
    }

    /// Queue `text` unless the same line is still up.
    pub fn say(&mut self, text: String, color: [u8; 3], tick: u64) {
        self.recent
            .retain(|(_, t)| tick.saturating_sub(*t) < SAME_LINE_TICKS);
        if self.recent.iter().any(|(l, _)| *l == text) {
            return;
        }
        self.recent.push((text.clone(), tick));
        self.out.push(HudMessage { text, color });
    }

    /// Read the sim's events if its tick is new. `viewer` is the local
    /// player: the lines are coloured for his team and the statistics are
    /// his. Call once per stepped tick (and freely more often).
    pub fn tick(&mut self, sim: &Sim, viewer: UnitHandle) {
        let tick = sim.tick;
        if self.seen == Some(tick) {
            return;
        }
        self.seen = Some(tick);
        let my_team = sim.units[viewer.index as usize].team;
        // A team as the viewer's team sees it (§1.2).
        let relation = |team: u8| {
            if team == my_team {
                message_color::OWN
            } else {
                message_color::ENEMY
            }
        };
        self.note_capture_regions(sim);

        for e in &sim.events {
            match *e {
                Event::Kill { shooter, victim } => {
                    let (k, v) = (
                        &sim.units[shooter.index as usize],
                        &sim.units[victim.index as usize],
                    );
                    // The gate (§1.1): only a human's death makes a line.
                    if v.is_player {
                        let (kn, vn) = (self.unit_name(sim, shooter), self.unit_name(sim, victim));
                        let text = if shooter == victim {
                            fill(&self.texts.line(SUICIDE), &[&vn])
                        } else if k.team == v.team {
                            fill(&self.texts.line(TEAM_KILLED), &[&kn, &vn])
                        } else {
                            fill(&self.texts.line(KILLED), &[&kn, &vn])
                        };
                        self.say(text, relation(v.team), tick);
                    }
                    if shooter == viewer {
                        if shooter == victim {
                            self.kills -= 1;
                            self.points += points::SUICIDE;
                        } else if k.team == v.team {
                            self.kills -= 1;
                            self.points += points::TEAM_KILL;
                        } else {
                            self.kills += 1;
                            self.points += if v.is_player {
                                points::KILL_HUMAN
                            } else {
                                points::KILL_AI
                            };
                        }
                    }
                }
                Event::EnvDeath { unit, .. } => {
                    if sim.units[unit.index as usize].is_player {
                        let name = self.unit_name(sim, unit);
                        let text = fill(&self.texts.line(DIED), &[&name]);
                        self.say(text, message_color::NEUTRAL, tick);
                    }
                    if unit == viewer {
                        self.points += points::SUICIDE;
                    }
                }
                Event::Captured { cp, team } => {
                    if POST_LINES {
                        let text = fill(
                            &self.texts.line(CAPTURED),
                            &[
                                &self.teams[usize::from(team.min(1))].0,
                                &self.post_name(cp, sim),
                            ],
                        );
                        self.say(text, relation(team), tick);
                    }
                    if let Some(first) = self.capture_credit(sim, viewer, cp, team) {
                        self.captures += 1;
                        self.points += if first {
                            points::CAPTURE
                        } else {
                            points::CAPTURE_ASSIST
                        };
                    }
                }
                Event::Neutralized { cp, by } => {
                    if POST_LINES {
                        // The team that held it when the last tick was read.
                        let loser = self
                            .owners
                            .get(cp as usize)
                            .copied()
                            .flatten()
                            .unwrap_or(1 - by.min(1));
                        let text = fill(
                            &self.texts.line(LOST),
                            &[
                                &self.teams[usize::from(loser.min(1))].0,
                                &self.post_name(cp, sim),
                            ],
                        );
                        self.say(text, relation(loser), tick);
                    }
                }
                // The LOCAL player's own orders, and only theirs — slot 0 is
                // the local player everywhere in this binary. The press shows
                // the cross (card SQHUD); the PS2 prints no line.
                Event::Command { slot: 0, kind, .. } => {
                    self.order = Some(Press {
                        tick,
                        kind,
                        took: 0,
                    });
                }
                Event::OrderTaken {
                    slot: 0, did: true, ..
                } => {
                    if let Some(o) = &mut self.order {
                        o.took += 1;
                    }
                }
                _ => {}
            }
        }
        self.owners = sim.cps.iter().map(|c| c.owner).collect();
    }

    /// Keep, per unit, the post whose capture region it stands in and the
    /// tick it entered (the PS2 keeps the same pair on the player, and the
    /// capture's 5 points go to the one longest in, §9).
    fn note_capture_regions(&mut self, sim: &Sim) {
        self.inside.resize(sim.units.len(), None);
        for (i, u) in sim.units.iter().enumerate() {
            let at = (u.alive && u.capture_posts)
                .then(|| {
                    sim.cps
                        .iter()
                        .position(|c| c.def.capture.contains(u.soldier.pos))
                })
                .flatten()
                .map(|c| c as u32);
            self.inside[i] = match (at, self.inside[i]) {
                (Some(cp), Some((was, since))) if cp == was => Some((cp, since)),
                (Some(cp), _) => Some((cp, sim.tick)),
                (None, _) => None,
            };
        }
    }

    /// `None`: the viewer had no part in `team`'s capture of `cp`.
    /// `Some(true)`: nobody of that team was in the region longer.
    fn capture_credit(&self, sim: &Sim, viewer: UnitHandle, cp: u32, team: u8) -> Option<bool> {
        let me = &sim.units[viewer.index as usize];
        let (_, since) = self
            .inside
            .get(viewer.index as usize)
            .copied()
            .flatten()
            .filter(|(at, _)| *at == cp && me.team == team)?;
        let longer = sim.units.iter().enumerate().any(|(i, u)| {
            u.team == team
                && i != viewer.index as usize
                && matches!(self.inside.get(i), Some(Some((at, s))) if *at == cp && *s < since)
        });
        Some(!longer)
    }

    /// The lines made since the last call, oldest first.
    pub fn take(&mut self) -> Vec<HudMessage> {
        std::mem::take(&mut self.out)
    }

    /// The viewer's statistics as BF2 publishes them (§5).
    pub fn stats(&self, sim: &Sim, viewer: UnitHandle) -> HudStats {
        HudStats {
            points: self.points,
            kills: self.kills,
            deaths: sim.units[viewer.index as usize].deaths as i32,
            hero_points: 0,
            captures: self.captures,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cp::CpDef;
    use core::region::Region;
    use core::sim::{buttons, DeathCause, PadState};
    use core::unit::{Controller, UnitSpec};
    use glam::Vec3;

    /// BF2's own lines and names, as `bake locl` writes them.
    fn texts() -> Texts {
        let mut entries: Vec<(u32, String)> = [KILLED, TEAM_KILLED, SUICIDE, DIED, CAPTURED, LOST]
            .iter()
            .map(|(h, t)| (*h, t.to_string()))
            .collect();
        for (key, text) in [
            ("common.sides.all.name", "Rebels"),
            ("common.sides.imp.name", "Empire"),
            ("common.sides.all.firstnames.01", "Ben"),
            ("common.sides.all.lastnames.01", "Pulastra"),
            ("level.hoth.cp5", "Echo Base"),
        ] {
            entries.push((manifest::hud::locl_key(key), text.to_string()));
        }
        entries.sort_by_key(|(h, _)| *h);
        let table = HudLocl {
            language: "english".into(),
            entries,
        };
        Texts {
            bf2: Some(table.clone()),
            map: Some(table),
            bf1_map: false,
        }
    }

    /// The Empire's player (team 1) with an AI beside him, two Rebel AI
    /// (team 0), and a post the Empire holds.
    struct Scene {
        sim: Sim,
        me: UnitHandle,
        mate: UnitHandle,
        rebels: [UnitHandle; 2],
        feed: EventFeed,
    }

    fn scene() -> Scene {
        let mut sim = Sim::empty(3);
        sim.heightfield = Some(core::heightfield::Heightfield {
            size: 2,
            unit: 400.0,
            offset_x: 200.0,
            offset_z: 200.0,
            heights: vec![0.0; 4],
        });
        let me = sim
            .spawn(&UnitSpec::player_in_slot(0, 1, Vec3::ZERO))
            .expect("player");
        let ai = |sim: &mut Sim, team: u8, x: f32| {
            sim.spawn(&UnitSpec {
                controller: Controller::Ai,
                ..UnitSpec::trooper(team, Vec3::new(x, 0.0, 0.0))
            })
            .expect("bot")
        };
        let mate = ai(&mut sim, 1, 2.0);
        let rebels = [ai(&mut sim, 0, 40.0), ai(&mut sim, 0, 42.0)];
        let (capture_time, neutralize_time) = CpDef::class_times(true);
        sim.add_cp(
            CpDef {
                name: "cp5".into(),
                pos: Vec3::new(100.0, 0.0, 0.0),
                capture: Region::sphere(Vec3::new(100.0, 0.0, 0.0), 8.0),
                control: None,
                value_bleed: 10,
                value_atk: [10; 2],
                value_def: [10; 2],
                spawn_nodes: vec![(Vec3::new(100.0, 0.0, 0.0), 0.0)],
                capture_time,
                neutralize_time,
            },
            Some(1),
        );
        let feed = EventFeed::new(texts(), ["all", "imp"], &["level.hoth.cp5".to_string()]);
        Scene {
            sim,
            me,
            mate,
            rebels,
            feed,
        }
    }

    /// One tick whose events are exactly `events`.
    fn happen(s: &mut Scene, events: &[Event]) -> Vec<HudMessage> {
        s.sim.tick += 1;
        s.sim.events = events.to_vec();
        s.feed.tick(&s.sim, s.me);
        s.feed.take()
    }

    fn line(text: &str, color: [u8; 3]) -> HudMessage {
        HudMessage {
            text: text.into(),
            color,
        }
    }

    /// Card EVMSG's gate (`re-event-messages.md` §1.1): only a human's death
    /// makes a line, coloured by the victim's team as the viewer sees it;
    /// the player's own kill is his statistics.
    #[test]
    fn only_a_humans_death_makes_a_line() {
        let mut s = scene();
        let [r0, r1] = s.rebels;
        let (me, mate) = (s.me, s.mate);
        // AI kills AI, either way round: nothing.
        let kill = |shooter, victim| Event::Kill { shooter, victim };
        assert!(happen(&mut s, &[kill(r0, mate), kill(mate, r1)]).is_empty());
        assert_eq!(s.feed.stats(&s.sim, me), HudStats::default());

        // An AI kills the human: the line, in the viewer's own-team colour.
        assert_eq!(
            happen(&mut s, &[kill(r0, me)]),
            [line("Ben Pulastra killed Player 1", [1, 86, 213])]
        );
        // The Empire has no name lists: two letters and a number.
        let name = s.feed.unit_name(&s.sim, mate);
        let b = name.as_bytes();
        assert!(
            name.len() == 6
                && b[..2].iter().all(u8::is_ascii_uppercase)
                && b[2] == b' '
                && b[3..].iter().all(u8::is_ascii_digit),
            "{name:?}"
        );

        // The human kills an ordinary AI: no line, kills +1 and a point.
        assert!(happen(&mut s, &[kill(me, r1)]).is_empty());
        let st = s.feed.stats(&s.sim, me);
        assert_eq!((st.kills, st.points), (1, points::KILL_AI));

        // A human suicide: the line, kills −1, a point lost.
        assert_eq!(
            happen(&mut s, &[kill(me, me)]),
            [line("Player 1 killed himself", [1, 86, 213])]
        );
        let st = s.feed.stats(&s.sim, me);
        assert_eq!((st.kills, st.points), (0, 0));

        // A teammate killed by the human: no line (the victim is an AI),
        // kills −1, two points lost.
        assert!(happen(&mut s, &[kill(me, mate)]).is_empty());
        let st = s.feed.stats(&s.sim, me);
        assert_eq!((st.kills, st.points), (-1, -2));

        // Dying with no killer: "died", nobody's grey, a point lost.
        let fell = Event::EnvDeath {
            unit: me,
            cause: DeathCause::Fall,
        };
        assert_eq!(
            happen(&mut s, &[fell]),
            [line("Player 1 died", [192, 192, 192])]
        );
        assert_eq!(s.feed.stats(&s.sim, me).points, -3);
        // An AI's fall: nothing.
        let fell = Event::EnvDeath {
            unit: r0,
            cause: DeathCause::Fall,
        };
        assert!(happen(&mut s, &[fell]).is_empty());
    }

    /// `[ours]` post lines in BF2's own strings, coloured by the team that
    /// captured or lost the post; standing in the region longest earns the
    /// capture's 5 points, standing in it at all its 2.
    #[test]
    fn post_lines_name_the_team_and_the_post() {
        let mut s = scene();
        let me = s.me;
        // The Rebels take the Empire's post: lost (the viewer's team, blue),
        // then captured (the enemy, red).
        assert_eq!(
            happen(&mut s, &[Event::Neutralized { cp: 0, by: 0 }]),
            [line("Empire lost the Echo Base", [1, 86, 213])]
        );
        s.sim.cps[0].owner = None;
        assert_eq!(
            happen(&mut s, &[Event::Captured { cp: 0, team: 0 }]),
            [line("Rebels captured the Echo Base", [200, 100, 100])]
        );
        assert_eq!(s.feed.stats(&s.sim, me).captures, 0);

        // The Empire takes it back with the player in the region first and
        // a teammate arriving later: his capture, 5 points.
        s.sim.cps[0].owner = Some(0);
        let at_post = Vec3::new(100.0, 0.0, 0.0);
        s.sim.units[me.index as usize].soldier.pos = at_post;
        assert!(happen(&mut s, &[]).is_empty());
        let mate = s.mate;
        s.sim.units[mate.index as usize].soldier.pos = at_post;
        assert_eq!(
            happen(&mut s, &[Event::Neutralized { cp: 0, by: 1 }]),
            [line("Rebels lost the Echo Base", [200, 100, 100])]
        );
        assert_eq!(
            happen(&mut s, &[Event::Captured { cp: 0, team: 1 }]),
            [line("Empire captured the Echo Base", [1, 86, 213])]
        );
        let st = s.feed.stats(&s.sim, me);
        assert_eq!((st.captures, st.points), (1, points::CAPTURE));

        // Arriving second is an assist.
        s.sim.units[me.index as usize].soldier.pos = Vec3::ZERO;
        assert!(happen(&mut s, &[]).is_empty());
        s.sim.units[me.index as usize].soldier.pos = at_post;
        for _ in 0..SAME_LINE_TICKS {
            assert!(happen(&mut s, &[]).is_empty());
        }
        happen(&mut s, &[Event::Captured { cp: 0, team: 1 }]);
        let st = s.feed.stats(&s.sim, me);
        assert_eq!(
            (st.captures, st.points),
            (2, points::CAPTURE + points::CAPTURE_ASSIST)
        );
    }

    /// A post with no name on the disc is the PS2's "Command Post", and a
    /// BF1 name is set in BF2's case.
    #[test]
    fn post_and_team_names_fall_back() {
        let t = texts();
        assert_eq!(t.post_name("level.hoth.cp5"), "Echo Base");
        assert_eq!(t.post_name("level.hoth.cp9"), "Command Post");
        assert_eq!(t.team_name("gun"), "GUN");
        assert_eq!(title_case("PIER ONE"), "Pier One");
        assert_eq!(title_case("WESTERN LZ"), "Western LZ");
        assert_eq!(
            fill("%s lost the %s", &["Empire", "Bunker"]),
            "Empire lost the Bunker"
        );
        // The three ways the PS2 composes an AI's name.
        let (first, last) = (vec!["Unit".to_string()], vec!["Farelle".to_string()]);
        assert_eq!(ai_name(&first, &last, 7), "Unit Farelle");
        assert!(ai_name(&first, &[], 7).starts_with("Unit "));
        assert_eq!(ai_name(&[], &[], 7).len(), 6);
        assert_eq!(
            ai_name(&[], &[], 7),
            ai_name(&[], &[], 7),
            "a unit keeps its name"
        );
    }

    fn pad(buttons: u32) -> PadState {
        PadState {
            buttons,
            ..PadState::default()
        }
    }

    /// A battle of the player and `mates` bots of his team in a 5 m ring.
    fn squad_scene(mates: usize) -> (Sim, UnitHandle, EventFeed) {
        let mut s = scene();
        for h in s.rebels {
            s.sim.units[h.index as usize].alive = false;
        }
        s.sim.units[s.mate.index as usize].alive = false;
        for k in 0..mates {
            let a = k as f32 / mates as f32 * std::f32::consts::TAU;
            s.sim
                .spawn(&UnitSpec {
                    controller: Controller::Ai,
                    ..UnitSpec::trooper(1, Vec3::new(5.0 * a.cos(), 0.0, 5.0 * a.sin()))
                })
                .expect("bot");
        }
        (s.sim, s.me, s.feed)
    }

    /// Press `button`, run a second of ticks (every bot has thought), and
    /// answer the cross then.
    fn order(sim: &mut Sim, me: UnitHandle, feed: &mut EventFeed, button: u32) -> SquadCross {
        sim.step(&[pad(0)]);
        feed.tick(sim, me);
        sim.step(&[pad(button)]);
        feed.tick(sim, me);
        for _ in 0..core::TICK_HZ {
            sim.step(&[pad(0)]);
            feed.tick(sim, me);
        }
        assert!(feed.take().is_empty(), "an order prints no line");
        feed.squad(sim.tick).expect("the cross is up a second on")
    }

    /// Card SQHUD: the press shows the cross — its arm, the PS2's 2 s timer
    /// on the sim's clock — and prints no line, however many render frames
    /// look at the tick.
    #[test]
    fn an_order_shows_the_cross_and_no_line() {
        let (mut sim, me, mut feed) = squad_scene(0);
        feed.tick(&sim, me);
        assert!(feed.squad(sim.tick).is_none());
        sim.step(&[pad(buttons::SQUAD_FOLLOW)]);
        for _ in 0..3 {
            feed.tick(&sim, me);
        }
        assert!(feed.take().is_empty());
        let at = sim.tick;
        let hz = u64::from(core::TICK_HZ);
        let c = feed.squad(at).expect("shown at the press");
        assert_eq!((c.arm, c.timer, c.count), (hud_squad::RIGHT, 1.0, None));
        assert_eq!(
            c.labels,
            ["MOVE OUT", "HOLD POSITION", "FOLLOW ME", "AT EASE"]
        );
        assert_eq!(feed.squad(at + hz).map(|c| c.timer), Some(0.5));
        assert_eq!(feed.squad(at + 2 * hz).map(|c| c.timer), Some(0.0));
        assert!(feed.squad(at + 2 * hz + 1).is_none(), "gone after 2 s");
        // Each order has its own arm.
        for (button, arm) in [
            (buttons::SQUAD_HOLD, hud_squad::DOWN),
            (buttons::SQUAD_AT_EASE, hud_squad::LEFT),
            (buttons::SQUAD_MOVE_OUT, hud_squad::UP),
        ] {
            assert_eq!(order(&mut sim, me, &mut feed, button).arm, arm);
        }
    }

    /// Card SQHUD's count (the maintainer, INBOX 2026-09-30): the units the order
    /// changed. FOLLOW ME with five in reach shows 2, HOLD POSITION with
    /// three shows 3, a release the number released, nobody no number.
    #[test]
    fn the_cross_counts_the_units_that_took_the_order() {
        let (mut sim, me, mut feed) = squad_scene(5);
        let c = order(&mut sim, me, &mut feed, buttons::SQUAD_FOLLOW);
        assert_eq!(c.count, Some(2), "FOLLOW ME, five in reach");
        let c = order(&mut sim, me, &mut feed, buttons::SQUAD_MOVE_OUT);
        assert_eq!(c.count, Some(2), "MOVE OUT releases the two followers");
        let c = order(&mut sim, me, &mut feed, buttons::SQUAD_AT_EASE);
        assert_eq!(c.count, None, "nobody left to release");

        let (mut sim, me, mut feed) = squad_scene(3);
        let c = order(&mut sim, me, &mut feed, buttons::SQUAD_HOLD);
        assert_eq!(c.count, Some(3), "HOLD POSITION, three in reach");

        let (mut sim, me, mut feed) = squad_scene(0);
        for button in [buttons::SQUAD_FOLLOW, buttons::SQUAD_HOLD] {
            let c = order(&mut sim, me, &mut feed, button);
            assert_eq!(c.count, None, "nobody in reach");
        }
    }
}
