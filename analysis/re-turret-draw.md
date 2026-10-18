# OPUS RE (loop lane 2, 2026-10-13, card VTURD) - [O] UNAUDITED
# RE: a building turret that turns with its aim and shows its gunner - the posed nodes, the gunner's draw rule, seat and pose, the views, the dead turret

**The questions** (card VTURD, the maintainer's INBOX 2026-10-12 18:55, ask 1). The port draws a building turret as a static
model: the gun never turns and nobody is seen in the seat. What does the PS2 do? (1) Which node takes the yaw and
which the pitch, per turret class. (2) Which classes show their gunner and which hide him - the rule and a census of
every armed-building class on both discs. (3) Where the gunner sits, how he is posed, whether he turns with the gun.
(4) What the gunner himself sees (first / third person). (5) What a destroyed turret draws.

**Method.** A session that died at its usage limit (2026-10-12 23:42 - 00:14) left three reader reports and PS2
captures; this session treated them as claim lists. Two more readers took the code they had not opened (the
gunner's matrix; the view tests). The formulas were then replayed on the PS2's own RAM, one row per game update,
with a human in four turrets (BF1 Naboo gun turret; BF2 Hoth dish turret and laser tower, Mygeeto anti-aircraft
gun) - 1,838 rows - and every class fact was counted by script over the raw LVL chunks of both discs. A verifier
workflow (17 agents, claims and citations only; every twin and absence claim to two of them) judged 101 claims:
82 supported as written, 19 supported in substance with a scope, gate or citation put right, none refuted
(section 9).

Tags: [V] read in the decomp / ELF at the cited lines, [L] measured on the running PS2, [D] derived, [H]
hypothesis; [O] = load-bearing, lifted, unaudited. B1 = `SWBFRONT.ELF-decomp.c`, B2 = `SWBFRNT2.ELF-decomp.c`.
T = a MountedTurret, K = its class, S = the seat block in K (BF1 K+0x50, BF2 K+0x60), A / I = an aimer's class
record / instance, B = the owning building, E = a soldier. A matrix is 4 rows (X, Y = up, Z = forward, translation);
a point is `p.x*row0 + p.y*row1 + p.z*row2 + row3`; "M1 x M2" applies M1 first.

## 0. Short answers

1. **The turret node carries the yaw, the aimer node the whole aim** [O][V][L]. `TurretNodeName`'s node is set to
   a look-along matrix of `(yaw x YawTurnFactor, pitch x PitchTurnFactor)` at its bind position. Every building
   turret with a seat authors `PitchTurnFactor = 0.0` except one (BF1 Bespin's anti-aircraft gun), so that node
   turns and never tilts. `AimerNodeName`'s node is set to a look-along matrix of the AIMER's own direction - the
   gun's real line of fire, which chases the turret's aim point at the aimer's rates and can sit 2 to 21 degrees off
   the camera's aim when the target is near. A `BarrelNodeName` node copies its parent's matrix and slides back
   along it by `state x BarrelRecoil`, the state jumping to 1 at a shot and falling by 4 a second. Nothing else
   moves: a table entry is one node's whole model-space matrix and children do not follow a parent.
2. **A gunner is drawn when his seat has a pose key** [O][V][L]: `PilotAnimation` or `Pilot9Pose`. BF1 looks at
   nothing else. BF2 also hides him when `IsPilotExposed = 0`, which no building turret authors. By the census: BF1
   6 classes drawn, 1 hidden (Hoth's laser tower), 2 with no soldier at all; BF2 15 drawn, 7 hidden (Hoth's laser
   tower among them), 47 with no soldier. He is drawn in every view, his own included, without his hand weapon.
   Read in RAM on Hoth: seated in the dish turret the player's soldier is in the scene list with a body (state
   0x11); seated in the laser tower he is out of the scene list and has no body.
3. **He rides one node** [O][V][L]: `soldier = L x E x O` - the `PilotPosition` hardpoint's bind matrix relative to
   its parent, the CURRENT matrix of that one parent node, the building's world matrix. Under the turret node (gun
   turrets, the dish, the towers: 15 of 24 seats) he swings round with the yaw and stays upright. Under the aimer
   node (the anti-aircraft guns: 7 seats) he swings and tilts with the barrels, one update late. A hidden gunner
   sits at the building's origin.
4. **His pose is one clip** [O][V][L]: `human_<value>` from the soldier banks. A `PilotAnimation` clip (`drive`: 1
   frame) simply plays. A `Pilot9Pose` clip has 9 frames: the turret hands `a = yawFrac + 4`, `b = yawFrac + 7` or
   `+ 1`, `c = |pitchFrac|` and the pose is frame a blended toward frame b by c - a 3 x 3 grid of yaw against
   pitch. For a turret that turns full circle the yaw term is the turn STICK, not the angle.
5. **A building turret is always third person** [O][V][L]: the camera rides the turret's aim frame, so the gun
   stays where it is on the screen and the world turns. There is no first-person turret view on the pad. The one
   other view is the zoom (R3): the camera goes to the eye point, the scope frame is drawn, and the WHOLE building
   is left out of that one viewport.
6. **A dead turret is a static model** [O][V][L]: its tasks stop, the angles freeze, the building swaps to
   `DestroyedGeometryName` (state 1) - and none of the 49 destroyed models on the discs holds a posed node. The
   state machine is `re-build-zones.md` 1.

## 1. The census [O][L - counted by script over the raw LVL chunks]

`lead_tmp/vturd/turcensus2.py bf1|bf2` reads every `entc` of every LVL (BF1 120 files / 710 class names, BF2 165 /
1,416), applies a child's props after its parent's, and keeps the classes with a TURRET section whose root label
is `armedbuilding`, `commandarmedanimatedbuilding`, `portableturret` or `defensegridturret`: **BF1 9, BF2 69**.
`nodecensus.py` joins each with its model's `skel` (which node hangs from which). Tables: section 10.

- Gunner verdicts (PilotType self / remote = no soldier; a pose key = drawn; else hidden): **BF1 6 / 1 / 2**
  (drawn / hidden / no soldier), **BF2 15 / 7 / 47**.
- `IsPilotExposed`, `ControlsUnit`, `AnimatesUnitAsTurret`, `Virtual`, `MountPos`, `HierarchyLevel`, `NextAimer`,
  `ForceMode`, `YawTurnFactor` and `AnimatedPilotPosition` are in none of the 78 TURRET sections: 0 hits in the key
  lists of both discs [receipt:ee16ab66] (control: `PitchTurnFactor` and `TurretNodeName` are found by the same
  search). `IsPilotExposed = 0` is authored elsewhere - on flyers' turret sections (BF1 cloud car and gunship, BF2
  the gunships and the snowspeeder's gunner; `turcensus2.py <game> --veh`) - so BF2's second term is alive, for
  vehicles.
- `PitchTurnFactor = 0.0` on every section with a `TurretNodeName` but four: BF1 `bes_bldg_aaa_turret` and the two
  fambaa classes, BF2 `uta1_bldg_turret` (default 1.0: that node tilts too).
- The six showcase maps (map file / class x placed): BF1 NAB1 `com_weap_gunturret` x21, TAT1 x12, YAV2 x6 (drawn,
  `minigun_9pose`, gunner under the turret node). BF2 HOT1 `tur_bldg_hoth_dishturret` x16 (drawn,
  `dishcannon_9pose`), `tur_bldg_hoth_lasermortar` x14 (hidden), `tur_bldg_chaingun_tripod` x13 and `_roof` x1
  (self-piloted); MYG1 `tur_bldg_recoilless_lg` x9 (drawn, `drive`, gunner under the aimer node),
  `tur_bldg_recoilless_myg_auto` x4 (self); UTA1 `uta1_bldg_turret` x2 (self; the one BF2 class whose turret node
  tilts).
- **Every turret model splits exactly into rigid per-bone parts** (`softcensus.py`): a hard-skinned segment is on
  one bone, a soft one has every vertex on one bone at full weight, and no triangle has vertices on two bones in 7
  of 8 BF1 models and 23 of 24 BF2 models (6,838 and 15,796 triangles). The exceptions are the fambaa (an animated
  creature, 241 triangles) and the space-map `imp_bldg_boxturret` (8 of 434).
- The seat clips exist under `human_<value>` on both discs (`clipcensus.py`): `human_minigun_9pose` 9 frames,
  `human_dishcannon_9pose` 9 frames, `human_drive` 1 frame, 26 tracks each - BF1 bank `human_animation_1`
  (COMMON.LVL), BF2 bank `human_4` (INGAME.LVL). The bare `<value>` is in no bank. The port bakes none of them yet.

## 2. The posed nodes [O]

### 2.1 What the class keeps
- [V] `TurretNodeName` (0xb69a7cb4) stores four things, no matrix: the node's name hash, its PARENT's name hash,
  the node's bind position relative to its parent and its bind Y axis relative to its parent - BF1 K+0x1118 /
  0x111c / 0x10f0 / 0x1100 (`FUN_00225f08`, B1:139193-139219), BF2 K+0x818 / 0x81c / 0x7f0 / 0x800 (`FUN_00270770`,
  B2:191473-191499). The helper (`FUN_004969f8` B1:557437 / `FUN_0057d6e8` B2:712985) writes `M_node x
  inverse(M_parent)` (a rigid inverse) and returns the parent's hash; the two vectors are stored only when the node
  has a parent. Key 0xf2aa4a90 (`MountPos` by its hash) writes the same pivot field; it and the two unnamed keys
  below are in no building TURRET section [receipt:a5ad4948]. A node's name hash is the engine's CRC, not the keys'
  FNV (`re-anim-banks.md` 2.3).
- [V] Defaults (`FUN_00225cd8` B1:138857-138903, `FUN_00270540` B2:191365-191402): both factors 1.0; the yaw and
  pitch limits -10 / +10 stored RAW, which the tick reads as "no limit, wrap" (B2:189323-189385).
- [V] `AimerNodeName` (0xc182abc2): node hash, parent hash, bind position and bind rotation (a quaternion) relative
  to the parent - BF1 A+0x350 / 0x354 / 0x330 / 0x340 (`FUN_0026e260`, B1:192515-192543), BF2 A+0x110 / 0x114 /
  0xf0 / 0x100 (`FUN_00302f10`, B2:294254-294286). Aimer defaults (B1:192340-192384, B2:294080-294125): pitch
  +-45 deg, yaw +-30 deg, pitch rate 1.0, yaw rate 2.0 rad/s.
- [V] `BarrelNodeName` (0x1e534b12): a 0x30-byte record per barrel - node hash, the barrel's parent hash, its bind
  POSITION relative to the parent (no rotation), `BarrelRecoil` (0x5a29d1cb) at +0xc (B1:192569-192586,
  192475-192483; B2:294311-294328, 294218-294225). `FirePointName` is a muzzle offset only: no node is posed for it.

### 2.2 The turret node (in the turret's tick)
- [V] BF1 `FUN_00223850` B1:137537-137635, BF2 `FUN_0026cea0` B2:189395-189477, statement for statement:

      a = yaw   * YawTurnFactor          b = pitch * PitchTurnFactor
      D = ( sin a cos b, -sin b, cos a cos b )                      model axes
      P = bind position, U = bind Y  - each first carried through the PARENT's entry when it has one
      X = normalize(U x D)    Z = normalize(D)    Y = Z x X
      node = rows ( X, Y, Z, P )         BF1 T+0x320, BF2 T+0x200; registered under the node's own hash

  While the seat holds a player and the building is alive, yaw is `atan2(x, z)` of the aim in the building's frame
  and pitch is `-asin(y)` (B1:137405-137408, B2:189298-189300): **a positive turret pitch is nose DOWN**; yaw
  turns +z toward +x. For a `Virtual` class (BF1 K+0x1164, BF2 byte K+0x84a) the direction, the basis, the store and
  the table entry are skipped (P and U are still carried through the parent) - a key on neither disc
  [receipt:7710e328].
- [V] The same basis from the UNSCALED angles, times the building's world matrix, is the turret's **aim frame**
  (BF1 T+0x2e0, BF2 T+0x1c0; B1:137732-137812, B2:189567-189711): the eye point, the aim point and the camera
  hang from it, and it is the aimer's base frame. (BF2 only: a class with byte K+0x84c set, key 0x865061fc, takes
  the direction from the owner's main controllable instead, MIPS 0x26e3b0-0x26e48c - 3 PROP chunks on the BF2
  disc, none in a building TURRET section [receipt:a5ad4948].)
