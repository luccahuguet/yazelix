#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_runtime_dir resolve_yazelix_nu_bin]
use ../utils/config_parser.nu [parse_yazelix_config]
use ../utils/runtime_env.nu get_runtime_env

const FLOATING_WRAPPER_ENV_KEYS = [
    "PATH"
    "YAZELIX_RUNTIME_DIR"
    "IN_YAZELIX_SHELL"
    "NIX_CONFIG"
    "ZELLIJ_DEFAULT_LAYOUT"
    "YAZI_CONFIG_HOME"
    "YAZELIX_MANAGED_HELIX_BINARY"
    "EDITOR"
    "VISUAL"
    "HELIX_RUNTIME"
]

def get_current_shell_wrapper_env [] {
    mut wrapper_env = {}

    for key in $FLOATING_WRAPPER_ENV_KEYS {
        let value = ($env | get -o $key | default null)
        if $value != null {
            let text = ($value | into string)
            if ($text | is-not-empty) {
                $wrapper_env = ($wrapper_env | upsert $key $text)
            }
        }
    }

    $wrapper_env
}

def serialize_wrapper_env_value [value: any] {
    let described = ($value | describe)

    if ($described | str starts-with "list") {
        $value | each {|entry| $entry | into string } | str join (char esep)
    } else {
        $value | into string
    }
}

export def build_floating_wrapper_env_args [wrapper_env: record] {
    $wrapper_env
    | transpose key value
    | each {|row| $"($row.key)=(serialize_wrapper_env_value $row.value)" }
}

export def get_floating_wrapper_env [] {
    let current_shell_env = (get_current_shell_wrapper_env)
    let config = (parse_yazelix_config)
    (get_runtime_env $config) | merge $current_shell_env
}

export def get_new_editor_pane_launch_env [yazi_id: string = ""] {
    mut pane_env = (get_floating_wrapper_env)

    if ($yazi_id | str trim | is-not-empty) {
        $pane_env = ($pane_env | upsert YAZI_ID $yazi_id)
    }

    $pane_env
}

export def open_floating_runtime_script [
    pane_name: string
    script_relative_path: string
    cwd: string
    extra_env: record = {}
    command_args: list<string> = []
    width_percent: int = 90
    height_percent: int = 90
] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let runtime_script = ($runtime_dir | path join $script_relative_path)
    let runtime_nu = (resolve_yazelix_nu_bin)
    if not ($runtime_script | path exists) {
        error make {msg: $"Floating runtime script not found at: ($runtime_script)"}
    }
    if not ($runtime_nu | path exists) {
        error make {msg: $"Resolved Yazelix Nushell binary not found at: ($runtime_nu)"}
    }

    let wrapper_env = ((get_floating_wrapper_env) | merge $extra_env)
    let env_args = (build_floating_wrapper_env_args $wrapper_env)
    let width_arg = $"($width_percent)%"
    let height_arg = $"($height_percent)%"
    let x_offset = (((100 - $width_percent) / 2) | math floor | into int)
    let y_offset = (((100 - $height_percent) / 2) | math floor | into int)
    let x_arg = $"($x_offset)%"
    let y_arg = $"($y_offset)%"

    ^zellij run --name $pane_name --floating --close-on-exit --width $width_arg --height $height_arg --x $x_arg --y $y_arg --cwd $cwd -- env ...$env_args $runtime_nu $runtime_script ...$command_args
}
