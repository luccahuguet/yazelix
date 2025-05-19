# Minimal Nushell config for Yazelix
# Location: ~/.config/yazelix/nushell/config.nu

# Disable Nushell welcome banner
$env.config.show_banner = false

source ~/.config/yazelix/nushell/starship_init.nu
source ~/.config/yazelix/nushell/zoxide_init.nu
source ~/.config/yazelix/nushell/mise_init.nu

export alias lg = lazygit