- [L] `replay_nodes.py <tag>`: the node matrix in RAM equals the formula on **758 of 758** BF1 rows (`nab1_gt`
  407, `nab1_gty` 351) and **1,077 of 1,080** BF2 rows (`myg1_lg` 342 of 342, `hot1_dish` 368 of 369, `hot1_tower`
  367 of 369; the BF2 sampler does not read a row atomically). The aim frame fits on 758 and 1,073. Alternatives
  on the same rows: full pitch on the node 71-142 rows a capture, no yaw 31-39, yaw negated 31-39.
- [V] The sticks reach the angles through a rate: BF1 `rate += (target - rate) x min(Filter x dt, 1)`
  (B1:137456-137466), BF2 a linear slew of `Filter x dt` toward `-stick x speed` (B2:189229-189262), the speed
  `MaxTurnSpeed` / `MaxPitchSpeed`; then `angle += dt x rate` and the clamp to `YawLimits` / `PitchLimits`
  (B2:189319-189385). This is the sim's side (the port's seat look is PROVISIONAL, `core/src/vehicle/gun.rs`).

### 2.3 The aimer and its node
- [V] One aimer chain per weapon section, its own task. Its base matrix is the turret's AIM FRAME when the class
  has a `TurretNodeName`, else the building's world matrix (B1:136779-136783, B2:188004-188011).
