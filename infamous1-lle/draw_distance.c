/*
 * PS3_DRAW_DISTANCE=<factor>: objects are drawn that much farther away.
 *
 * inFAMOUS decides per scene-tree node whether it is near enough to draw in
 * an SPU job named "visprog" (job module 50f371dc382d, EBOOT 0x636300; a PPU
 * copy of the same interpreter at func_00339AB0). A node does not carry its
 * own distance: a byte in it picks one of eight "max render distance"
 * classes from a table in the EBOOT,
 *
 *   0x0078AE90  f32[8]  0, 400, 750, 1500, 3500, 6000, 12000, 1e6
 *
 * and the job draws the node while |node - camera|^2 (height counted at 0.6)
 * is under (class distance x the camera's LOD scale)^2. func_00252130 builds
 * the job's parameters from the table each frame (entries 1..7, times
 * camera +0x2A8, whose 1.0 the level scripts' LodAdjust objects multiply);
 * func_004B6EAC, func_004B80F8, func_00115890, func_00112A90 and
 * func_000C9A48 read it the same way on the PPU. Nothing writes it.
 *
 * So the knob scales the table's classes 1-6 once, after the ELF is in
 * memory and before the world loads (entry 0 stays 0, and 1e6 -- "always" --
 * stays). Things that stream in or swap to impostors on their own schedule
 * are not affected; this is the distance at which a loaded object is drawn.
 *
 *   PS3_DRAW_DISTANCE=<1-4>  the factor (1 = the PS3's distances); unset: 1
 *   PS3_DRAW_DISTANCE_PCT=<100-400> in PS3_TOGGLE_FILE: the same, live
 *
 * dd_scaled is pure, tested in tests/test_draw_distance.c.
 */
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern uint8_t* vm_base;

#define DD_TABLE_EA   0x0078AE90u
#define DD_CLASSES    8

/* The PS3's table, to check that the EBOOT is the one this was read from. */
static const float dd_stock[DD_CLASSES] = { 0.0f, 400.0f, 750.0f, 1500.0f, 3500.0f, 6000.0f, 12000.0f, 1000000.0f };

/* The factor from the environment: 1 when unset or not a number, clamped
 * to [1, 4] (a nearer draw distance than the PS3's is not the point, and
 * past 4 the SPU job's work grows with the square of it). */
static float dd_factor(const char* e)
{
    if (!e || !*e) return 1.0f;
    char* end = NULL;
    const float f = strtof(e, &end);
    if (end == e || !(f == f)) return 1.0f;
    return f < 1.0f ? 1.0f : f > 4.0f ? 4.0f : f;
}

/* Class i of the table at `factor`: 0 and the last ("always") unchanged. */
static float dd_scaled(int i, float factor)
{
    if (i <= 0 || i >= DD_CLASSES - 1) return dd_stock[i < 0 ? 0 : i >= DD_CLASSES ? DD_CLASSES - 1 : i];
    return dd_stock[i] * factor;
}

static float dd_r(uint32_t ea)
{
    const uint8_t* p = vm_base + ea;
    const uint32_t u = ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3];
    float f; memcpy(&f, &u, 4); return f;
}
static void dd_w(uint32_t ea, float f)
{
    uint32_t u; memcpy(&u, &f, 4);
    uint8_t* p = vm_base + ea;
    p[0] = (uint8_t)(u >> 24); p[1] = (uint8_t)(u >> 16); p[2] = (uint8_t)(u >> 8); p[3] = (uint8_t)u;
}

/* Write the table for `factor`. The first write checks that the table is the
 * PS3's (this EBOOT's); later ones rewrite it from the stock values. 1 if
 * written, 0 if the table is not the expected one. */
static int dd_checked;
static int dd_apply(float factor)
{
    if (!vm_base) return 0;
    if (!dd_checked) {
        for (int i = 0; i < DD_CLASSES; i++)
            if (dd_r(DD_TABLE_EA + 4u * (uint32_t)i) != dd_stock[i]) return 0;
        dd_checked = 1;
    }
    for (int i = 0; i < DD_CLASSES; i++)
        dd_w(DD_TABLE_EA + 4u * (uint32_t)i, dd_scaled(i, factor));
    return 1;
}

#ifndef DRAW_DISTANCE_TEST
#include "ps3emu/ps3_toggles.h"

/* Once per presented frame (main.cpp). The first call applies
 * PS3_DRAW_DISTANCE (the ELF is loaded before the first frame; the world is
 * not built yet); then, about twice a second, PS3_DRAW_DISTANCE_PCT from
 * PS3_TOGGLE_FILE (100 = the PS3's) changes it live, for A/B on one scene. */
void infamous1_draw_distance_frame(void)
{
    static int state;      /* 0 first frame, 1 running, -1 table not found */
    static float current = 1.0f;
    static unsigned frames;
    if (state < 0) return;
    float want = current;
    if (state == 0) {
        state = 1;
        want = dd_factor(getenv("PS3_DRAW_DISTANCE"));
        if (want == 1.0f) return;
    } else {
        if (++frames % 30) return;
        const int pct = ps3_toggle_get("PS3_DRAW_DISTANCE_PCT", -1);
        if (pct < 0) return;
        char buf[16];
        snprintf(buf, sizeof buf, "%g", pct / 100.0);
        want = dd_factor(buf);
        if (want == current) return;
    }
    if (dd_apply(want)) {
        current = want;
        fprintf(stderr, "[draw distance] x%.2f: object distances %.0f .. %.0f\n",
                (double)want, (double)dd_scaled(1, want), (double)dd_scaled(DD_CLASSES - 2, want));
    } else {
        state = -1;
        fprintf(stderr, "[draw distance] not applied: the distance table at %08X is not the expected one\n",
                DD_TABLE_EA);
    }
}
#endif
