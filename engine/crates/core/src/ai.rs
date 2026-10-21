//! Infantry AI: Phase A — navigation on the authored planning graphs (PLAN Log
//! "PHASE A DESIGN"). Everything that is not the graph is still the P4
//! PLACEHOLDER, and every constant in this module is PROVISIONAL and ours —
//! not fidelity. It drives the SAME soldier + weapon code as the player
//! through a synthesized pad, so movement and accuracy rules are shared; the
//! AI spread floor (0, no free-shot buffer) lives in `core::weapon`; the AI
//! sniper distance table is `weapon::ai_sniper_yaw`, gated by
//! `WeaponClass::ai_distance_table`.
//!
//! Behaviour:
//!
//! - PERCEIVE (Phase C, `crate::perception`): a bounded memory of six
//!   contacts per unit, filled by the PS2's view cone and view distance
//!   through the ordnance LOS ray and by hearing, and a target chosen out of
//!   THAT memory — nearest-first with the originals' hysteresis. Aim at the
//!   flat-fire lead point of a SEEN target (at the remembered point of one
//!   that was lost), fire once the tier's reaction time has passed, and
//!   FIGHT a seen target inside the weapon's range from a position of the
//!   bot's own fixed style (card FGHT, `behave::fight_position`: half of
//!   them stand to shoot, spaced out, half circle on a ring round it);
//! - otherwise FOLLOW A ROUTE over `core::nav`: the nearest command post not
//!   owned by my team that the graph can actually reach, hop by hop through
//!   the authored hubs, the post's own capture volume being the last leg. With
//!   no reachable post the unit HUNTS the nearest enemy's hub; with no graph
//!   at all it walks straight at the post, exactly as it did before Phase A.
//!
//! Phase A.1 adds the LOCAL layer under that route (PLAN Log "PHASE A.1
//! DESIGN" and its "A.1 addendum"), in two parts:
//!
//! - the CORNER WALK, which is the PS2's own: every leg is tested against the
//!   authored `BARR` barriers that stop the unit's type and, when it crosses
//!   one, [`crate::nav::NavGraph::detour`] returns the intermediate corners to
//!   walk instead. Barriers never touch the routing tables.
//! - the WALL SWEEP, which is OURS and is not on the PS2 at all: one
//!   [`crate::collision::CollisionWorld::wall_sweep`] per moving unit per tick
//!   in the direction it wants to go, and when that is blocked the wall
//!   tangent then a fan about it. The originals only have barriers, a contact
//!   push and stuck timers; the map's model geometry that no barrier covers is
//!   what this is for. The pad is built from the chosen WORLD direction, so a
//!   unit strafes while its yaw catches up and nothing gates walking on the
//!   yaw error any more (Phase A's `ADVANCE_CONE` is gone).
//!
//! Phase A.2 (PLAN Log "PHASE A.2 DESIGN") closes the two failures the A.1
//! sweep left, both of which looked the same from outside — a unit at full
//! speed making no progress:
//!
//! - the probe became a SWEEP. A.1 translated the probe capsule a metre ahead
//!   and asked for a contact THERE, so a wall thinner than the gap it jumped
//!   was never touched while the wall pass pushed the unit back every tick.
//! - the wall pass RECORDS its push ([`crate::unit::Unit::wall_push`], the
//!   port's stand-in for the PS2's `soldier+0x970` accumulator), hashed with
//!   the rest of the unit. The design had the steering read it as "blocked"
//!   even when the sweep is clear, and take the side-step's side from it;
//!   MEASURED over the 32-map sweep, both halves cost maps and neither is
//!   wired in. With the sweep and the same-level waypoints alone: 27 of 32
//!   finish, median `stuck_max` 1,770. Adding the "blocked" half: 24 of 32,
//!   median 3,300. Adding only the side-step's side: 25 of 32, median 2,640.
//!   Adding the "blocked" half WITHOUT the same-level waypoints: 22 of 32,
//!   median 4,230. (A.1 at HEAD: 26 of 32, median 1,980.) The push is
//!   non-zero on ~60 % of unit-ticks on the jammed maps, which is what a
//!   push-out threshold of "any penetration at all" reports and not what a
//!   contact accumulator should mean; the field is kept, hashed and tested so
//!   the rule can be tried again against a real accumulator.
//! - waypoints are resolved on the UNIT'S OWN LEVEL (`route::hub_waypoint`), and a
//!   leg whose hub has no floor there is rejected for an alternative.
//! - [`crate::nav::Arc::JET_JUMP`] arcs are off a jetless soldier's graph.
//!
//! Phase A.4 unstuck the remembered slide side, which A.1 could set but
//! never change while the way ahead stayed blocked: the steering flips the
//! side when its own tangent is blocked and the other is open, and the
//! stuck window writes the side-step's side into [`AiState::slide`] so the
//! PS2's "side-step to the side we did not use last time" reaches the
//! wall-follow. Card OBST makes the side a COMMITMENT (`ai/avoid.rs`: one
//! side per obstacle, an inside corner turned, the leave rule, a small body
//! passed before it is touched) — [`PROBE_DIST`], [`FAN_STEP`], the fan
//! order, [`SIDESTEP_OFFSET`] and the stuck constants are unchanged.
//!
//! Card STFAL (`analysis/re-round-stalls.md` C1, C8) gives every step the
//! steering commits a FLOOR: a candidate whose floor one probe ahead is more
//! than [`LEDGE_DROP`] under both the feet and the point walked at is
//! refused like a wall, the side-step is dropped over a drop, and a unit with
//! nowhere open stands and lets the stuck window re-plan. The probe also asks
//! the wall pass about the vehicles' HULLS at its end, which the lifted sweep
//! passes over. Over 32 maps x 9 seeds finishes hold (255 → 254 of 288;
//! 28 → 26 at seed 42), BF2 KAM1's environment deaths go 8.2 → 5.1 a
//! minute, BF1 BES1's 11.3 → 7.1, and BF1 KAS2's Empire-spawn cluster
//! 18 → 3 episodes.
//!
//! Card BNDS (the maintainer, 2026-10-01) gives the AI BOUNDS and a way back, from
//! data the disc carries (`sim/bounds.rs` has the level test):
//!
//! - a `BARR` barrier at an EDGE of the playable level — the ground past it
//!   is off the level or missing — is a wall for the steering
//!   (`steer::barrier_ahead`). BF2 Kamino rings every platform's rim rail
//!   with barriers and the PS2's walk step never crosses one; the port's
//!   slide and side-step carried units over the rails and down the hulls.
//! - the mission's `DisableBarriers` calls are replayed (BF2 Kamino's door
//!   barriers), a hub is reached and routed from ON ITS LEVEL (the resolved
//!   hub floors: `route::hub_on_level`, `route::hub_reached`), and an arc no
//!   soldier can walk is not planned over.
//! - a bot that is OFF the level anyway (pushed, blasted, fallen) walks BACK:
//!   at the nearest hub it can stand level with, never further down, jumping
//!   a low wall from where it stands (card AIRC); with no way back for [`LOST_RECYCLE`] it is
//!   given up and respawns ([`LostState`]). Card VMESH: one on foot SHUT inside a watertight
//!   map body (`Sim::shut_in_solid`: a closed prop such as BF1 Endor's cp2 boulder, never a
//!   room, cave or tunnel) is lost too, whatever it is doing, and given up after
//!   [`SHUT_RECYCLE`].
//!
//! Card RCYC (the maintainer, 2026-10-01) is the last resort beside it, for a bot ON
//! the level that goes nowhere (`stall`, [`StallState`]): one that wants to
//! walk and has not moved [`RECYCLE_ROAM`] net (in the endgame: onto new
//! ground) for [`RECYCLE_AFTER`] of such ticks dies as stalled, with its ticket, and respawns. It should exist and
//! never be needed: the sweep counts and places every use
//! (`round_to_outcome`, `sweep_seeds.py`), and each repeated spot is a defect.
//! Card RLOOP (the maintainer, 2026-10-11) is its short form: a bot whose walk fails
//! [`PIN_FAILS`] progress windows without leaving one place ([`PIN_ROAM`])
//! is given up after about 22 s, whatever plans it was handed meanwhile:
//! sent on card RETW's floor walk to its leg's waypoint when there is one,
//! else recycled.
//!
//! Over 8 seeds x 32 maps (lead_tmp/fable2/README.md): BF2 Kamino's world
//! deaths 84 → 11 and finished rounds 0 → 4 of 8, BF2 Polis Massa 0 → 8 of 8.
//!
//! Card HOLD (the maintainer, 2026-10-01) — bots hold ground and fight, and not as
//! statues ([`HoldState`], `behave`):
//!
//! - (card TPOS: the hold a bot took at a hub when it perceived an enemy -
//!   a hint or a drawn hub point for a 10-24 s dwell - is gone; a fight's
//!   position is the Tactician's, `ai::tactic`.) An enemy seen inside
//!   [`HOLD_RANGE`] is fought from a position taken where the bot stands.
//! - on its position it looks the way the nearest enemy is and is lively by
//!   its tier's chances (`skill::Tier`): a SHUFFLE to a spot a couple of
//!   metres off every few seconds, a STRAFE on a burst.
//! - card REACT (the maintainer, 2026-10-07; ours, the PS2's attacker ignores a hit):
//!   a bot on a position that is HIT draws once a burst ([`REACT_COOLDOWN`])
//!   over its tier's row (`skill::Tier::react`) — it stays, strafes, rolls
//!   aside (pressed [`ROLL_LEAD`] after it sets off: a standing soldier
//!   rolls on the spot) or moves to a new position (`behave::react`).
//! - card JITR: its goal has ONE writer (`HoldState::step_to`) and a commit
//!   time ([`HOLD_COMMIT`]) the shuffle and the strafe share; a
//!   holder's answer to a hit is such a step too (never the route's 5 m
//!   side-step, which a holder does not count down); a step of under
//!   [`HOLD_WALK_DIST`] is walked; a goal the steering cannot go straight at
//!   for [`HOLD_BLOCKED`] is given up for where the bot stands; and no two
//!   holders take points within [`HOLD_SPACING`].
//! - a grenade on the ground (`Sim::hazards`, the PS2's 7.5 m warning) is run
//!   from, after the tier's delay and by its chance. Nothing in the port
//!   throws one yet (card GREN); the list is its hook.
//! - in the endgame ([`HOLD_ENDGAME`]) nobody waits in a position.
//!
//! BF1 KAS2, 8 seeds x 5 min against the PS2's live sample
//! (lead_tmp/fable2/README.md): standing still 20.5 % → 24.3 % (PS2 55 %),
//! in the lagoon 42.7 % → 41.1 % (PS2 22 %). Holding more than this — every
//! bot at every hub in reach of an enemy reads 55 % and 20 % — stood BF2
//! Kamino's rounds still: at the port's 3-5 kills a minute there (the PS2's
//! 17.6) a round is decided by who takes the posts.
//!
//! Outside a position an engaged unit keeps WALKING ITS ROUTE while it aims
//! and fires (design decision 4).
//!
//! Deterministic: sim state only. The chooser's random word is a per-unit hash
//! of (unit index, tick, generation), like [`unit_rand`], so `think` keeps
//! taking `&Sim` and replays bit-exactly.
//!
//! ## Where the code lives
//!
//! Split by queue SPL4 (2026-09-24, a pure move) into six parts, each
//! re-exported here so every `crate::ai::X` path is unchanged: `state` (the AI
//! state, its route leg and its hash), `command` (a squad order's AI side),
//! `draw` (the per-unit hash draws and the D13 stance pick), `route`
//! (objectives and legs on the planning graph), `steer` (the local layer) and
//! `brain` (`think` itself); card VAI added `ride` (a bot's vehicle: the
//! boarding claim, the driver's pad, the dismount — `think` hands a seated
//! bot to it). This file keeps every tuning constant — together,
//! so the sweep can move them in one place — and the test fixtures the parts'
//! tests share.
//!
//! Card SPL11 (2026-10-06, no rule changed) split `brain` by OWNER, the seams
//! of `analysis/bot-design.md` 3: `behave` (`think`: what the bot is doing —
//! one priority chain a tick, which hands exactly one `AimRequest` to the
//! gunner and one `MoveRequest` to the mover, and writes no stick itself),
//! `gunner_foot` (the only writer of a foot soldier's look sticks and FIRE),
//! `mover` (the only writer of its move sticks, ROLL_STANCE and JUMP, and the
//! progress books), and the route's advance in `route`.
//!
//! Card MOVR (2026-10-06, `analysis/bot-design.md` 4.7, 5, 6.3) — the mover
//! commits, and a walk has an outcome:
//!
//! - a bot that has ARRIVED stands: the stick stays released until its
//!   request's destination moves more than [`DEST_DEADBAND`] or another link
//!   of the chain walks it, and a shove of under [`ARRIVED_SLACK`] does not
//!   walk it back; the last step is scaled (the PS2's);
//! - the steering's heading is PICKED on every [`HEADING_PERIOD`]-th tick and
//!   held between — the slide's side used to be re-picked 30 times a second;
//! - the think's phase is the unit's index, not its spawn tick;
//! - every tick's walk answers [`Walk`] (following / arrived / failed), and a
//!   walk that fails [`GIVE_UP_AFTER`] progress windows in a row gives its
//!   goal up: the commander draws the bot another post (`behave::recover`).
//!   [`Behaviour`] names what the bot is doing, for the sweep's
//!   unexplained-standing counter (B9) and the black box.
//!
//! Card MOVR2 (2026-10-07): a leg is walked by the PS2's rule — at the
//! bot's projection on the leg's line advanced by its speed + [`LEG_LEAD`]
//! (`route::leg_point`; card MOUTH, 2026-10-12: past a detour corner that
//! line starts at the corner, the PS2 follower's segment). The PS2's stuck
//! clock, its 1.0 m / 0.5 m arrival radii and a give-up at the third failed window were measured again on
//! card OBST's steering and are NOT taken (`analysis/bot-design.md` 9, D18).
//!
//! Measured and NOT taken (the card's report has the tables): the PS2's own
//! stuck clock (2 m in 3 s / no node progress in 5 s) in place of the 2 s
//! window, its arrival radii (1.0 m at a node, 0.5 m at the last) and its
//! leg aim (the projection advanced by |v| + 0.2 m) each cost pinball
//! windows on BF2 Kamino or Polis Massa with the wall-slide steering of
//! that day — card MOVR2 tries them again on card OBST's avoidance.

