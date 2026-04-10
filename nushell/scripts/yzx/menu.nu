#!/usr/bin/env nu
# yzx menu - Interactive command palette and config opener

use ../integrations/zellij.nu [get_current_tab_workspace_root_including_bootstrap]
use ../integrations/zellij_runtime_wrappers.nu [open_floating_runtime_wrapper]
use ../utils/common.nu [get_yazelix_runtime_dir resolve_yazelix_nu_bin]

const PALETTE_CATEGORY_STYLE = {
    session: (ansi green)
    workspace: (ansi cyan)
    config: (ansi blue)
    system: (ansi yellow)
    help: (ansi purple)
}

const PALETTE_EXCLUDED_COMMANDS = [
    "yzx menu"
    "yzx menu --popup"
    "yzx env"
    "yzx run"
    "yzx cwd"
]

const PUBLIC_YZX_COMMAND_CATALOG = [
    {id: "yzx", category: "help", description: ""}
    {id: "yzx config", category: "config", description: "Show the active Yazelix configuration"}
    {id: "yzx config migrate", category: "config", description: "Preview or apply known Yazelix config migrations"}
    {id: "yzx config reset", category: "config", description: ""}
    {id: "yzx cwd", category: "workspace", description: "Retarget the current tab workspace root via a path or zoxide query."}
    {id: "yzx desktop install", category: "system", description: ""}
    {id: "yzx desktop launch", category: "system", description: ""}
    {id: "yzx desktop uninstall", category: "system", description: ""}
    {id: "yzx doctor", category: "system", description: "Run health checks and diagnostics"}
    {id: "yzx edit", category: "config", description: ""}
    {id: "yzx edit config", category: "config", description: ""}
    {id: "yzx edit packs", category: "config", description: ""}
    {id: "yzx enter", category: "session", description: "Start Yazelix in the current terminal"}
    {id: "yzx env", category: "system", description: "Load yazelix environment without UI"}
    {id: "yzx gc", category: "system", description: "Garbage collection for Nix store"}
    {id: "yzx home_manager", category: "system", description: "Home Manager takeover helpers for Yazelix-owned paths."}
    {id: "yzx home_manager prepare", category: "system", description: "Preview or archive manual-install artifacts before Home Manager takeover."}
    {id: "yzx import", category: "config", description: "Import native config files into Yazelix-managed override paths."}
    {id: "yzx import helix", category: "config", description: "Import the native Helix config into Yazelix-managed overrides."}
    {id: "yzx import yazi", category: "config", description: "Import native Yazi config files into Yazelix-managed overrides."}
    {id: "yzx import zellij", category: "config", description: "Import the native Zellij config into Yazelix-managed overrides."}
    {id: "yzx keys", category: "help", description: "Show Yazelix-owned keybindings and remaps."}
    {id: "yzx keys helix", category: "help", description: "Alias for `yzx keys hx`."}
    {id: "yzx keys hx", category: "help", description: "Explain how to discover Helix keybindings and commands."}
    {id: "yzx keys nu", category: "help", description: "Show a small curated subset of useful Nushell keybindings."}
    {id: "yzx keys nushell", category: "help", description: "Alias for `yzx keys nu`."}
    {id: "yzx keys yazi", category: "help", description: "Explain how to view Yazi's built-in keybindings."}
    {id: "yzx keys yzx", category: "help", description: "Alias for the default Yazelix keybinding view."}
    {id: "yzx launch", category: "session", description: "Launch yazelix"}
    {id: "yzx menu", category: "help", description: "Interactive command palette for Yazelix"}
    {id: "yzx packs", category: "system", description: "Show packs and their sizes"}
    {id: "yzx popup", category: "workspace", description: ""}
    {id: "yzx repair", category: "system", description: ""}
    {id: "yzx repair zellij-permissions", category: "system", description: ""}
    {id: "yzx restart", category: "session", description: "Restart yazelix"}
    {id: "yzx reveal", category: "workspace", description: ""}
    {id: "yzx run", category: "system", description: "Run a command in the Yazelix environment and exit"}
    {id: "yzx screen", category: "workspace", description: "Preview the animated welcome screen directly in the current terminal."}
    {id: "yzx sponsor", category: "help", description: ""}
    {id: "yzx status", category: "system", description: "Canonical inspection command"}
    {id: "yzx tutor", category: "help", description: "Show the Yazelix guided overview."}
    {id: "yzx tutor helix", category: "help", description: "Alias for `yzx tutor hx`."}
    {id: "yzx tutor hx", category: "help", description: "Launch Helix's built-in tutorial."}
    {id: "yzx tutor nu", category: "help", description: "Launch Nushell's built-in tutorial in a fresh Nushell process."}
    {id: "yzx tutor nushell", category: "help", description: "Alias for `yzx tutor nu`."}
    {id: "yzx update", category: "system", description: "Choose the Yazelix update owner path"}
    {id: "yzx update upstream", category: "system", description: "Refresh Yazelix from the upstream installer surface"}
    {id: "yzx update home_manager", category: "system", description: "Refresh the current Home Manager flake input and print the switch step"}
    {id: "yzx update nix", category: "system", description: ""}
    {id: "yzx whats_new", category: "help", description: ""}
    {id: "yzx why", category: "help", description: "Elevator pitch: Why Yazelix"}
]

def format_palette_item [entry: record] {
    let category = ($entry.category | into string)
    let color = ($PALETTE_CATEGORY_STYLE | get $category)
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

def is_palette_eligible_command [cmd: string] {
    let normalized = ($cmd | into string | str trim)
    not (
        ($normalized in $PALETTE_EXCLUDED_COMMANDS)
        or ($normalized | str starts-with "yzx dev")
    )
}

def get_palette_menu_items [] {
    $PUBLIC_YZX_COMMAND_CATALOG
    | where {|entry| is_palette_eligible_command $entry.id }
    | sort-by id
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
