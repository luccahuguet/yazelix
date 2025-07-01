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

# Helix wrapper (ensure runtime is set correctly)
export def --env --wrapped hx [...rest] {
    # Ensure helix config directory exists
    let helix_config_dir = $"($env.HOME)/.config/helix"
    if not ($helix_config_dir | path exists) {
        mkdir $helix_config_dir
    }

    # Set runtime based on mode - both modes need HELIX_RUNTIME set
    # The runtime path is already set by the Nix environment, but ensure it's available
    if ($env.HELIX_RUNTIME? | is-empty) {
        # Fallback: try to find runtime from helix binary
        try {
            let helix_path = (which hx | str trim)
            let runtime_path = ($helix_path | path dirname | path dirname | path join "share/helix/runtime")
            if ($runtime_path | path exists) {
                $env.HELIX_RUNTIME = $runtime_path
            }
        } catch {
            # If we can't find it, helix will use its default
        }
    }

    run-external hx ...$rest
}

# Yazelix aliases
export alias yazelix = ~/.config/yazelix/bash/launch-yazelix.sh
export alias yzx = ~/.config/yazelix/bash/launch-yazelix.sh

# Version info alias
export alias yazelix-versions = nu ~/.config/yazelix/nushell/scripts/utils/version-info.nu




