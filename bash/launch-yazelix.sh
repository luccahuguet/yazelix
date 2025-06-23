#!/bin/bash
# ~/.config/yazelix/bash/launch-yazelix.sh

# Resolve HOME using shell expansion
HOME=$(eval echo ~)
if [ -z "$HOME" ] || [ ! -d "$HOME" ]; then
  echo "Error: Cannot resolve HOME directory"
  exit 1
fi

echo "Resolved HOME=$HOME"

# Set absolute path for Yazelix WezTerm config
WEZTERM_CONFIG="$HOME/.config/yazelix/terminal_configs/wezterm_nix/.wezterm.lua"

# Check if WezTerm config exists
if [ ! -f "$WEZTERM_CONFIG" ]; then
  echo "Error: WezTerm config not found at $WEZTERM_CONFIG"
  exit 1
fi

# Check if WezTerm is installed
if ! command -v wezterm &> /dev/null; then
  echo "Error: WezTerm is not installed. Please install WezTerm to use Yazelix."
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

# Launch WezTerm in a detached manner
nohup wezterm --config-file "$WEZTERM_CONFIG" >/dev/null 2>&1 &
