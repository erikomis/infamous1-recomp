# Pop-in: what decides when things appear (inFAMOUS, NPUA80480)

## The answer: the draw-distance table, and a knob on it

The distance cull is the SPU job **"visprog"** (job module
`jobmod_50f371dc382d_e3d90`, EBOOT `0x636300`; the string `visprog` at
`0x5B9540`, registered by `func_003394B8`; a PPU copy of the interpreter at
`func_00339AB0`). A scene node does not carry its own distance: a byte in it
(record `+1`) picks one of eight classes from

    0x0078AE90  f32[8]  0, 400, 750, 1500, 3500, 6000, 12000, 1e6

`func_00252130` builds the job's parameters each frame: the camera position
at `+0x00`, the classes 1..7 times the camera's LOD scale (`camera+0x2A8`)
at `+0x48..`, and 0.6 (`[0x786D30]`) at `+0x64`. The job draws a node while
`|node - camera|^2`, with height counted at 0.6, is under the class distance
squared (`spu_recomp.c` functions `0x61A8`, `0x6358` and three more node
formats). `func_004B6EAC`, `func_004B80F8`, `func_00115890` (two class
bytes, +0x2E and +0x2F: MRD and IMPOSTER_MRD?), `func_00112A90` and
`func_000C9A48` read the table the same way. Nothing writes it.

`camera+0x2A8` is set by `func_00335BA8`: 1.0, a zoom term, then the product
of the LodAdjust list (below), reached there as `0x91A4D0 + 0x100`, which is
why a search for `0x91A5D0` itself missed the reader.

**Knob** (`../../draw_distance.c`): `PS3_DRAW_DISTANCE=<1-4>` scales
classes 1-6 of the table once at boot (0 and "always" stay), after checking
it is the PS3's; `PS3_DRAW_DISTANCE_PCT` in `PS3_TOGGLE_FILE` changes it live.
Launcher: "Draw distance" (PS3's / 1.5x / 2x / 3x).

**Measured** (Erik's save, rooftop of "3. The Escape", `PS3RECOMP_ENG_DRAWSTATS`,
one run, the factor changed live): looking down towards the street 10.3k
draws a frame at 1x, 17.9k at 4x, 10.2k back at 1x, 16.4k at 2.5x; facing the
water 4.1k vs 6.5k. The rooftop views themselves look the same (the extra
nodes are hazy, off frame or in the shadow maps); about 24 -> 21 fps headless
at 4x there. Still to see in play: the street in motion. Impostor swaps and
streaming have their own distances and are not changed by it.

---

The research that led there:

## Names: the OID manifest

The EBOOT has almost no names because the engine refers to everything by
OID, and `USRDIR/cache/oid_manifest.txt` (155k lines, in the game's own
files) lists them: **OID = line number - 1**. Checked on the hit points the
heartbeat uses: the property record at `0x739D44` holds OID `0x9FF`
(`MAX_HIT_POINTS`, line 2560) with offset `0x750`, and the next one OID
`0xA00` with `0x754`, the two floats found by code shape before
(`../health/README.md`). With it, any name in the manifest can be looked up
in the EBOOT's data (script function tables, property records, OID lists).

## What the names say about LOD

Around `LOD` (OID `0x4F6`) the manifest has the art pipeline's build rules:
`ASSIGN_TO_IMPOSTER`, `imposter`, `LOD`, `LOD_OVERRIDE_BIT`, `SPEEDTREE_FILE`,
`uLod`, `MRD`, `IMPOSTER_MRD`, `MERGE_GEOM`; further on `PM_LOD` with `type`,
`percent`, `distances`, `ranges`, and `RENDER_RADIUS` / `RENDER_CENTER`.
`MRD` reads as "max render distance": a per-object distance baked into the
level data by the tools, with impostors (flat stand-ins) taking over past
`IMPOSTER_MRD`. So the pop-in is most likely each object's own baked
distance, compared at run time (PPU or an SPU culling job) with the camera's
distance; the fix to look for is a multiplier at that comparison.

## LodAdjust (the scripts' multiplier)

The level scripts have `LodAdjustCreate(rSLod)` (script function table
`0x790DF0`, code `0x394490` -> `0x500F8`): a 0x20-byte object (vtable
`0x75DBA0`, the factor at `+0x14`, next/prev at `+0x18`/`+0x1C`) appended to
a list at `0x91A5D0`; the destructors (`0x4DA1E0`, `0x4E5780`) unlink it.
An earlier note here said nothing reads that list: wrong, `func_00335BA8`
multiplies the camera's LOD scale by every factor in it, reaching the list as
`0x91A4D0 + 0x100`. The list was empty on the rooftop, and a node appended
by hand (0.25, 4) changed nothing visible in the view facing the water; why
is not settled (that camera may not be the one visprog uses, or nothing in
that view sat near a class boundary). The table knob above does not depend
on it.

## Next

- Play a street with the knob at 1x and 2x: does what popped in now appear
  earlier, and at what cost in frame rate.
- Impostors (`IMPOSTER_MRD`, the second class byte `func_00115890` reads)
  and streaming have their own distances: if buildings still swap or load
  late, those are the next places.
