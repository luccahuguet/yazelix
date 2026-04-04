#!/usr/bin/env nu
# yzx menu - Interactive command palette and config opener

use ../integrations/zellij.nu [get_current_tab_workspace_root_including_bootstrap open_floating_runtime_wrapper resolve_tab_cwd_target set_tab_workspace_root]
use ../integrations/yazi.nu [sync_active_sidebar_yazi_to_directory sync_managed_editor_cwd]
use ./command_palette_catalog.nu [get_palette_menu_items]
use ../utils/common.nu get_yazelix_runtime_dir

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

def prompt_for_cwd_target [] {
    let target = (input "yzx cwd (path or zoxide query, blank=current dir)> " | str trim)
    if ($target | is-empty) { pwd } else { $target }
}

def run_menu_cwd_action [] {
    if ($env.ZELLIJ? | is-empty) {
        error make {msg: "yzx cwd only works inside Zellij. Start Yazelix first, then run it from the tab you want to update."}
    }

    let resolved_dir = try {
        resolve_tab_cwd_target (prompt_for_cwd_target)
    } catch {|err|
        error make {msg: $err.msg}
    }

    let result = (set_tab_workspace_root $resolved_dir "yzx_menu_cwd.log")

    match $result.status {
        "ok" => {
            let editor_sync_result = (sync_managed_editor_cwd $result.workspace_root "yzx_menu_cwd.log")
            let sidebar_sync_result = (sync_active_sidebar_yazi_to_directory $result.workspace_root "yzx_menu_cwd.log")
            print $"✅ Updated current tab workspace directory to: ($result.workspace_root)"
            print $"   Tab renamed to: ($result.tab_name)"
            print "   Existing panes keep their current working directories."
            print "   New managed actions will use the updated tab directory."
            if $editor_sync_result.status == "ok" {
                print "   Managed editor cwd synced to the updated directory."
            }
            if $sidebar_sync_result.status == "ok" {
                print "   Sidebar Yazi synced to the updated directory."
            }
        }
        "not_ready" => {
            error make {msg: "Yazelix tab state is not ready yet. Wait a moment for the pane orchestrator plugin to finish loading, then try again."}
        }
        "permissions_denied" => {
            error make {msg: "The Yazelix pane orchestrator plugin is missing required Zellij permissions. Reload the Yazelix session and try again."}
        }
        _ => {
            let reason = ($result.reason? | default "unknown error")
            error make {msg: $"Failed to update the current tab workspace directory: ($reason)"}
        }
    }
}

def run_menu_action [cmd: string] {
    if $cmd == "yzx cwd" {
        run_menu_cwd_action
        return
    }

    let yazelix_module = ((get_yazelix_runtime_dir) | path join "nushell" "scripts" "core" "yazelix.nu")
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
