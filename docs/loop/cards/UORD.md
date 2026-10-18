---
id: UORD
title: Primaries that are not bolts fly straight in the port: missile (the torpedo launchers), shell (the IMP and CIS pilots' mortar), emitterordnance (the clone pilot's arc caster), bullet (the jet trooper's EMP launcher) - 13 of the 34 fielded BF1 classes
milestone: M3
value: 4
phase: re
status: todo
deps: HVYW
files:
declares:
needs:
effort:
branch:
resume:
created: 2026-09-29
alpha: yes
---

## Why
UAUD1 (analysis/unit-audit-bf1.md §3.1). The port builds every primary's ordnance as a straight bolt: core/src/weapon.rs 180-181 "Gravity
is deliberately NOT read ... bolts fly straight". The disc's ordnance BASEs
(`engine/artifacts/lead_tmp/unit-audit/census_ia.md`, "Weapons") say otherwise for 13 of the 34 fielded classes.

## Evidence
- missile: `*_weap_inf_torpedo_launcher_ord` (vanguard x3, shock trooper x2, ARC trooper, assault droid; the Wookiee
  rocketeer local).
- shell: `imp_weap_inf_mortar_launcher_ord1` (pilottie, pilotatst, pilotatat), `cis_weap_inf_mortar_launcher_ord1`.
- emitterordnance: `rep_weap_inf_arccaster_ord` (clone pilot).
- bullet: `rep_weap_inf_rocket_launcher_ord` (jet trooper).

## Acceptance
- RE ([O]) per ordnance class, both games: flight (guidance and lock time, the gravity arc, the emitter's beam or
  arc), damage and splash.
- Build in `core`; sim / sweep declared. Grenades (sticky / grenade kinds) are GREN's; vehicle weapons share these
  classes (VTUR).

## Notes
- UAUD2 (2026-09-29, analysis/unit-audit-bf2.md §3.1): BF2 has 26 of 49 fielded primaries that are not bolts: missile 12 (9 rocket
  launchers, 2 EMP launchers, the MagnaGuard's Bulldog), bullet 10 (9 shotguns, the clone commander's chaingun),
  emitterordnance 4 (3 incinerators, the dark trooper's arc caster).

## Planner note (2026-09-30, M-ALPHA)
`alpha: yes` — rockets and mortars that fly straight. Value 3 (kept). Showcase maps: BF1 NAB1, BF1 TAT1 (the port's `tat1`), BF1 YAV2, BF2 UTA1, BF2 MYG1, BF2 HOT1 (PLAN §Milestones M-ALPHA).

## Note from GREN (2026-09-30, analysis/re-grenades.md §4.1, §7.3)
The shell's flight is read and measured there: gravity = `Gravity` x -9.8 added to velocity.y before the move, 30 Hz,
position += velocity x min(life, dt); under water velocity -= (0.7 dt / (0.7 dt + 1)) x (vx, vy + 0.5, vz) (B1
`FUN_0027e478`, B2 `FUN_003202d0`); launch speed = `Velocity` lerped toward the `NextCharge` class's by the charge (B1
`FUN_002762a0`); the life timer and `ExplosionExpire` are the base's (B1 `FUN_00272e98`). The AI's aim for any ordnance
with gravity is the low-arc solver (§7.3). The mortar launchers are `grenade` WEAPONS (the throw cycle of §3) firing a
`sticky` / `shell` ordnance — card GREN builds that cycle and the sticky.

- GREN landed 2026-10-02 (d4886ba): a channel-0 slot (primary or the sidearm a player switches to with R3) is still
  built by `weapon_from_db` as a bolt weapon, so the mortar launchers (a `grenade` weapon + `sticky` ordnance on
  channel 0: BF1 pilots' primary, the smuggler's and BF2 officers' second slot) fire straight bolts. The thrown path
  exists (`thrown_weapon_from_db`, `Sim::launch` — the primary's shot loop would only need to call `launch`); it was
  kept off channel 0 because bots hold these as primaries and have no ballistic aim yet (card GRAI).

## Note (planner 2026-10-12 23:40): raised 3 -> 4, after HVYW
- Card HVYW (built on branch `hvyw`, not landed) makes rocket troops fire at vehicles and at groups of 3+ - and its
  report says a rocket at a group hits ONE man, because the port's missile is a straight bolt with no blast. So the
  blast (the ordnance's explosion: radius, damage by distance, who it pushes) is the first slice of this card, before
  the arcs of the shell and the grenade launcher. Measure with HVYW's census (`lead_tmp/hvyw/tools/hvcen.py`):
  soldiers hit per rocket fired at a group, against the PS2's ExplosionName numbers on the disc.
- `deps: HVYW` for order (both edit `core/src/weapon.rs`). Undo: `loopctl.py set UORD value=3 deps=`.
