#!/bin/sh
# Play the prologue by script, headless: tools/play_prologue.sh [name] [seconds]
#
# Boots, clears the title screen (START -- cross does not advance it, see
# NOTES.md 2026-11-10), confirms the menu and answers the tutorials with
# timed stick holds and button presses: the right stick in each direction
# (look up / left / right / down, two cycles so the prompt is met whenever
# it appears), the left stick in each direction, then the left stick held
# forward for "Head towards the disturbance", and L1 held with R1 taps for
# "Hold L1 to aim" at the pier gate. Frames land in out/<name>/ every 10 s,
# the log in out/<name>.log.
#
# Times are wall-clock seconds, measured on an M-series Mac with nothing
# else running (out/start-run: the first world frame and its look-up prompt
# lands between 145 s and 200 s), and every hold is generous; another load
# on the machine shifts everything later still.
here=$(cd "$(dirname "$0")/.." && pwd)
name=${1:-prologue}; secs=${2:-1200}
# lx,ly,rx,ry,from,until (0-255, 128 = centred); up to 16 holds.
PAD_STICK="128,128,128,0,196,202;128,128,0,128,202,208;128,128,255,128,208,214;128,128,128,255,214,220"
PAD_STICK="$PAD_STICK;128,128,128,0,232,238;128,128,0,128,238,244;128,128,255,128,244,250;128,128,128,255,250,256"
PAD_STICK="$PAD_STICK;128,0,128,128,264,272;0,128,128,128,272,280;255,128,128,128,280,288;128,0,128,128,296,$secs"

# sec:mask[:hold] -- START 0x0008, CROSS 0x4000, R1 0x0800, L1 0x0400; up to 32.
PAD_SCRIPT="99:0x0008,112:0x0008"                                   # title: clear it
PAD_SCRIPT="$PAD_SCRIPT,124:0x4000,138:0x4000,152:0x4000"           # menu: new game, confirm, dialog
PAD_SCRIPT="$PAD_SCRIPT,720:0x0400:400"                             # aim, held through the gate
PAD_SCRIPT="$PAD_SCRIPT,730:0x0800,750:0x0800,770:0x0800,790:0x0800,810:0x0800,830:0x0800"   # bolts
PAD_SCRIPT="$PAD_SCRIPT,860:0x0800,890:0x0800,920:0x0800,950:0x0800,980:0x0800,1010:0x0800"
PAD_SCRIPT="$PAD_SCRIPT,1040:0x0800,1070:0x0800,1100:0x0800,1130:0x0800"

PAD_STICK="$PAD_STICK" PAD_SCRIPT="$PAD_SCRIPT" exec "$here/tools/capture.sh" "$name" "$secs"
