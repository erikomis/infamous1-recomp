# inFAMOUS (NPUA80480) — static recompilation, firmware LLE

The current port. It builds against
[NoahBPeterson/ps3recomp](https://github.com/NoahBPeterson/ps3recomp) branch
`infamous/rebase-v0.13.0` (commit `770660a`, upstream PR #213) with the fixes
in `patches/`, and runs Sony's own liblv2, libsysmodule and libsre, lifted from
your firmware. It follows the
[inFAMOUS 2 port](https://github.com/erikomis/infamous2-recomp)'s
`infamous2-lle/`, and `../infamous1/` holds the local game files (`vfs/`).

## Status

On macOS/arm64 (Apple M4 Pro):

- Studio logos, the Bink movies (`splash_america.bik`,
  `tv_default_feed_2x2.bik`), music and streaming.
- The 3D title screen "Press the Start Button"; START (`0x0008`) leaves it.
- The opening blast and Cole in the prologue tutorial ("Use the right stick to
  look up at the chopper"). ~44 fps in the menu and cutscenes, headless.
- The launcher starts it in a Metal window (`BootCheck: type=2,
  titleId=NPUA80480`).

Not checked: anything past the tutorial, long sessions.

## Fixes

In the toolkit (`patches/ps3recomp-fork-770660a-with-upstream-prs.patch`,
applied by `../tools/setup_toolkit.sh`): the inFAMOUS 2 port's patch, plus

- **`sys_memory`:** the end of the allocator window is `PS3_MEM_ALLOC_SIZE`
  (hex, 16 MB to 3 GB, default 256 MB). The 256 MB window ran out during the
  boot's scan of the streaming cache, so the audio/streaming init never
  finished. `run.sh` sets 768 MB.
- **`cellPad`:** DualSense adaptive triggers. The physical trigger the map
  binds to R1 (the bolt; the right one when R1 is on neither) gets a gun
  click and reads as pressed at it (`PS3_PAD_FIRE_AT`, 50%), the other a
  light resistance, a buzz on the firing trigger while it is held and the
  game runs the motors; sent only on a change, freed on close and at exit,
  `PS3_PAD_ADAPTIVE=0` turns it off. Test: `libs/input/tests/test_pad_adaptive.c`
  (and a `SendEffect` stub in `test_pad_hotplug.c`).

In the port:

- `runtime_overrides/cellGame.c` reads `CATEGORY` from `PARAM.SFO` (`HG`) and
  reports an HDD boot; `run.sh` passes `PS3_BOOT_PATH`. Test:
  `tests/test_game_boot.c`.
- `game_patches.cpp` only applies patches whose program-header hash matches
  this EBOOT. `gpu_light.c` and `gpu_ssao.c` are placeholders: the inFAMOUS 2
  hooks reference that game's job modules.

## Layout

| Path | What it is |
| ---- | ---------- |
| `main.cpp`, `stubs.cpp`, `CMakeLists.txt` | The fork's project template, plus PARAM.SFO at boot and the `lle/` modules |
| `runtime_overrides/` | `cellGame.c` (HDD boot) |
| `patches/` | The toolkit patch and the original diffs of the upstream PRs it carries |
| `lle/`, `lle_spu/` | liblv2, libsysmodule, libsre and libsre's SPURS kernel (generated, not in git) |
| `recompiled/`, `spu/` | The title's PPU and SPU lifts (generated, not in git) |
| `tools/boot.sh`, `tools/capture.sh` | Timed headless run; headless run keeping a PNG every 10 s |
| `tools/play_prologue.sh` | Title screen → tutorial with a scripted pad |
| `tools/profile_run.sh` | Fps per period and the hottest code |
| `tools/decrypt_self.py` | Decrypts a retail SELF without RPCS3, with the player's key file (`--selftest` uses made-up keys) |
| `tools/pup_extract.py` | Takes the firmware modules out of Sony's `PS3UPDAT.PUP` |
| `tools/extract_eboot_jobmods.py` | The `C0DEC0DE` job modules embedded in the EBOOT (21 here) |
| `tools/log_triage.py` | Summarises a game log for a bug report |

## Rebuilding

`tools/rebuild.py` runs every step below in one go, from your RPCS3 install
(or PKGs) and your `.rap` into a separate folder that mirrors this repository
(`--out`, default `~/Library/Application Support/infamous1-recomp/install`),
and only redoes what changed on a rerun. `--dry-run` checks the requirements
and prints the plan; its output lines are what the launcher (`../launcher/`)
reads. Checked on 2026-11-11 from a clean install: `lle/`, `lle_spu/`,
`recompiled/` and `spu/` came out byte for byte the same as the tested build,
and the binary boots to the tutorial.

```sh
python3 tools/rebuild.py --dry-run --game "$HOME/Library/Application Support/rpcs3/dev_hdd0/game/NPUA80480" \
    --rap "$HOME/Library/Application Support/rpcs3/dev_hdd0/home/00000001/exdata/<content id>.rap"
```

Decryption goes through `rpcs3 --decrypt` by default, or `tools/decrypt_self.py`
with your own key file (`--keys FILE` or `PS3_KEYS`); `--firmware` takes a
`dev_flash` folder or Sony's `PS3UPDAT.PUP`. The decrypted EBOOT is checked
against the tested `APP_VER 01.00` build (SHA-256 `5e22402a…`) and the firmware
modules against 4.93; a difference only warns.

By hand, with `T=../ps3recomp-fork/tools` and the toolkit's venv as `P`:

```sh
# EBOOT: rpcs3 --decrypt (or tools/decrypt_self.py with your own keys) into game/EBOOT.elf
# firmware: lle/ and lle_spu/ are game-independent; same commands as the inFAMOUS 2 port

# the title
$P $T/ppu_loader.py game/EBOOT.elf -o out/
$P $T/ppu_lifter.py game/EBOOT.elf --functions out/EBOOT.functions.json \
    --hle-stubs out/EBOOT.imports.json --code-end 0x55EB44 \
    --max-chunk-lines 150000 --output recompiled/
$P $T/extract_spu_images.py game/EBOOT.elf -o spu/images          # 8 SPU ELFs
# the Wws_Job policy module: 11648 bytes at guest 0x69E400 (file offset 0x68E400)
dd if=game/EBOOT.elf of=out/wwsjob_pm.bin bs=1 skip=$((0x68E400)) count=11648
$P $T/wrap_spu_elf.py out/wwsjob_pm.bin --base 0xA00 --entry 0xA00 \
    --out spu/images/wwsjob_pm_at_0069E400.elf
$P $T/build_spu_workloads.py --images spu/images --lifted spu --out spu/spu_workloads.c \
    --register-fn infamous1_spu_register_all --constructor --title infamous1 \
    --eboot game/EBOOT.elf
$P tools/relift_spu_returns.py --toolkit ../ps3recomp-fork   # audio image call returns
$P tools/extract_eboot_jobmods.py game/EBOOT.elf --out spu/jobmods  # 21 job modules
$P $T/lift_jobmods.py --jobmods spu/jobmods --out spu \
    --register spu/jobmods_register.c --symbol-prefix infamous1
$P tools/relift_jobmod.py jobmod_542edebc6499_e80 --extra 0x18BB0

cmake -B build -G Ninja -DCMAKE_BUILD_TYPE=Release \
    -DPS3RECOMP_DIR="$PWD/../ps3recomp-fork" -DPython3_EXECUTABLE="$P"
cmake --build build -j 8
```

This EBOOT has no `DA7ABA5E` data modules, and the scanner found no
relocatable job blobs, so `lift_datamods.py` and `lift_relocmods.py` do not
apply. The `Wws_Job` policy module is passed at run time
(`CellSpursWorkloadAttribute.pm`): the SPURS kernel GETs guest `0x69E400`
(11,648 bytes, in the EBOOT's first segment) into LS `0xA00`. It is cut out,
wrapped with `wrap_spu_elf.py` as `spu/images/wwsjob_pm_at_0069E400.elf`
before `build_spu_workloads.py`, and `stubs.cpp` registers its code region
as image 9.

## Running

```sh
./run.sh                                   # Metal window
HEADLESS=1 ./run.sh                        # no window
tools/capture.sh title 120                 # headless, PNGs in out/title/
PAD_SCRIPT="95:0x0008,115:0x0008,130:0x4000,145:0x4000" tools/capture.sh start 200
tools/play_prologue.sh                     # title → tutorial
```

The VFS is `../infamous1/vfs` (`INFAMOUS1_VFS` picks another), a
copy-on-write clone of RPCS3's `dev_hdd0/game/NPUA80480`, so nothing the port
writes touches the emulator's install.
