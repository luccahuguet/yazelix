#!/usr/bin/env nu
# Test runner for maintainer-only yzx checks
# Test lane: maintainer

use ../utils/common.nu [get_yazelix_state_dir]
use ../maintainer/repo_checkout.nu [require_yazelix_repo_root]

def profile_suite_runner [runner: closure] {
    let started = (date now)
    let result = (do $runner)
    let elapsed_seconds = (((date now) - $started) / 1sec | into float)

    {
        result: $result
        elapsed_seconds: $elapsed_seconds
    }
}

def setup_dev_bump_fixture [] {
    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_dev_bump_XXXXXX | str trim)
    let fixture_root = ($tmp_root | path join "repo")
    let utils_dir = ($fixture_root | path join "nushell" "scripts" "utils")
    let maintainer_dir = ($fixture_root | path join "nushell" "scripts" "maintainer")
    let docs_dir = ($fixture_root | path join "docs")

    mkdir $fixture_root
    mkdir ($fixture_root | path join "nushell")
    mkdir ($fixture_root | path join "nushell" "scripts")
    mkdir $utils_dir
    mkdir $maintainer_dir
    mkdir $docs_dir

    ^cp ($repo_root | path join "README.md") ($fixture_root | path join "README.md")
    ^cp ($repo_root | path join "CHANGELOG.md") ($fixture_root | path join "CHANGELOG.md")
    ^cp ($repo_root | path join "yazelix_default.toml") ($fixture_root | path join "yazelix_default.toml")
    ^cp ($repo_root | path join "docs" "upgrade_notes.toml") ($docs_dir | path join "upgrade_notes.toml")
    ^cp ($repo_root | path join "nushell" "scripts" "utils" "common.nu") ($utils_dir | path join "common.nu")
    ^cp ($repo_root | path join "nushell" "scripts" "utils" "constants.nu") ($utils_dir | path join "constants.nu")
    ^cp ($repo_root | path join "nushell" "scripts" "utils" "upgrade_notes.nu") ($utils_dir | path join "upgrade_notes.nu")
    ^cp ($repo_root | path join "nushell" "scripts" "maintainer" "readme_surface.nu") ($maintainer_dir | path join "readme_surface.nu")
    ^cp ($repo_root | path join "nushell" "scripts" "maintainer" "version_bump.nu") ($maintainer_dir | path join "version_bump.nu")

    ^git -C $fixture_root init --quiet
    ^git -C $fixture_root config user.email "codex@example.com"
    ^git -C $fixture_root config user.name "Codex"
    ^git -C $fixture_root add -A
    ^git -C $fixture_root commit --quiet -m "Fixture baseline"

    {
        repo_root: $fixture_root
        helper_module: ($maintainer_dir | path join "version_bump.nu")
        constants_path: ($utils_dir | path join "constants.nu")
        notes_path: ($docs_dir | path join "upgrade_notes.toml")
        changelog_path: ($fixture_root | path join "CHANGELOG.md")
        readme_path: ($fixture_root | path join "README.md")
    }
}

def commit_dev_bump_fixture_change [fixture: record, message: string] {
    ^git -C $fixture.repo_root add -A
    ^git -C $fixture.repo_root commit --quiet -m $message
}

def setup_flake_interface_fixture [] {
    let repo_root = (require_yazelix_repo_root)
    let tmp_root = (^mktemp -d /tmp/yazelix_flake_interface_XXXXXX | str trim)
    let fixture_root = ($tmp_root | path join "repo")
    let nushell_dev_dir = ($fixture_root | path join "nushell" "scripts" "dev")
    let packaging_dir = ($fixture_root | path join "packaging")

    mkdir $fixture_root
    mkdir ($fixture_root | path join "assets")
    mkdir ($fixture_root | path join "config_metadata")
    mkdir ($fixture_root | path join "configs")
    mkdir ($fixture_root | path join "docs")
    mkdir ($fixture_root | path join "home_manager")
    mkdir $nushell_dev_dir
    mkdir $packaging_dir
    mkdir ($fixture_root | path join "rust_plugins")
    mkdir ($fixture_root | path join "shells")

    for file_name in [
        ".taplo.toml"
        "CHANGELOG.md"
        "flake.lock"
        "flake.nix"
        "maintainer_shell.nix"
        "yazelix_default.toml"
        "yazelix_package.nix"
        "yazelix_runtime_package.nix"
    ] {
        ^cp ($repo_root | path join $file_name) ($fixture_root | path join $file_name)
    }

    for file_name in [
        "mk_runtime_tree.nix"
        "mk_yazelix_package.nix"
        "runtime_deps.nix"
    ] {
        ^cp ($repo_root | path join "packaging" $file_name) ($packaging_dir | path join $file_name)
    }

    ^cp ($repo_root | path join "nushell" "scripts" "dev" "validate_flake_interface.nu") ($nushell_dev_dir | path join "validate_flake_interface.nu")
    "{ ... }: {}" | save --force --raw ($fixture_root | path join "home_manager" "module.nix")

    {
        fixture_root: $fixture_root
        validator_path: ($nushell_dev_dir | path join "validate_flake_interface.nu")
        package_path: ($fixture_root | path join "yazelix_package.nix")
    }
}

def run_flake_interface_validator [fixture: record] {
    cd $fixture.fixture_root
    ^nu $fixture.validator_path | complete
}

def get_fixture_current_version [fixture: record] {
    (
        open --raw $fixture.constants_path
        | parse --regex 'export const YAZELIX_VERSION = "(?<version>v[^"]+)"'
        | get -o version.0
        | default ""
    )
}

