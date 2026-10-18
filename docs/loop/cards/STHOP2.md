---
id: STHOP2
title: STHOP's leftovers (cleanse CLNC): a soldier at rest on an open steep face stands (the wedge rule), a sprint cannot start on a ramp (the gate reads XZ speed), the steep-face contact removes the whole normal velocity, crouch's airtime — measure each on the PS2, then match
milestone: M3
value: 4
phase: measure
status: todo
deps:
files:
declares:
needs:
effort:
branch:
resume:
created: 2026-09-30
alpha: yes
---
## Why
Found by cleanse CLNC's review of STHOP (81aae53 / de3b9c1 / eb90b56), each reproduced with a scratch test on main 5043dc3 + comments (report `reports/20260930_2125_cleanse_core.md`). All four are movement a player feels on any map with ramps or cliffs.

## Evidence (port numbers measured 2026-09-30; scratch test = Soldier::step + resolve_ground on an analytic plane, dt 1/30)
1. **Wedged on an open face.** `resolve_ground`'s wedge test asks only whether the feet's height changed by < `WEDGE_MAX_DY` (5 mm) over the tick. A soldier put at rest on an OPEN plane of n.y 0.6 and of n.y 0.3 (72 deg) is grounded on 90 of 90 ticks, vel 0, stance Stand, in both games — he stands on a cliff with full control (spawn, the separate_units y-snap, a vehicle exit, the top of a slide; walking along a contour passes the same test). The PS2 stands him only when his steep contacts' MEAN normal has y > 0.6 (re-soldier-ground.md 1.4) [D].
2. **Sprint on a ramp.** The sprint gate (`planar_speed >= 0.9 * max_speed`, soldier.rs) and the roll impulse read XZ speed; since STHOP the speed is along the plane. Holding forward + sprint from rest, 120 ticks: BF2 rules UPHILL never sprints at n.y 0.95 (XZ 7.72, 3-D 8.12, gate 8.10), 0.90, 0.826, 0.75; downhill sprints at tick 4 on all. BF1 rules: n.y 0.90 never sprints either way (XZ 7.29, 3-D 8.10); 0.95 sprints. re-stance-roll.md says the PS2 compares 'actual speed' — which speed, and whether the PS2's own uphill scaling also blocks a sprint, is unmeasured.
3. **Steep-face contact.** The port removes the whole into-face velocity on a face of n.y <= 0.6; the PS2's wall contact flattens the velocity's copy of the normal (only horizontal velocity removed) and lifts by the true normal (re-soldier-ground.md 1.3). With BF2's full air control the port turns stick speed into climb speed up the face [inferred, not measured].
4. **Crouch airborne** stands up at 0.3 s and then falls at air_time > 1.0 s in total; BF2's Stand case may zero its timer on its first airborne update (1.3 s total) — the latch after crouch is unread [H].

## Acceptance
- PS2 numbers for each (BF2: `lead_tmp/sthop/ramp_bf2.py` sampler + the RAM hold of `pcsx2/walk_bf2.py`; BF1: `ps2live.py`): (1) the player released at rest on an open face of n.y < 0.6 — grounded bit and speed after 1 s; (2) forward + sprint from rest up and down KAM1's ramp (n.y 0.826) and one ~18 deg ramp — the tick the sprint stance starts, or never; (3) stick held into a ~55 deg face for 2 s — height gained; (4) step off an edge crouched — time to the Fall stance.
- Then build: the same four scratch runs on the port match the PS2 (grounded yes/no; sprint tick within 3; height within 0.3 m; fall time within 2 ticks), as unit tests in soldier.rs; sweep_seeds.py finishes not below main's; the leak census read per class (float / floor / push) before the gates.

- Planner 2026-10-02 14:45: `files:` emptied while the phase is `measure` (the PS2 numbers come first; the list held `sim/step.rs`). Restore when you set phase=build: `engine/crates/core/src/soldier.rs,engine/crates/core/src/sim/step.rs` (JARC and BF1BK edit `soldier.rs` too: one at a time).
