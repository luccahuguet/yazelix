# Keep managed Yazelix Nushell prompts on one physical terminal line.
#
# Zellij stacked panes collapse inactive panes to one row. Multiline prompts in
# those panes redraw when focus moves through the stack, which looks like Enter
# was pressed because each redraw adds another prompt to scrollback.

let __yazelix_prompt_command = ($env.PROMPT_COMMAND? | default null)
if $__yazelix_prompt_command != null {
    $env.PROMPT_COMMAND = {||
        let rendered = if (($__yazelix_prompt_command | describe) == "closure") {
            do $__yazelix_prompt_command
        } else {
            $__yazelix_prompt_command
        }

        $rendered
            | str trim --right
            | str replace --all (char cr) ""
            | str replace --all (char nl) " "
    }
}

let __yazelix_prompt_command_right = ($env.PROMPT_COMMAND_RIGHT? | default null)
if $__yazelix_prompt_command_right != null {
    $env.PROMPT_COMMAND_RIGHT = {||
        let rendered = if (($__yazelix_prompt_command_right | describe) == "closure") {
            do $__yazelix_prompt_command_right
        } else {
            $__yazelix_prompt_command_right
        }

        $rendered
            | str trim --right
            | str replace --all (char cr) ""
            | str replace --all (char nl) " "
    }
}
