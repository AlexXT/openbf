# OPUS RE (2026-10-12) - [O] UNAUDITED

# What a soldier's roll looks like on the PS2: one clip, and the body turned into the roll (BF1 and BF2)

Card ROLLS, lane 2. Corpora: B1 = `analysis/SWBFRONT.ELF-decomp.c`, B2 = `analysis/SWBFRNT2.ELF-decomp.c`; ELF words by
`off = va - 0x160000 + 0x1000`; the discs' `zaa_` banks; live PS2 RAM and video (BF1 Theed, BF2 Coruscant, 2026-10-12).
Tags: **[V]** read at the cited lines or instructions, **[M]** measured in PS2 memory, on the disc or in a frame, **[D]**
derived, **[H]** hypothesis; `[O]` marks the load-bearing claims of this unaudited note. Scratch, tools and data:
`engine/artifacts/lead_tmp/rolls/`. Builds on `re-stance-roll.md` (the roll's mover and gates) and
`re-soldier-anim-layers.md` (the animator: the two clip players, the kept pose, the leg twist).

## 0. Summary

The maintainer's ask (INBOX 2026-10-12 18:55, 2): the port plays the forward roll while the soldier moves sideways; the PS2's
sideways roll looks different. Read in both games' code and seen on the PS2:

1. **There is one roll clip.** `[O]` Every roll, in every direction, plays `diveforward` - a forward dive and
   somersault. Neither game has a left, right or backward roll clip: none in the animators' name tables, none as a
   string in either ELF, and none under any of the 20 and 26 million names tried against the two discs' clips
   (sections 1, 6).
2. **The body is turned into the roll.** `[O]` While the clip plays, the animator turns the skeleton's root bone
   (`DummyRoot`) about the up axis toward the direction the soldier travels, a quarter of the remaining angle per update.
   BF1: a fixed 90 degrees to the roll's side. BF2: the angle of the soldier's own movement against his facing, taken
   afresh every update. So a left roll is the same forward dive, made head first to the soldier's LEFT, while the
   camera and the aim keep pointing forward; when the roll ends the body turns back the same way (sections 2-4).
3. **It is the same for the player and the bots.** `[O]` The animator is fed the soldier's stance and position by the
   soldier's own code, whoever controls him; an AI soldier's roll on BF1 showed the same rows and picture (section 4.3).
4. **Not every direction can be rolled.** BF1 rolls only sideways (stick full left or right with Cross; forward, back
   and the diagonals jump). BF2 rolls in any direction within 120 degrees of straight ahead; with the stick further
   back the button is the crouch (section 5; this corrects `re-stance-roll.md` 4.2, which said 30).

**So the PS2's picture is not "the body keeps facing forward".** What keeps facing forward is the camera and the
soldier's aim; the drawn body turns about 90 degrees and dives head first to the side. The port's fault is that it
plays the dive WITHOUT that turn (the body faces forward and somersaults forward while it slides sideways). Strips for
The maintainer: `engine/artifacts/lead_tmp/rolls/sheet_ps2_bf2.png` (forward, left, right, diagonal, and the refused back roll),
`sheet_ps2_bf1.png` (left, right, and the jump a forward press gives), `sheet_ps2_bf1_bot.png` (an AI soldier's roll).

The port today: `game/src/animator.rs` plays the right clips (each weapon set's own `diveforward` pair is already
baked) and has no root twist at all. Section 7 is the change list: one input, one eased angle, no clip and no bake
change.

## 1. The clip [V][M]

- **BF1.** The animator's state setter `FUN_0022a0a8(animator, state, matrix, crc)` sends states 10 and 0xb - and
  nothing else - to special animation id 0x14, with a 0.3 s fade owed to whatever state comes next (B1 142229-142234:
  `+0x78 = 0x14`, `+0x20c0 = 0x3e99999a`) `[O]`. Id 0x14 of the special-name table at ELF 0x517af0 is `diveforward`
  (word 0x517b40 -> 0x534ff0). The state is the soldier's stance: the soldier's animator driver `FUN_002013a0` loads a1
  from soldier+0xbe4 (instructions 0x2019ec-0x2019fc; B1 113791), and stance 10 / 0xb are roll right / roll left
  (`FUN_00209488` / `FUN_002093d0`, B1 118647-118726; `re-stance-roll.md` 2).
- **BF2.** `FUN_0027bae0(animator, state, matrix, crc)` sends states 5 and 0xd to mode 0x18 (B2 200460-200462,
  200625-200627, the store at 200683) `[O]`; id 0x18 of the simple-name table at ELF 0x5fde48 is `diveforward` (word
  0x5fdea8 -> 0x620f40). The driver `FUN_00239bb0` loads a1 from soldier+0x7e4, the stance (0x13 instead when bit
  0x2000 of soldier+0x4e0 is set and the stance is 0; instructions 0x23a99c-0x23a9c4, B2 153823-153827). Stance 5 is
  the roll (`FUN_002471b0`, B2 162224). Stance 0xd is mapped to the same clip but looks unreachable: of the stance
  setter `FUN_00247bf8`'s 26 call sites 25 pass a constant and none of them is 0xd (`re_check.py callers`); the one
  variable call was not traced [H].
- Each setter has three more callers, a second object that keeps its own state word (B1 `FUN_001be0c0` /
  `FUN_001be1e8`, B2 `FUN_0037fa48` / `FUN_0037fd30`): not the soldier, not read [H].
- **Per weapon set** (the two clip players, `re-soldier-anim-layers.md` 3.6 / 7): the legs' player takes the rifle's
  clip, the upper body's the set's own where the disc has one. On the discs [M] (`checks/disc_dive.py`):

  | game | clip | frames / tracks | who plays it |
  |---|---|---|---|
  | BF1 | `human_rifle_diveforward` | 35 / 26 | every soldier's legs; the upper body of the rifle, bazooka, bdroid and lascannon sets |
  | BF1 | `human_tool_diveforward` | 49 / 16 | the upper body of the tool, pistol, grenade and sabre sets |
  | BF1 | `human_sbdroid_diveforward` | 49 / 26 | the super battle droid's upper body |
  | BF2 | `human_rifle_diveforward` | 35 / 26 | both players of the rifle and bazooka sets; the legs of every `human`-family set |
  | BF2 | `human_tool_diveforward` | 49 / 16 | the upper body of the tool class's sets and its children's (pistol, melee) |
  | BF2 | `sbdroid_rifle_diveforward` | 49 / 26 | the super battle droid's upper body |
  | BF2 | `grevious_sabre_diveforward_full` | 36 / 32 | both players of Grievous's sabre set |
  | BF2 | `gam_sabre_diveforward` 35 / 25, `gamlz_rifle_diveforward` 35 / 14 | | the Gamorrean guard's bank (GAM.LVL) and its low-detail twin; sets not rebuilt here [H] |
  | BF2 | `humanlz_rifle_diveforward` | 2 / 14 | the low-detail soldier (the ELF's `humanlz` clip list names `rifle_diveforward`, 0x5fddd0) |
  | both | `human_bazooka_stand_diveroll_forward` | 55 / 17 | nobody: no table and no ELF string names it [D] |
  | BF1 | `human_rifle_dive2prone` | 39 / 26 | nobody, likewise [D] |

  Every one of them is a FORWARD dive. The last two are leftovers: the builders form names only from the ELF's
  tables, and neither `diveroll` nor `dive2prone` is a string in either ELF.

  In RAM during the rolls of section 4 the legs' clip is one pointer for the whole roll, name hash 0xdc2308eb
  (`human_rifle_diveforward`), 35 frames, in both games, and the player's upper body holds the same clip; the BF1 AI
  soldier of 4.3 held 0xf6192200 (`human_tool_diveforward`, 49 frames) on his upper body over the rifle's on his legs
  [M].
- **The clip is cut short and played at its own speed.** The roll's stance ends at 0.75 of the clip (`re-stance-roll.md`
  3.6 / 4.5): 27 updates on BF1, 26 on BF2, the legs' phase 0.029 on the first update and 0.772 / 0.765 on the last
  [M]. BF1 hands the clip the plain update time (`$f12 = $f21` at 0x22ad24); BF2 a rate of 1.0 (`$f13` at 0x27ce6c). It
  is not fitted to the roll's speed.
- **No other roll clip exists.** The search, as counts: section 6.

## 2. BF1: the turn [V]

`FUN_0022aa10`, the pose function, switches on the special id (animator+0x78). Case 0x14 (B1 142590-142596; the jump
table word at 0x534ba0 is 0x0022acf4; instructions 0x22acf4-0x22ad2c):

```
angle = -1.5707964                      lui 0xbfc9 / ori 0x0fdb -> $f13
if animator.state (+0x60) == 0xb:       the LEFT roll
    angle = +1.5707964                  lui 0x3fc9 / ori 0x0fdb -> $f13
FUN_0022c7f0(dt, angle, 0.0, animator, special id, 0)        the special-animation player; no weapon action (a2 = 0)
```

`FUN_0022c7f0` poses the two players (the two `FUN_0022bbf0` calls, B1 143653-143656) and then (143665-143666):

```
twist (+0x8c) = twist + (angle - twist) * 0.25
FUN_0022bcc0(w, animator)               w = the weight the LEGS' player was just blended with
```

`FUN_0022bcc0` (B1 142931-142985) finds the bone of name hash 0x4446e9b8 - crc32 of `dummyroot`, bone 0 - and
multiplies its quaternion in the kept pose (animator + 0x1c80 + 0x20 x index) by a rotation about (0, 1, 0) of
`twist x w` `[O]`.

- The angle is positive toward the soldier's LEFT: the animator's row 0 is his left (`re-soldier-anim-layers.md` 3.1;
  +90 degrees off his forward row in every live row), and the left roll (state 0xb, the stick left) is the one given
  +pi/2.
- The stored twist is the same field the walking soldier's leg twist uses (the plain case moves it 25 % of the way to
  the strafe angle each update while he moves, B1 142836-142837, and turns the same bone, 142849-142851). So a roll
  STARTS from the strafe twist the soldier had - on BF1 an alert soldier strafing left runs on the backward clip with
  his legs at -72 degrees, and his roll swings the body from -71 through 0 to +90 - and after the roll the same 25 % a
  update takes it back.
- `w` is the legs' layer weight (`FUN_0022bbf0`: the state fade x the clip's own blend-in, 0.15 s for the dive): for
  the roll's first 4 to 8 updates the bone is turned by less than the stored twist [M: section 4.2].
