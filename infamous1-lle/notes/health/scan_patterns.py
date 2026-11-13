"""The hit-point code patterns of inFAMOUS 2 searched for in another EBOOT.

    scan_patterns.py <EBOOT.elf>

adjust:    lfs cur; fadds; lfs max = cur - 4 (same base); fsubs; fsel  (min(cur + delta, max))
threshold: lfs max; fmuls; lfs cur = max + 4 (same base); fcmpu        (cur vs a share of max)
On inFAMOUS 2's EBOOT it finds its known pair, +0x7EC/+0x7F0; on inFAMOUS 1's,
the threshold pattern gives +0x750/+0x754. README.md, step 1.
"""
import sys, struct, collections
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from elftool import Elf
e = Elf(sys.argv[1])
# the text: the first PT_LOAD (executable)
va, off, fsz = e.segs[0]
words = struct.unpack('>%dI' % (fsz // 4), e.d[off:off + fsz - fsz % 4])
def op(w): return w >> 26
def lfs(w):
    if op(w) != 48: return None
    d = w & 0xFFFF; d = d - 0x10000 if d & 0x8000 else d
    return ((w >> 21) & 31, (w >> 16) & 31, d)
def a_xo(w): return (w >> 1) & 31
def is_(w, o, x): return op(w) == o and a_xo(w) == x
adj = collections.Counter(); thr = collections.Counter(); where = collections.defaultdict(list)
for i in range(len(words) - 10):
    w = words[i:i + 10]
    a = lfs(w[0])
    if not a: continue
    # adjust: lfs cur; ...fadds...; lfs max=cur-4 (same base); fsubs; fsel
    for j in range(1, 4):
        b = lfs(w[j])
        if b and b[1] == a[1] and b[2] == a[2] - 4:
            rest = w[j + 1:j + 5]
            if any(is_(x, 59, 21) for x in w[1:j + 1] + rest) and any(is_(x, 59, 20) for x in rest) and any(is_(x, 63, 23) for x in rest):
                adj[(b[2], a[2])] += 1; where[('adj', b[2], a[2])].append(va + 4 * i)
    # threshold: lfs max; fmuls; lfs cur=max+4 (same base); fcmpu
    for j in range(1, 4):
        b = lfs(w[j])
        if b and b[1] == a[1] and b[2] == a[2] + 4:
            if any(is_(x, 59, 25) for x in w[1:j + 1]) and any(op(x) == 63 and ((x >> 1) & 1023) == 0 for x in w[j + 1:j + 3]):
                thr[(a[2], b[2])] += 1; where[('thr', a[2], b[2])].append(va + 4 * i)
print('adjust (max, cur):', [(hex(k[0]), hex(k[1]), n) for k, n in adj.most_common(12)])
print('threshold (max, cur):', [(hex(k[0]), hex(k[1]), n) for k, n in thr.most_common(12)])
for k, v in sorted(where.items(), key=lambda kv: -len(kv[1]))[:12]:
    print(k[0], hex(k[1]), hex(k[2]), ' '.join(hex(x) for x in v[:8]))
