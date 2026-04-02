const REPO_ROOT = ((path self) | path dirname | path join ".." ".." ".." | path expand)
const DEVENV_SKEW_WARNING = "is newer than devenv input"

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

def get_locked_devenv_package_root [] {
    let helper_path = ($REPO_ROOT | path join "locked_devenv_package.nix")
    let expr = $"let repo = builtins.toPath \"($REPO_ROOT)\"; flake = builtins.getFlake \(toString repo\); pkgs = flake.inputs.nixpkgs.legacyPackages.$\{builtins.currentSystem\}; in \(import \(repo + \"/($helper_path | path basename)\"\) { inherit pkgs; src = repo; }\).outPath"
    let result = (^nix eval --impure --raw --expr $expr | complete)
    require_success $result "Failed to resolve the locked devenv package path"
    $result.stdout | str trim
}

def require_no_devenv_skew_warning [package_root: string, temp_home: string] {
    let shell_probe = (
        run_package_nu
            $package_root
            $temp_home
            ([
                $"use '($package_root | path join "nushell" "scripts" "utils" "environment_bootstrap.nu")' get_devenv_base_command"
                "let cmd = (get_devenv_base_command | append [\"shell\" \"--\" \"true\"])"
                "let bin = ($cmd | first)"
                "let args = ($cmd | skip 1)"
                "^$bin ...$args | complete | to json -r"
            ] | str join "\n")
    )
    require_success $shell_probe "Packaged Yazelix failed to probe the shell-enter path"

    let resolved = ($shell_probe.stdout | str trim | from json)
    if $resolved.exit_code != 0 {
        error make { msg: $"Packaged shell-enter probe failed: ($resolved | to json -r)" }
    }

    let stderr_text = ($resolved.stderr | default "")
    if ($stderr_text | str contains $DEVENV_SKEW_WARNING) {
        error make { msg: $"Packaged Yazelix still emits the upstream devenv skew warning: ($stderr_text | str trim)" }
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