- Of `FUN_0022c7f0`'s nine call sites (all in `FUN_0022aa10`: B1 142444, 142454, 142478, 142483, 142523, 142568,
  142588, 142595, 142599) only the roll's passes a constant angle; five pass 0, one the old twist x 0.9 (lands, get-ups,
  prone moves), two the stored twist (the thrown tumbles, which set it from the fall's direction; the second of these,
  142588, is also reached from case 5 with 0).
- The 0.3 s owed fade: on the next change of state the fade length (+0x20bc) becomes the larger of its old value and
  0.3 (B1 141963-141969): the pose leaves the dive over 0.3 s.

## 3. BF2: the turn [V]

`FUN_0027caf0`, the pose function, switches on the mode (animator+0x20b8). Case 0x18 (B2 201331-201348; jump table
word at 0x620a10 = 0x0027ce20; instructions 0x27ce20-0x27ce80):

```
move  = animator+0xc0                   this update's change of position (FUN_0027bae0, B2 200197-200214;
                                        zeroed past 10 m and on the first update after a reset)
angle = atan2f(move . row 0, move . row 2)        FUN_0052b090 = atan2f; row 0 his left, row 2 his forward
FUN_0027d9a8(dt, 1.0, angle, 0.0, animator, mode, 0)         the simple-animation player; no weapon action
```

