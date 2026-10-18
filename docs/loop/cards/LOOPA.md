---
id: LOOPA
title: gates.py / land.py stop at once with ADAPTER LOST when the GPU the goldens were rendered on is absent (split from LOOPT: the oracle guard refuses a builder's edit to gates.py / land.py without [approvals.LOOPA] oracle = "guards")
milestone: M0
value: 5
phase: build
status: done
deps:
files: scripts/gates.py,scripts/land.py
declares: oracle=guards
needs:
effort:
branch: loopa
resume: landed 2a60fd7b 2026-10-16
created: 2026-10-15
alpha: yes
---

## Why
LOOPT part 2. 2026-10-15 18:26 the NVIDIA GPU fell off the bus; wgpu took the Intel iGPU and SKIPC's and VRIDE's gates moved 89 goldens each (33 declared + 56 the diff cannot reach); each session spent turns proving it was the adapter. LOOPT's build session (2026-10-15 20:00) was refused by the oracle guard on its first edit to scripts/gates.py ("guards" class: gates.py, land.py, oracle_guard.py). Not worked around.

## Needs
`[approvals.LOOPA]` `since = "2026-10-15"` `oracle = "guards"` in engine/artifacts/loop/gates.toml (the planner or the maintainer adds it). Then this is a ~1 hour build.

## Do (the design LOOPT drafted)
- gates.py: `GOLDEN_ADAPTER = "NVIDIA"` (a substring of a `vulkaninfo --summary` deviceName; render/device.rs asks wgpu for HighPerformance), env `OPENBF_GOLDEN_ADAPTER` overrides; `vulkan_devices()` parses the deviceName lines (None = cannot tell = not checked); `adapter_check()` -> (present, line); `--adapter-any` / `OPENBF_ADAPTER_ANY=1` skips it.
- In `run()`, before the group loop: when `gpu` is asked (and not already done) and `golden_plan()` is non-empty, check; absent -> one row `gpu ADAPTER LOST` (exit 7), no group runs, SUMMARY.md first line = the message, `rows_exit()` -> 7 for main() and the state file (1 stays a gate FAIL). A diff that moves no golden is not checked.
- land.py `main()`, before `--detach` and for every run but `--abort`: if `plan_goldens(wt, "main")` is non-empty and the adapter is absent, print one line and exit 7, nothing touched (no MAIN claim, no rebase, the card as it was). `--adapter-any` passed through. `undeclared()`: an ADAPTER LOST row (lost mid-landing) reads as such, not as a move.

## Acceptance
- `OPENBF_GOLDEN_ADAPTER=NoSuchGPU gates.py --tree <a worktree whose diff touches a golden dep> --auto`: SUMMARY.md first line `ADAPTER LOST`, exit 7, no golden written, no other group ran; with the adapter present the gpu step runs as today (same tree, exit 0/1 as before).
- `OPENBF_GOLDEN_ADAPTER=NoSuchGPU land.py <ID>` on a parked card that moves a golden: exit 7 at once, card / MAIN / branch untouched.
- A sim-only diff (no golden step) runs with the adapter hidden.
- gates.py / land.py self-tests still pass.
