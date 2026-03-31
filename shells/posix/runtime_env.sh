#!/bin/sh

runtime_dir="$1"

if [ -z "$runtime_dir" ]; then
  echo "Error: missing Yazelix runtime directory" >&2
  return 1 2>/dev/null || exit 1
fi

export YAZELIX_RUNTIME_DIR="${YAZELIX_RUNTIME_DIR:-$runtime_dir}"
export YAZELIX_DIR="$YAZELIX_RUNTIME_DIR"
export YAZELIX_CONFIG_DIR="${YAZELIX_CONFIG_DIR:-$HOME/.config/yazelix}"
