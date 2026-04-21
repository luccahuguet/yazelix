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

yzx_core_bin="${YAZELIX_YZX_CORE_BIN:-$runtime_dir/libexec/yzx_core}"
if [ ! -x "$yzx_core_bin" ]; then
  if [ -x "$runtime_dir/rust_core/target/release/yzx_core" ]; then
    yzx_core_bin="$runtime_dir/rust_core/target/release/yzx_core"
  elif [ -x "$runtime_dir/rust_core/target/debug/yzx_core" ]; then
    yzx_core_bin="$runtime_dir/rust_core/target/debug/yzx_core"
  elif command -v "$yzx_core_bin" >/dev/null 2>&1; then
    yzx_core_bin="$(command -v "$yzx_core_bin")"
  else
    printf '%s\n' "Error: missing usable yzx_core helper for Helix config generation" >&2
    exit 1
  fi
fi

jq_bin="$runtime_dir/toolbin/jq"
if [ ! -x "$jq_bin" ]; then
  if command -v jq >/dev/null 2>&1; then
    jq_bin="$(command -v jq)"
  else
    printf '%s\n' "Error: missing usable jq binary for Helix config generation" >&2
    exit 1
  fi
fi

config_home="${XDG_CONFIG_HOME:-${HOME:-}/.config}"
config_dir="${YAZELIX_CONFIG_DIR:-$config_home/yazelix}"
data_home="${XDG_DATA_HOME:-${HOME:-}/.local/share}"
state_dir="${YAZELIX_STATE_DIR:-$data_home/yazelix}"

stdout_file="$(mktemp)"
stderr_file="$(mktemp)"
cleanup() {
  rm -f "$stdout_file" "$stderr_file"
}
trap cleanup EXIT HUP INT TERM

if ! "$yzx_core_bin" helix-materialization.generate \
  --runtime-dir "$runtime_dir" \
  --config-dir "$config_dir" \
  --state-dir "$state_dir" >"$stdout_file" 2>"$stderr_file"; then
  if [ -s "$stderr_file" ] && "$jq_bin" -e '.status == "error"' "$stderr_file" >/dev/null 2>&1; then
    error_message="$("$jq_bin" -r '.error.message // ""' "$stderr_file")"
    error_remediation="$("$jq_bin" -r '.error.remediation // ""' "$stderr_file")"
    error_class="$("$jq_bin" -r '.error.class // ""' "$stderr_file")"
    case "$error_class" in
      config) error_label="config problem" ;;
      generated-state) error_label="generated-state problem" ;;
      host-dependency) error_label="host-dependency problem" ;;
      *) error_label="$error_class" ;;
    esac
    if [ -n "$error_message" ]; then
      printf '%s\n' "$error_message" >&2
    fi
    if [ -n "$error_label" ]; then
      printf '\nFailure class: %s.\n' "$error_label" >&2
    fi
    if [ -n "$error_remediation" ]; then
      printf 'Recovery: %s\n' "$error_remediation" >&2
    fi
  elif [ -s "$stderr_file" ]; then
    cat "$stderr_file" >&2
  else
    printf '%s\n' "Error: failed to generate the managed Helix config" >&2
  fi
  exit 1
fi

"$jq_bin" -r '.data.import_notice.lines[]?' "$stdout_file" >&2
managed_config="$("$jq_bin" -r '.data.generated_path // ""' "$stdout_file")"
if [ -z "$managed_config" ]; then
  printf '%s\n' "Error: failed to resolve the managed Helix config path" >&2
  exit 1
fi

exec "$helix_binary" -c "$managed_config" "$@"
