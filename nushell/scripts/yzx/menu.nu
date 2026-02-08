#!/usr/bin/env nu
# yzx menu - Interactive command palette and config opener

use ../utils/config_parser.nu parse_yazelix_config

def classify_menu_command [cmd: string] {
    if ($cmd | str starts-with "yzx launch") or ($cmd == "yzx restart") {
        {tag: "session", color: (ansi green)}
    } else if ($cmd | str starts-with "yzx config") {
        {tag: "config", color: (ansi cyan)}
    } else if ($cmd | str starts-with "yzx update") or ($cmd | str starts-with "yzx gc") or ($cmd | str starts-with "yzx packs") or ($cmd == "yzx doctor") {
        {tag: "system", color: (ansi yellow)}
    } else if ($cmd == "yzx help") or ($cmd == "yzx why") or ($cmd == "yzx info") or ($cmd == "yzx versions") {
        {tag: "help", color: (ansi blue)}
    } else {
        {tag: "other", color: (ansi purple)}
    }
}

def get_menu_items [] {
    help commands
    | where name =~ '^yzx( |$)'
    | where name != "yzx"
    | where name != "yzx menu"
    | where name != "yzx menu --popup"
    | where not ($it.name | str starts-with "yzx sweep")
    | where not ($it.name | str starts-with "yzx dev")
    | where $it.name != "yzx env"
    | where $it.name != "yzx bench"
    | where $it.name != "yzx config_status"
    # TODO: Move `yzx lint` under `yzx dev` and remove the top-level command.
    | where $it.name != "yzx lint"
    | where $it.name != "yzx profile"
    | where $it.name != "yzx test"
    | where $it.name != "yzx run"
    | sort-by name
    | each {|row|
        let semantic = classify_menu_command $row.name
        let tag = $"($semantic.color)[($semantic.tag)](ansi reset)"
        let description = ($row.description | default "" | str replace -a "\n" " " | str trim)
        {
            id: $row.name
            label: (if ($description | is-empty) {
                $"($row.name)  ($tag)"
            } else {
                $"($row.name)  ($tag)  (ansi dark_gray)- ($description)(ansi reset)"
            })
        }
    }
}

# In popup mode, pause after most commands so output can be read before closing.
def should_pause_in_popup [cmd: string] {
    not (
        ($cmd | str starts-with "yzx launch")
        or ($cmd | str starts-with "yzx env")
        or ($cmd | str starts-with "yzx restart")
    )
}

def popup_post_action_decision [] {
    print ""
    print "Backspace: return to menu | Enter/Esc: close"
    loop {
        let event = (input listen --types [key])
        let code = ($event.code? | default "")
        if $code == "backspace" {
            clear
            return "menu"
        }
        if ($code == "enter") or ($code == "esc") {
            return "close"
        }
    }
}

def run_menu_action [cmd: string] {
    let yazelix_module = $"($env.HOME)/.config/yazelix/nushell/scripts/core/yazelix.nu"
    ^nu -c $"use ($yazelix_module) *; ($cmd)"
}

# Interactive command palette for Yazelix
export def "yzx menu" [
    --popup  # Open menu in a Zellij floating pane
] {
    if $popup {
        if ($env.ZELLIJ? | is-empty) {
            error make {msg: "Not in a Zellij session; run `yzx menu` directly or start Yazelix/Zellij first."}
        }

        let wrapper = $"($env.HOME)/.config/yazelix/configs/zellij/scripts/yzx_menu_popup.nu"
        zellij run --name yzx_menu --floating --close-on-exit --width 90% --height 90% --x 5% --y 5% -- nu $wrapper
        return
    }

    let in_popup = ($env.ZELLIJ_PANE_ID? | is-not-empty) and ($env.YAZELIX_MENU_POPUP? == "true")
    let items = get_menu_items

    if $in_popup {
        loop {
            let selected = ($items | get label | input list --fuzzy "yzx menu \(Esc to cancel\)> ")
            if ($selected | is-empty) {
                return
            }

            let entry = ($items | where label == $selected | first)
            run_menu_action $entry.id

            if (should_pause_in_popup $entry.id) {
                if (popup_post_action_decision) == "menu" {
                    continue
                }
            }

            return
        }
    } else {
        let selected = ($items | get label | input list --fuzzy "yzx menu \(Esc to cancel\)> ")
        if ($selected | is-empty) {
            return
        }
        let entry = ($items | where label == $selected | first)
        run_menu_action $entry.id
    }
}

# Open the active Yazelix configuration file in your editor
export def "yzx config open" [
    --print  # Print the config path without opening
] {
    let config = parse_yazelix_config
    let config_path = $config.config_file

    if $print {
        $config_path
    } else if ($env.EDITOR? | is-empty) {
        error make {msg: $"EDITOR is not set. Set it in yazelix.toml under [editor] command, or export EDITOR in your shell.\nConfig path: ($config_path)"}
    } else {
        ^$env.EDITOR $config_path
    }
}
