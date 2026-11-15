#!/usr/bin/env bash
# Helpers for capture.sh: start the real app under Xvfb, drive it with xdotool, grab the screen.
# Linux only. Needs Xvfb, xdotool and ImageMagick's `import` (apt install xvfb xdotool imagemagick).

DISPLAY_NUM="${DISPLAY_NUM:-99}"
export DISPLAY=":${DISPLAY_NUM}"
# Software rendering: there is no GPU under Xvfb.
export WEBKIT_DISABLE_COMPOSITING_MODE=1 LIBGL_ALWAYS_SOFTWARE=1

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP="${APP:-$REPO/target/debug/mackey-app}"
OUT="${OUT:-$REPO/docs/images}"
# The screen is taller than any shot so the Settings page can be captured whole; shot() crops.
SCREEN_W=1400
SCREEN_H=1400
WIN_W=1280
WIN_H=800
WX=0
WY=0

XVFB_PID=""
WM_PID=""
APP_PID=""
WID=""
SCRATCH="$(mktemp -d)"

start_display() {
  if xdpyinfo -display "$DISPLAY" >/dev/null 2>&1; then
    echo "display $DISPLAY is already in use (a stale Xvfb?); set DISPLAY_NUM to a free number" >&2
    exit 1
  fi
  Xvfb "$DISPLAY" -screen 0 "${SCREEN_W}x${SCREEN_H}x24" >/dev/null 2>&1 &
  XVFB_PID=$!
  sleep 1
  # Without a window manager a resize does not reach the webview, so use a bare one if present
  # (apt install openbox), configured without decorations so the window is its client area.
  if command -v openbox >/dev/null; then
    mkdir -p "$SCRATCH/wm/openbox"
    cp "$(dirname "${BASH_SOURCE[0]}")/openbox-rc.xml" "$SCRATCH/wm/openbox/rc.xml"
    XDG_CONFIG_HOME="$SCRATCH/wm" openbox >/dev/null 2>&1 &
    WM_PID=$!
    sleep 1
  fi
}

# launch W H: a fresh app with empty settings (its own HOME), window moved to 0,0 and sized W x H.
launch() {
  local w="${1:-1280}" h="${2:-800}" home
  home="$(mktemp -d "$SCRATCH/home.XXXXXX")"
  HOME="$home" XDG_CONFIG_HOME="$home/config" XDG_DATA_HOME="$home/data" \
    "$APP" >"$SCRATCH/app.log" 2>&1 &
  APP_PID=$!
  for _ in $(seq 1 40); do
    WID="$(xdotool search --name Mackey 2>/dev/null | tail -1 || true)"
    [ -n "$WID" ] && break
    sleep 0.5
  done
  [ -n "$WID" ] || { echo "the app window did not appear" >&2; cat "$SCRATCH/app.log" >&2; exit 1; }
  sleep 2
  resize "$w" "$h"
}

# resize W H: set the window size (and what shot() crops to).
resize() {
  WIN_W="$1"
  WIN_H="$2"
  xdotool windowsize "$WID" "$WIN_W" "$WIN_H"
  sleep 2
  xdotool windowmove "$WID" 0 0
  sleep 1
  # Where the client area really is (a window manager may offset it by its frame).
  eval "$(xdotool getwindowgeometry --shell "$WID")"
  WX="$X"
  WY="$Y"
  echo "window $WIN_W x $WIN_H at $WX,$WY (geometry $WIDTH x $HEIGHT)"
}

quit_app() {
  [ -n "$APP_PID" ] && kill "$APP_PID" 2>/dev/null || true
  wait "$APP_PID" 2>/dev/null || true
  APP_PID=""
}

cleanup() {
  quit_app
  [ -n "$WM_PID" ] && kill "$WM_PID" 2>/dev/null || true
  [ -n "$XVFB_PID" ] && kill "$XVFB_PID" 2>/dev/null || true
  rm -rf "$SCRATCH"
}

# click X Y
click() {
  xdotool mousemove "$((WX + $1))" "$((WY + $2))"
  sleep 0.3
  xdotool click 1
  sleep 1
}

# key K...: press keys one at a time so each redraw settles.
keys() {
  for k in "$@"; do
    xdotool key "$k"
    sleep 0.8
  done
}

# pick X Y TEXT: open the drop-down at X,Y and choose the entry that starts with TEXT.
pick() {
  click "$1" "$2"
  xdotool type --delay 80 "$3"
  sleep 0.3
  xdotool key Return
  sleep 1
}

# shot NAME: capture the window area (the window sits at 0,0) to $OUT/NAME.png (the pointer is parked out of the way).
shot() {
  xdotool mousemove "$((SCREEN_W - 5))" "$((SCREEN_H - 5))"
  sleep 0.5
  import -window root -crop "${WIN_W}x${WIN_H}+${WX}+${WY}" +repage "$OUT/$1.png"
  echo "wrote $OUT/$1.png"
}
