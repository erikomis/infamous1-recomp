# Cole's health in guest memory (inFAMOUS, NPUA80480)

What `heartbeat.c` reads, how it was found, and how it was checked. Found on
2026-11-13 following the inFAMOUS 2 port's recipe
(`infamous2-lle/notes/health/README.md` in erikomis/infamous2-recomp), which
had to change at its first step: this older build of the engine does not
carry the script property names in its EBOOT. Tools in this folder:
`elftool.py` (from the inFAMOUS 2 port), `scan_patterns.py`,
`heap_candidates.py`; the EBOOT is APP_VER 01.00, SHA-256 `5e22402a…`.

## The answer

| What | Where | Notes |
|---|---|---|
| the hero object | `u32` at **`0x008F7580`** | field `+0x30` of the object at `0x008F7550` (vtable `0x0072A620`; 21 functions' constant pools hold `0x8F7550`). 0 while there is none. |
| its vtable | `0x00727A80` | the only heap object of that class in the dump |
| maximum hit points | `f32` at hero **`+0x750`** | 400.0 at the start |
| current hit points | `f32` at hero **`+0x754`** | 400.0 unhurt |
| the world clock | `0x007BC7D0` | 141 constant pools; dt at `+0x00`, scale at **`+0x10`** (`0x007BC7E0`), elapsed 1/1440 s at `+0x04` |

The game's own low-health effect tests the same pair: `func_0027CD08` reads
`[[this+0x30]+0x38]`, and when `current < max * 0.33` (the 0.33 is in its
constant pool) it runs its vector math; at that share the picture is black
and white (below). All values are big-endian.

Not found: inFAMOUS 2's HUD-hide and letterbox counters and its pause count
(their names are not in this EBOOT), the recovery state machine, and karma
(the only karma string is the shader constant `g_rgbKarma`).

## How it was found

### 1. No names, so the code's shape

`strings -a EBOOT.elf | grep -i hit_point` finds nothing (only 154 upper-case
names in the whole EBOOT; inFAMOUS 2 has its script tables full of them). So
the search went by the shape of inFAMOUS 2's hit-point code instead:
`scan_patterns.py` looks for the two places inFAMOUS 2 uses both floats
(maximum just before current, 4 bytes apart):

- adjust: `lfs cur; fadds; lfs max; fsubs; fsel`, i.e. `min(cur + delta, max)`
- threshold: `lfs max; fmuls; lfs cur; fcmpu`, i.e. `cur` against a share of `max`

On inFAMOUS 2's EBOOT it finds exactly its known pair (+0x7EC/+0x7F0) in both.
On this one the threshold form gives **+0x750/+0x754**, at `0x1ECC14`
(`cur > max * [this+0x1014]`) and `0x27CD8C` (`cur < max * 0.33`, on
`[[this+0x30]+0x38]`, after a type check through the object's vtable).

### 2. The hero, from a memory dump

No static global loads the object and reads +0x750/+0x754 within a function
(inFAMOUS 2's hero pointer is found that way; here the code reaches him
through objects it is given). So the running game was asked instead: the
tutorial (scripted pad, `tools/capture.sh`, 215 and 225 s) with
`PS3_MEMDUMP_AT=0:1000000:215:static-215.bin,40000000:10000000:215:heap-215.bin,...`
(the same at 225 s), then `heap_candidates.py`:

    311 candidates with value >= 10
    obj 4a0fe0f0 vt 00727a80 value 400.0  520 refs  ...
    obj 49ed0610 vt 00730090 value 255.0   80 refs
    obj 4a26bac0 vt 00722b58 value 150.0   48 refs   (and a dozen more 150.0 of that class)

The one object of its class, 400.0 of 400.0 in both dumps and pointed at
from 520 places is Cole. Seven static words hold its address; the one to use
is the one the code uses: `0x8F7580`, field `+0x30` of `0x8F7550`, whose
address is in 21 constant pools (`elftool.py refs 8F7550`). The world clock:
in the static dump, a dt in (0, 1/15] with scales of 1.0 at +0x10/+0x20/+0x30
and an elapsed count that grows about 1440 a second between the dumps:
`0x7BC7D0`, in 141 pools.

## 3. Checking it in the running game

A check build (`cmake -B build -DPORT_HEARTBEAT_POKE=ON`) writes Cole's
current hit points at given times. Run `hb1`: the tutorial with
`PS3_HEARTBEAT_LOG=2 PS3_RUMBLE_LOG=1 PS3_HEARTBEAT_POKE=225:120,255:20,280:400`
(`proof/hb1-heartbeat.log`):

    t=20.30   health 400.0/400.0 hero=4A0FE0F0 hud-hidden | scale=0     loading
    t=20.85   health 400.0/400.0 hero=4A0FE0F0 | scale=1
    t=225.02  poke: hit points 400.0 -> 120.0
    t=225.02  on: health 30% (120.0/400.0), 80 bpm
    t=255.03  poke: hit points 120.0 -> 20.0                            beating faster
    t=280.01  poke: hit points 20.0 -> 400.0
    t=280.02  off: health back (400.0/400.0)
    [cellPad] host pulse port 0: lo 34639, hi 12123 -> motors 34639, 12123

The game's own picture follows the number (`proof/hb1-*.jpg`, mean
saturation of the frame): full colour at 400 (99), black and white with
blood at the edges at 120 (38) and at 20 (43), full colour again at 400
(101). The hero object was at the same address as in the dumps; the clock's
scale read 0 through every loading screen.

## Still open

- **The game's recovery.** A written value stays where it is (the
  inFAMOUS 2 poke also marks a hit in its recovery state machine; this one's
  is not known). A real hit, and a death and respawn (does `0x8F7580` follow
  a new hero object, as inFAMOUS 2's pointer does?), are still to be seen.
- **Cutscenes.** Only the clock's scale silences the heartbeat; a cutscene
  that keeps the world clock running is not told apart.
- **Karma**, for the light bar: not found.
- `../../game_clock.c` names the inFAMOUS 2 clock (`0xA2C500`), outside this
  EBOOT's image; this game's is `0x7BC7D0`. It is only a diagnostic, off by
  default.
