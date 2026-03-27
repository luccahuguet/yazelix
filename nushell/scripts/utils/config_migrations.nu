#!/usr/bin/env nu
# Shared migration registry and preview/apply engine for Yazelix config migrations

const CONFIG_MIGRATION_RULES = [
    {
        id: "remove_zellij_widget_tray_layout"
        title: "Remove the broken layout widget from zellij.widget_tray"
        kind: "removed_value"
        introduced_in: null
        introduced_after_version: "v13.7"
        introduced_on: "2026-03-27"
        auto_apply: true
        user_visible: true
        guarded_paths: ["zellij.widget_tray"]
        rationale: "The zjstatus layout widget was removed because it was broken and now fails validation."
        manual_fix: "Remove \"layout\" from [zellij].widget_tray."
    }
    {
        id: "unify_terminal_preference_list"
        title: "Replace legacy terminal preference fields with terminal.terminals"
        kind: "field_reshape"
        introduced_in: "v11.6"
        introduced_after_version: null
        introduced_on: "2026-01-03"
        auto_apply: true
        user_visible: true
        guarded_paths: ["terminal.preferred_terminal", "terminal.extra_terminals", "terminal.terminals"]
        rationale: "Yazelix now uses one ordered terminal list instead of separate primary and extra terminal fields."
        manual_fix: "Replace terminal.preferred_terminal and terminal.extra_terminals with one ordered [terminal].terminals list."
    }
    {
        id: "remove_shell_enable_atuin"
        title: "Drop the removed shell.enable_atuin toggle"
        kind: "removed_field"
        introduced_in: "v12.10"
        introduced_after_version: null
        introduced_on: "2026-02-22"
        auto_apply: true
        user_visible: true
        guarded_paths: ["shell.enable_atuin"]
        rationale: "Yazelix no longer uses a shell-level enable_atuin toggle and now relies on direct tool availability."
        manual_fix: "Remove shell.enable_atuin from [shell]."
    }
    {
        id: "review_legacy_cursor_trail_settings"
        title: "Review legacy Ghostty cursor-trail settings manually"
        kind: "manual_only"
        introduced_in: "v13.2"
        introduced_after_version: null
        introduced_on: "2026-03-14"
        auto_apply: false
        user_visible: true
        guarded_paths: [
            "terminal.cursor_trail"
            "terminal.ghostty_cursor_effects_random"
            "terminal.ghostty_cursor_effects"
        ]
        rationale: "The old cursor-trail settings were split into separate color and effect fields, and the old combinations are not always safe to rewrite automatically."
        manual_fix: "Replace the legacy cursor-trail fields with terminal.ghostty_trail_color, terminal.ghostty_trail_effect, and terminal.ghostty_mode_effect after reviewing the old intent."
    }
]

def maybe_get [data: any, path: list<string>] {
    mut current = $data

    for segment in $path {
        if not ((($current | describe) | str contains "record")) {
            return null
        }

        let keys = ($current | columns)
        if not ($segment in $keys) {
            return null
        }

        $current = ($current | get $segment)
    }

    $current
}

def has_path [data: any, path: list<string>] {
    mut current = $data

    for segment in $path {
        if not ((($current | describe) | str contains "record")) {
            return false
        }

        let keys = ($current | columns)
        if not ($segment in $keys) {
            return false
        }

        $current = ($current | get $segment)
    }

    true
}

def format_path [path: list<string>] {
    $path | str join "."
}

def compact_string_list [values: list<any>] {
    mut compact = []

    for value in $values {
        let normalized = ($value | into string | str trim)
        if ($normalized | is-empty) {
            continue
        }
        if not ($normalized in $compact) {
            $compact = ($compact | append $normalized)
        }
    }

    $compact
}

def quoted_list [values: list<string>] {
    if ($values | is-empty) {
        "[]"
    } else {
        let rendered = ($values | each {|value| $"\"($value)\"" } | str join ", ")
        $"[($rendered)]"
    }
}

def format_release_context [rule: record] {
    if ($rule.introduced_in | is-not-empty) {
        $"($rule.introduced_in) on ($rule.introduced_on)"
    } else if ($rule.introduced_after_version | is-not-empty) {
        $"after ($rule.introduced_after_version) on ($rule.introduced_on)"
    } else {
        $rule.introduced_on
    }
}

def get_rule [id: string] {
    $CONFIG_MIGRATION_RULES | where id == $id | first
}

def make_result [
    id: string
    status: string
    changes: list<string>
    matched_paths: list<string>
    config_after: any
] {
    let rule = (get_rule $id)
    $rule | merge {
        status: $status
        changes: $changes
        matched_paths: $matched_paths
        config_after: $config_after
    }
}