mod arsenal;
mod avoid;
mod behave;
mod command;
mod draw;
mod gunner;
mod gunner_foot;
pub mod hover_drive;
pub mod man;
mod mover;
mod ride;
mod route;
pub mod snipe;
mod stall;
mod state;
mod steer;
pub mod tactic;
pub mod team;
pub mod walker_drive;

pub use arsenal::vehicle_bonus;
pub use avoid::ROUND_MAX;
pub use behave::think;
pub use command::CmdState;
pub use draw::{
    route_rand, stance_hold, unit_dir, unit_rand, unit_unit, SALT_BURST, SALT_CHARGE, SALT_DODGE,
    SALT_ERR_AXIS, SALT_ERR_DIR, SALT_FOLLOW, SALT_FORMATION, SALT_GRENADE, SALT_HOLD,
    SALT_NEED_GUNNER, SALT_REACT, SALT_STANCE, SALT_TAP,
};
pub(crate) use gunner_foot::release_blasts_own_side;
pub use gunner_foot::yaw_toward;
pub use route::{first_leg, floor_walked};
pub use state::{
    AiState, Behaviour, FightState, FlankState, HoldState, LostState, MoverState, NavTick,
    RideState, RoleState, Round, Route, RouteSource, SnipeBook, SnipeState, StallState, SteerPick,
    TposState, Walk, WalkAsk,
};
pub use steer::{rotate_xz, way_ahead, Stop, StopKind};

