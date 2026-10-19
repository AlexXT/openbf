---
id: AITHK
title: AI think rate by distance and view: far or unseen bots think less often (the PS2's own rule), so a big bot count costs less CPU - deterministic, in the sim
milestone: M0
value: 4
phase: re
status: todo
deps:
files:
declares:
needs:
effort:
branch:
resume: RE FIRST: read what the PS2 does with far or unseen AI soldiers (think interval by distance and by whether the player can see them) in the SWBFRONT / SWBFRNT2 decomps, PCSX2-confirmed; then build it as a SIM rule (the port's sim is deterministic: no render-side shortcut).
created: 2026-10-19
---

## Why
(the maintainer, 2026-10-19: speed on a weak laptop; bot count) Every bot thinks every tick. Engines save most of their AI CPU by letting bots that are far from the player or out of its view think less often. The PS2's own rule must be found (card SAIR / re-soldier-ai.md read the soldier AI; the think cadence by distance may or may not be there), because a made-up cadence changes the fights the gates measure.

## Evidence
`analysis/re-soldier-ai.md`, `analysis/re-tickets-spawn.md`, `analysis/bot-design.md` (the layers and the tick), `core/src/ai/` (the think call); the PC game's think cadence (every 4 / 3 / 2 / 1 / 0.25 s by distance and view cone) is a lead for the PS2 search, NOT the PS2's numbers.

## Acceptance
1. `analysis/re-ai-think-rate.md` [O] (or a section in re-soldier-ai.md): the PS2's rule with decomp lines, or a receipted statement that the PS2 has none.
2. If the PS2 has it: build it in `core` as a deterministic rule; gates: the sim hashes move only as declared, the fidelity gate and the sweep stay within tolerance, and ms per tick at 32 / 64 / 128 bots before and after is in the card.
3. If the PS2 has none: park the card with the finding; a modern-port variant (a presentation-only cadence for unseen far bots) is a proposal to the maintainer in STATUS, not a build.

## Notes

