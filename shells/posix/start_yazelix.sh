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

exec nu "$HOME/.config/yazelix/nushell/scripts/core/start_yazelix.nu"