pub(crate) use ride::{claim_seat, in_reach};
pub(crate) use route::{hub_on_level, mover, plan_leg, start_hub};

// ---------------------------------------------------------------------------
// Card MOVR (`analysis/bot-design.md` 4.7, 5, 6.3): the foot mover's rules.
// All four are OURS and PROVISIONAL, chosen by the sweep (the card's report
// has every variant's numbers, the PS2's own stuck clock and arrival radii
// among them; card MOVR2 measured those again on card OBST's steering, 24
// seeds x 32 maps, and 3 against 6 failed windows: `analysis/bot-design.md`
// 9, D18).
// ---------------------------------------------------------------------------
/// A request whose destination is within this of the one the mover is
/// committed to is the same request (m): the design's deadband.
pub const DEST_DEADBAND: f32 = 0.5;
/// A bot that has arrived stands until it is this far outside its arrive
/// radius (m): a push by a friend does not walk it back.
pub const ARRIVED_SLACK: f32 = 1.5;
/// The mover picks its heading (the steering's probes) on every this-many-th
/// tick, the phase by unit index, and holds it between (the PS2 steers
/// every frame near the human and every 3rd or 5th away, re-soldier-ai 1.3).
pub const HEADING_PERIOD: u64 = 3;
/// A walk that has failed this many progress windows in a row gives its
/// goal up (`behave::recover`): the post, for the commander to draw another,
/// or the boarding claim.
pub const GIVE_UP_AFTER: u8 = 6;

/// Stop advancing when an engaged enemy is closer than this. PROVISIONAL.
pub const HOLD_RANGE: f32 = 25.0;
/// Fire when yaw/pitch errors are within this (radians). PROVISIONAL.
pub const AIM_TOLERANCE: f32 = 0.035;
/// Card FFLN: a bot does not fire while a friend's capsule axis is within
/// this of its line of fire (m): the capsule's radius and half as much
/// again. PROVISIONAL (ours).
pub const LINE_CLEAR: f32 = crate::unit::SOLDIER_RADIUS * 1.5;
/// ...widening per metre along the line by the weapon's spread and this:
/// [`AIM_TOLERANCE`], the angle the aim may be off its target by when the
/// trigger goes down. PROVISIONAL (ours).
pub const LINE_SPREAD: f32 = AIM_TOLERANCE;
/// ...and the line is watched this far past the target (m), where a miss
/// flies on into the friends fighting it. PROVISIONAL (ours).
pub const LINE_PAST: f32 = 5.0;
/// Re-evaluate targets every N ticks (spreads the O(n²) scans). PROVISIONAL.
pub const THINK_PERIOD: u64 = 10;
// ---------------------------------------------------------------------------
// Phase C commit 3: the aim model (brief D9/D10). The P4 placeholder — a
// bounded random angle re-rolled every 15 ticks — is GONE; what replaces it is
// the PS2's own shape: a second-order tracker that steers an aim DIRECTION
// toward the wanted one, aiming at a tracked POSITION that carries a skill-
// dependent error radius.
// ---------------------------------------------------------------------------

/// The tracker slews (instead of accelerating) when the wanted direction is
/// more than 30° away: `cos 30°` and `sin 30°` (RE §4.2 step 3).
pub const SLEW_COS: f32 = 0.866_025_4;
/// `sin 30°`, the slew step's other half.
pub const SLEW_SIN: f32 = 0.5;
/// The tracked-position error can never exceed this many metres, whatever the
/// skill or the target's speed (RE §4.3, `FUN_002ae7b8`'s `min.s` against
/// 2.0; `analysis/re-ai-sniper.md` §1 confirms the constant).
pub const ERR_MAX: f32 = 2.0;
/// With NO target the PS2 holds the error at a constant 1.5 m floor
/// (`FUN_002ae7b8`, B2:233299 — LEAD ADDENDUM 4a): `err` persists across
/// targets and is never reset on acquisition, so a unit that spent a while
/// target-less starts its next target at >= 1.5 m.
pub const ERR_NO_TARGET: f32 = 1.5;
/// A unit's error grows by this times the CHANGE of its velocity between two
/// thinks of its block, over its own MaxSpeed (card CLSE2,
/// `analysis/re-soldier-ai.md` 18.1: a start or a stop 0.455, a reversal at
/// full speed 0.91, a straight run nothing). A BF1 round's shooter error
/// grows by it per metre its target moved, over the target's top speed (D10).
pub const ERR_GROWTH: f32 = 0.455;
/// How much the target's CURRENT speed widens the error on top of the
/// tracked growth (RE §4.3's per-tick term).
pub const ERR_SPEED_TERM: f32 = 0.6;

// ---------------------------------------------------------------------------
// Phase A navigation constants. ALL FOUR ARE PROVISIONAL (ours) and live here,
// together, so the sweep can move them in one place — PLAN Phase A design
// decision 4 ("measured by the sweep, not tuned by taste").
// ---------------------------------------------------------------------------

/// A hub counts as reached at its own radius, floored at this (m): the discs
/// author radii of 1–6 m and eleven hubs of radius 0.
pub const ARRIVE_MIN_RADIUS: f32 = 1.5;
/// Ticks in one progress window.
pub const STUCK_WINDOW: u16 = 60;
/// Metres of XZ progress toward the waypoint a unit must make in one window.
pub const STUCK_PROGRESS: f32 = 0.3;
/// Ticks of side-stepping after a window with no progress.
pub const SIDESTEP_TICKS: i8 = 30;

