#!/bin/sh
YAZELIX_STATE_DIR="${YAZELIX_STATE_DIR:-${XDG_DATA_HOME:-${HOME:-/tmp}/.local/share}/yazelix-next}"
export YAZELIX_STATE_DIR

if [ -z "${YAZELIX_HELIX_BRIDGE_SESSION_ID:-}" ]; then
  YAZELIX_HELIX_BRIDGE_SESSION_ID="yzn-helper-$(@date@ +%s)-$$"
fi
export YAZELIX_HELIX_BRIDGE_SESSION_ID

export YAZELIX_HELIX_BRIDGE=1
YAZELIX_HELIX_BRIDGE_INSTANCE_ID="hx-$(@date@ +%s)-$$"
export YAZELIX_HELIX_BRIDGE_INSTANCE_ID
YAZELIX_HELIX_BRIDGE_AUTH_TOKEN="$(@od@ -An -N32 -tx1 /dev/urandom | @tr@ -d ' \n')"
export YAZELIX_HELIX_BRIDGE_AUTH_TOKEN
export YAZELIX_HELIX_MANAGED_CONFIG_PATH="@yznHelixConfig@"

@mkdir@ -p "$YAZELIX_STATE_DIR"
exec @hx@ --config-dir "@yznHelixConfig@" "$@"
