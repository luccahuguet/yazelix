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

runtime_dir="$(cd "$(dirname "$SCRIPT_PATH")/../.." && pwd)"
runtime_env_script="$runtime_dir/shells/posix/runtime_env.sh"
managed_config="$runtime_dir/nushell/config/config.nu"

if [ ! -f "$runtime_env_script" ]; then
  printf '%s\n' "Error: missing Yazelix runtime env helper: $runtime_env_script" >&2
  exit 1
fi

if [ ! -f "$managed_config" ]; then
  printf '%s\n' "Error: missing managed Nushell config: $managed_config" >&2
  exit 1
fi

export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$runtime_dir"
. "$runtime_env_script"
unset YAZELIX_BOOTSTRAP_RUNTIME_DIR

exec "$YAZELIX_NU_BIN" --login --env-config /dev/null --config "$managed_config" "$@"
