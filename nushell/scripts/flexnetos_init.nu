# FlexNetOS interactive behavior layered onto Yazelix Nova's packaged Nushell.

$env.config.show_banner = false
$env.PROMPT_COMMAND_RIGHT = {|| "" }

export alias lg = lazygit
export def clp [] { clip copy }