// ---------------------------------------------------------------------------
// Phase A.1 steering constants. ALL PROVISIONAL (ours), all in one place for
// the same reason as the four above — the sweep moves them, taste does not.
// ---------------------------------------------------------------------------

/// How far ahead the wall probe looks (m).
pub const PROBE_DIST: f32 = 1.0;
/// A detour corner counts as reached at this XZ distance (m). The PS2's mover
/// arrives at 1.0 m on the last node of a path and 4.0 m between (RE_LOG,
/// "Arrival and locate"); a corner is an intermediate node, so 1.0 m is the
/// tighter of the two and keeps a unit on the outside of the barrier.
pub const CORNER_ARRIVE: f32 = 1.0;
/// The PS2's leg rule (BF2 `FUN_002a5fa8`, `analysis/re-soldier-ai.md` 2): a
/// soldier on a leg aims at its projection on the leg's line advanced by its
/// speed (m/s read as m) plus this (m) (`route::leg_point`, card MOVR2).
pub const LEG_LEAD: f32 = 0.2;
/// PROVISIONAL (ours): a stuck unit's re-plan beside its arc's line goes to
/// the hub's centre first when it is further than this off the line (m)
/// (`route::replan`).
pub const ARC_LINE_TOL: f32 = 1.0;
/// The margin the corner walk gives a soldier: its capsule radius plus 0.5 m.
/// OURS, PROVISIONAL: the PS2 tests a leg with the unit radius alone
/// (`FUN_0026c0d0` -> `FUN_00268f68` -> `FUN_00267d18`, RE_LOG "The corner
/// search, read again") and puts its corners 0.5 m further out; the port's
/// test is 0.5 m wider than that. Measured by card OBST2 on the PS2's search
/// (8 seeds x 32 maps): with the capsule radius alone 558 of 52,079 crossing
/// legs find no way for 3,462 of 68,987 and recycles are 47 for 62, but
/// The maintainer's Kamino scene has 13 of 396 trails over 6 reversals for 3 of 376
/// (the corners then lie 1.0 m off a pod's wall) - not taken.
pub const DETOUR_RADIUS: f32 = crate::unit::SOLDIER_RADIUS + 0.5;
/// How far to the side an armed side-step offsets the move target (m). The
/// PS2's contact side-step is 5 m along the path perpendicular
/// (`FUN_0026aba8`); ours is triggered by the stuck window rather than by a
/// contact normal, which is the PROVISIONAL part.
pub const SIDESTEP_OFFSET: f32 = 5.0;
/// Card STFAL: a candidate step is refused when the floor one [`PROBE_DIST`]
/// ahead is more than this below both the unit's feet and the point it is
/// walking at (m). Over the 3.3 m ledge a BF2 TAT1 route walks down
/// (MEASURED: 2 m held units there for thousands of ticks), under the
/// 6.6 m MUS1 drop that strands a unit on the 59.4 level and the 10 m from
/// KAM1's decks to its rims.
pub const LEDGE_DROP: f32 = 4.0;
/// Card STFAL: the floor probe also asks this far either side of the probe
/// point (m). Half a soldier's radius: the full radius held units beside
/// holes in a map's collision (BF2 GEO1, TAT1) for thousands of ticks.
pub const LEDGE_MARGIN: f32 = 0.25;
/// Card STFAL: the wall pass's push at the probe's end counts as a wall when
/// it pushes back against the step by more than this (m).
pub const HULL_PUSH_MIN: f32 = 0.05;
/// Card OFFG: the steering's ramp test (`steer::ramp_clear`) samples the floor
/// ahead this often (m), takes a rise of more than [`RAMP_SAMPLE_RISE`]
/// between two samples for a step and not a ramp, and leaves a rise under
/// [`RAMP_MIN_RISE`] over the whole probe to the level sweep.
pub const RAMP_SAMPLE: f32 = 0.25;
pub const RAMP_SAMPLE_RISE: f32 = 0.3;
pub const RAMP_MIN_RISE: f32 = 0.1;
/// One step of the blocked-forward fan (radians): the design's 30 deg.
pub const FAN_STEP: f32 = 30.0 * crate::weapon::DEG;
/// Detour corners carried on one leg. A longer corner path is cut to this
/// many, and at the last of them the rest is searched for
/// (`route::advance_route`, card OBST2: about 2 % of the discs' crossing
/// legs need more - 1,251 of 61,849 in 64 rounds - a walk round the rails
/// of a pit or a platform).
pub const DETOUR_MAX: usize = 6;

// ---------------------------------------------------------------------------
// Card BNDS (the maintainer, 2026-10-01): the AI's bounds and the way back. The BOUNDS
// are the disc's own `BARR` barriers — the PS2's walk step never crosses one
// (RE_LOG "Barriers are the PS2's Obstacles": `FUN_00250fc8` re-aims at a
// corner per step) — taken by the steering as walls. The way BACK is ours:
// every constant here is PROVISIONAL.
// ---------------------------------------------------------------------------
/// A hub is on a unit's level, and is ARRIVED at, when its resolved floor is
/// within this of the unit's feet (m): over a hub disc on a ramp, under the
/// 8.6 m between BF2 Kamino's stacked walkways.
pub const HUB_LEVEL_TOL: f32 = 3.0;
/// A soldier whose own hub is on another storey asks this many of the nearest
/// hubs for one on its own (`route::level_hub`, card STLVL). PROVISIONAL
/// (ours): [`crate::sim::LEVEL_HUBS`], the level test's count.
pub const LEVEL_HUB_TRIES: usize = 6;
/// ...a hub whose floor is within this of the feet (m) is on their storey
/// without asking the walk. PROVISIONAL (ours): a step and a kerb.
pub const LEVEL_HUB_NEAR: f32 = 1.0;
/// ...and the floor walked from the feet reaches a hub when it arrives within
/// this of the hub's floor (m). PROVISIONAL (ours): under a soldier's height.
pub const LEVEL_HUB_REACH: f32 = 1.5;
/// The margin the steering keeps from a barrier (m): the PS2 tests its legs
/// against the rectangle grown by the unit's radius.
pub const BARRIER_MARGIN: f32 = crate::unit::SOLDIER_RADIUS;
/// A barrier is an edge of the level when the ground this far past the unit
/// along its step (m) is off the level or missing (`steer::barrier_ahead`):
/// past a rail and the first metres of a hull, onto what is clearly below.
pub const EDGE_LOOK: f32 = 6.0;
/// A lost bot refuses a step whose floor is more than this under its feet
/// (m): it never walks further down a hull.
pub const LOST_DOWN: f32 = 0.4;
/// A lost bot jumps a wall ahead when the sweep is clear this far over its
/// feet (m): under the soldier's 1.78 m jump apex.
pub const LOST_JUMP_CLEAR: f32 = 1.3;
/// ...and tries again this many ticks after a jump (1.5 s: the jump's 0.87 s,
/// the landing's soft lock and a few steps of the slide along the wall).
pub const LOST_JUMP_AGAIN: u64 = 45;
/// Card RETW: a lost bot walking its floor path back (`Sim::return_path`)
/// has reached a point of it this close in XZ (m)...
pub const RETURN_ARRIVE: f32 = 1.0;
/// ...and this close in height (m). PROVISIONAL (ours).
pub const RETURN_ARRIVE_DY: f32 = 1.0;
/// ...and looks for a walk at most this many times a spell. PROVISIONAL.
pub const RETURN_PLANS: u8 = 3;
/// A bot off the level this long with no way back is given up (ticks, 45 s):
/// it dies as lost ([`crate::sim::DeathCause::Lost`]) and respawns.
pub const LOST_RECYCLE: u64 = 45 * crate::TICK_HZ as u64;
/// Card VMESH: a bot on foot shut inside a map body
/// (`CollisionWorld::enclosing` 1 m over its feet) this long is given up as
/// lost, whatever it is doing - capturing included (ticks, 5 s). No walk
/// leaves a closed body, nobody outside sees him and he contests the post
/// from inside (BF1 Endor's cp2 boulder: 806 s on one bot). PROVISIONAL
/// (ours): the PS2 puts nobody inside a body.
pub const SHUT_RECYCLE: u64 = 5 * crate::TICK_HZ as u64;
/// The height over a soldier's feet the shut test asks at (m): the chest,
/// over the floor a closed prop has inside it.
pub const SHUT_PROBE_UP: f32 = 1.0;

