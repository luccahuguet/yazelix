#!/usr/bin/env nu

use ../utils/common.nu resolve_external_command_path
use ./devenv_lock_contract.nu [DEVENV_SKEW_WARNING get_locked_devenv_package_root]

const INSTALL_TIMEOUT_SECONDS = 480
const PROBE_TIMEOUT_SECONDS = 60

def make_temp_home [] {
    (^mktemp -d /tmp/yazelix_flake_install_XXXXXX | str trim)
}

def require_path_exists [path: string, label: string] {
    if not ($path | path exists) {
        error make { msg: $"Missing ($label): ($path)" }
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

def verify_installed_runtime [temp_home: string] {
    print "🔍 Verifying installed runtime layout ..."

    let runtime_current = ($temp_home | path join ".local" "share" "yazelix" "runtime" "current")
    let runtime_devenv = ($runtime_current | path join "bin" "devenv")
    let runtime_nu = ($runtime_current | path join "bin" "nu")
    let runtime_yzx_cli = ($runtime_current | path join "shells" "posix" "yzx_cli.sh")
    let devenv_cli_module = ($runtime_current | path join "nushell" "scripts" "utils" "devenv_cli.nu")
    let runtime_helper_module = ($runtime_current | path join "configs" "zellij" "scripts" "runtime_helper.nu")
    let yzx_path = ($temp_home | path join ".local" "bin" "yzx")
    let nushell_config = ($temp_home | path join ".config" "nushell" "config.nu")
    let user_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix.toml")
    let pack_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix_packs.toml")
    let config_root = ($temp_home | path join ".config" "yazelix")
    let zellij_config = ($temp_home | path join ".local" "share" "yazelix" "configs" "zellij" "config.kdl")
    let yazi_theme = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "theme.toml")
    let yazi_flavor_root = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "flavors")
    let locked_package_root = (get_locked_devenv_package_root)

    require_path_exists $runtime_current "installed runtime symlink"
    require_path_exists $runtime_devenv "runtime-local devenv binary"
    require_path_exists $runtime_nu "runtime-local Nushell binary"
    require_path_exists $runtime_yzx_cli "runtime-local POSIX yzx launcher"
    require_path_exists $yzx_path "installed yzx wrapper"
    require_path_exists $nushell_config "generated Nushell hook config"
    require_path_exists $user_config "seeded user config"
    require_path_exists $pack_config "seeded pack config"
    require_path_exists $zellij_config "generated Zellij config"
    require_path_exists $yazi_theme "generated Yazi theme config"
    require_path_exists $yazi_flavor_root "generated Yazi flavors directory"

    require_file_contains $nushell_config "/runtime/current/" "generated Nushell hook config"
    require_file_not_contains $yazi_theme "[flavor]" "generated Yazi theme config"

    if ((ls $yazi_flavor_root | where type == dir | length) < 1) {
        error make { msg: $"Generated Yazi flavors directory is empty: ($yazi_flavor_root)" }
    }

    let wrapper_target = (^readlink $yzx_path | str trim)
    let expected_wrapper_target = ($runtime_current | path join "bin" "yzx")
    if ($wrapper_target != $expected_wrapper_target) {
        error make { msg: $"Installed yzx wrapper should point at runtime/current, not a pinned store path. Expected ($expected_wrapper_target), got ($wrapper_target)" }
    }

    let version_result = (
        with-env {
            HOME: $temp_home
            XDG_CONFIG_HOME: ($temp_home | path join ".config")
            XDG_DATA_HOME: ($temp_home | path join ".local" "share")
        } {
            run_completed_external "probing installed `yzx --version-short`" $yzx_path ["--version-short"]
        }
    )

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

    let runtime_devenv_probe = (
        run_completed_external "probing runtime-local devenv resolution" "env" [
            "-i"
            $"HOME=($temp_home)"
            "PATH=/usr/bin:/bin"
            $"XDG_CONFIG_HOME=($temp_home | path join '.config')"
            $"XDG_DATA_HOME=($temp_home | path join '.local' 'share')"
            $"YAZELIX_RUNTIME_DIR=($runtime_current)"
            $"YAZELIX_DIR=($runtime_current)"
            $runtime_nu
            "-c"
            $"use ($devenv_cli_module | into string) *; print \(resolve_preferred_devenv_path\)"
        ]
    )

    if $runtime_devenv_probe.exit_code != 0 {
        if ($runtime_devenv_probe.stdout | is-not-empty) {
            print $runtime_devenv_probe.stdout
        }
        if ($runtime_devenv_probe.stderr | is-not-empty) {
            print $runtime_devenv_probe.stderr
        }
        error make { msg: "Installed runtime failed to resolve its own runtime-local devenv path during flake install smoke validation" }
    }

    let selected_devenv = ($runtime_devenv_probe.stdout | str trim | path expand)
    let expected_devenv = ($runtime_devenv | path expand)
    if ($selected_devenv != $expected_devenv) {
        error make { msg: $"Installed runtime selected the wrong devenv path: expected ($expected_devenv), got ($selected_devenv)" }
    }

    let resolved_runtime_devenv = (^readlink -f $runtime_devenv | str trim)
    let expected_locked_devenv = (^readlink -f ($locked_package_root | path join "bin" "devenv") | str trim)
    if ($resolved_runtime_devenv != $expected_locked_devenv) {
        error make { msg: $"Installed runtime devenv is not sourced from the locked package. Expected ($expected_locked_devenv), got ($resolved_runtime_devenv)" }
    }

    let stale_runtime_probe = (
        run_completed_external "probing stale config-root runtime recovery" "env" [
            "-i"
            $"HOME=($temp_home)"
            "PATH=/usr/bin:/bin"
            $"XDG_CONFIG_HOME=($temp_home | path join '.config')"
            $"XDG_DATA_HOME=($temp_home | path join '.local' 'share')"
            $"YAZELIX_RUNTIME_DIR=($config_root)"
            $"YAZELIX_DIR=($config_root)"
            $runtime_nu
            "-c"
            $"use '($runtime_helper_module | into string)' [get_runtime_nu_path]; print \(get_runtime_nu_path\)"
        ]
    )

    if $stale_runtime_probe.exit_code != 0 {
        if ($stale_runtime_probe.stdout | is-not-empty) {
            print $stale_runtime_probe.stdout
        }
        if ($stale_runtime_probe.stderr | is-not-empty) {
            print $stale_runtime_probe.stderr
        }
        error make { msg: "Installed runtime failed to recover the canonical Nushell path when YAZELIX_RUNTIME_DIR still points at the config root" }
    }

    let recovered_runtime_nu = ($stale_runtime_probe.stdout | str trim | path expand)
    let expected_runtime_nu = ($runtime_nu | path expand)
    if ($recovered_runtime_nu != $expected_runtime_nu) {
        error make { msg: $"Installed runtime recovered the wrong Nushell path from a stale config-root runtime env. Expected ($expected_runtime_nu), got ($recovered_runtime_nu)" }
    }

    let shell_probe_command = ([
        $"use '($runtime_current | path join "nushell" "scripts" "utils" "environment_bootstrap.nu")' get_devenv_base_command"
        "get_devenv_base_command | append [\"shell\" \"--\" \"true\"] | to json -r"
    ] | str join "\n")
    let shell_probe_resolution = (
        run_completed_external "resolving installed runtime shell-enter command" "env" [
            "-i"
            $"HOME=($temp_home)"
            "PATH=/usr/bin:/bin"
            $"XDG_CONFIG_HOME=($temp_home | path join '.config')"
            $"XDG_DATA_HOME=($temp_home | path join '.local' 'share')"
            $"YAZELIX_RUNTIME_DIR=($runtime_current)"
            $"YAZELIX_DIR=($runtime_current)"
            $runtime_nu
            "-c"
            $shell_probe_command
        ]
    )

    if $shell_probe_resolution.exit_code != 0 {
        if ($shell_probe_resolution.stdout | is-not-empty) {
            print $shell_probe_resolution.stdout
        }
        if ($shell_probe_resolution.stderr | is-not-empty) {
            print $shell_probe_resolution.stderr
        }
        error make { msg: "Installed runtime failed to resolve the shell-enter command during flake install smoke validation" }
    }

    let shell_command = ($shell_probe_resolution.stdout | str trim | from json)
    let shell_bin = ($shell_command | first)
    let shell_args = ($shell_command | skip 1)
    let shell_probe = (
        run_completed_external "running installed runtime shell-enter probe" $shell_bin $shell_args $INSTALL_TIMEOUT_SECONDS
    )

    if $shell_probe.exit_code != 0 {
        if ($shell_probe.stdout | is-not-empty) {
            print $shell_probe.stdout
        }
        if ($shell_probe.stderr | is-not-empty) {
            print $shell_probe.stderr
        }
        error make { msg: "Installed runtime shell-enter probe command failed during flake install smoke validation" }
    }

    let probe_stdout = ($shell_probe.stdout | default "")
    let probe_stderr = ($shell_probe.stderr | default "")
    if (($probe_stdout | str contains $DEVENV_SKEW_WARNING) or ($probe_stderr | str contains $DEVENV_SKEW_WARNING)) {
        error make { msg: $"Installed runtime still emits the upstream devenv skew warning: (($probe_stderr + $probe_stdout) | str trim)" }
    }
}

export def main [] {
    let temp_home = (make_temp_home)

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

    verify_installed_runtime $temp_home
    print "✅ Flake install smoke check passed"
}
