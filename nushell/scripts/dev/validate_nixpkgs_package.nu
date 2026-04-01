#!/usr/bin/env nu

def make_temp_home [] {
    (^mktemp -d /tmp/yazelix_nixpkgs_package_XXXXXX | str trim)
}

def require_success [result: record, failure_message: string] {
    if $result.exit_code != 0 {
        if ($result.stdout | is-not-empty) {
            print $result.stdout
        }
        if ($result.stderr | is-not-empty) {
            print $result.stderr
        }
        error make { msg: $failure_message }
    }
}

def build_yazelix_package [] {
    with-env { HOME: (make_temp_home) } {
        ^nix build .#yazelix --print-out-paths --extra-experimental-features "nix-command flakes" | complete
    }
}

def get_package_env [temp_home: string, package_root?: string] {
    mut package_env = {
        HOME: $temp_home
        XDG_CONFIG_HOME: ($temp_home | path join ".config")
        XDG_DATA_HOME: ($temp_home | path join ".local" "share")
        SHELL: ($env.SHELL? | default "/bin/sh")
    }

    if $package_root != null {
        $package_env = (
            $package_env
            | insert YAZELIX_RUNTIME_DIR $package_root
            | insert YAZELIX_DIR $package_root
        )
    }

    $package_env
}

def run_yzx [package_root: string, temp_home: string, ...args: string] {
    let yzx_path = ($package_root | path join "bin" "yzx")

    with-env (get_package_env $temp_home) {
        ^$yzx_path ...$args | complete
    }
}

def run_package_nu [package_root: string, temp_home: string, command: string] {
    let nu_path = ($package_root | path join "bin" "nu")

    with-env (get_package_env $temp_home $package_root) {
        ^$nu_path -c $command | complete
    }
}

def verify_yazelix_package [package_root: string] {
    let temp_home = (make_temp_home)

    let version_result = (run_yzx $package_root $temp_home "--version-short")
    require_success $version_result "Packaged yzx --version-short failed"
    let version_text = ($version_result.stdout | str trim)
    if not ($version_text | str starts-with "Yazelix v") {
        error make { msg: $"Unexpected packaged yzx version output: ($version_text)" }
    }

    let doctor_result = (run_yzx $package_root $temp_home "doctor" "--verbose")
    require_success $doctor_result "Packaged yzx doctor --verbose failed"

    let expected_devenv = ($package_root | path join "bin" "devenv")
    let devenv_result = (
        run_package_nu
            $package_root
            $temp_home
            ([
                $"use '($package_root | path join "nushell" "scripts" "utils" "devenv_cli.nu")' *"
                "print (resolve_preferred_devenv_path)"
            ] | str join "\n")
    )
    require_success $devenv_result "Packaged resolve_preferred_devenv_path probe failed"
    let resolved_devenv = ($devenv_result.stdout | str trim)
    if $resolved_devenv != $expected_devenv {
        error make { msg: $"Packaged Yazelix did not prefer its runtime-owned devenv. Expected ($expected_devenv), got ($resolved_devenv)" }
    }

    let env_result = (run_yzx $package_root $temp_home "env" "--no-shell")
    require_success $env_result "Packaged yzx env --no-shell failed"
}

export def main [] {
    let build_result = (build_yazelix_package)
    require_success $build_result "Failed to build .#yazelix during nixpkgs package validation"

    let package_root = ($build_result.stdout | lines | last | str trim)
    if ($package_root | is-empty) {
        error make { msg: "nix build .#yazelix did not return an output path" }
    }

    with-env { SHELL: "/usr/bin/true" } {
        verify_yazelix_package $package_root
    }
    print "✅ Nixpkgs-style Yazelix package smoke check passed"
}
