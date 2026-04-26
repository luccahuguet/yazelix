#!/usr/bin/env nu
# Development helper commands for maintainers

use ../utils/runtime_paths.nu [
    get_yazelix_state_dir
    get_yazelix_runtime_dir
]
use ../utils/yzx_core_bridge.nu [resolve_yzx_control_path]

# Development and maintainer commands
export def "yzx dev" [] {
    print "Run 'yzx dev --help' to see available maintainer subcommands"
}

def require_yazelix_repo_root [] {
    let pwd = ($env.PWD? | default "" | into string | str trim)
    let result = (if ($pwd | is-empty) {
        {exit_code: 1, stdout: "", stderr: ""}
    } else {
        ^git -C $pwd rev-parse --show-toplevel | complete
    })
    let repo_root = ($result.stdout | str trim | path expand)
    let repo_contract_ok = (
        ($result.exit_code == 0)
        and ($repo_root | path exists)
        and (($repo_root | path join "flake.nix") | path exists)
        and (($repo_root | path join "yazelix_default.toml") | path exists)
    )
    if not $repo_contract_ok {
        error make {msg: "This maintainer command requires a writable Yazelix repo checkout. Run it from the repo root or another directory inside the same checkout."}
    }

    $repo_root
}

def run_repo_maintainer_command [repo_root: string, ...maintainer_args: string] {
    let command = ([
        "develop"
        "-c"
        "cargo"
        "run"
        "--quiet"
        "--manifest-path"
        ($repo_root | path join "rust_core" "Cargo.toml")
        "-p"
        "yazelix_core"
        "--bin"
        "yzx_repo_maintainer"
        "--"
        "--repo-root"
        $repo_root
        ...$maintainer_args
    ])
    do { cd $repo_root; ^nix ...$command } | complete
}

def run_repo_maintainer_json_command [repo_root: string, ...maintainer_args: string] {
    let result = (run_repo_maintainer_command $repo_root ...$maintainer_args)
    if $result.exit_code != 0 {
        let stderr = ($result.stderr | default "" | str trim)
        error make {msg: $"Yazelix Rust maintainer command failed: ($stderr)"}
    }
    if ($result.stdout | is-empty) {
        {}
    } else {
        $result.stdout | from json
    }
}

def run_repo_maintainer_checked [repo_root: string, failure_message: string, ...maintainer_args: string] {
    let result = (run_repo_maintainer_command $repo_root ...$maintainer_args)
    if ($result.stdout | is-not-empty) {
        print --raw $result.stdout
    }
    if ($result.stderr | is-not-empty) {
        print --stderr --raw $result.stderr
    }
    if $result.exit_code != 0 {
        error make {msg: $failure_message}
    }
}

def require_fast_cargo [] {
    if ((which cargo | is-empty) or (which rustc | is-empty)) {
        error make {
            msg: "Fast Rust maintainer commands require cargo and rustc on PATH. Install the maintainer Rust toolchain once, or use the explicit Nix/package gates when you need full environment realization."
        }
    }
}

def rust_target_specs [repo_root: string, target: string] {
    let specs = [
        {
            name: "core"
            manifest_path: ($repo_root | path join "rust_core" "Cargo.toml")
            check_args: ["-p", "yazelix_core"]
            test_args: ["-p", "yazelix_core"]
        }
        {
            name: "pane_orchestrator"
            manifest_path: ($repo_root | path join "rust_plugins" "zellij_pane_orchestrator" "Cargo.toml")
            check_args: ["--lib"]
            test_args: ["--lib"]
        }
    ]

    match $target {
        "all" => $specs
        "core" => ($specs | where name == "core")
        "pane_orchestrator" => ($specs | where name == "pane_orchestrator")
        _ => {
            error make {msg: $"Unknown Rust target '($target)'. Expected one of: core, pane_orchestrator, all."}
        }
    }
}

def parse_rust_target_and_tail [args: list<string>, default_target: string] {
    let known_targets = ["core", "pane_orchestrator", "all"]
    if ($args | is-empty) {
        return {target: $default_target, tail: []}
    }

    let first = ($args | first)
    if ($first in $known_targets) {
        return {target: $first, tail: ($args | skip 1)}
    }

    {target: $default_target, tail: $args}
}