`FUN_0027d9a8` poses the two players and then (B2 201533-201555):

```
target = angle, or 0 when the upper clip's AimType is FullBody (byte +0x20e6 == 4)
twist (+0x20cc) = twist + (target - twist) * min(1, 7.5 * dt)          0.25 at 30 updates a second
FUN_00280048(twist, animator)
```

`FUN_00280048` (B2 202741-202775) WRITES the quaternion (0, sin(twist / 2), 0, cos(twist / 2)) into the kept pose of
the bone of name hash 0x4446e9b8 (animator + 0x910 + 0x20 x index) `[O]` - an overwrite, where BF1 multiplies.

- The target is the direction of TRAVEL against the facing, read again every update: any roll direction gets its own
  angle (a diagonal roll about 37 degrees), and a roll that curves is followed.
- `diveforward` carries no AimType in the disc's `sanm` chunk (`lead_tmp/anim/sanm_bf2.txt` line 27), so the gate
  never zeroes a roll's target. The byte is written from the upper clip's descriptor (`FUN_0027f458`, B2
  202445-202450; `FUN_0027fb50`, 202612-202615, from an argument); it is reset to 0 in `FUN_0027b710` and has its two
  high bits cleared in `FUN_00280158` - no other writer sets the value 4.
- The same field is the walking soldier's leg twist in BF2 too (`re-soldier-anim-layers.md` 7): a left strafe already
  holds about +71 degrees when the roll starts, so BF2's left roll only moves the body 19 degrees more.
