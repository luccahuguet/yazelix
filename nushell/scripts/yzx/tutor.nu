#!/usr/bin/env nu
# Guided tutors for Yazelix and selected upstream tools

def heading [text: string] {
    print $"(ansi cyan_bold)($text)(ansi reset)"
}

def accent [text: string] {
    $"(ansi yellow_bold)($text)(ansi reset)"
}

def command_label [text: string] {
    $"(ansi white)($text)(ansi reset)"
}

def get_external_command_path [command_name: string] {
    let command_path = (
        which $command_name
        | where type == "external"
        | get path
        | first
    )

    if ($command_path | is-empty) {
        error make {msg: $"Required command not found: ($command_name)"}
    }

    $command_path
}

def print_yazelix_tutor [] {
    heading "Yazelix tutor"
    print ""
    print "Yazelix is a managed terminal workspace built around Zellij, Yazi, and Helix."
    print "The important unit is the current tab workspace root: managed actions use that directory unless a tool is doing something more specific."
    print ""
    heading "Start here"
    print $"1. Launch a session with (command_label 'yzx launch') or (command_label 'yzx launch --here')."
    print $"2. Learn the workspace-critical bindings with (command_label 'yzx keys')."
    print $"3. Use (command_label 'yzx cwd <dir>') when you want to retarget the current tab manually. Opening a file from Yazi into the managed editor also moves the workspace root to that file's directory."
    print $"4. Use (command_label 'yzx menu') for fuzzy command discovery \(or (command_label 'Alt+Shift+M') inside Yazelix\) and (command_label 'yzx doctor') when behavior looks wrong."
    print ""
    heading "Mental model"
    print $"(accent 'Managed panes:') Yazelix treats the editor/sidebar flow as a coordinated workspace, not just a pile of unrelated panes."
    print $"(accent 'Directory flow:') The current tab root drives new panes, popup commands, and workspace-aware actions."
    print $"(accent 'Discoverability:') (command_label 'yzx help') is the command reference, (command_label 'yzx keys') is the keybinding surface, and (command_label 'yzx tutor') is the guided overview."
    print ""
    heading "Next steps"
    print $"(accent 'Helix tutor:') (command_label 'yzx tutor hx')"
    print $"(accent 'Nushell tutor:') (command_label 'yzx tutor nu')"
    print $"(accent 'Command reference:') (command_label 'yzx help')"
    print $"(accent 'Project overview:') (command_label 'README.md')"
}

# Show the Yazelix guided overview.
export def "yzx tutor" [] {
    print_yazelix_tutor
}

# Launch Helix's built-in tutorial.
export def "yzx tutor hx" [] {
    let hx_path = (get_external_command_path "hx")
    ^$hx_path --tutor
}

# Alias for `yzx tutor hx`.
export def "yzx tutor helix" [] {
    yzx tutor hx
}

# Launch Nushell's built-in tutorial in a fresh Nushell process.
export def "yzx tutor nu" [] {
    let nu_path = (get_external_command_path "nu")
    ^$nu_path -c "tutor"
}

# Alias for `yzx tutor nu`.
export def "yzx tutor nushell" [] {
    yzx tutor nu
}
