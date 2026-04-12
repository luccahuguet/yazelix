#!/usr/bin/env nu
# yzx menu - Interactive command palette and config opener

use ../integrations/zellij.nu [get_current_tab_workspace_root_including_bootstrap]
use ../integrations/zellij_runtime_wrappers.nu [open_floating_runtime_script]
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
    "yzx env"
    "yzx run"
    "yzx cwd"
]

const PALETTE_DESCRIPTION_OVERRIDES = {
    "yzx config reset": "Reset managed Yazelix config surfaces back to their defaults."
    "yzx desktop install": "Install or refresh the Yazelix desktop entry and icon assets."
    "yzx desktop launch": "Launch Yazelix through the desktop-entry path."
    "yzx desktop uninstall": "Remove Yazelix-managed desktop entry and icon assets."
    "yzx edit": "Open the managed Yazelix config directory."
    "yzx edit config": "Open the active Yazelix config file."
    "yzx home_manager": "Home Manager takeover helpers for Yazelix-owned paths."
    "yzx home_manager prepare": "Preview or archive manual-install artifacts before Home Manager takeover."
    "yzx popup": "Open a floating terminal tool pane, for example `yzx popup lazygit`."
    "yzx restart": "Restart Yazelix."
    "yzx reveal": "Reveal a path in the managed Yazi sidebar."
    "yzx screen": "Preview the animated welcome screen directly in the current terminal."
    "yzx sponsor": "Show the sponsorship links and support message."
    "yzx update home_manager": "Refresh the current Home Manager input and print the switch step."
    "yzx update nix": "Refresh the runtime lock and print the local install step."
    "yzx update upstream": "Refresh Yazelix from the upstream installer surface."
    "yzx whats_new": "Show the latest release notes."
}

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

def fetch_yzx_command_catalog [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let runtime_nu = (resolve_yazelix_nu_bin)
    let probe = (do {
        cd $runtime_dir
        ^$runtime_nu -c 'source nushell/scripts/core/yazelix.nu; scope commands | where {|command| ($command.name == "yzx") or ($command.name | str starts-with "yzx ")} | sort-by name | select name description extra_description | to json -r' | complete
    })

    if $probe.exit_code != 0 {
        let stderr = ($probe.stderr | default "" | str trim)
        error make {msg: $"Failed to inspect the exported yzx command surface for the command palette: ($stderr)"}
    }

    $probe.stdout | from json
}

def palette_category_for_command [cmd: string] {
    if $cmd == "yzx" {
        "help"
    } else if (
        ($cmd | str starts-with "yzx launch")
        or ($cmd | str starts-with "yzx enter")
        or ($cmd | str starts-with "yzx restart")
    ) {
        "session"
    } else if (
        ($cmd | str starts-with "yzx popup")
        or ($cmd | str starts-with "yzx reveal")
        or ($cmd | str starts-with "yzx screen")
    ) {
        "workspace"
    } else if (
        ($cmd | str starts-with "yzx config")
        or ($cmd | str starts-with "yzx edit")
        or ($cmd | str starts-with "yzx import")
    ) {
        "config"
    } else if (
        ($cmd | str starts-with "yzx keys")
        or ($cmd | str starts-with "yzx tutor")
        or ($cmd | str starts-with "yzx why")
        or ($cmd | str starts-with "yzx whats_new")
        or ($cmd | str starts-with "yzx sponsor")
    ) {
        "help"
    } else {
        "system"
    }
}

def palette_description_for_command [command: record] {
    let id = ($command.name | into string)
    let discovered = (
        [
            ($command.description? | default "" | str trim)
            ($command.extra_description? | default "" | str trim)
        ]
        | where {|value| $value | is-not-empty }
        | get -o 0
        | default ""
    )
    let override = ($PALETTE_DESCRIPTION_OVERRIDES | get -o $id | default "")

    if ($override | is-not-empty) {
        $override
    } else {
        $discovered
    }
}

def get_palette_command_entries [] {
    fetch_yzx_command_catalog
    | where {|command| is_palette_eligible_command $command.name }
    | each {|command|
        {
            id: $command.name
            category: (palette_category_for_command $command.name)
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
        open_floating_runtime_script "yzx_menu" "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu" $popup_cwd
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
