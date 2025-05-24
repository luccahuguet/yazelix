#!/bin/bash
# ~/.config/yazelix/shell_scripts/start-yazelix.sh

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
# This is important for nix develop to find the flake.nix in the current directory
cd "$YAZELIX_DIR" || { echo "Error: Cannot cd to $YAZELIX_DIR"; exit 1; }

# Run nix develop with explicit HOME.
# The YAZELIX_DEFAULT_SHELL variable will be set by the shellHook of the flake
# and used by the inner zellij command.
# We use bash -c '...' to ensure $YAZELIX_DEFAULT_SHELL is expanded after nix develop sets it.
# Added --cwd "$HOME" to the zellij command to start it in the user's home directory.
HOME="$HOME" nix develop --impure --command bash -c \
  "zellij --config-dir \"$YAZELIX_DIR/zellij\" options --default-cwd \"$HOME\" --default-layout yazelix --default-shell \"\$YAZELIX_DEFAULT_SHELL\""