// ---------------------------------------------------------------------------
// Card RCYC (the maintainer, 2026-10-01): the last-resort recycle of a bot that is on
// the level and going nowhere. The PS2 sets a stuck flag (2 m in 3 s,
// re-round-stalls.md §3); what its agents do with it is unread. This rule is
// ours and PROVISIONAL.
// ---------------------------------------------------------------------------
/// A bot that wants to walk and has made no net progress for this many such
/// ticks (120 s) is given up: it dies as stalled
/// ([`crate::sim::DeathCause::Stalled`]) and respawns.
pub const RECYCLE_AFTER: u32 = 120 * crate::TICK_HZ;
/// Net progress mid-round (m): this far from where the count started —
/// coming this much nearer a waypoint is such a move, so the one test covers
/// both. Card OBSR: 15 m, not card RCYC's 5 — a bot LAPPING a few metres of
/// ground moves 5 m every second and was never given up (AUDIT OBST: BF1
/// Tatooine's rock gap held bots for 480 - 600 s, and the count of recycles
/// could not see them). A bot that walks for two minutes and is never 15 m
/// from where it began is going nowhere. MEASURED, 32 seeds x 32 maps (main
/// e54841a + the rim rule): 206 recycles -> 284, the 78 more at the places
/// bots lapped unseen (BF1 Tatooine's gap 23 -> 44, BF1 Bespin (0, -160)
/// 0 -> 24, BF1 Geonosis (108, -12) 0 -> 7); rounds finished 1,021 of 1,024
/// both ways. At 30 m: 350 (BF2 Geonosis 16 -> 42), not taken.
pub const RECYCLE_ROAM: f32 = 15.0;
/// New ground (m): once either team is down to [`HOLD_ENDGAME`]
/// reinforcements, progress is this far BEYOND the ground covered since the
/// watch opened ([`StallState`]'s box).
/// Measured, 8 seeds x 32 maps x 120,000 ticks (lead_tmp/rcyc/README.md):
/// against a point alone a bot PACING a wall is never given up — BF2 UTA1
/// seed 42 stood 1 v 9 for 30,000 ticks, its last bot walking a 14 x 9 m
/// box at 4 m/s (20 m still missed BF2 KAM1 seed 4's 32 x 19 m one); new
/// ground alone, all round, made the recycle 31 % of BF2 Kamino's deaths
/// (630 of them) — a rule of play there, not a last resort.
pub const RECYCLE_NET: f32 = 5.0;
/// Card RLOOP (ours, PROVISIONAL): a walk's failed progress windows are
/// counted at ONE place — while the bot has not been this far (m) from where
/// the first of them happened.
pub const PIN_ROAM: f32 = 2.0;
/// ...and a bot with this many is given up ([`stall::watch`]): 10 windows of
/// [`STUCK_WINDOW`] = 20 s of failing, about 22 - 26 s of standing. The
/// Recover ladder's own count starts again whenever a new plan's waypoint is
/// nearer than the last one's, so a bot handed two plans in turn against one
/// wall (BF2 Endor (98, -1, -50): `-1 -> 12` / `12 -> 11`; BF2 Kashyyyk
/// (46, 28, -109)) never reached the post's give-up and stood for the 120 s
/// of [`RECYCLE_AFTER`]. MEASURED, 8 seeds x 32 maps on main 4ca302f
/// (`lead_tmp/rloop/sw`): spells of 20 s+ inside a 2 m box in recover /
/// travel 59 (39 over 30 s, 2,835 s) -> 32 (0 over 30 s, 749 s) at 10; 5
/// over 30 s at 12 and 33 at 15. No bound is free: 27 bots in those 256
/// rounds walked away after 10 or more failures at one place (the most 27),
/// and at 10 they are given up instead.
pub const PIN_FAILS: u8 = 10;
/// ...and sent on a floor walk to its leg's waypoint ([`stall::send_off`]);
/// pinned again within this of where that walk began (m), it is recycled.
/// PROVISIONAL (ours): the walk itself may be up to 60 m long; a
/// bot that got off its pin and was dragged back near it is the same trap.
pub const PIN_WALKED: f32 = 4.0;

