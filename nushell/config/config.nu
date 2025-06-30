# Minimal Nushell config for Yazelix
# Location: ~/.config/yazelix/nushell/config.nu

# Disable Nushell welcome banner
$env.config.show_banner = false

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
    
    # Check helix mode and patchy setting
    let helix_mode = ($env.YAZELIX_HELIX_MODE? | default "default")
    let use_patchy = ($env.YAZELIX_USE_PATCHY_HELIX? | default "false") == "true"
    let use_custom_helix = $use_patchy or ($helix_mode in ["patchy", "steel", "source"])
    
    if $use_custom_helix {
        # First check if YAZELIX_PATCHY_HX is set and valid
        let custom_env = $env.YAZELIX_PATCHY_HX? | default ""
        if ($custom_env | is-not-empty) and ($custom_env | path exists) {
            # Set HELIX_RUNTIME for custom binary to find runtime files
            let custom_runtime = $"($env.HOME)/.config/yazelix/helix_patchy/runtime"
            $env.HELIX_RUNTIME = $custom_runtime
            run-external $custom_env ...$rest
            return
        }
        
        # Fallback: Check for custom binary in default location
        let custom_default = $"($env.HOME)/.config/yazelix/helix_patchy/target/release/hx"
        if ($custom_default | path exists) {
            # Set HELIX_RUNTIME for custom binary to find runtime files
            let custom_runtime = $"($env.HOME)/.config/yazelix/helix_patchy/runtime"
            $env.HELIX_RUNTIME = $custom_runtime
            run-external $custom_default ...$rest
            return
        }
    }
    
    # Fallback to system helix
    run-external "hx" ...$rest
}

# Yazelix aliases
export alias yazelix = ~/.config/yazelix/bash/launch-yazelix.sh
export alias yzx = ~/.config/yazelix/bash/launch-yazelix.sh

# Version info alias
export alias yazelix-versions = nu ~/.config/yazelix/nushell/scripts/utils/version-info.nu

# Patchy Helix management alias
export alias yazelix_patchy = nu ~/.config/yazelix/nushell/scripts/utils/patchy_helix.nu


