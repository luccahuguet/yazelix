#!/usr/bin/env nu
# yzx menu - Interactive command palette and config opener

use ../utils/runtime_paths.nu get_yazelix_runtime_dir
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface resolve_yzx_core_helper_path run_yzx_core_json_command]

const PALETTE_CATEGORY_STYLE = {
    session: (ansi green)
    workspace: (ansi cyan)
    config: (ansi blue)
    system: (ansi yellow)
    help: (ansi purple)
}

def format_palette_item [entry: record] {
    let category = ($entry.category | into string)
    let color = ($PALETTE_CATEGORY_STYLE | get -o $category | default (ansi reset))
    let tag = $"($color)[($category)](ansi reset)"
    let description = ($entry.description | default "" | str trim)

    {
        id: $entry.id
        label: (if ($description | is-empty) {
            $"($entry.id)  ($tag)"
        } else {
            $"($entry.id)  ($tag)  (ansi dark_gray)- ($description)(ansi reset)"
        })
    }
}

def fetch_yzx_command_catalog [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let helper_path = (resolve_yzx_core_helper_path $runtime_dir)
    let probe = (^$helper_path yzx-command-metadata.list | complete)

    if $probe.exit_code != 0 {
        let stderr = ($probe.stderr | default "" | str trim)
        error make {msg: $"Failed to inspect Rust-owned yzx command metadata for the command palette: ($stderr)"}
    }

    let envelope = ($probe.stdout | from json)
    if (($envelope.status? | default "") != "ok") {
        error make {msg: $"Rust-owned yzx command metadata returned a non-ok envelope: (($probe.stdout | str trim))"}
    }

    $envelope.data.commands | default []
}

def palette_description_for_command [command: record] {
    [
        ($command.extra_description? | default "" | str trim)
        ($command.description? | default "" | str trim)
    ]
    | where {|value| $value | is-not-empty }
    | get -o 0
    | default ""
}

def get_palette_command_entries [] {
    fetch_yzx_command_catalog
    | where {|command| (($command.menu_category? | default "") | str trim | is-not-empty) }
    | each {|command|
        {
            id: $command.name
            category: ($command.menu_category | into string)
            description: (palette_description_for_command $command)
        }
    }
    | sort-by id
}

def get_palette_menu_items [] {
    get_palette_command_entries
    | each {|entry| format_palette_item $entry }
}

# In popup mode, pause after most commands so output can be read before closing.
def should_pause_in_popup [cmd: string] {
    not (
        ($cmd | str starts-with "yzx launch")
        or ($cmd | str starts-with "yzx enter")
        or ($cmd | str starts-with "yzx env")
        or ($cmd | str starts-with "yzx restart")
    )
}

def select_with_fzf [items: list<record>] {
    let result = (
        $items
        | get label
        | str join "\n"
        | ^fzf --ansi --border rounded
            --header "  Yazelix Command Palette"
            --prompt "  yzx> "
            --pointer "▸"
            --layout reverse
            --cycle
            --color "border:blue,header:bold:blue,prompt:bold:yellow,pointer:bold:cyan,hl:bold:magenta,hl+:bold:magenta,info:dim"
        | complete
    )
    if $result.exit_code != 0 {
        return null
    }
    let selected = ($result.stdout | ansi strip | str trim)
    $items | where {|item| ($item.label | ansi strip) == $selected} | first
}

def popup_post_action_prompt [] {
    "Backspace: return to menu | Enter: close"
}

def popup_post_action_key_decision [code: string] {
    match $code {
        "backspace" => "menu"
        "enter" => "close"
        _ => "continue"
    }
}

def popup_post_action_decision [] {
    print ""
    print (popup_post_action_prompt)
    loop {
        let event = (input listen --types [key])
        let code = ($event.code? | default "")
        match (popup_post_action_key_decision $code) {
            "menu" => {
                clear
                return "menu"
            }
            "close" => {
                return "close"
            }
            _ => {}
        }
    }
}

def run_popup_palette [items: list<record>] {
    loop {
        let entry = (select_with_fzf $items)
        if $entry == null {
            return
        }

        run_menu_action $entry.id

        if (should_pause_in_popup $entry.id) {
            if (popup_post_action_decision) == "menu" {
                continue
            }
        }

        return
    }
}

def run_menu_action [cmd: string] {
    let yzx_cli = ((get_yazelix_runtime_dir) | path join "shells" "posix" "yzx_cli.sh")
    let args = ($cmd | str trim | split row " " | skip 1)
    ^sh $yzx_cli ...$args
}

# Interactive command palette for Yazelix
export def "yzx menu" [
    --popup  # Open menu in a Zellij floating pane
    --pane  # Run the popup-pane menu UI in the current pane
] {
    if $popup and $pane {
        error make {msg: "Use either `yzx menu --popup` or `yzx menu --pane`, not both."}
    }

    if $popup {
        if ($env.ZELLIJ? | is-empty) {
            error make {msg: "Not in a Zellij session; run `yzx menu` directly or start Yazelix/Zellij first."}
        }

        let open_response = (^zellij action pipe --plugin yzpp --name toggle -- menu | complete)
        if $open_response.exit_code != 0 {
            error make {msg: ($open_response.stderr | default "" | str trim)}
        }
        let open_stdout = ($open_response.stdout | default "" | str trim)
        let open_ok = (match $open_stdout {
            "ok" | "opened" => true
            "focused" | "closed" => true
            _ => false
        })
        if not $open_ok {
            error make {msg: $"Failed to open the Yazelix menu popup pane: ($open_stdout)"}
        }
        return
    }

    let items = get_palette_menu_items

    if $pane {
        run_popup_palette $items
        return
    }

    let entry = (select_with_fzf $items)
    if $entry == null {
        return
    }
    run_menu_action $entry.id
}