def plan_remove_zellij_widget_tray_layout [config: record] {
    let path = ["zellij", "widget_tray"]
    let tray = (maybe_get $config $path)

    if $tray == null {
        return null
    }

    if not (($tray | describe) | str contains "list") {
        return (make_result "remove_zellij_widget_tray_layout" "manual_only" [] [(format_path $path)] $config)
    }

    let normalized = ($tray | each {|value| $value | into string })
    let filtered = ($normalized | where {|value| $value != "layout" })

    if ($filtered | length) == ($normalized | length) {
        return null
    }

    let updated_zellij = (($config.zellij? | default {}) | upsert widget_tray $filtered)

    (make_result "remove_zellij_widget_tray_layout" "auto" ['Remove "layout" from [zellij].widget_tray.'] [(format_path $path)] ($config | upsert zellij $updated_zellij))
}

def plan_unify_terminal_preference_list [config: record] {
    let preferred_path = ["terminal", "preferred_terminal"]
    let extra_path = ["terminal", "extra_terminals"]
    let terminals_path = ["terminal", "terminals"]

    let has_preferred = (has_path $config $preferred_path)
    let has_extra = (has_path $config $extra_path)

    if (not $has_preferred) and (not $has_extra) {
        return null
    }

    if (has_path $config $terminals_path) {
        return (make_result
            "unify_terminal_preference_list"
            "manual_only"
            []
            [
                (format_path $preferred_path)
                (format_path $extra_path)
                (format_path $terminals_path)
            ]
            $config
        )
    }

    let preferred_value = if $has_preferred { maybe_get $config $preferred_path } else { null }
    let extra_value = if $has_extra { maybe_get $config $extra_path } else { [] }

    if ($preferred_value != null) and not ((($preferred_value | describe) | str contains "string") or (($preferred_value | describe) == "string")) {
        return (make_result "unify_terminal_preference_list" "manual_only" [] [(format_path $preferred_path)] $config)
    }

    if not (($extra_value | describe) | str contains "list") {
        return (make_result "unify_terminal_preference_list" "manual_only" [] [(format_path $extra_path)] $config)
    }

    let terminals = (compact_string_list [
        ($preferred_value | default "")
        ...$extra_value
    ])

    if ($terminals | is-empty) {
        return (make_result
            "unify_terminal_preference_list"
            "manual_only"
            []
            [
                (format_path $preferred_path)
                (format_path $extra_path)
            ]
            $config
        )
    }

    let updated_terminal = (
        (($config.terminal? | default {}) | reject preferred_terminal extra_terminals)
        | upsert terminals $terminals
    )

    (make_result
        "unify_terminal_preference_list"
        "auto"
        [$"Replace terminal.preferred_terminal and terminal.extra_terminals with [terminal].terminals = (quoted_list $terminals)."]
        [
            (format_path $preferred_path)
            (format_path $extra_path)
        ]
        ($config | upsert terminal $updated_terminal)
    )
}

def plan_remove_shell_enable_atuin [config: record] {
    let path = ["shell", "enable_atuin"]

    if not (has_path $config $path) {
        return null
    }

    let updated_shell = (($config.shell? | default {}) | reject enable_atuin)

    (make_result "remove_shell_enable_atuin" "auto" ["Remove the obsolete shell.enable_atuin setting."] [(format_path $path)] ($config | upsert shell $updated_shell))
}

def plan_review_legacy_cursor_trail_settings [config: record] {
    let legacy_paths = [
        ["terminal", "cursor_trail"]
        ["terminal", "ghostty_cursor_effects_random"]
        ["terminal", "ghostty_cursor_effects"]
    ]
    let matched = (
        $legacy_paths
        | where {|path| has_path $config $path }
        | each {|path| format_path $path }
    )

    if ($matched | is-empty) {
        return null
    }

    (make_result "review_legacy_cursor_trail_settings" "manual_only" [] $matched $config)
}

def get_plan_step [rule_id: string, config: record] {
    match $rule_id {
        "remove_zellij_widget_tray_layout" => (plan_remove_zellij_widget_tray_layout $config)
        "unify_terminal_preference_list" => (plan_unify_terminal_preference_list $config)
        "remove_shell_enable_atuin" => (plan_remove_shell_enable_atuin $config)
        "review_legacy_cursor_trail_settings" => (plan_review_legacy_cursor_trail_settings $config)
        _ => (error make {msg: $"Unknown config migration rule id: ($rule_id)"})
    }
}

export def get_config_migration_rules [] {
    $CONFIG_MIGRATION_RULES
}

