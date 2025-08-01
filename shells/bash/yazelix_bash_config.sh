#!/bin/bash
# ~/.config/yazelix/shells/bash/yazelix_bash_config.sh
# This file is part of Yazelix and should be persisted in the repository.

# Source Helix mode detection using Nushell (essential dependency)
if [ -z "$YAZELIX_HELIX_MODE" ]; then
    eval "$(nu -c 'use ~/.config/yazelix/nushell/scripts/utils/helix_mode.nu export_helix_env; export_helix_env')"
fi

# Define the directory where Yazelix generates individual initializer scripts.
# Using XDG-compliant state directory (not config directory)
YAZELIX_BASH_INITIALIZERS_DIR="$HOME/.local/share/yazelix/initializers/bash"

# Source individual initializers if they exist and are non-empty.
if [ -f "$YAZELIX_BASH_INITIALIZERS_DIR/starship_init.sh" ] && [ -s "$YAZELIX_BASH_INITIALIZERS_DIR/starship_init.sh" ]; then
  source "$YAZELIX_BASH_INITIALIZERS_DIR/starship_init.sh"
fi

if [ -f "$YAZELIX_BASH_INITIALIZERS_DIR/zoxide_init.sh" ] && [ -s "$YAZELIX_BASH_INITIALIZERS_DIR/zoxide_init.sh" ]; then
  source "$YAZELIX_BASH_INITIALIZERS_DIR/zoxide_init.sh"
fi

# mise_init.sh is generated conditionally by the shellHook.
# Source it if it exists and is non-empty.
if [ -f "$YAZELIX_BASH_INITIALIZERS_DIR/mise_init.sh" ] && [ -s "$YAZELIX_BASH_INITIALIZERS_DIR/mise_init.sh" ]; then
  source "$YAZELIX_BASH_INITIALIZERS_DIR/mise_init.sh"
fi

# Yazelix Aliases for Bash
alias yazelix="nu $HOME/.config/yazelix/nushell/scripts/core/launch_yazelix.nu"
alias yzx="$HOME/.config/yazelix/shells/bash/yzx"
alias lg='lazygit'

# Ensure this script doesn't produce output unless it's an error,
# as it's sourced by .bashrc.
