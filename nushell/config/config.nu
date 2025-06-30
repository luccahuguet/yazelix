# Minimal Nushell config for Yazelix
# Location: ~/.config/yazelix/nushell/config.nu

# Disable Nushell welcome banner
$env.config.show_banner = false

# Set Helix mode environment variables if not already set
if ($env.YAZELIX_HELIX_MODE? | is-empty) {
    use ../scripts/utils/helix_mode.nu set_helix_env
    set_helix_env
}

# Initializes some programs
source ~/.config/yazelix/nushell/initializers/starship_init.nu
source ~/.config/yazelix/nushell/initializers/zoxide_init.nu
source ~/.config/yazelix/nushell/initializers/mise_init.nu
source ~/.config/yazelix/nushell/initializers/carapace_init.nu

# Sources the `clip` command
use ~/.config/yazelix/nushell/modules/system *

# Tools aliases
export alias lg = lazygit

# Patchy Helix wrapper (use patchy-built hx if available)
export def --env --wrapped hx [...rest] {
    # Ensure helix config directory exists
    let helix_config_dir = $"($env.HOME)/.config/helix"
    if not ($helix_config_dir | path exists) {
        mkdir $helix_config_dir
    }
    
    # Get the appropriate Helix binary
    use ../scripts/utils/helix_mode.nu get_helix_binary
    let editor_command = get_helix_binary
    
    # Set runtime for custom builds
    if ($editor_command != "hx") {
        let custom_runtime = $"($env.HOME)/.config/yazelix/helix_patchy/runtime"
        $env.HELIX_RUNTIME = $custom_runtime
    }
    
    run-external $editor_command ...$rest
}

# Yazelix aliases
export alias yazelix = ~/.config/yazelix/bash/launch-yazelix.sh
export alias yzx = ~/.config/yazelix/bash/launch-yazelix.sh

# Version info alias
export alias yazelix-versions = nu ~/.config/yazelix/nushell/scripts/utils/version-info.nu

# Patchy Helix management alias
export alias yazelix_patchy = nu ~/.config/yazelix/nushell/scripts/utils/patchy_helix.nu


