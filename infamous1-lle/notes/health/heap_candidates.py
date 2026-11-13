"""Objects in a guest-heap dump that could be the hero, and who points at them.

    heap_candidates.py <folder>    (heap-215.bin, heap-225.bin: 0x40000000+256 MB;
                                    static-215.bin: 0+16 MB; PS3_MEMDUMP_AT)

An object here: an aligned word that is a vtable in the EBOOT's data
(0x720000-0x9EEEE0), f32 at +0x750 equal to +0x754 and at least 10, the
same in both dumps; listed with the static and heap words pointing at it,
most-referenced first. README.md, step 2.
"""
import struct, sys, collections, array
D = sys.argv[1]; H0 = 0x40000000
h = open(f'{D}/heap-215.bin','rb').read(); h2 = open(f'{D}/heap-225.bin','rb').read()
s = open(f'{D}/static-215.bin','rb').read()
def u32(b, o): return struct.unpack_from('>I', b, o)[0]
def f32(b, o): return struct.unpack_from('>f', b, o)[0]
H = array.array('I', h); H.byteswap(); S = array.array('I', s); S.byteswap()
cands = {}
for i in range(0, len(H) - 0x1D6):
    v = H[i]
    if not (0x720000 <= v < 0x9EEEE0): continue
    o = i * 4
    a = f32(h, o + 0x750)
    if not (10.0 <= a <= 100000.0) or f32(h, o + 0x754) != a: continue
    if u32(h2, o) != v or f32(h2, o + 0x750) != a: continue
    cands[H0 + o] = (v, a)
print(len(cands), 'candidates with value >= 10')
refs = collections.defaultdict(list)
for i, w in enumerate(H):
    if w in cands: refs[w].append(('heap', H0 + 4 * i))
for i, w in enumerate(S):
    if w in cands: refs[w].append(('static', 4 * i))
shown = 0
for ea, (v, a) in sorted(cands.items(), key=lambda kv: -len(refs[kv[0]])):
    r = refs[ea]
    if not r: continue
    print('obj %08x vt %08x value %.1f  %d refs: %s' % (ea, v, a, len(r), ' '.join('%s:%08x' % x for x in r[:6])))
    shown += 1
    if shown > 25: break