- [V] Each update it steers its own yaw and pitch toward the turret's aim POINT (BF1 T+0x20, BF2 T+0x170: the eye's
  ray out to its hit, at most 512 m): `c = dir x want`; `pitch -= clamp(c . M.row0, +-AimerPitchRate dt)`,
  `yaw += clamp(c . M.row1, +-AimerYawRate dt)`; clamp to `AimerPitchLimits` / `AimerYawLimits`;
  `dir = sin(yaw)cos(pitch) M.row0 + sin(pitch) M.row1 + cos(yaw)cos(pitch) M.row2` (BF1 `FUN_0026d748`
  B1:191984-192005, `FUN_0026dd70` B1:192112-192221; BF2 the headerless tick at 0x3021f8, MIPS 0x30269c-0x302754,
  `FUN_00302a10` B2:293851-293962). **The aimer's pitch is positive UP**, the opposite of the turret's. Equal
  limits pin an angle; for a BOT in the seat the wanted direction first goes through the AI's own function
  (`FUN_00236700` B1:191960-191968, `FUN_002a83e0` MIPS 0x302660-0x302694; not read - card VAIM's ground); a
  `ControlsUnit` seat with a human skips the clamp and the direction write - a key with 0 PROP chunks among the
  597 and 781 class keys of the two discs [receipt:7710e328].
- [V] The node (BF1 `FUN_0026ce10` B1:191242-191366, BF2 `FUN_003018d8` MIPS 0x30190c-0x301b1c):

      L = the node's bind transform relative to its parent (quaternion + position)
      F = the aimer's direction, rotated into the building's model space
      U = L.row1 x parentEntry (rotation)        P = L.row3 x parentEntry (with translation)
      R = normalize(U x F)     Up = F x R        node = rows ( R, Up, F, P )

  So the aimer node is NOT "the pitch": it looks along the gun's whole direction, from a place that swings with the
  turret node. Written by the aimer's own update and again by the owner's draw for every section that has an
  aimer (`FUN_002255b0` B1:138485-138494, `FUN_0026f610` B2:190702; BF2 may swap section 0's direction for that
  call, B2:190669-190708 - not read).
- [V] Barrels, same functions (B1:191371-191449; MIPS 0x301b20-0x301cc8): `entry(barrel) = entry(parent)`, then
  row 3 = `(bind offset, z less state x BarrelRecoil when state > 0) x entry(parent)`. A shot sets the current
  barrel's state to 1.0 and steps to the next barrel (B1:191494, B2:293613-293632); every update takes `4.0 x dt`
  off every state, with no floor (B1:192018-192023; MIPS 0x3027f0-0x30282c) - it runs to -260 in the captures and
  slides only while above 0. `BarrelLength` also writes the recoil field, as a quarter of the length.
- [L] `replay_aimer.py`: the aimer node's matrix fits **342 of 342** BF2 rows (`myg1_lg`) and **351 of 351** BF1
  rows (`nab1_gty`); "the node is the turret's aim basis" fits 0 at 5e-4 (75 and 110 within a degree). The
  aimer's direction is off the turret's aim by a median 2.7 deg (max 5.4) on the Mygeeto gun and up to 21 deg on
  the Naboo gun turret aimed at the ground beside it: the aim POINT is near and the muzzle is not the eye. Barrels:
  the parent's rotation on 342 of 342 rows each, row 3 with the slide on 341 and 342 ("no slide": 334). Recoil
  decay: median 3.996 (BF2, 12 falling steps) and 4.000 (BF1, 21) per second. What the rows cannot tell apart:
  "up through the parent" from a constant (0, 1, 0) (both parents are pure yaws), and no BF1 barrel matrix was
  captured - those two are code readings.
- [V] The turret tick calls no aimer function in either game - none of the 46 / 48 call sites of the aimer's
  functions is in `FUN_00223850` / `FUN_0026cea0` [receipt:fbfe404d] [receipt:0f74ebe8]. The two are tasks joined
  by two pointers (the aim frame, the aim point); the turret's runs first, so a reader of the aimer node inside the
  turret tick sees last update's (section 3.2).

### 2.4 The table and the draw
- [V] The pose table is a complete per-node pose keyed by the node's own name: the owner fills it with a pointer
  to the skeleton's bind matrix for every named node nobody posed (`FUN_00498ad8` B1:559330-559366, `FUN_0057f088`
  B2:714470-714510); the draw fetches each segment's matrix by its node's name and multiplies by the object's
  matrix (`FUN_00497b80` B1:558414, `FUN_00496248` B1:557316-557330; `FUN_0057e740` B2:713960, `FUN_0057cf60`
  B2:712875-712884). **Nothing composes parents**: a node under a posed node stays at its bind matrix unless
  something writes its own entry - the aimer and barrel code do exactly that.
- [D] So what moves on a turret is the turret node, each aimer node and each barrel node, and no other. By the
  census every DRAWN bone of the ground turrets is one of those or the base.

## 3. The gunner [O]

### 3.1 Drawn or hidden
- [V] BF1, the soldier's seat slot `FUN_0020c880` (B1:121207; Controllable vtable 0x532be0 +0x6c): with
  `animHash == 0` it takes his collision body and his scene object out and returns (B1:121244-121254); else scene
  object in, body in only when `exposed != 0`, the three pose numbers and the hash stored, soldier state 0xc
  (B1:121289-121335). `IsPilotExposed` (S+0x220, default 1) reaches nothing but that body test (B1:156266): in BF1
  it means "can be shot", not "is drawn".
- [V] BF2 `FUN_0024de60` (B2:167409; vtable 0x61e998 +0x8c): *(decompiler detail omitted)* - no pilot
  animation OR `IsPilotExposed` 0 - body out, scene object out, return (B2:167517-167526). Else state 0x11.
