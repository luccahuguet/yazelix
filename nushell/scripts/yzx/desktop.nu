#!/usr/bin/env nu

use ../utils/atomic_writes.nu write_text_atomic
use ../utils/common.nu [require_installed_yazelix_runtime_dir]

const DESKTOP_LAUNCH_CLEARED_ENV_KEYS = [
    "DEVENV_PROFILE"
    "DEVENV_ROOT"
    "IN_NIX_SHELL"
    "IN_YAZELIX_SHELL"
    "YAZELIX_DIR"
    "YAZELIX_MENU_POPUP"
    "YAZELIX_POPUP_PANE"
    "YAZELIX_TERMINAL"
    "YAZI_ID"
    "ZELLIJ"
    "ZELLIJ_DEFAULT_LAYOUT"
    "ZELLIJ_PANE_ID"
    "ZELLIJ_SESSION_NAME"
    "ZELLIJ_TAB_NAME"
    "ZELLIJ_TAB_POSITION"
]

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

def get_desktop_launch_env [runtime_dir: string] {
    $DESKTOP_LAUNCH_CLEARED_ENV_KEYS
    | reduce -f {YAZELIX_RUNTIME_DIR: $runtime_dir} {|key, env_record|
        $env_record | upsert $key null
    }
}

export def "yzx desktop install" [
    --print-path(-p) # Print only the installed desktop-file path
] {
    let runtime_dir = (require_installed_yazelix_runtime_dir)
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
    write_text_atomic $desktop_path $desktop_entry --raw | ignore
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
    let runtime_dir = (require_installed_yazelix_runtime_dir)
    let fast_launch_module = ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
    let launch_env = (get_desktop_launch_env $runtime_dir)
    let resolved_nu_bin = (
        $env.YAZELIX_NU_BIN?
        | default "nu"
        | into string
        | str trim
    )
    let nu_bin = if ($resolved_nu_bin | is-empty) { "nu" } else { $resolved_nu_bin }

    if not ($fast_launch_module | path exists) {
        error make {msg: $"Missing Yazelix desktop launch module at ($fast_launch_module)"}
    }

    let fast_launch = with-env $launch_env {
        ^$nu_bin $fast_launch_module $env.HOME --desktop-fast-path | complete
    }

    if ($fast_launch.exit_code == 0) {
        return
    }

    let stderr = ($fast_launch.stderr | str trim)
    if ($stderr | str contains "Failure class: desktop-bootstrap-unavailable.") {
        let launch_module = ($runtime_dir | path join "nushell" "scripts" "yzx" "launch.nu")
        if not ($launch_module | path exists) {
            error make {msg: $"Missing Yazelix fallback launch module at ($launch_module)"}
        }
        with-env $launch_env {
            ^$nu_bin -c $"use \"($launch_module)\" *; yzx launch --home"
        }
        return
    }

    error make {msg: (if ($stderr | is-not-empty) { $stderr } else { $fast_launch.stdout | str trim })}
}
