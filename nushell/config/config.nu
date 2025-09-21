# Minimal Nushell config for Yazelix
# Location: ~/.config/yazelix/nushell/config.nu

# Disable Nushell welcome banner
$env.config.show_banner = false

# Set Helix mode environment variables if not already set
if ($env.YAZELIX_HELIX_MODE? | is-empty) {
    use ../scripts/utils/helix_mode.nu set_helix_env
    set_helix_env
}

# Initializes programs from XDG-compliant state directory (only if files exist)
let init_dir = "~/.local/share/yazelix/initializers/nushell"

let starship_init = $"($init_dir)/starship_init.nu"
if ($starship_init | path exists) {
    source $starship_init
}

let zoxide_init = $"($init_dir)/zoxide_init.nu"
if ($zoxide_init | path exists) {
    source $zoxide_init
}

# Optional tools (generated only when available/enabled)
let mise_init = $"($init_dir)/mise_init.nu"
if ($mise_init | path exists) {
    source $mise_init
}

let carapace_init = $"($init_dir)/carapace_init.nu"
if ($carapace_init | path exists) {
    source $carapace_init
}

# Atuin history manager (optional)
let atuin_init = $"($init_dir)/atuin_init.nu"
if ($atuin_init | path exists) {
    source $atuin_init
}

# Sources the `clip` command
use ~/.config/yazelix/nushell/modules/system *

# Tools aliases
export alias lg = lazygit

# Note: yazelix commands are available directly from the script
# Examples: yazelix help, yazelix get_config, yazelix versions, etc.

# Yazelix command suite
use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *



