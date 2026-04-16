#!/usr/bin/env nu

use common.nu [get_yazelix_state_dir]
use config_surfaces.nu get_main_user_config_path

const HOME_MANAGER_FILES_MARKER = "-home-manager-files/"
const MANUAL_DESKTOP_ICON_SIZES = ["48x48", "64x64", "128x128", "256x256"]
const MANAGED_SHELL_BLOCK_START_PREFIX = "# YAZELIX START"
const MANAGED_SHELL_BLOCK_END_PREFIX = "# YAZELIX END"
const MISSING_SHELL_BLOCK_INDEX = 999999999

def get_xdg_config_home [] {
    let configured = (
        $env.XDG_CONFIG_HOME?
        | default ""
        | into string
        | str trim
    )

    if ($configured | is-not-empty) {
        $configured | path expand
    } else if (($env.HOME? | default "" | into string | str trim) | is-not-empty) {
        $env.HOME | path join ".config"
    } else {
        "~/.config" | path expand
    }
}

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

export def get_manual_yzx_wrapper_path [] {
    ($env.HOME | path join ".local" "bin" "yzx")
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

def get_legacy_shell_block_surfaces [] {
    let config_home = (get_xdg_config_home)
    let home_dir = $env.HOME
    let surfaces = [
        {
            id: "bashrc"
            label: "legacy ~/.bashrc Yazelix shell block"
            path: ($home_dir | path join ".bashrc")
        }
        {
            id: "bash_profile"
            label: "legacy ~/.bash_profile Yazelix shell block"
            path: ($home_dir | path join ".bash_profile")
        }
        {
            id: "profile"
            label: "legacy ~/.profile Yazelix shell block"
            path: ($home_dir | path join ".profile")
        }
        {
            id: "zshrc"
            label: "legacy ~/.zshrc Yazelix shell block"
            path: ($home_dir | path join ".zshrc")
        }
        {
            id: "nushell_config"
            label: "legacy ~/.config/nushell/config.nu Yazelix shell block"
            path: ($config_home | path join "nushell" "config.nu")
        }
        {
            id: "fish_config"
            label: "legacy ~/.config/fish/config.fish Yazelix shell block"
            path: ($config_home | path join "fish" "config.fish")
        }
    ]

    $surfaces
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

def symlink_target_looks_like_legacy_yazelix_wrapper [target?: string] {
    if $target == null {
        return false
    }

    let normalized = ($target | into string | str trim)

    (
        ($normalized | str contains "yazelix-runtime")
        and ($normalized | str ends-with "/bin/yzx")
    )
}

def file_contents_look_like_legacy_yazelix_wrapper [path: string] {
    let raw = try {
        open --raw $path
    } catch {
        return false
    }

    (
        ($raw | str contains "shells/posix/yzx_cli.sh")
        or ($raw | str contains "Stable Yazelix CLI entrypoint for external tools and editors.")
        or (
            ($raw | str contains "YAZELIX_BOOTSTRAP_RUNTIME_DIR")
            and ($raw | str contains "Yazelix")
        )
    )
}

export def is_legacy_manual_yzx_wrapper_path [path?: string] {
    let candidate = if $path == null {
        get_manual_yzx_wrapper_path
    } else {
        $path | path expand
    }

    if not ($candidate | path exists) {
        return false
    }

    let target = (read_symlink_target $candidate)
    if (symlink_target_looks_like_legacy_yazelix_wrapper $target) {
        return true
    }

    file_contents_look_like_legacy_yazelix_wrapper $candidate
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

def join_lines_preserving_trailing_newline [lines: list<string>, had_trailing_newline: bool] {
    if ($lines | is-empty) {
        return ""
    }

    let joined = ($lines | str join "\n")
    if $had_trailing_newline {
        $"($joined)\n"
    } else {
        $joined
    }
}

def get_managed_shell_block_record [path: string] {
    if not ($path | path exists) {
        return null
    }

    if (($path | path type) != "file") {
        return null
    }

    let raw = try {
        open --raw $path
    } catch {
        return null
    }

    let lines = ($raw | lines)
    let had_trailing_newline = ($raw | str ends-with "\n")
    let enumerated = ($lines | enumerate)
    let start_index = (
        $enumerated
        | where {|entry| $entry.item | str starts-with $MANAGED_SHELL_BLOCK_START_PREFIX}
        | get -o 0.index
        | default $MISSING_SHELL_BLOCK_INDEX
    )
    if $start_index == $MISSING_SHELL_BLOCK_INDEX {
        return null
    }

    let end_index = (
        $enumerated
        | where {|entry| ($entry.index > $start_index) and ($entry.item | str starts-with $MANAGED_SHELL_BLOCK_END_PREFIX) }
        | get -o 0.index
        | default $MISSING_SHELL_BLOCK_INDEX
    )
    if $end_index == $MISSING_SHELL_BLOCK_INDEX {
        return null
    }

    let block_lines = ($lines | skip $start_index | take (($end_index - $start_index) + 1))
    mut before_lines = ($lines | take $start_index)
    let after_lines = ($lines | skip ($end_index + 1))
    if (not ($before_lines | is-empty)) and (($before_lines | last) == "") {
        $before_lines = ($before_lines | take (($before_lines | length) - 1))
    }
    let remaining_lines = ($before_lines | append $after_lines)

    {
        start_line: ($start_index + 1)
        end_line: ($end_index + 1)
        block_contents: (join_lines_preserving_trailing_newline $block_lines $had_trailing_newline)
        remaining_contents: (join_lines_preserving_trailing_newline $remaining_lines $had_trailing_newline)
    }
}

export def collect_legacy_yazelix_shell_block_artifacts [] {
    get_legacy_shell_block_surfaces
    | each {|surface|
        let block = (get_managed_shell_block_record $surface.path)
        if $block == null {
            null
        } else {
            {
                id: $"shell_block_($surface.id)"
                class: "cleanup"
                label: $surface.label
                path: $surface.path
                artifact_kind: "shell_block"
                start_line: $block.start_line
                end_line: $block.end_line
                block_contents: $block.block_contents
                remaining_contents: $block.remaining_contents
            }
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

    let manual_yzx_wrapper = (get_manual_yzx_wrapper_path)
    if (is_legacy_manual_yzx_wrapper_path $manual_yzx_wrapper) {
        $artifacts = ($artifacts | append {
            id: "manual_yzx_wrapper"
            class: "cleanup"
            label: "legacy ~/.local/bin/yzx wrapper"
            path: $manual_yzx_wrapper
        })
    }

    for shell_block_artifact in (collect_legacy_yazelix_shell_block_artifacts) {
        $artifacts = ($artifacts | append $shell_block_artifact)
    }

    $artifacts
}

export def has_home_manager_managed_install [] {
    let main_config = (get_manual_main_config_path)
    is_home_manager_owned_surface $main_config
}
