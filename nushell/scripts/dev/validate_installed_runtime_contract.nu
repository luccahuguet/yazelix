#!/usr/bin/env nu

const CONTRACT_TIMEOUT_SECONDS = 120

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
        error make { msg: $"($label) still contains forbidden text `($needle)`: ($path)" }
    }
}

def require_list_contains [items: list<string>, expected: string, label: string] {
    if not ($items | any {|item| $item == $expected }) {
        error make { msg: $"($label) is missing expected entry `($expected)`. Found: (($items | str join ', '))" }
    }
}

def require_list_not_contains [items: list<string>, forbidden: string, label: string] {
    if ($items | any {|item| $item == $forbidden }) {
        error make { msg: $"($label) unexpectedly contains forbidden entry `($forbidden)`. Found: (($items | str join ', '))" }
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
        let timeout_bin = ($timeout_result | get -o 0.path)
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

def build_flake_output_path [attr: string, label: string] {
    let result = (run_completed_external $label "nix" ["build" "--no-link" "--print-out-paths" $".#($attr)"])
    if $result.exit_code != 0 {
        if ($result.stdout | is-not-empty) {
            print $result.stdout
        }
        if ($result.stderr | is-not-empty) {
            print $result.stderr
        }
        error make { msg: $"Failed while ($label)" }
    }

    let output_path = ($result.stdout | lines | where {|line| ($line | str trim) != "" } | get -o 0 | default "" | str trim)
    if ($output_path | is-empty) {
        error make { msg: $"($label) did not return an output path" }
    }

    require_path_exists $output_path $"built flake output for .#($attr)"
    $output_path
}

def validate_rust_routed_nu_modules [runtime_root: string, label: string] {
    let scripts_dir = ($runtime_root | path join "nushell" "scripts")

    for relative_path in [
        ["core" "yzx_session.nu"]
        ["yzx" "desktop.nu"]
        ["yzx" "dev.nu"]
        ["yzx" "edit.nu"]
        ["yzx" "enter.nu"]
        ["yzx" "import.nu"]
        ["yzx" "launch.nu"]
        ["yzx" "menu.nu"]
        ["yzx" "popup.nu"]
        ["yzx" "screen.nu"]
        ["yzx" "tutor.nu"]
        ["yzx" "whats_new.nu"]
    ] {
        require_path_exists ($scripts_dir | path join ...$relative_path) $"($label) Rust-routed Nu module"
    }
}

export def main [] {
    print "🔍 Validating installed-runtime contract surfaces ..."

    let cli_wrapper = "shells/posix/yzx_cli.sh"
    let detached_launch_probe = "shells/posix/detached_launch_probe.sh"
    let runtime_env = "shells/posix/runtime_env.sh"
    let environment_setup = "nushell/scripts/setup/environment.nu"
    let runtime_tree = "packaging/mk_runtime_tree.nix"
    let flake_path = "flake.nix"

    require_path_exists $flake_path "flake definition"
    require_path_missing "shells/posix/install_yazelix.sh.in" "legacy flake installer template"
    require_path_exists $cli_wrapper "stable POSIX CLI wrapper"
    require_path_exists $detached_launch_probe "detached launch probe helper"
    require_path_exists $runtime_env "runtime env helper"
    require_path_exists $environment_setup "environment setup script"
    require_path_exists $runtime_tree "runtime tree builder"

    require_file_contains $cli_wrapper 'export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$RUNTIME_DIR"' "stable POSIX CLI wrapper"
    require_file_contains $cli_wrapper 'runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh"' "stable POSIX CLI wrapper"
    require_file_contains $cli_wrapper 'yzx_root_bin="${YAZELIX_YZX_BIN:-$RUNTIME_DIR/libexec/yzx}"' "stable POSIX CLI wrapper"
    require_file_contains $cli_wrapper 'exec "$yzx_root_bin" "$@"' "stable POSIX CLI wrapper"
    require_file_not_contains $runtime_env 'export YAZELIX_DIR=' "runtime env helper"

    require_file_not_contains $environment_setup "get_installed_yazelix_runtime_reference_dir" "environment setup script"
    require_file_not_contains $environment_setup "ensure_user_cli_wrapper" "environment setup script"

    require_file_contains $runtime_tree "import ./runtime_deps.nix" "runtime tree builder"
    require_file_contains $runtime_tree 'ln -s ${src}/yazelix_default.toml "$out/yazelix_default.toml"' "runtime tree builder"
    require_file_contains $runtime_tree 'for bin_dir in ${escapedRuntimeBinDirs}; do' "runtime tree builder"
    require_file_contains $runtime_tree 'cat > "$out/bin/yzx" <<EOF' "runtime tree builder"
    require_file_not_contains $runtime_tree 'yazelix_packs_default.toml' "runtime tree builder"

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
    for expected in ["default" "runtime" "yazelix"] {
        require_list_contains $package_keys $expected "x86_64-linux package outputs"
    }
    require_list_not_contains $package_keys "install" "x86_64-linux package outputs"
    require_list_not_contains $package_keys "locked_devenv" "x86_64-linux package outputs"

    let app_keys = ($flake | get apps."x86_64-linux" | columns)
    for expected in ["default" "yazelix"] {
        require_list_contains $app_keys $expected "x86_64-linux app outputs"
    }
    require_list_not_contains $app_keys "install" "x86_64-linux app outputs"

    let runtime_out = (build_flake_output_path "runtime" "building runtime package for installed-runtime validation")
    validate_rust_routed_nu_modules $runtime_out "built runtime package"
    require_path_exists ($runtime_out | path join $detached_launch_probe) "built runtime detached launch probe helper"

    let yazelix_out = (build_flake_output_path "yazelix" "building yazelix package for installed-runtime validation")
    validate_rust_routed_nu_modules $yazelix_out "built yazelix package"
    require_path_exists ($yazelix_out | path join $detached_launch_probe) "built yazelix detached launch probe helper"

    let built_yzx = ($yazelix_out | path join "bin" "yzx")
    require_path_exists $built_yzx "built yazelix CLI wrapper"

    let smoke_result = (run_completed_external
        "smoke-running built yazelix public CLI"
        "env"
        ["YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1" $built_yzx "why"]
    )
    if $smoke_result.exit_code != 0 {
        if ($smoke_result.stdout | is-not-empty) {
            print $smoke_result.stdout
        }
        if ($smoke_result.stderr | is-not-empty) {
            print $smoke_result.stderr
        }
        error make { msg: "Built yazelix package failed the public CLI smoke check" }
    }
    require_file_contains $built_yzx "shells/posix/yzx_cli.sh" "built yazelix CLI wrapper"
    if not (($smoke_result.stdout | default "") | str contains "Yazelix is a reproducible terminal IDE") {
        error make { msg: "Built yazelix public CLI smoke check returned unexpected output for `yzx why`" }
    }

    print "✅ Installed-runtime contract smoke passed"
}
