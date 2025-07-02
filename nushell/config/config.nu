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

# Note: yazelix commands are available with native subcommand support
# Examples: yazelix help, yazelix get_config, yazelix versions, etc.
# Short alias: yzx help, yzx get_config, yzx versions, etc.






