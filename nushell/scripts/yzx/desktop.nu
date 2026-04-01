#!/usr/bin/env nu

use ../utils/common.nu [require_yazelix_runtime_dir]

def get_runtime_target [runtime_dir: string] {
    let result = (^readlink -f $runtime_dir | complete)
    if $result.exit_code == 0 {
        $result.stdout | str trim
    } else {
        $runtime_dir | path expand
    }
}

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

def render_desktop_entry [launcher_path: string, runtime_target: string] {
    [
        "[Desktop Entry]"
        "Version=1.4"
        "Type=Application"
        "Name=Yazelix"
        "Comment=Yazi + Zellij + Helix integrated terminal environment"
        "Icon=yazelix"
        "StartupWMClass=com.yazelix.Yazelix"
        $"Exec=(quote_desktop_exec_arg $launcher_path)"
        $"X-Yazelix-Runtime-Target=(quote_desktop_exec_arg $runtime_target)"
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
    let runtime_target = (get_runtime_target $runtime_dir)
    let launcher_path = ($runtime_dir | path join "shells" "posix" "desktop_launcher.sh")

    if not ($launcher_path | path exists) {
        error make {msg: $"Missing Yazelix desktop launcher at ($launcher_path)"}
    }

    let applications_dir = (get_desktop_applications_dir)
    let desktop_path = (get_desktop_entry_path)
    let desktop_entry = (render_desktop_entry $launcher_path $runtime_target)

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
