#!/usr/bin/env nu

use ../utils/common.nu resolve_external_command_path

const INSTALL_TIMEOUT_SECONDS = 2700
const PROBE_TIMEOUT_SECONDS = 60

def make_temp_home [] {
    (^mktemp -d /tmp/yazelix_profile_install_XXXXXX | str trim)
}

def prepare_temp_home [temp_home: string] {
    let parent = ($temp_home | path dirname)
    mkdir $parent
    if ($temp_home | path exists) {
        rm -rf $temp_home
    }
}

def require_path_exists [path: string, label: string] {
    if not ($path | path exists) {
        error make { msg: $"Missing ($label): ($path)" }
    }
}

def require_path_missing [path: string, label: string] {
    if ($path | path exists) {
        error make { msg: $"Unexpected ($label): ($path)" }
    }
}

def require_file_contains [path: string, needle: string, label: string] {
    let content = (open --raw $path)
    if not ($content | str contains $needle) {
        error make { msg: $"($label) does not contain expected text `($needle)`: ($path)" }
    }
}

def require_file_not_contains [path: string, needle: string, label: string] {
    let content = (open --raw $path)
    if ($content | str contains $needle) {
        error make { msg: $"($label) unexpectedly contains text `($needle)`: ($path)" }
    }
}

def require_non_empty_dir [path: string, label: string] {
    require_path_exists $path $label
    let entries = (ls $path | where type == file)
    if (($entries | length) < 1) {
        error make { msg: $"($label) is empty: ($path)" }
    }
}

def resolve_ghostty_shader_reference [ghostty_config_path: string, shader_ref: string] {
    let raw_ref = ($shader_ref | str trim | str replace -r '^"(.*)"$' '$1')
    if ($raw_ref | str starts-with "/") {
        return $raw_ref
    }

    let relative_ref = if ($raw_ref | str starts-with "./") {
        $raw_ref | str replace -r '^\./' ''
    } else {
        $raw_ref
    }

    ($ghostty_config_path | path dirname | path join $relative_ref)
}

def require_ghostty_shader_references_exist [ghostty_config_path: string] {
    require_path_exists $ghostty_config_path "generated Ghostty config"

    let shader_refs = (
        open --raw $ghostty_config_path
        | lines
        | each {|line| $line | str trim}
        | where {|line| $line | str starts-with "custom-shader = "}
        | each {|line|
            $line
            | split row "="
            | skip 1
            | str join "="
            | str trim
        }
    )

    if ($shader_refs | is-empty) {
        error make { msg: $"Generated Ghostty config references no shader assets: ($ghostty_config_path)" }
    }

    for shader_ref in $shader_refs {
        let shader_path = (resolve_ghostty_shader_reference $ghostty_config_path $shader_ref)
        require_path_exists $shader_path $"generated Ghostty shader `($shader_ref)`"
    }
}

def run_completed_external [
    label: string
    cmd_bin: string
    cmd_args: list<string>
    timeout_seconds: int = $PROBE_TIMEOUT_SECONDS
] {
    print $"⏳ ($label) ..."

    let timeout_bin = (resolve_external_command_path "timeout")
    let result = if $timeout_bin == null {
        ^$cmd_bin ...$cmd_args | complete
    } else {
        ^$timeout_bin -k "15" ($timeout_seconds | into string) $cmd_bin ...$cmd_args | complete
    }

    if $result.exit_code == 124 {
        let stdout = ($result.stdout | default "" | str trim)
        let stderr = ($result.stderr | default "" | str trim)
        let detail = if ($stderr | is-not-empty) {
            $stderr
        } else if ($stdout | is-not-empty) {
            $stdout
        } else {
            "No subprocess output was captured before timeout."
        }
        error make { msg: $"Timed out after ($timeout_seconds)s while ($label).\n($detail)" }
    }

    $result
}

