# Yazelix Fish Configuration
# This file is sourced by ~/.config/fish/config.fish

# Source Helix mode detection using Nushell (essential dependency)
if test -z $YAZELIX_HELIX_MODE
    eval (nu -c 'use ~/.config/yazelix/nushell/scripts/utils/helix_mode.nu export_helix_env; export_helix_env')
end

# Source generated initializers if they exist
set -l FISH_INITIALIZERS_DIR "$HOME/.config/yazelix/fish/initializers"

# Source each initializer if it exists
for file in $FISH_INITIALIZERS_DIR/*.fish
    if test -f $file
        source $file
    end
end

# Yazelix aliases
alias yazelix="$HOME/.config/yazelix/bash/launch-yazelix.sh"
alias yzx="$HOME/.config/yazelix/bash/launch-yazelix.sh"
alias lg='lazygit'

# Patchy Helix function (use patchy-built hx if available)
function hx --description "Helix editor with Yazelix mode support"
    # Ensure helix config directory exists
    set -l helix_config_dir "$HOME/.config/helix"
    if not test -d $helix_config_dir
        mkdir -p $helix_config_dir
    end

    # Use custom Helix if available
    if test -n $YAZELIX_PATCHY_HX -a -f $YAZELIX_PATCHY_HX
        set -l custom_runtime "$HOME/.config/yazelix/helix_patchy/runtime"
        set -gx HELIX_RUNTIME $custom_runtime
        $YAZELIX_PATCHY_HX $argv
    else
        command hx $argv
    end
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
                if test "$mode" = "steel" -o "$mode" = "patchy" -o "$mode" = "source"
                    set -gx YAZELIX_HELIX_MODE $mode
                    set -gx YAZELIX_PATCHY_HX "$HOME/.config/yazelix/helix_patchy/target/release/hx"
                else
                    set -gx YAZELIX_HELIX_MODE $mode
                end
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