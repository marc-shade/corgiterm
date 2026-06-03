#!/usr/bin/env bash
# Terminal rendering torture test for CorgiTerm.
#
# Exercises every failure mode that the hand-rolled pango renderer garbled:
# fixed-grid alignment, box drawing, wide (CJK/emoji) characters, combining
# marks / ZWJ sequences, ANSI 16 / 256 / truecolor, and text attributes.
# Run this INSIDE corgiterm and visually confirm: the ruler columns line up,
# the box stays square, wide chars do not overrun the following ASCII, and
# colors/attributes render correctly.
#
# Usage:  bash scripts/terminal-torture.sh

esc=$'\033'

echo "== ASCII alignment ruler (columns must line up) =="
for _ in 1 2 3; do
  echo "0123456789012345678901234567890123456789012345678901234567890123456789"
done
echo

echo "== Box drawing (must stay square) =="
echo "┌──────────┬──────────┐"
echo "│ left     │ right    │"
echo "├──────────┼──────────┤"
echo "│ 12345    │ abcde    │"
echo "└──────────┴──────────┘"
echo

echo "== Wide chars (CJK/emoji must not overrun the |) =="
echo "日本語テスト|END"
echo "中文测试abc|END"
echo "emoji 🐕🚀✅|END"
echo

echo "== Combining marks / ZWJ =="
printf 'e + combining acute: e\xcc\x81\n'
echo "ZWJ family: 👨‍👩‍👧‍👦   flag: 🇯🇵"
echo

echo "== ANSI 16 colors =="
for c in 0 1 2 3 4 5 6 7; do printf "${esc}[3${c}m B${c} ${esc}[0m"; done; echo
for c in 0 1 2 3 4 5 6 7; do printf "${esc}[9${c}m b${c} ${esc}[0m"; done; echo
echo

echo "== 256-color ramp =="
for i in $(seq 16 51); do printf "${esc}[48;5;${i}m ${esc}[0m"; done; echo
echo

echo "== Truecolor gradient =="
for i in $(seq 0 2 70); do r=$((i*3)); printf "${esc}[48;2;${r};80;160m ${esc}[0m"; done; echo
echo

echo "== Attributes =="
printf "${esc}[1mbold${esc}[0m  ${esc}[2mdim${esc}[0m  ${esc}[3mitalic${esc}[0m  ${esc}[4munderline${esc}[0m  ${esc}[9mstrike${esc}[0m  ${esc}[7minverse${esc}[0m\n"
echo
echo "Torture test complete. Try: vim, less, htop (alternate-screen apps)."