def run_profile_install [temp_home: string] {
    let state_root = ($temp_home | path join ".local" "share")
    let config_root = ($temp_home | path join ".config")
    let profile_root = ($temp_home | path join ".nix-profile")

    with-env {
        HOME: $temp_home
        XDG_CONFIG_HOME: $config_root
        XDG_DATA_HOME: $state_root
    } {
        run_completed_external "running `nix profile add --profile ... .#yazelix` for cold profile-install validation" "nix" [
            "--extra-experimental-features"
            "nix-command flakes"
            "profile"
            "add"
            "--profile"
            $profile_root
            ".#yazelix"
        ] $INSTALL_TIMEOUT_SECONDS
    }
}

def run_installed_yzx [temp_home: string, ...args: string] {
    let yzx_path = ($temp_home | path join ".nix-profile" "bin" "yzx")
    let label = $"probing installed yzx command: ($args | str join ' ')"
    with-env {
        HOME: $temp_home
        XDG_CONFIG_HOME: ($temp_home | path join ".config")
        XDG_DATA_HOME: ($temp_home | path join ".local" "share")
    } {
        run_completed_external $label $yzx_path $args
    }
}

def verify_installed_runtime [temp_home: string] {
    print "🔍 Verifying profile-installed runtime layout ..."

    let profile_root = ($temp_home | path join ".nix-profile")
    let yzx_path = ($profile_root | path join "bin" "yzx")
    let local_wrapper = ($temp_home | path join ".local" "bin" "yzx")
    let legacy_runtime_link = ($temp_home | path join ".local" "share" "yazelix" "runtime" "current")
    let desktop_entry = ($temp_home | path join ".local" "share" "applications" "com.yazelix.Yazelix.desktop")
    let user_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix.toml")
    let pack_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix_packs.toml")
    let nushell_config = ($temp_home | path join ".config" "nushell" "config.nu")

    require_path_exists $yzx_path "profile-installed yzx wrapper"
    require_path_missing $local_wrapper "legacy user-local yzx wrapper"
    require_path_missing $legacy_runtime_link "legacy installed runtime symlink"
    require_path_missing $desktop_entry "default user-local desktop entry before explicit desktop install"
    require_path_missing $user_config "managed user config before first runtime entry"
    require_path_missing $pack_config "legacy managed pack config before first runtime entry"
    require_path_missing $nushell_config "host Nushell hook config before first runtime entry"

    let readlink_result = (^readlink -f $yzx_path | complete)
    if $readlink_result.exit_code != 0 {
        error make { msg: $"Failed to resolve installed yzx wrapper target: (($readlink_result.stderr | str trim))" }
    }
    let wrapper_target = ($readlink_result.stdout | str trim)
    let runtime_root = ($wrapper_target | path dirname | path dirname)
    let runtime_bin = ($runtime_root | path join "bin")
    let runtime_toolbin = ($runtime_root | path join "toolbin")
    let runtime_libexec = ($runtime_root | path join "libexec")
    let runtime_yzx_cli = ($runtime_root | path join "shells" "posix" "yzx_cli.sh")
    let runtime_ghostty_wrapper = ($runtime_root | path join "shells" "posix" "yazelix_ghostty.sh")
    let runtime_taplo_config = ($runtime_root | path join ".taplo.toml")
    let runtime_yazelix_default = ($runtime_root | path join "yazelix_default.toml")
    let runtime_ghostty_shader_root = ($runtime_root | path join "configs" "terminal_emulators" "ghostty" "shaders")
    let runtime_ghostty_shader_builder = ($runtime_ghostty_shader_root | path join "build_shaders.nu")
    let runtime_ghostty_trail_variant = ($runtime_ghostty_shader_root | path join "variants" "reef.glsl")
    let runtime_ghostty_effect_template = ($runtime_ghostty_shader_root | path join "upstream_effects" "ripple_rectangle_cursor.glsl")
    let generated_ghostty_root = ($temp_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty")
    let generated_ghostty_config = ($generated_ghostty_root | path join "config")
    let generated_ghostty_effect_dir = ($generated_ghostty_root | path join "shaders" "generated_effects")
    let runtime_terminal_materialization_script = ($runtime_root | path join "nushell" "scripts" "core" "launch_yazelix.nu")

    require_path_exists $yzx_path "profile-installed yzx entrypoint"
    require_path_exists $runtime_toolbin "runtime toolbin"
    require_path_exists ($runtime_libexec | path join "nu") "runtime-local Nushell binary"
    require_path_exists ($runtime_libexec | path join "yzx") "runtime-local Rust yzx root helper"
    require_path_exists ($runtime_libexec | path join "yzx_control") "runtime-local yzx_control helper"
    require_path_exists $runtime_yzx_cli "runtime-local POSIX yzx launcher"
    require_path_exists $runtime_ghostty_wrapper "runtime-local Ghostty env wrapper"
    require_path_exists $runtime_yazelix_default "runtime-local default config"
    require_path_exists $runtime_ghostty_shader_builder "runtime-local Ghostty shader builder"
    require_path_exists $runtime_ghostty_trail_variant "runtime-local Ghostty trail shader variant"
    require_path_exists $runtime_ghostty_effect_template "runtime-local Ghostty cursor effect template"

    for expected_tool in ["zellij" "ghostty" "yazi" "hx" "nvim" "fish" "zsh" "bash" "nix" "jq" "fd" "rg"] {
        require_path_exists ($runtime_libexec | path join $expected_tool) $"runtime tool `($expected_tool)`"
    }
    for expected_exported_tool in ["nu" "zellij" "yazi" "hx" "nvim" "bash" "jq" "fd" "rg"] {
        require_path_exists ($runtime_toolbin | path join $expected_exported_tool) $"exported runtime tool `($expected_exported_tool)`"
    }
    require_path_missing ($runtime_toolbin | path join "dirname") "runtime-private helper leaked into exported toolbin"
    if (($nu.os-info.name | str downcase) == "linux") {
        require_path_exists ($runtime_libexec | path join "nixGLMesa") "runtime tool `nixGLMesa`"
        require_path_exists ($runtime_libexec | path join "pgrep") "runtime tool `pgrep`"
    }

    let expected_wrapper_target = ($runtime_root | path join "bin" "yzx")
    if ($wrapper_target != $expected_wrapper_target) {
        error make { msg: $"Installed yzx wrapper should point at the packaged runtime. Expected ($expected_wrapper_target), got ($wrapper_target)" }
    }

    let version_result = (run_installed_yzx $temp_home "--version-short")
    if $version_result.exit_code != 0 {
        if ($version_result.stdout | is-not-empty) {
            print $version_result.stdout
        }
        if ($version_result.stderr | is-not-empty) {
            print $version_result.stderr
        }
        error make { msg: "Installed yzx --version-short failed during cold profile-install validation" }
    }

    let version_text = ($version_result.stdout | str trim)
    if not ($version_text | str starts-with "Yazelix v") {
        error make { msg: $"Unexpected installed yzx version output: ($version_text)" }
    }

    let posix_launcher_result = (
        run_completed_external "probing runtime-local POSIX yzx launcher" "env" [
            "-i"
            $"HOME=($temp_home)"
            "PATH=/usr/bin:/bin"
            $"XDG_CONFIG_HOME=($temp_home | path join '.config')"
            $"XDG_DATA_HOME=($temp_home | path join '.local' 'share')"
            "sh"
            $runtime_yzx_cli
            "--version-short"
        ]
    )

    if $posix_launcher_result.exit_code != 0 {
        if ($posix_launcher_result.stdout | is-not-empty) {
            print $posix_launcher_result.stdout
        }
        if ($posix_launcher_result.stderr | is-not-empty) {
            print $posix_launcher_result.stderr
        }
        error make { msg: "Runtime-local POSIX yzx launcher failed under minimal PATH during cold profile-install validation" }
    }

    let posix_version_text = ($posix_launcher_result.stdout | str trim)
    if not ($posix_version_text | str starts-with "Yazelix v") {
        error make { msg: $"Unexpected runtime-local POSIX yzx output: ($posix_version_text)" }
    }

    let runtime_probe = (
        run_installed_yzx
            $temp_home
            "run"
            "nu"
            "-c"
            'let runtime_dir = ($env.YAZELIX_RUNTIME_DIR | default ""); let path_entries = ($env.PATH | default []); let runtime_libexec = (if ($runtime_dir | is-empty) { "" } else { $runtime_dir | path join "libexec" }); print ({shell: ($env.IN_YAZELIX_SHELL | default ""), runtime: $runtime_dir, path0: ($path_entries | get -o 0 | default ""), path1: ($path_entries | get -o 1 | default ""), libexec_on_path: (if ($runtime_libexec | is-empty) { false } else { $path_entries | any {|entry| $entry == $runtime_libexec } }), yzx: ((which yzx | get -o 0.path | default "")), editor: ($env.EDITOR | default "")} | to json -r)'
    )

    if $runtime_probe.exit_code != 0 {
        if ($runtime_probe.stdout | is-not-empty) {
            print $runtime_probe.stdout
        }
        if ($runtime_probe.stderr | is-not-empty) {
            print $runtime_probe.stderr
        }
        error make { msg: "Installed yzx run probe failed during cold profile-install validation" }
    }

    let probe = ($runtime_probe.stdout | str trim | from json)
    if (
        ($probe.shell != "true")
        or ($probe.runtime != $runtime_root)
        or ($probe.path0 != $runtime_toolbin)
        or ($probe.path1 != $runtime_bin)
        or ($probe.libexec_on_path)
        or ($probe.yzx != ($runtime_bin | path join "yzx"))
        or (not ($probe.editor | str contains "yazelix_hx.sh"))
    ) {
        error make {
            msg: $"Installed runtime probe saw the wrong Yazelix env: ($probe | to json -r)"
        }
    }

    let ghostty_config_probe = (
        run_installed_yzx
            $temp_home
            "run"
            "nu"
            "-c"
            $"use \"($runtime_terminal_materialization_script)\" [generate_all_terminal_configs]; generate_all_terminal_configs \"($runtime_root)\""
    )

    if $ghostty_config_probe.exit_code != 0 {
        if ($ghostty_config_probe.stdout | is-not-empty) {
            print $ghostty_config_probe.stdout
        }
        if ($ghostty_config_probe.stderr | is-not-empty) {
            print $ghostty_config_probe.stderr
        }
        error make { msg: "Installed runtime failed to materialize Ghostty shader-backed terminal config during cold profile-install validation" }
    }

    require_ghostty_shader_references_exist $generated_ghostty_config
    require_non_empty_dir $generated_ghostty_effect_dir "generated Ghostty cursor effect shaders directory"
}

def run_install_phase [temp_home: string] {
    prepare_temp_home $temp_home

    let install_result = (run_profile_install $temp_home)
    if $install_result.exit_code != 0 {
        if ($install_result.stdout | is-not-empty) {
            print $install_result.stdout
        }
        if ($install_result.stderr | is-not-empty) {
            print $install_result.stderr
        }
        error make { msg: "Cold profile-install validation failed while running `nix profile add --profile ... .#yazelix`" }
    }

    print "✅ Cold profile-install build phase passed"
}

def run_verify_phase [temp_home: string] {
    require_path_exists $temp_home "cold profile-install temp home"
    verify_installed_runtime $temp_home
    print "✅ Cold profile-install verification phase passed"
}

export def main [phase?: string, temp_home?: string] {
    let selected_phase = ($phase | default "all")

    match $selected_phase {
        "all" => {
            let resolved_temp_home = if $temp_home == null {
                make_temp_home
            } else {
                prepare_temp_home $temp_home
                $temp_home
            }

            run_install_phase $resolved_temp_home
            run_verify_phase $resolved_temp_home
            print "✅ Cold profile-install check passed"
        }
        "install" => {
            if $temp_home == null {
                error make { msg: "The `install` phase requires an explicit temp_home path" }
            }
            run_install_phase $temp_home
        }
        "verify" => {
            if $temp_home == null {
                error make { msg: "The `verify` phase requires an explicit temp_home path" }
            }
            run_verify_phase $temp_home
        }
        _ => {
            error make { msg: $"Unsupported cold profile-install phase `($selected_phase)`. Expected one of: all, install, verify" }
        }
    }
}
