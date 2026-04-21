#!/usr/bin/env nu

use ../utils/atomic_writes.nu write_text_atomic
use ../utils/common.nu get_yazelix_runtime_dir
use ../utils/startup_profile.nu [profile_startup_step propagate_startup_profile_env]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface run_yzx_core_json_command]

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
const INSTALL_OWNERSHIP_EVALUATE_COMMAND = "install-ownership.evaluate"

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
        "Terminal=true"
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

def evaluate_install_ownership_report [runtime_dir: string] {
    run_yzx_core_json_command $runtime_dir (
        build_default_yzx_core_error_surface
    ) [
        $INSTALL_OWNERSHIP_EVALUATE_COMMAND
        "--from-env"
        "--runtime-dir"
        ($runtime_dir | path expand)
    ] "Yazelix Rust install-ownership helper returned invalid JSON."
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
    let clean_env = ($DESKTOP_LAUNCH_CLEARED_ENV_KEYS
    | reduce -f {YAZELIX_RUNTIME_DIR: $runtime_dir} {|key, env_record|
        $env_record | upsert $key null
    })

    propagate_startup_profile_env $clean_env
}

# Install the user-local Yazelix desktop entry and icons
export def "yzx desktop install" [
    --print-path(-p) # Print only the installed desktop-file path
] {
    let runtime_dir = (get_yazelix_runtime_dir)
    if $runtime_dir == null {
        error make {msg: "Cannot resolve a Yazelix runtime root for desktop integration."}
    }
    let install_report = (evaluate_install_ownership_report $runtime_dir)
    if $install_report.install_owner == "home-manager" {
        error make {msg: "Home Manager owns Yazelix desktop integration for this install. Reapply your Home Manager configuration for the profile desktop entry, or run `yzx desktop uninstall` only to remove a stale user-local entry."}
    }
    let launcher_path = $install_report.desktop_launcher_path

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

# Remove the user-local Yazelix desktop entry and icons
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

def print_desktop_progress [message: string] {
    print $"Yazelix: ($message)"
}

def acknowledge_desktop_failure [error_text: string] {
    print $""
    print $"Yazelix: Launch failed."
    print $""
    print $error_text
    print $""
    print "Press Enter to close this window."
    try { input } catch { null }
}

# Launch Yazelix from the desktop entry fast path
export def "yzx desktop launch" [] {
    let runtime_dir = (profile_startup_step "desktop" "resolve_runtime_dir" {
        get_yazelix_runtime_dir
    })
    if $runtime_dir == null {
        acknowledge_desktop_failure "Cannot resolve a Yazelix runtime root for desktop launch."
        error make {msg: "Cannot resolve a Yazelix runtime root for desktop launch."}
    }
    let fast_launch_module = ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
    let launch_env = (profile_startup_step "desktop" "build_launch_env" {
        get_desktop_launch_env $runtime_dir
    })
    let runtime_nu = ($runtime_dir | path join "libexec" "nu")
    let nu_bin = if ($runtime_nu | path exists) {
        $runtime_nu
    } else {
        "nu"
    }

    if not ($fast_launch_module | path exists) {
        acknowledge_desktop_failure $"Missing Yazelix desktop launch module at ($fast_launch_module)"
        error make {msg: $"Missing Yazelix desktop launch module at ($fast_launch_module)"}
    }

    print_desktop_progress "Preparing session..."

    let fast_launch = profile_startup_step "desktop" "fast_path_handoff" {
        with-env $launch_env {
            ^$nu_bin $fast_launch_module $env.HOME --desktop-fast-path | complete
        }
    }

    if ($fast_launch.exit_code == 0) {
        return
    }

    let stderr = ($fast_launch.stderr | str trim)
    let error_text = (if ($stderr | is-not-empty) { $stderr } else { $fast_launch.stdout | str trim })
    acknowledge_desktop_failure $error_text
    error make {msg: $error_text}
}
