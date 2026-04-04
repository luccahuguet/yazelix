#!/usr/bin/env nu

use common.nu [get_yazelix_state_dir]

export const STARTUP_PROFILE_SCHEMA_VERSION = 1

def now_rfc3339 [] {
    date now | format date "%Y-%m-%dT%H:%M:%S%.3f%:z"
}

def now_ns [] {
    date now | into int
}

def append_profile_record [report_path: string, record: record] {
    let report_dir = ($report_path | path dirname)
    if not ($report_dir | path exists) {
        mkdir $report_dir
    }

    $"(($record | to json -r))\n" | save --append --raw $report_path
}

export def startup_profile_enabled [] {
    let report_path = ($env.YAZELIX_STARTUP_PROFILE_REPORT? | default "" | into string | str trim)
    let enabled = ($env.YAZELIX_STARTUP_PROFILE? | default "" | into string | str trim)

    ($report_path | is-not-empty) or ($enabled == "true")
}

export def get_startup_profile_report_path [] {
    let configured = ($env.YAZELIX_STARTUP_PROFILE_REPORT? | default "" | into string | str trim)
    if ($configured | is-not-empty) {
        return ($configured | path expand)
    }

    let run_id = ($env.YAZELIX_STARTUP_PROFILE_RUN_ID? | default "" | into string | str trim)
    if ($run_id | is-empty) {
        return ""
    }

    get_yazelix_state_dir | path join "profiles" "startup" $"($run_id).jsonl"
}

export def create_startup_profile_run [
    scenario: string
    metadata?: record
] {
    let run_id = ([
        "startup_profile"
        (date now | format date "%Y%m%d_%H%M%S_%3f")
    ] | str join "_")
    let report_path = (get_yazelix_state_dir | path join "profiles" "startup" $"($run_id).jsonl")
    let resolved_metadata = ($metadata | default {})

    if ($report_path | path exists) {
        rm -f $report_path
    }

    append_profile_record $report_path {
        type: "run"
        schema_version: $STARTUP_PROFILE_SCHEMA_VERSION
        run_id: $run_id
        scenario: $scenario
        created_at: (now_rfc3339)
        metadata: $resolved_metadata
    }

    {
        run_id: $run_id
        report_path: $report_path
        env: {
            YAZELIX_STARTUP_PROFILE: "true"
            YAZELIX_STARTUP_PROFILE_RUN_ID: $run_id
            YAZELIX_STARTUP_PROFILE_REPORT: $report_path
            YAZELIX_STARTUP_PROFILE_SCENARIO: $scenario
        }
    }
}

export def record_startup_profile_event [
    component: string
    step: string
    started_ns: int
    ended_ns: int
    metadata?: record
] {
    if not (startup_profile_enabled) {
        return
    }

    let report_path = (get_startup_profile_report_path)
    if ($report_path | is-empty) {
        return
    }

    let duration_ms = (((($ended_ns - $started_ns) / 1000000.0) * 100.0) | math round | $in / 100.0)
    append_profile_record $report_path {
        type: "step"
        schema_version: $STARTUP_PROFILE_SCHEMA_VERSION
        run_id: ($env.YAZELIX_STARTUP_PROFILE_RUN_ID? | default "")
        scenario: ($env.YAZELIX_STARTUP_PROFILE_SCENARIO? | default "")
        component: $component
        step: $step
        started_ns: $started_ns
        ended_ns: $ended_ns
        duration_ms: $duration_ms
        recorded_at: (now_rfc3339)
        metadata: ($metadata | default {})
    }
}

export def profile_startup_step [
    component: string
    step: string
    code: closure
    metadata?: record
] {
    if not (startup_profile_enabled) {
        return (do $code)
    }

    let started_ns = (now_ns)
    let result = (do $code)
    let ended_ns = (now_ns)
    record_startup_profile_event $component $step $started_ns $ended_ns $metadata
    $result
}

export def load_startup_profile_report [report_path: string] {
    if not ($report_path | path exists) {
        error make {msg: $"Startup profile report not found: ($report_path)"}
    }

    let lines = (open --raw $report_path | lines | where {|line| not ($line | str trim | is-empty) })
    let records = ($lines | each {|line| $line | from json })
    if ($records | is-empty) {
        error make {msg: $"Startup profile report is empty: ($report_path)"}
    }

    let run_record = ($records | where type == "run" | first)
    let step_records = ($records | where type == "step" | default [])

    let total_duration_ms = if ($step_records | is-empty) {
        0.0
    } else {
        let started_ns = ($step_records | get started_ns | math min)
        let ended_ns = ($step_records | get ended_ns | math max)
        (((($ended_ns - $started_ns) / 1000000.0) * 100.0) | math round | $in / 100.0)
    }

    {
        run: $run_record
        steps: ($step_records | sort-by started_ns)
        total_duration_ms: $total_duration_ms
        report_path: $report_path
    }
}

export def render_startup_profile_summary [summary: record] {
    let has_context = (
        $summary.steps
        | any {|record|
            let phase = ($record.metadata.phase? | default "" | into string | str trim)
            let pid = ($record.metadata.pid? | default "" | into string | str trim)
            ($phase | is-not-empty) or ($pid | is-not-empty)
        }
    )
    let rows = (
        $summary.steps
        | each {|record|
            let phase = ($record.metadata.phase? | default "" | into string | str trim)
            let pid = ($record.metadata.pid? | default "" | into string | str trim)
            let context = if ($phase | is-not-empty) and ($pid | is-not-empty) {
                $"($phase)#($pid)"
            } else if ($phase | is-not-empty) {
                $phase
            } else {
                $pid
            }
            if $has_context {
                {
                    Context: $context
                    Component: $record.component
                    Step: $record.step
                    "Duration (ms)": $record.duration_ms
                }
            } else {
                {
                    Component: $record.component
                    Step: $record.step
                    "Duration (ms)": $record.duration_ms
                }
            }
        }
    )

    if ($rows | is-empty) {
        "No startup profile steps recorded."
    } else {
        ($rows | table)
    }
}
