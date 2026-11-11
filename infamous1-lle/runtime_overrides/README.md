# Local runtime fixes

`cellGame.c` is based on the local inFAMOUS 2 `ps3recomp-fork` checkout
(base commit `770660a3`, including its existing local changes). The executable
compiles this copy so the shared toolkit used by other ports stays unchanged.

The additional fix reads PARAM.SFO `CATEGORY`: `HG` selects HDD boot and `DG`
selects disc boot. The same mapping exists in the Afro Samurai toolkit.
NPUA80480 is `HG`; `run.sh` supplies the matching `PS3_BOOT_PATH`.

The old runtime always reported disc boot. During bring-up the game repeatedly
enumerated `USRDIR/sounds/music` while its cleanup requests hit no-op unlink
handlers, eventually exhausting the guest allocation window. Do not implement
deletion of the installed assets as a workaround.

`tests/test_game_boot.c` verifies the boot type and content paths using the
player's PARAM.SFO. It includes this implementation and supplies single-threaded
memory/coherence mocks; it does not execute the game or test rendering.