def run_fast_cargo_checked [repo_root: string, label: string, cargo_args: list<string>] {
    require_fast_cargo
    print $"Running: cargo ($cargo_args | str join ' ')"
    let result = (do { cd $repo_root; ^cargo ...$cargo_args } | complete)
    if ($result.stdout | is-not-empty) {
        print --raw $result.stdout
    }
    if ($result.stderr | is-not-empty) {
        print --stderr --raw $result.stderr
    }
    if $result.exit_code != 0 {
        error make {msg: $"Fast Rust ($label) failed."}
    }
}

# Refresh maintainer flake inputs and run update canaries
export def "yzx dev update" [
    --yes      # Skip confirmation prompt
    --no-canary  # Skip canary refresh/build checks after updating flake.lock
    --activate: string = ""  # Required unless --canary-only: installer, home_manager, or none
    --home-manager-dir: string = "~/.config/home-manager"  # Home Manager flake directory used when --activate home_manager
    --home-manager-input: string = "yazelix-hm"  # Home Manager flake input name to refresh before switch
    --home-manager-attr: string = ""  # Optional Home Manager flake output attribute appended as #attr during switch
    --canary-only  # Run canary checks without updating flake.lock or syncing pins
    --canaries: list<string> = []  # Canary subset: default, shell_layout
] {
    let repo_root = (require_yazelix_repo_root)
    mut args = ["dev-update"]
    if $yes { $args = ($args | append "--yes") }
    if $no_canary { $args = ($args | append "--no-canary") }
    if ($activate | is-not-empty) {
        $args = ($args | append ["--activate", $activate])
    }
    if ($home_manager_dir | is-not-empty) {
        $args = ($args | append ["--home-manager-dir", $home_manager_dir])
    }
    if ($home_manager_input | is-not-empty) {
        $args = ($args | append ["--home-manager-input", $home_manager_input])
    }
    if ($home_manager_attr | is-not-empty) {
        $args = ($args | append ["--home-manager-attr", $home_manager_attr])
    }
    if $canary_only { $args = ($args | append "--canary-only") }
    for canary in $canaries {
        $args = ($args | append ["--canary", $canary])
    }
    run_repo_maintainer_checked $repo_root "Yazelix Rust update workflow failed" ...$args
}

# Bump the tracked Yazelix version and create release metadata
export def "yzx dev bump" [
    version: string  # Version tag to release, for example v14
] {
    let repo_root = (require_yazelix_repo_root)
    let result = (run_repo_maintainer_json_command $repo_root "version-bump" $version)
    print $"✅ Bumped Yazelix from ($result.previous_version) to ($result.target_version)"
    print $"   commit: ($result.commit_sha)"
    print $"   tag: ($result.tag)"
}

# Sync GitHub issue lifecycle into Beads locally
export def "yzx dev sync_issues" [
    --dry-run  # Show the local GitHub→Beads reconciliation plan without mutating Beads
] {
    let repo_root = (require_yazelix_repo_root)
    mut args = ["sync-issues"]
    if $dry_run {
        $args = ($args | append "--dry-run")
    }
    run_repo_maintainer_checked $repo_root "Yazelix Rust issue sync failed" ...$args
}

# Build the Zellij pane-orchestrator wasm
export def "yzx dev build_pane_orchestrator" [
    --sync  # Sync the built wasm into the repo/runtime paths after a successful build
] {
    let repo_root = (require_yazelix_repo_root)
    mut args = ["build-pane-orchestrator"]
    if $sync {
        $args = ($args | append "--sync")
    }
    run_repo_maintainer_checked $repo_root "Yazelix Rust pane-orchestrator build failed" ...$args
}

# Inspect the current Yazelix tab session state
export def "yzx dev inspect_session" [
    --json  # Emit the raw pane-orchestrator session snapshot as JSON
] {
    let yzx_control_bin = (resolve_yzx_control_path)
    mut args = ["zellij", "inspect-session"]
    if $json {
        $args = ($args | append "--json")
    }
    ^$yzx_control_bin ...$args
}

