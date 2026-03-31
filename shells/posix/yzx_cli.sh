#!/bin/sh
# Stable Yazelix CLI entrypoint for external tools and editors.

PATH="$HOME/.local/state/nix/profile/bin:$HOME/.nix-profile/bin:/nix/var/nix/profiles/default/bin:$PATH"

for nix_profile in "$HOME/.nix-profile/etc/profile.d/nix.sh" "/nix/var/nix/profiles/default/etc/profile.d/nix.sh"; do
  if [ -f "$nix_profile" ]; then
    . "$nix_profile"
    break
  fi
done

if ! command -v nu >/dev/null 2>&1; then
  echo "Error: nu not found in PATH after loading Nix profile." >&2
  exit 1
fi

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

if [ ! -f "$runtime_env_script" ]; then
  echo "Error: Missing Yazelix runtime env helper: $runtime_env_script" >&2
  exit 1
fi

. "$runtime_env_script" "$RUNTIME_DIR"

if [ ! -f "$core_script" ]; then
  echo "Error: Missing Yazelix CLI module: $core_script" >&2
  echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
  exit 1
fi

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

nu_command="use $(format_nu_token "$core_script") *; yzx"

for arg in "$@"; do
  nu_command="$nu_command $(format_nu_token "$arg")"
done

exec nu -c "$nu_command"