def get_next_patch_version [current_version: string] {
    let parsed = (
        $current_version
        | parse --regex '^(?<major>v\d+)(?:\.(?<patch>\d+))?$'
        | get -o 0
        | default null
    )
    if $parsed == null {
        error make {msg: $"Could not derive next patch version from `($current_version)`"}
    }

    let next_patch = (($parsed.patch? | default "0") | into int) + 1
    $"($parsed.major).($next_patch)"
}

def prepare_releasable_unreleased_fixture [fixture: record] {
    let updated_notes = (
        open $fixture.notes_path
        | upsert releases.unreleased.headline "Backend seam cleanup and release automation"
        | upsert releases.unreleased.summary [
            "Finalized the source-vs-installed runtime identity cleanup so repo shells stop exporting a fake installed runtime root."
            "Added `yzx dev bump` to rotate release metadata, update `YAZELIX_VERSION`, and create the matching release tag."
        ]
    )
    $updated_notes | to toml | save --force --raw $fixture.notes_path

    let changelog = (open --raw $fixture.changelog_path)
    let custom_unreleased = (
        [
            "## Unreleased"
            ""
            "Backend seam cleanup and release automation"
            ""
            "Upgrade impact: no user action required"
            ""
            "Highlights:"
            "- Finalized the source-vs-installed runtime identity cleanup so repo shells stop exporting a fake installed runtime root."
            "- Added `yzx dev bump` to rotate release metadata, update `YAZELIX_VERSION`, and create the matching release tag."
        ] | str join "\n"
    )
    let updated_changelog = (
        $changelog
        | str replace -r '(?ms)^## Unreleased\n.*?(?=\n## )' $custom_unreleased
    )
    $updated_changelog | save --force --raw $fixture.changelog_path
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: issue/bead reconciliation planning catches create, reopen, close, and duplicate cases.
def test_issue_bead_reconciliation_plan [] {
    print "🧪 Testing issue/bead reconciliation plans create, reopen, close, and reject duplicates..."

    try {
        let command = '
            source nushell/scripts/maintainer/issue_bead_contract.nu
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
            source nushell/scripts/maintainer/issue_bead_contract.nu
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
                expected_body: $"Automated: Tracked in Beads as `($bead.id)`."
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

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Regression: maintainer commands resolve a writable repo root even when the stable CLI carries an installed runtime env.
def test_maintainer_repo_root_prefers_checkout_over_installed_runtime_env [] {
    print "🧪 Testing maintainer repo-root resolution prefers the checkout over installed runtime env..."

    let repo_root = ($env.PWD | path expand)
    let repo_subdir = ($repo_root | path join "nushell" "scripts")
    let fake_runtime = (get_yazelix_state_dir | path join "runtime" "current" | path expand)

    try {
        let resolved = (with-env {
            YAZELIX_RUNTIME_DIR: $fake_runtime
            YAZELIX_DIR: "/nix/store/fake-yazelix-runtime"
        } {
            cd $repo_subdir
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

def write_profile_handoff_probe_nu [probe_path: string] {
    [
        "#!/bin/sh"
        ": > \"$YZX_PROFILE_NU_LOG\""
        "for arg in \"$@\"; do"
        "  printf '%s\n' \"$arg\" >> \"$YZX_PROFILE_NU_LOG\""
        "done"
        "cat >> \"$YAZELIX_STARTUP_PROFILE_REPORT\" <<EOF"
        "{\"type\":\"step\",\"schema_version\":1,\"run_id\":\"${YAZELIX_STARTUP_PROFILE_RUN_ID}\",\"scenario\":\"${YAZELIX_STARTUP_PROFILE_SCENARIO}\",\"component\":\"inner\",\"step\":\"zellij_handoff_ready\",\"started_ns\":1,\"ended_ns\":2,\"duration_ms\":0.0,\"recorded_at\":\"2026-04-18T00:00:00.000+00:00\",\"metadata\":{}}"
        "EOF"
        "exit 0"
    ] | str join "\n" | save --force --raw $probe_path
    ^chmod +x $probe_path
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: profiling from outside the repo must fall back to the active installed runtime instead of requiring a writable checkout.
def test_dev_profile_desktop_uses_installed_runtime_outside_repo [] {
    print "🧪 Testing desktop startup profiling falls back to the installed runtime outside the repo..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_desktop_profile_runtime_fallback_XXXXXX | str trim)
    let temp_home = ($tmp_root | path join "home")
    let state_dir = ($tmp_root | path join "state")
    let fake_bin = ($tmp_root | path join "bin")
    let fake_nu = ($fake_bin | path join "nu")
    let invocation_log = ($tmp_root | path join "nu_invocation.log")
    let runtime_root = ($tmp_root | path join "runtime")
    let desktop_module = ($runtime_root | path join "nushell" "scripts" "yzx" "desktop.nu")
    let profile_module = ($repo_root | path join "nushell" "scripts" "yzx" "dev.nu")

    mkdir $temp_home
    mkdir $state_dir
    mkdir $fake_bin
    mkdir ($desktop_module | path dirname)
    mkdir ($runtime_root | path join "nushell" "scripts" "core")
    "" | save --force --raw ($runtime_root | path join "yazelix_default.toml")
    "" | save --force --raw $desktop_module
    "" | save --force --raw ($runtime_root | path join "nushell" "scripts" "core" "start_yazelix.nu")
    write_profile_handoff_probe_nu $fake_nu

    let result = (try {
        let snippet = (
            [
                $"cd \"($temp_home)\""
                $"source \"($profile_module)\""
                "let summary = (run_desktop_profile_command)"
                "{"
                "    scenario: $summary.run.scenario"
                "    source_kind: ($summary.run.metadata.source_kind? | default \"\")"
                "    source_root: ($summary.run.metadata.source_root? | default \"\")"
                "} | to json -r"
            ] | str join "\n"
        )
        let output = (with-env {
            HOME: $temp_home
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_RUNTIME_DIR: $runtime_root
            YAZELIX_NU_BIN: $fake_nu
            YZX_PROFILE_NU_LOG: $invocation_log
            IN_YAZELIX_SHELL: ""
        } {
            do { ^nu -c $snippet } | complete
        })
        let stdout = ($output.stdout | str trim)
        let resolved = if $output.exit_code == 0 { $stdout | lines | last | from json } else { null }
        let invocation = if ($invocation_log | path exists) { open --raw $invocation_log | lines } else { [] }
        let command = ($invocation | get -o 1 | default "")

        if (
            ($output.exit_code == 0)
            and ($resolved.scenario == "desktop_launch")
            and ($resolved.source_kind == "installed_runtime")
            and ($resolved.source_root == $runtime_root)
            and (($invocation | get -o 0 | default "") == "-c")
            and ($command | str contains $"use \"($desktop_module)\" *; yzx desktop launch")
        ) {
            print "  ✅ Desktop profiling now works from outside the repo by using the installed runtime root"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim)) invocation=(($invocation | to json -r))"
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
# Regression: desktop profiling must invoke the exported leaf command and wait for the profiled startup handoff before summarizing.
def test_dev_profile_desktop_invokes_leaf_command_and_waits_for_handoff [] {
    print "🧪 Testing desktop startup profiling invokes yzx desktop launch and waits for handoff..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_desktop_profile_harness_XXXXXX | str trim)
    let temp_home = ($tmp_root | path join "home")
    let state_dir = ($tmp_root | path join "state")
    let fake_bin = ($tmp_root | path join "bin")
    let fake_nu = ($fake_bin | path join "nu")
    let invocation_log = ($tmp_root | path join "nu_invocation.log")
    let profile_module = ($repo_root | path join "nushell" "scripts" "yzx" "dev.nu")
    let desktop_module = ($repo_root | path join "nushell" "scripts" "yzx" "desktop.nu")

    mkdir $temp_home
    mkdir $state_dir
    mkdir $fake_bin
    write_profile_handoff_probe_nu $fake_nu

    let result = (try {
        let snippet = (
            [
                $"source \"($profile_module)\""
                "let summary = (run_desktop_profile_command)"
                "{"
                "    scenario: $summary.run.scenario"
                "    steps: ($summary.steps | each {|step| {component: $step.component step: $step.step}})"
                "} | to json -r"
            ] | str join "\n"
        )
        let output = (with-env {
            HOME: $temp_home
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_NU_BIN: $fake_nu
            YZX_PROFILE_NU_LOG: $invocation_log
            IN_YAZELIX_SHELL: ""
        } {
            do { ^nu -c $snippet } | complete
        })
        let stdout = ($output.stdout | str trim)
        let resolved = if $output.exit_code == 0 { $stdout | lines | last | from json } else { null }
        let invocation = if ($invocation_log | path exists) { open --raw $invocation_log | lines } else { [] }
        let command = ($invocation | get -o 1 | default "")

        if (
            ($output.exit_code == 0)
            and ($resolved.scenario == "desktop_launch")
            and ($resolved.steps | any {|step| $step.component == "inner" and $step.step == "zellij_handoff_ready" })
            and (($invocation | get -o 0 | default "") == "-c")
            and ($command | str contains $"use \"($desktop_module)\" *; yzx desktop launch")
            and not ($command | str contains "\"desktop launch\"")
        ) {
            print "  ✅ Desktop profiling uses the exported leaf command and waits for the handoff marker"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim)) invocation=(($invocation | to json -r))"
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
# Regression: managed-launch profiling must invoke the exported leaf command with real flags and wait for profiled startup completion.
def test_dev_profile_launch_invokes_leaf_command_with_flags [] {
    print "🧪 Testing managed-launch startup profiling invokes yzx launch with flags..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_launch_profile_harness_XXXXXX | str trim)
    let temp_home = ($tmp_root | path join "home")
    let state_dir = ($tmp_root | path join "state")
    let fake_bin = ($tmp_root | path join "bin")
    let fake_nu = ($fake_bin | path join "nu")
    let invocation_log = ($tmp_root | path join "nu_invocation.log")
    let profile_module = ($repo_root | path join "nushell" "scripts" "yzx" "dev.nu")
    let launch_module = ($repo_root | path join "nushell" "scripts" "yzx" "launch.nu")

    mkdir $temp_home
    mkdir $state_dir
    mkdir $fake_bin
    write_profile_handoff_probe_nu $fake_nu

    let result = (try {
        let snippet = (
            [
                $"source \"($profile_module)\""
                "let summary = (run_launch_profile_command --terminal ghostty --verbose)"
                "{"
                "    scenario: $summary.run.scenario"
                "    steps: ($summary.steps | each {|step| {component: $step.component step: $step.step}})"
                "} | to json -r"
            ] | str join "\n"
        )
        let output = (with-env {
            HOME: $temp_home
            YAZELIX_STATE_DIR: $state_dir
            YAZELIX_NU_BIN: $fake_nu
            YZX_PROFILE_NU_LOG: $invocation_log
            IN_YAZELIX_SHELL: ""
        } {
            do { ^nu -c $snippet } | complete
        })
        let stdout = ($output.stdout | str trim)
        let resolved = if $output.exit_code == 0 { $stdout | lines | last | from json } else { null }
        let invocation = if ($invocation_log | path exists) { open --raw $invocation_log | lines } else { [] }
        let command = ($invocation | get -o 1 | default "")

        if (
            ($output.exit_code == 0)
            and ($resolved.scenario == "managed_launch")
            and ($resolved.steps | any {|step| $step.component == "inner" and $step.step == "zellij_handoff_ready" })
            and (($invocation | get -o 0 | default "") == "-c")
            and ($command | str contains $"use \"($launch_module)\" *; yzx launch --terminal \"ghostty\" --verbose")
        ) {
            print "  ✅ Managed-launch profiling uses the exported leaf command, preserves flags, and waits for the handoff marker"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim)) invocation=(($invocation | to json -r))"
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
# Defends: detached terminal launch profiling records the spawn/probe wait as its own measurable phase.
def test_startup_profile_records_detached_terminal_probe [] {
    print "🧪 Testing startup profiling records detached terminal launch probe timing..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_terminal_probe_profile_XXXXXX | str trim)
    let state_dir = ($tmp_root | path join "state")
    let profile_module = ($repo_root | path join "nushell" "scripts" "utils" "startup_profile.nu")
    let terminal_module = ($repo_root | path join "nushell" "scripts" "utils" "terminal_launcher.nu")

    mkdir $state_dir

    let result = (try {
        let snippet = (
            [
                $"use \"($profile_module)\" [create_startup_profile_run load_startup_profile_report]"
                $"use \"($terminal_module)\" [run_detached_terminal_launch]"
                "let run = (create_startup_profile_run \"terminal_probe_unit\" {mode: \"maintainer\"})"
                "with-env $run.env {"
                "    run_detached_terminal_launch \"sleep 2\" \"Probe Terminal\""
                "}"
                "let summary = (load_startup_profile_report $run.report_path)"
                "{"
                "    steps: ($summary.steps | each {|step| {component: $step.component step: $step.step metadata: $step.metadata duration_ms: $step.duration_ms}})"
                "} | to json -r"
            ] | str join "\n"
        )
        let output = (with-env {
            YAZELIX_STATE_DIR: $state_dir
        } {
            do { ^nu -c $snippet } | complete
        })
        let stdout = ($output.stdout | str trim)
        let resolved = if $output.exit_code == 0 { $stdout | lines | last | from json } else { null }
        let probe_step = if $resolved != null {
            $resolved.steps | where {|step| $step.component == "terminal_launcher" and $step.step == "detached_launch_probe" } | get -o 0
        } else {
            null
        }

        if (
            ($output.exit_code == 0)
            and ($probe_step != null)
            and (($probe_step.metadata.terminal? | default "") == "Probe Terminal")
            and (($probe_step.duration_ms | into float) > 0.0)
        ) {
            print "  ✅ Detached terminal spawn/probe wait is now measured as a first-class startup profile step"
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
    let bashrc_path = ($temp_home | path join ".bashrc")
    let nushell_config_dir = ($temp_home | path join ".config" "nushell")
    let nushell_config_path = ($nushell_config_dir | path join "config.nu")
    let generated_layout_dir = ($temp_home | path join ".local" "share" "yazelix" "configs" "zellij" "layouts")
    let profile_module = ($repo_root | path join "nushell" "scripts" "yzx" "dev.nu")

    mkdir $temp_home
    mkdir $state_dir
    mkdir ($config_dir | path dirname)
    mkdir $nushell_config_dir
    mkdir $generated_layout_dir
    "" | save --force $bashrc_path
    "" | save --force $nushell_config_path
    "layout { pane }" | save --force ($generated_layout_dir | path join "yzx_side.kdl")

    let result = (try {
        let snippet = (
            [
                $"source \"($profile_module)\""
                "let summary = (run_dev_profile_harness \"maintainer_e2e\" [])"
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
            IN_YAZELIX_SHELL: ""
            YAZELIX_TERMINAL: ""
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
            and ($steps | any {|step| $step.component == "shellhook" and $step.step == "generate_initializers" })
            and ($steps | any {|step| $step.component == "inner" and $step.step == "materialize_runtime_configs" })
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
# Defends: maintainer update requires an explicit activation target for real updates instead of silently falling through to a legacy default.
def test_dev_update_requires_explicit_activation_for_real_updates [] {
    print "🧪 Testing yzx dev update requires an explicit activation target unless canary-only is requested..."

    let repo_root = ($env.PWD | path expand)
    let dev_module = ($repo_root | path join "nushell" "scripts" "maintainer" "update_workflow.nu")

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
        let expected = "yzx dev update now requires --activate profile|home_manager|none unless you are using --canary-only."

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
# Defends: maintainer update refreshes the runtime flake pin directly so packaged runtime tool versions actually move.
def test_dev_update_refreshes_runtime_flake_inputs [] {
    print "🧪 Testing yzx dev update refreshes flake nixpkgs directly..."

    let repo_root = ($env.PWD | path expand)
    let dev_module = ($repo_root | path join "nushell" "scripts" "maintainer" "update_workflow.nu")
    let tmp_root = (^mktemp -d /tmp/yazelix_dev_update_inputs_XXXXXX | str trim)
    let bin_dir = ($tmp_root | path join "bin")
    let log_path = ($tmp_root | path join "update.log")
    let nix_script = ($bin_dir | path join "nix")
    let current_path = if (($env.PATH | describe) | str contains "list") {
        $env.PATH | str join (char esep)
    } else {
        $env.PATH | into string
    }

    mkdir $bin_dir
    [
        "#!/usr/bin/env bash"
        "printf 'nix:%s\\n' \"$*\" >> \"$YZX_TEST_LOG\""
        "exit 0"
    ] | str join "\n" | save --force --raw $nix_script
    ^chmod +x $nix_script

    let result = (try {
        let snippet = (
            [
                $"source \"($dev_module)\""
                "refresh_repo_runtime_inputs $env.YZX_TEST_REPO_ROOT"
            ] | str join "\n"
        )
        let output = (with-env {
            PATH: $"($bin_dir)(char esep)($current_path)"
            YZX_TEST_LOG: $log_path
            YZX_TEST_REPO_ROOT: $repo_root
        } {
            ^nu -c $snippet | complete
        })
        let stdout = ($output.stdout | str trim)
        let log_lines = if ($log_path | path exists) {
            open --raw $log_path | lines
        } else {
            []
        }
        let expected_log = [
            $"nix:flake update nixpkgs --flake ($repo_root)"
        ]

        if (
            ($output.exit_code == 0)
            and ($log_lines == $expected_log)
            and ($stdout | str contains "✅ flake.lock nixpkgs input updated.")
        ) {
            print "  ✅ Maintainer update now refreshes the runtime flake pin directly"
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

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Defends: maintainer update syncs runtime tool pins from the locked flake instead of host tool drift.
def test_dev_update_syncs_runtime_tool_pins_from_locked_flake [] {
    print "🧪 Testing yzx dev update syncs Nix and Nushell runtime pins from the locked flake..."

    let repo_root = ($env.PWD | path expand)
    let dev_module = ($repo_root | path join "nushell" "scripts" "maintainer" "update_workflow.nu")
    let tmp_root = (^mktemp -d /tmp/yazelix_dev_update_runtime_pins_XXXXXX | str trim)
    let fixture_repo = ($tmp_root | path join "repo")
    let constants_dir = ($fixture_repo | path join "nushell" "scripts" "utils")
    let bin_dir = ($tmp_root | path join "bin")
    let log_path = ($tmp_root | path join "nix.log")
    let nix_script = ($bin_dir | path join "nix")
    let current_path = if (($env.PATH | describe) | str contains "list") {
        $env.PATH | str join (char esep)
    } else {
        $env.PATH | into string
    }

    mkdir $constants_dir
    mkdir $bin_dir
    mkdir $fixture_repo
    ^git -C $fixture_repo init -q
    "{}" | save --force --raw ($fixture_repo | path join "flake.nix")
    "" | save --force --raw ($fixture_repo | path join "yazelix_default.toml")
    [
        'export const YAZELIX_VERSION = "v14"'
        'export const PINNED_NIX_VERSION = "0.0.0"'
        'export const PINNED_NUSHELL_VERSION = "0.0.0"'
        ""
    ] | str join "\n" | save --force --raw ($constants_dir | path join "constants.nu")

    [
        "#!/usr/bin/env bash"
        "printf 'nix:%s\\n' \"$*\" >> \"$YZX_TEST_LOG\""
        "case \"$*\" in"
        "  *nixVersions.latest.version*) printf '2.34.5\\n' ;;"
        "  *nushell.version*) printf '0.112.1\\n' ;;"
        "  *) printf 'unexpected nix invocation: %s\\n' \"$*\" >&2; exit 1 ;;"
        "esac"
    ] | str join "\n" | save --force --raw $nix_script
    ^chmod +x $nix_script

    let result = (try {
        let snippet = (
            [
                $"source \"($dev_module)\""
                "sync_runtime_pins"
            ] | str join "\n"
        )
        let output = (with-env {
            PATH: $"($bin_dir)(char esep)($current_path)"
            YZX_TEST_LOG: $log_path
        } {
            do {
                cd $fixture_repo
                ^nu -c $snippet | complete
            }
        })
        let stdout = ($output.stdout | str trim)
        let constants = (open --raw ($constants_dir | path join "constants.nu"))
        let log_lines = if ($log_path | path exists) {
            open --raw $log_path | lines
        } else {
            []
        }
        let log_text = ($log_lines | str join "\n")

        if (
            ($output.exit_code == 0)
            and ($constants | str contains 'export const PINNED_NIX_VERSION = "2.34.5"')
            and ($constants | str contains 'export const PINNED_NUSHELL_VERSION = "0.112.1"')
            and ($stdout | str contains "✅ Updated runtime pins: nix 2.34.5, nushell 0.112.1")
            and ($log_text | str contains "nixVersions.latest.version")
            and ($log_text | str contains "nushell.version")
        ) {
            print "  ✅ Runtime pins now sync from the locked flake, including Nushell, without trusting host tool drift"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) constants=($constants) log=(($log_lines | to json -r)) stderr=(($output.stderr | str trim))"
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
# Defends: maintainer profile activation removes old default-profile Yazelix entries and installs the local checkout package.
def test_dev_update_profile_activation_reinstalls_local_package [] {
    print "🧪 Testing yzx dev update profile activation removes old profile entries and installs the local checkout package..."

    let repo_root = ($env.PWD | path expand)
    let dev_module = ($repo_root | path join "nushell" "scripts" "maintainer" "update_workflow.nu")
    let tmp_root = (^mktemp -d /tmp/yazelix_dev_update_profile_activation_XXXXXX | str trim)
    let bin_dir = ($tmp_root | path join "bin")
    let log_path = ($tmp_root | path join "activation.log")
    let nix_script = ($bin_dir | path join "nix")
    let current_path = if (($env.PATH | describe) | str contains "list") {
        $env.PATH | str join (char esep)
    } else {
        $env.PATH | into string
    }

    mkdir $bin_dir
    let profile_list_json = ({
        elements: {
            yazelix: {
                active: true
                originalUrl: "github:luccahuguet/yazelix"
                attrPath: "packages.x86_64-linux.yazelix"
                storePaths: ["/nix/store/fake-yazelix"]
            }
            git: {
                active: true
                originalUrl: "flake:nixpkgs"
                attrPath: "legacyPackages.x86_64-linux.git"
                storePaths: ["/nix/store/fake-git"]
            }
        }
        version: 3
    } | to json -r)
    [
        "#!/usr/bin/env bash"
        "printf 'nix:%s\\n' \"$*\" >> \"$YZX_TEST_LOG\""
        "if [ \"$1\" = \"profile\" ] && [ \"$2\" = \"list\" ] && [ \"$3\" = \"--json\" ]; then"
        "  printf '%s\\n' \"$YZX_TEST_PROFILE_LIST_JSON\""
        "  exit 0"
        "fi"
        "printf 'profile-stream-line\\n'"
        "exit 0"
    ] | str join "\n" | save --force --raw $nix_script
    ^chmod +x $nix_script

    let result = (try {
        let snippet = (
            [
                $"source \"($dev_module)\""
                "activate_updated_profile_runtime $env.YZX_TEST_REPO_ROOT"
            ] | str join "\n"
        )
        let output = (with-env {
            PATH: $"($bin_dir)(char esep)($current_path)"
            YZX_TEST_LOG: $log_path
            YZX_TEST_REPO_ROOT: $repo_root
            YZX_TEST_PROFILE_LIST_JSON: $profile_list_json
        } {
            ^nu -c $snippet | complete
        })
        let stdout = ($output.stdout | str trim)
        let log_lines = if ($log_path | path exists) {
            open --raw $log_path | lines
        } else {
            []
        }

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "🔄 Activating updated local Yazelix package in the default Nix profile...")
            and ($stdout | str contains "Streaming local profile activation logs")
            and ($stdout | str contains "Removing existing Yazelix profile entries before installing the local checkout: yazelix")
            and ($stdout | str contains "profile-stream-line")
            and ($stdout | str contains "✅ Default-profile Yazelix package updated from the local checkout.")
            and ($log_lines | any {|line| $line == "nix:profile list --json" })
            and ($log_lines | any {|line| $line == "nix:profile remove yazelix" })
            and ($log_lines | any {|line| $line == "nix:profile add --refresh -L .#yazelix" })
        ) {
            print "  ✅ Profile activation now replaces older default-profile Yazelix entries and installs the local checkout package"
            true
        } else {
            print $"  ❌ Unexpected profile activation result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim)) log=(($log_lines | str join '; '))"
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
# Defends: Home Manager activation refreshes the configured flake input and switches the requested flake ref instead of falling back to the installer path.
def test_dev_update_home_manager_activation_refreshes_input_and_switches_requested_ref [] {
    print "🧪 Testing yzx dev update Home Manager activation refreshes the input lock and switches the requested ref..."

    let repo_root = ($env.PWD | path expand)
    let dev_module = ($repo_root | path join "nushell" "scripts" "maintainer" "update_workflow.nu")
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
                "let result = (activate_updated_home_manager_runtime $env.YZX_TEST_FLAKE_DIR \"yazelix-hm\" \"lucca@loqness\")"
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
            $"nix:flake update yazelix-hm --flake ($flake_dir)"
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

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: Home Manager-owned live-session restart must use the profile yzx wrapper and avoid recreating user-local manual surfaces.
def test_home_manager_profile_restart_uses_owner_wrapper_without_manual_surfaces [] {
    print "🧪 Testing Home Manager profile-owned restart uses the profile yzx wrapper without manual surfaces..."

    let repo_root = ($env.PWD | path expand)
    let tmp_root = (^mktemp -d /tmp/yazelix_home_manager_restart_smoke_XXXXXX | str trim)
    let temp_home = ($tmp_root | path join "home")
    let config_dir = ($temp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    let hm_store = ($tmp_root | path join "abc-home-manager-files")
    let hm_main_config = ($hm_store | path join ".config" "yazelix" "user_configs" "yazelix.toml")
    let profile_yzx = ($temp_home | path join ".nix-profile" "bin" "yzx")
    let profile_desktop = ($temp_home | path join ".nix-profile" "share" "applications" "yazelix.desktop")
    let manual_yzx = ($temp_home | path join ".local" "bin" "yzx")
    let manual_desktop = ($temp_home | path join ".local" "share" "applications" "com.yazelix.Yazelix.desktop")
    let fake_bin = ($tmp_root | path join "bin")
    let yzx_log = ($tmp_root | path join "profile_yzx.log")
    let zellij_log = ($tmp_root | path join "zellij.log")

    mkdir $temp_home
    mkdir $user_config_dir
    mkdir ($hm_main_config | path dirname)
    mkdir ($profile_yzx | path dirname)
    mkdir ($profile_desktop | path dirname)
    mkdir $fake_bin

    cp ($repo_root | path join "yazelix_default.toml") $hm_main_config
    rm -f ($user_config_dir | path join "yazelix.toml")
    ^ln -s $hm_main_config ($user_config_dir | path join "yazelix.toml")

    [
        "#!/bin/sh"
        ": > \"$YZX_TEST_PROFILE_YZX_LOG\""
        "printf '%s\n' \"$@\" >> \"$YZX_TEST_PROFILE_YZX_LOG\""
        "printf 'BOOTSTRAP=%s\n' \"${YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE-unset}\" >> \"$YZX_TEST_PROFILE_YZX_LOG\""
        "exit 0"
    ] | str join "\n" | save --force --raw $profile_yzx
    ^chmod +x $profile_yzx

    [
        "#!/bin/sh"
        "printf '%s\n' \"$*\" >> \"$YZX_TEST_ZELLIJ_LOG\""
        "exit 0"
    ] | str join "\n" | save --force --raw ($fake_bin | path join "zellij")
    ^chmod +x ($fake_bin | path join "zellij")

    [
        "[Desktop Entry]"
        "Type=Application"
        "Name=Yazelix"
        "Terminal=true"
        $"Exec=\"($profile_yzx)\" desktop launch"
    ] | str join "\n" | save --force --raw $profile_desktop

    let result = (try {
        let output = (with-env {
            HOME: $temp_home
            XDG_CONFIG_HOME: ($temp_home | path join ".config")
            XDG_DATA_HOME: ($temp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            PATH: ([$fake_bin] | append $env.PATH)
            ZELLIJ_SESSION_NAME: "old-yazelix"
            YAZELIX_TERMINAL: "ghostty"
            YZX_TEST_PROFILE_YZX_LOG: $yzx_log
            YZX_TEST_ZELLIJ_LOG: $zellij_log
        } {
            ^nu -c $"use \"($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")\" *; yzx restart" | complete
        })
        let yzx_lines = if ($yzx_log | path exists) { open --raw $yzx_log | lines } else { [] }
        let zellij_lines = if ($zellij_log | path exists) { open --raw $zellij_log | lines } else { [] }
        let bootstrap_line = ($yzx_lines | where {|line| $line | str starts-with "BOOTSTRAP=" } | get -o 0 | default "")
        let bootstrap_path = ($bootstrap_line | str replace "BOOTSTRAP=" "")

        if (
            ($output.exit_code == 0)
            and (($yzx_lines | get -o 0 | default "") == "launch")
            and ($bootstrap_path | path exists)
            and ($zellij_lines | any {|line| $line == "kill-session old-yazelix" })
            and ($profile_desktop | path exists)
            and (not ($manual_yzx | path exists))
            and (not ($manual_desktop | path exists))
        ) {
            print "  ✅ Home Manager restart uses the profile yzx owner and does not recreate manual yzx or desktop surfaces"
            true
        } else {
            print $"  ❌ Unexpected Home Manager restart result: exit=($output.exit_code) yzx=(($yzx_lines | to json -r)) zellij=(($zellij_lines | to json -r)) manual_yzx=(($manual_yzx | path exists)) manual_desktop=(($manual_desktop | path exists)) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
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
    let dev_module = ($repo_root | path join "nushell" "scripts" "maintainer" "update_workflow.nu")

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
        let expected = "Unknown activation mode: hm. Expected one of: profile, home_manager, none"

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

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Defends: yzx dev bump rotates Unreleased metadata, updates YAZELIX_VERSION, creates a dedicated commit, and creates the matching tag.
def test_dev_bump_rotates_release_metadata_and_tags_the_repo [] {
    print "🧪 Testing yzx dev bump rotates release metadata and creates the matching tag..."

    let fixture = (setup_dev_bump_fixture)
    let current_version = (get_fixture_current_version $fixture)
    let target_version = (get_next_patch_version $current_version)
    prepare_releasable_unreleased_fixture $fixture
    commit_dev_bump_fixture_change $fixture "Prepare unreleased release notes"

    let result = (try {
        let snippet = (
            [
                $"use \"($fixture.helper_module)\" [perform_version_bump]"
                $"perform_version_bump \"($fixture.repo_root)\" \"($target_version)\" | to json -r"
            ] | str join "\n"
        )
        let output = (^nu -c $snippet | complete)
        let stdout = ($output.stdout | str trim)
        let resolved = if $output.exit_code == 0 {
            $stdout | lines | last | from json
        } else {
            null
        }
        let constants = (open --raw $fixture.constants_path)
        let notes = (open $fixture.notes_path)
        let changelog = (open --raw $fixture.changelog_path)
        let readme = (open --raw $fixture.readme_path)
        let commit_subject = (^git -C $fixture.repo_root log -1 --pretty=%s | str trim)
        let tags = (^git -C $fixture.repo_root tag --list | lines)

        if (
            ($output.exit_code == 0)
            and ($resolved != null)
            and ($resolved.previous_version == $current_version)
            and ($resolved.target_version == $target_version)
            and ($constants | str contains $"export const YAZELIX_VERSION = \"($target_version)\"")
            and (($notes.releases | columns) | any {|column| $column == $target_version })
            and ($notes.releases.unreleased.headline == $"Post-($target_version) work in progress")
            and ($notes.releases.unreleased.summary == [$"Reserved for post-release changes after ($target_version) lands."])
            and ($changelog | str contains $"## ($target_version) - ")
            and ($changelog | str contains "## Unreleased")
            and ($changelog | str contains "Backend seam cleanup and release automation")
            and ($readme | lines | first) == $"# Yazelix ($target_version)"
            and ($readme | str contains $"## Latest Tagged Release: ($target_version)")
            and ($readme | str contains "Backend seam cleanup and release automation")
            and ($commit_subject == $"Bump version to ($target_version)")
            and ($target_version in $tags)
        ) {
            print "  ✅ yzx dev bump now rotates release metadata, updates the version constant, commits, and tags deterministically"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) commit_subject=($commit_subject) tags=(($tags | to json -r))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf ($fixture.repo_root | path dirname)
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: yzx dev bump refuses to run on a dirty worktree instead of mixing release automation with unrelated edits.
def test_dev_bump_rejects_dirty_worktrees [] {
    print "🧪 Testing yzx dev bump rejects dirty worktrees..."

    let fixture = (setup_dev_bump_fixture)
    let target_version = (get_next_patch_version (get_fixture_current_version $fixture))
    "dirty\n" | save --append --raw $fixture.readme_path

    let result = (try {
        let snippet = (
            [
                $"use \"($fixture.helper_module)\" [perform_version_bump]"
                "try {"
                $"    perform_version_bump \"($fixture.repo_root)\" \"($target_version)\" | ignore"
                "    print \"unexpected-success\""
                "} catch {|err|"
                "    print $err.msg"
                "}"
            ] | str join "\n"
        )
        let output = (^nu -c $snippet | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "yzx dev bump requires a clean git worktree.") {
            print "  ✅ yzx dev bump now fails fast on a dirty worktree"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf ($fixture.repo_root | path dirname)
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: yzx dev bump refuses to reuse an existing release tag.
def test_dev_bump_rejects_existing_target_tags [] {
    print "🧪 Testing yzx dev bump rejects existing target tags..."

    let fixture = (setup_dev_bump_fixture)
    let target_version = (get_next_patch_version (get_fixture_current_version $fixture))
    prepare_releasable_unreleased_fixture $fixture
    commit_dev_bump_fixture_change $fixture "Prepare unreleased release notes"
    ^git -C $fixture.repo_root tag -a $target_version -m "Existing tag"

    let result = (try {
        let snippet = (
            [
                $"use \"($fixture.helper_module)\" [perform_version_bump]"
                "try {"
                $"    perform_version_bump \"($fixture.repo_root)\" \"($target_version)\" | ignore"
                "    print \"unexpected-success\""
                "} catch {|err|"
                "    print $err.msg"
                "}"
            ] | str join "\n"
        )
        let output = (^nu -c $snippet | complete)
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == $"Tag already exists: ($target_version)") {
            print "  ✅ yzx dev bump now refuses to reuse an existing git tag"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf ($fixture.repo_root | path dirname)
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: validate_flake_interface must fail when the first-party flake package narrows meta.platforms below the systems exported by flake.nix.
def test_validate_flake_interface_rejects_narrowed_first_party_platforms [] {
    print "🧪 Testing validate_flake_interface rejects exported darwin systems that are missing from first-party meta.platforms..."

    let fixture = (setup_flake_interface_fixture)
    let result = (try {
        let baseline = (run_flake_interface_validator $fixture)
        let baseline_stdout = ($baseline.stdout | str trim)
        let baseline_ok = (
            ($baseline.exit_code == 0)
            and ($baseline_stdout | str contains "First-party flake package is available on all exported systems")
        )

        let narrowed_package = (
            open --raw $fixture.package_path
            | str replace -r '(?ms)let\s+firstPartyPlatforms = \[\n.*?\n  \];' (
                [
                    "let"
                    "  firstPartyPlatforms = ["
                    '    "x86_64-linux"'
                    '    "aarch64-linux"'
                    "  ];"
                ] | str join "\n"
            )
        )
        $narrowed_package | save --force --raw $fixture.package_path

        let broken = (run_flake_interface_validator $fixture)
        let broken_output = (
            [$broken.stdout $broken.stderr]
            | str join "\n"
            | str replace -r '(?m)\n\s*\|\s*' ""
            | str trim
        )
        let broken_meta_mentions = (
            ($broken_output | split row 'meta.platforms=["x86_64-linux","aarch64-linux"]' | length) - 1
        )

        if (
            $baseline_ok
            and ($broken.exit_code != 0)
            and ($broken_output | str contains "reports as unavailable on exported systems")
            and ($broken_output | str contains "aarch64-darwin")
            and ($broken_meta_mentions == 2)
        ) {
            print "  ✅ validate_flake_interface now fails when exported darwin package systems fall out of first-party meta.platforms"
            true
        } else {
            print $"  ❌ Unexpected result: baseline_exit=($baseline.exit_code) baseline_stdout=($baseline_stdout) broken_exit=($broken.exit_code) broken_output=($broken_output)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf ($fixture.fixture_root | path dirname)
    $result
}

def main [] {
    print "=== Testing yzx Maintainer Commands ==="
    print ""

    let results = [
        (test_issue_bead_reconciliation_plan)
        (test_issue_bead_comment_plan)
        (test_dev_update_requires_explicit_activation_for_real_updates)
        (test_dev_update_refreshes_runtime_flake_inputs)
        (test_dev_update_syncs_runtime_tool_pins_from_locked_flake)
        (test_dev_update_profile_activation_reinstalls_local_package)
        (test_dev_update_activation_mode_rejects_unknown_values)
        (test_dev_update_home_manager_activation_refreshes_input_and_switches_requested_ref)
        (test_home_manager_profile_restart_uses_owner_wrapper_without_manual_surfaces)
        (test_dev_bump_rotates_release_metadata_and_tags_the_repo)
        (test_dev_bump_rejects_dirty_worktrees)
        (test_dev_bump_rejects_existing_target_tags)
        (test_maintainer_repo_root_prefers_checkout_over_installed_runtime_env)
        (test_default_budget_profiler_does_not_wait_on_background_children)
        (test_vendored_yazi_plugin_refresh_applies_patch_and_refuses_dirty_targets)
        (test_nushell_initializer_restores_current_path_first)
        (test_validate_flake_interface_rejects_narrowed_first_party_platforms)
        (test_startup_profile_report_schema_is_structured_and_summarizable)
        (test_dev_profile_desktop_uses_installed_runtime_outside_repo)
        (test_dev_profile_desktop_invokes_leaf_command_and_waits_for_handoff)
        (test_dev_profile_launch_invokes_leaf_command_with_flags)
        (test_startup_profile_records_detached_terminal_probe)
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