// ---------------------------------------------------------------------------
// Card HOLD (the maintainer, 2026-10-01): bots hold ground and fight. On the PS2 a bot
// is stationary 55 % of the time (BF1 KAS2, 5 minutes of live RAM against the
// port's 20 % — lead_tmp/kas2ai/README.md); the port marched post to post and
// stood only inside `HOLD_RANGE` of an enemy it saw. The rule is ours and
// every constant PROVISIONAL; the per-tier liveliness is `skill::Tier`'s.
// ---------------------------------------------------------------------------
/// No Tactician's position (nor a sniper's spot) is taken once either team
/// is down to this many reinforcements: the endgame is for closing in.
pub const HOLD_ENDGAME: u32 = 10;
/// A CLOSE hold — an enemy seen inside [`HOLD_RANGE`] — lasts this long past
/// the last tick he was (ticks, 2 s).
pub const HOLD_CLOSE: u64 = 2 * crate::TICK_HZ as u64;
/// A hold from which no enemy has been perceived for this long is given up
/// (ticks, 4 s): the bot holds to FIGHT, and with nobody to fight it advances.
pub const HOLD_QUIET: u64 = 6 * crate::TICK_HZ as u64;
/// With no hint, a point within this of the hub's centre (m; the hub's own
/// radius when that is smaller).
pub const HOLD_SPOT_RADIUS: f32 = 5.0;
/// A shuffle steps this far from the spot at most (m), a strafe this far
/// sideways (m), and a holder counts as standing on its point within
/// [`HOLD_ARRIVE`].
pub const HOLD_SHUFFLE: f32 = 2.5;
pub const HOLD_STRAFE: f32 = 2.5;
pub const HOLD_ARRIVE: f32 = 0.6;
/// Ticks between shuffle opportunities (3 s).
pub const HOLD_MOVE_PERIOD: u64 = 3 * crate::TICK_HZ as u64;
// Card JITR (the maintainer's snapshot, 2026-10-02: a holder ran 46 m in 10 s with 26
// reversals inside a 1.5 m box). All five are OURS and PROVISIONAL until card
// SAIR says what the PS2's soldier does on a position.
/// A holder's goal is kept this long (2 s): no shuffle and no strafe takes
/// another one before it. They draw from this one budget.
pub const HOLD_COMMIT: u64 = 2 * crate::TICK_HZ as u64;
/// A holder within this of its goal (m) WALKS there — the stick at
/// [`HOLD_WALK_STICK`] — and does not run: every shuffle and strafe, and the
/// last metres of the walk to the position.
pub const HOLD_WALK_DIST: f32 = 3.0;
pub const HOLD_WALK_STICK: f32 = 0.5;
/// A holder whose steering has been turned off the straight line to its goal
/// for this many ticks (1 s, summed over the goal's life) stands where it is:
/// a point behind a crate is not circled.
pub const HOLD_BLOCKED: u8 = crate::TICK_HZ as u8;
/// No position, shuffle or strafe within this of another holder's point or
/// of a player (m).
pub const HOLD_SPACING: f32 = 2.0;
/// A roll carries a soldier about this far (m): RollSpeedFactor 1.7 x
/// MaxSpeed 7 over 0.85 s. A hit bot only rolls with this much clear floor.
pub const ROLL_REACH: f32 = 10.0;

// ---------------------------------------------------------------------------
// Card FGHT: the PS2's FIGHT (`analysis/re-soldier-ai.md` 4.2, 4.4, 4.5; BF1's
// twin in section 5) — a seen target inside the weapon's range is fought from
// a position of the bot's own fixed style, until card TPOS's Tactician.
// ---------------------------------------------------------------------------
/// A fight is on while its seen target is inside the weapon's `MaxRange`
/// plus this (m; the combat helper's attack gives up beyond it).
pub const FIGHT_BEYOND: f32 = 10.0;
/// The circle's pause on a ring point (1.9 s; attack mode 3).
pub const CIRCLE_PAUSE: u64 = 57;
/// A circling bot on its way stops for a pause at this RATE (a second): the
/// PS2's attack mode 3 draws 1 in 12 a think (`FUN_002a3ff0`: a bound of 6
/// times `DAT_006980a0` = 2; `analysis/re-soldier-ai.md` 1.1, 4.5) and its
/// fighter 7-28 m in front of the player thinks every 0.4 s — 0.21 a second
/// (farther off, a think every 0.8-1.6 s, 0.05-0.10; inside 7 m 0.83).
pub const CIRCLE_STOP_RATE: f32 = 1.0 / (12.0 * 0.4);
/// ...as the chance per think of the port, which thinks every
/// [`THINK_PERIOD`] ticks (0.33 s): about 1 in 14 (card FGHTA; it was 1 in 6
/// a think, 0.5 a second). `FEEL_NOTES.md`.
pub const CIRCLE_STOP: f32 = CIRCLE_STOP_RATE * THINK_PERIOD as f32 / crate::TICK_HZ as f32;
/// The walk to a ring point ends this near it (m). OURS, PROVISIONAL: the
/// PS2's path ends at its last node.
pub const RING_ARRIVE: f32 = 1.5;
/// A ring point is taken in the hub nearest the drawn one, when one is within
/// this of it (m): the walk stays on the authored graph. OURS, PROVISIONAL
/// (the PS2 hands its point to the path finder): ring points drawn anywhere
/// walked bots off Bespin's platforms (42 more deaths under the map in 8
/// rounds) and into Geonosis's death regions (22 more).
pub const RING_SNAP: f32 = 10.0;
/// ...and never one more than this farther from the target than the bot
/// stands (m): a circle does not back away.
pub const RING_BACK: f32 = 2.0;
/// The share of the bots whose fixed style is "stand and shoot" (the rest
/// circle), Easy first, then every other difficulty: the PS2's weights for
/// codes 2 and 3 with under 5 m of open ground round the bot, where zig-zag
/// and flank are not drawn (74-93 % of its combat rows; re-soldier-ai 4.4)
/// — Easy 3 : 1, Normal and Elite 1 : 1.
pub const STAND_SHARE: [f32; 2] = [0.75, 0.5];
/// A third of the standers crouch (attack mode 2: `n % 3 == 1`).
pub const STAND_CROUCH: f32 = 1.0 / 3.0;
/// OURS (the maintainer, 2026-10-05: "everyone stops in their tracks which puts them
/// all right next to each other ... spread out a little bit"): a bot that
/// stands to shoot within this of another fighter's position or of a player
/// (m) takes a free point within [`FIGHT_STEP`] instead of where it stands.
/// PROVISIONAL; `FEEL_NOTES.md`.
pub const FIGHT_SPACING: f32 = 4.0;
/// ...that far off at most (m).
pub const FIGHT_STEP: f32 = 7.0;

// ---------------------------------------------------------------------------
// Phase C D11 — fire discipline. The burst's bounds and pause are the weapon
// class's `BarrageMin` / `BarrageMax` / `BarrageDelay` (card CLSE4,
// `WeaponClass::barrage`; `analysis/re-soldier-ai.md` 20.3).
// ---------------------------------------------------------------------------

/// Ticks between two dodges of a bot walking its leg (2 s, RE §4.1).
pub const DODGE_COOLDOWN: u64 = 60;
/// Card REACT: ticks between two reactions to a hit of a bot on a position
/// (1.5 s: a burst is one draw). OURS, PROVISIONAL (the maintainer: "about 1-2 s").
pub const REACT_COOLDOWN: u64 = 45;
/// Card REACT: a hit bot's roll is pressed this many ticks after it sets
/// off sideways. A soldier standing still rolls on the spot (`soldier.rs`:
/// the roll's impulse scales a speed over 0.1 m/s) — as card HOLD's roll
/// did, pressed on the tick of the hit. OURS, PROVISIONAL.
pub const ROLL_LEAD: u64 = 6;
/// Card REACT: a roll needs floor this far past [`ROLL_REACH`] and this far
/// to each side of its line (m). OURS, PROVISIONAL.
pub const ROLL_PAST: f32 = 3.0;
pub const ROLL_LANE: f32 = 1.0;
/// Card REACT: with no ring point to take, a hit bot's new position is a
/// spot within this of where it stands (m; half of it at least). OURS,
/// PROVISIONAL.
pub const REACT_STEP: f32 = 7.0;
/// The floor of the charge-release draw `U(0.15, charge_hi)` (RE §4.4).
pub const CHARGE_MIN: f32 = 0.15;
/// D13: the AI presses ROLL_STANCE only once this long has passed in the
/// current stance. It is not a tuning knob — it MIRRORS the soldier's own
/// gate (`soldier::Soldier::step`, the stance-cycle arm), and it has to,
/// because the cycle is edge-triggered: a press the soldier drops still
/// leaves `prev_buttons` set, so the next tick's press is not an edge and the
/// unit would change stance exactly once in its life.
pub const STANCE_TAP_DELAY: f32 = 0.5;

