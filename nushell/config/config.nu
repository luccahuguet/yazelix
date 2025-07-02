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

# Yazelix command function
export def yazelix [command: string = "help", ...args] {
    use ~/.config/yazelix/nushell/scripts/yazelix.nu *
    match $command {
        "help" => { help }
        "get_config" => {
            if ($args | is-empty) {
                get_config
            } else {
                get_config ($args | get 0)
            }
        }
        "check_config" => { check_config }
        "config_status" => {
            if ($args | is-empty) {
                config_status
            } else {
                config_status ($args | get 0)
            }
        }
        "versions" => { versions }
        "version" => { version }
        "info" => { info }
        "launch" => { launch }
        "start" => { start }
        _ => {
            print "❌ Unknown command: ($command)"
            print ""
            help
        }
    }
}

export alias yzx = yazelix

# Yazelix command suite
use ~/.config/yazelix/nushell/scripts/yazelix.nu *




