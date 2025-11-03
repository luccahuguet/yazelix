#!/bin/bash
# ~/.config/yazelix/shells/bash/start_yazelix.sh

# Resolve HOME using shell expansion
HOME=$(eval echo ~)
if [ -z "$HOME" ] || [ ! -d "$HOME" ]; then
  echo "Error: Cannot resolve HOME directory"
  exit 1
fi

echo "Resolved HOME=$HOME"

# Set absolute path for Yazelix directory
YAZELIX_DIR="$HOME/.config/yazelix"

# Navigate to Yazelix directory
# This is important for devenv to find devenv.nix in the current directory
cd "$YAZELIX_DIR" || { echo "Error: Cannot cd to $YAZELIX_DIR"; exit 1; }

# Ensure devenv is available
if ! command -v devenv >/dev/null 2>&1; then
  echo ""
  echo "❌ 'devenv' command not found."
  echo "   Yazelix v11+ moved from flake-based 'nix develop' shells to devenv."
  echo "   Install devenv with:"
  echo "     nix profile install github:cachix/devenv/latest"
  echo "   After installing, relaunch Yazelix (or run 'devenv shell --impure')."
  echo "   Old commands like 'nix develop' are no longer supported."
  echo ""
  exit 1
fi

# Detect configuration changes (requires Nushell)
if command -v nu >/dev/null 2>&1; then
  NEEDS_REFRESH=$(nu -c 'use ~/.config/yazelix/nushell/scripts/utils/config_state.nu compute_config_state; let state = compute_config_state; if $state.needs_refresh { "true" } else { "" }')
  if [ "$NEEDS_REFRESH" = "true" ]; then
    echo "♻️  Config changed since last launch – rebuilding environment"
    export YAZELIX_FORCE_REFRESH="true"
  fi
fi

# Run devenv shell with explicit HOME.
# The YAZELIX_DEFAULT_SHELL variable will be set by the enterShell hook
# and used by the inner zellij command.
# We use bash -c '...' to ensure $YAZELIX_DEFAULT_SHELL is expanded after devenv sets it.
HOME="$HOME" devenv shell --impure -- bash -c \
  "zellij --config-dir \"$YAZELIX_DIR/configs/zellij\" options \
    --default-cwd \"$HOME\" \
    --default-layout \"\$ZELLIJ_DEFAULT_LAYOUT\" \
    --default-shell \"\$YAZELIX_DEFAULT_SHELL\""

if [ "$NEEDS_REFRESH" = "true" ] && command -v nu >/dev/null 2>&1; then
  nu -c 'use ~/.config/yazelix/nushell/scripts/utils/config_state.nu [compute_config_state mark_config_state_applied]; mark_config_state_applied (compute_config_state)'
fi