// ---------------------------------------------------------------------------
// Card VAI: vehicles (`ride`). Which bot boards what, and when it gets out,
// is openbf's own AI (PLAN §AI); the PS2's AI boarding (B1 FUN_00244848 →
// FUN_0023c418) was located, not transcribed (re-vehicles §5). Every value
// here is PROVISIONAL and ours; the arrival radius is the PS2's (RE_LOG
// "Arrival and locate": an EntityHover's is ForwardSpeed × 0.5).
// ---------------------------------------------------------------------------
/// A bot on foot goes for a free vehicle no farther than this (XZ, m).
/// Card WAI2: the PS2's bots come 8-71 m for a walker, once 146 m
/// (re-walker-ai §13); 40 m left BF1 Geonosis's AT-TEs unclaimed.
pub const BOARD_SEEK: f32 = 75.0;
/// ...and only when its objective is farther than this (XZ, m).
pub const BOARD_TRIP: f32 = 80.0;
/// A claim not boarded within this many ticks is given up.
/// (Card WAI2: 30 s, for 75 m walked over the graph.)
pub const BOARD_GIVEUP: u64 = 30 * crate::TICK_HZ as u64;
/// Card WAI2: a bot boards from this much past the player's reach (m) — a
/// claimer walking at an AT-AT's origin slid on its hull 8.1-9.7 m from it,
/// 0.1-1.7 m short, until its claim ran out.
pub const BOARD_SLACK: f32 = 2.0;
/// Card TBRD: the PS2's board agent asks to enter when the bot's body sphere
/// and the vehicle's are this close (m, 3-D): BF2 1.5 (`FUN_002c42d8`, `lui
/// 0x3fc0` at 0x2c4564), BF1 0.5 (`FUN_00244848`, `lui 0x3f00` at 0x244a4c;
/// analysis/re-soldier-ai.md 7.2, 7.5). The port applies it to building
/// turrets; a vehicle keeps the player's reach and [`BOARD_SLACK`].
pub const BOARD_PAD_BF2: f32 = 1.5;
pub const BOARD_PAD_BF1: f32 = 0.5;
/// Card TBRD, OURS: a bot whose body's centre is this far (m) above or below
/// a building turret's origin, inside its reach in XZ, is on another floor
/// and gives its claim up (`ride::on_another_floor`): over a soldier's
/// height, under the 2.4 m that BF1 Bespin's balcony gun turrets stand
/// over the walk below them.
pub const OTHER_FLOOR: f32 = 2.0;
/// After a dismount or a failed claim, no new claim for this many ticks.
pub const BOARD_BLOCK: u64 = 30 * crate::TICK_HZ as u64;
/// A driver whose route stuck run reaches this many ticks gets out and walks.
pub const DRIVE_STUCK_BAIL: u16 = 3 * STUCK_WINDOW;
/// The ARRIVAL radius the hover's ForwardSpeed × 0.5 gives (RE_LOG): the
/// factor.
pub const HOVER_ARRIVE_FACTOR: f32 = 0.5;
/// Card WAI: a walker whose type cannot reach a post's hub is claimed for
/// the hub nearest the post only when the ride ends this much nearer the
/// post than the vehicle stands (XZ, m).
pub const RIDE_GAIN: f32 = 40.0;
/// Card VDROP: a hover whose type cannot reach a post's hub rides to the
/// hub nearest the post, as a walker does (`route::post_hub`).
pub const HOVER_NEAREST: bool = false;
/// Card VTAI: a bot pilot holds its throttle this long after a teammate
/// claims one of its gunner seats, for the gunner to walk up and board.
pub const GUNNER_WAIT: u64 = 8 * crate::TICK_HZ as u64;

// ---------------------------------------------------------------------------
// Card TURR (`analysis/bot-design.md` 6.1, D11; the maintainer's annoyance 1): a bot
// mans a building turret only with a PURPOSE and gets out by five rules
// (`man`). The PS2's own values are marked; the rest is OURS, PROVISIONAL.
// ---------------------------------------------------------------------------
/// The offer pass runs once in this many ticks (1 s).
pub const MAN_OFFER_PERIOD: u64 = crate::TICK_HZ as u64;
/// A free turret is offered to the nearest free bot within this (m): the
/// PS2's offer scan, `DAT_00698454` = 50 m (re-soldier-ai 7.1).
pub const MAN_SEEK: f32 = 50.0;
/// The defence clause: a turret this near (XZ, m) to a post the team owns.
pub const MAN_POST_NEAR: f32 = 40.0;
/// ...with an enemy the team knows of this near the post (XZ, m).
pub const MAN_SEEN_NEAR: f32 = 60.0;
/// At most this many turrets within [`MAN_POST_NEAR`] of one post are manned
/// or claimed by a team.
pub const MAN_POST_CAP: usize = 2;
/// Exit 1: this long (ticks, 10 s - the PS2's clock, re-soldier-ai 7.3)
/// with no target the turret can hit.
pub const MAN_NO_TARGET: u64 = 10 * crate::TICK_HZ as u64;
/// Exit 2: this long seated (45 s)...
pub const MAN_LONG: u64 = 45 * crate::TICK_HZ as u64;
/// ...with no shot fired in the last this-many ticks (20 s).
pub const MAN_NO_SHOT: u64 = 20 * crate::TICK_HZ as u64;
/// Exit 3: the turret under this share of its health.
pub const MAN_LOW_HEALTH: f32 = 0.3;
/// Exit 4: the endgame - a team down to [`HOLD_ENDGAME`] tickets, or the
/// gunner's own team down to this many living units.
pub const MAN_ENDGAME_ALIVE: usize = 3;
/// Exit 5: the purpose test has failed for this long (ticks, 10 s).
pub const MAN_PURPOSE_LOST: u64 = 10 * crate::TICK_HZ as u64;
/// After leaving, the turret is closed to that bot this long (20 s: the
/// PS2's unboard cooldown, re-soldier-ai 7.3).
pub const MAN_LEFT_CLOSED: u64 = 20 * crate::TICK_HZ as u64;
/// A turret the bot's walk did not reach is closed to it this long (60 s).
pub const MAN_UNREACHED_CLOSED: u64 = 60 * crate::TICK_HZ as u64;

#[cfg(test)]
mod tests {
    //! The fixtures the parts' test modules share; each part's own tests
    //! live beside its code, in that part's `mod tests`.
    use crate::nav::{self, Arc, Hub, NavGraph, RouteTable};
    use crate::sim::Sim;
    use crate::unit::UnitHandle;
    use glam::Vec3;

