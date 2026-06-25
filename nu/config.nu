source "@carapaceInit@"
source "@zoxideInit@"

$env.PROMPT_COMMAND = { || ^@starship@ prompt --cmd-duration ($env.CMD_DURATION_MS? | default 0) $"--status=($env.LAST_EXIT_CODE? | default 0)" }
$env.PROMPT_COMMAND_RIGHT = { || ^@starship@ prompt --right --cmd-duration ($env.CMD_DURATION_MS? | default 0) $"--status=($env.LAST_EXIT_CODE? | default 0)" }
$env.PROMPT_INDICATOR = ""
$env.PROMPT_INDICATOR_VI_INSERT = ""
$env.PROMPT_INDICATOR_VI_NORMAL = ""
$env.PROMPT_MULTILINE_INDICATOR = "::: "
$env.config.show_banner = false