export def validate_config_migration_rules [] {
    let required_fields = [
        "id"
        "title"
        "kind"
        "introduced_on"
        "auto_apply"
        "user_visible"
        "guarded_paths"
        "rationale"
        "manual_fix"
    ]
    let ids = ($CONFIG_MIGRATION_RULES | get id)
    let duplicate_ids = (
        $ids
        | uniq --count
        | where count > 1
        | get -o value
        | default []
    )

    mut errors = []

    if not ($duplicate_ids | is-empty) {
        $errors = ($errors | append ($duplicate_ids | each {|id| $"Duplicate config migration rule id: ($id)" }))
    }

    for rule in $CONFIG_MIGRATION_RULES {
        for field in $required_fields {
            if not ($field in ($rule | columns)) {
                $errors = ($errors | append $"Config migration rule ($rule.id) is missing required field: ($field)")
                continue
            }

            let value = ($rule | get $field)
            if ($field == "guarded_paths") and (($value | describe) | str contains "list") {
                if ($value | is-empty) {
                    $errors = ($errors | append $"Config migration rule ($rule.id) must declare at least one guarded path")
                }
                continue
            }

            if ($field == "auto_apply") or ($field == "user_visible") {
                continue
            }

            if ($value == null) or (($value | into string | str trim) == "") {
                $errors = ($errors | append $"Config migration rule ($rule.id) has empty required field: ($field)")
            }
        }

        if (($rule.auto_apply == true) and ($rule.kind == "manual_only")) or (($rule.auto_apply == false) and ($rule.kind != "manual_only") and ($rule.id != "review_legacy_cursor_trail_settings")) {
            $errors = ($errors | append $"Config migration rule ($rule.id) has inconsistent kind/auto_apply metadata")
        }

        if (($rule.introduced_in | is-empty) and ($rule.introduced_after_version | is-empty)) {
            $errors = ($errors | append $"Config migration rule ($rule.id) must declare introduced_in or introduced_after_version")
        }
    }

    $errors
}

export def build_config_migration_plan_from_record [config: record, config_path: string = "<memory>"] {
    mut current_config = $config
    mut results = []

    for rule in $CONFIG_MIGRATION_RULES {
        let step = (get_plan_step $rule.id $current_config)
        if $step == null {
            continue
        }

        $results = ($results | append $step)

        if $step.status == "auto" {
            $current_config = $step.config_after
        }
    }

    let auto_results = ($results | where status == "auto")
    let manual_results = ($results | where status == "manual_only")

    {
        config_path: $config_path
        original_config: $config
        migrated_config: $current_config
        results: $results
        auto_results: $auto_results
        manual_results: $manual_results
        auto_count: ($auto_results | length)
        manual_count: ($manual_results | length)
        has_auto_changes: (not ($auto_results | is-empty))
        has_manual_items: (not ($manual_results | is-empty))
    }
}

export def get_config_migration_plan [config_path: string] {
    let config = open $config_path
    build_config_migration_plan_from_record $config $config_path
}

export def render_config_migration_plan [plan: record] {
    let lines = [
        "Yazelix config migration preview"
        $"Config: ($plan.config_path)"
        $"Known rule matches: ($plan.results | length)"
        $"Safe rewrites: ($plan.auto_count)"
        $"Manual follow-up items: ($plan.manual_count)"
    ]
    mut rendered = $lines

    if ($plan.results | is-empty) {
        $rendered = ($rendered | append [
            ""
            "No known config migrations detected."
        ])
        return ($rendered | str join "\n")
    }

    for result in $plan.results {
        let prefix = if $result.status == "auto" { "AUTO" } else { "MANUAL" }
        $rendered = ($rendered | append [
            ""
            $"[($prefix)] ($result.id)"
            $"  Title: ($result.title)"
            $"  Introduced: (format_release_context $result)"
            $"  Rationale: ($result.rationale)"
        ])

        if not ($result.matched_paths | is-empty) {
            let joined_paths = ($result.matched_paths | str join ", ")
            $rendered = ($rendered | append [$"  Matched paths: ($joined_paths)"])
        }

        if $result.status == "auto" {
            for change in $result.changes {
                $rendered = ($rendered | append [$"  Change: ($change)"])
            }
        } else {
            $rendered = ($rendered | append [$"  Manual fix: ($result.manual_fix)"])
        }
    }

    $rendered = ($rendered | append [
        ""
        "Preview only. Re-run with `yzx config migrate --apply` to write the safe rewrites."
    ])

    if $plan.has_manual_items {
        $rendered = ($rendered | append [
            "Manual-only items will stay untouched on apply."
        ])
    }

    $rendered | str join "\n"
}

export def apply_config_migration_plan [plan: record] {
    if not $plan.has_auto_changes {
        return {
            status: "noop"
            config_path: $plan.config_path
            backup_path: null
            applied_count: 0
            manual_count: $plan.manual_count
        }
    }

    let timestamp = (date now | format date "%Y%m%d_%H%M%S")
    let backup_path = $"($plan.config_path).backup-($timestamp)"
    let rewritten = ($plan.migrated_config | to toml)

    cp $plan.config_path $backup_path

    try {
        $rewritten | save --force --raw $plan.config_path
        {
            status: "applied"
            config_path: $plan.config_path
            backup_path: $backup_path
            applied_count: $plan.auto_count
            manual_count: $plan.manual_count
        }
    } catch {|err|
        try {
            cp $backup_path $plan.config_path
        }
        error make {msg: $"Failed to apply config migrations: ($err.msg)"}
    }
}
