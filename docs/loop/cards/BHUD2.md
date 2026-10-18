---
id: BHUD2
title: The BF2 HUD, part 2 — draw it: a HUD interpreter over BHUD's baked tree (Group, Bitmap, BarBitmap, BarSegmented, Text, Model3D; fader, colour pulse, z-order, Shadow) with the PS2's own screen mapping and Hor+ anchors, fed from the sim — health, stamina, weapons + ammo (BF1 IconTexture fallback), reinforcements, the reticle; retires the egui placeholder
milestone: M3
value: 5
phase: build
status: done
deps: BHUD
files: engine/crates/ui/src/hud.rs, engine/crates/render/src/ (a HUD pass: textured quads + unlit meshes, no depth), engine/crates/game/src/main/battle.rs (hud_model feed), engine/crates/game/src/hud_feed.rs (new, if the feed outgrows battle.rs), engine/CRATES.md
declares: goldens=tat1_hud.png,tat1_spawn.png; rebake=common
needs:
effort:
branch: bhud2
resume: landed ea3883c6 2026-10-02
created: 2026-09-30
alpha: yes
domain: render
---
## Why
analysis/re-hud.md §2–§4 (card BHUD's RE): the PS2 draws every HUD element from the hud_ tree under one camera; the port's
HUD (engine/goldens/tat1_hud.png) is an egui placeholder in neither BF2's layout nor its art.

## Do
- Interpreter: evaluate 1playerhud (+ hudtransforms) per frame: Position/Rotation/Scale nest by group; Viewport fractions →
  the PS2 mapping x = (−0.0059 + 1.0139·fx)·(4/3)H, y = (−0.0242 + 1.0484·fy)·H; element-local px → H/378.7 × H/427.3;
  Model3D unit → 0.2129 H × 0.1887 H (orthographic), MeshInfo overrides; z 0 in front; fader (in/hold/sustain/out, hold
  default forever); colour eases to Color at ColorChangeRate and snaps to ColorChange every ColorPulseRate; alpha =
  fader·Alpha; BitmapStyle Shadow = a (+1, +1) native-px black copy at half alpha; BarBitmap clips rect and u by value.
- Hor+: anchor each top-level element left / centre / right by its fx (< 1/3, middle, > 2/3), keeping its 4:3 offset
  (COMPARISON_RULES); record the anchor rule and the kept 1.128 mesh/glyph stretch in FEEL_NOTES.md.
- Feed (the 1playerhud events): health/max (+ healthColor pulse when regenerating), stamina, weapon1/2 = class name → mesh
  via hudtransforms, else the weapon's own model, else (a BF1 weapon with no BF2 art) its BF1 IconTexture sheet as a
  Bitmap (PLAN §Presentation); ammo = clip + spare as ONE number (re-hud §3), clip fraction ring, infinite icon;
  reinforcements team1 = the player's team (blue), team2 = the other (red), pulse period 1/bleed; the reticle group at the
  crosshair coloured by AIMF's reticle_pick (card RETC: fold its draw in here).
- Keep the pad warning and the round banner (egui) until BF2's own elements replace them.

## Acceptance (measurable)
- A 640×480 4:3 port render with HOT1 postA v1's values (health full, 250 + 4 ammo, counts 147 / 145) against the PS2 frame
  engine/artifacts/pcsx2/ref/bf2_hot1/cp1_cp6_postA_v1.png: health fill x 38–170 and rows 405–414 (PS2 39–169 × 406–413),
  count centres within ±2 px of 172.5 / 466.5, reticle outline 76 × 70 within ±3 px (re-hud §7).
- tat1_hud.png moves (declared; viewed); a 16:9 render shows the left/right groups on the screen edges, nothing stretched.
- No sim, sweep or arena move.
## Note from BHUD's build (2026-09-30)
BHUD baked the layouts (`common_bf2/hud/<layout>.ron`, index `hud/index.ron`), the fonts (`common_bf2/fonts/`) and ten
COMMON/CORE sheets into `common_bf2/textures/` (schema `manifest::hud`). It did NOT bake the four side-LVL HUD meshes
(`hud_cis_trishot` CIS, `hud_wampa` SNW, `hud_stealth` ALL, `hud_imp_flamethrower` IMP): they belong with the side
weapons — add them to rebake.sh's side group (`bake weapon <SIDE>.LVL hud_… side_bf2`) when this card draws weapon icons
(declare `rebake=side`). Records keep their names case-blind (`HudNode.name`; unnamed = `#` + 8 hex digits).

- Planner 2026-09-30 17:30: Two new consumers: SQMK's one-line order text ("Follow me — 3 units", drawn in the placeholder box now) moves into the PS2 HUD message style here; EVMSG (M3 5) replaces the placeholder kill/capture feed with BHUD2 elements once its RE is done (its build deps BHUD2).
- EVMSG RE 2026-09-30 (`analysis/re-event-messages.md`): the interpreter needs a MultiLineText (queue / admit / expire by DisplayTime / fade / scroll, EventText + EventColor; RE §2 has the state machine and the PS2 timings) — used by `player1systemMessages` and `player1tooltipsTEXT`; build it here or leave it to EVMSG (whichever lands first). The `player1stats` group (points + kills pop-up) is fed by EVMSG.

- Planner 2026-09-30 21:45: value 4 -> 5. It gates EVMSG (5), BMAP (4) and the restyle of SQMK's order text, and the placeholder HUD is on the alpha checklist (PLAN: "the real BF2 HUD").
