#!/usr/bin/env nu

use common.nu [get_yazelix_state_dir]
use config_surfaces.nu get_main_user_config_path

const HOME_MANAGER_FILES_MARKER = "-home-manager-files/"
const MANUAL_DESKTOP_ICON_SIZES = ["48x48", "64x64", "128x128", "256x256"]

def get_xdg_data_home [] {
    let configured = (
        $env.XDG_DATA_HOME?
        | default ""
        | into string
        | str trim
    )

    if ($configured | is-not-empty) {
        $configured | path expand
    } else if (($env.HOME? | default "" | into string | str trim) | is-not-empty) {
        $env.HOME | path join ".local" "share"
    } else {
        "~/.local/share" | path expand
    }
}

def read_symlink_target [path: string] {
    if not ($path | path exists) {
        return null
    }

    if (($path | path type) != "symlink") {
        return null
    }

    try {
        let result = (^readlink $path | complete)
        if $result.exit_code != 0 {
            return null
        }

        let target = ($result.stdout | str trim)
        if ($target | is-empty) { null } else { $target }
    } catch {
        null
    }
}

def is_home_manager_symlink_target [target?: string] {
    if $target == null {
        return false
    }

    let normalized = ($target | into string | str trim)
    $normalized | str contains $HOME_MANAGER_FILES_MARKER
}

export def get_manual_runtime_reference_path [] {
    get_yazelix_state_dir | path join "runtime" "current"
}

export def get_manual_desktop_entry_path [] {
    (get_xdg_data_home | path join "applications" "com.yazelix.Yazelix.desktop")
}

def get_manual_desktop_icon_path [size: string] {
    (get_xdg_data_home | path join "icons" "hicolor" $size "apps" "yazelix.png")
}

export def get_manual_main_config_path [] {
    get_main_user_config_path
}

export def is_home_manager_owned_surface [path: string] {
    is_home_manager_symlink_target (read_symlink_target $path)
}

export def is_manual_runtime_reference_path [path?: string] {
    let candidate = if $path == null {
        get_manual_runtime_reference_path
    } else {
        $path | path expand --no-symlink
    }

    let target = (read_symlink_target $candidate)
    if $target == null {
        return false
    }

    not (is_home_manager_symlink_target $target)
}

export def is_manual_desktop_entry_path [path?: string] {
    let candidate = if $path == null {
        get_manual_desktop_entry_path
    } else {
        $path | path expand
    }

    if not ($candidate | path exists) {
        return false
    }

    let raw = try {
        open --raw $candidate
    } catch {
        return false
    }

    (
        ($raw | str contains "Name=Yazelix")
        and (
            ($raw | str contains "X-Yazelix-Managed=true")
            or (
                $raw
                | lines
                | any {|line|
                    ($line | str starts-with "Exec=")
                    and ($line | str contains " desktop launch")
                }
            )
        )
    )
}

def collect_manual_desktop_icon_artifacts [] {
    $MANUAL_DESKTOP_ICON_SIZES
    | each {|size|
        let icon_path = (get_manual_desktop_icon_path $size)
        if ($icon_path | path exists) {
            {
                id: $"desktop_icon_($size)"
                label: $"manual desktop icon \(($size)\)"
                path: $icon_path
            }
        } else {
            null
        }
    }
    | compact
}

export def collect_home_manager_prepare_artifacts [] {
    mut artifacts = []

    let main_config = (get_manual_main_config_path)
    if ($main_config | path exists) and not (is_home_manager_owned_surface $main_config) {
        $artifacts = ($artifacts | append {
            id: "main_config"
            class: "blocker"
            label: "managed yazelix.toml surface"
            path: $main_config
        })
    }

    let desktop_entry = (get_manual_desktop_entry_path)
    if (is_manual_desktop_entry_path $desktop_entry) {
        $artifacts = ($artifacts | append {
            id: "desktop_entry"
            class: "cleanup"
            label: "manual desktop entry"
            path: $desktop_entry
        })
    }

    for icon_artifact in (collect_manual_desktop_icon_artifacts) {
        $artifacts = ($artifacts | append ($icon_artifact | upsert class "cleanup"))
    }

    $artifacts
}

export def has_home_manager_managed_install [] {
    let main_config = (get_manual_main_config_path)
    is_home_manager_owned_surface $main_config
}
