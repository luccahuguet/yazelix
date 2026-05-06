# Keep managed Yazelix Nushell prompts stack-safe.
#
# Zellij stacked panes collapse inactive panes to one row. A full multiline
# prompt in those panes redraws when focus moves through the stack, which looks
# like Enter was pressed because each redraw adds another prompt to scrollback.
# Normal panes can keep a controlled two-row prompt: context above, input below.

def __yazelix_prompt_lines [rendered: string] {
    $rendered
        | str trim --right
        | str replace --all (char cr) ""
        | split row (char nl)
        | where {|line| ($line | str trim | is-not-empty) }
}

def __yazelix_prompt_flattened [rendered: string] {
    __yazelix_prompt_lines $rendered | str join " "
}

def __yazelix_prompt_for_rows [rendered: string, rows: int] {
    let lines = (__yazelix_prompt_lines $rendered)
    let line_count = ($lines | length)
    if $rows <= 1 or $line_count <= 1 {
        return ($lines | str join " ")
    }

    let header_count = ($line_count - 1)
    let header = ($lines | first $header_count | str join " ")
    let input = ($lines | last)
    if ($header | str trim | is-empty) {
        $input
    } else {
        [$header $input] | str join (char nl)
    }
}

def __yazelix_render_prompt_command [prompt_command: any] {
    if (($prompt_command | describe) == "closure") {
        do $prompt_command
    } else {
        $prompt_command
    }
}

let __yazelix_prompt_command = ($env.PROMPT_COMMAND? | default null)
if $__yazelix_prompt_command != null {
    $env.PROMPT_COMMAND = {||
        let rendered = (__yazelix_render_prompt_command $__yazelix_prompt_command)
        let rows = (try { (term size).rows } catch { 24 })
        __yazelix_prompt_for_rows $rendered $rows
    }

    $env.TRANSIENT_PROMPT_COMMAND = {||
        let rendered = (__yazelix_render_prompt_command $__yazelix_prompt_command)
        __yazelix_prompt_flattened $rendered
    }
    $env.TRANSIENT_PROMPT_INDICATOR = ""
    $env.TRANSIENT_PROMPT_INDICATOR_VI_NORMAL = ""
    $env.TRANSIENT_PROMPT_INDICATOR_VI_INSERT = ""
    $env.TRANSIENT_PROMPT_MULTILINE_INDICATOR = ""
}

let __yazelix_prompt_command_right = ($env.PROMPT_COMMAND_RIGHT? | default null)
if $__yazelix_prompt_command_right != null {
    $env.PROMPT_COMMAND_RIGHT = {||
        let rendered = (__yazelix_render_prompt_command $__yazelix_prompt_command_right)
        __yazelix_prompt_flattened $rendered
    }
}