- BF2 does know how to give a direction its own clip - and uses it for jumps, not rolls: `FUN_0027b9e0` (B2
  200127-200162) picks mode 1 to 5 (`jump`, `jump_forward`, `jump_backward`, `jump_left`, `jump_right`) from the
  move's components when the set has them (the sabre sets do). The roll's case has no such choice.
- A bot's roll is the same stance: the AI's command path `FUN_00249f80` sets the soldier's velocity to the direction
  its controller asks for (or his forward row) and calls the roll action through vtable slot +0xb0, without the
  energy cost (B2 164571-164593); the animator then sees stance 5 and the movement like anyone's.

## 4. Seen on the PS2 [M]

`lead_tmp/rolls/ps2/roll_strip.py bf1|bf2`: a no-AI battle (BF2 Coruscant, the rebel soldier; BF1 Theed with the AI
units parked), a lane found clear, and each roll made to travel along it (the camera turned so that the stick's
direction lies on the lane). One row per game update: the soldier's block and the whole block of the animator that
poses him, beside a video stamped with the wall clock. `analyse.py <out> <game> [trial] --rows` prints the rows;
`sheet.py` cuts the strips. Angles are degrees, positive = the soldier's left; "travel" = the velocity's heading less
the body's; "twist" = animator+0x8c (BF1) / +0x20cc (BF2).

### 4.1 BF2 (Circle with the stick held)

| stick | stance | travel | twist on update 1 / 2 / 3 / 5 / 8 / 12 / last | travel - twist from update 9 on |
|---|---|---|---|---|
| forward | 5 for 26 updates | +0.0 | -0.1 / -0.0 / -0.0 / -0.0 / -0.0 / +0.0 / +0.0 | 0.0 |
| left | 5 x 26 | +90.0 | +76.0 / +79.5 / +82.1 / +85.6 / +88.1 / +89.4 / +89.9 | at most 1.4, mean 0.3 |
| right | 5 x 26 | -90.1 | -76.0 / -79.5 / -82.1 / -85.6 / -88.2 / -89.5 / -90.1 | at most 1.4, mean 0.3 |
| forward-left | 5 x 26 | +37.9 | +36.5 / +36.6 / +36.7 / +36.8 / +37.0 / +37.5 / +38.1 | at most 0.7, mean 0.2 |
| forward-right | 5 x 26 | -37.5 | -36.7 / -36.8 / -36.8 / -37.0 / -37.1 / -37.2 / -37.8 | at most 0.2, mean 0.1 |
| back | none: 0 -> 1, a crouch | | | |
| back-left (135), pressed in that crouch | none: 1 -> 0, he stands | | | |

