# Per-map sky, sun, ambient and fog — `sky_` and `lght`

Status: **[V] format + name dictionary verified; [H] the rendering reduction is
ours and PROVISIONAL.** 2026-08-31, Opus session 3.

Ported in `mungefs::sky`, `mungefs::light`, `bake::lighting`. Everything below
is a measurement over the discs or a confirmed hash preimage; the rendering
choices built on top are marked PROVISIONAL and live in `bake::lighting`.

## 1. Where a map's look is stored

Every one of the 285 map/side/common LVLs on the two discs was walked.

| | BF1 | BF2 |
|---|---|---|
| `sky_` chunks per map | exactly 1 (16/16 maps) | exactly 1 (18/18 maps) |
| `lght` chunks per map | **zero, on every map** | 1–21, of which 1–3 are non-stub |
| `SunInfo` | 16/16 | 1/18 (GEO1 only) |
| `SkyInfo.AmbientColor` | 16/16 | **0/18** |
| `SkyInfo.FogColor` | 14/16 | 12/18 |

So the two games use two different pipelines for the same thing: **BF1 puts the
sun and the ambient in `sky_`; BF2 moved them into `lght`** (`GlobalLights` +
the `Light` block it names). BF2 keeps fog in `sky_`.

A stub `lght` is 12 bytes: a bare `NAME` whose u32 is `fnv1a_lowercase` of a
`.LGT` layer name, with no DATA/SCOP. Fifteen BF2 maps have exactly one
non-stub `lght`; **DEA1, FEL1 and UTA1 have three** — a world layer carrying
`GlobalLights` plus local point-light sets (`jail_blue1`, `river11`, `macy16`).
Take the one with `GlobalLights`, not the first.

## 2. Container format — and an erratum

Both chunks are munged config trees (`mungefs::config`): a flat run of `DATA`
records where a following `SCOP` attaches to the previous `DATA` as its
children.

```
DATA = [u32 fnv1a_lower(name)][u8 argc][argc x u32 args][u32 blobLen][blob]
```

**ERRATUM (2026-08-31).** The A2-era note said a string argument "is encoded as
the u32 4 in its slot". That is only the one-argument case. The real rule:

> a string argument's slot holds **`4 * argc + the byte offset of that string
> inside the blob`**, and the blob packs NUL-terminated strings back to back.

Byte-verified on BF2 MUS1's `sky_`:

```
argc = 3, args = [12, 38, 0x40C00000], blobLen = 35
blob = "mus_sfx_lavaspray_skydome\0hp_lava5\0"
  12 -> 12 - 12 =  0 -> "mus_sfx_lavaspray_skydome"
  38 -> 38 - 12 = 26 -> "hp_lava5"
  0x40C00000                        -> 6.0f
