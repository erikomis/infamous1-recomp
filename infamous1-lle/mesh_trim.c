/*
 * Mesh trimming off at scaled resolutions (holes in dense meshes).
 *
 * The game culls its meshes on the SPUs with Edge geometry: a function of the
 * job jobmod_2e6aac472eee_ec2c0 (image 202, LS 0x8520, EBOOT 0x006158A0)
 * drops the triangles too small to cover a pixel centre of the PS3's own
 * 1280x720 grid, among others. At PS3_RES_SCALE or PS3_ASPECT above 1 the GPU
 * rasterises a finer grid, where those triangles DO cover pixel centres, and
 * nothing draws them: dense meshes get pixel-sized holes. At 1x they would
 * have covered no pixel, so the trimming is invisible there.
 *
 * RPCS3's "Disable Mesh Trimming" patch (Yahfz) writes `bi $0` (0x35000000)
 * over that function's first instruction (ai $17,$3,276; 0x1C450191), so it
 * returns at once and the triangles all go out. A patch to the module's bytes
 * never reaches the translated job, so rebuild.py lifts that instruction with
 * spu_lifter.py --insn-guard 0x8520=infamous1_edge_skip_trim:ret, and this
 * decides at run time whether it returns there (the inFAMOUS 2 port's
 * mesh_trim.c, whose patch nops a branch instead):
 *
 *   PS3_MESH_TRIM unset  trimming off while the targets are scaled
 *                        (PS3_RES_SCALE or PS3_ASPECT), on at 1x as on a PS3
 *   PS3_MESH_TRIM=1      always trim (the game's own behaviour)
 *   PS3_MESH_TRIM=0      never trim
 */
#include <stdio.h>
#include <stdlib.h>

#include "spu_context.h"

float rsx_fp_wpos_scale(void);   /* rsx_fp_decompiler.c: 1 only when no axis is scaled */

static int s_mode = -2;          /* -2 not read, -1 auto, 0 never trim, 1 always trim */

int infamous1_edge_skip_trim(spu_context* ctx)
{
    (void)ctx;
    int m = __atomic_load_n(&s_mode, __ATOMIC_RELAXED);
    if (__builtin_expect(m == -2, 0)) {
        const char* e = getenv("PS3_MESH_TRIM");
        m = (e && *e == '0') ? 0 : (e && *e == '1') ? 1 : -1;
        __atomic_store_n(&s_mode, m, __ATOMIC_RELAXED);
    }
    const int skip = m >= 0 ? m == 0 : rsx_fp_wpos_scale() != 1.0f;
    static int said = -1;   /* the decision, once per change */
    if (__builtin_expect(__atomic_load_n(&said, __ATOMIC_RELAXED) != skip, 0) &&
        __atomic_exchange_n(&said, skip, __ATOMIC_RELAXED) != skip)
        fprintf(stderr, "[mesh trim] Edge geometry triangle trimming %s (PS3_MESH_TRIM=%s)\n",
                skip ? "off" : "on, as on a PS3",
                m < 0 ? "auto" : m ? "1" : "0");
    return skip;
}