# Show fast Rust inner-loop commands
export def "yzx dev rust" [] {
    print "Fast Rust inner-loop commands:"
    print "  yzx dev rust fmt [TARGET] [--check]"
    print "  yzx dev rust check [TARGET]"
    print "  yzx dev rust test [TARGET] [cargo test args...]"
    print ""
    print "TARGET: core, pane_orchestrator, or all"
    print "For tests, TARGET can be omitted; unmatched args are passed to core cargo test."
    print "These commands run cargo directly from the current environment. Use Nix/Home Manager/package validation as explicit final gates."
}

# Format Rust code directly without entering nix develop
export def "yzx dev rust fmt" [
    target: string = "all"  # core, pane_orchestrator, or all
    --check                 # Check formatting without changing files
] {
    let repo_root = (require_yazelix_repo_root)
    for spec in (rust_target_specs $repo_root $target) {
        mut args = ["fmt", "--manifest-path", $spec.manifest_path, "--all"]
        if $check {
            $args = ($args | append ["--", "--check"])
        }
        run_fast_cargo_checked $repo_root $"rust fmt ($spec.name)" $args
    }
}

# Run fast cargo check directly without entering nix develop
export def "yzx dev rust check" [
    target: string = "core"  # core, pane_orchestrator, or all
] {
    let repo_root = (require_yazelix_repo_root)
    for spec in (rust_target_specs $repo_root $target) {
        let args = (["check", "--manifest-path", $spec.manifest_path] | append $spec.check_args)
        run_fast_cargo_checked $repo_root $"rust check ($spec.name)" $args
    }
}

# Run fast cargo tests directly without entering nix develop
export def "yzx dev rust test" [
    ...args: string  # Optional target followed by extra cargo test args, such as a focused test filter
] {
    let repo_root = (require_yazelix_repo_root)
    let parsed = (parse_rust_target_and_tail $args "core")
    for spec in (rust_target_specs $repo_root $parsed.target) {
        let cargo_args = (["test", "--manifest-path", $spec.manifest_path] | append $spec.test_args | append $parsed.tail)
        run_fast_cargo_checked $repo_root $"rust test ($spec.name)" $cargo_args
    }
}

def clear_startup_profile_caches [] {
    let state_root = (get_yazelix_state_dir | path join "state")
    let cache_paths = [
        ($state_root | path join "rebuild_hash")
    ]
    for cache_path in $cache_paths {
        if ($cache_path | path exists) {
            rm -f $cache_path
        }
    }
}

def resolve_profile_source_root [] {
    let repo_root = (try { require_yazelix_repo_root } catch { null })
    if $repo_root != null {
        return {
            root: $repo_root
            kind: "repo_checkout"
        }
    }

    let runtime_root = (get_yazelix_runtime_dir)
    if $runtime_root != null {
        return {
            root: $runtime_root
            kind: "installed_runtime"
        }
    }

    error make {
        msg: "yzx dev profile requires either a Yazelix repo checkout or an installed Yazelix runtime."
    }
}

def build_profile_metadata [scenario: string, clear_cache: bool, source_info: record] {
    {
        scenario: $scenario
        clear_cache: $clear_cache
        source_root: $source_info.root
        source_kind: $source_info.kind
        cwd: (pwd)
        in_yazelix_shell: (($env.IN_YAZELIX_SHELL? | default "") == "true")
        host: (sys host)
    }
}

def get_profile_cli_path [source_root: string] {
    $source_root | path join "shells" "posix" "yzx_cli.sh"
}

def build_profile_env [profile_run: record, source_root: string] {
    $profile_run.env
    | upsert YAZELIX_RUNTIME_DIR $source_root
    | upsert YAZELIX_STARTUP_PROFILE_SKIP_WELCOME "true"
    | upsert YAZELIX_STARTUP_PROFILE_EXIT_BEFORE_ZELLIJ "true"
    | upsert YAZELIX_SHELLHOOK_SKIP_WELCOME "true"
}

