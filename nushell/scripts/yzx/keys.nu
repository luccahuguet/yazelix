#!/usr/bin/env nu
# Keybinding discoverability helpers for Yazelix and related tools

def heading [text: string] {
    print $"(ansi cyan_bold)($text)(ansi reset)"
}

def accent_key [text: string] {
    $"(ansi yellow_bold)($text)(ansi reset)"
}

def accent_cmd [text: string] {
    $"(ansi white)($text)(ansi reset)"
}

def accent_note [text: string] {
    $"(ansi magenta)($text)(ansi reset)"
}

def label [text: string] {
    $"(ansi yellow_bold)($text)(ansi reset)"
}

def print_table [rows: list] {
    print ($rows | table)
}

def print_yazelix_keys [] {
    heading "Yazelix keybindings"
    print ""
    heading "Workspace navigation"
    print_table [
        {keybinding: (accent_key "Ctrl+y"), action: "Toggle focus between the managed editor and sidebar"}
        {keybinding: (accent_key "Alt+y"), action: "Toggle the sidebar open/closed"}
    ]
    heading "Command and mode access"
    print_table [
        {keybinding: (accent_key "Alt+Shift+m"), action: "Open the yzx command palette popup"}
        {keybinding: (accent_key "Ctrl+Alt+g"), action: "Locked mode"}
        {keybinding: (accent_key "Ctrl+Alt+s"), action: "Scroll mode"}
        {keybinding: (accent_key "Ctrl+Alt+o"), action: "Session mode"}
    ]
    heading "Tab and pane movement"
    print_table [
        {keybinding: (accent_key "Alt+w / Alt+q"), action: "Walk next/previous tab"}
        {keybinding: (accent_key "Alt+Shift+H / Alt+Shift+L"), action: "Move current tab left/right"}
        {keybinding: (accent_key "Alt+Shift+f"), action: "Toggle pane fullscreen"}
    ]
    heading "More"
    print $"(label 'Yazi:') (accent_cmd 'yzx keys yazi')"
    print $"(label 'Helix:') (accent_cmd 'yzx keys hx')"
    print $"(label 'Nushell:') (accent_cmd 'yzx keys nu')"
}

def print_yazi_keys [] {
    heading "Yazi keybindings"
    print ""
    print ([
        {
            step: "Open key help"
            action: $"Focus the Yazi pane and press (accent_key '`~`')"
            notes: "Shows Yazi's keybindings and commands"
        }
        {
            step: "Optional"
            action: $"Press (accent_key '`Alt+Shift+f`') first"
            notes: "Fullscreen the pane for easier reading"
        }
    ] | table)
    print $"(label 'For Yazelix-specific bindings:') (accent_cmd 'yzx keys')"
}

def print_helix_keys [] {
    heading "Helix keybindings"
    print ""
    print_table [
        {
            topic: "Browse commands"
            how: $"Press (accent_key '`<space>?`')"
        }
        {
            topic: "Full keymap docs"
            how: (accent_cmd "https://docs.helix-editor.com/master/keymap.html")
        }
    ]
    print_table [
        {
            caveat: "No default Helix-local Yazi binding in Yazelix"
            details: $"Use Zellij-level (accent_key '`Ctrl+y`') and (accent_key '`Alt+y`') for managed workspace navigation"
        }
    ]
    print $"(label 'For Yazelix-specific bindings:') (accent_cmd 'yzx keys')"
}

def print_nushell_keys [] {
    heading "Nushell keybindings"
    print ""
    print_table [
        {keybinding: (accent_key "Ctrl+r"), action: "Search shell history", notes: ""}
        {keybinding: (accent_key "Ctrl+f"), action: "Complete the current history hint", notes: (accent_note "Different from Tab completion")}
        {keybinding: (accent_key "Ctrl+o"), action: "Open the current command in your editor", notes: ""}
        {keybinding: (accent_key "Alt+Enter"), action: "Insert a newline without executing", notes: ""}
    ]
    heading "More"
    print $"(label 'Guided intro:') run (accent_key '`tutor`') inside Nushell"
    print $"(label 'Full reference:') (accent_cmd 'https://www.nushell.sh/book/line_editor.html')"
    print $"(label 'For Yazelix-specific bindings:') (accent_cmd 'yzx keys')"
}

# Show Yazelix-owned keybindings and remaps.
export def "yzx keys" [] {
    print_yazelix_keys
}

# Alias for the default Yazelix keybinding view.
export def "yzx keys yzx" [] {
    print_yazelix_keys
}

# Explain how to view Yazi's built-in keybindings.
export def "yzx keys yazi" [] {
    print_yazi_keys
}

# Explain how to discover Helix keybindings and commands.
export def "yzx keys hx" [] {
    print_helix_keys
}

# Alias for `yzx keys hx`.
export def "yzx keys helix" [] {
    print_helix_keys
}

# Show a small curated subset of useful Nushell keybindings.
export def "yzx keys nu" [] {
    print_nushell_keys
}

# Alias for `yzx keys nu`.
export def "yzx keys nushell" [] {
    print_nushell_keys
}
