#!/usr/bin/env nu
# Historical upgrade-notes E2E runner

use ./yzx_test_helpers.nu [get_repo_root]
use ../utils/upgrade_summary.nu [build_upgrade_summary_report]

def log_line [log_file: string, line: string] {
    print $line
    $"($line)\n" | save --append --raw $log_file
}

def log_block [log_file: string, title: string, content: string] {
    log_line $log_file $"=== ($title) ==="
    if ($content | is-empty) {
        log_line $log_file "<empty>"
    } else {
        for line in ($content | lines) {
            log_line $log_file $line
        }
    }
    log_line $log_file ""
}

export def main [] {
    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_historical_notes_e2e_XXXXXX | str trim)
    let state_dir = ($tmp_home | path join ".local" "share" "yazelix")
    let log_file = ($tmp_home | path join "historical_upgrade_notes_e2e.log")
    mkdir ($tmp_home | path join ".local" "share")
    mkdir $state_dir
    "" | save --force --raw $log_file

    let tags = (
        ^git -C $repo_root tag --sort=creatordate
        | lines
        | where {|tag| ($tag | str starts-with "v12") or ($tag | str starts-with "v13") }
    )
    let notes = (open ($repo_root | path join "docs" "upgrade_notes.toml"))
    let release_keys = ($notes.releases | columns)
    let missing = ($tags | where {|tag| not ($tag in $release_keys) })

    log_line $log_file "Case: historical exact-version upgrade notes can be loaded for the supported v12/v13 floor"
    log_block $log_file "Supported tags" ($tags | str join "\n")
    log_block $log_file "Release keys" ($release_keys | str join "\n")
    log_block $log_file "Missing keys" ($missing | str join "\n")

    let sample_versions = ["v12", "v12.10", "v13.2", "v13.3", "v13.7"]
    let reports = (
        $sample_versions
        | each {|version|
            with-env {
                HOME: $tmp_home
                YAZELIX_RUNTIME_DIR: $repo_root
                YAZELIX_CONFIG_OVERRIDE: ($repo_root | path join "yazelix_default.toml")
                YAZELIX_STATE_DIR: $state_dir
            } {
                build_upgrade_summary_report $version
            }
        }
    )

    for report in $reports {
        log_block $log_file $"Report: ($report.version)" ($report | to json -r)
    }

    let ok = (
        ($missing | is-empty)
        and ($reports | all {|report| $report.found })
        and (($reports | where version == "v12.10" | first).entry.upgrade_impact == "migration_available")
        and (($reports | where version == "v13.2" | first).entry.upgrade_impact == "manual_action_required")
        and (($reports | where version == "v13.3" | first).output | str contains "Restart workspace bootstrap follow-through")
    )

    if $ok {
        log_line $log_file "Result: PASS"
    } else {
        log_line $log_file "Result: FAIL"
    }

    rm -rf $tmp_home

    if $ok {
        print ""
        print "✅ Historical upgrade notes e2e checks passed (1/1)"
    } else {
        print ""
        print "❌ Historical upgrade notes e2e checks failed (0/1)"
        error make {msg: "historical upgrade notes e2e checks failed"}
    }
}
