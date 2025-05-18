#!/bin/bash

# Exit on errors
set -e

# Normalize working directory to project root
cd "$(dirname "$0")/.."

# Ensure PATH includes buildInputs
export PATH="$PATH:/run/current-system/sw/bin"

# Create logs directory
mkdir -p "$HOME/.config/yazelix/logs" || echo "Warning: Could not create logs directory at $HOME/.config/yazelix/logs"

# Set up logging to ~/.config/yazelix/logs/setup-yazelix.log
LOG_FILE="$HOME/.config/yazelix/logs/setup-yazelix.log"
echo "=== Yazelix setup log: $(date) ===" >> "$LOG_FILE"

# Redirect all output to log file and terminal
exec > >(tee -a "$LOG_FILE") 2>&1

# Set up Zellij config directory
export ZELLIJ_CONFIG_DIR="$HOME/.config/yazelix/zellij"
mkdir -p "$ZELLIJ_CONFIG_DIR" || { echo "Error: Failed to create ZELLIJ_CONFIG_DIR"; exit 1; }
if [ -d "$PWD/config/zellij" ]; then
  cp -r "$PWD/config/zellij/." "$ZELLIJ_CONFIG_DIR/" || { echo "Error: Failed to copy Zellij configs"; exit 1; }
fi

# Set up Yazi config
export YAZI_CONFIG_HOME="$PWD/config/yazi"

# Ensure Nushell config directory
export XDG_CONFIG_HOME="$HOME/.config"
mkdir -p "$HOME/.config/nushell" || echo "Warning: Could not create Nushell config directory; it may already exist or be managed elsewhere."

# Ensure ~/.config/yazelix/nushell/config.nu exists (assumed to be persisted in repo)
if [ ! -f "$HOME/.config/yazelix/nushell/config.nu" ]; then
  echo "Warning: ~/.config/yazelix/nushell/config.nu not found. Creating minimal file."
  mkdir -p "$HOME/.config/yazelix/nushell"
  echo "# Yazelix Nushell config" > "$HOME/.config/yazelix/nushell/config.nu"
fi

# Set STARSHIP_SHELL for Nushell detection
export STARSHIP_SHELL=nu

# Generate Starship initialization script
# Note: $HOME/.config/yazelix/nushell already exists
echo "# Starship initialization for Nushell" > "$HOME/.config/yazelix/nushell/starship_init.nu"
starship init nu >> "$HOME/.config/yazelix/nushell/starship_init.nu"

# Append Starship script source to ~/.config/yazelix/nushell/config.nu
if ! grep -q "source ~/.config/yazelix/nushell/starship_init.nu" "$HOME/.config/yazelix/nushell/config.nu"; then
  echo "source ~/.config/yazelix/nushell/starship_init.nu" >> "$HOME/.config/yazelix/nushell/config.nu"
fi

# Remove old starship.nu if it exists
if [ -f "$HOME/.config/yazelix/nushell/starship.nu" ]; then
  rm "$HOME/.config/yazelix/nushell/starship.nu"
fi

# Generate Zoxide initialization script
# Note: $HOME/.config/yazelix/nushell already exists
echo "# Zoxide initialization for Nushell" > "$HOME/.config/yazelix/nushell/zoxide_init.nu"
zoxide init nushell >> "$HOME/.config/yazelix/nushell/zoxide_init.nu"

# Append Zoxide script source to ~/.config/yazelix/nushell/config.nu
if ! grep -q "source ~/.config/yazelix/nushell/zoxide_init.nu" "$HOME/.config/yazelix/nushell/config.nu"; then
  echo "source ~/.config/yazelix/nushell/zoxide_init.nu" >> "$HOME/.config/yazelix/nushell/config.nu"
fi

# Manage ~/.config/nushell/config.nu for users with existing Nushell configs
if [ -f "$HOME/.config/nushell/config.nu" ]; then
  # Back up existing config if not already backed up
  if [ ! -f "$HOME/.config/nushell/config.nu.bak" ]; then
    cp "$HOME/.config/nushell/config.nu" "$HOME/.config/nushell/config.nu.bak"
    echo "Backed up existing ~/.config/nushell/config.nu to ~/.config/nushell/config.nu.bak"
  fi
  # Check if source command already exists
  if ! grep -q "source ~/.config/yazelix/nushell/config.nu" "$HOME/.config/nushell/config.nu"; then
    echo "# Source Yazelix Nushell config for Starship and Zoxide integration" >> "$HOME/.config/nushell/config.nu"
    echo "source ~/.config/yazelix/nushell/config.nu" >> "$HOME/.config/nushell/config.nu"
    echo "Added Yazelix config source to ~/.config/nushell/config.nu"
  fi
else
  # Create config.nu with source command for new users
  echo "# Nushell config file" > "$HOME/.config/nushell/config.nu"
  echo "# Source Yazelix Nushell config for Starship and Zoxide integration" >> "$HOME/.config/nushell/config.nu"
  echo "source ~/.config/yazelix/nushell/config.nu" >> "$HOME/.config/nushell/config.nu"
  echo "Created new ~/.config/nushell/config.nu with Yazelix config source"
fi

# Set editor
export EDITOR=hx

# Set Zellij default layout. comment it to disable yazelix as the default layout
export ZELLIJ_DEFAULT_LAYOUT=yazelix

# Print welcome message
echo "Yazelix environment ready! Use 'z' for smart directory navigation."
