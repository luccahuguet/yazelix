#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)

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
        YAZELIX_INSTALL_SKIP_DEVENV: "1"
    } {
        ^nix run .#install --extra-experimental-features "nix-command flakes" | complete
    }
}

def verify_installed_runtime [temp_home: string] {
    let runtime_current = ($temp_home | path join ".local" "share" "yazelix" "runtime" "current")
    let yzx_path = ($temp_home | path join ".local" "bin" "yzx")
    let nushell_config = ($temp_home | path join ".config" "nushell" "config.nu")
    let user_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix.toml")
    let pack_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix_packs.toml")
    let zellij_config = ($temp_home | path join ".local" "share" "yazelix" "configs" "zellij" "config.kdl")
    let yazi_theme = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "theme.toml")
    let yazi_flavor = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "flavors" "tokyo-night.yazi" "flavor.toml")

    require_path_exists $runtime_current "installed runtime symlink"
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
