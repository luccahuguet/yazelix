#!/usr/bin/env nu

const CONTRACT_TIMEOUT_SECONDS = 120

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

def require_list_contains [items: list<string>, expected: string, label: string] {
    if not ($items | any {|item| $item == $expected }) {
        error make { msg: $"($label) is missing expected entry `($expected)`. Found: (($items | str join ', '))" }
    }
}

def run_completed_external [
    label: string
    cmd_bin: string
    cmd_args: list<string>
    timeout_seconds: int = $CONTRACT_TIMEOUT_SECONDS
] {
    print $"⏳ ($label) ..."

    let timeout_result = (which timeout | where type == "external")
    let result = if ($timeout_result | is-empty) {
        ^$cmd_bin ...$cmd_args | complete
    } else {
        let timeout_bin = ($timeout_result | get 0.path)
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

export def main [] {
    print "🔍 Validating installed-runtime contract surfaces ..."

    let install_template = "shells/posix/install_yazelix.sh.in"
    let cli_wrapper = "shells/posix/yzx_cli.sh"
    let environment_setup = "nushell/scripts/setup/environment.nu"
    let runtime_tree = "mk_runtime_tree.nix"
    let flake_path = "flake.nix"

    require_path_exists $flake_path "flake definition"
    require_path_exists $install_template "flake installer template"
    require_path_exists $cli_wrapper "stable POSIX CLI wrapper"
    require_path_exists $environment_setup "environment setup script"
    require_path_exists $runtime_tree "runtime tree builder"

    require_file_contains $install_template 'runtime_current="$runtime_root/current"' "flake installer template"
    require_file_contains $install_template '@coreutils_bin@/ln -sfn "$runtime_target" "$runtime_current"' "flake installer template"
    require_file_contains $install_template '@coreutils_bin@/ln -sfn "$runtime_current/bin/yzx" "$yzx_link"' "flake installer template"
    require_file_contains $install_template 'YAZELIX_RUNTIME_DIR="$runtime_current"' "flake installer template"
    require_file_contains $install_template '@nu_bin@ "$runtime_current/nushell/scripts/setup/environment.nu" --skip-welcome' "flake installer template"

    require_file_contains $cli_wrapper 'export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$RUNTIME_DIR"' "stable POSIX CLI wrapper"
    require_file_contains $cli_wrapper 'runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh"' "stable POSIX CLI wrapper"
    require_file_contains $cli_wrapper 'exec "$YAZELIX_NU_BIN" -c "$nu_command"' "stable POSIX CLI wrapper"

    require_file_contains $environment_setup "get_installed_yazelix_runtime_reference_dir" "environment setup script"
    require_file_contains $environment_setup 'path join "bin" "yzx"' "environment setup script"

    require_file_contains $runtime_tree "import ./locked_devenv_package.nix" "runtime tree builder"
    require_file_contains $runtime_tree 'ln -s ${lockedDevenv}/bin/devenv "$out/bin/devenv"' "runtime tree builder"
    require_file_contains $runtime_tree 'ln -s ${pkgs.nushell}/bin/nu "$out/bin/nu"' "runtime tree builder"
    require_file_contains $runtime_tree 'cat > "$out/bin/yzx" <<EOF' "runtime tree builder"

    let flake_show = (run_completed_external "evaluating flake package/app surface" "nix" ["flake" "show" "--json"])
    if $flake_show.exit_code != 0 {
        if ($flake_show.stdout | is-not-empty) {
            print $flake_show.stdout
        }
        if ($flake_show.stderr | is-not-empty) {
            print $flake_show.stderr
        }
        error make { msg: "Failed to evaluate flake outputs during installed-runtime contract validation" }
    }

    let flake = ($flake_show.stdout | from json)
    let package_keys = ($flake | get packages."x86_64-linux" | columns)
    for expected in ["default" "runtime" "install" "yazelix" "locked_devenv"] {
        require_list_contains $package_keys $expected "x86_64-linux package outputs"
    }

    let install_app_type = ($flake | get apps."x86_64-linux".install.type)
    if $install_app_type != "app" {
        error make { msg: $"Unexpected flake install app type: ($install_app_type)" }
    }

    print "✅ Installed-runtime contract smoke passed"
}
