#!/bin/zsh
# ~/.config/yazelix/zsh/yazelix_zsh_config.zsh
# This file is part of Yazelix and should be persisted in the repository.

# Define the directory where Yazelix generates individual initializer scripts.
# This path MUST match the one used in the flake.nix shellHook.
YAZELIX_ZSH_INITIALIZERS_DIR="$HOME/.config/yazelix/zsh/initializers"

# Source individual initializers if they exist and are non-empty.
if [[ -f "$YAZELIX_ZSH_INITIALIZERS_DIR/starship_init.zsh" && -s "$YAZELIX_ZSH_INITIALIZERS_DIR/starship_init.zsh" ]]; then
  source "$YAZELIX_ZSH_INITIALIZERS_DIR/starship_init.zsh"
fi

if [[ -f "$YAZELIX_ZSH_INITIALIZERS_DIR/zoxide_init.zsh" && -s "$YAZELIX_ZSH_INITIALIZERS_DIR/zoxide_init.zsh" ]]; then
  source "$YAZELIX_ZSH_INITIALIZERS_DIR/zoxide_init.zsh"
fi

# mise_init.zsh is generated conditionally by the shellHook.
# Source it if it exists and is non-empty.
if [[ -f "$YAZELIX_ZSH_INITIALIZERS_DIR/mise_init.zsh" && -s "$YAZELIX_ZSH_INITIALIZERS_DIR/mise_init.zsh" ]]; then
  source "$YAZELIX_ZSH_INITIALIZERS_DIR/mise_init.zsh"
fi

# carapace_init.zsh for completion support
if [[ -f "$YAZELIX_ZSH_INITIALIZERS_DIR/carapace_init.zsh" && -s "$YAZELIX_ZSH_INITIALIZERS_DIR/carapace_init.zsh" ]]; then
  source "$YAZELIX_ZSH_INITIALIZERS_DIR/carapace_init.zsh"
fi

# Yazelix Aliases for Zsh
alias yazelix="$HOME/.config/yazelix/bash/launch-yazelix.sh"
alias yzx="$HOME/.config/yazelix/bash/launch-yazelix.sh"
alias lg='lazygit'

# Add other Zsh-specific aliases or functions for Yazelix here if needed.
# For example, you could move environment variable exports specific to Zsh sessions here:
# export SOME_ZSH_SPECIFIC_VAR="value"

# Ensure this script doesn't produce output unless it's an error,
# as it's sourced by .zshrc. 