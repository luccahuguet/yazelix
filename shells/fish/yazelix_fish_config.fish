# Yazelix Fish Configuration
# This file is sourced by ~/.config/fish/config.fish

# Source Helix mode detection using Nushell (essential dependency)
if test -z $YAZELIX_HELIX_MODE
    eval (nu -c 'use ~/.config/yazelix/nushell/scripts/utils/helix_mode.nu export_helix_env; export_helix_env')
end

# Source generated initializers if they exist
# Using XDG-compliant state directory (not config directory)
set -l FISH_INITIALIZERS_DIR "$HOME/.local/share/yazelix/initializers/fish"

# Source each initializer if it exists
for file in $FISH_INITIALIZERS_DIR/*.fish
    if test -f $file
        source $file
    end
end

# Yazelix aliases
alias yazelix="nu $HOME/.config/yazelix/nushell/scripts/core/launch_yazelix.nu"
alias yzx="$HOME/.config/yazelix/shells/bash/yzx"
alias lg='lazygit'

# Helix function (ensure runtime is set correctly)
function hx --description "Helix editor with Yazelix mode support"
    # Ensure helix config directory exists
    set -l helix_config_dir "$HOME/.config/helix"
    if not test -d $helix_config_dir
        mkdir -p $helix_config_dir
    end

    # Set runtime based on mode - both modes need HELIX_RUNTIME set
    # The runtime path is already set by the Nix environment, but ensure it's available
    if test -z $HELIX_RUNTIME
        # Fallback: try to find runtime from helix binary
        set -l helix_path (which hx 2>/dev/null)
        if test -n $helix_path
            set -l runtime_path (dirname (dirname $helix_path))/share/helix/runtime
            if test -d $runtime_path
                set -gx HELIX_RUNTIME $runtime_path
            end
        end
    end

    command hx $argv
end

# Function to detect Helix mode from yazelix.nix configuration
function detect_helix_mode --description "Detect Helix mode from yazelix.nix configuration"
    # Only run if environment variables are not already set
    if test -z $YAZELIX_HELIX_MODE
        set -l yazelix_config "$HOME/.config/yazelix/yazelix.nix"
        set -l default_config "$HOME/.config/yazelix/yazelix_default.nix"

        set -l config_file
        if test -f $yazelix_config
            set config_file $yazelix_config
        else
            set config_file $default_config
        end

        if test -f $config_file
            # Extract helix_mode from the nix configuration file
            set -l helix_mode_line (grep "helix_mode" $config_file | head -1)
            if test -n $helix_mode_line
                # Extract the mode value from the line
                set -l mode (echo $helix_mode_line | sed 's/helix_mode = //' | sed 's/"//g' | sed 's/;//' | tr -d ' ')

                # Set environment variables based on detected mode
                set -gx YAZELIX_HELIX_MODE $mode
            else
                set -gx YAZELIX_HELIX_MODE "default"
            end
        else
            set -gx YAZELIX_HELIX_MODE "default"
        end
    end
end

# Detect Helix mode on config load
detect_helix_mode