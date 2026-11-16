# Pop-in: what decides when things appear (inFAMOUS, NPUA80480)

Under study; nothing changed in the game yet.

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

## A dead end: LodAdjust

The level scripts have `LodAdjustCreate(rSLod)` (script function table
`0x790DF0`, code `0x394490` -> `0x500F8`): a 0x20-byte object (vtable
`0x75DBA0`, the float at `+0x14`, next/prev at `+0x18`/`+0x1C`) appended to
a list at `0x91A5D0`; the destructors (`0x4DA1E0`, `0x4E5780`) unlink it.
Nothing else reads that list: no other function loads its address from a
constant pool, no word in a runtime memory dump (static or heap, Erik's save
on the rooftop) points at it, and the list is empty there. A node appended
by hand with factor 0.25 or 4 changed nothing in the rooftop view. Either
unused in this build or read where no dump showed it.

## Next

- A scene with pop-in that a scripted pad can reach (the street below the
  rooftop of "3. The Escape"), frames every few seconds while walking.
- Find the comparison: the instance records' MRD floats in a memory dump of
  that scene (values that match the distances objects pop at), then the code
  (PPU or SPU job) that reads them.
