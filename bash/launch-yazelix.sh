#!/bin/bash
# ~/.config/yazelix/bash/launch-yazelix.sh

# Resolve HOME using shell expansion
HOME=$(eval echo ~)
if [ -z "$HOME" ] || [ ! -d "$HOME" ]; then
  echo "Error: Cannot resolve HOME directory"
  exit 1
fi

echo "Resolved HOME=$HOME"

# Terminal config path will be set by the terminal detection logic below

# Check if a supported terminal is installed
TERMINAL=""
TERMINAL_CONFIG=""

if command -v ghostty &> /dev/null; then
  TERMINAL="ghostty"
  TERMINAL_CONFIG="$HOME/.config/yazelix/terminal_configs/ghostty/config"
elif command -v wezterm &> /dev/null; then
  TERMINAL="wezterm"
  TERMINAL_CONFIG="$HOME/.config/yazelix/terminal_configs/wezterm/.wezterm.lua"
else
  echo "Error: Neither Ghostty nor WezTerm is installed. Please install one of these terminals to use Yazelix."
  echo "  - Ghostty: https://ghostty.org/"
  echo "  - WezTerm: https://wezfurlong.org/wezterm/"
  exit 1
fi

echo "Using terminal: $TERMINAL"

# Check if terminal config exists
if [ ! -f "$TERMINAL_CONFIG" ]; then
  echo "Error: $TERMINAL config not found at $TERMINAL_CONFIG"
  exit 1
fi

# Determine shell configuration file
SHELL_CONFIG_FILE=""
CURRENT_SHELL_NAME=$(basename "$SHELL")

if [ "$CURRENT_SHELL_NAME" = "bash" ]; then
  SHELL_CONFIG_FILE="$HOME/.bashrc"
elif [ "$CURRENT_SHELL_NAME" = "zsh" ]; then
  SHELL_CONFIG_FILE="$HOME/.zshrc"
else
  # Fallback for unknown shells, user might need to adjust
  SHELL_CONFIG_FILE="$HOME/.bashrc"
  echo "Warning: Could not reliably determine shell from '$SHELL'."
  echo "Attempting to use $SHELL_CONFIG_FILE for aliases."
  echo "If this is incorrect, please add aliases manually."
fi

YAZELIX_LAUNCH_SCRIPT_PATH="$HOME/.config/yazelix/bash/launch-yazelix.sh"
# Standardized markers
YAZELIX_ALIAS_BLOCK_START="# BEGIN YAZELIX ALIASES (added by Yazelix)"
YAZELIX_ALIAS_BLOCK_END="# END YAZELIX ALIASES (added by Yazelix)"
YAZELIX_ALIAS_YAZELIX="alias yazelix=\"$YAZELIX_LAUNCH_SCRIPT_PATH\""
YAZELIX_ALIAS_YZX="alias yzx=\"$YAZELIX_LAUNCH_SCRIPT_PATH\""

# Check if the alias block already exists using the start marker
if [ -f "$SHELL_CONFIG_FILE" ] && grep -qF -- "$YAZELIX_ALIAS_BLOCK_START" "$SHELL_CONFIG_FILE"; then
  echo "Yazelix aliases (marked by Yazelix) already configured in $SHELL_CONFIG_FILE."
else
  echo "Yazelix can add 'yazelix' and 'yzx' aliases to your $SHELL_CONFIG_FILE."
  read -r -p "Would you like to add these aliases? (Y/n) " response
  response=${response,,} # tolower

  if [[ "$response" =~ ^(yes|y|"")$ ]]; then
    echo "Adding Yazelix aliases to $SHELL_CONFIG_FILE..."
    touch "$SHELL_CONFIG_FILE" # Ensure file exists
    {
      echo "" # Add a newline for separation
      echo "$YAZELIX_ALIAS_BLOCK_START"
      echo "$YAZELIX_ALIAS_YAZELIX"
      echo "$YAZELIX_ALIAS_YZX"
      echo "$YAZELIX_ALIAS_BLOCK_END"
    } >> "$SHELL_CONFIG_FILE"
    echo "Aliases added. Please run 'source $SHELL_CONFIG_FILE' or open a new terminal to use them."
  else
    echo "Skipping Yazelix alias installation."
    echo "You can add them manually to your $SHELL_CONFIG_FILE if you change your mind:"
    echo "  $YAZELIX_ALIAS_BLOCK_START"
    echo "  $YAZELIX_ALIAS_YAZELIX"
    echo "  $YAZELIX_ALIAS_YZX"
    echo "  $YAZELIX_ALIAS_BLOCK_END"
  fi
fi

# Launch terminal in a detached manner
if [ "$TERMINAL" = "ghostty" ]; then
  nohup ghostty --config "$TERMINAL_CONFIG" >/dev/null 2>&1 &
elif [ "$TERMINAL" = "wezterm" ]; then
  nohup wezterm --config-file "$TERMINAL_CONFIG" >/dev/null 2>&1 &
fi
