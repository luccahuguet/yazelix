#!/bin/sh

if [ "${1:-}" != "--detached" ]; then
  if [ "$#" -lt 4 ]; then
    echo "Error: desktop deferred launch probe requires log path, parent pid, '--', and command argv" >&2
    exit 64
  fi

  launch_log="$1"
  : > "$launch_log"

  if command -v setsid >/dev/null 2>&1; then
    nohup setsid "$0" --detached "$@" >/dev/null 2>&1 < /dev/null &
  else
    nohup "$0" --detached "$@" >/dev/null 2>&1 < /dev/null &
  fi

  printf '%s\n' "$launch_log"
  exit 0
fi
shift

launch_log="$1"
parent_pid="$2"
shift 2

needs_reload=0
if [ "${1:-}" = "--reload" ]; then
  needs_reload=1
  shift
fi

if [ "${1:-}" != "--" ]; then
  echo "Error: desktop deferred launch probe requires '--' before the command argv" >&2
  exit 64
fi
shift

if [ "$#" -eq 0 ]; then
  echo "Error: desktop deferred launch probe requires a command argv" >&2
  exit 64
fi

: > "$launch_log"

i=0
while [ "$i" -lt 100 ]; do
  if ! kill -0 "$parent_pid" 2>/dev/null; then
    break
  fi
  sleep 0.05
  i=$((i + 1))
done

# Give the terminal emulator a brief moment to destroy the starter window
# after the launcher process exits, before asking the real Yazelix window
# to appear.
sleep 0.15

unset ZELLIJ ZELLIJ_SESSION_NAME ZELLIJ_PANE_ID ZELLIJ_TAB_NAME ZELLIJ_TAB_POSITION
if [ "$needs_reload" -eq 1 ]; then
  unset IN_YAZELIX_SHELL IN_NIX_SHELL
fi

if command -v setsid >/dev/null 2>&1; then
  nohup setsid "$@" >"$launch_log" 2>&1 < /dev/null &
else
  nohup "$@" >"$launch_log" 2>&1 < /dev/null &
fi
pid=$!

i=0
while [ "$i" -lt 6 ]; do
  sleep 0.05
  if ! kill -0 "$pid" 2>/dev/null; then
    wait "$pid"
    exit "$?"
  fi
  i=$((i + 1))
done

rm -f "$launch_log"
exit 0
