/* draw_distance.c: the factor, the scaled classes, and the write to guest
 * memory (only over the PS3's table).
 *
 *   cc -std=gnu17 -Wall -Wextra -DDRAW_DISTANCE_TEST infamous1-lle/tests/test_draw_distance.c \
 *      -o /tmp/test_draw_distance && /tmp/test_draw_distance
 */
#include "../draw_distance.c"
#include <assert.h>
#include <sys/mman.h>

uint8_t* vm_base;

int main(void)
{
    /* The factor: unset / junk 1, clamped to [1, 4]. */
    assert(dd_factor(NULL) == 1.0f && dd_factor("") == 1.0f && dd_factor("x") == 1.0f);
    assert(dd_factor("2") == 2.0f && dd_factor("1.5") == 1.5f);
    assert(dd_factor("0.5") == 1.0f && dd_factor("9") == 4.0f && dd_factor("nan") == 1.0f);

    /* Classes: 0 and "always" unchanged, the rest scaled. */
    assert(dd_scaled(0, 2.0f) == 0.0f && dd_scaled(7, 2.0f) == 1000000.0f);
    assert(dd_scaled(1, 2.0f) == 800.0f && dd_scaled(6, 2.0f) == 24000.0f);
    for (int i = 1; i < 7; i++) assert(dd_scaled(i, 1.0f) == dd_stock[i]);

    /* Guest memory: written only over the PS3's table, only once. */
    vm_base = mmap(NULL, 0x01000000, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    assert(vm_base != MAP_FAILED);
    assert(!dd_apply(2.0f));                          /* zeros: not the table, nothing written */
    assert(dd_r(DD_TABLE_EA + 4) == 0.0f);
    for (int i = 0; i < 8; i++) dd_w(DD_TABLE_EA + 4u * (uint32_t)i, dd_stock[i]);
    assert(dd_apply(2.0f));
    assert(dd_r(DD_TABLE_EA) == 0.0f && dd_r(DD_TABLE_EA + 4) == 800.0f && dd_r(DD_TABLE_EA + 24) == 24000.0f);
    assert(dd_r(DD_TABLE_EA + 28) == 1000000.0f);
    assert(dd_apply(3.0f));                           /* live: from the stock values, not compounded */
    assert(dd_r(DD_TABLE_EA + 4) == 1200.0f && dd_r(DD_TABLE_EA + 24) == 36000.0f);
    assert(dd_apply(1.0f));                           /* and back to the PS3's */
    assert(dd_r(DD_TABLE_EA + 4) == 400.0f && dd_r(DD_TABLE_EA + 28) == 1000000.0f);
    /* The words around the table (the job's shuffle constants) untouched. */
    assert(vm_base[DD_TABLE_EA - 1] == 0 && vm_base[DD_TABLE_EA + 32] == 0);
    printf("test_draw_distance: all passed\n");
    return 0;
}
