#!/bin/sh

if [ "$#" -eq 0 ]; then
  echo "Error: missing Ghostty command" >&2
  exit 1
fi

script_dir="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
default_runtime_dir="$(CDPATH= cd -- "$script_dir/../.." && pwd)"
runtime_dir="${YAZELIX_RUNTIME_DIR:-$default_runtime_dir}"

if [ -d "$runtime_dir/bin" ]; then
  PATH="$runtime_dir/bin:$PATH"
  export PATH
fi

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

exec "$@"