=> Effect("mus_sfx_lavaspray_skydome", "hp_lava5", 6.0)
```

`4 * argc` is exactly why a one-argument string record stores `4` and why
`lght`'s `Light("sun")` (argc 2) stores `8` — the 8 was unexplained until this
rule was found. The old reading returned only the first string and read every
later string slot as a denormal float. Fixed in `mungefs::config`
(`ConfigNode::strings` / `str_arg`), with a regression test on the MUS1 record.

## 3. Property names — recovered by confirmed preimage

Names survive only as `fnv1a_lowercase` hashes. They were recovered the way
`bake sound-names` recovers sound names: hash candidate strings, keep only
exact preimages. Candidate corpora, in the order they paid off:

1. Real shipped `.sky` / `.LGT` source files published by the PC mod community
   (Zero Editor's own output — the same authored text these chunks were munged
   from). These supplied the sky/light vocabulary verbatim.
2. `PrismaticFlower/swbf-unmunge`'s curated 3,850-name hash table, which
   cracked the whole PS2 weather family in one pass.
3. The two PS2 ELFs and the PC `BattlefrontII.exe` (`strings`). These gave the
   common short names — and nothing sky-specific: the PC executable contains
   **none** of these property names.
4. Generated CamelCase combinations — one hit, treated as unproven.

Result: **122 of 123 `sky_` hashes and 23 of 23 `lght` hashes confirmed.** The
one holdout is `0xed4a20e6` (`DomeInfo`, one f32 = 1.0, BF2 COR1 and KAS2
only); a 7.4e9-candidate search found nothing that is not an expected false
positive.

**A hash match alone is evidence, not proof.** `Type` and
`StaticReflectionName` genuinely collide at `0x5127f14d` — a real FNV-1a-32
collision between two plausible names. The field is `Type` (the `lght` values
are 1/2/3 = directional/omni/spot). Names that also appear verbatim in a
shipped source file are the ones to trust.

Two names whose preimage is confirmed but whose VALUES do not fit:

- `Enable` (`0xaf8bb8ce`, `SkyInfo`) stores **1559.0** on eight BF1 maps
  (BES1, BES2, GEO1, KAS2, NAB1, NAB2, TAT1, TAT2) and 1.0 on the other eight.
  Byte-verified, not a parse slip. Either the name is another collision or the
  field is not a boolean. Not used.
- `Ambient` (`DomeInfo`) stores **256, 256, 256** on BF1 KAS2 and NAB1 — so it
  is not a u8 and must not be clamped as one.

Colour scales are per field, measured over every map:

| field | scale |
|---|---|
| `SkyInfo.*Color`, `SunInfo.Color`, `SunInfo.BackColor`, `DomeInfo.Ambient`, `LowResTerrain.FogColor` | 0..255 |
| `lght` `GlobalLights.Top` / `Bottom` | 0..255 (max seen 243) |
| `lght` `Light.Color` | **0..1** (max exactly 1.0; values like 0.9 and 0.65 are hand-typed, not k/255) |
| `RainInfo.DomeColor` | 0..1 |

## 4. Two conventions, cross-checked against each other

> **Superseded by `analysis/re-sun-direction.md` (card SUND, 2026-09-26, [O]).** Both conventions below are wrong, and
> they agreed only with each other:
> - BF1's sun is L = (−sin a0·cos a1, cos a0, −sin a0·sin a1) from `SunInfo.Angle` in degrees (B1 FUN_004931e0). This
>   section's reading is its 90° azimuth rotation.
> - BF2's light direction is row 2 of the matrix of q = (x, y, z, w) = (−a3, a0, −a1, a2), from `Rotation` as stored
>   (B2 FUN_00596978 → FUN_0053ac20). This section's reading is its z-mirror.
> - Both match RAM on every map with a PCSX2 state (≤ 0.004°).
> - BF2 never reads `SunInfo.Angle`; on the PS2, GEO1's two stored suns are 43° apart.

**`lght` `Rotation` is (w, x, y, z) and a light's forward is its rotated −Z
axis.** **`SunInfo.Angle` is (elevation + 90, −azimuth) in degrees**, giving
direction-to-sun `(cos el · sin az, sin el, cos el · cos az)` in the engine's
Y-up world.

These are not assumptions. BF2 GEO1 is the only map that stores its sun BOTH
ways — `lght Light "sun" Rotation(-0.4091, -0.8727, 0.2625, -0.0462)` and
`sky_ SunInfo Angle(140, -10)`:

| reading | direction to sun | elevation | azimuth |
|---|---|---|---|
| quaternion (w,x,y,z), rotate −Z | (0.134, 0.738, 0.661) | 47.6° | 11.5° |
| `SunInfo.Angle` per the rule above | (0.112, 0.766, 0.633) | 50.0° | 10.0° |

**They agree to 2.8°.** All 24 combinations of (w,x,y,z)/(x,y,z,w) with the six
signed axes were tried; nothing else lands within 20°. Pinned as an executable
test (`mungefs::sky::tests::geo1_sun_angle_matches_its_light_quaternion`).

Supporting measurement: `Angle[0]` lies in 120..180 on all 16 BF1 maps, i.e.
30..90° above the horizon, which is what a sun angle must do and what the raw
value cannot do if read directly. `Angle[1]` ranges −55..280, i.e. a free
azimuth.

## 5. What each map authors

| map | SunInfo Angle | SunInfo Color | Ambient | Char.Ambient | FogColor | FogRange | lght key light | Top | Bottom |
|---|---|---|---|---|---|---|---|---|---|
| BF1 BES1 | 130, -55 | 242,219,95 | 90,90,135 | 90,90,135 | 255,0,0 | 0..2700 | — | — | — |
| BF1 BES2 | 120, 180 | 255,200,100 | 50,50,70 | 100,100,120 | 128,128,128 | 250..350 | — | — | — |
| BF1 END1 | 140, -10 | 255,255,255 | 100,110,100 | 140,140,120 | — | — | — | — | — |
| BF1 GEO1 | 140, -10 | 120,120,120 | 120,101,76 | 209,156,73 | 252,252,252 | 0..3000 | — | — | — |
| BF1 HOT1 | 120, -20 | 160,160,160 | 255,255,255 | 166,169,173 | 128,128,128 | 0..3000 | — | — | — |
| BF1 KAM1 | 155, 1 | 65,85,105 | 100,100,100 | 145,143,161 | 128,128,128 | 600..1200 | — | — | — |
| BF1 KAS1 | 180, 0 | 120,120,130 | 90,90,100 | 120,120,135 | — | — | — | — | — |
| BF1 KAS2 | 145, 280 | 210,200,190 | 78,87,91 | — | 128,128,128 | 0..3000 | — | — | — |
| BF1 NAB1 | 140, 10 | 210,200,190 | 105,105,105 | — | 128,128,128 | 0..3000 | — | — | — |
| BF1 NAB2 | 145, -30 | 250,245,165 | 67,63,87 | 95,78,78 | 128,128,128 | 0..30000 | — | — | — |
| BF1 RHN1 | 130, 140 | 235,245,255 | 110,120,133 | 100,100,130 | 200,200,200 | 0..3000 | — | — | — |
| BF1 RHN2 | 130, 55 | 148,148,160 | 80,85,95 | 80,85,95 | 128,128,128 | 0..3000 | — | — | — |
| BF1 TAT1 | 130, -55 | 242,219,95 | 80,75,100 | 100,95,120 | 0,0,10 | 0..2700 | — | — | — |
| BF1 TAT2 | 130, 120 | 255,240,210 | 120,118,140 | 130,128,150 | 128,128,128 | 0..30000 | — | — | — |
| BF1 YAV1 | 140, -30 | 240,240,235 | 70,70,70 | 75,75,75 | 128,128,128 | 1800..3000 | — | — | — |
| BF1 YAV2 | 150, 0 | 70,78,80 | 100,100,100 | — | 128,128,128 | 1800..3000 | — | — | — |
| BF2 COR1 | — | — | — | — | — | — | sun type 1 | 0,0,0 | 0,0,0 |
| BF2 DAG1 | — | — | — | — | 95,111,126 | — | sun type 1 | 101,114,127 | 70,90,115 |
| BF2 DEA1 | — | — | — | — | — | — | sun type 1 | 65,65,65 | 70,65,65 |
| BF2 END1 | — | — | — | — | 144,153,156 | 90..350 | sun type 1 | 80,80,80 | 70,70,70 |
| BF2 FEL1 | — | — | — | — | 80,100,80 | -10..62 | sun type 1 | 70,70,60 | 80,80,60 |
| BF2 GEO1 | 140, -10 | 120,120,120 | — | — | 0,0,0 | 0..0 | sun type 1 | 100,80,50 | 90,60,30 |
| BF2 HOT1 | — | — | — | — | 203,201,203 | 100..1000 | sun type 1 | 170,170,170 | 170,170,170 |
| BF2 KAM1 | — | — | — | — | 0,0,0 | 400..1356 | sun type 1 | 100,100,100 | 88,88,88 |
| BF2 KAS2 | — | — | — | — | 104,106,108 | 0..0 | sun type 1 | 100,100,100 | 50,50,55 |
| BF2 MUS1 | — | — | — | — | 198,12,8 | — | sun type 1 | 30,10,10 | 0,0,0 |
| BF2 MYG1 | — | — | — | — | 194,197,199 | 0..600 | sun type 1 | 191,191,191 | 88,97,97 |
| BF2 NAB2 | — | — | — | — | — | — | sun type 1 | 66,77,102 | 0,0,0 |
| BF2 POL1 | — | — | — | — | 180,180,200 | 0..0 | sun type 1 | 150,150,170 | 90,90,120 |
| BF2 TAN1 | — | — | — | — | — | — | sun type 1 | 243,243,243 | 52,52,54 |
| BF2 TAT2 | — | — | — | — | — | — | sun type 1 | 90,90,90 | 120,120,120 |
| BF2 TAT3 | — | — | — | — | 0,0,0 | 20..40 | sun type 1 | 80,80,80 | 90,90,90 |
| BF2 UTA1 | — | — | — | — | — | — | sun type 1 | 85,85,90 | 95,95,100 |
| BF2 YAV1 | — | — | — | — | 144,153,156 | 90..350 | sun type 1 | 75,83,85 | 42,44,20 |

(`—` = the map does not author that field. Regenerate with the reproduction
command below.)

## 6. Sky domes and the low-res terrain

**Elevation census (added 2026-08-31, Opus session 5).** The two domes
measured below are no longer the only data: `mungefs --example
sky_dome_census` measures every shipped `DomeModel` mesh at the scale the
engine draws it. **29 of the 32 baked maps have a base dome that reaches below
the horizon** (seven below -88 degrees); only BF1 KAS1, BF1 KAS2 and BF2 UTA1
stop at 0. **Five maps** ship a shell that is entirely below the horizon
(six shells, counting BF1 NAB1's two — and NAB1's `sky_mountains` is 74 %
below rather than wholly): BF1 BES1 `bes1_sky_dome_clouds_2`, BF1 NAB1
`sky_water`, BF1 RHN2 `rhenvar2_skydome_clouds_2`, BF2 COR1
`cor1_sky_buildings_1`, BF2 GEO1 `geo_sky_explosions`, and BF1 NAB1
`sky_mountains` at 74 %. Full table in
`analysis/RE_LOG.md`.

**`DomeInfo.Ambient` (added 2026-08-31, session 5)** takes exactly two values
over all 34 maps and both are neutral greys — `255`/`256` on 25 maps and `128`
on four (both Hoths, both Geonosises), absent on five. It carries no per-map
look, and reading 128 as a 0.5 multiplier halved those four skies.
`bake::lighting::dome_ambient` renders both neutrals as unity.

`DomeInfo` names real sky-dome models, and they are **already in each map's own
`modl` list** (so already baked): `tat1_sky_dome`, `hoth_sky_dome`,
`bes1_sky_dome`, plus parallax shells with an `Offset` and a `MovementScale`
(`tat1_skyrocks_01` at offset 100, movement scale 0.995). Across the 34 maps
there are 89 `DomeModel` entries: 74 resolve to a `modl` in the same LVL, 11
are authored placeholders with an empty `Geometry`, and 4 dangle
(`hoth_mountain_2` on both Hoths, `kas_moon` on BF1 KAS1,
`geo_sky_dome_arena` on BF2 GEO1 — none of the four exists in any of the 285
LVLs). `SkyObject` geometry (`imp_fly_destroyer_DOME`...) lives in the SIDE
LVLs, not the map.

`DomeInfo.LowResTerrain` names a texture, a `MaxDistance`, a `PatchResolution`
and its own fog: BF1 TAT1 stores `Texture("tat1")`, `MaxDistance(3000)`,
`FogNear(400)`, `FogFar(3500)`, `FogColor(164, 174, 194, 80)`. **This is
directly relevant to the open outside-the-level-terrain defect** — it is the
map telling us that something low-resolution is drawn out there, with its own
haze. Recorded in the manifest; nothing consumes it yet, because the `LOWR`
chunk format is still HOLD FOR FABLE.

## 7. The rendering reduction — ours, PROVISIONAL

`bake::lighting` turns the above into one sun + a hemisphere ambient + optional
fog per map. The three judgement calls (colour space, which maps get fog, the
ambient floor) are documented in that module and in the PLAN Log entry; they
are OUR rendering decisions, not decoded PS2 behaviour.

**Added 2026-09-25 (queue LITE, `analysis/re-model-lighting.md`, `[O]`).**
The PS2's own reduction is now read, and the port's misses the terms that make
it brighter:
- **COL0.** The disc's per-vertex colour on static models multiplies the lit
  colour, and on 0x200 segments it is added.
- **Prelit models.** Any model with a 0x200 segment gets no ambient and none of
  the `Static()` lights.
- **Light2.** BF2 lights normal models with `GlobalLights.Light2` as well.
- **Terrain.** BF2 draws it with its lightmap and no ambient or sun.
- **BF2 sun colour.** `lght` `Light.Color` is a gamma-space multiplier like the
  rest; the bake uses it raw as a linear value.
- **BF2's 0.5 scale.** It is `Effect("HDR")`, and its composite doubles the frame
  back.
- **HOLD 11(a)**, Hoth's clipping: the terrain is not lit by the ambient or the
  sun at all on BF2. The same question for BF1's terrain is queue TERL.

**HOLD FOR FABLE — what actually enables fog on the PS2.** Ten of the sixteen
BF1 maps ship `FogRange(0, 2700..30000)` with placeholder colours (six of them
the neutral 128/128/128, plus Bespin's 255/0/0 and Tatooine's 0/0/10); rendered
literally they turn those maps red, white and black. There is very likely an
explicit enable we have not confirmed a preimage for — `Enable` (§3) is the
obvious candidate and its values do not fit. The three heuristics in
`bake::lighting::fog_from` stand in for it.

**HOLD FOR FABLE — the `SunInfo` back light.** Every BF1 map authors
`BackAngle` / `BackColor` / `BackDegree` (TAT1: BackAngle(180, 0),
BackColor(150, 150, 150)). It is parsed and recorded but not rendered; whether
the PS2 lights with it, and how it combines with the key light, is not decoded.
**Answered 2026-09-26 (`analysis/re-sun-direction.md` §5, [O]):** neither ELF
reads it. Their three hashes appear nowhere, and BF1's model light set carries
the sun and three black null lights only. Card BACKL is closed as unused.

## 8. Reproduction

```
# per-map table (the throwaway dumper used above lives in the session
# scratchpad; the same data is in every baked manifest.ron)
grep -A 12 'lighting:' engine/assets_baked/*/manifest.ron
```
