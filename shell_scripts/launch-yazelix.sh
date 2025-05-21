#!/bin/bash
# ~/.config/yazelix/shell_scripts/launch-yazelix.sh

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

# Add aliases to Bash/Zsh configuration
SHELL_CONFIG="$HOME/.bashrc"
if [ -f "$HOME/.zshrc" ]; then
  SHELL_CONFIG="$HOME/.zshrc"
fi

if ! grep -q "alias yazelix=" "$SHELL_CONFIG"; then
  echo "# Yazelix aliases" >> "$SHELL_CONFIG"
  echo "alias yazelix=\"$HOME/.config/yazelix/shell_scripts/launch-yazelix.sh\"" >> "$SHELL_CONFIG"
  echo "alias yzx=\"$HOME/.config/yazelix/shell_scripts/launch-yazelix.sh\"" >> "$SHELL_CONFIG"
  echo "Added yazelix and yzx aliases to $SHELL_CONFIG. Run 'source $SHELL_CONFIG' to apply in the current session."
else
  echo "Aliases yazelix and yzx already exist in $SHELL_CONFIG"
fi

# Launch WezTerm in a detached manner
nohup wezterm --config-file "$WEZTERM_CONFIG" >/dev/null 2>&1 &
