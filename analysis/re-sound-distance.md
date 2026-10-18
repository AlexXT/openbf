# OPUS RE (2026-10-02) - [O] UNAUDITED

# A 3-D sound's level by distance (card SNDT)

What the PS2 multiplies a sound property's gain by, as a function of the distance to the listener, and which `snd_`
keys feed it. Bounded: one function and its inputs. Tags: [V] read (lines cited), [L] measured (RAM / disc),
[D] derived, [H] hypothesis. Corpora: B1 `analysis/SWBFRONT.ELF-decomp.c`, B2 `analysis/SWBFRNT2.ELF-decomp.c`.
Evidence and tools: `engine/artifacts/lead_tmp/sndt/`.

## 0. Result

```
k = listener.rolloff_factor (10.0) * property.rolloff
d <  rollindistance          : level = d / ((rollindistance - d) * k + rollindistance)
rollindistance <= d <= min   : level = 1
min < d < mutedistance       : level = (R - x) / (x * k + R),   x = d - min,  R = maxdistance - min
d >= mutedistance            : level = 0
```
The same on both games `[O]`. With k large the falling branch is about `R / (k x)`: an inverse-distance law that
reaches 0 at `maxdistance`. The rifle fire template (5 - 400 m, rolloff 6 on BF1): 0.113 at 50 m, 0.027 at 153 m.
The port's old curve was `((max - d) / (max - min)) ^ rolloff` (ours, PROVISIONAL), now replaced
(`game::sounds::attenuation`).

## 1. Claims

1. **[V] The level function**, B1 `FUN_0044cac8` (header 510270), lines 510280-510299: `a local = argument 1[0xb]` (the
   distance struct P), `a local = (float)argument 1[8]` (the distance d), `a local = P+4`, `a local = P+8`. `d < P+4` ->
   `(a local - (a local - a local)) / ((a local - a local) * *(*argument 1 + 0x1c) * *(P + 0x14) + a local)` (510285). Else if
   `P+8 < d`: when `d < P+0xc` -> `a local = P+0x10 - a local`, `(a local - (a local - a local)) / ((a local - a local) *
   *(*argument 1 + 0x1c) * *(P + 0x14) + a local)` (510292), otherwise 0.0 (510296). Else 1.0 (510299).
2. **[V] P's fields by `snd_` key**, B1 `FUN_0044d130` (header 510563): `rolloff` 0xab59d4bb -> `argument 1[5]` (+0x14),
   flag 0x20 (510573-510577); `rollin` 0x1ce8c2e1 -> `[0]`, flag 1; `rollindistance` 0x80a163fe -> `[1]` (+4), flag 2;
   `mutedistance` 0xe4b12edf -> `[3]` (+0xc), flag 8 (510609-510613); `mindistance` 0xb835cf50 -> `[2]` (+8), flag 4;
   `maxdistance` 0xe6915cf6 -> `[4]` (+0x10), flag 0x10. The flags byte is at +0x18. Key 0x0fba43f2 sets / clears
   flag 0x40 (the doppler branch of the function at 510480 reads it). Key names by `fnv1a_lowercase` [L].
3. **[V] P is the property's +0x28**: B1 `FUN_004599e0` (header 518898; the one-shot start) sets `a local = a local + 0x28`
   (518951, `a local` = the property) and `…(argument 1 + 0x2c) = a local` (518959) on the 3-D instance, then calls
   `FUN_0044ca78(argument 1)` and `FUN_0044cac8(argument 1,&a local)` (518961-518962).
4. **[V] d and the listener**: `FUN_0044ca78` (510248) stores `*argument 1 = FUN_0044a128(argument 1[9], argument 1 + 4,
   argument 1 + 8)`. `FUN_0044a128` (header 508271) walks 4 listeners at `0x66eff0`, stride 0x14 words (0x50), bit 0 of
   the first word = active, returns the nearest and writes its distance through its third argument
   (`…argument 3 = a local`, 508350). So `*argument 1 + 0x1c` in claim 1 is a field of the listener.
