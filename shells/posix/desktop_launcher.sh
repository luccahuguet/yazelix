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

if [ ! -f "$runtime_env_script" ]; then
  echo "Error: Missing Yazelix runtime env helper: $runtime_env_script" >&2
  exit 1
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
