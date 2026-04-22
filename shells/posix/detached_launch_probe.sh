#!/bin/sh

launch_log="$1"
shift

needs_reload=0
if [ "${1:-}" = "--reload" ]; then
  needs_reload=1
  shift
fi

if [ "${1:-}" != "--" ]; then
  echo "Error: detached launch probe requires '--' before the command argv" >&2
  exit 64
fi
shift

if [ "$#" -eq 0 ]; then
  echo "Error: detached launch probe requires a command argv" >&2
  exit 64
fi

unset ZELLIJ ZELLIJ_SESSION_NAME ZELLIJ_PANE_ID ZELLIJ_TAB_NAME ZELLIJ_TAB_POSITION
if [ "$needs_reload" -eq 1 ]; then
  unset IN_YAZELIX_SHELL IN_NIX_SHELL
fi

: > "$launch_log"
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
    status=$?
    printf '%s\n' "$launch_log"
    exit "$status"
  fi
  i=$((i + 1))
done

rm -f "$launch_log"
exit 0
