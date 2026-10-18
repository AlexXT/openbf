# Bot quality: the PS2's soldiers beside the port's (card BOTM, 2026-10-02)

Measured, not RE'd: live PS2 RAM against the port's sweep, through ONE instrument (`scripts/bot_quality.py`). Read it
with `analysis/re-conquest-goal.md` (what a PS2 bot is told to do) and card SAIR (how it does it, not RE'd yet).

## The instrument
- **Port:** `round_to_outcome` with `BOT_SAMPLES=<dir>` writes every alive unit 4 times a second of game time (ticks
  8, 15, 23, 30 ...: the black box trail's rate) plus every shot, hit and kill with both teams. 4 seeds (1-4) x 540 s;
  `sweep_seeds.py --botq` runs it over any seeds and maps and puts the tables below in `seeds.md`.
- **PS2:** live RAM from PCSX2 (SW renderer), sampled every 0.25 s of WALL time and stamped with GAME time (BF2 the
  mission clock, BF1 the EE cycle counter / 294.912 MHz). The machine was loaded (load 18-44), so the emulator ran at
  33-100 % speed and the samples are 0.08-0.25 s of game time apart: denser than 4 Hz, never sparser.
  BF2: `lead_tmp/botm/tools/bf2_player.py` (= card CONT's `cont_ps2.py`, 16 AI a team, GCW, plus the player SPAWNED
  and his health pinned); BF1 KAS2: `lead_tmp/botm/tools/bf1_ps2.py` (`ps2live.BF1.soldiers`, teams 1-2 only, a CW
  battle from `bf1_kas2_cw_cis_cp5_cp10_dock_one_spawn.p2s`, the player standing at Dock One).
- **One form for both:** each life (spawn to death; a jump over 15 m on foot also cuts it) is put on a 4 Hz grid of game
  time by linear interpolation (a raw gap over 0.8 s cuts the life), so a source's own rate does not shape a number.
- **Reversal** = two consecutive 0.25 s steps, each over 5 cm, whose directions oppose (dot < 0). Window = 40 points
  (10 s, the trail's length). **Staying** window: under 4 m net. **Pinball** window: staying AND (over 6 reversals OR
  over 15 m of path) - card JITR's bar. Target-post changes: from a life's first order (CONT's rule; member+0x18 is
  re-drawn every second until the controller first thinks); the port's from its objective's post (`commander.of`),
  changes to or from `hunt` not counted.
- **Checks run** (`bot_quality.py --selftest`, 7/7 ok): an oscillator given 19 / 9 turns in 10 s reads 19 / 9; the same
  oscillator sampled at an uneven 5.4 Hz reads 9; a straight runner reads 0 reversals, path / net 1.000, 100 % running;
  a stander with 1 cm noise reads 0; three standers read 66.7 % "teammate within 1.5 m" and 0 / 100 % contact 20 / 60 m.
  **The maintainer's snapshot** (`lead_tmp/jitr/snapshot_20261002_kam1_f9.json`, `--snapshot`): #30 reads **26 reversals**,
  46.1 m path, 3.1 m net (the card's numbers); #18 11, #19 8. **PS2 target changes** on CONT's own capture: one per
  308 foot unit-seconds (CONT's `analyse.py`: 323). **Port contact** within 20 m: 3.1 % on KAM1 seed 1-4 against the
  sweep's own `shares` line 2.5 % (seed 1).

## The PS2's bots wait more when nobody plays (why the PS2 column has the player spawned)
With the player left on the spawn map every PS2 controller thinks at its slowest interval: a life's first order came
4.00 s after spawning (median, 161 lives); with the player spawned it came 1.07 s (near him) / 2.04 s (far) after.
KAM1 standing still 55.7 % (parked) -> 43.3 % (spawned); POL1 39.1 -> 24.0 %. The tables use the spawned captures;
the parked ones (`lead_tmp/cont/ps2/bf2_{kam1,pol1}_members.csv`, `bf2_dea1_botm_members.csv`) stay as a reference.

## The maintainer's scene on the PS2 (BF2 Kamino, an invincible player held at (-4.0, 60.3, 108.6))
`bf2_player.py bf2_kam1 --at -4.0,60.3,108.6`, mission 30-180 s (the player was one-shot at 180 s; the pin is 5 ms).

| bots by mean distance to the player | 10 s windows | reversals mean | windows over 6 | pinball | standing still |
|---|---:|---:|---:|---:|---:|
| PS2, within 20 m | 49 | 1.98 | 0 | 0 | 55 % |
| PS2, over 20 m | 329 | 0.61 | 0.6 % | 0.6 % | 42 % |
| port, the maintainer's F9 snapshot, within 20 m (6 bots, last 10 s) | 6 | 9.00 | 3 of 6 | 2 of 6 | - |

The PS2's bots near a player do turn more often (strafing, 2 a 10 s) but never pinball; a PS2 bot that stays put
stands: its staying windows walk 0.1-2 m in 10 s, the port's 6-20 m.

## The four maps (PS2 | port median of 4 seeds (min .. max) | port / PS2; furthest from 1 first)
Full tables (every row, the port-only rows): `lead_tmp/botm/out/<map>.md`, heat maps `<map>_heat.png` (PS2 left).

**BF2 KAM1** (PS2 mission 28-255 s; port 41-540 s)

| measure | PS2 | port | port / PS2 |
|---|---:|---:|---:|
| in a vehicle or turret seat (%) | 9.4 | 0.0 | 0.00 |
| idle 20 s and not fighting (%) | 21.1 | 1.3 (0.5 .. 5.0) | 0.06 |
| path in a staying window (m) | 1.3 | 13.8 (11.6 .. 20.4) | 10.7 |
| reversals in a staying window | 0.36 | 3.76 (2.99 .. 4.42) | 10.4 |
| inside a 20 s spell within 2 m (%) | 28.1 | 4.5 (4.3 .. 9.7) | 0.16 |
| pinball windows (%) | 1.22 | 5.90 (5.26 .. 6.41) | 4.8 |
| an enemy within 20 m (%) | 15.1 | 3.1 (2.7 .. 4.8) | 0.21 |
| top-3 cells' share of deaths (%) | 42.3 | 16.6 (14.0 .. 20.3) | 0.39 |
| standing still (%) | 43.3 | 18.2 (17.5 .. 22.6) | 0.42 |
| reversals per bot per 10 s | 0.77 | 1.50 (1.33 .. 1.60) | 1.9 |
| a teammate within 1.5 m (%) | 6.6 | 12.5 (11.6 .. 16.6) | 1.9 |
| running over 3.5 m/s (%) | 51.7 | 72.5 (72.3 .. 77.5) | 1.4 |
| path / net | 1.26 | 1.33 | 1.06 |
| deaths per minute | 20.6 | 18.2 (15.4 .. 20.7) | 0.88 |

**BF2 POL1** (PS2 mission 22-182 s)

| measure | PS2 | port | port / PS2 |
|---|---:|---:|---:|
| in a vehicle or turret seat (%) | 18.0 | 0.0 | 0.00 |
| windows with over 6 reversals (%) | 1.2 | 12.2 (10.4 .. 20.3) | 10.0 |
| idle 20 s and not fighting (%) | 6.5 | 1.1 (0.6 .. 3.9) | 0.17 |
| target-post changes per bot-minute | 0.168 | 0.035 (0.028 .. 0.190) | 0.21 |
| reversals in a staying window | 1.07 | 4.86 (4.70 .. 6.75) | 4.5 |
| a teammate within 1.5 m (%) | 5.7 | 21.9 (20.1 .. 27.8) | 3.9 |
| pinball windows (%) | 3.33 | 8.14 (7.12 .. 13.65) | 2.4 |
| reversals per bot per 10 s | 0.97 | 2.18 (2.15 .. 3.27) | 2.2 |
| deaths per minute | 35.8 | 16.8 (16.1 .. 19.6) | 0.47 |
| standing still (%) | 24.0 | 19.8 (19.6 .. 25.3) | 0.83 |
| path / net | 1.41 | 1.53 | 1.08 |

**BF2 DEA1** (PS2 mission 39-213 s)

| measure | PS2 | port | port / PS2 |
|---|---:|---:|---:|
| idle 20 s and not fighting (%) | 3.2 | 0.0 (0.0 .. 0.5) | 0.00 |
| target-post changes per bot-minute | 0.550 | 0.027 (0.011 .. 0.110) | 0.05 |
| an enemy within 20 m (%) | 18.6 | 1.9 (1.8 .. 2.8) | 0.10 |
| reversals in a staying window | 0.44 | 3.52 (1.99 .. 4.07) | 7.9 |
| windows with over 6 reversals (%) | 1.3 | 3.8 (3.7 .. 5.1) | 3.0 |
| a teammate within 1.5 m (%) | 9.3 | 21.5 (21.2 .. 25.2) | 2.3 |
| reversals per bot per 10 s | 0.79 | 1.30 (1.25 .. 1.37) | 1.65 |
| deaths per minute | 30.3 | 35.9 (28.9 .. 43.6) | 1.18 |
| path / net | 1.38 | 1.36 | 0.99 |

**BF1 KAS2** (PS2 165 s of a CW battle; port GCW-or-mission default, 41-540 s)

| measure | PS2 | port | port / PS2 |
|---|---:|---:|---:|
| inside a 20 s spell within 2 m (%) | 32.4 | 0.3 (0.0 .. 1.2) | 0.01 |
| path in a staying window (m) | 1.3 | 19.3 (19.3 .. 23.7) | 15.5 |
| reversals in a staying window | 0.36 | 4.38 (3.77 .. 5.94) | 12.1 |
| staying windows (%) | 37.7 | 4.2 (3.4 .. 4.9) | 0.11 |
| windows with over 6 reversals (%) | 1.0 | 6.6 (5.7 .. 11.0) | 6.3 |
| a teammate within 1.5 m (%) | 2.0 | 11.8 (10.2 .. 13.4) | 5.8 |
| standing still (%) | 56.8 | 14.4 (14.3 .. 14.8) | 0.25 |
| running over 3.5 m/s (%) | 39.3 | 80.4 (78.7 .. 80.7) | 2.0 |
| in a vehicle or turret seat (%) | 10.8 | 10.8 (9.6 .. 11.3) | 1.00 |
| deaths per minute | 25.1 | 32.3 (31.7 .. 34.4) | 1.29 |

**Port only** (no PS2 reading yet: kills by team need EVMSG's sites or a last-hit field - card FFLN step 1):

| map | team kills / kills | friend hits / hits | hits / shots | shots while moving | shots / bot-min | wall slide | slide flips / 10 s |
|---|---:|---:|---:|---:|---:|---:|---:|
| BF2 KAM1 | 15.6 % | 15.7 % | 20.4 % | 50.3 % | 16.8 | 9.6 % | 0.7 |
| BF2 POL1 | 24.8 % | 28.0 % | 27.9 % | 39.2 % | 11.3 | 12.2 % | 1.4 |
| BF2 DEA1 | 25.8 % | 25.2 % | 19.9 % | 45.5 % | 33.9 | 4.7 % | 0.7 |
| BF1 KAS2 | 18.4 % | 15.3 % | 22.2 % | 51.2 % | 22.8 | 8.8 % | 1.2 |

## What the numbers say (for JITR, FFLN, OBST, BOTD)
1. **A port bot that stays, jitters; a PS2 bot that stays, stands.** Every map: the staying window's path is 6-20 m in
   the port against 1-9 m, its reversals 3.5-4.9 against 0.4-1.1. Pinball windows 2-8 % of all windows (PS2 1-3 %).
   The worst-five list is led by this on 3 of 4 maps. JITR's target.
2. **The port never stands for long.** 20 s spells within 2 m: 0.3-8 % of foot time against 8-32 %; standing still
   14-23 % against 24-57 %; running 68-80 % against 39-69 %. The PS2's bots take a spot and wait (often not fighting:
   21 % of KAM1's foot time).
3. **Port bots bunch.** A teammate within 1.5 m: 12-22 % against 2-9 % (x1.9-5.8) - the maintainer's "standing next to each
   other". 
4. **Port bots fight from range.** An enemy within 20 m: 2-4 % against 15-19 % on KAM1 / DEA1 (CONT measured the same
   gap); deaths per minute are close (0.88-1.29) except POL1 (0.47).
5. **Turrets:** BF2 PS2 bots spend 9-18 % of their time in a seat on KAM1 / POL1; the port's 0.0 % (BF1 KAS2 matches
   at 10.8 %, the speeders). Map use (cells walked, half-time cells) is within x0.8-1.5.
6. **Team kills 16-26 % of the port's kills** with friendly fire at 1.0 (FFLN). The PS2's share, measured by FFLN
   (re-blaster-damage.md section 6.3): 12.1 % on BF2 KAM1 (11 of 91, one battle) - its bots shoot through their
   friends too. With FFLN's line-of-fire rule the port's is 0.6-3.1 % per map (1.7 % over 32 maps x 8 seeds).

## Proposed gate lines (not gates: BOTD sets the bars; some deliberately better than the PS2, the maintainer's wish)
- `pinball windows <= max(1.5 x PS2, 2 %)` per map with a PS2 column; the maintainer's scene: 0 bots over 6 reversals within 20 m.
- `reversals in a staying window <= 2 x PS2` and `path in a staying window <= 3 m`.
- `a teammate within 1.5 m <= 1.5 x PS2`.
- `team kills / kills <= PS2's share` (12.1 % on BF2 KAM1; the port's rule holds 3.1 % or less on every map) -
  FFLN's acceptance.
- Bands, not one-sided: `standing still`, `20 s spells`, `an enemy within 20 m` within x0.67-1.5 of the PS2's -
  a port bot that stands mute for ever must fail too.
- `deaths per minute` within x0.67-1.5 (rounds must still be fights).

## Caveats
- The PS2 runs are one capture each, 160-227 s of BF2 battle with the player at his spawn and 165 s of BF1; the port's
  are 4 seeds x 500 s. The PS2's numbers move with whether a player is in the battle (above), and probably with where.
- BF1 KAS2: the PS2's battle was minutes old and CW, with team 3 (Wookiee locals) left out of every count.
- "Seated" on BF2 PS2 = member+0x14c set (a vehicle or a turret); BF1 = the soldier's two position copies over 2 m apart.
- "Fighting" = the port's bot has a target; the PS2's controller's order is 2 (fight). Not the same thing.
