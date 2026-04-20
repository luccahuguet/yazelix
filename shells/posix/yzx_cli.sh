#!/bin/sh
# Stable Yazelix CLI entrypoint for external tools and editors.

PATH="$HOME/.local/state/nix/profile/bin:$HOME/.nix-profile/bin:/nix/var/nix/profiles/default/bin:$PATH"

resolve_stable_profile_yzx() {
  if [ -n "${HOME:-}" ]; then
    if [ -e "$HOME/.nix-profile/bin/yzx" ] || [ -L "$HOME/.nix-profile/bin/yzx" ]; then
      printf '%s\n' "$HOME/.nix-profile/bin/yzx"
      return 0
    fi
  fi

  if [ -n "${USER:-}" ]; then
    user_profile_yzx="/etc/profiles/per-user/$USER/bin/yzx"
    if [ -e "$user_profile_yzx" ] || [ -L "$user_profile_yzx" ]; then
      printf '%s\n' "$user_profile_yzx"
      return 0
    fi
  fi

  return 1
}

resolve_wrapper_target() {
  target="$1"
  if [ -z "$target" ]; then
    return 1
  fi

  resolved="$(readlink -f "$target" 2>/dev/null || true)"
  if [ -n "$resolved" ]; then
    printf '%s\n' "$resolved"
  else
    printf '%s\n' "$target"
  fi
}

maybe_redirect_stale_store_invocation() {
  if [ "${YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT:-}" = "1" ]; then
    return 0
  fi

  invoked_yzx_path="${YAZELIX_INVOKED_YZX_PATH:-$0}"
  case "$invoked_yzx_path" in
    /nix/store/*/bin/yzx) ;;
    *) return 0 ;;
  esac

  stable_profile_yzx="$(resolve_stable_profile_yzx)" || return 0
  stable_profile_target="$(resolve_wrapper_target "$stable_profile_yzx")"
  invoked_yzx_target="$(resolve_wrapper_target "$invoked_yzx_path")"

  if [ "$stable_profile_target" = "$invoked_yzx_target" ]; then
    return 0
  fi

  export YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH="$invoked_yzx_path"
  export YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1
  exec "$stable_profile_yzx" "$@"
}

maybe_redirect_stale_store_invocation "$@"

for nix_profile in "$HOME/.nix-profile/etc/profile.d/nix.sh" "/nix/var/nix/profiles/default/etc/profile.d/nix.sh"; do
  if [ -f "$nix_profile" ]; then
    . "$nix_profile"
    break
  fi
done

SCRIPT_PATH="$0"
if [ -L "$SCRIPT_PATH" ]; then
  LINK_TARGET="$(readlink "$SCRIPT_PATH")"
  case "$LINK_TARGET" in
    /*) SCRIPT_PATH="$LINK_TARGET" ;;
    *) SCRIPT_PATH="$(dirname "$SCRIPT_PATH")/$LINK_TARGET" ;;
  esac
fi

RUNTIME_DIR="$(cd "$(dirname "$SCRIPT_PATH")/../.." && pwd)"
runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh"
core_script="$RUNTIME_DIR/nushell/scripts/core/yazelix.nu"
reveal_script="$RUNTIME_DIR/nushell/scripts/integrations/reveal_in_yazi.nu"

if [ ! -f "$runtime_env_script" ]; then
  echo "Error: Missing Yazelix runtime env helper: $runtime_env_script" >&2
  exit 1
fi

export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$RUNTIME_DIR"
. "$runtime_env_script" || exit 1
unset YAZELIX_BOOTSTRAP_RUNTIME_DIR

yzx_control_bin="${YAZELIX_YZX_CONTROL_BIN:-$RUNTIME_DIR/libexec/yzx_control}"
yzx_core_bin="${YAZELIX_YZX_CORE_BIN:-$RUNTIME_DIR/libexec/yzx_core}"

case "${1:-}" in
  "" | help | -h | --help)
    if [ ! -x "$yzx_core_bin" ]; then
      echo "Error: Missing Yazelix core helper: $yzx_core_bin" >&2
      echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
      exit 1
    fi
    exec "$yzx_core_bin" yzx-command-metadata.help
    ;;
esac

case "${1:-}" in
  env | run | update)
    if [ ! -x "$yzx_control_bin" ]; then
      echo "Error: Missing Yazelix control-plane helper: $yzx_control_bin" >&2
      echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
      exit 1
    fi
    subcommand="$1"
    shift
    exec "$yzx_control_bin" "$subcommand" "$@"
    ;;
esac

format_nu_token() {
  case "$1" in
    "")
      printf "''"
      ;;
    *[!A-Za-z0-9_./:=+-]*)
      printf "'%s'" "$(printf "%s" "$1" | sed "s/'/'\\\\''/g")"
      ;;
    *)
      printf "%s" "$1"
      ;;
  esac
}

exec_leaf_module_command() {
  module_path="$1"
  command_prefix="$2"
  shift 2

  if [ ! -f "$module_path" ]; then
    echo "Error: Missing Yazelix leaf command module: $module_path" >&2
    echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
    exit 1
  fi

  nu_command="use $(format_nu_token "$module_path") *; $command_prefix"
  for arg in "$@"; do
    nu_command="$nu_command $(format_nu_token "$arg")"
  done

  exec "$YAZELIX_NU_BIN" -c "$nu_command"
}

dispatch_leaf_command() {
  case "${1:-}" in
    desktop)
      shift
      exec_leaf_module_command "$RUNTIME_DIR/nushell/scripts/yzx/desktop.nu" "yzx desktop" "$@"
      ;;
    enter)
      shift
      exec_leaf_module_command "$RUNTIME_DIR/nushell/scripts/yzx/enter.nu" "yzx enter" "$@"
      ;;
    reveal)
      shift
      if [ ! -f "$reveal_script" ]; then
        echo "Error: Missing Yazelix reveal helper: $reveal_script" >&2
        echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
        exit 1
      fi
      exec "$YAZELIX_NU_BIN" "$reveal_script" "$@"
      ;;
    menu)
      shift
      exec_leaf_module_command "$RUNTIME_DIR/nushell/scripts/yzx/menu.nu" "yzx menu" "$@"
      ;;
    popup)
      shift
      exec_leaf_module_command "$RUNTIME_DIR/nushell/scripts/yzx/popup.nu" "yzx popup" "$@"
      ;;
  esac
}

dispatch_leaf_command "$@"

if [ ! -f "$core_script" ]; then
  echo "Error: Missing Yazelix CLI module: $core_script" >&2
  echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
  exit 1
fi

nu_command="use $(format_nu_token "$core_script") *; yzx"

for arg in "$@"; do
  nu_command="$nu_command $(format_nu_token "$arg")"
done

exec "$YAZELIX_NU_BIN" -c "$nu_command"
