#!/bin/sh
# Minimal POSIX launcher for desktop entries and keybinds

PATH="$HOME/.local/state/nix/profile/bin:$HOME/.nix-profile/bin:/nix/var/nix/profiles/default/bin:$PATH"

# Load Nix profile if available (mirrors login shell behavior)
for nix_profile in "$HOME/.nix-profile/etc/profile.d/nix.sh" "/nix/var/nix/profiles/default/etc/profile.d/nix.sh"; do
  if [ -f "$nix_profile" ]; then
    . "$nix_profile"
    break
  fi
done

RUNTIME_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
launcher_script="$RUNTIME_DIR/nushell/scripts/core/desktop_launcher.nu"
runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh"
state_runtime_link="$HOME/.local/share/yazelix/runtime/current"

desktop_data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
desktop_entry_path="$desktop_data_home/applications/com.yazelix.Yazelix.desktop"

resolve_realpath() {
  if command -v readlink >/dev/null 2>&1; then
    readlink -f "$1" 2>/dev/null && return 0
  fi
  return 1
}

get_desktop_runtime_target() {
  [ -f "$desktop_entry_path" ] || return 1
  sed -n 's/^X-Yazelix-Runtime-Target="\(.*\)"$/\1/p' "$desktop_entry_path" | head -n 1
}

fail_stale_desktop_entry() {
  echo "Error: stale Yazelix desktop entry" >&2
  echo "Your installed desktop entry points at an outdated Yazelix runtime." >&2
  echo "Repair with: yzx desktop install" >&2
  exit 1
}

if [ ! -f "$runtime_env_script" ]; then
  echo "Error: Missing Yazelix runtime env helper: $runtime_env_script" >&2
  exit 1
fi

if [ -f "$desktop_entry_path" ]; then
  desktop_runtime_target="$(get_desktop_runtime_target || true)"
  current_runtime_target="$(resolve_realpath "$state_runtime_link" || true)"

  if [ -z "$desktop_runtime_target" ]; then
    fail_stale_desktop_entry
  fi

  if [ -n "$current_runtime_target" ] && [ "$desktop_runtime_target" != "$current_runtime_target" ]; then
    fail_stale_desktop_entry
  fi
fi

export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$RUNTIME_DIR"
. "$runtime_env_script" || exit 1
unset YAZELIX_BOOTSTRAP_RUNTIME_DIR

if [ ! -f "$launcher_script" ]; then
  echo "Error: Missing Yazelix desktop launcher: $launcher_script" >&2
  echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
  echo "Failure class: generated-state problem." >&2
  echo "Recovery: Restore the missing launcher script, or reinstall/regenerate Yazelix and try again." >&2
  exit 1
fi

exec "$YAZELIX_NU_BIN" "$launcher_script"
