#!/usr/bin/env nu

use common.nu [get_yazelix_state_dir]
use config_surfaces.nu [get_main_user_config_path get_pack_sidecar_path get_managed_taplo_support_path]

const HOME_MANAGER_FILES_MARKER = "-home-manager-files/"

def get_xdg_data_home [] {
    let configured = (
        $env.XDG_DATA_HOME?
        | default "~/.local/share"
        | into string
        | str trim
    )

    $configured | path expand
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

def manual_yzx_launcher_target [] {
    (get_manual_runtime_reference_path | path join "bin" "yzx")
}

def manual_desktop_exec_variants [launcher_path: string] {
    [
        $"Exec=\"($launcher_path)\" desktop launch"
        $"Exec=($launcher_path) desktop launch"
    ]
}

export def get_manual_runtime_reference_path [] {
    get_yazelix_state_dir | path join "runtime" "current"
}

export def get_manual_yzx_cli_path [] {
    ($env.HOME | path join ".local" "bin" "yzx")
}

export def get_manual_desktop_entry_path [] {
    (get_xdg_data_home | path join "applications" "com.yazelix.Yazelix.desktop")
}

export def get_manual_main_config_path [] {
    get_main_user_config_path
}

export def get_manual_pack_config_path [] {
    get_pack_sidecar_path (get_main_user_config_path)
}

def get_manual_taplo_support_path [] {
    get_managed_taplo_support_path
}

def get_manual_generated_config_paths [] {
    [
        (get_yazelix_state_dir | path join "configs" "yazi")
        (get_yazelix_state_dir | path join "configs" "zellij")
    ]
}

export def is_home_manager_owned_surface [path: string] {
    is_home_manager_symlink_target (read_symlink_target $path)
}

def is_manual_runtime_reference_path [path?: string] {
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

export def is_manual_yzx_cli_path [path?: string] {
    let candidate = if $path == null {
        get_manual_yzx_cli_path
    } else {
        $path | path expand --no-symlink
    }

    let target = (read_symlink_target $candidate)
    if $target == null {
        return false
    }

    $target == (manual_yzx_launcher_target)
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

    let launcher_path = (get_manual_yzx_cli_path)
    let exec_variants = (manual_desktop_exec_variants $launcher_path)

    (($raw | str contains "Name=Yazelix") and ($raw | lines | any {|line| $line in $exec_variants }))
}

export def collect_manual_uninstall_artifacts [] {
    mut artifacts = []

    let manual_cli = (get_manual_yzx_cli_path)
    if (is_manual_yzx_cli_path $manual_cli) {
        $artifacts = ($artifacts | append {
            id: "launcher"
            label: "manual stable yzx shim"
            path: $manual_cli
        })
    }

    let runtime_reference = (get_manual_runtime_reference_path)
    if (is_manual_runtime_reference_path $runtime_reference) {
        $artifacts = ($artifacts | append {
            id: "runtime_current"
            label: "installed runtime/current reference"
            path: $runtime_reference
        })
    }

    let desktop_entry = (get_manual_desktop_entry_path)
    if (is_manual_desktop_entry_path $desktop_entry) {
        $artifacts = ($artifacts | append {
            id: "desktop_entry"
            label: "manual desktop entry"
            path: $desktop_entry
        })
    }

    let taplo_support = (get_manual_taplo_support_path)
    if ($taplo_support | path exists) and not (is_home_manager_owned_surface $taplo_support) {
        $artifacts = ($artifacts | append {
            id: "taplo_support"
            label: "managed Taplo support file"
            path: $taplo_support
        })
    }

    for generated_path in (get_manual_generated_config_paths) {
        if ($generated_path | path exists) {
            $artifacts = ($artifacts | append {
                id: ($generated_path | path basename)
                label: $"generated (($generated_path | path basename)) config tree"
                path: $generated_path
            })
        }
    }

    $artifacts
}

export def collect_home_manager_prepare_artifacts [] {
    mut artifacts = []

    let runtime_reference = (get_manual_runtime_reference_path)
    if (is_manual_runtime_reference_path $runtime_reference) {
        $artifacts = ($artifacts | append {
            id: "runtime_current"
            class: "blocker"
            label: "installed runtime/current reference"
            path: $runtime_reference
        })
    }

    let main_config = (get_manual_main_config_path)
    if ($main_config | path exists) and not (is_home_manager_owned_surface $main_config) {
        $artifacts = ($artifacts | append {
            id: "main_config"
            class: "blocker"
            label: "managed yazelix.toml surface"
            path: $main_config
        })
    }

    let pack_config = (get_manual_pack_config_path)
    if ($pack_config | path exists) and not (is_home_manager_owned_surface $pack_config) {
        $artifacts = ($artifacts | append {
            id: "pack_config"
            class: "blocker"
            label: "managed yazelix_packs.toml surface"
            path: $pack_config
        })
    }

    let manual_cli = (get_manual_yzx_cli_path)
    if (is_manual_yzx_cli_path $manual_cli) {
        $artifacts = ($artifacts | append {
            id: "launcher"
            class: "cleanup"
            label: "manual stable yzx shim"
            path: $manual_cli
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

    $artifacts
}

export def has_home_manager_managed_install [] {
    let runtime_reference = (get_manual_runtime_reference_path)
    let main_config = (get_manual_main_config_path)
    let pack_config = (get_manual_pack_config_path)

    (
        (is_home_manager_owned_surface $runtime_reference)
        or (is_home_manager_owned_surface $main_config)
        or (is_home_manager_owned_surface $pack_config)
    )
}
