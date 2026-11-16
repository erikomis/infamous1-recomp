/*
 * The DualSense / DualShock 4 light bar follows Cole's karma.
 *
 * inFAMOUS colours Cole's lightning by his karma: blue on the good side,
 * red on the evil one, both deeper with each rank. A PS3 pad has no light
 * bar; here the port reads the karma from guest memory and hands the colour
 * to the toolkit (cellPad_host_set_light), which uses it in place of the
 * fixed PS3_PAD_LIGHT colour -- the glow towards red with the game's hits
 * (PS3_PAD_LIGHT_HIT) still goes on top of it.
 *
 * Where karma lives (notes/karma/README.md has how it was found):
 *
 *   0x007D6110  the progress object, in the constant pools of 32 functions;
 *               its +0x10 is the save game's STATE image (the autosave
 *               writes it out, a load reads it in)
 *     +0x18     f32   karma: -0.0 in a new game, 50.0 in Erik's save
 *                     "Reputation: Guardian"; 34 game functions read it
 *
 * The ranks are the game's own (func_0010B1D0, the table at 0x786CF0): 0 is
 * neutral; otherwise the first of [-800,-500] Infamous, [-500,-200] Outlaw,
 * [-200,0] Thug, [0,200] Guardian, [200,500] Champion, [500,800] Hero that
 * holds it, bounds included.
 *
 *   PS3_KARMA_LIGHT=0   off: the light bar keeps PS3_PAD_LIGHT's colour
 *   PS3_KARMA_LOG=1     print the karma and rank as they change
 *
 * Only when the light bar wears the game's colour (PS3_PAD_LIGHT unset or
 * main.cpp's electric blue, KL_GAME_COLOUR); a colour the player picked, or
 * "off", stays. kl_rank/kl_colour/kl_step are pure, tested in
 * tests/test_karma_light.c.
 */
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

extern uint8_t* vm_base;
/* Toolkit, libs/input/cellPad.c: the light bar's colour (0xRRGGBB), -1 = the
 * PS3_PAD_LIGHT colour again. */
void cellPad_host_set_light(int rgb);

#define KL_KARMA_EA     0x007D6128u
#define KL_GAME_COLOUR  "1e78ff"      /* main.cpp's default: Cole's electric blue */

enum { KL_INFAMOUS, KL_OUTLAW, KL_THUG, KL_NEUTRAL, KL_GUARDIAN, KL_CHAMPION, KL_HERO, KL_RANKS };
static const char* const kl_names[KL_RANKS] = {
    "Infamous", "Outlaw", "Thug", "neutral", "Guardian", "Champion", "Hero" };

/* The game's rank of a karma value (func_0010B1D0). Outside every range the
 * game answers 6; karma never leaves [-800, 800] in play, so here the ends
 * are the last ranks of their side instead. */
static int kl_rank(float k)
{
    static const float lo[6] = { -800.0f, -500.0f, -200.0f, 0.0f, 0.0f, 200.0f };
    static const float hi[6] = { -500.0f, -200.0f, 0.0f, 0.0f, 200.0f, 500.0f };
    if (!(k == k)) return KL_NEUTRAL;
    if (k == 0.0f) return KL_NEUTRAL;
    for (int i = 0; i < 6; i++)
        if (k >= lo[i] && k <= hi[i]) return i;
    return k < 0.0f ? KL_INFAMOUS : KL_HERO;
}

/* The colour of a rank: the game's electric blue at neutral, lighter blues
 * up the good side, orange-red to pure red down the evil one. */
static uint32_t kl_colour(int rank)
{
    static const uint32_t c[KL_RANKS] = {
        0xFF0000u,   /* Infamous */
        0xFF2A0Au,   /* Outlaw   */
        0xFF5A1Eu,   /* Thug     */
        0x1E78FFu,   /* neutral: KL_GAME_COLOUR */
        0x2A8CFFu,   /* Guardian */
        0x46AAFFu,   /* Champion */
        0x78C8FFu,   /* Hero     */
    };
    return c[rank < 0 ? 0 : rank >= KL_RANKS ? KL_RANKS - 1 : rank];
}

/* One step of the fade from `from` towards `to` (0xRRGGBB each), `a` of the
 * way (0..1): a rank change shades over instead of jumping. */
static uint32_t kl_step(uint32_t from, uint32_t to, float a)
{
    if (a >= 1.0f) return to;
    if (a <= 0.0f) return from;
    uint32_t out = 0;
    for (int s = 0; s < 24; s += 8) {
        const float f = (float)((from >> s) & 255u), t = (float)((to >> s) & 255u);
        const int v = (int)lroundf(f + (t - f) * a);
        out |= (uint32_t)(v < 0 ? 0 : v > 255 ? 255 : v) << s;
    }
    return out;
}

/* The karma in guest memory (big-endian f32); 0 while the image is not there
 * or the word is not a karma (not finite or far outside the ranks). */
static float kl_read(void)
{
    if (!vm_base) return 0.0f;
    const uint8_t* p = vm_base + KL_KARMA_EA;
    const uint32_t u = ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3];
    float k;
    memcpy(&k, &u, 4);
    return (isfinite(k) && fabsf(k) <= 2000.0f) ? k : 0.0f;
}

#ifndef KARMA_LIGHT_TEST
static double kl_now(void)
{
    struct timespec ts;
    timespec_get(&ts, TIME_UTC);   /* C11: also MSVC's */
    return (double)ts.tv_sec + (double)ts.tv_nsec * 1e-9;
}

/* Once per presented frame (main.cpp). Reads the karma a few times a second
 * and fades the light bar to its rank's colour over half a second. */
void infamous1_karma_frame(void)
{
    static int state;   /* 0 not started, 1 running, -1 off */
    static int log_on, rank = -1;
    static uint32_t shown, from, target;
    static double last_read, fade_t0;
    static float karma_seen = NAN;
    if (state < 0) return;
    const double now = kl_now();
    if (state == 0) {
        const char* e = getenv("PS3_KARMA_LIGHT");
        const char* light = getenv("PS3_PAD_LIGHT");
        if ((e && e[0] == '0' && e[1] == 0) || (light && *light && strcmp(light, KL_GAME_COLOUR) != 0)) {
            state = -1;
            return;
        }
        e = getenv("PS3_KARMA_LOG");
        log_on = e && e[0] == '1';
        shown = from = target = kl_colour(KL_NEUTRAL);
        last_read = fade_t0 = now;
        cellPad_host_set_light((int)shown);
        state = 1;
    }
    if (now - last_read >= 0.25) {
        last_read = now;
        const float k = kl_read();
        const int r = kl_rank(k);
        if (log_on && !(k == karma_seen))
            fprintf(stderr, "[karma] %.1f (%s)\n", (double)k, kl_names[r]);
        karma_seen = k;
        if (r != rank) { rank = r; from = shown; target = kl_colour(r); fade_t0 = now; }
    }
    if (shown != target) {
        const uint32_t c = kl_step(from, target, (float)((now - fade_t0) / 0.5));
        if (c != shown) { shown = c; cellPad_host_set_light((int)shown); }
    }
}
#endif
