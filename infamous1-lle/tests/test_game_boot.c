#include "../runtime_overrides/cellGame.c"
#include <assert.h>
#include <unistd.h>
uint8_t* vm_base;
uint32_t ppu_vm_size;
int g_resv_store_active;
uint32_t g_ww_lo, g_ww_hi;
void ppu_resv_break_store(uint64_t ea) { (void)ea; }
void ps3_ww_report_inline(uint32_t a, uint64_t v, int w) { (void)a; (void)v; (void)w; }
int spu_coh_is_reserved(uint32_t a) { (void)a; return 0; }
void spu_coh_notify_write(uint32_t a) { (void)a; }
void spu_lockline_lock(void) {}
void spu_lockline_unlock(void) {}

void spu_coh_notify_write_from_ppu(uint32_t a) { (void)a; }
uint64_t spu_lockline_mask_range(uint32_t a, uint32_t n) { (void)a; (void)n; return 0; }
void spu_lockline_lock_mask(uint64_t m) { (void)m; }
void spu_lockline_unlock_mask(uint64_t m) { (void)m; }
ps3_guest_caller_fn g_ps3_guest_caller;

const char* ppu_vfs_root = ".";
int main(int argc, char** argv)
{
    assert(argc == 2);
    vm_base = calloc(1, 65536); assert(vm_base);
    cellGame_init_from_paramsfo(argv[1]);
    assert(cellGameBootCheck((void*)0x100, (void*)0x104, NULL, (void*)0x200) == CELL_OK);
    assert(vm_read32(0x100) == CELL_GAME_GAMETYPE_HDD);
    assert(strcmp((char*)vm_base + 0x200, "NPUA80480") == 0);
    assert(cellGameContentPermit((void*)0x300, (void*)0x400) == CELL_OK);
    assert(strcmp((char*)vm_base + 0x300, "/dev_hdd0/game/NPUA80480") == 0);
    assert(strcmp((char*)vm_base + 0x400, "/dev_hdd0/game/NPUA80480/USRDIR") == 0);
    free(vm_base);
    puts("PASS: digital title boots from HDD and returns its installed content paths");
}