- [V] Defaults: no animation (BF1 0, BF2 -1), exposed 1 (B1:155800-155816, B2:284986-284988).
- [V] He is drawn without his hand weapon: BF1 state 0xc skips the weapon draw (B1:114261-114266), BF2 state 0x11
  likewise (B2:155344-155345). The states that keep it need `ControlsUnit` / `AnimatesUnitAsTurret`, which have 0
  PROP chunks on either disc [receipt:7710e328] (control: `PilotAnimation`, `IsPilotExposed` and `ForceMode` are
  found by the same search, [receipt:c7f4019a]).
- [L] `ps2/sheet_hot1_bots2_dish_zoom.png`: bots on two of BF2 Hoth's dish turrets - a soldier in his own side's
  uniform standing behind the dish, both hands on the grips, no rifle. A manned laser tower shows nobody
  (`ps2/sheet_hot1_bots2_mortar_zoom.png`), but a picture cannot tell "not drawn" from "inside the pillar", so the
  flags were read (`ps2/turret_b2y.py`, the player on a no-AI Hoth): on foot the soldier's in-scene byte (E+0x118)
  is 1, his body is in the world and his state is 0; **in the laser tower the byte is 0, the body word 0, the state
  8**, through rest, turn, zoom and back; **in the dish turret 1, a body, state 0x11**, zoomed too. In RAM the
  tower's gunner sits at the building's origin in all 369 rows.
- [V] The soldier's own zoom flag, which would hide him in his own view, is cleared at Enter (B1:120188 *(decompiler detail omitted)*, B2:165203) and his own update is unlinked while he sits [D].

### 3.2 Where he is
- [V] Both games, one function compiled twice (`FUN_0023e028` B1:156148, `FUN_002f74c0` B2:285315), called by the
  turret tick after it has stored the turret node's matrix: with a pilot animation, a table and a parent key,
  `E = lookup(table, S+0x1c4)` and the matrix handed to the soldier is **L x E x O** (B1:156190-156203, MIPS
  0x23e148-0x23e264; B2:285353-285366, MIPS 0x2f760c-0x2f7728). L = S+0x1d0, the `PilotPosition` (0x51ca39a6)
  hardpoint's bind matrix relative to its parent; S+0x1c4 = that PARENT's name hash (B1:156093-156101,
  B2:285261-285269); O = the building's world matrix (B+0x200 / owner+0x120). No animation: O itself. No entry:
  `L x O`. The soldier stores it as his world matrix.
- [D] One node only. Hardpoint under the turret node: yaw, no tilt, same update. Under the aimer node: the
  aimer's look-along matrix as its last update left it. Under any other node: the bind matrix - he does not move
  (BF2's two tripod chaingun classes, none placed).
- [L] Turret-node seats: his position in the turret node's frame is constant to 4 decimals and IS the hardpoint's
  bind offset - BF1 gun turret (0, -0.338, -0.152) on 758 rows, BF2 dish (-0.003, -0.350, -1.880) on 369 - and
  his up axis stays (0, 1, 0) while the pitch runs its whole range. Aimer-node seat (BF2 anti-aircraft gun): he
  tilts with the barrels (r 0.997) and sits at the hardpoint within 5 mm of the aimer node of the update BEFORE
  on 334 of 341 rows (the 7 follow a sampling gap); of the same update on 206.

### 3.3 His pose
- [V] `PilotAnimation` / `Pilot9Pose` name the clip `human_<value>`: BF1 keeps the CRC of that name (S+0x210;
  format string `human_%s` at 0x535f40), BF2 an index into a table of at most 10 names and the nine-pose flag in a
  byte (B2:285218, 285271, 198483). [H] A name that arrives when BF2's table is full stores -1 = hidden; the disc
  authors 11 values in all, whether one level loads more than 10 was not counted.
- [V] The numbers (B1:137688-137730, B2:189522-189566). Not nine-pose: (-1, -1, -1) and the clip plays. Nine-pose:

      pf = clamp(((pitch - pLo) / (pHi - pLo) - 0.5) x 2, -1, 1)          0.5 when pLo >= pHi
      yf = the same of yaw when 0 < yHi - yLo < 4.712389 (270 deg)
           else the turn stick (BF1: raw; BF2: the filtered yaw rate over the turn speed)
      a = yf + 4      b = yf + (pf > 0 ? 7 : 1)      c = |pf|

  with the class's own `PitchLimits` / `YawLimits` (no pitch limits: pf = 0.5, so frames yf + 4 and yf + 7 half
  and half). BF2 makes no call at all while the seat's pilot-type word is 3 = `remote` (B2:189522; the word is
  copied from `PilotType` at B2:283202 and set to 3 on every seat of an entity when someone takes its remote
  terminal, `FUN_002334d0` B2:148386-148391): a remote gunner's body stays where it is.
- [V] The pose (BF1 `FUN_0022aa10` B1:142393-142426, BF2 `FUN_0027caf0` B2:200931-200943): sample the clip at
  FRAME a, sample it at frame b, blend by c. Frames 3-4-5 are yaw -1 / 0 / +1 at the middle of the pitch range;
  frames 6-7-8 the same row at pf = +1 (the turret's pitch at its upper limit = nose DOWN); frames 0-1-2 at pf = -1.
- [D] A gun turret, the dish and the towers have `YawLimits -180 180`: their gunner turns with the node and only
  LEANS with the stick. BF1's barge turret (-70 70) is the one seat posed by its yaw angle.

## 4. The views [O]

- [V] First or third person is a per-player record resolved through the seat: `ForceMode` 1 / 2 forces third /
  first, else the seat's "takes the player's preference" virtual, else third (`FUN_001b55f0` B1:59033-59076,
  `FUN_001ce678` B2:77705-77765). The MountedTurret's virtual returns 0 in both games (0x5342e4 -> 0x1b64e0,
  0x62003c -> 0x1d05f0: `jr ra; move v0, zero`) and `ForceMode` is in no building TURRET section
  [receipt:ee16ab66]: **a building turret is third person, always**.
- [V] The camera (`FUN_00225988` B1:138618-138755, `FUN_0026fdf8` B2:190874-191022 - **corrected 2026-10-16, card
  VTURZ: BF2's camera virtual is `FUN_00270018` (slot +0x10, 0x620010, the soldier's camera slot); `FUN_0026fdf8`
  is slot +0xc, see 4.1**; B2:190914-190918, 190991,
  191019): unzoomed - or whenever the building's alive bit is clear - the class's camera section (`TrackCenter` /
  `TrackOffset`) on the AIM FRAME (on the building's own matrix for a class with key 0xd6c12c3b, in no building
  section [receipt:a5ad4948]); zoomed, a look-along frame at the eye point (`EyePointOffset` on the aim frame)
  along the aim, its Z row the aim direction negated.
