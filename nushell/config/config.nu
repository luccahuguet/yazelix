# Minimal Nushell config for Yazelix
# Location: ~/.config/yazelix/nushell/config.nu

# Disable Nushell welcome banner
$env.config.show_banner = false

# Initializes some programs
source ~/.config/yazelix/nushell/initializers/starship_init.nu
source ~/.config/yazelix/nushell/initializers/zoxide_init.nu
source ~/.config/yazelix/nushell/initializers/mise_init.nu

# sources the `clip` command
use ~/.config/yazelix/nushell/modules/system *

export alias lg = lazygit
