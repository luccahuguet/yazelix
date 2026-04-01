#!/usr/bin/env bash
set -euo pipefail

runtime_dir="${YAZELIX_RUNTIME_DIR:-}"
if [ -z "$runtime_dir" ] || [ ! -d "$runtime_dir" ]; then
  printf '%s\n' "Error: missing Yazelix runtime directory" >&2
  exit 1
fi

nu_bin="${YAZELIX_NU_BIN:-nu}"
if ! command -v "$nu_bin" >/dev/null 2>&1; then
  printf '%s\n' "Error: Nushell is not available for Ghostty launch" >&2
  exit 1
fi

if ! CONF="$("$nu_bin" -c "source \"${runtime_dir}/nushell/scripts/utils/terminal_launcher.nu\"; print (resolve_terminal_config_from_env \"ghostty\")")"; then
  exit 1
fi

# On Wayland, stale IM variables (e.g. GTK_IM_MODULE=ibus without daemon)
# can break dead-key/compose input in GTK terminals.
if [ -n "${WAYLAND_DISPLAY:-}" ]; then
  use_simple_im=0
  if [ -z "${GTK_IM_MODULE:-}" ]; then
    use_simple_im=1
  fi
  if [ "${GTK_IM_MODULE:-}" = "ibus" ]; then
    if ! command -v pgrep >/dev/null 2>&1 || ! pgrep -x ibus-daemon >/dev/null 2>&1; then
      use_simple_im=1
    fi
  fi
  case "${GTK_IM_MODULE:-}" in
    fcitx|fcitx5)
      if ! command -v pgrep >/dev/null 2>&1 || { ! pgrep -x fcitx5 >/dev/null 2>&1 && ! pgrep -x fcitx >/dev/null 2>&1; }; then
        use_simple_im=1
      fi
      ;;
  esac
  if [ "$use_simple_im" -eq 1 ]; then
    export GTK_IM_MODULE="simple"
    unset QT_IM_MODULE XMODIFIERS
  fi
elif [ -z "${GTK_IM_MODULE:-}" ]; then
  export GTK_IM_MODULE="simple"
fi

ghostty_bin="$(command -v ghostty)"
if [ -z "$ghostty_bin" ]; then
  printf '%s\n' "Error: ghostty is not available in PATH" >&2
  exit 1
fi

launch_prefix=()
if command -v nixGLDefault >/dev/null 2>&1; then
  launch_prefix+=( "$(command -v nixGLDefault)" )
elif command -v nixGLIntel >/dev/null 2>&1; then
  launch_prefix+=( "$(command -v nixGLIntel)" )
fi

startup_script="${runtime_dir}/shells/posix/start_yazelix.sh"
if [ ! -x "$startup_script" ]; then
  printf '%s\n' "Error: missing Yazelix startup script at ${startup_script}" >&2
  exit 1
fi

exec "${launch_prefix[@]}" "$ghostty_bin" \
  --config-default-files=false \
  --config-file="$CONF" \
  --gtk-single-instance=false \
  --class="com.yazelix.Yazelix" \
  --x11-instance-name="yazelix" \
  --title="Yazelix - Ghostty" "$@" \
  -e sh -c "exec \"$startup_script\""
