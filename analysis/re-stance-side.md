# LEAD VERIFICATION (Fable lead, 2026-09-09, session 17)
**A — VERIFIED**, and `analysis/re-stance-roll.md` §3.2 is corrected in place: block A (`FUN_0020aad0`) is the AI/agent controller path, block B (`FUN_00205cc8`'s bytes) is the local pad path.
**B — VERIFIED AS CODE, RETRACTED AS A LIVE PATH.** Lead re-read: the order-kind → state ladder at 161150–161158 (kind 2 → state 5 in the window; the agent's kind 3 → state 6 follows the same shape). **The reachability claim — that no reachable site ever issues an order of kind 3 — is the AGENT's**, over `a4-bf1-reachable.txt`, and is not re-derived here.
**C — VERIFIED AS CODED BUT DEAD, lead re-read:** the raw ELF words at 0x2deec0 are `03e00008 0000102d` = `jr ra; daddu v0,zero,zero`, and the decomp body is `return 0;` — so `FUN_002d53c8`'s hint lookup can never succeed and its stance states never run.
**PORT IMPACT: none.** The `PrimaryStance` mask rule (RE_LOG 2026-09-08 evening) stands unchanged; HOLD 17's three PROVISIONAL side findings are CLOSED. Everything below this header is the RE agent's report, verbatim.

# RE: the three PROVISIONAL stance side findings — verify or retract (Fable RE agent, session 17, 2026-09-09)

Lines are `analysis/SWBFRONT.ELF-decomp.c` (BF1) / `analysis/SWBFRNT2.ELF-decomp.c` (BF2); ELF reads from `extracted/bf{1,2}/*.ELF`
(PT_LOAD parsed). X = the soldier's controllable sub-object at `soldier+0x2a0` (vptr X+0x294 = `soldier+0x534`), so `soldier+0x3c8` = X+0x128,
`+0x2ee..+0x2f1` = X+0x4e..+0x51. Class names come from the ASCII string the linker emits right before each vtable (BF1 "UnitAgent" 0x536618 →
vt 0x536628, "UnitCombatAgent" 0x536830 → vt 0x536840, "PlayerController" 0x5384f0 → vt 0x538500; BF2 "UnitTrooperAgent" 0x6235e8 → vt 0x623618 —
all 11 BF2 order-factory ctors land inside their own name's bracket of the alphabetically sorted table, so the convention holds) [V].

## A. `re-stance-roll.md` §3.2 blocks are labelled the wrong way round — VERIFIED (doc is swapped; s15 correction stands)
- Block A = the int commands X+0x128/+0x12c/+0x130/+0x140 (`soldier+0x3c8` stance, `+0x3cc` roll L, `+0x3d0` roll R, `+0x3e0` jump), consumed by
  `FUN_0020aad0` 119721–119877 (controllable thunk 124057, runs for every soldier). Their ONLY writers on X are the AI soldier controller, vtable
  `0x5375e8` (ELF: +0x54 = 0x24ffa8, +0x58 = 0x24fe18, +0x5c = 0x24fe50, +0x44 = 0x24fb48): `FUN_0024ffa8` 169296–169305 → `FUN_0023c2a8`
  154686–154692 `X+0x128 = v`; `FUN_0024fe18` 169201–169209 `X+0x12c = 1`; `FUN_0024fe50` 169215–169223 `X+0x130 = 1`; `FUN_0024fb48` tail
  169240–169250 `X+0x140 = 1` + point at +0x150; ctrl → X via `FUN_0024ef18` 168480–168486. That controller is built inside the UnitAgent ctor
  `FUN_00243cb8` 160329–160364 (`FUN_0024f278` 168664–168677, non-Flyer entities). Resets: `FUN_0023aa88` 153532–153544, 153449–153452 [V].
  Pushers of +0x54 are agents only: 161237/161300/161345 (CombatAgent), 164501, 164609, 165397, 166073, 174562 (Snipe), 175273 [V].
- Block B = the bytes X+0x4e/+0x4f (jump) and X+0x50/+0x51 (stance), consumed by `FUN_00205cc8` 117417–117474 / 117515–117517. Their ONLY
  tick-time writer is `FUN_0025e098` 179449–180926 = `PlayerController` update (vt 0x538500 slot +0x1c; ctor `FUN_0025e030` 179421–179437 binds
  `[9] = FUN_003036e8(player index at entity+0xe8)` = the pad record): pad word `[9]+0x10` bit 2 → 179932–179983, bit 3 → 179985–180038.
  Everything else is ctor init to 0xfe (72186…, 153341, 156444) or a clear (116805–116807, 153430–153433) [V]. PlayerController never writes
  X+0x128..+0x140 — its +0x128/+0x130 stores (180434–180441, 180883–180894) are on weapon objects from `X vt+0x3c(slot)` (aim-assist target) [V].
- Corrected labels: **block A (`FUN_0020aad0`) = AI/agent controller path; block B (`FUN_00205cc8` bytes) = local pad path.** Also correcting s15's
  "tap = crouch, hold = prone": +0x51 = 1 on release after < 6 ticks (tap), 2 on a press within 10 ticks of a tap (double-tap), 3 on hold ≥ 6
  (unused); `FUN_00205cc8` crouch-toggles on (+0x50 == −1, +0x51 == 1) and prones on (+0x50 == +1, +0x51 == 2) — double-tap [V].
  Bonus: the BF1 AI dodge-roll is CombatAgent state 5, 161268–161297 (50 %, threat > 45° to a side → vt+0x5c/+0x58) [V].
- Port impact: **no** — the mask rule is untouched; this only fixes which input layer owns which command block.

## B. BF1 `FUN_002450b8` requests prone directly — VERIFIED as code, RETRACTED as a live path (dead: no order of kind 3 is ever issued)
- `FUN_002450b8` 161183–161397 = `UnitCombatAgent::OnEnterState` (vt 0x536840: +0x8 this, +0xc exit `FUN_00245698`, +0x38 tick `FUN_002456e0`
  161413–161724). State 6 entry 161235–161241: `ctrl vt+0x44()`, **`ctrl vt+0x54(ctrl, 2)`** (→ `FUN_0024ffa8` → `soldier+0x3c8 = 2` → `FUN_0020aad0`
  119872–119875 → `vt+0x84` = `FUN_00208c70` go-prone), timer [0x2d] = now + 4 s; tick 161566–161572: expired → state 3 (two random 30–75° look
  turns, `FUN_00245dd0` 161724–161820, then 100). No hint is read anywhere in the class — it bypasses the mask [V].
- Condition: state 6 is ONLY the initial state picked by the ctor `FUN_00244f80` 161118–161170 from the order kind `FUN_00263b78(order)` 183820
  (= order+0x1c): 0 → state 2 move-to-point, 1 → 4 attack target, 2 → 5 stand/face point, **3 → 6 prone**; the tick sets only 3/4/100 [V].
- Reachability: `FUN_002450b8` (a4-bf1-reachable.txt 3244), the ctor and its sole caller `FUN_00262b60` 182920–182931 (unit method: agent for the
  order at unit+0x20, `FUN_00263140` 183141) are `dataptr` = linker tier only, normal for virtuals. But kind 3 has no producer: the only setter
  `FUN_00263b70` 183812 is reached from `FUN_00263a60` (171001 kind 2, 171132 kind 0, 171181 kind 2, 171450 kind 0), `FUN_002639a8` (171336
  kind 1) and `FUN_00263b10` — which is absent from the reachable set; no `+0x1c = 3` store exists in 150000–186000; base ctors `FUN_0025c750`
  178392 / `FUN_0025c6c8` don't touch +0x1c; `FUN_0025c920` copies the order verbatim [V]. So the prone request is dead code on the PS2.
- Port impact: **no** — BF1 AI prone remains hint-only (SnipeHints with bit 2), exactly as the port rule assumes.

## C. BF2 `FUN_002d53c8` is a third hint behaviour — VERIFIED as coded (named below), but its hint lookup is DEAD; "Fortification type 7?" retracted
- `FUN_002d53c8` 260525–260794 = OnEnterState (+0x8) of an unnamed sub-behaviour class, vt 0x6240e8 (ctor `FUN_002d52f0` 260476–260491, start
  `FUN_002d5348` 260493–260514 with a target handle; +0xc reset `FUN_002d5c20` = raw `sw zero,0x30(a0); jr ra; sb zero,0x1c(a0)`; +0x10 tick
  `FUN_002d5c30` 260796–260937 — same slot layout as Cover 0x623c08 and Snipe 0x624298) [V]. It is embedded at +0x100 of **`UnitTrooperAgent`**
  (vt 0x623618; ctor `FUN_002cb970` 253817–253838 also embeds Snipe `FUN_002d7be0` at +0xb0 and a hint helper vt 0x624050 at +0x170), the agent
  the order factory `FUN_002b70d0` 238964–239110 builds for **order type 9 with a target entity** (239035–239053; siblings: 1 Wait, 2 Combat,
  3 WaitSecondary, 4 Board, 5 UnBoard, 8 Random, 10 CTFOffense, 0xb Deathmatch, 0xc Defend, 0xd Destroy) [V].
- Hint fields: state 1 (260762–260772) `FUN_002df9e8(ctrl, target, 10, list)` 267680 = `FUN_002df858(ctrl, 2 = CoverHint, …)` 267624–267660,
  then `FUN_002d5fb8` 260984–261009 picks one usable hint ≠ [5]; state 3 (260674–260683) claims it `FUN_002ded48` and walks to it, 30–45 s;
  state 5 (260643–260665) `[7] = 1`, look point [8..0xb] = own pos + `FUN_002dedb8(hint)` (forward) × 10, **`[0xc] = FUN_002def18(hint)`** = the
  Primary pick from `(hint+0x50 >> 8) & 3` (s15 267135–267143); state 6 (260638–260641) `[0xc] = 0` [V].
- Push: the agent tick `FUN_002cc0a0` 254232–254256 (agent state 4) ticks it, reads `[0xc]` via `FUN_002d5f88` 260957 and the look flag/point via
  `FUN_002d5f78/80`, then owner +0x48 `FUN_002c20e0` 247434–247437 (with look point) or +0x44 `FUN_002c20a8` 247418–247428 → `vt+0x3c` =
  `FUN_002c1f50` 247341–247357 → `FUN_002ba970(ctrl+0x430, stance)` — the Snipe/Cover funnel (ELF 0x623618 +0x3c/+0x44/+0x48 = 0x2c1f50/0x2c20a8/
  0x2c20e0); agent state-4 exit resets it (`FUN_002cbd48` 254047–254049 → `FUN_002d53a0` 260515) [V].
- Dead: `FUN_002df858`'s filter 267643–267645 keeps a hint only if `FUN_002deec0(hint) != 0 && == target`, and `FUN_002deec0` (decomp 267103,
  raw 0x2deec0 `03e00008 0000102d` = `return 0`) is a stub — so the query always returns 0, [5] = 0, state 1 → 4 never 3, and states 3/5 are
  unreachable; the live loop is 1 → 2/4 → 6 → 1 pushing stance 0 [V]. BF2 has no hint subclasses left (a3-subsystem-map.md 105), which fits.
- Port impact: **no** — a dead third consumer changes nothing; if a port ever revives it, it uses the same `& 3` picker as Snipe/Cover.

## Ready-to-paste RE_LOG.md lines (replace the PROVISIONAL bullet at 2327–2329)
- [V] `re-stance-roll.md` §3.2's labels ARE swapped (s17 `lead_tmp/s17/re/stance_side.md` A): block A (`soldier+0x3c8/+0x3cc/+0x3d0/+0x3e0`, `FUN_0020aad0`
  119860–119877) is written only by the AI soldier controller vt `0x5375e8` (+0x54 `FUN_0024ffa8` → `FUN_0023c2a8` 154691; +0x58/+0x5c roll L/R `FUN_0024fe18/fe50`);
  block B (`+0x2ee..+0x2f1`) only by `PlayerController::update` `FUN_0025e098` 179932–180038 from pad bits 2/3 (tap-release = crouch, double-tap = prone).
- [V] BF1 `FUN_002450b8` (`UnitCombatAgent` OnEnterState, vt 0x536840) does push prone directly — state 6, `ctrl vt+0x54(ctrl,2)` 161237, held 4 s, no hint read — but
  state 6 is only the ctor's initial state for order kind 3 (`FUN_00244f80` 161154–161156) and no reachable site issues kind 3 (kinds 0/1/2 at 171001/171132/171181/
  171336/171450; setter `FUN_00263b10` unreachable): DEAD on the PS2. BF1 AI prone stays hint-only (SnipeHints bit 2); port rule unchanged.
- [V] BF2 `FUN_002d53c8` (vt 0x6240e8, the sub-behaviour at `UnitTrooperAgent`+0x100, order type 9) IS coded as a third hint-stance consumer — state 5 `[0xc] =
  FUN_002def18(hint)` 260664, pushed by the agent tick 254240–254256 via `FUN_002c20e0/20a8` → `FUN_002c1f50` → `FUN_002ba970` — but its CoverHint query
  `FUN_002df858` 267643 needs `FUN_002deec0 != 0`, a `return 0` stub (0x2deec0): states 3/5 unreachable, DEAD; not "Fortification type 7". Port rule unchanged.
