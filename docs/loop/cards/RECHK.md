---
id: RECHK
title: re_check absent misses Ghidra's 64-bit sign-extended hash form (0xffffffffXXXXXXXX) and code-built constants: GeometryLowRes read '0 hits' for three weeks (re-softskin 7, corrected by card LOD)
milestone: M0
value: 3
phase: build
status: todo
deps:
files: scripts/re_check.py
declares:
needs:
effort:
branch:
resume:
created: 2026-10-16
---
## Why
Card LOD (2026-10-16) found BF1 FUN_0020f2e0 123451 / BF2 FUN_00252bb0 171132 comparing *(decompiler detail omitted)* (GeometryLowRes): `re_check absent b1 0xa490eba1` returns 0 hits (the term with its 0x prefix is not a substring of the 64-bit form), and `--elf` scans only whole LE u32 data words, so a hash built by lui/ori in code is invisible too (the GeometryName control 0x47c86b4a: 53 decomp hits, 0 raw ELF hits). re-softskin 7 called the property dead on that evidence.

## Evidence
`engine/artifacts/lead_tmp/re/lod/checks.txt`; `lead_tmp/re/lod/tools/luiscan.py` (lui + ori/addiu pair scan: GeometryLowRes 1 build per ELF, GeometryName 19 / 25); analysis/re-lod.md 5.

## Acceptance
`re_check.py absent b1 0xa490eba1 --elf` reports FUN_0020f2e0 line 123451 AND the BF1 code build at 0x20f788 (B2: 171132, 0x253438); a term >= 0x80000000 is searched in the hex, 0xffffffff-extended, negative-hex and both decimal forms; the receipt lists each form; a scripts selftest case for each with one present-key control (GeometryName) and one absent key.