The animator's mode is 0x18 on every roll row; the pose buffer's root quaternion is a pure yaw equal to the twist on
every row; the roll's speed is 10.5 m/s in every direction. Before a side roll the strafing soldier's twist already
reads +-71; after it the twist goes back to the strafe's angle, and once he stands still the turn-on-the-spot clip
takes it to 0 at 9.6 degrees an update. A second left roll repeats the first to 0.1 degree.

### 4.2 BF1 (Cross with the stick full left or right)

| stick | stance | travel | twist on update 1 / 2 / 3 / 5 / 8 / 12 / last | travel - twist from update 9 on |
|---|---|---|---|---|
| left | 11 for 27 updates | +90.0 | -31.2 / -0.9 / +21.8 / +51.6 / +73.8 / +84.9 / +89.9 | at most 12.1 (update 9), mean 2.5 |
| right | 10 x 27 | -90.0 | +31.1 / +0.8 / -21.9 / -51.7 / -73.8 / -84.9 / -89.9 | at most 12.1, mean 2.5 |
| forward, back, forward-left, back-left | 4: a jump | | | |

On every roll row (twist[k] - twist[k-1]) / (+-90 - twist[k-1]) is 0.250: the code's rule, from the -71.6 / +71.6 the
strafe left. The special id is 0x14 on every roll row. The root quaternion's yaw lags the twist on the first updates
- the weight of section 2: the session's first roll for 4 (-62.6, -35.2, +2.9, +34.9 against the twist's -31.2, -0.9,
+21.8, +38.9; equal from the 5th), the rolls after it for 8 (right roll: +70.5, +63.6, +44.6, +11.5, -23.6, -48.7,
-64.1, -72.8; equal from the 9th), a 0.3 s fade being still owed from the roll before. So the DRAWN body is within 15
degrees of the travel direction from the 9th or 10th update (0.3 s) on. After the roll the twist goes back through 0
toward the strafe's angle and the pose follows over the 0.3 s fade. A second left roll repeats the first's twist to
0.1 degree.

### 4.3 An AI soldier (BF1)

