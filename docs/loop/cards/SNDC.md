---
id: SNDC
title: Gameplay audio census: which sounds the PS2 plays in a battle (weapons, impacts, explosions, vehicles, foley, voice, music, ambience, UI) against what the port plays — one table per game on a showcase map, a card per gap
milestone: M1
value: 5
phase: measure
status: done
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
The maintainer's INBOX note 2026-09-29 22:40, item 1: M-ALPHA must "sound like a game". Only SNDP (value 1, sample edge cases)
and USND (unit voice) cover audio today. The port plays the primary's fire and reload sounds, the ordnance impact, door
sounds and the faction's squad chatter (USND's reading of `game/src/main/windowed.rs`). Nobody has listed what else the
PS2 plays.

## Do
- **The PS2 side.** On two showcase states (BF2 UTA1 or HOT1, and BF1 NAB1), record PCSX2's audio for a minute of
  battle with a bot fight (`ps2live.py` on BF1). Also list the sound properties the map's classes, weapons and
  ordnance name (the baked `sounds/names.ron`, the ODF keys). Categories: weapons, impacts, explosions, vehicles
  (engine, weapons), soldier foley (footsteps, jump, land), voice, music, ambience (the map's sound regions and
  streams), UI / HUD.
- **The port side.** The same list from `game::sounds` and its callers.

## Acceptance
- A table per game (category → PS2 event → sound property → the port plays it Y/N) in
  `analysis/audio-census.md`, each "Y/N" reproduced (for the port, by a log of played sound names from a scripted
  battle).
- One card per gap worth building (tagged `alpha: yes` if a showcase battle plays it), at most 2 new unless value ≥ 4.
  Otherwise, notes on USND / SNDP / MSND.

- Planner 2026-09-30 17:30: ANNC (M3 5, alpha) owns the commander voice-overs (cis_off_com_report_*, SetOutOfBoundsVoiceOver, REP_bonus_*); drop them from this card's scope if ANNC lands first, and vice versa.

- ANNC RE 2026-09-30 (`analysis/re-announcer.md`): the commander lines are specified (ANNC, build) — drop them from the census's gap list. Useful here: (a) `lead_tmp/annc/annc_live.py watch --game bf1|bf2 --planet <P>` logs every line a live PCSX2 battle plays, by name, per stream slot, with queue counts (BF1 NAB1: slot 0 music, slot 1 `cw_vo` = commander + ALL unit chatter, 3-4 lines always pending; BF2 UTA1: slot 0 voice slow, 1 voice quick, 2 music, chatter sparse) — the PS2 side of the census without recording audio. (b) The same VoiceOverManager owns the ambient MUSIC (`SetAmbientMusic(team, fraction, name, state)`: start / middle / end by ticket fraction 1.0 / 0.99 / 0.1) and the battle end plays VictoryMusic / DefeatMusic for the listener's team (§2.5) — not built, no card: a census row. (c) Unit chatter shares the commander's voice queue (4 pending, key order; chatter template 0xCFFF9324: Gain 0.6, Mode3D, PlayInterval 20 ± 5 s in BF1, 30 ± 5 in BF2).

## Planner note (2026-10-02 09:30)
Value 4 -> 5: "sounds like a game" is in M-ALPHA's definition and no card says what is missing; this census is what
turns it into build cards. M1 had one value-5 card (UWPN). Since this card was written: EVTK (0b99471) made the
windowed game start each event sound once per tick (it was twice at 60 Hz) and moved the event -> sound mapping into
`game::event_sounds` (`each`, `listener`), which is the port side's list; ANNC landed the commander (drop him from the
gaps; ANNC2 has his two wrong inputs). Use `lead_tmp/annc/annc_live.py` for the PS2's voice and music streams.
Card limit: at most 2 new cards unless value >= 4. Reverse: `loopctl.py set SNDC value=4`.

## Result (2026-10-02, lane 3) - DONE
`analysis/audio-census.md`. PS2: two 105 game-second battles sampled live with a new RAM sampler
(`lead_tmp/sndc/snd_live.py`: every sound start by name from the 78 one-shot voices, the 39 handle voices and the 6
streams) plus a pause-menu / respawn sample each; port: a SOUND_LOG of the headless battle (scratch patch). 24 event
rows per game: the port plays 5 (BF1) / 4 of 21 heard (BF2). Second finding: the bake resolves `Inherit` only inside
one SOUND LVL, the templates are in CORE.LVL -> every port sound is gain 1.0 and silent beyond 50 m (disc: weapons
5 - 400 m). Cards: SNDT (M1 5, build), FOLY (M1 5, re), VSND (M2 4, re), WSND (M1 4, re), AMBM (M1 4, re); notes on
USND, GREN, PWUP. Report `reports/20261002_1250_card_SNDC.md`.
