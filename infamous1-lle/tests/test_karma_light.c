/* karma_light.c: the rank of a karma value, the colours, the fade, and the
 * reading of the game's memory.
 *
 *   cc -std=gnu17 -Wall -Wextra -DKARMA_LIGHT_TEST infamous1-lle/tests/test_karma_light.c \
 *      -o /tmp/test_karma_light && /tmp/test_karma_light
 *
 * Nothing of the game or the toolkit is linked: the guest memory is a zeroed
 * buffer the test writes into.
 */
#include "../karma_light.c"
#include <assert.h>
#include <sys/mman.h>

uint8_t* vm_base;
void cellPad_host_set_light(int rgb) { (void)rgb; }

static void put_f32(uint32_t ea, float v)
{
    uint32_t u; memcpy(&u, &v, 4);
    vm_base[ea] = (uint8_t)(u >> 24); vm_base[ea + 1] = (uint8_t)(u >> 16);
    vm_base[ea + 2] = (uint8_t)(u >> 8); vm_base[ea + 3] = (uint8_t)u;
}

static void test_ranks(void)
{
    /* The game's table (0x786CF0), first match, bounds included. */
    assert(kl_rank(0.0f) == KL_NEUTRAL && kl_rank(-0.0f) == KL_NEUTRAL);
    assert(kl_rank(50.0f) == KL_GUARDIAN);           /* Erik's save: "Reputation: Guardian" */
    assert(kl_rank(0.5f) == KL_GUARDIAN && kl_rank(200.0f) == KL_GUARDIAN);
    assert(kl_rank(200.5f) == KL_CHAMPION && kl_rank(500.0f) == KL_CHAMPION);
    assert(kl_rank(500.5f) == KL_HERO && kl_rank(800.0f) == KL_HERO && kl_rank(5000.0f) == KL_HERO);
    assert(kl_rank(-0.5f) == KL_THUG && kl_rank(-200.0f) == KL_OUTLAW);   /* -200 is in [-500,-200], listed first */
    assert(kl_rank(-199.5f) == KL_THUG);
    assert(kl_rank(-500.0f) == KL_INFAMOUS && kl_rank(-499.5f) == KL_OUTLAW);
    assert(kl_rank(-800.0f) == KL_INFAMOUS && kl_rank(-5000.0f) == KL_INFAMOUS);
    assert(kl_rank(NAN) == KL_NEUTRAL);
    assert(!strcmp(kl_names[kl_rank(50.0f)], "Guardian"));
}

static void test_colours(void)
{
    /* Neutral is the game's electric blue (main.cpp's default). */
    char hex[8];
    snprintf(hex, sizeof hex, "%06x", kl_colour(KL_NEUTRAL));
    assert(!strcmp(hex, KL_GAME_COLOUR));
    /* Good: blue, lighter up the ranks; evil: red, purer down them. */
    for (int r = KL_GUARDIAN; r <= KL_HERO; r++) {
        const uint32_t c = kl_colour(r), p = kl_colour(r - 1);
        assert((c & 255u) == 255u);                                   /* full blue */
        assert(((c >> 16) & 255u) > ((p >> 16) & 255u));              /* lighter */
    }
    for (int r = KL_INFAMOUS; r <= KL_THUG; r++) {
        const uint32_t c = kl_colour(r);
        assert(((c >> 16) & 255u) == 255u && (c & 255u) < 64u);       /* red */
        if (r > KL_INFAMOUS) assert(((c >> 8) & 255u) > ((kl_colour(r - 1) >> 8) & 255u));
    }
    assert(kl_colour(KL_INFAMOUS) == 0xFF0000u);
}

static void test_fade(void)
{
    const uint32_t a = 0x1E78FFu, b = 0xFF0000u;
    assert(kl_step(a, b, 0.0f) == a && kl_step(a, b, -1.0f) == a);
    assert(kl_step(a, b, 1.0f) == b && kl_step(a, b, 2.0f) == b);
    const uint32_t m = kl_step(a, b, 0.5f);
    assert(((m >> 16) & 255u) == 143u && ((m >> 8) & 255u) == 60u && (m & 255u) == 128u);
    /* Monotonic per channel. */
    uint32_t prev = a;
    for (int i = 1; i <= 20; i++) {
        const uint32_t c = kl_step(a, b, (float)i / 20.0f);
        assert(((c >> 16) & 255u) >= ((prev >> 16) & 255u) && (c & 255u) <= (prev & 255u));
        prev = c;
    }
}

static void test_read(void)
{
    vm_base = mmap(NULL, 0x01000000, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    assert(vm_base != MAP_FAILED);
    assert(kl_read() == 0.0f);                       /* nothing loaded */
    put_f32(KL_KARMA_EA, -0.0f);                     /* a new game */
    assert(kl_rank(kl_read()) == KL_NEUTRAL);
    put_f32(KL_KARMA_EA, 50.0f);
    assert(kl_read() == 50.0f && kl_rank(kl_read()) == KL_GUARDIAN);
    put_f32(KL_KARMA_EA, -650.0f);
    assert(kl_rank(kl_read()) == KL_INFAMOUS);
    put_f32(KL_KARMA_EA, NAN);
    assert(kl_read() == 0.0f);
    put_f32(KL_KARMA_EA, 1.0e9f);                    /* not a karma */
    assert(kl_read() == 0.0f);
    vm_base = NULL;
    assert(kl_read() == 0.0f);
}

int main(void)
{
    test_ranks();
    test_colours();
    test_fade();
    test_read();
    printf("test_karma_light: all passed\n");
    return 0;
}