5. **[V] The listener's +0x1c is 10.0 at construction**: B1 `FUN_00449f78` (header 508173) `a local[7] = 0x41200000`
   (508184), beside `[6] = 1.0` and `[8] = 1.0`. **[L]** RAM: listener +0x18 / +0x1c / +0x20 of every active listener
   in the 329 readable reference states (`lead_tmp/sndt/listener_ram.py`): B1 151 x (1.0, 10.0, 1.0); B2 175 x
   (1.0, 10.0, 1.0) and 3 x (0.4, 10.0, 1.0). **[H]** nothing changes it in play: no direct write of `0x66f00c` /
   `0x73d9ac` in either decomp, no `Rolloff` script callback string in either ELF (BF2 has `ScriptCB_SetDopplerFactor`,
   which fits +0x18 = 0.4 in three states); a write through a pointer is not ruled out by a search.
6. **[V] BF2 is the same function**: B2 `FUN_005407e8` (header 673848), the two quotients at 673869-673870 and
   673880-673881 with the same offsets and the same `*(*argument 1 + 0x1c)` factor; B2 listener constructor
   `FUN_0053da70` (671618) `a local[7] = 0x41200000` (671630), array `0x73d990` (671419). BF2 adds one term B1 does not
   have: when the sign bit of the byte at P+0x20 is set the level is multiplied by a direction term floored at
   `*(P + 0x18)` (673883-673907) - a cone. Not ported; which key sets it is not traced [H].
7. **[V] The start multiplies the level by the sound space and refuses silence**: `FUN_004599e0` 518971-518975:
   `a local = a local * FUN_0044db38(a local)`, where `FUN_0044db38` (header 511001) returns
   `(*(argument 1 + 0x18) + *(argument 1 + 0x20)) * 0.5` of a struct `FUN_0044d650` (510811) fills with 1.0; then
   `if ((a local == 0.0) && (argument 5 != 0)) return 0`. Before the read, `FUN_0046a2f8(a local, a local, a local ==
   …(a local + 0x40))` (518970) leaves +0x18 / +0x20 at 1.0 only when the sound's space IS the listener's space;
   otherwise it copies them from the sound's space - `obstructionhfgain` 0x0db31274 and `occlusionhfgain` 0x7c8a1b8b,
   clamped 0..1 (`FUN_0044d698`). So a sound in another sound space than the listener is scaled by the mean of the
   two (verifier, RAM: the default space 0x676020 holds 0.5 / 0.5 or 0.1 / 0.1 in 38 BF1 states, 0-9 spaces per
   map). **Not ported** - card note on AMBM. With no 3-D instance (`argument 1 == 0`) `a local = *(a local + 0x14)`
   (518953) and no distance is read; `FUN_0045be28` (520465-520469) picks that branch by the `mode3d` bit of claim 8.
   This start-time value is the cull and the voice-steal priority; the playing volume is level x gain in the voice
   update (`FUN_0044c890` 510162, called from `FUN_00457020` 517304: level x gain only when the `mode3d` bit is set,
   otherwise the gain alone), which is what the port computes. No caller applies a second distance curve.
8. **[V] `mode3d`**: key 0x1a8b16b5 (`fnv1a_lowercase("mode3d")` [L]) in B1 `FUN_0044c938` (header 510189, 510203):
   value > 0 sets bit 0 of the byte at +0x34 of its first argument; `FUN_0045a830` calls it with `a local + 0x10`
   (519665), so the bit is the property's +0x44. `FUN_004599e0` reads `…(a local + 0x44) & 1` (518980) to
   pick the voice set. **[D]** a property with `mode3d` 0 is a 2-D sound and is not attenuated - from the disc, not
   from the two branches of claim 7 (the start and the voice update both skip the level when the bit is clear), and
   the disc agrees: of the two discs' sound / stream properties with `mode3d` 1, 50,784 of 50,946 carry distances;
   with `mode3d` 0, 223 of 10,000 do (section 2). No `mode3d` anywhere in the chain is read as 2-D [H] (the bit's
   constructor default is not traced) - PROVISIONAL in code (`SoundProp::flat`).
9. **[V] A mission script can rescale a group's distances**: `ScaleSoundParameter` (B1 header 32503, 0x0018d278; B2
   0x0019fcb0) multiplies a named group's rolloff / min / mute / max. **[L]** only 12 BF2 space missions (`spa*`)
   call it, no BF1 script and no ground map does (`grep -rlia ScaleSoundParameter extracted/bf{1,2}/lua`), so the
   disc values are the run-time values on every map the port ships.

## 2. The disc [L]

