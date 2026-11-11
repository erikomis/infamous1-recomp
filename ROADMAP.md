# Roadmap

🌐 **English** · [Português (BR)](ROADMAP.pt-BR.md)

Status of the native inFAMOUS 1 (`NPUA80480`) port to macOS/arm64 and what
comes next. `[x]` = done and verified; `[ ]` = to do. Evidence and logs are in
[NOTES.md](NOTES.md); the features brought from the inFAMOUS 2 port are in
[PORT_FEATURES.md](PORT_FEATURES.md).

## Status at a glance (2026-11-10)

- **Runs:** boot, studio logos, Bink videos, the 3D title screen, START, the opening explosion and Cole in the prologue tutorial ("Use the right stick to look up at the chopper"). Played headless with a scripted pad (`tools/play_prologue.sh`); the launcher also starts the game in a Metal window.
- **Speed:** ~44 fps in the menu and cutscenes, headless, on Apple silicon (the console aimed at 30). Gameplay fps is not measured yet.
- **App:** the macOS bundle builds, 72 tests pass (3 optional ones skipped), and its Play button starts the game (`BootCheck: type=2, titleId=NPUA80480`). The install/rebuild wizard and the self-update are disabled.
- **Release:** none yet.
- **What is left for a "1.0 playable on Mac":** play the whole story (nobody has gone past the tutorial) and fix what that turns up, move the lighting and SSAO jobs to the GPU for speed, and give the app a rebuild wizard and a release.
- **Not yet:** Windows and Linux.

## History

*Kept as a record of how the port got here.*

- **2026-11-06/08, bring-up:** the LLE boot (firmware, SPURS, `Wws_Job` policy module) on the inFAMOUS 1 fork, modelled on the inFAMOUS 2 port. The boot stalled in `boot4`/`boot5`, with `sys_memory allocate(0x100000)` returning forever (26,865 tries, `ENOMEM` after 125 allocations up to `0x4FF00000`).
- **2026-11-09, the title screen:** the memory window fix (`PS3_MEM_ALLOC_SIZE`) and the HDD boot (`cellGame` `CATEGORY`) got the recomp to the 3D title screen "Press the Start Button", and past the explosion to Cole in the tutorial.
- **2026-11-10, the app and the feature comparison:** the inFAMOUS 2 launcher adapted (own title, paths, ids and settings), anisotropic 16× checked, the lighting-job candidate found, and this roadmap.

---

## What is missing to be 100% playable

### Repository (quick wins, 2026-11-11)

- [x] The toolkit patch in this repository: `tools/setup_toolkit.sh` and `rebuild.py` looked for `ps3recomp-fork-770660a-with-upstream-prs.patch`, which only existed in the inFAMOUS 2 tree, so a clean checkout could not build. Now in `infamous1-lle/patches/`, made from the toolkit checkout the port builds against: the inFAMOUS 2 patch plus the `PS3_MEM_ALLOC_SIZE` fix. Checked: `setup_toolkit.sh` applies it to a clean clone of `770660a`.
- [x] Screenshots of the port in `docs/screenshots/` and a gallery at the top of the README, as in the inFAMOUS 2 and Resistance repositories (headless frames from `out/start-run`).
- [x] CI for the launcher (macOS, Windows, Linux) and the Python/shell tools (`.github/workflows/`), plus Dependabot for the action pins. Every step was run locally first: the tools' self-tests, the staging, `rebuild.py --check-json`, `npm run typecheck/build`, `cargo test --locked` (72 passed).
- [x] The launcher's UI mock (`launcher/scripts/ui-mock.js`) showed inFAMOUS 2 names, paths and saves.
- [x] `rebuild.py` did not ship `runtime_overrides/cellGame.c` (the HDD boot fix) with the port sources, so a rebuild would have lost it; the staging check caught it. Also `infamous1-lle/README.md`, which the rebuild ships, was missing.

### Blockers

- [x] Boot past the memory window (the allocator ended 256 MB after its base; now `PS3_MEM_ALLOC_SIZE`, 768 MB in `run.sh`). Evidence: `out/boot/boot7.log`, zero `null-write`, 66 allocation calls in total, the whole flow to the Bink videos and "Game State Thread".
- [x] Title screen → gameplay: START (`0x0008`, not ✕) leaves the title; the explosion and the tutorial render (`out/start-run/`).
- [ ] **Play the whole story.** Nobody has gone past the tutorial. `tools/play_prologue.sh` records the first minutes; later missions need a windowed session with a controller.
- [ ] Long-session stability: memory, streaming, freezes and stalls over hours.
- [ ] The full campaign's save and load, checked in a window (the saves manager recognises `BCUS98119_INFAMOUS_*`; no original save was touched).

