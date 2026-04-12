#!/usr/bin/env nu

use ../utils/atomic_writes.nu write_text_atomic
use ../utils/common.nu get_yazelix_runtime_dir
use ../utils/install_ownership.nu has_home_manager_managed_install
use ../utils/launcher_resolution.nu resolve_desktop_launcher_path

const DESKTOP_LAUNCH_CLEARED_ENV_KEYS = [
    "IN_YAZELIX_SHELL"
    "YAZELIX_DIR"
    "YAZELIX_MENU_POPUP"
    "YAZELIX_POPUP_PANE"
    "YAZELIX_NU_BIN"
    "YAZELIX_TERMINAL"
    "YAZI_ID"
    "ZELLIJ"
    "ZELLIJ_DEFAULT_LAYOUT"
    "ZELLIJ_PANE_ID"
    "ZELLIJ_SESSION_NAME"
    "ZELLIJ_TAB_NAME"
    "ZELLIJ_TAB_POSITION"
]
const DESKTOP_ICON_SIZES = ["48x48", "64x64", "128x128", "256x256"]

def get_xdg_data_home [] {
    let data_home = (
        $env.XDG_DATA_HOME?
        | default ""
        | into string
        | str trim
    )

    if ($data_home | is-not-empty) {
        $data_home | path expand
    } else if (($env.HOME? | default "" | into string | str trim) | is-not-empty) {
        $env.HOME | path join ".local" "share"
    } else {
        "~/.local/share" | path expand
    }
}

def get_desktop_applications_dir [] {
    (get_xdg_data_home | path join "applications")
}

def get_desktop_icons_root [] {
    (get_xdg_data_home | path join "icons" "hicolor")
}

def get_desktop_icon_path [size: string] {
    (get_desktop_icons_root | path join $size "apps" "yazelix.png")
}

def get_runtime_icon_source_path [runtime_dir: string, size: string] {
    ($runtime_dir | path join "assets" "icons" $size "yazelix.png")
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
        "X-Yazelix-Managed=true"
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

def refresh_icon_cache [icons_root: string] {
    if (which gtk-update-icon-cache | is-empty) {
        return
    }

    ^gtk-update-icon-cache --force --ignore-theme-index $icons_root | complete | ignore
}

def get_desktop_entry_path [] {
    (get_desktop_applications_dir | path join "com.yazelix.Yazelix.desktop")
}

def get_desktop_icon_entries [runtime_dir: string] {
    $DESKTOP_ICON_SIZES
    | each {|size|
        {
            size: $size
            source: (get_runtime_icon_source_path $runtime_dir $size)
            destination: (get_desktop_icon_path $size)
        }
    }
}

def install_desktop_icons [runtime_dir: string] {
    let icon_entries = (get_desktop_icon_entries $runtime_dir)
    let missing_sources = (
        $icon_entries
        | where {|entry| not ($entry.source | path exists) }
        | each {|entry| $entry.source }
    )
    if not ($missing_sources | is-empty) {
        let missing_text = ($missing_sources | str join "\n")
        error make {msg: $"Missing Yazelix desktop icon assets:\n($missing_text)"}
    }

    for entry in $icon_entries {
        mkdir ($entry.destination | path dirname)
        ^cp --force $entry.source $entry.destination
    }
}

def uninstall_desktop_icons [] {
    for size in $DESKTOP_ICON_SIZES {
        let icon_path = (get_desktop_icon_path $size)
        if ($icon_path | path exists) {
            rm $icon_path
        }
    }
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
    if (has_home_manager_managed_install) {
        error make {msg: "Home Manager owns Yazelix desktop integration for this install. Reapply your Home Manager configuration for the profile desktop entry, or run `yzx desktop uninstall` only to remove a stale user-local entry."}
    }

    let runtime_dir = (get_yazelix_runtime_dir)
    if $runtime_dir == null {
        error make {msg: "Cannot resolve a Yazelix runtime root for desktop integration."}
    }
    let launcher_path = (resolve_desktop_launcher_path $runtime_dir)

    if not ($runtime_dir | path exists) {
        error make {msg: $"Missing Yazelix runtime at ($runtime_dir)"}
    }

    if not ($launcher_path | path exists) {
        error make {msg: $"Missing stable Yazelix CLI at ($launcher_path)"}
    }

    let applications_dir = (get_desktop_applications_dir)
    let icons_root = (get_desktop_icons_root)
    let desktop_path = (get_desktop_entry_path)
    let desktop_entry = (render_desktop_entry $launcher_path)

    mkdir $applications_dir
    install_desktop_icons $runtime_dir
    write_text_atomic $desktop_path $desktop_entry --raw | ignore
    validate_desktop_entry $desktop_path
    refresh_desktop_database $applications_dir
    refresh_icon_cache $icons_root

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
    let icons_root = (get_desktop_icons_root)
    let desktop_path = (get_desktop_entry_path)

    if ($desktop_path | path exists) {
        rm $desktop_path
    }
    uninstall_desktop_icons
    refresh_desktop_database $applications_dir
    refresh_icon_cache $icons_root

    if $print_path {
        print $desktop_path
    } else {
        print $"Removed Yazelix desktop entry: ($desktop_path)"
    }
}

export def "yzx desktop launch" [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    if $runtime_dir == null {
        error make {msg: "Cannot resolve a Yazelix runtime root for desktop launch."}
    }
    let fast_launch_module = ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
    let launch_env = (get_desktop_launch_env $runtime_dir)
    let runtime_nu = ($runtime_dir | path join "bin" "nu")
    let nu_bin = if ($runtime_nu | path exists) {
        $runtime_nu
    } else {
        "nu"
    }

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
    error make {msg: (if ($stderr | is-not-empty) { $stderr } else { $fast_launch.stdout | str trim })}
}
