#!/bin/sh

launch_log="$1"
launch_cmd="$2"

: > "$launch_log"
nohup bash -lc "$launch_cmd" >"$launch_log" 2>&1 < /dev/null &
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
