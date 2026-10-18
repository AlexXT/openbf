//! D14: the scripted squad-order log — FOLLOW ME by the idle player, then
//! HOLD POSITION (card SQACK: two follow, everyone within 30 m holds) — and
//! the Tier-4 capture it drives.
//!
//! ```text
//! cargo run --release -p game --example squad_log [out.ron]
//! ```
//!
//! It writes an `InputLog` of 6,000 idle pads (the player NEVER moves: the
//! point of the capture is that the squad comes to him and then stays put)
//! with exactly two presses — `SQUAD_FOLLOW` on tick 30 and `SQUAD_HOLD` on
//! tick 3,030 — and prints them. One tick each, because phase 0b takes the
//! EDGE: a held button is one order (`core::squad`, D2).
//!
//! The capture the log is for, once it exists:
//!
//! ```text
//! target/release/game --map assets_baked/tat1 --mode con \
//!     --mission missions/tat1_con.ron --headless --walk --hud \
//!     --walk-replay artifacts/squad/follow_hold.ron \
//!     --events <dir>/events.csv --capture-series 500,<dir>
//! ```
//!
//! `events.csv`'s two `command` rows carry the slot, the kind and how many
//! bots each order reached, and an `order_taken` row per unit that took it,
//! which is what makes the capture checkable rather than merely watchable.

use core::sim::{buttons, InputLog, PadState};
use std::path::PathBuf;

/// The tick each order is pressed on. Tick 30 gives the round one plan period
/// to place the bots before the first order; 3,030 is 100 s later, long enough
/// for the squad to have closed on the player and settled into formation.
const FOLLOW_TICK: usize = 30;
const HOLD_TICK: usize = 3_030;
const TICKS: usize = 6_000;

fn main() -> anyhow::Result<()> {
    let out = std::env::args().nth(1).map_or_else(
        || PathBuf::from("artifacts/squad/follow_hold.ron"),
        PathBuf::from,
    );
    let log = InputLog {
        seed: 42,
        slots: 1,
        ticks: (0..TICKS)
            .map(|t| PadState {
                buttons: match t {
                    FOLLOW_TICK => buttons::SQUAD_FOLLOW,
                    HOLD_TICK => buttons::SQUAD_HOLD,
                    _ => 0,
                },
                ..PadState::default()
            })
            .collect(),
    };
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        &out,
        ron::ser::to_string_pretty(&log, ron::ser::PrettyConfig::default())?,
    )?;
    println!(
        "{}: {TICKS} ticks, follow at {FOLLOW_TICK}, hold at {HOLD_TICK}",
        out.display()
    );
    Ok(())
}
