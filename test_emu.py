#!/usr/bin/env python3
import subprocess
import struct
import os
import sys

WASM_PATH = "target/wasm32-unknown-unknown/release/gba_emu.wasm"

def run_wasmtime(code, args=[], stdin=""):
    full_args = ["wasmtime", "--invoke", "emu_init", WASM_PATH]
    # Actually let's use a wastrunner approach
    return ""

def compare_with_oracle(rom_path, frames, replay=None):
    # Run oracle
    cmd = ["oracle", "run", rom_path, str(frames)]
    if replay:
        cmd.extend(["--replay", replay])
    result = subprocess.run(cmd, capture_output=True, text=True)
    print("Oracle:", result.stdout.strip())
    if result.stderr:
        print("Oracle stderr:", result.stderr[:500])
    return result.returncode == 0

if __name__ == "__main__":
    rom = sys.argv[1] if len(sys.argv) > 1 else "dev-roms/anguna.gba"
    frames = int(sys.argv[2]) if len(sys.argv) > 2 else 1
    compare_with_oracle(rom, frames)
