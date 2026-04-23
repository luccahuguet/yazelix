#!/usr/bin/env nu
# yzx menu - Interactive command palette and config opener

use ../utils/runtime_paths.nu get_yazelix_runtime_dir
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface resolve_yzx_core_helper_path run_yzx_core_json_command get_current_tab_workspace_root run_zellij_pipe]
use ../utils/transient_pane_contract.nu [
    build_transient_pane_open_contract
    close_current_transient_pane
    is_transient_pane_mode_active
]

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

def resolve_menu_popup_contract [
    transient_pane_facts: record
    runtime_dir: string
    workspace_root?: string
    current_dir?: string
] {
    build_transient_pane_open_contract "menu" $runtime_dir ($transient_pane_facts.popup_width_percent? | default 90) ($transient_pane_facts.popup_height_percent? | default 90) $workspace_root $current_dir []
}

# Interactive command palette for Yazelix
export def "yzx menu" [
    --popup  # Open menu in a Zellij floating pane
] {
    if $popup {
        if ($env.ZELLIJ? | is-empty) {
            error make {msg: "Not in a Zellij session; run `yzx menu` directly or start Yazelix/Zellij first."}
        }

        let runtime_dir = (get_yazelix_runtime_dir | path expand)
        let transient_pane_facts = (run_yzx_core_json_command $runtime_dir (build_default_yzx_core_error_surface) [
            "transient-pane-facts.compute"
        ] "Yazelix Rust transient-pane-facts helper returned invalid JSON.")
        let popup_contract = (resolve_menu_popup_contract $transient_pane_facts $runtime_dir ((get_current_tab_workspace_root --include-bootstrap) | default "") (pwd))
        let payload = ({
            kind: ($popup_contract.kind? | default "" | into string | str trim)
            args: ($popup_contract.args? | default [])
            cwd: ($popup_contract.cwd? | default "" | into string)
            runtime_dir: ($popup_contract.runtime_dir? | default "" | into string)
        } | to json -r)
        let open_response = (run_zellij_pipe "open_transient_pane" $payload)
        let open_ok = (match ($open_response | str trim) {
            "ok" | "opened" => true
            _ => false
        })
        if not $open_ok {
            error make {msg: $"Failed to open the Yazelix menu popup pane: ($open_response)"}
        }
        return
    }

    let in_popup = ($env.ZELLIJ_PANE_ID? | is-not-empty) and (is_transient_pane_mode_active "menu")
    let items = get_palette_menu_items

    if $in_popup {
        let result = (try {
            run_popup_palette $items
            {ok: true}
        } catch {|err|
            {ok: false, msg: $err.msg}
        })
        try { close_current_transient_pane }
        if not $result.ok {
            error make {msg: $result.msg}
        }
        return
    } else {
        let entry = (select_with_fzf $items)
        if $entry == null {
            return
        }
        run_menu_action $entry.id
    }
}
