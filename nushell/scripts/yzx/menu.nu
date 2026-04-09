#!/usr/bin/env nu
# yzx menu - Interactive command palette and config opener

use ../integrations/zellij.nu [get_current_tab_workspace_root_including_bootstrap]
use ../integrations/zellij_runtime_wrappers.nu [open_floating_runtime_wrapper]
use ./command_palette_catalog.nu [get_palette_menu_items]
use ../utils/common.nu [get_yazelix_runtime_dir resolve_yazelix_nu_bin]

# In popup mode, pause after most commands so output can be read before closing.
def should_pause_in_popup [cmd: string] {
    not (
        ($cmd | str starts-with "yzx launch")
        or ($cmd | str starts-with "yzx enter")
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
    let yazelix_module = ((get_yazelix_runtime_dir) | path join "nushell" "scripts" "core" "yazelix.nu")
    let runtime_nu = (resolve_yazelix_nu_bin)
    ^$runtime_nu -c $"use ($yazelix_module) *; ($cmd)"
}

# Interactive command palette for Yazelix
export def "yzx menu" [
    --popup  # Open menu in a Zellij floating pane
] {
    if $popup {
        if ($env.ZELLIJ? | is-empty) {
            error make {msg: "Not in a Zellij session; run `yzx menu` directly or start Yazelix/Zellij first."}
        }

        let popup_cwd = ((get_current_tab_workspace_root_including_bootstrap) | default (pwd))
        open_floating_runtime_wrapper "yzx_menu" "yzx_menu_popup.nu" $popup_cwd
        return
    }

    let in_popup = ($env.ZELLIJ_PANE_ID? | is-not-empty) and ($env.YAZELIX_MENU_POPUP? == "true")
    let items = get_palette_menu_items

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
