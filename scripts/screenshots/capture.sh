#!/usr/bin/env bash
# Capture the key screens of the real Mackey app (real Rust core, WebKit webview) under Xvfb.
# Usage: scripts/screenshots/capture.sh [name...]   (no names: all shots)
# Output: docs/images/real-*.png. See README.md. Linux only; coordinates assume the 1280x800 layout.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
trap cleanup EXIT

ONLY=("$@")
want() { [ ${#ONLY[@]} -eq 0 ] || [[ " ${ONLY[*]} " == *" $1 "* ]]; }

# Positions in the 1280x800 Explore layout (px).
CHIP_IV=(448 172)      # "C" in G major
CHIP_ii=(326 172)      # "Am" in G major
TONIC=(375 34)
MODE=(517 34)
TUNING=(480 78)
NAV_SETTINGS=(56 141)
NAV_TUNER=(56 105)
NAV_EXPLORE=(56 69)

start_display

fresh() { quit_app; launch "${1:-1280}" "${2:-800}"; }

want real-explore-g-major && {
  fresh; shot real-explore-g-major
}

want real-explore-c-voicing && {
  fresh; click "${CHIP_IV[@]}"; keys v; shot real-explore-c-voicing
}

want real-explore-c-triad && {
  # C major triad on strings 2 3 4, first inversion (E on the D string, G, C).
  fresh; click "${CHIP_IV[@]}"; keys v v s i; shot real-explore-c-triad
}

want real-explore-am-arpeggio && {
  # Am arpeggio, position 5 of 21 as the app numbers them.
  fresh; click "${CHIP_ii[@]}"; keys v v v p p p p; shot real-explore-am-arpeggio
}

want real-explore-a-minor-pentatonic && {
  # A minor, scale overlay on, A minor pentatonic, first of its five boxes.
  fresh
  pick "${TONIC[@]}" A
  pick "${MODE[@]}" minor
  click 1100 380; keys o
  pick 619 323 "A minor pentatonic"
  click 1176 323
  shot real-explore-a-minor-pentatonic
}

want real-explore-left-handed && {
  fresh; click "${NAV_SETTINGS[@]}"; click 259 152; click "${NAV_EXPLORE[@]}"
  shot real-explore-left-handed
}

want real-explore-drop-d && {
  fresh; pick "${TUNING[@]}" Drop
  click 400 266; xdotool type "D"; xdotool key Return; sleep 1
  click 1100 380   # leave the text field so the shortcuts work again
  keys v
  shot real-explore-drop-d
}

want real-settings && {
  fresh; click "${NAV_SETTINGS[@]}"; resize 1280 1180; shot real-settings
}

want real-explore-ipad && {
  fresh 1180 820; click "${CHIP_IV[@]}"; keys v; shot real-explore-ipad
}

want real-tuner && {
  # Last: the tuner starts the audio input, which on a machine with no microphone is a null
  # device that delivers silence flat out and keeps the CPU busy.
  fresh; click "${NAV_TUNER[@]}"; click 280 185; sleep 3; shot real-tuner
}
