#!/usr/bin/env nu

use ../utils/common.nu [require_yazelix_runtime_dir]

def get_desktop_applications_dir [] {
    let data_home = (
        $env.XDG_DATA_HOME?
        | default "~/.local/share"
        | into string
        | str trim
    )

    ($data_home | path expand | path join "applications")
}

def quote_desktop_exec_arg [value: string] {
    let escaped = (
        $value
        | str replace -a '\' '\\'
        | str replace -a '"' '\"'
        | str replace -a '$' '\$'
        | str replace -a '`' '\`'
    )

    $"\"($escaped)\""
}

def get_stable_yzx_path [] {
    ($env.HOME | path join ".local" "bin" "yzx")
}

def render_desktop_exec [launcher_path: string] {
    $"(quote_desktop_exec_arg $launcher_path) desktop launch"
}

def render_desktop_entry [launcher_path: string] {
    [
        "[Desktop Entry]"
        "Version=1.4"
        "Type=Application"
        "Name=Yazelix"
        "Comment=Yazi + Zellij + Helix integrated terminal environment"
        "Icon=yazelix"
        "StartupWMClass=com.yazelix.Yazelix"
        $"Exec=(render_desktop_exec $launcher_path)"
        "Categories=Development;"
    ] | str join "\n"
}

def validate_desktop_entry [desktop_path: string] {
    if (which desktop-file-validate | is-empty) {
        return
    }

    let result = (^desktop-file-validate $desktop_path | complete)
    if $result.exit_code != 0 {
        let stderr = ($result.stderr | str trim)
        error make {msg: $"Generated desktop entry failed validation: ($stderr)"}
    }
}

def refresh_desktop_database [applications_dir: string] {
    if (which update-desktop-database | is-empty) {
        return
    }

    ^update-desktop-database $applications_dir | complete | ignore
}

def get_desktop_entry_path [] {
    (get_desktop_applications_dir | path join "com.yazelix.Yazelix.desktop")
}

export def "yzx desktop install" [
    --print-path(-p) # Print only the installed desktop-file path
] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let launcher_path = (get_stable_yzx_path)

    if not ($runtime_dir | path exists) {
        error make {msg: $"Missing Yazelix runtime at ($runtime_dir)"}
    }

    if not ($launcher_path | path exists) {
        error make {msg: $"Missing stable Yazelix CLI at ($launcher_path)"}
    }

    let applications_dir = (get_desktop_applications_dir)
    let desktop_path = (get_desktop_entry_path)
    let desktop_entry = (render_desktop_entry $launcher_path)

    mkdir $applications_dir
    $desktop_entry | save --force --raw $desktop_path
    validate_desktop_entry $desktop_path
    refresh_desktop_database $applications_dir

    if $print_path {
        print $desktop_path
    } else {
        print $"Installed Yazelix desktop entry: ($desktop_path)"
    }
}

export def "yzx desktop uninstall" [
    --print-path(-p) # Print only the desktop-file path that was removed or would be removed
] {
    let applications_dir = (get_desktop_applications_dir)
    let desktop_path = (get_desktop_entry_path)

    if ($desktop_path | path exists) {
        rm $desktop_path
        refresh_desktop_database $applications_dir
    }

    if $print_path {
        print $desktop_path
    } else {
        print $"Removed Yazelix desktop entry: ($desktop_path)"
    }
}

export def "yzx desktop launch" [] {
    let runtime_dir = (require_yazelix_runtime_dir)
    let launcher_script = ($runtime_dir | path join "nushell" "scripts" "core" "desktop_launcher.nu")

    if not ($launcher_script | path exists) {
        error make {msg: $"Missing Yazelix desktop launcher at ($launcher_script)"}
    }

    if ($env.YAZELIX_NU_BIN? | is-not-empty) {
        ^$env.YAZELIX_NU_BIN $launcher_script
    } else {
        ^nu $launcher_script
    }
}
