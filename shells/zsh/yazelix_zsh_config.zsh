#!/bin/zsh
# ~/.config/yazelix/shells/zsh/yazelix_zsh_config.zsh
# This file is part of Yazelix and should be persisted in the repository.

# Source Helix mode detection using Nushell (essential dependency)
if [[ -z "$YAZELIX_HELIX_MODE" ]]; then
    eval "$(nu -c 'use ~/.config/yazelix/nushell/scripts/utils/helix_mode.nu export_helix_env; export_helix_env')"
fi

# Define the directory where Yazelix generates individual initializer scripts.
# Using XDG-compliant state directory (not config directory)
YAZELIX_ZSH_INITIALIZERS_DIR="$HOME/.local/share/yazelix/initializers/zsh"

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
alias yazelix="nu $HOME/.config/yazelix/nushell/scripts/core/launch_yazelix.nu"
alias yzx="$HOME/.config/yazelix/shells/bash/yzx"
alias lg='lazygit'

# Helix function (ensure runtime is set correctly)
hx() {
    # Ensure helix config directory exists
    local helix_config_dir="$HOME/.config/helix"
    if [[ ! -d "$helix_config_dir" ]]; then
        mkdir -p "$helix_config_dir"
    fi

    # Set runtime based on mode - both modes need HELIX_RUNTIME set
    # The runtime path is already set by the Nix environment, but ensure it's available
    if [[ -z "$HELIX_RUNTIME" ]]; then
        # Fallback: try to find runtime from helix binary
        local helix_path=$(which hx 2>/dev/null)
        if [[ -n "$helix_path" ]]; then
            local runtime_path=$(dirname "$(dirname "$helix_path")")/share/helix/runtime
            if [[ -d "$runtime_path" ]]; then
                export HELIX_RUNTIME="$runtime_path"
            fi
        fi
    fi

    command hx "$@"
}

# Add other Zsh-specific aliases or functions for Yazelix here if needed.
# For example, you could move environment variable exports specific to Zsh sessions here:
# export SOME_ZSH_SPECIFIC_VAR="value"

# Ensure this script doesn't produce output unless it's an error,
# as it's sourced by .zshrc.