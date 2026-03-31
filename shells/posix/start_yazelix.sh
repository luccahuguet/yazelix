#!/bin/sh
# Minimal POSIX launcher for terminal configs (no bashrc)

PATH="$HOME/.local/state/nix/profile/bin:$HOME/.nix-profile/bin:/nix/var/nix/profiles/default/bin:$PATH"

# Load Nix profile if available (mirrors login shell behavior)
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

RUNTIME_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
startup_script="$RUNTIME_DIR/nushell/scripts/core/start_yazelix.nu"
runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh"

if [ ! -f "$runtime_env_script" ]; then
  echo "Error: Missing Yazelix runtime env helper: $runtime_env_script" >&2
  exit 1
fi

. "$runtime_env_script" "$RUNTIME_DIR"

if [ ! -f "$startup_script" ]; then
  echo "Error: Missing Yazelix startup script: $startup_script" >&2
  echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
  echo "Failure class: generated-state problem." >&2
  echo "Recovery: Restore the missing startup script, or reinstall/regenerate Yazelix and try again." >&2
  exit 1
fi

exec nu "$startup_script"