One AI soldier put 6 m ahead of the player; its roll request word (soldier+0x3cc left / +0x3d0 right, which only the
AI's soldier controller writes: `re-stance-side.md` A) written once. The stance goes to 11 / 10 on the next update for
26 updates, the animator shows special id 0x14 and the twist goes +16.6 ... +89.9 (left) and -21.8 ... -89.9 (right)
by the same 0.250 a update; the picture is the same dive to his side, seen from in front (`sheet_ps2_bf1_bot.png`).
Two of the four bot trials are unusable as pictures: an animator is stepped only while its soldier is drawn (in one
the soldier stood beside the camera and his animator did not move for the whole roll; in another it froze from the
16th update as he left the view).

### 4.4 The strips

`sheet_ps2_bf2.png`, `sheet_ps2_bf1.png`, `sheet_ps2_bf1_bot.png`: eight frames a roll at -0.07, 0.10, 0.23, 0.37,
0.50, 0.65, 0.80 and 1.10 s from the roll's first update, each labelled with the stance and twist of the row it shows
(the recorded picture runs 0.05-0.09 s of game time behind the rows on BF1 and 0.12-0.22 s on BF2; `latency.py`
measures each trial's lag from its standing start and the sheet takes it off). What they show: forward - the soldier's
back, a forward somersault; left / right - the body turned side on to the camera, diving head first to that side,
upside down at 0.5 s, on its feet and turning back at 1.1 s; the camera does not turn.

## 5. Which way a soldier can roll [V][M]

- **BF1**: the jump button with the stick over 0.9 sideways and more sideways than forward (`re-stance-roll.md` 3.2):
  left or right only. Forward, back and both diagonals (0.71 sideways) with Cross gave a jump (4.2).
- **BF2**: `FUN_00247b88(fwd, strafe)` (B2 162687-162700; called from the stand and the crouch cases, B2 160313,
  160383) is true when `0.81 <= fwd^2 + strafe^2` and (`0 < fwd` or `fwd x -1.732 < |strafe|`). With the stick behind
  the side axis that needs |strafe| > 1.732 x |fwd|: at most 30 degrees behind it. **A roll goes anywhere within 120
  degrees of straight ahead; the 120 degrees behind are refused** and the button is the stance change. Live: the
  stick straight back crouched him, and at 135 degrees it stood him up from that crouch (4.1). `re-stance-roll.md`
  4.2 and 4.5 said "except the straight-back +-30 degree cone"; the code and the port's `Soldier::roll_predicate`
  (which copies the code) are right, that sentence was not. Corrected there.
- So neither game has a backward roll, and BF1 has no forward one.

## 6. The search for another roll clip

- **ELF strings** (`checks/tables.py` -> `checks/tables.txt`: every ASCII string of four characters or more holding
  dive, roll or tumble, both ELFs): the soldier clip names among them are `diveforward` (B1 0x534ff0, B2 0x620f40),
  `rifle_diveforward` (B2 0x620c18, in the low-detail soldier's clip list: the pointer to `humanlz` is at 0x5fdda8,
  the list starts at 0x5fddb0) and `thrown_tumblefront` / `thrown_tumbleback`. `roll`, `IdleToRoll`, `Rolling`,
  `RollToIdle` are the droideka's (beside `fold` / `unfold`); the rest are sound and flyer words. `RollLeft` /
  `RollRight` are keys of a table of camera curves read beside `Crouch`, `CrouchToProne`, `StandToProne` (B1
  `FUN_001911a0`, lines 35354-35356, instructions 0x19125c / 0x191278) [D]; in B2 the two strings (0x617100,
  0x617110) are referenced by no pointer word and no `lui` / `addiu` pair [M]. `diveleft`, `diveright`,
  `divebackward`, `diveback`, `dive_left`, `dive_right`, `rollforward`, `rollbackward`: 0 hits in both decomps and
  both ELFs [receipt:3abb577f].
- **Name tables**: BF1's 31 special names (0x517af0) and BF2's 38 simple names (0x5fde48) are listed whole in
  `checks/tables.txt`; one dive each.
- **State setters**: in the animators' code 0x14 is stored to the special id at one place (B1 142231) and 0x18 is
  assigned to the mode at two (B2 200461, 200626) - the roll cases of section 1.
- **The discs** (clip names are stored only as CRC-32, so names are tested, not searched). `checks/disc_dive.py`
  reads every `zaa_` bank of every LVL file - BF1 120 files, 39 bank chunks, 312 distinct clips; BF2 165 files, 113
  bank chunks, 637 distinct clips - and tests candidate names: families (49 named ones and every bank's own name) x
  16 weapon classes x 8 prefixes x 16 words (dive, roll, tumble, dodge, diveroll, combatroll, sideroll, dive2prone,
  roll2stand, evade, sidestep, flip, ...) x 27 direction forms x 4 layer suffixes: 20,378,736 names on BF1,
  26,129,520 on BF2 (`checks/disc_dive_bf1.txt`, `disc_dive_bf2.txt`).
  - BF1: 8 hits. Five are the forward dives of section 1. Three are hash collisions with clips the tables do name
    (1.5 chance hits expected): `mace_shotgun_rollsideways` = `human_rifle_thrown_landbackmedium` (one frame),
    `anakin_lightsabre_standalert_dodgerollsideways_lower` = `human_rifle_jetpack_hover`,
    `obiwan_rifle_sprint_rolldivef_full` = `human_sbdroid_stand_walkforward`.
  - BF2: 16 hits. Eight are forward dives. One is the `jetpack_hover` collision again. Seven sit in a hero's or a
    creature's bank under another family's name (3.9 chance hits expected; a name prefix that collides brings every
    suffix with it): five were given their real names - `mace_sabre_jumpattack_land_full`,
    `dooku_sabre_jumpattack_fall_full`, `sidious_sabre_stand_walkbackward_full`,
    `sidious_sabre_stand_walkforward_full`, `acklay_sabre_standalert_walkforward_full` - and two were not: a
    ONE-frame clip in Grievous's bank and a 33-frame clip in Maul's (hit by a name with a `minigun` class Maul has
    not) [H].
  - A test that needs no name list (the first verifier's, re-run): BF1's `human_animation_2`, the bank that holds
    the rifle's dive, stores its 55 clips in alphabetical order of their full names and all 55 are named;
    `human_rifle_diveforward` sits between `human_rifle_crouch_walkforward` and `human_rifle_jump`, and it is the
    bank's only name with dive or roll in it.
  - Limits: only `zaa_` chunks are read; a clip under a name outside the grammar is not seen (the first run's
    narrower grammar - 1.8 million names - missed `stand_diveroll_forward`, `dive2prone` and the bank-named
    families; both verifiers of this claim found them, and neither supported "nothing else" as it then stood). So
    "no sideways roll clip on the discs" is [H] beyond the names tried. What makes the answer is the code: the
    animator plays only the clips its tables name, and they name one dive.

## 7. The port against this, and the build

Today (main c1ad907): `game::animator::body` sends `Stance::Roll` to `Special::Roll`, and `Rig::body_slots` gives it the
set's own pair by the name `diveforward` - the PS2's clips, already baked (52 of 52 baked set files name
`human_rifle_diveforward`, 50 `human_tool_diveforward`, 21 `human_sbdroid_diveforward`, 1
`sbdroid_rifle_diveforward`). There is no root twist: the model matrix is the soldier's position and aim yaw
(`soldier_vis.rs` `posed`), so the body faces the aim while it dives. The port has one roll for both games' maps
(BF2's, any direction the predicate allows - the maintainer's locked choice), and bots roll through the same `Soldier` (card
REACT).

1. **One input.** The roll's direction is already in the sim for every unit: `Soldier::vel` against the facing.
   `soldier_vis.rs` computes `horiz` and the aim frame for each unit - player and `ai` alike - where it fills
   `AnimInput`; add `travel = atan2(horiz . left, horiz . forward)` there (radians, positive = his left; undefined
   under about 0.3 m/s: keep the last). No `core` edit.
2. **One eased angle in `game::animator`.** `Animator::twist`: every update `twist += (target - twist) x k`, k the
   PS2's 0.25 an update at 30 updates a second made frame-rate free (`1 - 0.75^(30 dt)`; BF2's own form is min(1,
   7.5 dt)). Target: `travel` while the body is `Special::Roll`, else 0 until card ANIMM gives the walking soldier
   his strafe twist (the same field on the PS2 - ANIMM then only supplies the target outside a roll).
3. **Turn the root.** After the two players are applied, rotate the kept pose's `DummyRoot` about +Y by the twist:
   BF2 maps set its rotation outright; BF1 maps multiply the clip's by `twist x w`, w the legs' blend weight. The
   port strips root motion and draws at the soldier's position, so the turn is about his feet, as on the PS2.
4. **Sign.** The rig's +X is the soldier's left on the PS2 (bone names and row 0). The port's test fixes it: a left
   roll must put the dive's head on the soldier's left.
5. Not for this card: BF1's own side-roll mover (card BF1BK), the strafe twist and the turn on the spot (ANIMM), the
   first-person `RollLeft` / `RollRight` camera clips, the low-detail soldier's dive.

Acceptance numbers the build can be held to (from section 4): a side roll's body is within 15 degrees of the travel
direction from the roll's 9th update on (PS2: BF2 1.4, BF1 12.1) and within 2 at its end; a forward roll's stays
within 1 degree of 0; after the roll the twist is back under 10 degrees within 0.5 s.

## 8. Checks run

Scratch `engine/artifacts/lead_tmp/rolls/`: `claims.md` (the claim list the verifiers were given), `checks/` (`fn_b1.txt`,
`fn_b2.txt`, `tables.py` / `tables.txt`, `disc_dive.py` / `disc_dive_bf1.txt` / `disc_dive_bf2.txt`), `dis/` (the
disassembly of the seven functions), `ps2/` (`roll_strip.py`, `analyse.py`, `latency.py`, `sheet.py`; `out/`: 19 trials
of rows and video; `out_a/`: a first run's rows), the three sheets.

1. **fn** - `re_check.py fn` over every cited line (`checks/fn_b1.txt`, `fn_b2.txt`).
2. **absent** - `re_check.py absent b1 diveleft diveright divebackward diveback dive_left dive_right rollforward
   rollbackward --elf --card ROLLS`: 0 hits in both decomps and both ELFs [receipt:3abb577f]; the ELF string census
   and the disc's candidate count of section 6.
3. **twin** - each BF1 function with its BF2 twin opened: state setter `FUN_0022a0a8` / `FUN_0027bae0`, pose function
   `FUN_0022aa10` / `FUN_0027caf0`, special player `FUN_0022c7f0` / `FUN_0027d9a8`, root turn `FUN_0022bcc0` /
   `FUN_00280048`, soldier driver `FUN_002013a0` / `FUN_00239bb0`. They differ in the target (a constant by state /
   the travel angle), the step (0.25 a call / min(1, 7.5 dt)), the write (a product with the legs' weight / an
   overwrite) and BF2's AimType gate.
4. **word** - ELF 0x22acfc / 0x22ad10 (the two constants), 0x534ba0 and 0x620a10 (the jump-table entries), the two
   name tables, crc32 `dummyroot` = 0x4446e9b8 (`checks/tables.txt`).
5. **writers** - the BF2 gate byte animator+0x20e6: four writing functions (`FUN_0027b710` resets it, `FUN_0027f458`
   and `FUN_0027fb50` set it, `FUN_00280158` clears two high bits), the value 4 only from an AimType of 3;
   `callers`: `FUN_0022c7f0` 9, `FUN_0027d9a8` 8, `FUN_0027bae0` 4, `FUN_0022a0a8` 4, `FUN_00247b88` 2,
   `FUN_00247bf8` 26.

**Verifier workflow** (`rolls-re-verify`, 7 agents at medium effort, claims and citations only; the 6 absence and
twin claims went to two verifiers working apart): 39 claims. First reading of each: **30 SUPPORTED, 9 PARTLY, 0 NOT
SUPPORTED**; the second reading of the six: 2 SUPPORTED, 4 PARTLY, and it overturned one SUPPORTED (that BF2's state
0xd is a roll state - now [H], section 1). All PARTLY are folded in above, each re-read or re-run first: two line
ranges; one call site of `FUN_0022c7f0` (B1 142588) has a second entry that passes 0; the plain case's 0.25 easing is
the moving soldier's arm only (standing, the twist follows the turn on the spot); the gate byte's second writer and
its other two; the back-left press was made from the crouch; one bot trial's animator froze as he left the view;
BF2's `RollLeft` / `RollRight` strings are referenced by nothing; the disc census (section 6: both verifiers found
forward dives the first name list missed and named the stray one-frame hit; the claim was restated and its "nothing
else" dropped to [H]). The workflow's own finds: the alphabetical-bank test, BF2's directional jump chooser, the second
object that calls both state setters, and that BF1 multiplies the root's rotation under a weight where BF2 overwrites
it.

What stays **[H]** or unread: a sideways roll clip under a name nobody tried; BF2 stance 0xd; the second caller of the
state setters (an object with its own state word); the low-detail soldier's roll (`humanlz`: does it turn?); the
Gamorrean guard's sets; the first-person side of a roll (`re-stance-roll.md` 4.5's BF2 line is not supported by the
string check above); what a roll does to the picture when the soldier is not drawn (an animator is stepped only then);
BF2's back-diagonal between 90 and 120 degrees (the pad keys give 45-degree steps: not measured live); no BF2 AI
soldier's roll was filmed (the code path is read, section 3). Not built or compared with the port: the build card.