- [V] **Zoomed, the whole building is left out of the gunner's own viewport**: the armed building's draw asks the
  manned turret's view test first and returns before the table, the pose and the model when it says 0
  (`FUN_001c4878` B1:70155, MIPS `beq v0, zero, 0x1c4aa4`; `FUN_001e81e0` B2:96860), and the test is 0 exactly when
  the turret's zoom flag is set, on a zooming weapon, in that player's own view (`FUN_00225260` B1:138236-138256,
  `FUN_0026f0e0` B2:190156-190174). Other viewports still draw it. The zoom flag needs a weapon with `ZoomMax` >
  1 and the R3 press (B1:137361-137368); a weapon switch and a pilot change clear it. The (0, 0, -1000) write in
  the turret's refresh (B1:138430-138444, B2:190522-190531; BF2 also sends the node's descendants and earlier
  siblings there, `FUN_0026f4f8`) is the path of animated armed buildings and vehicles - and of a building with
  two human gunners, which no ground class can have (one TURRET section each).
- [V][D] The gunner himself is drawn in every view: BF2's own-view skips need a first-person record
  (B2:153230-153279, 153280-153328), BF1's gate passes any soldier no camera sits on (B1:59045), and the other
  return of both gates - his own zoom flag - is cleared at Enter (section 3.1). [L] the dish gunner stays in the
  scene list through the zoom.
- [V] The `FirstPerson` key (a cockpit model, `<FPM lvl>;<model>`) is selected only when the record says first
  person (B1:137370-137373, 138259-138308): never in a building turret. No `ThirdPerson` key exists: hash
  0x0cb308e3 has no hit in either corpus or ELF [receipt:45c076e5] and is on neither disc [receipt:7710e328].
- [L] Frames: BF1 Naboo gun turret third person `ps2/nab1_gty_5_left.png`, `_6_down.png` (the gunner's helmet and
  the shield plates at the bottom, fixed, while the view turns), zoomed `_7_zoom.png`, `_8_zoomright.png` (scope
  frame, no turret); BF2 Mygeeto anti-aircraft gun `ps2/myg1_lg_1_rest.png`, `_4_up.png` (two barrels and the flank
  plates fixed on screen through 30 degrees of pitch), zoomed `ps2/myg1_zoom_2_zoom.png`.

### 4.1 The zoom's camera, its stick zoom and its FOV (card VTURZ, 2026-10-16) [O]

The turret's camera interface (the controllable's, at B1 T+0x2c8, B2 T+0x28) has the soldier's slots (re-zoom.md 5):
- [V] Vtables (`re_check.py word`): B1 0x5342b0 = 0xfffffd38 (-0x2c8); +0xc 0x5342bc = 0x00226780 -> camera
  `FUN_00225988`, +0x10 0x5342c0 = 0x002267a0 -> zoom `FUN_00225c90`, +0x14 0x5342c4 = 0x002267c0 -> FOV
  `FUN_00225ba0` (each thunk `addiu a0, a0, -0x2c8`). B2 0x620000 = 0xffffffd8 (-0x28), 0x620004 = 0; +0x10 0x620010
  = 0x00271148 -> camera `FUN_00270018` (branches on the zoom byte T+0x180), +0x14 0x620014 = 0x00271168 -> zoom
  `FUN_002704f8`, +0x18 0x620018 = 0x00271188 -> FOV `FUN_002703d0` - the soldier's B2 0x61e948 +0x10/+0x14/+0x18.
