#!/usr/bin/env nu
# Test runner for maintainer-only yzx checks
# Test lane: maintainer

use ../utils/common.nu [get_yazelix_state_dir require_yazelix_repo_root]
use ../utils/devenv_cli.nu resolve_preferred_devenv_path
use ./validate_default_test_budget.nu [profile_suite_runner]

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: issue/bead reconciliation planning catches create, reopen, close, and duplicate cases.
def test_issue_bead_reconciliation_plan [] {
    print "🧪 Testing issue/bead reconciliation plans create, reopen, close, and reject duplicates..."

    try {
        let command = '
            source nushell/scripts/utils/issue_bead_contract.nu
            let github_issues = [
                {number: 500, state: "OPEN", title: "Missing bead", url: "https://github.com/luccahuguet/yazelix/issues/500", createdAt: "2026-03-22T12:30:00Z", body: ""}
                {number: 501, state: "OPEN", title: "Closed bead should reopen", url: "https://github.com/luccahuguet/yazelix/issues/501", createdAt: "2026-03-22T12:31:00Z", body: ""}
                {number: 502, state: "CLOSED", title: "Open bead should close", url: "https://github.com/luccahuguet/yazelix/issues/502", createdAt: "2026-03-22T12:32:00Z", body: ""}
                {number: 503, state: "OPEN", title: "Already aligned", url: "https://github.com/luccahuguet/yazelix/issues/503", createdAt: "2026-03-22T12:33:00Z", body: ""}
                {number: 504, state: "OPEN", title: "Duplicate bead", url: "https://github.com/luccahuguet/yazelix/issues/504", createdAt: "2026-03-22T12:34:00Z", body: ""}
                {number: 399, state: "OPEN", title: "Grandfathered backlog issue", url: "https://github.com/luccahuguet/yazelix/issues/399", createdAt: "2026-03-21T23:59:59Z", body: ""}
            ]
            let beads = [
                {id: "yazelix-reopen", status: "closed", external_ref: "https://github.com/luccahuguet/yazelix/issues/501"}
                {id: "yazelix-close", status: "open", external_ref: "https://github.com/luccahuguet/yazelix/issues/502"}
                {id: "yazelix-noop", status: "open", external_ref: "https://github.com/luccahuguet/yazelix/issues/503"}
                {id: "yazelix-dup-a", status: "open", external_ref: "https://github.com/luccahuguet/yazelix/issues/504"}
                {id: "yazelix-dup-b", status: "closed", external_ref: "https://github.com/luccahuguet/yazelix/issues/504"}
                {id: "yazelix-old", status: "open", external_ref: "https://github.com/luccahuguet/yazelix/issues/399"}
            ]
            let plan = (plan_issue_bead_reconciliation $github_issues $beads)
            {
                action_kinds: ($plan.actions | each { |action| $action.kind } | sort),
                errors: ($plan.errors | sort)
            } | to json -r
        '
        let output = (^nu -c $command | complete)
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)
        let expected_actions = ["close" "create" "noop" "reopen"]
        let expected_errors = ["Duplicate beads for GitHub issue #504: yazelix-dup-a, yazelix-dup-b"]

        if ($output.exit_code == 0) and ($resolved.action_kinds == $expected_actions) and ($resolved.errors == $expected_errors) {
            print "  ✅ Reconciliation planning matches the contract surface"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) resolved=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: issue/bead comment planning keeps one canonical Beads mapping comment.
def test_issue_bead_comment_plan [] {
    print "🧪 Testing issue/bead comment planning creates, repairs legacy placeholders, updates stale comments, and accepts canonical comments..."

    try {
        let command = '
            source nushell/scripts/utils/issue_bead_contract.nu
            let issue = {number: 600, state: "OPEN", title: "Comment contract", url: "https://github.com/luccahuguet/yazelix/issues/600", createdAt: "2026-03-22T12:40:00Z", body: ""}
            let bead = {id: "yazelix-comment", status: "open", external_ref: $issue.url}
            let missing = (plan_issue_bead_comment_sync $issue $bead [])
            let placeholder = (plan_issue_bead_comment_sync $issue $bead [{id: "IC_placeholder", body: "$action.body"}])
            let stale = (plan_issue_bead_comment_sync $issue $bead [{id: "IC_stale", body: "Tracked in Beads as `yazelix-old`."}])
            let current = (plan_issue_bead_comment_sync $issue $bead [{id: "IC_current", body: "Automated: Tracked in Beads as `yazelix-comment`."}])
            {
                missing: $missing.kind
                placeholder: $placeholder.kind
                stale: $stale.kind
                current: $current.kind
                expected_body: (canonical_issue_bead_comment_body $bead.id)
            } | to json -r
        '
        let output = (^nu -c $command | complete)
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)

        if (
            ($output.exit_code == 0)
            and ($resolved.missing == "create")
            and ($resolved.placeholder == "update")
            and ($resolved.stale == "update")
            and ($resolved.current == "noop")
            and ($resolved.expected_body == "Automated: Tracked in Beads as `yazelix-comment`.")
        ) {
            print "  ✅ Comment planning enforces one canonical Beads comment body"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) resolved=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: repo-local devenv shells clear inherited installed-runtime aliases but still expose the checkout runtime root.
def test_source_devenv_shell_clears_inherited_runtime_aliases [] {
    print "🧪 Testing repo-local devenv shells sanitize inherited runtime aliases to the checkout root..."

    let repo_root = ($env.PWD | path expand)
    let fake_runtime = (get_yazelix_state_dir | path join "runtime" "current" | path expand)
    let devenv_bin = (resolve_preferred_devenv_path)

    try {
        let output = (with-env {
            YAZELIX_RUNTIME_DIR: $fake_runtime
            YAZELIX_DIR: "/nix/store/fake-yazelix-runtime"
            YAZELIX_SHELLHOOK_SKIP_WELCOME: "true"
            YAZELIX_ENV_ONLY: "true"
        } {
            ^$devenv_bin --quiet shell -- bash -lc 'printf "%s|%s|%s|%s\n" "$(printenv YAZELIX_RUNTIME_DIR 2>/dev/null || printf unset)" "$(printenv YAZELIX_DIR 2>/dev/null || printf unset)" "$DEVENV_ROOT" "$EDITOR"' | complete
        })
        let summary = ($output.stdout | lines | last | default "")
        let expected_editor = ($repo_root | path join "shells" "posix" "yazelix_hx.sh")

        if ($output.exit_code == 0) and ($summary == $"($repo_root)|unset|($repo_root)|($expected_editor)") {
            print "  ✅ Repo-local devenv shell now replaces inherited runtime aliases with DEVENV_ROOT and exports an absolute managed Helix wrapper"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) summary=($summary) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: managed Ghostty wrappers must find sibling nixGL even when desktop launch clears DEVENV_PROFILE.
def test_managed_wrapper_prefers_sibling_nixgl_without_ambient_profile [] {
    print "🧪 Testing managed Ghostty wrappers resolve sibling nixGL without ambient DEVENV_PROFILE..."

    let repo_root = ($env.PWD | path expand)
    let devenv_bin = (resolve_preferred_devenv_path)

    try {
        let output = (with-env {
            YAZELIX_SHELLHOOK_SKIP_WELCOME: "true"
            YAZELIX_ENV_ONLY: "true"
        } {
            ^$devenv_bin --quiet shell -- bash -lc 'expected="$DEVENV_PROFILE/bin/nixGL"; wrapper="$DEVENV_PROFILE/bin/yazelix-ghostty"; printf "__EXPECTED__%s\n" "$expected"; env -u DEVENV_PROFILE PATH=/usr/bin:/bin YAZELIX_RUNTIME_DIR="$DEVENV_ROOT" bash -x "$wrapper" --version >/dev/null' | complete
        })
        let expected_nixgl = (
            $output.stdout
            | lines
            | where {|line| $line | str starts-with "__EXPECTED__" }
            | get -o 0
            | default ""
            | str replace "__EXPECTED__" ""
            | str trim
        )
        let trace = ($output.stderr | default "")
        let expected_self_line = ('+ self_nixgl=' + $expected_nixgl)
        let expected_exec_prefix = ('+ exec ' + $expected_nixgl + ' ')

        if (
            ($output.exit_code == 0)
            and ($expected_nixgl | is-not-empty)
            and ($trace | str contains $expected_self_line)
            and ($trace | str contains $expected_exec_prefix)
        ) {
            print "  ✅ Managed Ghostty wrapper now finds sibling nixGL without relying on ambient DEVENV_PROFILE"
            true
        } else {
            let trace_tail = ($trace | lines | last 20 | str join "\n")
            print $"  ❌ Unexpected result: exit=($output.exit_code) expected_nixgl=($expected_nixgl) stderr=($trace_tail)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Regression: maintainer commands resolve a writable repo root even when the stable CLI carries an installed runtime env.
def test_maintainer_repo_root_prefers_checkout_over_installed_runtime_env [] {
    print "🧪 Testing maintainer repo-root resolution prefers the checkout over installed runtime env..."

    let repo_root = ($env.PWD | path expand)
    let fake_runtime = (get_yazelix_state_dir | path join "runtime" "current" | path expand)

    try {
        let resolved = (with-env {
            DEVENV_ROOT: $repo_root
            YAZELIX_RUNTIME_DIR: $fake_runtime
            YAZELIX_DIR: "/nix/store/fake-yazelix-runtime"
        } {
            require_yazelix_repo_root
        })

        if $resolved == $repo_root {
            print "  ✅ Maintainer repo-root resolution now ignores the installed runtime env in repo shells"
            true
        } else {
            print $"  ❌ Unexpected result: resolved=($resolved) expected=($repo_root)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Regression: runtime-project lookup must stay read-only while materialization remains explicit.
def test_runtime_project_lookup_stays_read_only_until_materialized [] {
    print "🧪 Testing runtime-project lookup stays read-only until materialized..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_runtime_project_split_XXXXXX | str trim)
    let state_dir = ($tmp_root | path join "state")
    let runtime_dir = ($tmp_root | path join "runtime")
    let project_dir = ($state_dir | path join "runtime" "project")
    let stale_runtime_dir = ($tmp_root | path join "stale_runtime")
    let common_script = ($repo_root | path join "nushell" "scripts" "utils" "common.nu")

    mkdir $state_dir
    mkdir $runtime_dir
    mkdir $project_dir
    mkdir $stale_runtime_dir

    for entry in [".taplo.toml", "assets", "config_metadata", "configs", "nushell", "rust_plugins", "shells", "CHANGELOG.md", "devenv.lock", "devenv.nix", "devenv.yaml", "yazelix_default.toml", "yazelix_packs_default.toml"] {
        ^ln -s ($repo_root | path join $entry) ($runtime_dir | path join $entry)
    }
    "stale" | save --force ($stale_runtime_dir | path join "devenv.nix")
    ^ln -s ($stale_runtime_dir | path join "devenv.nix") ($project_dir | path join "devenv.nix")
    "stale-docs" | save --force ($project_dir | path join "docs")

    let result = (try {
        let snippet = (
            [
                $"use \"($common_script)\" [get_existing_yazelix_runtime_project_dir materialize_yazelix_runtime_project_dir]"
                "print ((get_existing_yazelix_runtime_project_dir) | default '<missing>')"
                "print (materialize_yazelix_runtime_project_dir)"
                "print ((get_existing_yazelix_runtime_project_dir) | default '<missing>')"
            ] | str join "\n"
        )
        let output = (with-env {
            YAZELIX_RUNTIME_DIR: $runtime_dir
            YAZELIX_STATE_DIR: $state_dir
        } {
            do { ^nu -c $snippet } | complete
        })
        let lines = ($output.stdout | lines)
        let devenv_nix_target = ($project_dir | path join "devenv.nix")
        let docs_target = ($project_dir | path join "docs")
        let resolved_devenv_nix_target = (if ($devenv_nix_target | path exists) { ^readlink -f $devenv_nix_target | str trim } else { "" })
        let resolved_runtime_devenv_nix = (^readlink -f ($runtime_dir | path join "devenv.nix") | str trim)

        if (
            ($output.exit_code == 0)
            and (($lines | get -o 0 | default "") == "<missing>")
            and (($lines | get -o 1 | default "") == $project_dir)
            and (($lines | get -o 2 | default "") == $project_dir)
            and ($devenv_nix_target | path exists)
            and (($devenv_nix_target | path type) == "symlink")
            and ($resolved_devenv_nix_target == $resolved_runtime_devenv_nix)
            and not ($docs_target | path exists)
        ) {
            print "  ✅ Stale runtime-project state is ignored until explicit materialization replaces it"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) lines=(($lines | to json -r)) target_exists=(($devenv_nix_target | path exists)) target_type=(if ($devenv_nix_target | path exists) { $devenv_nix_target | path type } else { '<missing>' }) target_resolved=($resolved_devenv_nix_target) docs_exists=(($docs_target | path exists)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: default-suite budget profiling must not wait for leaked background children from the runner.
def test_default_budget_profiler_does_not_wait_on_background_children [] {
    print "🧪 Testing default budget profiling returns promptly even when the runner leaves a background child alive..."

    try {
        let started = (date now)
        let profiled = (profile_suite_runner {
            ^bash -lc 'sleep 3 &'
            [true]
        })
        let wall_seconds = (((date now) - $started) / 1sec | into float)

        if ($wall_seconds < 1.5) and ($profiled.elapsed_seconds < 1.5) {
            print "  ✅ Budget profiling now measures the runner directly instead of waiting on a leaked background child subprocess"
            true
        } else {
            print $"  ❌ Budget profiling still waited too long: wall=($wall_seconds)s profiled=($profiled.elapsed_seconds)s"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=2 cost=0 uniqueness=2 total=8/10
# Regression: a successful build-shell keeps runtime-project profile artifacts aligned with the built profile.
def test_build_shell_output_records_runtime_owned_profile_without_runtime_project_artifacts [] {
    print "🧪 Testing build-shell output records the runtime-owned profile without requiring runtime-project .devenv artifacts..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_runtime_project_shell_align_XXXXXX | str trim)
    let temp_home = ($tmp_root | path join "home")
    let config_dir = ($temp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let state_dir = ($tmp_root | path join "state")
    let bootstrap_module = ($repo_root | path join "nushell" "scripts" "utils" "environment_bootstrap.nu")
    let launch_state_module = ($repo_root | path join "nushell" "scripts" "utils" "launch_state.nu")

    mkdir $user_config_dir
    mkdir $state_dir
    cp ($repo_root | path join "yazelix_default.toml") ($user_config_dir | path join "yazelix.toml")
    cp ($repo_root | path join "yazelix_packs_default.toml") ($user_config_dir | path join "yazelix_packs.toml")

    let result = (try {
        let snippet = (
            [
                $"use \"($bootstrap_module)\" [get_devenv_base_command]"
                $"use \"($launch_state_module)\" [record_launch_profile_state resolve_profile_from_build_shell_output resolve_runtime_owned_profile]"
                "let base = (get_devenv_base_command --quiet --skip-shellhook-welcome)"
                "let full = ($base | append [\"build\" \"shell\"])"
                "let cmd_bin = ($full | first)"
                "let cmd_args = ($full | skip 1)"
                "let result = (^$cmd_bin ...$cmd_args | complete)"
                "if $result.exit_code != 0 {"
                "    print --raw ($result.stdout | default \"\")"
                "    print --stderr --raw ($result.stderr | default \"\")"
                "    exit 1"
                "}"
                "let built_profile = (resolve_profile_from_build_shell_output $result.stdout)"
                "let project_root = ($env.YAZELIX_STATE_DIR | path join \"runtime\" \"project\")"
                "let profile_link = ($project_root | path join \".devenv\" \"profile\")"
                "let shell_link = ($project_root | path join \".devenv\" \"gc\" \"shell\")"
                "if ($profile_link | path exists) { rm --force $profile_link }"
                "if ($shell_link | path exists) { rm --force $shell_link }"
                "record_launch_profile_state {combined_hash: \"probe-hash\"} $built_profile"
                "let runtime_owned = (resolve_runtime_owned_profile)"
                "let recorded_state = (open (($env.YAZELIX_STATE_DIR | path join \"state\" \"launch_state.json\")))"
                "{"
                "    built_profile: $built_profile"
                "    runtime_owned: $runtime_owned"
                "    recorded_profile: ($recorded_state.profile_path | into string)"
                "    profile_link_exists: ($profile_link | path exists)"
                "    shell_link_exists: ($shell_link | path exists)"
                "} | to json -r"
            ] | str join "\n"
        )
        let output = (with-env {
            HOME: $temp_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $snippet | complete
        })
        let stdout = ($output.stdout | str trim)
        let resolved = if $output.exit_code == 0 {
            $stdout | lines | last | from json
        } else {
            null
        }

        if (
            ($output.exit_code == 0)
            and ($resolved != null)
            and (($resolved.built_profile? | default "") | is-not-empty)
            and ($resolved.built_profile == $resolved.runtime_owned)
            and ($resolved.built_profile == $resolved.recorded_profile)
            and (not $resolved.profile_link_exists)
            and (not $resolved.shell_link_exists)
        ) {
            print "  ✅ Build-shell output and recorded launch state are enough for runtime-owned profile resolution"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=0 uniqueness=2 total=8/10
# Regression: vendored Yazi plugin refresh applies declared overlay patches and refuses dirty managed files.
def test_vendored_yazi_plugin_refresh_applies_patch_and_refuses_dirty_targets [] {
    print "🧪 Testing vendored Yazi plugin refresh applies overlay patches and refuses dirty targets..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_vendored_yazi_refresh_XXXXXX | str trim)
    let upstream_dir = ($tmp_root | path join "upstream")
    let target_repo = ($tmp_root | path join "repo")
    let target_plugin_dir = ($target_repo | path join "configs" "yazi" "plugins" "demo.yazi")
    let patch_dir = ($target_repo | path join "config_metadata" "vendored_yazi_plugin_patches")
    let manifest_path = ($target_repo | path join "config_metadata" "vendored_yazi_plugins.toml")
    let patch_path = ($patch_dir | path join "demo.patch")
    let target_main = ($target_plugin_dir | path join "main.lua")
    let update_script = ($repo_root | path join "nushell" "scripts" "dev" "update_yazi_plugins.nu")

    mkdir $upstream_dir
    mkdir $target_plugin_dir
    mkdir $patch_dir

    "return \"base\"\n" | save --force --raw ($upstream_dir | path join "main.lua")
    do {
        cd $upstream_dir
        ^git init -q
        ^git config user.email test@example.com
        ^git config user.name "Yazelix Test"
        ^git add main.lua
        ^git commit -q -m "initial upstream"
    }
    let pinned_rev = (^git -C $upstream_dir rev-parse HEAD | str trim)

    [
        '[metadata]'
        'description = "test vendored yazi plugin manifest"'
        ''
        '[[plugins]]'
        'name = "demo.yazi"'
        'ownership = "upstream"'
        $"upstream_repo = \"($upstream_dir)\""
        'tracking_ref = "main"'
        $"pinned_rev = \"($pinned_rev)\""
        'source_subdir = "."'
        'target_dir = "configs/yazi/plugins/demo.yazi"'
        'managed_files = ["main.lua"]'
        'patch_file = "config_metadata/vendored_yazi_plugin_patches/demo.patch"'
    ] | str join "\n" | save --force --raw $manifest_path

    [
        'diff --git a/main.lua b/main.lua'
        '--- a/main.lua'
        '+++ b/main.lua'
        '@@ -1 +1 @@'
        '-return "base"'
        '+return "patched"'
        ''
    ] | str join "\n" | save --force --raw $patch_path

    "return \"stale\"\n" | save --force --raw $target_main

    do {
        cd $target_repo
        ^git init -q
        ^git config user.email test@example.com
        ^git config user.name "Yazelix Test"
        ^git add config_metadata configs
        ^git commit -q -m "seed vendored target"
    }

    let result = (try {
        let first_run = (^nu $update_script --repo-root $target_repo --manifest $manifest_path | complete)
        let first_content = (open --raw $target_main | str trim)
        let second_run = (^nu $update_script --repo-root $target_repo --manifest $manifest_path | complete)
        let second_text = ((($second_run.stdout | default "") + "\n" + ($second_run.stderr | default "")) | str trim)

        if (
            ($first_run.exit_code == 0)
            and ($first_content == 'return "patched"')
            and ($second_run.exit_code != 0)
            and ($second_text | str contains "Local changes detected in managed vendored plugin files")
        ) {
            print "  ✅ Vendored Yazi plugin refresh applies overlay patches and protects dirty managed files"
            true
        } else {
            print $"  ❌ Unexpected result: first_exit=($first_run.exit_code) first_content=($first_content) second_exit=($second_run.exit_code) second_output=($second_text)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

# Strength: defect=1 behavior=2 resilience=1 cost=1 uniqueness=1 total=6/10
# Invariant: generated Nushell initializers restore current PATH entries ahead of the saved PATH.
def test_nushell_initializer_restores_current_path_first [] {
    print "🧪 Testing the generated Nushell initializer preserves current PATH precedence..."

    try {
        let temp_home = (^mktemp -d | str trim)
        let yazelix_dir = $env.PWD
        let output = (with-env { HOME: $temp_home YAZELIX_QUIET_MODE: "true" } {
            ^nu nushell/scripts/setup/initializers.nu $yazelix_dir "nu" | complete
        })
        let aggregate = ($temp_home | path join ".local" "share" "yazelix" "initializers" "nushell" "yazelix_init.nu")
        let content = if ($aggregate | path exists) { open --raw $aggregate } else { "" }
        rm -rf $temp_home

        if ($output.exit_code == 0) and ($content | str contains '$env.PATH = ($current_path | append $initial_path | uniq)') {
            print "  ✅ Generated initializer keeps current PATH entries ahead of the saved PATH"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) aggregate=($aggregate)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=2 total=8/10
# Defends: startup profiling writes a structured report with stable run and step records that can be summarized later.
def test_startup_profile_report_schema_is_structured_and_summarizable [] {
    print "🧪 Testing startup profiling writes a structured and summarizable report..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_startup_profile_schema_XXXXXX | str trim)
    let state_dir = ($tmp_root | path join "state")
    let profile_module = ($repo_root | path join "nushell" "scripts" "utils" "startup_profile.nu")

    mkdir $state_dir

    let result = (try {
        let snippet = (
            [
                $"use \"($profile_module)\" [create_startup_profile_run profile_startup_step load_startup_profile_report]"
                "let run = (create_startup_profile_run \"unit_test\" {mode: \"maintainer\"})"
                "with-env $run.env {"
                "    profile_startup_step \"bootstrap\" \"prepare.parse_config\" {"
                "        sleep 5ms"
                "        42"
                "    } {phase: \"unit_test\"} | ignore"
                "}"
                "let summary = (load_startup_profile_report $run.report_path)"
                "{"
                "    report_path: $run.report_path"
                "    run: $summary.run"
                "    steps: $summary.steps"
                "    total_duration_ms: $summary.total_duration_ms"
                "} | to json -r"
            ] | str join "\n"
        )
        let output = (with-env {
            YAZELIX_STATE_DIR: $state_dir
        } {
            do { ^nu -c $snippet } | complete
        })
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | lines | last | from json)
        let first_step = ($resolved.steps | get 0)

        if (
            ($output.exit_code == 0)
            and ($resolved.report_path | path exists)
            and ($resolved.run.type == "run")
            and ($resolved.run.schema_version == 1)
            and ($resolved.run.scenario == "unit_test")
            and (($resolved.steps | length) == 1)
            and ($first_step.type == "step")
            and ($first_step.component == "bootstrap")
            and ($first_step.step == "prepare.parse_config")
            and (($first_step.metadata.phase? | default "") == "unit_test")
            and (($first_step.duration_ms | into float) > 0.0)
            and (($resolved.total_duration_ms | into float) >= ($first_step.duration_ms | into float))
        ) {
            print "  ✅ Startup profiling now writes a structured run header, stable step records, and a computable total wall time"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=0 uniqueness=2 total=8/10
# Defends: the profiling harness runs the real startup path and records owned startup boundaries into a structured report.
def test_startup_profile_harness_records_real_startup_boundaries [] {
    print "🧪 Testing startup profiling harness records real startup boundaries..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_startup_profile_harness_XXXXXX | str trim)
    let temp_home = ($tmp_root | path join "home")
    let state_dir = ($tmp_root | path join "state")
    let config_dir = ($temp_home | path join ".config" "yazelix")
    let generated_layout_dir = ($temp_home | path join ".local" "share" "yazelix" "configs" "zellij" "layouts")
    let profile_module = ($repo_root | path join "nushell" "scripts" "utils" "profile.nu")

    mkdir $temp_home
    mkdir $state_dir
    mkdir ($config_dir | path dirname)
    mkdir $generated_layout_dir
    "layout { pane }" | save --force ($generated_layout_dir | path join "yzx_side.kdl")

    let result = (try {
        let snippet = (
            [
                $"use \"($profile_module)\" [run_profiled_startup_harness]"
                "let summary = (run_profiled_startup_harness \"maintainer_e2e\" [\"--skip-refresh\"])"
                "{"
                "    report_path: $summary.report_path"
                "    scenario: $summary.run.scenario"
                "    steps: ($summary.steps | each {|step| {component: $step.component step: $step.step}})"
                "} | to json -r"
            ] | str join "\n"
        )
        let output = (with-env {
            HOME: $temp_home
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_CONFIG_DIR: $config_dir
            DEVENV_ROOT: $repo_root
            IN_YAZELIX_SHELL: ""
            YAZELIX_TERMINAL: ""
            DEVENV_PROFILE: ""
            IN_NIX_SHELL: ""
        } {
            do { ^nu -c $snippet } | complete
        })
        let stdout = ($output.stdout | str trim)
        let resolved = if $output.exit_code == 0 {
            $stdout | lines | last | from json
        } else {
            null
        }
        let steps = if $resolved == null {
            []
        } else {
            ($resolved.steps | default [])
        }

        if (
            ($output.exit_code == 0)
            and ($resolved != null)
            and ($resolved.report_path | path exists)
            and ($resolved.scenario == "maintainer_e2e")
            and ($steps | any {|step| $step.component == "startup" and $step.step == "entrypoint.config_migration_preflight" })
            and ($steps | any {|step| $step.component == "bootstrap" and $step.step == "prepare.parse_config" })
            and ($steps | any {|step| $step.component == "inner" and $step.step == "zellij_handoff_ready" })
        ) {
            print "  ✅ Profiling harness now records the real startup ownership boundaries"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: maintainer update requires an explicit activation target for real updates instead of silently falling through to installer behavior.
def test_dev_update_requires_explicit_activation_for_real_updates [] {
    print "🧪 Testing yzx dev update requires an explicit activation target unless canary-only is requested..."

    let repo_root = ($env.PWD | path expand)
    let dev_module = ($repo_root | path join "nushell" "scripts" "yzx" "dev.nu")

    try {
        let snippet = (
            [
                $"source \"($dev_module)\""
                "let canary_only = (try {"
                "    resolve_requested_update_activation_mode \"\" true | ignore"
                "    \"canary-ok\""
                "} catch {|err|"
                "    $err.msg"
                "})"
                "try {"
                "    resolve_requested_update_activation_mode \"\" false | ignore"
                "    print \"unexpected-success\""
                "} catch {|err|"
                "    {"
                "        canary_only: $canary_only"
                "        missing_error: $err.msg"
                "    } | to json -r | print"
                "}"
            ] | str join "\n"
        )
        let output = (^nu -c $snippet | complete)
        let stdout = ($output.stdout | str trim)
        let resolved = ($stdout | from json)
        let expected = "yzx dev update now requires --activate installer|home_manager|none unless you are using --canary-only."

        if (
            ($output.exit_code == 0)
            and ($resolved.canary_only == "canary-ok")
            and ($resolved.missing_error == $expected)
        ) {
            print "  ✅ Real maintainer updates now fail fast until an activation target is chosen explicitly, while canary-only stays exempt"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Defends: Home Manager activation refreshes the configured flake input and switches the requested flake ref instead of falling back to the installer path.
def test_dev_update_home_manager_activation_refreshes_input_and_switches_requested_ref [] {
    print "🧪 Testing yzx dev update Home Manager activation refreshes the input lock and switches the requested ref..."

    let repo_root = ($env.PWD | path expand)
    let dev_module = ($repo_root | path join "nushell" "scripts" "yzx" "dev.nu")
    let tmp_root = (^mktemp -d /tmp/yazelix_dev_update_home_manager_activation_XXXXXX | str trim)
    let bin_dir = ($tmp_root | path join "bin")
    let flake_dir = ($tmp_root | path join "home-manager")
    let log_path = ($tmp_root | path join "activation.log")
    let nix_script = ($bin_dir | path join "nix")
    let home_manager_script = ($bin_dir | path join "home-manager")
    let current_path = if (($env.PATH | describe) | str contains "list") {
        $env.PATH | str join (char esep)
    } else {
        $env.PATH | into string
    }

    mkdir $bin_dir
    mkdir $flake_dir
    "{ }\n" | save --force --raw ($flake_dir | path join "flake.nix")
    [
        "#!/usr/bin/env bash"
        "printf 'nix:%s\\n' \"$*\" >> \"$YZX_TEST_LOG\""
        "exit 0"
    ] | str join "\n" | save --force --raw $nix_script
    [
        "#!/usr/bin/env bash"
        "printf 'home-manager:%s\\n' \"$*\" >> \"$YZX_TEST_LOG\""
        "exit 0"
    ] | str join "\n" | save --force --raw $home_manager_script
    ^chmod +x $nix_script $home_manager_script

    let result = (try {
        let snippet = (
            [
                $"source \"($dev_module)\""
                "let result = (activate_updated_home_manager_runtime $env.YZX_TEST_FLAKE_DIR \"yazelix-hm\" \"lucca@loqness\" true)"
                "$result | to json -r"
            ] | str join "\n"
        )
        let output = (with-env {
            PATH: $"($bin_dir)(char esep)($current_path)"
            YZX_TEST_LOG: $log_path
            YZX_TEST_FLAKE_DIR: $flake_dir
        } {
            ^nu -c $snippet | complete
        })
        let stdout = ($output.stdout | str trim)
        let resolved = if $output.exit_code == 0 {
            $stdout | lines | last | from json
        } else {
            null
        }
        let log_lines = if ($log_path | path exists) {
            open --raw $log_path | lines
        } else {
            []
        }
        let expected_switch_ref = $"($flake_dir)#lucca@loqness"
        let expected_log = [
            $"nix:flake lock --update-input yazelix-hm ($flake_dir)"
            $"home-manager:switch --flake ($expected_switch_ref)"
        ]

        if (
            ($output.exit_code == 0)
            and ($resolved != null)
            and ($resolved.flake_dir == $flake_dir)
            and ($resolved.input_name == "yazelix-hm")
            and ($resolved.switch_ref == $expected_switch_ref)
            and ($log_lines == $expected_log)
        ) {
            print "  ✅ Home Manager activation now refreshes the configured input and switches the exact requested ref"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) log=(($log_lines | to json -r)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_root
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: maintainer update activation rejects unknown mode names instead of accepting ambiguous shorthand.
def test_dev_update_activation_mode_rejects_unknown_values [] {
    print "🧪 Testing yzx dev update activation parsing rejects unknown mode names..."

    let repo_root = ($env.PWD | path expand)
    let dev_module = ($repo_root | path join "nushell" "scripts" "yzx" "dev.nu")

    try {
        let snippet = (
            [
                $"source \"($dev_module)\""
                "try {"
                "    resolve_requested_update_activation_mode \"hm\" false | ignore"
                "    print \"unexpected-success\""
                "} catch {|err|"
                "    print $err.msg"
                "}"
            ] | str join "\n"
        )
        let output = (^nu -c $snippet | complete)
        let stdout = ($output.stdout | str trim)
        let expected = "Unknown activation mode: hm. Expected one of: installer, home_manager, none"

        if ($output.exit_code == 0) and ($stdout == $expected) {
            print "  ✅ Unknown maintainer activation names now fail fast with the supported mode list"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def main [] {
    print "=== Testing yzx Maintainer Commands ==="
    print ""

    let results = [
        (test_issue_bead_reconciliation_plan)
        (test_issue_bead_comment_plan)
        (test_dev_update_requires_explicit_activation_for_real_updates)
        (test_dev_update_activation_mode_rejects_unknown_values)
        (test_dev_update_home_manager_activation_refreshes_input_and_switches_requested_ref)
        (test_source_devenv_shell_clears_inherited_runtime_aliases)
        (test_managed_wrapper_prefers_sibling_nixgl_without_ambient_profile)
        (test_maintainer_repo_root_prefers_checkout_over_installed_runtime_env)
        (test_runtime_project_lookup_stays_read_only_until_materialized)
        (test_default_budget_profiler_does_not_wait_on_background_children)
        (test_build_shell_output_records_runtime_owned_profile_without_runtime_project_artifacts)
        (test_vendored_yazi_plugin_refresh_applies_patch_and_refuses_dirty_targets)
        (test_nushell_initializer_restores_current_path_first)
        (test_startup_profile_report_schema_is_structured_and_summarizable)
        (test_startup_profile_harness_records_real_startup_boundaries)
    ]

    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    print ""
    if $passed == $total {
        print $"✅ All yzx maintainer tests passed \(($passed)/($total)\)"
    } else {
        print $"❌ Some maintainer tests failed \(($passed)/($total)\)"
        error make { msg: "yzx maintainer tests failed" }
    }
}
