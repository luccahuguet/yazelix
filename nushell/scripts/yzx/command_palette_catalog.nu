#!/usr/bin/env nu
# Shared command-palette eligibility and grouping for yzx menu.

const PALETTE_CATEGORY_STYLE = {
    session: (ansi green)
    workspace: (ansi cyan)
    config: (ansi blue)
    generated: (ansi magenta)
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
    {id: "yzx cwd", category: "workspace", description: ""}
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
    {id: "yzx open hx", category: "generated", description: ""}
    {id: "yzx open yazi", category: "generated", description: ""}
    {id: "yzx open zellij", category: "generated", description: ""}
    {id: "yzx packs", category: "system", description: "Show packs and their sizes"}
    {id: "yzx popup", category: "workspace", description: ""}
    {id: "yzx repair", category: "system", description: ""}
    {id: "yzx repair zellij-permissions", category: "system", description: ""}
    {id: "yzx restart", category: "session", description: "Restart yazelix"}
    {id: "yzx reveal", category: "workspace", description: ""}
    {id: "yzx run", category: "system", description: "Run a command in the Yazelix environment and exit"}
    {id: "yzx screen", category: "workspace", description: ""}
    {id: "yzx sponsor", category: "help", description: ""}
    {id: "yzx status", category: "system", description: "Canonical inspection command"}
    {id: "yzx tutor", category: "help", description: "Show the Yazelix guided overview."}
    {id: "yzx tutor helix", category: "help", description: "Alias for `yzx tutor hx`."}
    {id: "yzx tutor hx", category: "help", description: "Launch Helix's built-in tutorial."}
    {id: "yzx tutor nu", category: "help", description: "Launch Nushell's built-in tutorial in a fresh Nushell process."}
    {id: "yzx tutor nushell", category: "help", description: "Alias for `yzx tutor nu`."}
    {id: "yzx update", category: "system", description: "Update dependencies and inputs"}
    {id: "yzx update all", category: "system", description: ""}
    {id: "yzx update nix", category: "system", description: ""}
    {id: "yzx update runtime", category: "system", description: ""}
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

export def is_palette_eligible_command [cmd: string] {
    let normalized = ($cmd | into string | str trim)
    not (
        ($normalized in $PALETTE_EXCLUDED_COMMANDS)
        or ($normalized | str starts-with "yzx dev")
    )
}

export def get_palette_menu_items [] {
    $PUBLIC_YZX_COMMAND_CATALOG
    | where {|entry| is_palette_eligible_command $entry.id }
    | sort-by id
    | each {|entry| format_palette_item $entry }
}
