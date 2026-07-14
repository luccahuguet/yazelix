#!/bin/sh
child=

cleanup() {
  status=$?
  trap - HUP INT TERM EXIT
  if [ -n "${child:-}" ] && kill -0 "$child" 2>/dev/null; then
    kill "$child" 2>/dev/null || true
    wait "$child" 2>/dev/null || true
  fi
  exit "$status"
}

trap cleanup HUP INT TERM EXIT
"$1" < /dev/tty &
child=$!
wait "$child"
status=$?
trap - HUP INT TERM EXIT
exit "$status"
