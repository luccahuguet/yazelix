#!/bin/sh

runtime_dir="${YAZELIX_BOOTSTRAP_RUNTIME_DIR:-$1}"

if [ -z "$runtime_dir" ]; then
  echo "Error: missing Yazelix runtime directory" >&2
  return 1 2>/dev/null || exit 1
fi

export YAZELIX_RUNTIME_DIR="$runtime_dir"
xdg_config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
xdg_data_home="${XDG_DATA_HOME:-$HOME/.local/share}"

export YAZELIX_CONFIG_DIR="${YAZELIX_CONFIG_DIR:-$xdg_config_home/yazelix}"
export YAZELIX_STATE_DIR="${YAZELIX_STATE_DIR:-$xdg_data_home/yazelix}"
export YAZELIX_LOGS_DIR="${YAZELIX_LOGS_DIR:-$YAZELIX_STATE_DIR/logs}"
export PATH="$runtime_dir/bin:$PATH"

runtime_nu="$runtime_dir/bin/nu"
if [ -x "$runtime_nu" ]; then
  export YAZELIX_NU_BIN="$runtime_nu"
elif command -v nu >/dev/null 2>&1; then
  export YAZELIX_NU_BIN="$(command -v nu)"
else
  echo "Error: nu not found in Yazelix runtime or PATH after loading Nix profile." >&2
  return 1 2>/dev/null || exit 1
fi
