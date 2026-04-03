#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  printf '%s\n' "Error: missing managed terminal name" >&2
  exit 1
fi

terminal_name="$1"
shift

terminal_config_mode="yazelix"
if [ "${1:-}" = "--config-mode" ]; then
  if [ "$#" -lt 2 ]; then
    printf '%s\n' "Error: missing terminal config mode after --config-mode" >&2
    exit 1
  fi
  terminal_config_mode="$2"
  shift 2
fi

runtime_dir="${YAZELIX_RUNTIME_DIR:-}"
if [ -z "$runtime_dir" ] || [ ! -d "$runtime_dir" ]; then
  printf '%s\n' "Error: missing Yazelix runtime directory" >&2
  exit 1
fi

nu_bin="${YAZELIX_NU_BIN:-nu}"
if ! command -v "$nu_bin" >/dev/null 2>&1; then
  printf '%s\n' "Error: Nushell is not available for managed terminal launch" >&2
  exit 1
fi

if ! CONF="$("$nu_bin" -c "source \"${runtime_dir}/nushell/scripts/utils/terminal_launcher.nu\"; print (resolve_terminal_config \"${terminal_name}\" \"${terminal_config_mode}\")")"; then
  exit 1
fi

startup_script="${runtime_dir}/shells/posix/start_yazelix.sh"
if [ ! -x "$startup_script" ]; then
  printf '%s\n' "Error: missing Yazelix startup script at ${startup_script}" >&2
  exit 1
fi

launch_prefix=()
if command -v nixGLDefault >/dev/null 2>&1; then
  launch_prefix+=( "$(command -v nixGLDefault)" )
elif command -v nixGLIntel >/dev/null 2>&1; then
  launch_prefix+=( "$(command -v nixGLIntel)" )
fi

case "$terminal_name" in
  ghostty)
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

    terminal_bin="$(command -v ghostty)"
    if [ -z "$terminal_bin" ]; then
      printf '%s\n' "Error: ghostty is not available in PATH" >&2
      exit 1
    fi

    exec "${launch_prefix[@]}" "$terminal_bin" \
      --config-default-files=false \
      --config-file="$CONF" \
      --gtk-single-instance=false \
      --class="com.yazelix.Yazelix" \
      --x11-instance-name="yazelix" \
      --title="Yazelix - Ghostty" "$@" \
      -e sh -c "exec \"$startup_script\""
    ;;
  kitty)
    terminal_bin="$(command -v kitty)"
    if [ -z "$terminal_bin" ]; then
      printf '%s\n' "Error: kitty is not available in PATH" >&2
      exit 1
    fi

    exec "${launch_prefix[@]}" "$terminal_bin" \
      --config="$CONF" \
      --class="com.yazelix.Yazelix" \
      --title="Yazelix - Kitty" "$@" \
      sh -c "exec \"$startup_script\""
    ;;
  wezterm)
    terminal_bin="$(command -v wezterm)"
    if [ -z "$terminal_bin" ]; then
      printf '%s\n' "Error: wezterm is not available in PATH" >&2
      exit 1
    fi

    exec "${launch_prefix[@]}" "$terminal_bin" \
      --config-file "$CONF" start --class=com.yazelix.Yazelix "$@" -- \
      sh -c "exec \"$startup_script\""
    ;;
  alacritty)
    terminal_bin="$(command -v alacritty)"
    if [ -z "$terminal_bin" ]; then
      printf '%s\n' "Error: alacritty is not available in PATH" >&2
      exit 1
    fi

    exec "${launch_prefix[@]}" "$terminal_bin" \
      --config-file "$CONF" \
      --class "com.yazelix.Yazelix" \
      --title "Yazelix - Alacritty" "$@" \
      -e sh -c "exec \"$startup_script\""
    ;;
  foot)
    terminal_bin="$(command -v foot)"
    if [ -z "$terminal_bin" ]; then
      printf '%s\n' "Error: foot is not available in PATH" >&2
      exit 1
    fi

    exec "${launch_prefix[@]}" "$terminal_bin" \
      --config "$CONF" \
      --app-id "com.yazelix.Yazelix" "$@" \
      sh -c "exec \"$startup_script\""
    ;;
  *)
    printf '%s\n' "Error: unsupported managed terminal '${terminal_name}'" >&2
    exit 1
    ;;
esac