### Graphics

- [x] Anisotropic filtering up to 16× (`out/aniso-smoke.log`, `out/aniso16/`).
- [ ] A/B captures of the image levers (FXAA/SMAA, `PS3_RES_SCALE`, MetalFX, sharpening) on the same scene, with `tools/capture.sh`.
- [ ] Check the rendering on later scenes (lighting, shadows, water, particles) in a window; the inFAMOUS 2 fixes were found that way.

### Stability

- [ ] Run `tools/profile_run.sh` in gameplay: fps per period and the hottest code, as a baseline before the GPU lighting work.
- [ ] A shader-cache warm-up, so the first-use hitches happen once.

### Saves

- [x] Saves manager: list, back up, restore, zip and RPCS3 exchange, tested in temporary folders.
- [ ] Check it on a real session's save, in a window.

### Controls

- [x] Controller, keyboard and mouse mapped through the shared runtime, bindings tested.
- [ ] Every combination of devices in a window (DualSense, DualShock 4, Xbox, keyboard and mouse).
- [ ] The in-game menu (F1 / Cmd+, / L3+R3) on this game.

### Rebuild

- [ ] Port `infamous1-lle/tools/rebuild.py` to this game. It is still the inFAMOUS 2 recipe: `--code-end 0x742e18` (here `0x55EB44`), the `Wws_Job` policy module cut from the EBOOT at `0x8D5B00` (here it is passed at run time), the data and relocatable modules (none here), the extra SPU entry points and the expected EBOOT hashes of inFAMOUS 2 1.03 (here APP_VER 01.00, SHA-256 `5e22402a…`). The manual recipe is in [`infamous1-lle/README.md`](infamous1-lle/README.md#rebuilding).
- [ ] An install/rebuild wizard in the app, on top of the ported `rebuild.py`. The inherited one is disabled; today the build is the local CMake in `infamous1-lle`.
- [ ] Look for the relocatable job lifts of inFAMOUS 2 in this EBOOT in other formats: the scanner found zero blobs with the expected signature (that does not rule out other formats).
- [ ] A `rebuild.py` run end to end from a clean checkout.

---

## After it is playable (improvements)

### Speed: lighting and SSAO on the GPU

In inFAMOUS 2 this took the New Marais walk from 12 to 15 fps and from 21 to 28 fps. Here it needs the work specific to this game, because the hooks of the second game depend on other jobs, local-store layouts, buffers and synchronisation.

1. [ ] Identify in a gameplay trace the jobs that produce lighting and SSAO.
2. [ ] Map each job's parameters, input/output buffers and completion signals.
3. [ ] Confirm the lighting job. The leading candidate is image **209** = `jobmod_6cbbde2f1bfc_e9d68` (entry `0xDD68`, 43,616 B, source streamed from `0x61E680` to LS `0x4000`), the most streamed overlay in boot and cutscenes (13×), the same profile as inFAMOUS 2's lighting job. Alternatives: image 215 (`jobmod_cbe85abb417c_e12058`, 6×) and 218 (`jobmod_d46637430ef9_e5bb8`, 6×). The test: stub the candidate's per-tile routine with `spu_replace_function` and look at the visual effect; in inFAMOUS 2 a broken lighting job meant glossy props blown out to white.
4. [ ] Compare the formulas with the Metal shaders of inFAMOUS 2 and adapt what differs.
5. [ ] Compare SPU against GPU on the same scene before turning the shortcuts on.
6. [ ] Port `gpu_light.c` and `gpu_ssao.c` (placeholders for now: the inFAMOUS 2 ones reference that game's `jobmod_*` symbols).

### Ultrawide and the HUD

- [ ] Map the cameras and the final composite shader. The existing camera patch is limited to the inFAMOUS 2 EBOOT hash, so the option is hidden here.

### The app

- [ ] Install/rebuild wizard (see Rebuild).
- [ ] Self-update with its own endpoint and key; the inFAMOUS 2 ones are not active.
- [ ] A signed release `.dmg`.

### Other platforms

- [ ] Windows and Linux.

### Out of scope for now

- Secondary missions that are repetitive: game design, not a technical defect.

---

## Problems of the original game

Raised by Erik (2026-11-10), with the technical plan for each item. Image and speed problems the port attacks by nature (runtime levers); game-logic problems need surgical patches in the lift (`game_patches.cpp` / runtime overrides), each a reverse-engineering job of its own.

### High priority

| Problem | What happened in the original | Plan in the port | State |
|---|---|---|---|
| FPS drops | Dips during explosions, fights and busy areas | 1) The recomp already runs ~44 fps (the console aimed at 30). 2) Move the lighting/SSAO job to the GPU (above). 3) The shader cache avoids translation hitches | 🟡 partial (2 and 3 pending) |
| Aliasing | Jagged edges on buildings, wires and objects | `PS3_AA=fxaa/smaa` + `PS3_UPSCALE=metalfx` + `PS3_RES_SCALE=2` (1440p), already in the launcher; validate the visual A/B | 🟡 available, validation pending |
| Climbing | Cole grabbed unwanted objects or missed the right point | A logic patch in the lift: map the raycast/grab (`game_patches.cpp`); may need tuning by constant | ⚪ not started (heavy) |
| Pop-in | Objects and detail appeared as the player got close | Streaming already runs from a local SSD (faster than the PS3). Draw distance is a game constant: changing it means finding the LOD/streaming constants in the EBOOT | 🟡 partial (streaming ok, LOD not) |

