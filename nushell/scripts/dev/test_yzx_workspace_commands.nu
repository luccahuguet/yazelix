#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md
# Defends: docs/workspace_session_contract.md

use ./yzx_test_helpers.nu [CLEAN_ZELLIJ_ENV_PREFIX get_repo_config_dir get_repo_root repo_path setup_managed_config_fixture]

def run_nu_snippet [snippet: string, extra_env?: record] {
    if ($extra_env | is-empty) {
        ^nu -c $snippet | complete
    } else {
        with-env $extra_env {
            ^nu -c $snippet | complete
        }
    }
}

def setup_cli_probe_fixture [label: string] {
    let tmpdir = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let fake_home = ($tmpdir | path join "home")
    let fake_profile_bin = ($fake_home | path join ".local" "state" "nix" "profile" "bin")
    let nu_log = ($tmpdir | path join "nu_invocation.txt")

    mkdir $fake_profile_bin

    {
        tmpdir: $tmpdir
        fake_home: $fake_home
        fake_profile_bin: $fake_profile_bin
        nu_log: $nu_log
    }
}

def write_probe_nu [probe_path: string, script_lines: list<string>] {
    $script_lines | str join "\n" | save --force --raw $probe_path
    ^chmod +x $probe_path
}

def read_probe_lines [log_path: string] {
    if ($log_path | path exists) {
        open --raw $log_path | lines
    } else {
        []
    }
}

def read_probe_string [log_path: string] {
    if ($log_path | path exists) {
        open --raw $log_path | str trim
    } else {
        ""
    }
}

def install_argument_logging_probe [fixture: record] {
    write_probe_nu ($fixture.fake_profile_bin | path join "nu") [
        "#!/bin/sh"
        ": > \"$NU_LOG\""
        "for arg in \"$@\"; do"
        "  printf '%s\n' \"$arg\" >> \"$NU_LOG\""
        "done"
        "exit 0"
    ]
}