def wait_for_profile_handoff [profile_run: record, scenario_label: string] {
    let yzx_control_bin = (resolve_yzx_control_path $profile_run.env.YAZELIX_RUNTIME_DIR?)
    let completed = (
        ^$yzx_control_bin profile wait-step
            $profile_run.report_path
            "inner"
            "zellij_handoff_ready"
            --timeout-ms 15000
        | str trim
    ) == "true"

    if not $completed {
        error make {
            msg: $"($scenario_label) profiling timed out waiting for inner.zellij_handoff_ready. Report: ($profile_run.report_path)"
        }
    }
}

def run_dev_profile_harness [
    scenario: string
    startup_args: list<string>
    --clear-cache
] {
    let clear_cache_enabled = $clear_cache
    let source_info = (resolve_profile_source_root)
    let yzx_cli = (get_profile_cli_path $source_info.root)
    let yzx_control_bin = (resolve_yzx_control_path $source_info.root)
    let meta_json = (build_profile_metadata $scenario $clear_cache_enabled $source_info | to json -r)
    let profile_run = (^$yzx_control_bin profile create-run $scenario --metadata $meta_json | from json)

    if $clear_cache_enabled {
        clear_startup_profile_caches
    }

    let profile_env = (build_profile_env $profile_run $source_info.root)
    let result = (with-env $profile_env {
        do { ^sh $yzx_cli enter ...$startup_args } | complete
    })

    if $result.exit_code != 0 {
        error make {
            msg: "Startup profiling failed"
            label: {
                text: ($result.stderr | default $result.stdout | default "profiled startup exited unsuccessfully")
                span: (metadata $yzx_cli).span
            }
        }
    }

    ^$yzx_control_bin profile print-report $profile_run.report_path
}

def run_cold_profile_command [clear_cache: bool] {
    if (($env.IN_YAZELIX_SHELL? | default "") == "true") {
        print "❌ Error: Cold launch profiling must be run from a vanilla terminal"
        print ""
        print "To profile a cold startup path:"
        print "  1. Open a new terminal outside Yazelix"
        print "  2. Run: yzx dev profile --cold"
        return
    }

    print "🚀 Profiling cold Yazelix startup..."
    run_dev_profile_harness "enter_cold" [] --clear-cache=$clear_cache | ignore
}

def run_default_profile_command [] {
    let in_yazelix_shell = (($env.IN_YAZELIX_SHELL? | default "") == "true")
    if $in_yazelix_shell {
        print "🚀 Profiling warm Yazelix startup from the current shell..."
        run_dev_profile_harness "enter_warm" [] | ignore
    } else {
        print "⚠️  Not currently inside a Yazelix shell."
        print "   Profiling the default current-terminal startup path instead."
        print ""
        run_dev_profile_harness "enter_default" [] | ignore
    }
}

# Run Yazelix test suite
export def "yzx dev test" [
    --verbose(-v)  # Show detailed test output
    --new-window(-n)  # Run tests in a new Yazelix window
    --lint-only  # Run only syntax validation
    --profile  # Print timing summaries for the default suite
    --sweep  # Run the non-visual configuration sweep only
    --visual  # Run the visual terminal sweep only
    --all(-a)  # Run the default suite plus sweep + visual lanes
    --delay: int = 3  # Delay between visual terminal launches in seconds
] {
    let repo_root = (require_yazelix_repo_root)
    mut args = ["run-tests"]
    if $verbose { $args = ($args | append "--verbose") }
    if $new_window { $args = ($args | append "--new-window") }
    if $lint_only { $args = ($args | append "--lint-only") }
    if $profile { $args = ($args | append "--profile") }
    if $sweep { $args = ($args | append "--sweep") }
    if $visual { $args = ($args | append "--visual") }
    if $all { $args = ($args | append "--all") }
    $args = ($args | append ["--delay", ($delay | into string)])
    run_repo_maintainer_checked $repo_root "Yazelix Rust test runner failed" ...$args
}

