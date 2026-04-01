#!/bin/sh
set -eu

runtime_dir="${YAZELIX_RUNTIME_DIR:-${YAZELIX_DIR:-}}"
if [ -z "$runtime_dir" ]; then
  printf '%s\n' "Error: missing Yazelix runtime directory" >&2
  exit 1
fi

helix_binary="${YAZELIX_MANAGED_HELIX_BINARY:-}"
if [ -z "$helix_binary" ]; then
  printf '%s\n' "Error: missing managed Helix binary path" >&2
  exit 1
fi

nu_bin="$runtime_dir/bin/nu"
if [ ! -x "$nu_bin" ]; then
  printf '%s\n' "Error: missing runtime-local Nushell at $nu_bin" >&2
  exit 1
fi

merger_script="$runtime_dir/nushell/scripts/setup/helix_config_merger.nu"
if [ ! -f "$merger_script" ]; then
  printf '%s\n' "Error: missing Helix config merger script at $merger_script" >&2
  exit 1
fi

managed_config="$("$nu_bin" "$merger_script" --print-path)"
if [ -z "$managed_config" ]; then
  printf '%s\n' "Error: failed to generate the managed Helix config" >&2
  exit 1
fi

exec "$helix_binary" -c "$managed_config" "$@"