`lead_tmp/sndt/census_inherit.py` over the `snd_` dumps of every SOUND LVL + CORE.LVL of both discs (66,308 records):
- 59,808 records have `Inherit`; 58,459 name a template their own LVL does not define; resolving through the game's
  CORE.LVL leaves **0 unresolved**. (The bake test's own count over the 35 SOUND LVLs: 59,790 `Inherit`, 0 unresolved.)
- 32 names are defined both in a planet LVL and in CORE; none of them is anything's template, so the lookup order
  (own LVL, then CORE) cannot change a result.
- `mutedistance` is set on 3,178 records, equal to `maxdistance` on all but 95 resolved properties
  ((min, mute, max) = (10, 30, 50), (1, 50, 60), (20, 50, 80) x 29 each, (7, 200, 350) x 4, ...; mute < max always).
- `rollindistance` > 0 on 372 resolved properties ((rollin, min) = (50, 50) x 205, (55, 60) x 109, (100, 100) x 38).
- CORE templates: weapons 0xB1AD507C gain 0.7, 5 - 400 m, rolloff 6.0 (BF2 3.5); explosions 0xBFA71AD3 30 - 450 m,
  rolloff 1; foley 0xFE30AE3A gain 0.7, 10 - 50 m, rolloff 3; chatter 0xCFFF9324 gain 0.6, 16 - 150 m, rolloff 3.

## 3. Checks run

```
re_check.py fn b1 518971  -> FUN_004599e0 (0x004599e0, header 518898)
re_check.py fn b1 510250  -> FUN_0044ca78 (header 510248)
re_check.py fn b1 510573 / 510609 -> FUN_0044d130 (header 510563)
re_check.py fn b1 508184  -> FUN_00449f78 (header 508173)
re_check.py fn b1 510200  -> FUN_0044c938 (header 510189)
re_check.py fn b1 519665  -> FUN_0045a830 (header 519528)
re_check.py fn b2 673870 / 673881 -> FUN_005407e8 (header 673848)
re_check.py fn b2 671630  -> FUN_0053da70 (header 671618)
re_check.py callers b1 FUN_0044cac8 -> 5: FUN_0035e4c8 349081, FUN_0044c890 510162, FUN_0044cb78 510331,
                                         FUN_0044e570 511391, FUN_004599e0 518962
re_check.py callers b2 FUN_005407e8 -> 5: FUN_00443750 503551, FUN_00540568 673719, FUN_00540b98 674023,
                                         FUN_005427e0 675159, FUN_0054cba8 681957
re_check.py writers b1 0x66f00c -> 0;  writers b2 0x73d9ac -> 0   (pointer writes not visible)
re_check.py absent b1 DAT_0066f00c -> 0 hits; absent b2 DAT_0073d9ac -> 0 hits
strings SWBFRONT.ELF / SWBFRNT2.ELF | grep -i rolloff -> RollOff(%01.02f), RoomRollOff(%02.02f), RollOff, RoomRollOff
                                         (debug prints / keys; no ScriptCB)
```
The constant 10.0 is an immediate in the constructor (0x41200000 at 508184 / 671630), checked in RAM instead of by
`word` (the listener array is BSS).

## 4. Verifier

One foreground Opus verifier, claims and citations only, 9 claims: 7 SUPPORTED, 2 PARTLY (the sound-space copy of
claim 7 and the "whole level" summary), 0 NOT SUPPORTED; folded in: the space gains, the start value being the cull /
priority and the voice update the volume, `ScaleSoundParameter` (claim 9), `FUN_0044d130`'s single caller
(`FUN_0044c938` 510222 with `+0x18`, so property +0x28 - the same struct as claim 3), the key names `modedoppler`
0x0fba43f2 and the space keys. Its own RAM sample (40 + 40 states, all four listener slots) read 10.0 everywhere;
its ELF scan found no store to the four listener +0x1c addresses in B1 (a write through a kept pointer is not
excluded: claim 5 stays [H] on "never changes").

## 5. Not settled

- The card named `FUN_0044db38` as the level function: it is the sound-space average of claim 7.
- The default of a 3-D property's distances when no link of its chain sets them (162 stream properties): the port
  keeps 5 / 50 / rolloff 1, PROVISIONAL.
- Whether `rollin` (0x1ce8c2e1, 22 records, P+0) is read anywhere; the level function does not read it.
- BF2's cone term (no writer found for its flag or floor; B2's parser `FUN_00541320` keeps the flags at +0x20 and has
  one more key, 0x47c87922 -> +0x1c, unnamed); the sound-space gains (claim 7); the doppler branch; a voice flag
  (`+0x74 & 1`) under which the voice update doubles the mono gains (B1 517258-517263), setter not traced.
