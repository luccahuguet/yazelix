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

scrub_yazelix_workspace_child_gui_env() {
  unset GIO_EXTRA_MODULES
  unset GIO_MODULE_DIR
  unset GSETTINGS_SCHEMA_DIR
  unset GI_TYPELIB_PATH
  unset GTK_PATH
  unset GTK_EXE_PREFIX
  unset GTK_DATA_PREFIX
  unset GDK_PIXBUF_MODULE_FILE
  unset GDK_PIXBUF_MODULEDIR
}

strip_runtime_path_entries() {
  current_path="${1:-}"
  cleaned_path=""
  remaining_path="$current_path"

  while :; do
    case "$remaining_path" in
      *:*)
        entry=${remaining_path%%:*}
        remaining_path=${remaining_path#*:}
        has_more=1
        ;;
      *)
        entry=$remaining_path
        remaining_path=""
        has_more=0
        ;;
    esac

    case "$entry" in
      ""|"${runtime_dir}/libexec"|"${runtime_dir}/toolbin"|"${runtime_dir}/bin")
        ;;
      *)
        if [ -z "$cleaned_path" ]; then
          cleaned_path="$entry"
        else
          cleaned_path="${cleaned_path}:$entry"
        fi
        ;;
    esac

    if [ "$has_more" -eq 0 ]; then
      break
    fi
  done

  printf '%s\n' "$cleaned_path"
}

prepend_existing_path_dir() {
  dir="$1"
  current_path="${2:-}"

  if [ ! -d "$dir" ]; then
    printf '%s\n' "$current_path"
    return 0
  fi

  if [ -n "$current_path" ]; then
    printf '%s:%s\n' "$dir" "$current_path"
  else
    printf '%s\n' "$dir"
  fi
}

# Export only the curated interactive tool surface. The full libexec helper
# closure stays runtime-private so host apps do not inherit shadowing helpers
# like coreutils ahead of the system PATH.
cleaned_path="$(strip_runtime_path_entries "${PATH:-}")"
if [ -n "${YAZELIX_TEST_PATH_PREPEND:-}" ] && [ -d "$YAZELIX_TEST_PATH_PREPEND" ]; then
  cleaned_path="$(prepend_existing_path_dir "$YAZELIX_TEST_PATH_PREPEND" "$cleaned_path")"
fi
cleaned_path="$(prepend_existing_path_dir "$runtime_dir/bin" "$cleaned_path")"
cleaned_path="$(prepend_existing_path_dir "$runtime_dir/toolbin" "$cleaned_path")"
export PATH="$cleaned_path"

runtime_nu="$runtime_dir/libexec/nu"
if [ -x "$runtime_nu" ]; then
  export YAZELIX_NU_BIN="$runtime_nu"
elif command -v nu >/dev/null 2>&1; then
  export YAZELIX_NU_BIN="$(command -v nu)"
else
  echo "Error: nu not found in Yazelix runtime or PATH after loading Nix profile." >&2
  return 1 2>/dev/null || exit 1
fi

runtime_yzx_core="$runtime_dir/libexec/yzx_core"
if [ -x "$runtime_yzx_core" ]; then
  export YAZELIX_YZX_CORE_BIN="$runtime_yzx_core"
fi

runtime_yzx_control="$runtime_dir/libexec/yzx_control"
if [ -x "$runtime_yzx_control" ]; then
  export YAZELIX_YZX_CONTROL_BIN="$runtime_yzx_control"
fi
