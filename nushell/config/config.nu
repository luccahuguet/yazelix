# Minimal Nushell config for Yazelix
# Location: ~/.config/yazelix/nushell/config.nu

# Disable Nushell welcome banner
$env.config.show_banner = false

# Set Helix mode environment variables if not already set
if ($env.YAZELIX_HELIX_MODE? | is-empty) {
    use ../scripts/utils/helix_mode.nu set_helix_env
    set_helix_env
}

# Initializes some programs from XDG-compliant state directory
source ~/.local/share/yazelix/initializers/nushell/starship_init.nu
source ~/.local/share/yazelix/initializers/nushell/zoxide_init.nu
source ~/.local/share/yazelix/initializers/nushell/mise_init.nu
source ~/.local/share/yazelix/initializers/nushell/carapace_init.nu

# Sources the `clip` command
use ~/.config/yazelix/nushell/modules/system *

# Tools aliases
export alias lg = lazygit

# Note: yazelix commands are available directly from the script
# Examples: yazelix help, yazelix get_config, yazelix versions, etc.

# Yazelix command suite
use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *




