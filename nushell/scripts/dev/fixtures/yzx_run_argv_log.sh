#!/bin/sh
# Test fixture: log argv as JSON for yzx_control run integration tests.
cmd="$1"
shift
printf '%s\n' "$@" | jq -R . | jq -s --arg cmd "$cmd" --arg cwd "$PWD" '{command: $cmd, args: ., cwd: $cwd, config_present: true}' > "$YZX_RUN_LOG"
