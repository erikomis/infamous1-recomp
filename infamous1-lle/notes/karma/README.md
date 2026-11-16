# Cole's karma in guest memory (inFAMOUS, NPUA80480)

For the DualSense light bar (`../../karma_light.c`).

## The answer

    0x007D6110  the progress object (its address is in the constant pools of
                32 functions); +0x10 is the save game's STATE image
      +0x18     f32   karma: -0.0 in a new game, 50.0 in Erik's save
                      ("Reputation: Guardian"), read by 34 game functions

The ranks are the game's own: `func_0010B1D0` (called with the karma by
`func_0006DB58`) answers 3 for exactly 0 and otherwise the index of the first
range of the table at `0x786CF0` that holds it, bounds included:

| index | range | rank |
|---|---|---|
| 0 | -800 .. -500 | Infamous |
| 1 | -500 .. -200 | Outlaw |
| 2 | -200 .. 0 | Thug |
| 3 | 0 (exactly) | neutral |
| 4 | 0 .. 200 | Guardian |
| 5 | 200 .. 500 | Champion |
| 6 | 500 .. 800 (no match) | Hero |

## How it was found

The EBOOT has no script property names (`KARMA`, `KARMA_CLASS` in
inFAMOUS 2); its only karma string is the shader constant `g_rgbKarma`, whose
registration (`0x75B470`, a material class's factory `func_004DC168` and
parameter binding `func_004DB020`) does not lead to the value.

The saves did. Erik's autosave after the tutorial ("3. The Escape",
`DETAIL=Reputation: Guardian`) and the tutorial's ("2. First Glimpse", no
reputation) differ in 36 bytes of `STATE`; the first is the f32 at offset 8:
`-0.0 -> 50.0`, followed by two u32 `12 -> 542`. A run on a copy of Erik's
save with `PS3_MEMDUMP_AT` (200 and 250 s, static and heap) found the run
`50.0, 542, 542` once, at `0x7D6128`: the STATE image at `0x7D6120`, inside
the object at `0x7D6110`. Its address is in 32 constant pools; following
each function's TOC load into its pool showed the field reads (34 of `+0x18`,
the karma, all as floats), among them `func_0006DB58` handing it to the rank
function above.

## Checked in the game

`PS3_KARMA_LOG=1` on copies of both saves (`out/karma-check-*.log`):

    Erik's save:    [karma] 0.0 (neutral)   (before the save loads)
                    [karma] 50.0 (Guardian)
    tutorial save:  [karma] 0.0 (neutral)

Still open: a karma change in play (a karma moment, a civilian hurt or
healed) seen moving the value and the light bar on a DualSense.
