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

if ! command -v nu >/dev/null 2>&1; then
  echo "Error: nu not found in PATH. Install Nushell or restart your shell." >&2
  exit 1
fi

RUNTIME_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
launcher_script="$RUNTIME_DIR/nushell/scripts/core/desktop_launcher.nu"

export YAZELIX_RUNTIME_DIR="${YAZELIX_RUNTIME_DIR:-$RUNTIME_DIR}"
export YAZELIX_DIR="$YAZELIX_RUNTIME_DIR"
export YAZELIX_CONFIG_DIR="${YAZELIX_CONFIG_DIR:-$HOME/.config/yazelix}"

if [ ! -f "$launcher_script" ]; then
  echo "Error: Missing Yazelix desktop launcher: $launcher_script" >&2
  echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
  echo "Failure class: generated-state problem." >&2
  echo "Recovery: Restore the missing launcher script, or reinstall/regenerate Yazelix and try again." >&2
  exit 1
fi

exec nu "$launcher_script"
