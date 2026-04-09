#!/bin/sh
set -eu

script_path="$0"
if [ -L "$script_path" ]; then
  link_target="$(readlink "$script_path")"
  case "$link_target" in
    /*) script_path="$link_target" ;;
    *) script_path="$(dirname "$script_path")/$link_target" ;;
  esac
fi

inferred_runtime_dir="$(cd "$(dirname "$script_path")/../.." && pwd)"
runtime_dir="${YAZELIX_RUNTIME_DIR:-$inferred_runtime_dir}"
if [ -z "$runtime_dir" ]; then
  printf '%s\n' "Error: missing Yazelix runtime directory" >&2
  exit 1
fi

helix_binary="${YAZELIX_MANAGED_HELIX_BINARY:-}"
if [ -z "$helix_binary" ]; then
  printf '%s\n' "Error: missing managed Helix binary path" >&2
  exit 1
fi

nu_bin="${YAZELIX_NU_BIN:-$runtime_dir/bin/nu}"
if [ ! -x "$nu_bin" ]; then
  if command -v "$nu_bin" >/dev/null 2>&1; then
    nu_bin="$(command -v "$nu_bin")"
  elif command -v nu >/dev/null 2>&1; then
    nu_bin="$(command -v nu)"
  else
    printf '%s\n' "Error: missing usable Nushell binary for Helix config generation" >&2
    exit 1
  fi
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
