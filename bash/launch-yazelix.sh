#!/usr/bin/env bash
# ~/.config/yazelix/bash/launch-yazelix.sh

# Resolve HOME using shell expansion
HOME=$(eval echo ~)
if [ -z "$HOME" ] || [ ! -d "$HOME" ]; then
  echo "Error: Cannot resolve HOME directory"
  exit 1
fi

echo "Resolved HOME=$HOME"

# Terminal config path will be set by the terminal detection logic below

# Read preference from environment (set by Nix shellHook)
PREFERRED_TERMINAL="${YAZELIX_PREFERRED_TERMINAL:-wezterm}"

# Check if a supported terminal is installed
TERMINAL=""
TERMINAL_CONFIG=""

if [ "$PREFERRED_TERMINAL" = "wezterm" ] && command -v wezterm &> /dev/null; then
  TERMINAL="wezterm"
  TERMINAL_CONFIG="$HOME/.config/yazelix/terminal_configs/wezterm/.wezterm.lua"
elif [ "$PREFERRED_TERMINAL" = "ghostty" ] && command -v ghostty &> /dev/null; then
  TERMINAL="ghostty"
  TERMINAL_CONFIG="$HOME/.config/yazelix/terminal_configs/ghostty/config"
elif command -v wezterm &> /dev/null; then
  # Fallback to wezterm if preferred terminal not available
  TERMINAL="wezterm"
  TERMINAL_CONFIG="$HOME/.config/yazelix/terminal_configs/wezterm/.wezterm.lua"
elif command -v ghostty &> /dev/null; then
  # Fallback to ghostty if wezterm not available
  TERMINAL="ghostty"
  TERMINAL_CONFIG="$HOME/.config/yazelix/terminal_configs/ghostty/config"
else
  echo "Error: Neither Ghostty nor WezTerm is installed. Please install one of these terminals to use Yazelix."
  echo "  - Ghostty: https://ghostty.org/"
  echo "  - WezTerm: https://wezfurlong.org/wezterm/"
  exit 1
fi

echo "Using terminal: $TERMINAL (preferred: $PREFERRED_TERMINAL)"

# Check if terminal config exists
if [ ! -f "$TERMINAL_CONFIG" ]; then
  echo "Error: $TERMINAL config not found at $TERMINAL_CONFIG"
  exit 1
fi

# Launch terminal in a detached manner
if [ "$TERMINAL" = "ghostty" ]; then
  nohup ghostty --config "$TERMINAL_CONFIG" >/dev/null 2>&1 &
elif [ "$TERMINAL" = "wezterm" ]; then
  nohup wezterm --config-file "$TERMINAL_CONFIG" >/dev/null 2>&1 &
fi
