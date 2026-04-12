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

def prepare_hostile_install_env [temp_home: string] {
    let hostile_bin = ($temp_home | path join ".hostile_bin")
    mkdir $hostile_bin

    let real_nu = (resolve_external_command_path "nu")
    if $real_nu == null {
        error make { msg: "Could not resolve `nu` while preparing hostile install env" }
    }

    let hostile_scripts = [
        {
            name: "nu"
            content: ([
                "#!/bin/sh"
                $"exec \"($real_nu)\" \"$@\""
                ""
            ] | str join "\n")
        }
        {
            name: "starship"
            content: ([
                "#!/bin/sh"
                "echo '# hostile starship path leaked into install output'"
                ""
            ] | str join "\n")
        }
        {
            name: "zoxide"
            content: ([
                "#!/bin/sh"
                "echo '# hostile zoxide path leaked into install output'"
                ""
            ] | str join "\n")
        }
        {
            name: "mise"
            content: ([
                "#!/bin/sh"
                "echo '# hostile mise path leaked into install output'"
                ""
            ] | str join "\n")
        }
        {
            name: "atuin"
            content: ([
                "#!/bin/sh"
                "echo '# hostile atuin path leaked into install output'"
                ""
            ] | str join "\n")
        }
        {
            name: "carapace"
            content: ([
                "#!/bin/sh"
                "echo '# hostile carapace path leaked into install output'"
                ""
            ] | str join "\n")
        }
    ]

    for hostile_script in $hostile_scripts {
        let script_path = ($hostile_bin | path join $hostile_script.name)
        let content = $hostile_script.content
        $content | save --force --raw $script_path
        chmod +x $script_path
    }

    {
        hostile_bin: $hostile_bin
        hostile_nu: ($hostile_bin | path join "nu")
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

    let hostile_env = (prepare_hostile_install_env $temp_home)
    let inherited_path = ($env.PATH? | default [])
    let inherited_path_desc = ($inherited_path | describe)
    let install_path = if ($inherited_path_desc | str starts-with "list") {
        [$hostile_env.hostile_bin] | append $inherited_path
    } else if (($inherited_path | into string | str trim) | is-empty) {
        [$hostile_env.hostile_bin]
    } else {
        [$hostile_env.hostile_bin, ($inherited_path | into string)]
    }

    with-env {
        HOME: $temp_home
        XDG_CONFIG_HOME: $config_root
        XDG_DATA_HOME: $state_root
        PATH: $install_path
        YAZELIX_NU_BIN: $hostile_env.hostile_nu
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

    let yzx_path = ($temp_home | path join ".local" "bin" "yzx")
    let wrapper_target = (^readlink -f $yzx_path | str trim)
    let runtime_root = ($wrapper_target | path dirname | path dirname)
    let runtime_bin = ($runtime_root | path join "bin")
    let runtime_nu = ($runtime_bin | path join "nu")
    let runtime_yzx_cli = ($runtime_root | path join "shells" "posix" "yzx_cli.sh")
    let runtime_taplo_config = ($runtime_root | path join ".taplo.toml")
    let runtime_yazelix_default = ($runtime_root | path join "yazelix_default.toml")
    let legacy_runtime_link = ($temp_home | path join ".local" "share" "yazelix" "runtime" "current")
    let nushell_config = ($temp_home | path join ".config" "nushell" "config.nu")
    let user_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix.toml")
    let pack_config = ($temp_home | path join ".config" "yazelix" "user_configs" "yazelix_packs.toml")
    let config_root = ($temp_home | path join ".config" "yazelix")
    let managed_taplo_config = ($config_root | path join ".taplo.toml")
    let desktop_entry = ($temp_home | path join ".local" "share" "applications" "com.yazelix.Yazelix.desktop")
    let zellij_config = ($temp_home | path join ".local" "share" "yazelix" "configs" "zellij" "config.kdl")
    let bash_initializer = ($temp_home | path join ".local" "share" "yazelix" "initializers" "bash" "yazelix_init.sh")
    let nushell_initializer = ($temp_home | path join ".local" "share" "yazelix" "initializers" "nushell" "yazelix_init.nu")
    let hostile_bin = ($temp_home | path join ".hostile_bin")
    let yazi_theme = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "theme.toml")
    let yazi_flavor_root = ($temp_home | path join ".local" "share" "yazelix" "configs" "yazi" "flavors")

    require_path_missing $legacy_runtime_link "legacy installed runtime symlink"
    require_path_exists $runtime_nu "runtime-local Nushell binary"
    require_path_exists $runtime_yzx_cli "runtime-local POSIX yzx launcher"
    require_path_exists $runtime_taplo_config "runtime-local Taplo formatter config"
    require_path_exists $runtime_yazelix_default "runtime-local default config"
    require_path_exists $yzx_path "installed yzx wrapper"
    require_path_exists $nushell_config "generated Nushell hook config"
    require_path_exists $user_config "seeded user config"
    require_path_exists $managed_taplo_config "managed Taplo formatter config"
    require_path_missing $desktop_entry "default user-local desktop entry"
    require_path_exists $zellij_config "generated Zellij config"
    require_path_exists $bash_initializer "generated Bash initializer"
    require_path_exists $nushell_initializer "generated Nushell initializer"
    require_path_exists $yazi_theme "generated Yazi theme config"
    require_path_exists $yazi_flavor_root "generated Yazi flavors directory"

    require_path_missing $pack_config "legacy seeded pack config"

    for expected_bin in ["nu" "yzx" "zellij" "ghostty" "yazi" "hx" "nvim" "fish" "zsh" "bash" "nix" "jq" "fd" "rg"] {
        require_path_exists ($runtime_bin | path join $expected_bin) $"runtime binary `($expected_bin)`"
    }
    if (($nu.os-info.name | str downcase) == "linux") {
        require_path_exists ($runtime_bin | path join "nixGLMesa") "runtime binary `nixGLMesa`"
    }

    require_file_contains $nushell_config $runtime_root "generated Nushell hook config"
    require_file_not_contains $nushell_config (["runtime" "current"] | str join "/") "generated Nushell hook config"
    require_file_not_contains $bash_initializer $hostile_bin "generated Bash initializer"
    require_file_not_contains $nushell_initializer $hostile_bin "generated Nushell initializer"
    require_file_not_contains $yazi_theme "[flavor]" "generated Yazi theme config"

    if ((ls $yazi_flavor_root | where type == dir | length) < 1) {
        error make { msg: $"Generated Yazi flavors directory is empty: ($yazi_flavor_root)" }
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
        or ($probe.runtime != $runtime_root)
        or ($probe.path0 != $runtime_bin)
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
