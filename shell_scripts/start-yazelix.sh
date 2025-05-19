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
cd "$YAZELIX_DIR" || { echo "Error: Cannot cd to $YAZELIX_DIR"; exit 1; }

# Run nix develop with explicit HOME
HOME="$HOME" nix develop --impure --command zellij --config-dir "$YAZELIX_DIR/zellij" options --default-layout yazelix --default-shell nu
