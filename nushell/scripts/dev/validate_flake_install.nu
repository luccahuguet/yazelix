#!/usr/bin/env nu

const REPO_ROOT = ((path self) | path dirname | path join ".." ".." ".." | path expand)
const DEVENV_SKEW_WARNING = "is newer than devenv input"

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

def get_locked_devenv_package_root [] {
    let expr = $"let repo = builtins.toPath \"($REPO_ROOT)\"; flake = builtins.getFlake \(toString repo\); pkgs = flake.inputs.nixpkgs.legacyPackages.$\{builtins.currentSystem\}; in \(import \(repo + \"/locked_devenv_package.nix\"\) { inherit pkgs; src = repo; }\).outPath"
    let result = (^nix eval --impure --raw --expr $expr | complete)
    if $result.exit_code != 0 {
        if ($result.stdout | is-not-empty) {
            print $result.stdout
        }
        if ($result.stderr | is-not-empty) {
            print $result.stderr
        }
        error make { msg: "Failed to resolve the locked devenv package path during flake install smoke validation" }
    }
    $result.stdout | str trim
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
        ^nix run .#install --extra-experimental-features "nix-command flakes" | complete
    }
}

def verify_installed_runtime [temp_home: string] {
    let runtime_current = ($temp_home | path join ".local" "share" "yazelix" "runtime" "current")
    let runtime_devenv = ($runtime_current | path join "bin" "devenv")
    let runtime_nu = ($runtime_current | path join "bin" "nu")
    let runtime_yzx_cli = ($runtime_current | path join "shells" "posix" "yzx_cli.sh")
    let devenv_cli_module = ($runtime_current | path join "nushell" "scripts" "utils" "devenv_cli.nu")
    let yzx_path = ($temp_home | path join ".local" "bin" "yzx")
    let nushell_config = ($temp_home | path join ".config" "nushell" "config.nu")
    let user_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix.toml")
    let pack_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix_packs.toml")
    let zellij_config = ($temp_home | path join ".local" "share" "yazelix" "configs" "zellij" "config.kdl")
    let yazi_theme = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "theme.toml")
    let yazi_flavor = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "flavors" "tokyo-night.yazi" "flavor.toml")
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
    require_path_exists $yazi_flavor "generated Yazi flavor file"

    require_file_contains $nushell_config "/runtime/current/" "generated Nushell hook config"

    let version_result = (
        with-env {
            HOME: $temp_home
            XDG_CONFIG_HOME: ($temp_home | path join ".config")
            XDG_DATA_HOME: ($temp_home | path join ".local" "share")
        } {
            ^$yzx_path --version-short | complete
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
        ^env -i
            HOME=$temp_home
            PATH="/usr/bin:/bin"
            XDG_CONFIG_HOME=($temp_home | path join ".config")
            XDG_DATA_HOME=($temp_home | path join ".local" "share")
            sh
            $runtime_yzx_cli
            --version-short
        | complete
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
        ^env -i
            HOME=$temp_home
            PATH="/usr/bin:/bin"
            XDG_CONFIG_HOME=($temp_home | path join ".config")
            XDG_DATA_HOME=($temp_home | path join ".local" "share")
            YAZELIX_RUNTIME_DIR=$runtime_current
            YAZELIX_DIR=$runtime_current
            $runtime_nu
            -c
            $"use ($devenv_cli_module | into string) *; print \(resolve_preferred_devenv_path\)"
        | complete
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

    let shell_probe_command = ([
        $"use '($runtime_current | path join "nushell" "scripts" "utils" "environment_bootstrap.nu")' get_devenv_base_command"
        "get_devenv_base_command | append [\"shell\" \"--\" \"true\"] | to json -r"
    ] | str join "\n")
    let shell_probe_resolution = (
        ^env -i
            HOME=$temp_home
            PATH="/usr/bin:/bin"
            XDG_CONFIG_HOME=($temp_home | path join ".config")
            XDG_DATA_HOME=($temp_home | path join ".local" "share")
            YAZELIX_RUNTIME_DIR=$runtime_current
            YAZELIX_DIR=$runtime_current
            $runtime_nu
            -c
            $shell_probe_command
        | complete
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
    let shell_probe = (^$shell_bin ...$shell_args | complete)

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