### Medium priority

| Problem | What happened | Plan in the port | State |
|---|---|---|---|
| Camera | Bad framing in tight spaces and while climbing | Map the camera framing/collision functions in the lift and adjust; note that `PS3_ASPECT=21:9` is separate and also pending | ⚪ not started |
| Combat | Enemies with high accuracy attacking from far away | A logic patch: find the accuracy/range constants in the game code | ⚪ not started |
| Enemy AI | Repetitive behaviour, unnatural reactions | Same: each behaviour is a change point; high cost, subjective gain | ⚪ not started |
| Checkpoints | Stretches that forced frustrating repeats | Find the respawn/save points in the game code and loosen the spacing | ⚪ not started |
| Textures | Surfaces with low definition and sharpness | **Validated 2026-11-10**: aniso 16×; plus `PS3_RES_SCALE`, `PS3_SHARPEN` and MetalFX in the launcher; texture mods (PNG packs) also available | 🟢 validated |

### Order of attack

1. **Validate the image presets** (AA + `RES_SCALE` 1440p): capture A/B, immediate, zero risk (no game code touched).
2. **`profile_run.sh`**: fps per period and the hottest code in gameplay, a baseline before the GPU lighting.
3. **GPU lighting/SSAO**: confirm image 209 by A/B, map the LS layout/labels, port `gpu_light.c`.
4. **Pop-in/LOD**: hunt for the streaming constants in the EBOOT.
5. **Camera / climbing / combat / AI / checkpoints**: each a reverse-engineering project of its own, by order of annoyance.

### Technical notes

- Logic patches follow the pattern of `game_patches.cpp`: filtered by the program-header hash, never apply an inFAMOUS 2 patch to this EBOOT.
- Every image validation is a capture A/B (`tools/capture.sh`) on the same scene, with the machine idle (`tools/profile_run.sh` serialises itself).
- Before touching GPU lighting, photograph the baseline with the original SPU job.

---

## Done and finished

### Boot and pipeline

- [x] LLE boot with the firmware (liblv2, libsysmodule, libsre) on the inFAMOUS 1 fork.
- [x] Extractor for the job modules embedded in the EBOOT (`extract_eboot_jobmods.py`), their lifting (`lift_spu_jobs.sh`, `lift_datamods.py`, `lift_relocmods.py`) and the SPU returns (`relift_spu_returns.py`).
- [x] HDD boot: `cellGame` reads `CATEGORY` from `PARAM.SFO`; `run.sh` passes `PS3_BOOT_PATH`. Isolated test `tests/test_game_boot.c`.
- [x] The memory window (`PS3_MEM_ALLOC_SIZE`).
- [x] The recursive clean-up found with the file-system trace (`func_00105550`).

### From the menu to gameplay

- [x] Studio logos, Bink videos (`splash_america.bik`, `tv_default_feed_2x2.bik`), music and streaming.
- [x] 3D title screen, START, the explosion, Cole in the tutorial.
- [x] `tools/play_prologue.sh` re-anchored on the evidence of `out/start-run` (the world starts at 145-200 s).

### Graphics and performance

- [x] Metal window at 1280×720, 32 shader pipelines compiled at boot.
- [x] Anisotropic filtering up to 16×.
- [x] ~44 fps in the menu and cutscenes, headless.

### App and tools

- [x] Tauri launcher adapted (title, paths, ids, settings, English/Portuguese), 72 tests pass, bundle built.
- [x] Install detection by `PARAM.SFO` (`NPUA80480`); saves `BCUS98119` accepted, `BCUS98125` rejected.
- [x] `tools/capture.sh` falls back to the toolkit's venv or `python3`.