    /// An L of three hubs: the post sits at hub 2, which is NOT in the
    /// straight line from the unit's start — so a unit that reaches it walked
    /// the corridor.
    pub(super) fn l_graph() -> NavGraph {
        let hub = |x: f32, z: f32| Hub {
            pos: Vec3::new(x, 0.0, z),
            radius: 2.0,
            arcs: Vec::new(),
        };
        let mut hubs = vec![hub(0.0, 0.0), hub(0.0, -40.0), hub(40.0, -40.0)];
        hubs[0].arcs = vec![0];
        hubs[1].arcs = vec![0, 1];
        hubs[2].arcs = vec![1];
        let arcs = vec![
            Arc {
                a: 0,
                b: 1,
                type_mask: 1,
                flags: 0,
            },
            Arc {
                a: 1,
                b: 2,
                type_mask: 1,
                flags: 0,
            },
        ];
        // Authored table: from hub 0 to hub 2 take slot 0 (arc 0); from hub 1
        // to hub 2 take slot 1 (arc 1).
        let n = 3usize;
        let mut rows = vec![Vec::new(); crate::nav::TYPE_COUNT * n];
        let mut r0 = vec![0u8; n];
        r0[2] = 31 << 3; // slot 0
        rows[nav::SOLDIER * n] = r0;
        let mut r1 = vec![0u8; n];
        r1[2] = (31 << 3) | 1; // slot 1
        rows[nav::SOLDIER * n + 1] = r1;
        NavGraph::new(
            hubs,
            arcs,
            RouteTable::from_rows(n, crate::nav::TYPE_COUNT, rows),
            Vec::new(),
            Vec::new(),
        )
        .expect("graph")
    }

    /// Card BNDS: a deck (top y 10, x and z within ±10) with a hull falling
    /// west from its rim to y 4 at x -22, a `BARR` barrier along that rim, two
    /// hubs on the deck and a post at the first. No terrain: off the deck and
    /// the hull there is nothing. The AI unit starts at the post.
    pub(super) fn rim_sim(barrier: bool) -> (Sim, UnitHandle) {
        rim_sim_with(barrier, Vec::new())
    }

    /// [`rim_sim`] with more collision bodies (a rail on the rim).
    pub(super) fn rim_sim_with(
        barrier: bool,
        extra: Vec<crate::collision::Body>,
    ) -> (Sim, UnitHandle) {
        use crate::collision::{Body, CollisionWorld, Shape};
        use crate::cp::CpDef;
        use crate::region::Region;
        use crate::unit::{Controller, UnitSpec};
        use glam::Mat4;
        let deck = Body::new(
            Mat4::from_translation(Vec3::new(0.0, 9.75, 0.0)),
            Shape::Box {
                half: Vec3::new(10.0, 0.25, 10.0),
            },
        );
        let fall = (6.0f32 / 12.0).atan();
        let hull = Body::new(
            Mat4::from_translation(Vec3::new(-16.0, 6.75, 0.0)) * Mat4::from_rotation_z(fall),
            Shape::Box {
                half: Vec3::new(6.8, 0.25, 10.0),
            },
        );
        let mut sim = Sim::empty(7);
        let mut bodies = vec![deck, hull];
        bodies.extend(extra);
        sim.collision = CollisionWorld::new(bodies, Vec::new());
        let hub = |x: f32, arcs: Vec<u16>| Hub {
            pos: Vec3::new(x, 12.0, 0.0),
            radius: 4.0,
            arcs,
        };
        let mut g = NavGraph::new(
            vec![hub(0.0, vec![0]), hub(6.0, vec![0])],
            vec![Arc {
                a: 0,
                b: 1,
                type_mask: 1,
                flags: 0,
            }],
            RouteTable::empty(2),
            Vec::new(),
            Vec::new(),
        )
        .expect("graph");
        if barrier {
            g.barriers.push(nav::Barrier {
                centre: glam::Vec2::new(-10.0, 0.0),
                axes: [glam::Vec2::X, glam::Vec2::Y],
                half: glam::Vec2::new(0.5, 12.0),
                mask: 1,
                enabled: true,
            });
        }
        sim.nav = Some(g);
        sim.add_cp(
            CpDef {
                name: "cp1".into(),
                pos: Vec3::new(0.0, 10.0, 0.0),
                capture: Region::sphere(Vec3::new(0.0, 10.0, 0.0), 6.0),
                control: None,
                value_bleed: 100,
                value_atk: [100, 100],
                value_def: [100, 100],
                spawn_nodes: vec![],
                capture_time: 10.0,
                neutralize_time: 12.0,
            },
            Some(1),
        );
        let h = sim
            .spawn(&UnitSpec {
                controller: Controller::Ai,
                ..UnitSpec::trooper(0, Vec3::new(0.0, 10.0, 0.0))
            })
            .expect("spawned");
        (sim, h)
    }

    thread_local! {
        /// A scene's bots all STAND to shoot (`Some((true, true))`: none
        /// crouched, at any range; `Some((true, false))`: by the weapon's
        /// band, as a drawn stander) or all circle (`Some((false, _))`),
        /// whatever their own draw (`None`: the draw, and the weapon's
        /// band): the tests of the gunner, the mover and the held position
        /// script a shooter that keeps its place ([`all_stand`]).
        pub(super) static STYLE: std::cell::Cell<Option<(bool, bool)>> =
            const { std::cell::Cell::new(None) };
    }

    /// Resets [`STYLE`] when the scene ends.
    pub(super) struct Style;

    impl Drop for Style {
        fn drop(&mut self) {
            STYLE.with(|s| s.set(None));
        }
    }

    /// Every bot of this thread's scenes stands to shoot (card FGHT).
    pub(super) fn all_stand() -> Style {
        STYLE.with(|s| s.set(Some((true, true))));
        Style
    }

    /// Every bot of this thread's scenes is a stander by style only: it
    /// stands where the weapon's band says, as one drawn a stander does.
    pub(super) fn all_standers() -> Style {
        STYLE.with(|s| s.set(Some((true, false))));
        Style
    }

    /// Every bot of this thread's scenes circles.
    pub(super) fn all_circle() -> Style {
        STYLE.with(|s| s.set(Some((false, false))));
        Style
    }

    /// A sim with the L graph, flat ground and one post in a capture sphere at
    /// hub 2. The AI unit starts at hub 0.
    pub(super) fn l_sim() -> (Sim, UnitHandle) {
        use crate::cp::CpDef;
        use crate::heightfield::Heightfield;
        use crate::region::Region;
        use crate::unit::{Controller, UnitSpec};
        let mut sim = Sim::empty(7);
        sim.heightfield = Some(Heightfield {
            size: 2,
            unit: 400.0,
            offset_x: 200.0,
            offset_z: 200.0,
            heights: vec![0.0; 4],
        });
        sim.nav = Some(l_graph());
        sim.add_cp(
            CpDef {
                name: "cp1".into(),
                pos: Vec3::new(40.0, 0.0, -40.0),
                capture: Region::sphere(Vec3::new(40.0, 0.0, -40.0), 6.0),
                control: None,
                value_bleed: 100,
                value_atk: [100, 100],
                value_def: [100, 100],
                spawn_nodes: vec![],
                capture_time: 10.0,
                neutralize_time: 12.0,
            },
            Some(1),
        );
        let h = sim
            .spawn(&UnitSpec {
                controller: Controller::Ai,
                ..UnitSpec::trooper(0, Vec3::ZERO)
            })
            .expect("spawned");
        (sim, h)
    }
}
