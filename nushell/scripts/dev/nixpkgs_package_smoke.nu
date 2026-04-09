use ./devenv_lock_contract.nu [DEVENV_SKEW_WARNING get_locked_devenv_package_root]

export def make_temp_home [] {
    (^mktemp -d /tmp/yazelix_nixpkgs_package_XXXXXX | str trim)
}

export def require_success [result: record, failure_message: string] {
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

export def get_package_env [temp_home: string, package_root?: string] {
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

export def run_yzx [package_root: string, temp_home: string, ...args: string] {
    let yzx_path = ($package_root | path join "bin" "yzx")

    with-env (get_package_env $temp_home) {
        ^$yzx_path ...$args | complete
    }
}

export def run_package_nu [package_root: string, temp_home: string, command: string] {
    let nu_path = ($package_root | path join "bin" "nu")

    with-env (get_package_env $temp_home $package_root) {
        ^$nu_path -c $command | complete
    }
}

def require_no_devenv_skew_warning [package_root: string, temp_home: string] {
    let shell_probe_resolution = (
        run_package_nu
            $package_root
            $temp_home
            ([
                $"use '($package_root | path join "nushell" "scripts" "utils" "devenv_backend.nu")' get_devenv_base_command"
                "get_devenv_base_command | append [\"shell\" \"--\" \"true\"] | to json -r"
            ] | str join "\n")
    )
    require_success $shell_probe_resolution "Packaged Yazelix failed to resolve the shell-enter command"

    let shell_command = ($shell_probe_resolution.stdout | str trim | from json)
    let shell_bin = ($shell_command | first)
    let shell_args = ($shell_command | skip 1)
    let shell_probe = (
        with-env (get_package_env $temp_home $package_root) {
            ^$shell_bin ...$shell_args | complete
        }
    )
    require_success $shell_probe "Packaged Yazelix shell-enter probe failed"

    let combined_output = (($shell_probe.stderr | default "") + ($shell_probe.stdout | default ""))
    if ($combined_output | str contains $DEVENV_SKEW_WARNING) {
        error make { msg: $"Packaged Yazelix still emits the upstream devenv skew warning: ($combined_output | str trim)" }
    }
}

export def verify_yazelix_package [package_root: string] {
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

    let locked_package_root = (get_locked_devenv_package_root)
    let expected_locked_devenv = (^readlink -f ($locked_package_root | path join "bin" "devenv") | str trim)
    let resolved_runtime_devenv = (^readlink -f $expected_devenv | str trim)
    if $resolved_runtime_devenv != $expected_locked_devenv {
        error make { msg: $"Packaged Yazelix runtime devenv is not sourced from the locked package. Expected ($expected_locked_devenv), got ($resolved_runtime_devenv)" }
    }

    let env_result = (run_yzx $package_root $temp_home "env" "--no-shell")
    require_success $env_result "Packaged yzx env --no-shell failed"
    require_no_devenv_skew_warning $package_root $temp_home
}