def setup_desktop_runtime_probe_fixture [label: string, --with_hidden_launch_module] {
    let cli_fixture = (setup_cli_probe_fixture $label)
    let runtime_store = ($cli_fixture.tmpdir | path join "runtime_store")
    let runtime_reference_root = ($cli_fixture.fake_home | path join ".local" "share" "yazelix" "runtime")
    let runtime_dir = ($runtime_store | path expand)

    mkdir ($runtime_dir | path join "nushell" "scripts" "core")
    mkdir $runtime_reference_root

    ^ln -s $runtime_dir ($runtime_reference_root | path join "current")
    ^ln -s (repo_path "nushell" "scripts" "core" "launch_yazelix.nu") ($runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
    if $with_hidden_launch_module {
        mkdir ($runtime_dir | path join "nushell" "scripts" "yzx")
        ^ln -s (repo_path "nushell" "scripts" "yzx" "launch.nu") ($runtime_dir | path join "nushell" "scripts" "yzx" "launch.nu")
    }
    ^ln -s (repo_path ".taplo.toml") ($runtime_dir | path join ".taplo.toml")
    ^ln -s (repo_path "yazelix_default.toml") ($runtime_dir | path join "yazelix_default.toml")

    $cli_fixture | merge {
        runtime_store: $runtime_store
        runtime_reference_root: $runtime_reference_root
        runtime_dir: $runtime_dir
    }
}

def setup_launch_path_fixture [label: string, persistent_sessions: bool, existing_session: bool] {
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let fake_bin = ($tmp_home | path join "bin")
    let zellij_log = ($tmp_home | path join "zellij.log")
    let existing_session_flag = if $existing_session { "true" } else { "false" }
    let real_nu = (which nu | get -o 0.path)

    mkdir $runtime_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir
    mkdir ($tmp_home | path join ".local" "share")
    mkdir $state_dir
    mkdir $fake_bin

    for entry in [".taplo.toml", "nushell", "shells", "configs", "config_metadata", "devenv.lock", "yazelix_default.toml", "docs", "CHANGELOG.md", "assets"] {
        ^ln -s (repo_path $entry) ($runtime_dir | path join $entry)
    }

    [
        "[core]"
        "skip_welcome_screen = true"
        "recommended_deps = true"
        "yazi_extensions = true"
        "yazi_media = false"
        ""
        "[zellij]"
        $"persistent_sessions = ($persistent_sessions)"
        'session_name = "yazelix"'
        ""
        "[shell]"
        'default_shell = "nu"'
    ] | str join "\n" | save --force --raw ($user_config_dir | path join "yazelix.toml")

    [
        "#!/bin/sh"
        'log="$TMP_ZELLIJ_LOG"'
        'cmd="$1"'
        'shift'
        'case "$cmd" in'
        '  setup)'
        '    if [ "$1" = "--dump-config" ]; then'
        "      cat <<'KDL'"
        "keybinds clear-defaults=true {}"
        "themes {}"
        "KDL"
        "      exit 0"
        "    fi"
        "    ;;"
        "  list-sessions)"
        $"    if [ \"($existing_session_flag)\" = \"true\" ]; then"
        "      printf '%s\\n' 'yazelix [Created 1s ago]'"
        "    fi"
        "    exit 0"
        "    ;;"
        "  options|attach)"
        "    printf '%s\\n' \"$cmd $*\" >> \"$log\""
        "    exit 0"
        "    ;;"
        "  *)"
        "    printf '%s\\n' \"$cmd $*\" >> \"$log\""
        "    exit 0"
        "    ;;"
        "esac"
    ] | str join "\n" | save --force --raw ($fake_bin | path join "zellij")
    ^chmod +x ($fake_bin | path join "zellij")
    ^ln -s $real_nu ($fake_bin | path join "nu")

    {
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        state_dir: $state_dir
        fake_bin: $fake_bin
        zellij_log: $zellij_log
        start_inner: ($runtime_dir | path join "nushell" "scripts" "core" "start_yazelix_inner.nu")
        layout_path: ($state_dir | path join "configs" "zellij" "layouts" "yzx_side.kdl")
        env: {
            HOME: $tmp_home
            PATH: ([$fake_bin] | append $env.PATH)
            TMP_ZELLIJ_LOG: $zellij_log
            YAZELIX_RUNTIME_DIR: $runtime_dir
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
        }
    }
}

def setup_enter_forwarding_fixture [label: string] {
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let scripts_dir = ($runtime_dir | path join "nushell" "scripts")
    let yzx_dir = ($scripts_dir | path join "yzx")
    let core_dir = ($scripts_dir | path join "core")
    let utils_dir = ($scripts_dir | path join "utils")
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let call_log = ($tmp_home | path join "start_yazelix_session.json")

    mkdir $runtime_dir
    mkdir ($runtime_dir | path join "nushell")
    mkdir $scripts_dir
    mkdir $yzx_dir
    mkdir $core_dir
    mkdir $utils_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir

    ^ln -s (repo_path "nushell" "scripts" "yzx" "launch.nu") ($yzx_dir | path join "launch.nu")
    ^ln -s (repo_path "nushell" "scripts" "yzx" "enter.nu") ($yzx_dir | path join "enter.nu")

    (open --raw (repo_path "yazelix_default.toml")) | save --force --raw ($user_config_dir | path join "yazelix.toml")
    "" | save --force --raw ($runtime_dir | path join "yazelix_default.toml")

    [
        "export def start_yazelix_session [cwd_override?: string, --verbose, --setup-only, --reuse, --skip-refresh, --force-reenter] {"
        "    {"
        "        cwd_override: ($cwd_override | default null)"
        "        verbose: $verbose"
        "        setup_only: $setup_only"
        "        reuse: $reuse"
        "        skip_refresh: $skip_refresh"
        "        force_reenter: $force_reenter"
        "    } | to json -r | save --force --raw $env.YAZELIX_TEST_LAUNCH_LOG"
        "}"
    ] | str join "\n" | save --force --raw ($core_dir | path join "start_yazelix.nu")

    [
        "export def compute_config_state [] {"
        "    {config: {}, config_file: \"\", combined_hash: \"\", cached_hash: \"\", needs_refresh: false}"
        "}"
        "export def record_materialized_state [state: record] {}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "config_state.nu")

    let bootstrap_stub = ([
        "export def prepare_environment [--verbose] {"
        "    error make {msg: \"PREPARE_ENVIRONMENT_SHOULD_NOT_RUN\"}"
        "}"
        "export def ensure_environment_available [] {"
        "    error make {msg: \"ENSURE_ENVIRONMENT_AVAILABLE_SHOULD_NOT_RUN\"}"
        "}"
    ] | str join "\n")
    $bootstrap_stub | save --force --raw ($utils_dir | path join "environment_bootstrap.nu")

    let backend_stub = ([
        "export def check_environment_status [] {"
        "    {already_in_env: false, in_nix_shell: false, in_yazelix_shell: false}"
        "}"
        "export def advance_runtime_state_after_rebuild [runtime_state: record] {"
        "    $runtime_state"
        "}"
        "export def rebuild_yazelix_environment ["
        "    --max-jobs: string = \"\""
        "    --build-cores: string = \"\""
        "    --refresh-eval-cache"
        "    --output-mode: string = \"normal\""
        "] {"
        "    error make {msg: \"REBUILD_ENVIRONMENT_SHOULD_NOT_RUN\"}"
        "}"
        "export def run_in_devenv_shell_command ["
        "    command: string"
        "    ...args: string"
        "    --max-jobs: string = \"\""
        "    --build-cores: string = \"\""
        "    --cwd: string = \"\""
        "    --runtime-dir: string = \"\""
        "    --env-only"
        "    --force-shell"
        "    --skip-welcome"
        "    --force-refresh"
        "    --verbose"
        "    --refresh-output-mode: string = \"normal\""
        "] {"
        "    error make {msg: \"DEVENV_RUNNER_SHOULD_NOT_RUN\"}"
        "}"
        "export def get_refresh_output_mode [config: any] {"
        "    \"normal\""
        "}"
        "export def resolve_refresh_request [needs_refresh: bool, --reuse, --skip-refresh] {"
        "    { should_refresh: ($needs_refresh and (not $reuse) and (not $skip_refresh)), mode: \"noop\" }"
        "}"
        "export def resolve_runtime_entry_state [refresh_request: record, --already-in-env, --in-yazelix-shell, --force-reenter] {"
        "    {"
        "        activation_surface: (if $in_yazelix_shell { \"live_yazelix_session\" } else if $already_in_env { \"ambient_backend_shell\" } else { \"external_process\" })"
        "        refresh_transition: (if ($refresh_request.should_refresh? | default false) { \"rebuild\" } else { \"none\" })"
        "        profile_request: \"none\""
        "        force_reenter: $force_reenter"
        "    }"
        "}"
        "export def resolve_launch_transition [runtime_state: record, --current-session-eligible, --profile-available] {"
        "    { execution: \"backend_shell\", profile_source: \"none\", rebuild_before_exec: false }"
        "}"
        "export def print_refresh_request_guidance [refresh_request: record] { null }"
    ] | str join "\n")
    $backend_stub | save --force --raw ($utils_dir | path join "devenv_backend.nu")

    [
        "export def get_launch_env [config: record, profile_path: string] { {} }"
        "export def get_launch_profile [config_state: record, --allow-stale] { null }"
        "export def require_reused_launch_profile [config_state: record, context: string] { \"/tmp/fake-profile\" }"
        "export def resolve_runtime_owned_profile [] { \"\" }"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "launch_state.nu")

    [
        "export def print_runtime_version_drift_warning [] {}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "doctor.nu")

    [
        "export def run_entrypoint_config_migration_preflight [entrypoint_label: string, --allow-noninteractive] { null }"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "entrypoint_config_migrations.nu")

    [
        "export def require_yazelix_runtime_dir [] {"
        "    $env.YAZELIX_RUNTIME_DIR"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "common.nu")

    [
        "export def describe_build_parallelism [build_cores: string, max_jobs: string] {"
        "    \"1 job x 1 core/job\""
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "build_policy.nu")

    [
        "export const TERMINAL_METADATA = {}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "constants.nu")

    [
        "export def check_runtime_script [script_path: string, field: string, label: string, context: string] {"
        "    {path: $script_path}"
        "}"
        "export def require_runtime_check [check: record] {"
        "    $check"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "runtime_contract_checker.nu")

    [
        "export def ensure_nix_available [] {}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "nix_detector.nu")

    {
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        call_log: $call_log
        launch_script: ($yzx_dir | path join "launch.nu")
        enter_script: ($yzx_dir | path join "enter.nu")
    }
}

def setup_refresh_activation_fixture [label: string] {
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let runtime_dir = ($tmp_home | path join "runtime")
    let scripts_dir = ($runtime_dir | path join "nushell" "scripts")
    let core_dir = ($scripts_dir | path join "core")
    let utils_dir = ($scripts_dir | path join "utils")
    let setup_dir = ($scripts_dir | path join "setup")
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let layouts_dir = ($state_dir | path join "configs" "zellij" "layouts")
    let call_log = ($tmp_home | path join "startup_calls.log")
    let fake_nu = ($tmp_home | path join "fake_nu.sh")
    let start_script = ($core_dir | path join "start_yazelix.nu")
    let layout_path = ($layouts_dir | path join "yzx_side.kdl")

    mkdir $runtime_dir
    mkdir ($runtime_dir | path join "nushell")
    mkdir $scripts_dir
    mkdir $core_dir
    mkdir $utils_dir
    mkdir $setup_dir
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir
    mkdir ($tmp_home | path join ".local" "share")
    mkdir $state_dir
    mkdir ($state_dir | path join "configs")
    mkdir ($state_dir | path join "configs" "zellij")
    mkdir $layouts_dir

    ^ln -s (repo_path "nushell" "scripts" "core" "start_yazelix.nu") $start_script

    (open --raw (repo_path "yazelix_default.toml")) | save --force --raw ($user_config_dir | path join "yazelix.toml")
    "" | save --force --raw ($runtime_dir | path join "yazelix_default.toml")
    "" | save --force --raw ($setup_dir | path join "environment.nu")
    "" | save --force --raw ($core_dir | path join "start_yazelix_inner.nu")
    "" | save --force --raw $layout_path

    [
        "#!/bin/sh"
        'printf "nu" >> "$YAZELIX_TEST_CALL_LOG"'
        'for arg in "$@"; do'
        '  printf "\t%s" "$arg" >> "$YAZELIX_TEST_CALL_LOG"'
        'done'
        'printf "\n" >> "$YAZELIX_TEST_CALL_LOG"'
        "exit 0"
    ] | str join "\n" | save --force --raw $fake_nu
    ^chmod +x $fake_nu

    let bootstrap_stub = ([
        "export def prepare_environment [--verbose] {"
        "    {"
        "        config: {max_jobs: \"half\", build_cores: \"2\", refresh_output: \"normal\"}"
        "        config_state: {needs_refresh: true, combined_hash: \"fresh-hash\"}"
        "        needs_refresh: true"
        "    }"
        "}"
        "export def ensure_environment_available [] {}"
    ] | str join "\n")
    $bootstrap_stub | save --force --raw ($utils_dir | path join "environment_bootstrap.nu")

    let backend_stub = ([
        "export def check_environment_status [] {"
        "    {already_in_env: false, in_nix_shell: false, in_yazelix_shell: false}"
        "}"
        "export def rebuild_yazelix_environment ["
        "    --max-jobs: string = \"\""
        "    --build-cores: string = \"\""
        "    --refresh-eval-cache"
        "    --output-mode: string = \"normal\""
        "] {"
        "    \"rebuild\" | save --append --raw $env.YAZELIX_TEST_CALL_LOG"
        "    \"\\n\" | save --append --raw $env.YAZELIX_TEST_CALL_LOG"
        "}"
        "export def run_in_devenv_shell_command ["
        "    command: string"
        "    ...args: string"
        "    --max-jobs: string = \"\""
        "    --build-cores: string = \"\""
        "    --cwd: string = \"\""
        "    --runtime-dir: string = \"\""
        "    --env-only"
        "    --force-shell"
        "    --skip-welcome"
        "    --force-refresh"
        "    --verbose"
        "    --refresh-output-mode: string = \"normal\""
        "] {"
        "    error make {msg: \"DEVENV_RUNNER_SHOULD_NOT_RUN\"}"
        "}"
        "export def get_refresh_output_mode [config: any] {"
        "    \"normal\""
        "}"
        "export def resolve_refresh_request [needs_refresh: bool, --reuse, --skip-refresh] {"
        "    { should_refresh: ($needs_refresh and (not $reuse) and (not $skip_refresh)), mode: \"noop\" }"
        "}"
        "export def resolve_runtime_entry_state [refresh_request: record, --already-in-env, --in-yazelix-shell, --force-reenter] {"
        "    {"
        "        activation_surface: (if $in_yazelix_shell { \"live_yazelix_session\" } else if $already_in_env { \"ambient_backend_shell\" } else { \"external_process\" })"
        "        refresh_transition: (if ($refresh_request.should_refresh? | default false) { \"rebuild\" } else { \"none\" })"
        "        profile_request: (if $force_reenter { \"none\" } else { \"verified_recorded_profile\" })"
        "        force_reenter: $force_reenter"
        "    }"
        "}"
        "export def resolve_startup_transition [runtime_state: record, --profile-available] {"
        "    if (($runtime_state.refresh_transition? | default \"none\") == \"rebuild\") {"
        "        { execution: \"activated_profile\", profile_source: \"fresh_runtime_profile\", rebuild_before_exec: true }"
        "    } else if $profile_available {"
        "        { execution: \"activated_profile\", profile_source: ($runtime_state.profile_request? | default \"verified_recorded_profile\"), rebuild_before_exec: false }"
        "    } else {"
        "        { execution: \"backend_shell\", profile_source: \"none\", rebuild_before_exec: false }"
        "    }"
        "}"
        "export def print_refresh_request_guidance [refresh_request: record] { null }"
    ] | str join "\n")
    $backend_stub | save --force --raw ($utils_dir | path join "devenv_backend.nu")

    [
        "export def run_entrypoint_config_migration_preflight [entrypoint_label: string, --allow-noninteractive] { null }"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "entrypoint_config_migrations.nu")

    [
        "export def --env activate_launch_profile [config: record, profile_path: string] {"
        "    $\"activate\\t($profile_path)\" | save --append --raw $env.YAZELIX_TEST_CALL_LOG"
        "    \"\\n\" | save --append --raw $env.YAZELIX_TEST_CALL_LOG"
        "    load-env {DEVENV_PROFILE: $profile_path, IN_YAZELIX_SHELL: \"true\", YAZELIX_RUNTIME_DIR: $env.YAZELIX_RUNTIME_DIR}"
        "}"
        "export def get_launch_profile [config_state: record, --allow-stale] { null }"
        "export def require_reused_launch_profile [config_state: record, context: string] { \"/tmp/reused-profile\" }"
        "export def resolve_runtime_owned_profile [] { \"/tmp/fresh-profile\" }"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "launch_state.nu")

    [
        "export def require_yazelix_runtime_dir [] {"
        "    $env.YAZELIX_RUNTIME_DIR"
        "}"
        "export def resolve_yazelix_nu_bin [] {"
        "    $env.YAZELIX_TEST_NU_BIN"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "common.nu")

    [
        "export def describe_build_parallelism [build_cores: string, max_jobs: string] {"
        "    \"1 job x 1 core/job\""
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "build_policy.nu")

    [
        "export def profile_startup_step [phase: string, step: string, code: closure, metadata?: record] {"
        "    do $code"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "startup_profile.nu")

    [
        "export def check_generated_layout [layout_path: string, context: string] {"
        "    {path: $layout_path}"
        "}"
        "export def check_runtime_script [script_path: string, field: string, label: string, context: string] {"
        "    {path: $script_path}"
        "}"
        "export def check_startup_working_dir [working_dir: string] {"
        "    {path: $working_dir}"
        "}"
        "export def require_runtime_check [check: record] {"
        "    $check"
        "}"
        "export def resolve_expected_layout_path [config: record, layouts_dir: string] {"
        "    $env.YAZELIX_TEST_LAYOUT_PATH"
        "}"
    ] | str join "\n" | save --force --raw ($utils_dir | path join "runtime_contract_checker.nu")

    {
        tmp_home: $tmp_home
        runtime_dir: $runtime_dir
        config_dir: $config_dir
        state_dir: $state_dir
        call_log: $call_log
        fake_nu: $fake_nu
        start_script: $start_script
        layout_path: $layout_path
    }
}

# Defends: startup rejects a missing working directory.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_startup_rejects_missing_working_dir [] {
    print "🧪 Testing startup rejects missing working directories..."

    try {
        let start_script = (repo_path "nushell" "scripts" "core" "start_yazelix.nu")
        let snippet = ([
            $"source \"($start_script)\""
            'try {'
            '    validate_startup_working_dir "/tmp/yazelix_missing_start_dir" | ignore'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Startup directory does not exist") {
            print "  ✅ Startup path validation fails early for missing directories"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Defends: launch rejects a file path as the working directory.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_rejects_file_working_dir [] {
    print "🧪 Testing launch rejects file paths as working directories..."

    let tmpdir = (^mktemp -d /tmp/yazelix_launch_path_test_XXXXXX | str trim)

    let result = (try {
        let file_path = ($tmpdir | path join "not_a_dir.txt")
        "hello" | save --force --raw $file_path
        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            'try {'
            '    validate_launch_working_dir $env.YAZELIX_TEST_FILE_PATH | ignore'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet {YAZELIX_TEST_FILE_PATH: $file_path})
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Launch path is not a directory") {
            print "  ✅ Launch path validation rejects files before terminal startup"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: desktop launch ignores hostile inherited shell env.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_cli_desktop_launch_ignores_hostile_shell_env [] {
    print "🧪 Testing yzx CLI desktop launch ignores hostile shell env..."

    let fixture = (setup_cli_probe_fixture "yazelix_posix_desktop_env")

    let result = (try {
        let env_file = ($fixture.tmpdir | path join "env.sh")

        write_probe_nu ($fixture.fake_profile_bin | path join "nu") [
            "#!/bin/sh"
            $"printf '%s\\n' \"$*\" > '($fixture.nu_log)'"
            "exit 0"
        ]

        [
            "echo SHOULD_NOT_SOURCE_ENV >&2"
            "exit 94"
        ] | str join "\n" | save --force --raw $env_file

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {HOME: $fixture.fake_home, BASH_ENV: $env_file, ENV: $env_file} {
            ^$launcher_script desktop launch | complete
        })
        let stderr = ($output.stderr | str trim)
        let nu_invocation = (read_probe_string $fixture.nu_log)

        if ($output.exit_code == 0) and ($stderr == "") and ($nu_invocation | str contains "yzx desktop launch") {
            print "  ✅ yzx CLI reaches desktop launch without sourcing hostile shell env files"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr) nu=($nu_invocation)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: desktop launch must use the installed core launch fast path with a clean external-launch environment.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_launch_uses_leaf_launch_module_with_clean_env [] {
    print "🧪 Testing yzx desktop launch uses the installed core launch fast path with a clean external-launch env..."

    let fixture = (setup_desktop_runtime_probe_fixture "yazelix_desktop_leaf_runtime")

    let result = (try {
        write_probe_nu ($fixture.fake_profile_bin | path join "nu") [
            "#!/bin/sh"
            $"printf '%s\\n' \"$1\" > '($fixture.nu_log)'"
            "shift"
            "for arg in \"$@\"; do"
            $"  printf '%s\\n' \"$arg\" >> '($fixture.nu_log)'"
            "done"
            $"printf 'YAZELIX_RUNTIME_DIR=%s\\n' \"${YAZELIX_RUNTIME_DIR-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZELIX_DIR=%s\\n' \"${YAZELIX_DIR-unset}\" >> '($fixture.nu_log)'"
            $"printf 'DEVENV_PROFILE=%s\\n' \"${DEVENV_PROFILE-unset}\" >> '($fixture.nu_log)'"
            $"printf 'DEVENV_ROOT=%s\\n' \"${DEVENV_ROOT-unset}\" >> '($fixture.nu_log)'"
            $"printf 'IN_YAZELIX_SHELL=%s\\n' \"${IN_YAZELIX_SHELL-unset}\" >> '($fixture.nu_log)'"
            $"printf 'IN_NIX_SHELL=%s\\n' \"${IN_NIX_SHELL-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZELIX_TERMINAL=%s\\n' \"${YAZELIX_TERMINAL-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZELIX_MENU_POPUP=%s\\n' \"${YAZELIX_MENU_POPUP-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZELIX_POPUP_PANE=%s\\n' \"${YAZELIX_POPUP_PANE-unset}\" >> '($fixture.nu_log)'"
            $"printf 'ZELLIJ_SESSION_NAME=%s\\n' \"${ZELLIJ_SESSION_NAME-unset}\" >> '($fixture.nu_log)'"
            $"printf 'YAZI_ID=%s\\n' \"${YAZI_ID-unset}\" >> '($fixture.nu_log)'"
            "exit 0"
        ]

        let desktop_script = (repo_path "nushell" "scripts" "yzx" "desktop.nu")
        let output = (with-env {
            HOME: $fixture.fake_home
            YAZELIX_NU_BIN: ($fixture.fake_profile_bin | path join "nu")
            YAZELIX_RUNTIME_DIR: ($fixture.tmpdir | path join "hostile_runtime")
            YAZELIX_DIR: "/hostile/legacy_runtime"
            DEVENV_PROFILE: ($fixture.tmpdir | path join "hostile_profile")
            DEVENV_ROOT: (repo_path)
            IN_YAZELIX_SHELL: "true"
            IN_NIX_SHELL: "impure"
            YAZELIX_TERMINAL: "ghostty"
            YAZELIX_MENU_POPUP: "true"
            YAZELIX_POPUP_PANE: "true"
            ZELLIJ_SESSION_NAME: "yazelix"
            YAZI_ID: "1234"
        } {
            ^nu -c $"use \"($desktop_script)\" *; yzx desktop launch" | complete
        })
        let stderr = ($output.stderr | str trim)
        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_launch_module = ($fixture.runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
        let expected_env = [
            $"YAZELIX_RUNTIME_DIR=($fixture.runtime_dir)"
            "YAZELIX_DIR=unset"
            "DEVENV_PROFILE=unset"
            "DEVENV_ROOT=unset"
            "IN_YAZELIX_SHELL=unset"
            "IN_NIX_SHELL=unset"
            "YAZELIX_TERMINAL=unset"
            "YAZELIX_MENU_POPUP=unset"
            "YAZELIX_POPUP_PANE=unset"
            "ZELLIJ_SESSION_NAME=unset"
            "YAZI_ID=unset"
        ]

        if (
            ($output.exit_code == 0)
            and ($stderr == "")
            and (($invocation | get -o 0 | default "") == $expected_launch_module)
            and (($invocation | get -o 1 | default "") == $fixture.fake_home)
            and (($invocation | get -o 2 | default "") == "--desktop-fast-path")
            and (($invocation | skip 3) == $expected_env)
        ) {
            print "  ✅ yzx desktop launch uses the installed core launch fast path and ignores inherited shell state"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr) invocation=(($invocation | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: desktop launch should fall back to the standard hidden-wait path only when no visible bootstrap terminal exists.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_desktop_launch_falls_back_to_hidden_wait_when_no_visible_bootstrap_terminal_exists [] {
    print "🧪 Testing yzx desktop launch falls back to the hidden-wait path only after the fast path reports no visible bootstrap terminal..."

    let fixture = (setup_desktop_runtime_probe_fixture "yazelix_desktop_fallback_runtime" --with_hidden_launch_module)

    let result = (try {
        let fast_launch_module = ($fixture.runtime_dir | path join "nushell" "scripts" "core" "launch_yazelix.nu")
        write_probe_nu ($fixture.fake_profile_bin | path join "nu") [
            "#!/bin/sh"
            "call_type=unknown"
            $"if [ \"$1\" = \"($fast_launch_module)\" ]; then"
            "  call_type=fast"
            "fi"
            $"printf '[%s]\\n' \"$call_type\" >> '($fixture.nu_log)'"
            $"printf '%s\\n' \"$1\" >> '($fixture.nu_log)'"
            "shift"
            "for arg in \"$@\"; do"
            $"  printf '%s\\n' \"$arg\" >> '($fixture.nu_log)'"
            "done"
            $"printf 'YAZELIX_RUNTIME_DIR=%s\\n' \"${YAZELIX_RUNTIME_DIR-unset}\" >> '($fixture.nu_log)'"
            $"printf 'DEVENV_PROFILE=%s\\n' \"${DEVENV_PROFILE-unset}\" >> '($fixture.nu_log)'"
            $"printf 'IN_YAZELIX_SHELL=%s\\n' \"${IN_YAZELIX_SHELL-unset}\" >> '($fixture.nu_log)'"
            "if [ \"$call_type\" = fast ]; then"
            "  printf 'Failure class: desktop-bootstrap-unavailable.\\n' >&2"
            "  exit 91"
            "fi"
            "exit 0"
        ]

        let desktop_script = (repo_path "nushell" "scripts" "yzx" "desktop.nu")
        let output = (with-env {
            HOME: $fixture.fake_home
            YAZELIX_NU_BIN: ($fixture.fake_profile_bin | path join "nu")
            DEVENV_PROFILE: ($fixture.tmpdir | path join "hostile_profile")
            IN_YAZELIX_SHELL: "true"
            YAZELIX_RUNTIME_DIR: ($fixture.tmpdir | path join "hostile_runtime")
        } {
            ^nu -c $"use \"($desktop_script)\" *; yzx desktop launch" | complete
        })
        let stderr = ($output.stderr | str trim)
        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_launch_module = ($fixture.runtime_dir | path join "nushell" "scripts" "yzx" "launch.nu")
        let expected_launch_command = $"use \"($expected_launch_module)\" *; yzx launch --home"

        if (
            ($output.exit_code == 0)
            and ($stderr == "")
            and (($invocation | get -o 0 | default "") == "[fast]")
            and (($invocation | get -o 1 | default "") == $fast_launch_module)
            and (($invocation | get -o 2 | default "") == $fixture.fake_home)
            and (($invocation | get -o 3 | default "") == "--desktop-fast-path")
            and (($invocation | get -o 4 | default "") == $"YAZELIX_RUNTIME_DIR=($fixture.runtime_dir)")
            and (($invocation | get -o 5 | default "") == "DEVENV_PROFILE=unset")
            and (($invocation | get -o 6 | default "") == "IN_YAZELIX_SHELL=unset")
            and (($invocation | get -o 7 | default "") == "[unknown]")
            and (($invocation | get -o 8 | default "") == "-c")
            and (($invocation | get -o 9 | default "") == $expected_launch_command)
            and (($invocation | get -o 10 | default "") == $"YAZELIX_RUNTIME_DIR=($fixture.runtime_dir)")
            and (($invocation | get -o 11 | default "") == "DEVENV_PROFILE=unset")
            and (($invocation | get -o 12 | default "") == "IN_YAZELIX_SHELL=unset")
        ) {
            print "  ✅ Desktop launch only falls back to hidden wait after the fast path reports no visible bootstrap terminal"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr) invocation=(($invocation | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: desktop fast path must not silently swap an explicit requested terminal for a different bootstrap terminal.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_desktop_fast_path_rejects_bootstrap_terminal_substitution_for_explicit_terminal [] {
    print "🧪 Testing desktop fast path refuses to substitute a different terminal when one was explicitly requested..."

    let tmpdir = (^mktemp -d /tmp/yazelix_desktop_terminal_override_XXXXXX | str trim)
    let real_nu = (which nu | get -o 0.path)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        mkdir $fake_bin

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "ghostty")
        ^chmod +x ($fake_bin | path join "ghostty")

        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            "try {"
            "    resolve_desktop_fast_path_candidates 'kitty' ['ghostty', 'kitty'] true true | ignore"
            "} catch {|err|"
            "    print $err.msg"
            "}"
        ] | str join "\n")
        let output = (with-env {
            PATH: $fake_bin
        } {
            ^$real_nu -c $snippet | complete
        })
        let stdout = ($output.stdout | str trim)
        let stderr = ($output.stderr | str trim)

        if (
            ($output.exit_code == 0)
            and ($stderr == "")
            and ($stdout | str contains "requested terminal 'kitty'")
            and ($stdout | str contains "desktop-bootstrap-unavailable")
            and (not ($stdout | str contains "ghostty"))
        ) {
            print "  ✅ Desktop fast path preserves an explicit terminal request instead of silently substituting another terminal"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($stderr)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: desktop fast path must not reuse stale managed wrappers when a rebuild is already needed.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_desktop_fast_path_uses_direct_host_terminal_during_reload_instead_of_stale_wrapper [] {
    print "🧪 Testing desktop fast path uses a direct host terminal during reload instead of a stale managed wrapper..."

    let tmpdir = (^mktemp -d /tmp/yazelix_desktop_wrapper_preference_XXXXXX | str trim)
    let real_nu = (which nu | get -o 0.path)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let config_dir = ($fake_home | path join ".config" "yazelix")
        let state_dir = ($fake_home | path join ".local" "share" "yazelix")
        let runtime_dir = ($tmpdir | path join "runtime")
        let fake_bin = ($tmpdir | path join "bin")
        let profile_dir = ($tmpdir | path join "profile")
        let profile_bin = ($profile_dir | path join "bin")
        let launch_state_path = ($state_dir | path join "state" "launch_state.json")

        mkdir $config_dir
        mkdir ($state_dir | path join "state")
        mkdir $runtime_dir
        mkdir ($runtime_dir | path join "nushell")
        mkdir ($runtime_dir | path join "shells")
        mkdir ($runtime_dir | path join "configs")
        mkdir ($runtime_dir | path join "docs")
        mkdir ($runtime_dir | path join "assets")
        mkdir $fake_bin
        mkdir $profile_bin

        ^ln -s (repo_path ".taplo.toml") ($runtime_dir | path join ".taplo.toml")
        "" | save --force --raw ($runtime_dir | path join "yazelix_default.toml")
        "" | save --force --raw ($runtime_dir | path join "devenv.nix")
        "" | save --force --raw ($runtime_dir | path join "devenv.yaml")
        "" | save --force --raw ($runtime_dir | path join "devenv.lock")
        "" | save --force --raw ($runtime_dir | path join "CHANGELOG.md")
        {
            combined_hash: "ignored-for-fast-path-resolution"
            profile_path: $profile_dir
        } | to json | save --force $launch_state_path

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "ghostty")
        ^chmod +x ($fake_bin | path join "ghostty")

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($profile_bin | path join "yazelix-ghostty")
        ^chmod +x ($profile_bin | path join "yazelix-ghostty")

        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            "let candidates = (resolve_desktop_fast_path_candidates '' ['ghostty'] true true)"
            "print ($candidates | to json -r)"
        ] | str join "\n")
        let output = (with-env {
            HOME: $fake_home
            PATH: ([$fake_bin, "/usr/bin", "/bin"] | str join (char esep))
            YAZELIX_RUNTIME_DIR: $runtime_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_CONFIG_DIR: $config_dir
        } {
            ^$real_nu -c $snippet | complete
        })
        let candidates = ($output.stdout | from json)
        let first_candidate = ($candidates | get -o 0 | default {})

        let chosen_command = ($first_candidate.command? | default "" | into string)

        if (
            ($output.exit_code == 0)
            and (($first_candidate.terminal? | default "") == "ghostty")
            and (($first_candidate.use_wrapper? | default false) == false)
            and (
                ($chosen_command == "ghostty")
                or ($chosen_command == ($fake_bin | path join "ghostty"))
            )
        ) {
            print "  ✅ Desktop fast path now uses a visible host bootstrap terminal instead of reusing a stale managed wrapper during reload"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: current-session and runtime-owned profile policies must stay intentionally distinct and ignore unrelated Zellij activation.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_profile_resolution_policies_separate_runtime_owned_and_current_session_state [] {
    print "🧪 Testing current-session and runtime-owned profile policies stay intentionally distinct..."

    let tmpdir = (^mktemp -d /tmp/yazelix_built_profile_runtime_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        let config_dir = ($fake_home | path join ".config" "yazelix")
        let state_dir = ($fake_home | path join ".local" "share" "yazelix")
        let runtime_dir = ($tmpdir | path join "runtime")
        let runtime_project = ($state_dir | path join "runtime" "project")
        let runtime_project_gc = ($runtime_project | path join ".devenv" "gc")
        let runtime_project_shell = ($runtime_project_gc | path join "shell")
        let embedded_runtime_profile = ($tmpdir | path join "runtime_owned_profile")
        let stale_env_profile = ($tmpdir | path join "stale_profile")
        let launch_state_path = ($state_dir | path join "state" "launch_state.json")
        let launch_state_profile = ($tmpdir | path join "recorded_profile")
        mkdir $config_dir
        mkdir ($state_dir | path join "runtime")
        mkdir ($state_dir | path join "state")
        mkdir $runtime_dir
        mkdir ($runtime_dir | path join "nushell")
        mkdir ($runtime_dir | path join "shells")
        mkdir ($runtime_dir | path join "configs")
        mkdir ($runtime_dir | path join "docs")
        mkdir ($runtime_dir | path join "assets")
        ^ln -s (repo_path ".taplo.toml") ($runtime_dir | path join ".taplo.toml")
        "" | save --force --raw ($runtime_dir | path join "yazelix_default.toml")
        "" | save --force --raw ($runtime_dir | path join "devenv.nix")
        "" | save --force --raw ($runtime_dir | path join "devenv.yaml")
        "" | save --force --raw ($runtime_dir | path join "devenv.lock")
        "" | save --force --raw ($runtime_dir | path join "CHANGELOG.md")
        mkdir ($runtime_project | path join ".devenv")
        mkdir $runtime_project_gc
        ^ln -s ($runtime_dir | path join "devenv.nix") ($runtime_project | path join "devenv.nix")
        mkdir $embedded_runtime_profile
        mkdir $stale_env_profile
        mkdir $launch_state_profile
        [
            "#!/usr/bin/env bash"
            $"declare -x DEVENV_PROFILE=\"($embedded_runtime_profile)\""
        ] | str join "\n" | save --force --raw $runtime_project_shell
        {
            combined_hash: "ignored-for-resolve-built-profile"
            profile_path: $launch_state_profile
        } | to json | save --force $launch_state_path

        let launch_state_module = (repo_path "nushell" "scripts" "utils" "launch_state.nu")
        let snippet = ([
            $"use \"($launch_state_module)\" [resolve_current_session_profile resolve_runtime_owned_profile]"
            "print (resolve_runtime_owned_profile)"
            "print (resolve_current_session_profile)"
        ] | str join "\n")

        let outside_result = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_RUNTIME_DIR: $runtime_dir
            DEVENV_PROFILE: $stale_env_profile
            IN_YAZELIX_SHELL: null
            YAZELIX_TERMINAL: null
        } {
            ^nu -c $snippet | complete
        })
        let zellij_only_result = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_RUNTIME_DIR: $runtime_dir
            DEVENV_PROFILE: $stale_env_profile
            IN_YAZELIX_SHELL: null
            YAZELIX_TERMINAL: null
            ZELLIJ_SESSION_NAME: "not-yazelix"
        } {
            ^nu -c $snippet | complete
        })
        let inside_result = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_RUNTIME_DIR: $runtime_dir
            DEVENV_PROFILE: $stale_env_profile
            IN_YAZELIX_SHELL: null
            YAZELIX_TERMINAL: "ghostty"
        } {
            ^nu -c $snippet | complete
        })
        let outside_lines = ($outside_result.stdout | lines)
        let zellij_only_lines = ($zellij_only_result.stdout | lines)
        let inside_lines = ($inside_result.stdout | lines)
        let outside_runtime_owned = ($outside_lines | get -o 0 | default "" | str trim)
        let outside_current_session = ($outside_lines | get -o 1 | default "" | str trim)
        let zellij_only_runtime_owned = ($zellij_only_lines | get -o 0 | default "" | str trim)
        let zellij_only_current_session = ($zellij_only_lines | get -o 1 | default "" | str trim)
        let inside_runtime_owned = ($inside_lines | get -o 0 | default "" | str trim)
        let inside_current_session = ($inside_lines | get -o 1 | default "" | str trim)

        if (
            ($outside_result.exit_code == 0)
            and ($zellij_only_result.exit_code == 0)
            and ($inside_result.exit_code == 0)
            and ($embedded_runtime_profile | is-not-empty)
            and ($launch_state_profile | is-not-empty)
            and ($outside_runtime_owned == ($launch_state_profile | path expand))
            and ($outside_current_session == $embedded_runtime_profile)
            and ($zellij_only_runtime_owned == ($launch_state_profile | path expand))
            and ($zellij_only_current_session == $embedded_runtime_profile)
            and ($inside_runtime_owned == ($launch_state_profile | path expand))
            and ($inside_current_session == ($stale_env_profile | path expand))
        ) {
            print "  ✅ Runtime-owned resolution now prefers the recorded launch profile over stale runtime-project shell artifacts, unrelated Zellij markers do not count as Yazelix sessions, and current-session resolution still honors the active Yazelix shell profile"
            true
        } else {
            print $"  ❌ Unexpected result: outside_runtime=($outside_runtime_owned) outside_current=($outside_current_session) zellij_runtime=($zellij_only_runtime_owned) zellij_current=($zellij_only_current_session) inside_runtime=($inside_runtime_owned) inside_current=($inside_current_session) outside_stderr=(($outside_result.stderr | str trim)) zellij_stderr=(($zellij_only_result.stderr | str trim)) inside_stderr=(($inside_result.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: yzx edit must ignore stale ambient Helix wrapper paths and derive the canonical managed editor command.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
def test_yzx_edit_resolves_managed_helix_wrapper_from_canonical_launch_env [] {
    print "🧪 Testing yzx edit resolves the managed Helix wrapper from the canonical launch env..."

    let fixture = (setup_managed_config_fixture
        "yazelix_edit_canonical_launch_env"
        (open --raw (repo_path "yazelix_default.toml"))
    )

    let result = (try {
        let helper_script = (repo_path "nushell" "scripts" "utils" "editor_launch_context.nu")
        let repo_root = (repo_path)
        cp (repo_path "yazelix_packs_default.toml") ($fixture.user_config_dir | path join "yazelix_packs.toml")
        let snippet = ([
            $"source \"($helper_script)\""
            "let context = (resolve_editor_launch_context)"
            "print ($context.editor)"
        ] | str join "\n")
        let output = (run_nu_snippet $snippet {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            EDITOR: "/shells/posix/yazelix_hx.sh"
            YAZELIX_RUNTIME_DIR: $repo_root
        })
        let lines = ($output.stdout | lines)
        let expected_editor = ($repo_root | path join "shells" "posix" "yazelix_hx.sh")

        if (
            ($output.exit_code == 0)
            and (($lines | get -o 0 | default "") == $expected_editor)
        ) {
            print "  ✅ yzx edit now ignores stale ambient wrapper paths and resolves the canonical managed editor wrapper"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: yzx reveal must use the lightweight reveal helper instead of bootstrapping the full command suite.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_cli_reveal_uses_lightweight_reveal_helper [] {
    print "🧪 Testing yzx CLI reveal uses the lightweight reveal helper..."

    let fixture = (setup_cli_probe_fixture "yazelix_posix_reveal_cli")

    let result = (try {
        let target_path = ($fixture.tmpdir | path join "target.txt")
        "" | save --force --raw $target_path

        install_argument_logging_probe $fixture

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {
            HOME: $fixture.fake_home
            NU_LOG: $fixture.nu_log
        } {
            ^$launcher_script reveal $target_path | complete
        })

        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_reveal_script = (repo_path "nushell" "scripts" "integrations" "reveal_in_yazi.nu")

        if (
            ($output.exit_code == 0)
            and (($invocation | get -o 0 | default "") == $expected_reveal_script)
            and (($invocation | get -o 1 | default "") == $target_path)
            and not ($invocation | any {|arg| $arg == "-c" })
            and not ($invocation | any {|arg| $arg | str contains "core/yazelix.nu" })
        ) {
            print "  ✅ yzx reveal now dispatches to the lightweight reveal helper instead of the full command suite"
            true
        } else {
            print $"  ❌ Unexpected yzx reveal invocation: exit=($output.exit_code) args=($invocation | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Regression: yzx menu must use the lightweight menu module instead of bootstrapping the full command suite.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_cli_menu_uses_lightweight_menu_module [] {
    print "🧪 Testing yzx CLI menu uses the lightweight menu module..."

    let fixture = (setup_cli_probe_fixture "yazelix_posix_menu_cli")

    let result = (try {
        install_argument_logging_probe $fixture

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {
            HOME: $fixture.fake_home
            NU_LOG: $fixture.nu_log
        } {
            ^$launcher_script menu --popup | complete
        })

        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_menu_script = (repo_path "nushell" "scripts" "yzx" "menu.nu")

        if (
            ($output.exit_code == 0)
            and (($invocation | get -o 0 | default "") == "-c")
            and (($invocation | get -o 1 | default "") | str contains $expected_menu_script)
            and (($invocation | get -o 1 | default "") | str contains "yzx menu --popup")
            and not (($invocation | get -o 1 | default "") | str contains "core/yazelix.nu")
        ) {
            print "  ✅ yzx menu now dispatches through the lightweight menu module instead of the full command suite"
            true
        } else {
            print $"  ❌ Unexpected yzx menu invocation: exit=($output.exit_code) args=($invocation | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Defends: docs/specs/yzx_command_palette_categories.md
# Regression: yzx menu should include most user-facing commands while keeping explicit exclusions for the palette itself, maintainer commands, shell-control commands, and tab-scoped actions.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_yzx_menu_palette_eligibility_is_broad_but_explicit [] {
    print "🧪 Testing yzx menu palette eligibility is broad but explicit..."

    let result = (try {
        let helper_module = (repo_path "nushell" "scripts" "yzx" "menu.nu")
        let output = (^nu -c $"
            source \"($helper_module)\"
            get_palette_menu_items | get id | to json -r
        " | complete)

        let ids = if $output.exit_code == 0 {
            try { $output.stdout | from json } catch { [] }
        } else {
            []
        }

        if (
            ($output.exit_code == 0)
            and ("yzx" in $ids)
            and ("yzx launch" in $ids)
            and ("yzx doctor" in $ids)
            and ("yzx update runtime" in $ids)
            and ("yzx desktop launch" in $ids)
            and (not ("yzx menu" in $ids))
            and (not ("yzx dev update" in $ids))
            and (not ("yzx env" in $ids))
            and (not ("yzx run" in $ids))
            and (not ("yzx cwd" in $ids))
        ) {
            print "  ✅ yzx menu now includes broad user-facing commands while explicitly excluding the palette itself, maintainer commands, shell-control commands, and tab-scoped actions"
            true
        } else {
            print $"  ❌ Unexpected palette eligibility result: exit=($output.exit_code) ids=($ids | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    $result
}

# Regression: yzx enter must use the lightweight enter module instead of bootstrapping the full command suite.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_cli_enter_uses_lightweight_enter_module [] {
    print "🧪 Testing yzx CLI enter uses the lightweight enter module..."

    let fixture = (setup_cli_probe_fixture "yazelix_posix_enter_cli")

    let result = (try {
        let target_dir = ($fixture.tmpdir | path join "project")
        mkdir $target_dir

        install_argument_logging_probe $fixture

        let launcher_script = (repo_path "shells" "posix" "yzx_cli.sh")
        let output = (with-env {
            HOME: $fixture.fake_home
            NU_LOG: $fixture.nu_log
        } {
            ^$launcher_script enter --path $target_dir | complete
        })

        let invocation = (read_probe_lines $fixture.nu_log)
        let expected_enter_script = (repo_path "nushell" "scripts" "yzx" "enter.nu")

        if (
            ($output.exit_code == 0)
            and (($invocation | get -o 0 | default "") == "-c")
            and (($invocation | get -o 1 | default "") | str contains $expected_enter_script)
            and (($invocation | get -o 1 | default "") | str contains "yzx enter --path")
            and not (($invocation | get -o 1 | default "") | str contains "core/yazelix.nu")
        ) {
            print "  ✅ yzx enter now dispatches through the lightweight enter module instead of the full command suite"
            true
        } else {
            print $"  ❌ Unexpected yzx enter invocation: exit=($output.exit_code) args=($invocation | to json -r) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmpdir
    $result
}

# Defends: current-terminal startup uses the requested directory for nonpersistent sessions.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions [] {
    print "🧪 Testing non-persistent current-terminal startup keeps the requested directory for both launch and restart..."

    let fixture = (setup_launch_path_fixture "yazelix_launch_here_path_nonpersistent" false false)

    let result = (try {
        let target_dir = ($fixture.tmp_home | path join "project")
        mkdir $target_dir
        let launch_output = (with-env $fixture.env {
            ^nu $fixture.start_inner $target_dir $fixture.layout_path | complete
        })
        let launch_stdout = ($launch_output.stdout | str trim)
        let launch_stderr = ($launch_output.stderr | str trim)
        let launch_zellij_log = if ($fixture.zellij_log | path exists) { open --raw $fixture.zellij_log | str trim } else { "" }

        let restart_state_dir = ($fixture.tmp_home | path join ".local" "share" "yazelix" "state" "restart")
        mkdir $restart_state_dir
        let restart_bootstrap_file = ($restart_state_dir | path join "sidebar_cwd_restart.txt")
        $target_dir | save --force --raw $restart_bootstrap_file
        "" | save --force --raw $fixture.zellij_log
        let restart_output = (with-env ($fixture.env | merge {
            YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE: $restart_bootstrap_file
        }) {
            ^nu $fixture.start_inner $target_dir $fixture.layout_path | complete
        })
        let restart_stdout = ($restart_output.stdout | str trim)
        let restart_stderr = ($restart_output.stderr | str trim)
        let restart_zellij_log = if ($fixture.zellij_log | path exists) { open --raw $fixture.zellij_log | str trim } else { "" }

        let expected_shell = ($fixture.runtime_dir | path join "shells" "posix" "yazelix_nu.sh")
        let launch_ok = ($launch_output.exit_code == 0) and ($launch_zellij_log | str contains $"options --default-cwd ($target_dir)") and ($launch_zellij_log | str contains $"--default-shell ($expected_shell)") and (not ($launch_stdout | str contains "--path ignored"))
        let restart_ok = ($restart_output.exit_code == 0) and ($restart_zellij_log | str contains $"options --default-cwd ($target_dir)") and ($restart_zellij_log | str contains $"--default-shell ($expected_shell)") and (not ($restart_stdout | str contains "--path ignored"))

        if $launch_ok and $restart_ok {
            print "  ✅ Non-persistent sessions keep the requested directory as Zellij's cwd, including restart bootstrap flows"
            true
        } else {
            print $"  ❌ Unexpected launch result: exit=($launch_output.exit_code) stdout=($launch_stdout) stderr=($launch_stderr) zellij=($launch_zellij_log)"
            print $"  ❌ Unexpected restart result: exit=($restart_output.exit_code) stdout=($restart_stdout) stderr=($restart_stderr) zellij=($restart_zellij_log)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: `yzx enter` forwards refresh ownership into the startup entrypoint instead of re-inspecting state first.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_enter_forwards_refresh_intent_to_startup_entrypoint [] {
    print "🧪 Testing yzx enter forwards refresh intent to the startup entrypoint..."

    let fixture = (setup_enter_forwarding_fixture "yazelix_enter_forwarding")

    let result = (try {
        let target_dir = ($fixture.tmp_home | path join "project")
        mkdir $target_dir

        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_TEST_LAUNCH_LOG: $fixture.call_log
        } {
            ^nu -c $"use \"($fixture.enter_script)\" *; yzx enter --path \"($target_dir)\" --skip-refresh --force-reenter" | complete
        })

        let forwarded = if ($fixture.call_log | path exists) {
            open $fixture.call_log
        } else {
            null
        }
        let stderr = ($output.stderr | str trim)
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stderr == "")
            and ($forwarded != null)
            and ($forwarded.cwd_override == $target_dir)
            and ($forwarded.skip_refresh == true)
            and ($forwarded.force_reenter == true)
            and ($forwarded.reuse == false)
            and ($forwarded.setup_only == false)
            and ($forwarded.verbose == false)
            and (not ($stdout | str contains "PREPARE_ENVIRONMENT_SHOULD_NOT_RUN"))
            and (not ($stdout | str contains "REBUILD_ENVIRONMENT_SHOULD_NOT_RUN"))
        ) {
            print "  ✅ yzx enter now hands skip-refresh and force-reenter intent straight to startup ownership"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($stderr) forwarded=($forwarded | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: refreshed startup should reuse the freshly built profile without replaying shell entry a second time.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_startup_refresh_activates_built_profile_without_second_shell_entry [] {
    print "🧪 Testing refreshed startup activates the built profile without replaying shell entry..."

    let fixture = (setup_refresh_activation_fixture "yazelix_refresh_activation")

    let result = (try {
        let target_dir = ($fixture.tmp_home | path join "project")
        mkdir $target_dir

        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_STATE_DIR: $fixture.state_dir
            YAZELIX_TEST_CALL_LOG: $fixture.call_log
            YAZELIX_TEST_NU_BIN: $fixture.fake_nu
            YAZELIX_TEST_LAYOUT_PATH: $fixture.layout_path
        } {
            ^nu -c $"source \"($fixture.start_script)\"; start_yazelix_session \"($target_dir)\"" | complete
        })

        let call_lines = (read_probe_lines $fixture.call_log)
        let rebuild_seen = ($call_lines | any {|line| $line == "rebuild" })
        let activation_seen = ($call_lines | any {|line| $line == "activate\t/tmp/fresh-profile" })
        let environment_seen = ($call_lines | any {|line| ($line | str contains "setup/environment.nu") })
        let inner_seen = ($call_lines | any {|line|
            ($line | str contains "\t-i") and ($line | str contains "start_yazelix_inner.nu") and ($line | str contains $target_dir) and ($line | str contains $fixture.layout_path)
        })
        let stdout = ($output.stdout | str trim)
        let stderr = ($output.stderr | str trim)

        if (
            ($output.exit_code == 0)
            and $rebuild_seen
            and $activation_seen
            and $inner_seen
            and (not $environment_seen)
            and (not ($stdout | str contains "DEVENV_RUNNER_SHOULD_NOT_RUN"))
            and (not ($stderr | str contains "DEVENV_RUNNER_SHOULD_NOT_RUN"))
        ) {
            print "  ✅ Refreshed startup rebuilds once, activates the fresh profile, and skips the second shell entry replay"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($stderr) calls=($call_lines | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: `yzx launch` no longer accepts current-terminal ownership through `--here`.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_yzx_launch_rejects_removed_here_flag [] {
    print "🧪 Testing yzx launch rejects the removed --here flag..."

    let fixture = (setup_enter_forwarding_fixture "yazelix_launch_rejects_here")

    let result = (try {
        let output = (with-env {
            HOME: $fixture.tmp_home
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.runtime_dir
            YAZELIX_TEST_LAUNCH_LOG: $fixture.call_log
        } {
            ^nu -c $"use \"($fixture.launch_script)\" *; yzx launch --here" | complete
        })

        let stderr = ($output.stderr | str trim)
        let forwarded_exists = ($fixture.call_log | path exists)

        if (
            ($output.exit_code != 0)
            and ($stderr | str contains "doesn't have flag")
            and ($stderr | str contains "--here")
            and (not $forwarded_exists)
        ) {
            print "  ✅ yzx launch now fails clearly when asked to use the removed current-terminal flag"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stderr=($stderr) forwarded_exists=($forwarded_exists)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Invariant: runtime entry state keeps activation surface, refresh transition, and profile intent explicit.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_runtime_entry_state_models_live_session_refresh_intent [] {
    print "🧪 Testing runtime entry state models live-session refresh intent explicitly..."

    try {
        use ../utils/devenv_backend.nu [resolve_runtime_entry_state]

        let runtime_state = (
            resolve_runtime_entry_state
            {should_refresh: true, mode: "refresh"}
            --already-in-env
            --in-yazelix-shell
        )

        if (
            ($runtime_state.activation_surface == "live_yazelix_session")
            and ($runtime_state.refresh_transition == "rebuild")
            and ($runtime_state.profile_request == "none")
            and ($runtime_state.should_refresh == true)
        ) {
            print "  ✅ Runtime state now names live activation and rebuild intent separately"
            true
        } else {
            print $"  ❌ Unexpected runtime state: ($runtime_state | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Regression: startup rebuilds should activate the fresh runtime-owned profile unless force-reenter is requested.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_startup_transition_prefers_fresh_runtime_profile_after_rebuild [] {
    print "🧪 Testing startup transition prefers the fresh runtime profile after rebuild..."

    try {
        use ../utils/devenv_backend.nu [resolve_runtime_entry_state resolve_startup_transition]

        let runtime_state = (
            resolve_runtime_entry_state
            {should_refresh: true, mode: "refresh"}
        )
        let transition = (resolve_startup_transition $runtime_state)

        if (
            ($transition.execution == "activated_profile")
            and ($transition.profile_source == "fresh_runtime_profile")
            and ($transition.rebuild_before_exec == true)
        ) {
            print "  ✅ Startup transition now records rebuild-then-activate explicitly"
            true
        } else {
            print $"  ❌ Unexpected startup transition: ($transition | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Regression: launch must not keep the live-session fast path when a rebuild is pending.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_launch_transition_blocks_live_session_fast_path_during_refresh [] {
    print "🧪 Testing launch transition blocks the live-session fast path during refresh..."

    try {
        use ../utils/devenv_backend.nu [resolve_launch_transition resolve_runtime_entry_state]

        let runtime_state = (
            resolve_runtime_entry_state
            {should_refresh: true, mode: "refresh"}
            --already-in-env
            --in-yazelix-shell
        )
        let transition = (
            resolve_launch_transition
            $runtime_state
            --current-session-eligible
            --profile-available
        )

        if (
            ($transition.execution == "backend_shell")
            and ($transition.rebuild_before_exec == true)
        ) {
            print "  ✅ Launch transition now forces rebuild ownership instead of reusing the live shell fast path"
            true
        } else {
            print $"  ❌ Unexpected launch transition: ($transition | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Invariant: env only reuses a cached launch profile from an external process when reuse is explicit.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_env_transition_keeps_cached_profile_reuse_explicit [] {
    print "🧪 Testing env transition keeps cached-profile reuse explicit..."

    try {
        use ../utils/devenv_backend.nu [resolve_env_transition resolve_runtime_entry_state]

        let reused_state = (
            resolve_runtime_entry_state
            {should_refresh: false, mode: "reuse"}
        )
        let reused_transition = (
            resolve_env_transition
            $reused_state
            --profile-available
        )

        let ambient_state = (
            resolve_runtime_entry_state
            {should_refresh: false, mode: "reuse"}
            --already-in-env
        )
        let ambient_transition = (
            resolve_env_transition
            $ambient_state
            --profile-available
        )

        if (
            ($reused_transition.execution == "launch_profile")
            and ($ambient_transition.execution == "backend_shell")
            and ($ambient_transition.rebuild_before_exec == false)
        ) {
            print "  ✅ Env transition only reuses cached profiles from the external entry surface"
            true
        } else {
            print $"  ❌ Unexpected env transitions: reused=($reused_transition | to json -r) ambient=($ambient_transition | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Invariant: plain backend-shell commands derive rebuild intent from the shared runtime state instead of raw needs-refresh checks.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_backend_shell_transition_tracks_rebuild_intent [] {
    print "🧪 Testing backend-shell transition tracks rebuild intent explicitly..."

    try {
        use ../utils/devenv_backend.nu [resolve_backend_shell_transition resolve_runtime_entry_state]

        let rebuild_state = (
            resolve_runtime_entry_state
            {should_refresh: true, mode: "refresh"}
        )
        let clean_state = (
            resolve_runtime_entry_state
            {should_refresh: false, mode: "noop"}
        )
        let rebuild_transition = (resolve_backend_shell_transition $rebuild_state)
        let clean_transition = (resolve_backend_shell_transition $clean_state)

        if (
            ($rebuild_transition.rebuild_before_exec == true)
            and ($clean_transition.rebuild_before_exec == false)
            and ($rebuild_transition.execution == "backend_shell")
        ) {
            print "  ✅ Backend-shell commands now inherit rebuild intent from the shared runtime state"
            true
        } else {
            print $"  ❌ Unexpected backend transitions: rebuild=($rebuild_transition | to json -r) clean=($clean_transition | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Defends: persistent-session reuse warns when current-terminal startup ignores the requested directory.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_here_path_warns_when_existing_persistent_session_ignores_it [] {
    print "🧪 Testing current-terminal startup warns when an existing persistent session ignores the requested directory..."

    let fixture = (setup_launch_path_fixture "yazelix_launch_here_path_persistent" true true)

    let result = (try {
        let target_dir = ($fixture.tmp_home | path join "project")
        mkdir $target_dir
        let output = (with-env $fixture.env {
            ^nu $fixture.start_inner $target_dir $fixture.layout_path | complete
        })
        let stdout = ($output.stdout | str trim)
        let zellij_log = if ($fixture.zellij_log | path exists) {
            open --raw $fixture.zellij_log | str trim
        } else {
            ""
        }

        if ($output.exit_code == 0) and ($stdout | str contains "Session 'yazelix' already exists - --path ignored.") and ($stdout | str contains "zellij kill-session yazelix") and ($zellij_log | str contains "attach yazelix") and (not ($zellij_log | str contains "--default-cwd")) {
            print "  ✅ Existing persistent sessions warn clearly and reattach without pretending --path will take effect"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) zellij=($zellij_log)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: launch falls through to the next managed terminal and ignores bare host terminal binaries.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_launch_falls_through_after_immediate_terminal_failure [] {
    print "🧪 Testing managed terminal launch skips bare host binaries and falls through after immediate failure..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_fallback_XXXXXX | str trim)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        let fake_runtime = ($tmpdir | path join "runtime")
        let fake_shells = ($fake_runtime | path join "shells" "posix")
        mkdir $fake_bin
        mkdir $fake_runtime
        mkdir ($fake_runtime | path join "shells")
        mkdir $fake_shells
        ^ln -s (repo_path ".taplo.toml") ($fake_runtime | path join ".taplo.toml")
        "" | save --force --raw ($fake_runtime | path join "yazelix_default.toml")

        [
            "#!/bin/sh"
            "echo wezterm-boom >&2"
            "exit 27"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "yazelix-wezterm")
        ^chmod +x ($fake_bin | path join "yazelix-wezterm")

        [
            "#!/bin/sh"
            "sleep 2"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "yazelix-alacritty")
        ^chmod +x ($fake_bin | path join "yazelix-alacritty")

        let fake_wezterm = ($fake_bin | path join "yazelix-wezterm")
        let fake_alacritty = ($fake_bin | path join "yazelix-alacritty")
        [
            "#!/bin/sh"
            "echo raw-kitty-should-not-run >&2"
            "exit 88"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "kitty")
        ^chmod +x ($fake_bin | path join "kitty")
        let launch_script = (repo_path "nushell" "scripts" "core" "launch_yazelix.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            "let candidates = (resolve_terminal_candidates '' ['wezterm', 'kitty', 'alacritty'] true)"
            "if (($candidates | length) != 2) { error make { msg: ($candidates | to json -r) } }"
            "if (($candidates | get 0.command) != $env.FAKE_WEZTERM) { error make { msg: 'wezterm wrapper not selected first' } }"
            "if (($candidates | get 1.command) != $env.FAKE_ALACRITTY) { error make { msg: 'bare kitty binary should not be treated as a managed candidate' } }"
            "let launched = (launch_terminal_candidates $candidates 'yazelix' $env.PWD false $env.YAZELIX_RUNTIME_DIR false '')"
            "print ($launched.terminal)"
        ] | str join "\n")
        let output = (with-env {
            YAZELIX_RUNTIME_DIR: $fake_runtime
            DEVENV_PROFILE: $tmpdir
            YAZELIX_STATE_DIR: ($tmpdir | path join "state")
            PATH: ([$fake_bin] | append $env.PATH)
            FAKE_WEZTERM: $fake_wezterm
            FAKE_ALACRITTY: $fake_alacritty
        } {
            run_nu_snippet $snippet
        })
        let stdout = ($output.stdout | str trim)
        let stderr = ($output.stderr | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "failed to start; trying Yazelix - Alacritty") and ($stdout | str ends-with "alacritty") and ($stderr == "") {
            print "  ✅ Managed launch ignores bare host binaries and falls through to the next Yazelix wrapper"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($stderr)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: startup preflight requires the generated layout path before deeper launch work.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_startup_requires_generated_layout_path [] {
    print "🧪 Testing startup preflight requires an existing Zellij layout..."

    try {
        let start_script = (repo_path "nushell" "scripts" "core" "start_yazelix.nu")
        let snippet = ([
            $"source \"($start_script)\""
            'try {'
            '    require_generated_layout "/tmp/yazelix_missing_layout.kdl" | ignore'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Missing Yazelix generated Zellij layout") and ($stdout | str contains "yzx refresh") and ($stdout | str contains "Failure class: generated-state problem.") {
            print "  ✅ Startup preflight fails clearly when the generated layout is missing"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Defends: new-window launch preflight requires the runtime launch script before deeper execution.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_launch_requires_runtime_launch_script [] {
    print "🧪 Testing new-window launch preflight requires the runtime launch script..."

    try {
        let launch_script = (repo_path "nushell" "scripts" "yzx" "launch.nu")
        let snippet = ([
            $"source \"($launch_script)\""
            'try {'
            '    require_launch_runtime_script "/tmp/yazelix_missing_launch_yazelix.nu" | ignore'
            '} catch {|err|'
            '    print $err.msg'
            '}'
        ] | str join "\n")
        let output = (run_nu_snippet $snippet)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout | str contains "Missing Yazelix launch script") and ($stdout | str contains "Reinstall/regenerate Yazelix") and ($stdout | str contains "Failure class: generated-state problem.") {
            print "  ✅ New-window launch fails clearly when the runtime launch script is missing"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Defends: yzx cwd fails clearly outside Zellij.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
def test_yzx_cwd_requires_zellij [] {
    print "🧪 Testing yzx cwd outside Zellij..."

    try {
        let yzx_script = (repo_path "nushell" "scripts" "core" "yazelix.nu")
        let output = (^bash -lc $"($CLEAN_ZELLIJ_ENV_PREFIX) nu -c 'use \"($yzx_script)\" *; yzx cwd .'" | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 1) and ($stdout | str contains "only works inside Zellij") {
            print "  ✅ yzx cwd fails clearly outside Zellij"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_workspace_canonical_tests [] {
    [
        (test_yzx_cli_desktop_launch_ignores_hostile_shell_env)
        (test_yzx_desktop_launch_uses_leaf_launch_module_with_clean_env)
        (test_yzx_desktop_launch_falls_back_to_hidden_wait_when_no_visible_bootstrap_terminal_exists)
        (test_desktop_fast_path_rejects_bootstrap_terminal_substitution_for_explicit_terminal)
        (test_desktop_fast_path_uses_direct_host_terminal_during_reload_instead_of_stale_wrapper)
        (test_profile_resolution_policies_separate_runtime_owned_and_current_session_state)
        (test_yzx_edit_resolves_managed_helix_wrapper_from_canonical_launch_env)
        (test_yzx_cli_reveal_uses_lightweight_reveal_helper)
        (test_yzx_cli_menu_uses_lightweight_menu_module)
        (test_yzx_menu_palette_eligibility_is_broad_but_explicit)
        (test_yzx_cli_enter_uses_lightweight_enter_module)
        (test_launch_falls_through_after_immediate_terminal_failure)
        (test_launch_here_path_uses_requested_directory_for_nonpersistent_sessions)
        (test_yzx_enter_forwards_refresh_intent_to_startup_entrypoint)
        (test_startup_refresh_activates_built_profile_without_second_shell_entry)
        (test_runtime_entry_state_models_live_session_refresh_intent)
        (test_startup_transition_prefers_fresh_runtime_profile_after_rebuild)
        (test_launch_transition_blocks_live_session_fast_path_during_refresh)
        (test_env_transition_keeps_cached_profile_reuse_explicit)
        (test_backend_shell_transition_tracks_rebuild_intent)
        (test_yzx_launch_rejects_removed_here_flag)
        (test_launch_here_path_warns_when_existing_persistent_session_ignores_it)
        (test_startup_rejects_missing_working_dir)
        (test_launch_rejects_file_working_dir)
        (test_startup_requires_generated_layout_path)
        (test_launch_requires_runtime_launch_script)
        (test_yzx_cwd_requires_zellij)
    ]
}
