#!/usr/bin/env python3
"""Compile and run the real Metal AA regression on macOS."""
from pathlib import Path
import platform
import subprocess
import tempfile


def main():
    if platform.system() != "Darwin":
        print("SKIP: Metal requires macOS")
        return
    sdk = Path(__file__).resolve().parents[2] / "ps3recomp-fork"
    with tempfile.TemporaryDirectory(prefix="infamous-hybrid-aa-") as tmp:
        binary = Path(tmp) / "test"
        subprocess.run([
            "clang", "-fobjc-arc", "-Iinclude", "-Ilibs/video",
            "libs/video/tests/test_metal_hybrid_aa.m",
            "libs/video/rsx_texture_layout.c", "-Wl,-dead_strip",
            "-framework", "Metal", "-framework", "Cocoa",
            "-framework", "QuartzCore", "-o", str(binary),
        ], cwd=sdk, check=True, timeout=120)
        subprocess.run([str(binary)], check=True, timeout=30)


if __name__ == "__main__":
    main()