def run_desktop_profile_command [] {
    if (($env.IN_YAZELIX_SHELL? | default "") == "true") {
        print "❌ Error: Desktop launch profiling must be run from outside a Yazelix shell"
        print ""
        print "Open a new terminal outside Yazelix and run: yzx dev profile --desktop"
        return
    }

    print "🚀 Profiling desktop launch startup..."
    let source_info = (resolve_profile_source_root)
    let yzx_cli = (get_profile_cli_path $source_info.root)
    let yzx_control_bin = (resolve_yzx_control_path $source_info.root)
    let meta_json = (build_profile_metadata "desktop_launch" false $source_info | to json -r)
    let profile_run = (^$yzx_control_bin profile create-run "desktop_launch" --metadata $meta_json | from json)

    let profile_env = (build_profile_env $profile_run $source_info.root)
    let result = (with-env $profile_env {
        do { ^sh $yzx_cli desktop launch } | complete
    })

    if $result.exit_code != 0 {
        let error_output = ($result.stderr | default $result.stdout | default "desktop launch profiling exited unsuccessfully")
        error make {
            msg: $"Desktop launch profiling failed: ($error_output)"
        }
    }

    wait_for_profile_handoff $profile_run "Desktop launch"

    ^$yzx_control_bin profile print-report $profile_run.report_path
}

def run_launch_profile_command [
    --clear-cache
    --terminal(-t): string = ""
    --verbose
] {
    if (($env.IN_YAZELIX_SHELL? | default "") == "true") {
        print "❌ Error: Managed launch profiling must be run from outside a Yazelix shell"
        print ""
        print "Open a new terminal outside Yazelix and run: yzx dev profile --launch"
        return
    }

    print "🚀 Profiling managed new-window launch..."
    let source_info = (resolve_profile_source_root)
    let yzx_cli = (get_profile_cli_path $source_info.root)
    let yzx_control_bin = (resolve_yzx_control_path $source_info.root)
    let meta_json = (build_profile_metadata "managed_launch" ($clear_cache | default false) $source_info | to json -r)
    let profile_run = (^$yzx_control_bin profile create-run "managed_launch" --metadata $meta_json | from json)

    if $clear_cache {
        clear_startup_profile_caches
    }

    let profile_env = (build_profile_env $profile_run $source_info.root)
    mut launch_args = [$yzx_cli, "launch"]
    if ($terminal | is-not-empty) {
        $launch_args = ($launch_args | append "--terminal" | append $terminal)
    }
    if $verbose {
        $launch_args = ($launch_args | append "--verbose")
    }
    let resolved_launch_args = $launch_args

    let result = (with-env $profile_env {
        do { ^sh ...$resolved_launch_args } | complete
    })

    if $result.exit_code != 0 {
        let error_output = ($result.stderr | default $result.stdout | default "managed launch profiling exited unsuccessfully")
        error make {
            msg: $"Managed launch profiling failed: ($error_output)"
        }
    }

    wait_for_profile_handoff $profile_run "Managed launch"

    ^$yzx_control_bin profile print-report $profile_run.report_path
}

# Profile launch sequence and identify bottlenecks
export def "yzx dev profile" [
    --cold(-c)        # Profile cold launch from vanilla terminal (emulates desktop entry or fresh terminal launch)
    --desktop         # Profile desktop entry launch path
    --launch          # Profile managed new-window launch path
    --clear-cache     # Clear recorded runtime/project cache state first so the profiled run exercises the rebuild-heavy path
    --terminal(-t): string = ""  # Override terminal selection for launch profiling
    --verbose         # Enable verbose logging for launch profiling
] {
    if $desktop {
        run_desktop_profile_command
    } else if $launch {
        run_launch_profile_command --clear-cache=$clear_cache --terminal=$terminal --verbose=$verbose
    } else if $cold {
        run_cold_profile_command $clear_cache
    } else {
        run_default_profile_command
    }
}

# Lint Nushell scripts with repo-tuned nu-lint config from the maintainer tool surface
export def "yzx dev lint_nu" [
    --format(-f): string = "pretty"  # Output format: pretty or compact
    ...paths: string                 # Specific files or directories (default: nushell/)
] {
    let repo_root = (require_yazelix_repo_root)
    mut args = ["lint-nu", "--format", $format]
    if ($paths | is-not-empty) {
        $args = ($args | append $paths)
    }
    run_repo_maintainer_checked $repo_root "Yazelix Rust nu-lint runner failed" ...$args
}
