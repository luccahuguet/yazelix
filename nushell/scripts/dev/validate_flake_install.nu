#!/usr/bin/env nu

use ../utils/common.nu resolve_external_command_path

const INSTALL_TIMEOUT_SECONDS = 1500
const PROBE_TIMEOUT_SECONDS = 60

def make_temp_home [] {
    (^mktemp -d /tmp/yazelix_flake_install_XXXXXX | str trim)
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

def run_flake_install [temp_home: string] {
    let state_root = ($temp_home | path join ".local" "share")
    let config_root = ($temp_home | path join ".config")
    let nushell_config_dir = ($config_root | path join "nushell")
    let bashrc_path = ($temp_home | path join ".bashrc")
    let nushell_config_path = ($nushell_config_dir | path join "config.nu")

    mkdir $state_root
    mkdir $config_root
    mkdir $nushell_config_dir
    "" | save --force $bashrc_path
    "" | save --force $nushell_config_path

    with-env {
        HOME: $temp_home
        XDG_CONFIG_HOME: $config_root
        XDG_DATA_HOME: $state_root
    } {
        run_completed_external "running `nix run .#install` for flake install smoke validation" "nix" ["run" ".#install" "--extra-experimental-features" "nix-command flakes"] $INSTALL_TIMEOUT_SECONDS
    }
}

def run_installed_yzx [temp_home: string, ...args: string] {
    let yzx_path = ($temp_home | path join ".local" "bin" "yzx")
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
    print "🔍 Verifying installed runtime layout ..."

    let runtime_current = ($temp_home | path join ".local" "share" "yazelix" "runtime" "current")
    let runtime_bin = ($runtime_current | path join "bin")
    let runtime_nu = ($runtime_bin | path join "nu")
    let runtime_yzx_cli = ($runtime_current | path join "shells" "posix" "yzx_cli.sh")
    let runtime_taplo_config = ($runtime_current | path join ".taplo.toml")
    let runtime_yazelix_default = ($runtime_current | path join "yazelix_default.toml")
    let yzx_path = ($temp_home | path join ".local" "bin" "yzx")
    let nushell_config = ($temp_home | path join ".config" "nushell" "config.nu")
    let user_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix.toml")
    let pack_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix_packs.toml")
    let config_root = ($temp_home | path join ".config" "yazelix")
    let managed_taplo_config = ($config_root | path join ".taplo.toml")
    let zellij_config = ($temp_home | path join ".local" "share" "yazelix" "configs" "zellij" "config.kdl")
    let yazi_theme = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "theme.toml")
    let yazi_flavor_root = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "flavors")
    let resolved_runtime_current = (^readlink -f $runtime_current | str trim)

    require_path_exists $runtime_current "installed runtime symlink"
    require_path_exists $runtime_nu "runtime-local Nushell binary"
    require_path_exists $runtime_yzx_cli "runtime-local POSIX yzx launcher"
    require_path_exists $runtime_taplo_config "runtime-local Taplo formatter config"
    require_path_exists $runtime_yazelix_default "runtime-local default config"
    require_path_exists $yzx_path "installed yzx wrapper"
    require_path_exists $nushell_config "generated Nushell hook config"
    require_path_exists $user_config "seeded user config"
    require_path_exists $managed_taplo_config "managed Taplo formatter config"
    require_path_exists $zellij_config "generated Zellij config"
    require_path_exists $yazi_theme "generated Yazi theme config"
    require_path_exists $yazi_flavor_root "generated Yazi flavors directory"

    require_path_missing $pack_config "legacy seeded pack config"
    require_path_missing ($runtime_bin | path join "devenv") "runtime-local devenv binary"
    require_path_missing ($runtime_current | path join "devenv.lock") "runtime-local devenv.lock"
    require_path_missing ($runtime_current | path join "devenv.nix") "runtime-local devenv.nix"
    require_path_missing ($runtime_current | path join "devenv.yaml") "runtime-local devenv.yaml"
    require_path_missing ($runtime_current | path join "yazelix_packs_default.toml") "runtime-local pack template"

    for expected_bin in ["nu" "yzx" "zellij" "yazi" "hx" "nvim" "fish" "zsh" "bash" "nix" "jq" "fd" "rg"] {
        require_path_exists ($runtime_bin | path join $expected_bin) $"runtime binary `($expected_bin)`"
    }

    require_file_contains $nushell_config $resolved_runtime_current "generated Nushell hook config"
    require_file_not_contains $nushell_config "/runtime/current/" "generated Nushell hook config"
    require_file_not_contains $yazi_theme "[flavor]" "generated Yazi theme config"

    if ((ls $yazi_flavor_root | where type == dir | length) < 1) {
        error make { msg: $"Generated Yazi flavors directory is empty: ($yazi_flavor_root)" }
    }

    let wrapper_target = (^readlink $yzx_path | str trim)
    let expected_wrapper_target = ($runtime_current | path join "bin" "yzx")
    if ($wrapper_target != $expected_wrapper_target) {
        error make { msg: $"Installed yzx wrapper should point at runtime/current. Expected ($expected_wrapper_target), got ($wrapper_target)" }
    }

    let version_result = (run_installed_yzx $temp_home "--version-short")
    if $version_result.exit_code != 0 {
        if ($version_result.stdout | is-not-empty) {
            print $version_result.stdout
        }
        if ($version_result.stderr | is-not-empty) {
            print $version_result.stderr
        }
        error make { msg: "Installed yzx --version-short failed during flake install smoke validation" }
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
        error make { msg: "Runtime-local POSIX yzx launcher failed under minimal PATH during flake install smoke validation" }
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
            'print ({shell: ($env.IN_YAZELIX_SHELL | default ""), runtime: ($env.YAZELIX_RUNTIME_DIR | default ""), path0: (($env.PATH | default []) | first | default ""), editor: ($env.EDITOR | default "")} | to json -r)'
    )

    if $runtime_probe.exit_code != 0 {
        if ($runtime_probe.stdout | is-not-empty) {
            print $runtime_probe.stdout
        }
        if ($runtime_probe.stderr | is-not-empty) {
            print $runtime_probe.stderr
        }
        error make { msg: "Installed yzx run probe failed during flake install smoke validation" }
    }

    let probe = ($runtime_probe.stdout | str trim | from json)
    if (
        ($probe.shell != "true")
        or ($probe.runtime != ($runtime_current | into string))
        or ($probe.path0 != ($runtime_bin | into string))
        or (not ($probe.editor | str contains "yazelix_hx.sh"))
    ) {
        error make {
            msg: $"Installed runtime probe saw the wrong Yazelix env: ($probe | to json -r)"
        }
    }
}

def run_install_phase [temp_home: string] {
    prepare_temp_home $temp_home

    let install_result = (run_flake_install $temp_home)
    if $install_result.exit_code != 0 {
        if ($install_result.stdout | is-not-empty) {
            print $install_result.stdout
        }
        if ($install_result.stderr | is-not-empty) {
            print $install_result.stderr
        }
        error make { msg: "Flake install smoke validation failed while running `nix run .#install`" }
    }

    print "✅ Flake install smoke build phase passed"
}

def run_verify_phase [temp_home: string] {
    require_path_exists $temp_home "flake install smoke temp home"
    verify_installed_runtime $temp_home
    print "✅ Flake install smoke verification phase passed"
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
            print "✅ Flake install smoke check passed"
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
            error make { msg: $"Unsupported flake install smoke phase `($selected_phase)`. Expected one of: all, install, verify" }
        }
    }
}
