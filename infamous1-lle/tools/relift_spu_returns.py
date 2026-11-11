#!/usr/bin/env python3
"""Relift inFAMOUS 1's audio image with resumable call-return entries.

Run with Python 3.10+ and --toolkit /path/to/ps3recomp-fork, then rebuild.
The firmware can resume at a call's return PC after an interrupt. Those PCs
must be registered even when they fall inside a lifted function.
"""
import argparse
from pathlib import Path
import re
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolkit", required=True, type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    name = "spu_0003_at_006A4E80"
    output = root / "spu" / name
    source = (output / "spu_recomp.c").read_text()
    returns = sorted({int(x, 16) for x in re.findall(
        r"spu_link\(0x([0-9A-Fa-f]+)\)", source)})
    if not returns:
        raise SystemExit("No call-return addresses found; refusing an incomplete relift")
    elf = root / "spu/images" / (name + ".elf")
    subprocess.run([
        sys.executable, str(args.toolkit / "tools/spu_lifter.py"), str(elf),
        "--auto-functions", str(elf), "--symbol-prefix", name + "_",
        "-o", str(output), "--extra-funcs", ",".join(hex(x) for x in returns),
    ], check=True)
    print(f"Registered {len(returns)} call-return entry points for {name}")


if __name__ == "__main__":
    main()
