#!/bin/sh
set -eu

SCRIPT_PATH="$0"
if [ -L "$SCRIPT_PATH" ]; then
  LINK_TARGET="$(readlink "$SCRIPT_PATH")"
  case "$LINK_TARGET" in
    /*) SCRIPT_PATH="$LINK_TARGET" ;;
    *) SCRIPT_PATH="$(dirname "$SCRIPT_PATH")/$LINK_TARGET" ;;
  esac
fi

RUNTIME_DIR="$(cd "$(dirname "$SCRIPT_PATH")/../.." && pwd)"
runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh"
yzx_cli="$RUNTIME_DIR/shells/posix/yzx_cli.sh"

if [ ! -f "$runtime_env_script" ]; then
  echo "Error: Missing Yazelix runtime env helper: $runtime_env_script" >&2
  exit 1
fi

if [ ! -x "$yzx_cli" ]; then
  echo "Error: Missing Yazelix CLI wrapper: $yzx_cli" >&2
  exit 1
fi

export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$RUNTIME_DIR"
. "$runtime_env_script"
unset YAZELIX_BOOTSTRAP_RUNTIME_DIR

if [ -n "${ZELLIJ:-}" ]; then
  zellij action rename-pane yzx_popup >/dev/null 2>&1 || true
fi

export YAZELIX_POPUP_PANE=true
exec "$yzx_cli" popup "$@"
