# Keep managed Yazelix Nushell prompts stack-safe.
#
# Starship returns the whole prompt as one multiline string. Split that into a
# one-line context prompt plus a newline-prefixed input indicator so transient
# redraws can stay tiny without losing the desired two-line active prompt.

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

def __yazelix_prompt_header_line [rendered: string] {
    let lines = (__yazelix_prompt_lines $rendered)
    let line_count = ($lines | length)
    if $line_count <= 1 {
        return ($lines | str join " ")
    }

    $lines | first ($line_count - 1) | str join " "
}

def __yazelix_prompt_input_line [rendered: string] {
    let lines = (__yazelix_prompt_lines $rendered)
    if ($lines | is-empty) {
        ""
    } else {
        $lines | last
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
        __yazelix_prompt_header_line $rendered
    }

    $env.PROMPT_INDICATOR = {||
        let rendered = (__yazelix_render_prompt_command $__yazelix_prompt_command)
        let input = (__yazelix_prompt_input_line $rendered)
        if ($input | str trim | is-empty) {
            ""
        } else {
            [(char nl) $input] | str join
        }
    }

    $env.TRANSIENT_PROMPT_COMMAND = {||
        let rendered = (__yazelix_render_prompt_command $__yazelix_prompt_command)
        __yazelix_prompt_input_line $rendered
    }
    $env.TRANSIENT_PROMPT_COMMAND_RIGHT = ""
    $env.TRANSIENT_PROMPT_INDICATOR = ""
    $env.PROMPT_INDICATOR_VI_NORMAL = $env.PROMPT_INDICATOR
    $env.PROMPT_INDICATOR_VI_INSERT = $env.PROMPT_INDICATOR
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