- [V] **Zoom**: B1 `FUN_00225c90` (B1:138811-138823) returns the weapon in hand's zoom value (+0xd8) when the zoom
  flag T+0x30 is set (and a weapon is in hand), else 1.0; B2 `FUN_002704f8` (B2:191320-191332) the weapon's +0xc4
  when the byte T+0x180 is set, else 1.0. No alive test (the soldier's has one).
- [V] **FOV**: B1 `FUN_00225ba0` (B1:138758-138806), B2 `FUN_002703d0` (B2:191253-191316): flag set -> the turret
  class's +0x1bc (`FirstPersonFOV`); flag clear -> +0x1c0 (`ThirdPersonFOV`), or +0x1bc when the pilot's view record
  says first person (never, for a building turret: section 4). The Controllable constructor's default is 48 deg
  (re-zoom.md 5). [L] Over every class file of the port's bake no armed-building class authors `FirstPersonFOV`
  (hash 0x4785d6b7); BF2's `tur_bldg_*` author `ThirdPersonFOV` 65 (a scan of `assets_baked/**/*.ron` by hash,
  VTURZ's report). So the zoomed vertical tan is tan(24 deg) x 0.7666 / zoom: `tur_bldg_recoilless_lg` (ZoomMin 3)
  12.98 deg, and [L] the PS2's own `myg1_zoom_2_zoom.png` fits 13.0-13.5 deg by edge correlation of port renders
  from its RAM camera (`lead_tmp/vturz/fit_fov.py`; the soldier's 70 deg would give 20.3).
- [V] **The toggle**: B1 `FUN_00223850` B1:137356-137368 (the building alive, a driver, the weapon in hand with
  ZoomMax +0x7c > 1, `(char)T+0x64 == 1`, `DAT_005e9a74`) sets T+0x30 = `FUN_00283c80(weapon, T+0x30)` - the
  soldier's toggle (on at ZoomMin; to ZoomMax only when ZoomRate is 0; else off); B2 `FUN_0026cea0` B2:189160-189178
  the same with `(T+0x64 & 3) == 3`, `DAT_006d85ad` and the byte T+0x180. A B2 weapon switch clears the byte
  (B2:189149).
- [V] **BF2's stick zoom**: in the same block a weapon class with ZoomRate (+0xb8) != 0 runs the continuous zoom
  `FUN_003258f8(dt, T+0x7c)` (B2:189181-189182; ELF 0x26d8ec `lwc1 $f13, 0x6c($s1)`, $s1 = T+0x10) every update,
  with no press gate: zoom += axis x zoom x ZoomRate x dt inside [ZoomMin, ZoomMax] (B2:317686-317714). T+0x10 is
  the turret's controllable (the stick getters `FUN_002f5a48` / `FUN_002f5a18` take *(decompiler detail omitted)*) and its +0x6c is
  the move stick's forward axis, written by the player controller `FUN_002b0548` (B2:234853-234863; clamped to 1, a
  small value raised to 0.3, the pair scaled into the unit circle) - re-hover's throttle slot. [D] Zoomed, the left
  stick zooms a BF2 turret in and out; the toggle resets the value to ZoomMin when it turns on, so only the zoomed
  updates matter.
- [V] **BF1 has no stick zoom on a turret**: `FUN_00283c18` is called from `FUN_001c9398`, `FUN_001cf8d0`, the
  soldier's `FUN_00204578` and the bitstream `FUN_0032db40` only [receipt:dbbaa9f9]; the ELF holds exactly 4 `jal`
  words 0x0c0a0f06 (0x1c98a4, 0x1cfa3c, 0x204718, 0x32deac), no `j` 0x080a0f06 and no pointer to it
  (`lead_tmp/vturz/vtab.py`).
- [L] **The zoomed eye** is `EyePointOffset` on the AIM frame (the look's yaw and pitch) from the turret node: the
  port's rule (`GunSetup::scope_eye`) stands within 0.1 mm and 0.000 deg of the PS2's camera on 8 zoomed RAM rows of
  `nab1_gty` and `myg1_zoom` (`game/tests/turret_views.rs`); turned by the yaw alone it is up to 2.6 m off. The first
  zoomed update's camera is still the third person's (`myg1_zoom` row 30).
- Not read: what draws the scope frame (still); whether the zoom divides the turret's turn rate.
- Verifier (one opus agent, claims and citations only, 2026-10-16): C1-C5, C7, C8 SUPPORTED (C1 and C3 with the
  outer gates added above), C6 PARTLY - the B2 camera slot, corrected above and in the bullet before.

## 5. A dead turret [O]

- The build state (0 built, 1 destroyed, 2 unbuilt), the swap to `DestroyedGeometryName` and its collision, the
  regeneration and the respawn are `re-build-zones.md` 1 [O].
- [V] The armed building's death switches each turret OFF before the base death (BF1 `FUN_001c4ad0`
  B1:70163-70193, BF2 `FUN_001e8498` B2:96868-96925): the Controllable is released and the turret's tasks are
  removed - tick, second task, every aimer (`FUN_00225520` B1:138379-138404, `FUN_0026f470` B2:190327). No matrix
  and no table is written: the angles and both matrices freeze.
- [V] The draw does not look at the state: 0 reads of the state word in `FUN_001c4878` (B+0x2a4) and
  `FUN_001e81e0` (E+0x274), C and MIPS. It fills the table from the CURRENT model and refreshes every turret; a
  node the model does not have gets nothing.
- [L] Of the 49 armed-building classes with a `DestroyedGeometryName` (BF1 4, BF2 45, naming 4 and 21 models) not one
  destroyed model's skeleton holds a turret, aimer or barrel node of its class (script over `skeldump.py`). So a
  dead turret is its destroyed model drawn rigid at the instance.
- [L] `ps2/myg1_destroyed_a.png`: BF2 Mygeeto's `newobjturret1`, state 1, health 0 of 4000, no pilot, yaw 0.379
  frozen: a low black wreck where the gun stood. `ps2/hot1_bots_lasermortar_turet4_a0.png`: a Hoth laser tower
  fallen in a battle - an open broken shell under an empty health bar. `antiaircraft_chunk1` is one name for two
  different models on the two discs.

## 6. For the port

What the port has: a building turret is a `VehicleKind::Turret` with a seat and a `GunState { look, aim }` (model
frame, yaw +z toward +x, pitch + = UP - the opposite sign to the PS2's turret pitch) - `core/src/vehicle/gun.rs`,
`game/src/vehicle_pads.rs`; the world draws the building's model static, flattened to one mesh at the bind pose
(TBIND); a class's `hardpoints` are node names with bind positions only; a rider is never drawn
(`game/src/soldier_vis.rs`); the soldier bake holds no seat clip.

1. **The turret turns (card VTURD, build).** Bake: every armed-building model as rigid parts, one per (segment,
   bone) - a soft segment split by its vertices' bone, exact by the census - and the rig per class: each node's
   parent and bind transform, the turret node with its two factors, the aimer nodes, the barrels with their recoil.
   Draw: per turret instance the node matrices of section 2 from `GunState` - turret node from `look`, aimer node
   from `aim` - and each part's vertices (baked in model space at the bind pose) through the inverse of its bone's
   bind matrix, then the node's matrix, then the instance, the way `SceneGpu::pose_doors` moves door parts. At rest
   the pose is the bind pose: no golden should move.
2. **The gunner (new card VTURG).** Bake the seat clips; a seat-pose state in the animator (clip, or frame a
   blended to frame b by c); draw a unit mounted on a turret seat whose class has a pose key at `L x node x O`
   without his hand weapon; BF2's `IsPilotExposed = 0` hides (vehicles only today).
3. **The dead turret and the zoom (new card VTURX, value 3).** `DestroyedGeometryName` drawn rigid in state 1
   (needs TBLD's state), the building left out of the gunner's own zoomed view.
4. Not in these cards, noted for the planner: the PS2's turret turn law (section 2.2: stick x `MaxTurnSpeed`
   through `TurnFilter`, the limits, the aimer chasing the aim point) against the port's PROVISIONAL seat look; and
   the same node posing applies to every vehicle's turret sections (the port draws vehicles rigid).

## 7. Still [H] / not read
- BF1's aimer-node seat (`bes_bldg_aaa_turret`) and the one-update lag there: read, not seen live. A seat hanging
  from an unposed node (BF2's tripod chainguns): read, not seen live, none placed.
- Which way the nine-pose rows look is derived from the numbers. Card VTURG (2026-10-15, built bc822ba) viewed the
  baked clips and the PS2's own seat-camera frames: [L] `human_minigun_9pose`'s rows differ strongly (frames 6-8 are
  up to 1.67 rad off frame 4 at the hands, frames 0-2 0.95 at the head) and with frame 1 at nose-up the port's
  gunner sits low behind the plates as the PS2's does in `nab1_gty_3_up` (the rows swapped stand him up tall:
  `lead_tmp/vturg/pairs/nab1_rows_flip_test.png`); at nose-down a ~4 degree residual stays (the frame's FOV was
  fitted by eye). `human_dishcannon_9pose`'s three rows are ONE pose on the disc (frames 0-2 = 3-5 = 6-7 within
  0.002 rad, frame 8 an outlier 1.3 rad off at the right upper arm): the dish gunner leans with the yaw number only.
- [V][L] The full-circle yaw number's sign (3.3): BF1 takes the raw stick T+0x78 (B1:137715-137719), BF2 the slewed
  yaw rate *(decompiler detail omitted)* over the speed (B2:189542-189546); on every capture row with the stick past 0.5 the yaw
  turns AGAINST the stick (BF1 `nab1_gty` 147 rows, BF2 `hot1_dish` 139 and `myg1_lg` 91; 0 rows with it), so for
  one turn the two games' numbers have opposite signs and their gunners lean opposite ways. The port keeps each
  game's (`game::soldier_vis`, BF1 = -rate / speed).
- What draws the zoom's scope frame (it is the same frame on a BF1 gun turret and a BF2 anti-aircraft gun, and it
  is not the class's `FirstPerson` model).
- The third-person camera section's own formula on the aim frame (`FUN_001b58f0` / `FUN_001ce9f0`); the port's
  seat camera stays PROVISIONAL.
- BF2's low-detail pass of the soldier render draws a weapon by another test (B2:155375-155378): whether a far
  gunner can show a rifle there is not settled. BF2's swap of section 0's aimer direction at draw time. BF2's
  pilot-animation table of 10 names.
- The three vehicle draws that call the turret refresh; the AI's change to an aimer's wanted direction
  (`FUN_00236700` / `FUN_002a83e0`, card VAIM's ground); the task at BF1 T+0x3dc.

## 8. Tools and data (`engine/artifacts/lead_tmp/vturd/`)
`turcensus2.py`, `nodecensus.py`, `skeldump.py`, `softcensus.py`, `clipcensus.py`, `glbnodes.py` (a baked GLB's
nodes, skins and triangles spanning joints); `ps2/turret_b1x.py`, `ps2/turret_b2x.py` (a human in a turret: one row
per game update of the turret, the aimer, the gunner and the camera, a picture per phase), `ps2/watch_b2.py`
(bot-manned turrets from outside), `ps2/look_b2.py` (the player beside a world point); `replay_nodes.py`,
`replay_aimer.py`, `checks/lag_test.py` (the formulas on the rows); the claims and the verifiers' answers in
`claims/`, `verify/`; the five reader reports `AGENT_A.md` .. `AGENT_E.md`.

## 9. Checks run and the verifier

**Self-checks** (outputs in `lead_tmp/vturd/checks/`): `re_check.py fn` over every cited line of the nine claim
batches - 344 lines, each in the function its claim names (`fn_b1.txt`, `fn_b2.txt`); `re_check.py word` on the 20
vtable words the chains rest on (`words.txt`: 0x532c4c = 0x210e48, 0x5342e0 = 0x226820, 0x5342e4 = 0x1b64e0,
0x539264 = 0x26d748, 0x52efa0 = 0x1c14b8, 0x62005c = 0x271248, 0x620038 = 0x271208, 0x62003c = 0x1d05f0,
0x625f94 = 0x3021f8, 0x61eb08 = 0x5b0eb0, ...) and the two stubs disassembled (`jr ra; daddu v0, zero, zero`);
`re_check.py absent` for every absence above - the receipts are beside the claims, each with a control that the
same search finds a key that is there (`[receipt:70b989b9]`, `[receipt:4e5c2578]`); the unnamed keys seen in
building TURRET sections were traced: 0xd0259d88 is read by two other classes' setters (B2:128039, 299828), seven
have no reader in either corpus [receipt:cd7a7025]; flags: the defaults and writers of `Virtual`, the zoom flag,
the pilot-type word and the two factors were read by the verifiers themselves. Twins: every BF2 function was
opened itself (B2 batches) and read by two verifiers.

**Workflow `vturd-re-verify`**: 17 agents (effort medium), claims and citations only; 13 first verifiers over 101
claims in nine batches, 4 second verifiers over the 45 twin and absence claims. Verdicts: first reading 84
SUPPORTED / 17 PARTLY / 0 NOT; second reading 39 / 6 / 0. Of the 45 read twice: 38 supported by both, 4 qualified
by both (the BF2 aim-frame override, the aimer clamp's exceptions, what the hide walk reaches, the pilot-type
word's second writer), 3 split (one scope point, one "every section", one vtable word cited for the sibling
class) - none needed a downgrade to [H], all 19 qualified claims are stated above in their corrected form. Nine
peripheral claims of the readers were left out of the workflow and are not relied on here. What the verifiers
found that the session had not: the remote terminal that sets the pilot-type word at run time; that the hide walk
also takes the node's siblings and that the building's early exit covers one human gunner only; the BF2
aim-direction override key; that the soldier's own-view gate has a zoom return of its own (the Enter clears were
then read); that the tower pictures could not prove "not drawn" (the scene byte was then read in RAM) and that
one "manned tower" frame was a destroyed tower; that the rows cannot test the aimer node's up vector.

**Readers**: five (three of the dead session, two of this one), reports `AGENT_A.md` .. `AGENT_E.md`; their
"not determined" lists are section 7.

## 10. The classes (from `turcensus2.py`, `nodecensus.py`; "placed" counts instances in every LVL)

**BF1** (9 classes)

| class | seat | pose key | gunner | turret node (PitchTurnFactor) | aimer node | barrels (recoil) | gunner hangs from | destroyed model | placed |
|---|---|---|---|---|---|---|---|---|---|
| bes_bldg_aaa_turret | soldier | PilotAnimation drive | DRAWN | `aimer_y` (1.0, the default) | `aimer_x` | barrel1|barrel2 (0.5|0.5) | `aimer_x` = the AIMER node | antiaircraft_chunk1 | 8 |
| com_weap_gunturret | soldier | Pilot9Pose minigun_9pose | DRAWN | `main_cylinder` (0.0) | `turret_main` | - | `main_cylinder` = the turret node | - | 193 |
| geo_bldg_geoturret | soldier | Pilot9Pose minigun_9pose | DRAWN | `aimer_y` (0.0) | `aimer_x` | barrel (-) | `aimer_y` = the turret node | geo_bldg_geoturret_dest | 10 |
| hoth_bldg_hoth_turret | soldier | - | HIDDEN | `turret_y` (0.0) | `turret_x` | lgun (-) | - | hoth_bldg_hoth_turret_destroyed | 12 |
| hoth_bldg_dish_turret | soldier | Pilot9Pose dishcannon_9pose | DRAWN | `turret_y` (0.0) | `turret_x` | barrel (0.18) | `turret_y` = the turret node | hoth_bldg_dish_turret_destroyed | 12 |
| nab_bldg_fambaa_generator | self | - | none | `hp_fire` (1.0, the default) | - | - | - | - | 0 |
| nab_bldg_fambaa_shield | self | - | none | `hp_fire` (1.0, the default) | - | - | - | - | 0 |
| tat_bldg_barge_turret | soldier | Pilot9Pose minigun_9pose | DRAWN | `aimer_01` (0.0) | `turret_01` | - | `aimer_01` = the turret node | - | 3 |
| yav_bldg_tower_turret | soldier | Pilot9Pose minigun_9pose | DRAWN | `cube_y` (0.0) | `gun` | barrel (0.0) | `cube_y` = the turret node | - | 7 |

**BF2** (69 classes)

| class | seat | pose key | gunner | turret node (PitchTurnFactor) | aimer node | barrels (recoil) | gunner hangs from | destroyed model | placed |
|---|---|---|---|---|---|---|---|---|---|
| tur_bldg_chaingun_roof | self | - | none | `aimer_y` (0.0) | `aimer_x` | - | - | com_bldg_chaingun_roof_dest | 83 |
| tur_bldg_chaingun_tripod | self | - | none | `aimer_gun` (0.0) | - | - | - | com_bldg_chaingun_tripod_dest | 13 |
| tur_bldg_laser | soldier | Pilot9Pose minigun_9pose | DRAWN | `main_cylinder` (0.0) | `turret_main` | - | `main_cylinder` = the turret node | - | 45 |
| tur_bldg_recoilless_lg | soldier | PilotAnimation drive | DRAWN | `aimer_y` (0.0) | `aimer_x` | barrel1|barrel2 (0.5|0.5) | `aimer_x` = the AIMER node | antiaircraft_chunk1 | 9 |
| tur_bldg_tower | soldier | Pilot9Pose minigun_9pose | DRAWN | `cube_y` (0.0) | `gun` | barrel (0.0) | `cube_y` = the turret node | yav_bldg_watchtower_hulk | 7 |
| tur_bldg_beam | soldier | Pilot9Pose minigun_9pose | DRAWN | `main_cylinder` (0.0) | `turret_main` | - | `main_cylinder` = the turret node | - | 8 |
| tur_bldg_chaingun | soldier | Pilot9Pose minigun_9pose | DRAWN | `aimer_gun` (0.0) | `aimer_gun` | - | `tripod` = an UNPOSED node | - | 0 |
| tur_bldg_guided_rocket | soldier | Pilot9Pose minigun_9pose | DRAWN | `main_cylinder` (0.0) | `turret_main` | - | `main_cylinder` = the turret node | - | 0 |
| tur_bldg_mortar | soldier | Pilot9Pose minigun_9pose | DRAWN | `main_cylinder` (0.0) | `turret_main` | - | `main_cylinder` = the turret node | - | 2 |
| tur_bldg_recoilless_md | soldier | PilotAnimation drive | DRAWN | `aimer_y` (0.0) | `aimer_x` | barrel1|barrel2 (0.5|0.5) | `aimer_x` = the AIMER node | fel1_weap_prop_turret_chunk | 0 |
| tur_bldg_recoilless_myg_auto | self | PilotAnimation drive | none | `aimer_y` (0.0) | `aimer_x` | barrel1|barrel2 (0.5|0.5) | `aimer_x` = the AIMER node | antiaircraft_chunk1 | 4 |
| tur_bldg_rep_cannon | soldier | - | HIDDEN | `aimer_head` (0.0) | `aimer_barrels` | barrel1|barrel2 (6.0|6.0) | - | rep_bldg_bandturret_chunk1 | 0 |
| tur_bldg_cis_missile | soldier | - | HIDDEN | `aimer_head` (0.0) | `aimer_barrels` | barrel1|barrel2 (3.0|3.0) | - | - | 0 |
| tur_bldg_cis_chaingun | soldier | - | HIDDEN | `aimer_head` (0.0) | `aimer_barrels` | barrel1|barrel2 (3.0|3.0) | - | - | 0 |
| tur_bldg_imp_beam | soldier | - | HIDDEN | `aimer_head` (0.0) | `aimer_barrel` | barrel (0.5) | - | all_bldg_blisterturret_chunk1 | 0 |
| tur_bldg_all_recoilless | soldier | - | HIDDEN | `aimer_head` (0.0) | `aimer_barrels` | barrel1|barrel2 (3.0|3.0) | - | - | 0 |
| tur_bldg_built_beam | soldier | Pilot9Pose minigun_9pose | DRAWN | `main_cylinder` (0.0) | `turret_main` | - | `main_cylinder` = the turret node | - | 0 |
| tur_bldg_built_chaingun | soldier | Pilot9Pose minigun_9pose | DRAWN | `aimer_gun` (0.0) | `aimer_gun` | - | `tripod` = an UNPOSED node | - | 0 |
| tur_weap_built_gunturret | soldier | Pilot9Pose minigun_9pose | DRAWN | `main_cylinder` (0.0) | `turret_main` | - | `main_cylinder` = the turret node | - | 27 |
| tur_bldg_spa_guided_rocket | soldier | - | HIDDEN | `aimer_head` (0.0) | `aimer_barrels` | barrel1|barrel2 (6.0|6.0) | - | rep_bldg_bandturret_chunk1 | 0 |
| tur_bldg_tat_barge | soldier | Pilot9Pose minigun_9pose | DRAWN | `aimer_01` (0.0) | `turret_01` | - | `aimer_01` = the turret node | - | 3 |
| tur_bldg_hoth_dishturret | soldier | Pilot9Pose dishcannon_9pose | DRAWN | `turret_y` (0.0) | `turret_x` | barrel (0.18) | `turret_y` = the turret node | hoth_bldg_dish_turret_destroyed | 16 |
| tur_bldg_hoth_lasermortar | soldier | - | HIDDEN | `turret_y` (0.0) | `turret_x` | lgun (-) | - | hoth_bldg_hoth_turret_destroyed | 14 |
| tur_bldg_geoturret | soldier | Pilot9Pose minigun_9pose | DRAWN | `aimer_y` (0.0) | `aimer_x` | barrel (-) | `aimer_y` = the turret node | geo_bldg_geoturret_dest | 3 |
| tur_bldg_recoilless_kas | soldier | PilotAnimation drive | DRAWN | `aimer_y` (0.0) | `aimer_x` | barrel1|barrel2 (0.5|0.5) | `aimer_x` = the AIMER node | antiaircraft_chunk1 | 5 |
| uta1_bldg_turret | self | - | none | `aimer_y` (1.0, the default) | `aimer_x` | barrel|barrel1 (0.5|0.5) | - | uta1_bldg_turret_destroyed | 2 |

The other 43 BF2 classes are `PilotType = self` or `remote` seats with no soldier (space-map hull turrets and the
dropped auto / defence-grid turrets; 27 self and 20 remote over all 47), every one with PitchTurnFactor 0.0.
