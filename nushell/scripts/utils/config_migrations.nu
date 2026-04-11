#!/usr/bin/env nu
# Shared migration planning/apply engine for Yazelix config migrations

use config_migration_transactions.nu apply_managed_config_transaction
use config_migration_rules.nu get_config_migration_rules

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

        $current = ($current | get -o $segment)
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

        $current = ($current | get -o $segment)
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

def rewrite_exact_string [data: any, old: string, new: string, path: list<string> = []] {
    let description = ($data | describe)

    if ($description | str contains "record") {
        mut updated = $data
        mut matched_paths = []

        for key in ($data | columns) {
            let child = ($data | get -o $key)
            let result = (rewrite_exact_string $child $old $new ($path | append $key))
            $updated = ($updated | upsert $key $result.value)
            $matched_paths = ($matched_paths | append $result.matched_paths)
        }

        return {
            value: $updated
            matched_paths: $matched_paths
        }
    }

    if ($description | str contains "list") {
        mut updated = []
        mut matched_paths = []
        mut index = 0

        for item in $data {
            let result = (rewrite_exact_string $item $old $new ($path | append ($index | into string)))
            $updated = ($updated | append $result.value)
            $matched_paths = ($matched_paths | append $result.matched_paths)
            $index = ($index + 1)
        }

        return {
            value: $updated
            matched_paths: $matched_paths
        }
    }

    let normalized = (try { $data | into string } catch { null })
    if $normalized == $old {
        return {
            value: $new
            matched_paths: [(format_path $path)]
        }
    }

    {
        value: $data
        matched_paths: []
    }
}

def quoted_list [values: list<string>] {
    if ($values | is-empty) {
        "[]"
    } else {
        let rendered = ($values | each {|value| $"\"($value)\"" } | str join ", ")
        $"[($rendered)]"
    }
}

def get_rule [id: string] {
    get_config_migration_rules | where id == $id | first
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

def plan_move_legacy_helix_command_to_editor_command [config: record] {
    let legacy_path = ["helix", "command"]
    let editor_path = ["editor", "command"]
    let legacy_value = (maybe_get $config $legacy_path)

    if $legacy_value == null {
        return null
    }

    if not (($legacy_value | describe) | str contains "string") {
        return (make_result "move_legacy_helix_command_to_editor_command" "manual_only" [] [(format_path $legacy_path)] $config)
    }

    let editor_value = (maybe_get $config $editor_path)
    if $editor_value != null {
        if not (($editor_value | describe) | str contains "string") {
            return (make_result "move_legacy_helix_command_to_editor_command" "manual_only" [] [(format_path $editor_path)] $config)
        }

        if $editor_value != $legacy_value {
            return (make_result
                "move_legacy_helix_command_to_editor_command"
                "manual_only"
                []
                [
                    (format_path $legacy_path)
                    (format_path $editor_path)
                ]
                $config
            )
        }
    }

    let updated_helix = (($config.helix? | default {}) | reject command)
    let updated_editor = (($config.editor? | default {}) | upsert command $legacy_value)
    let updated_config = ($config | upsert helix $updated_helix | upsert editor $updated_editor)
    let changes = if $editor_value == null {
        [$"Move [helix].command = \"($legacy_value)\" into [editor].command."]
    } else {
        ["Remove duplicated legacy [helix].command because [editor].command already owns editor selection."]
    }

    (make_result
        "move_legacy_helix_command_to_editor_command"
        "auto"
        $changes
        [(format_path $legacy_path)]
        $updated_config
    )
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

def plan_review_terminal_config_mode_auto [config: record] {
    let path = ["terminal", "config_mode"]
    let value = (maybe_get $config $path)

    if $value == null {
        return null
    }

    let normalized = ($value | into string | str downcase)
    if $normalized != "auto" {
        return null
    }

    (make_result "review_terminal_config_mode_auto" "manual_only" [] [(format_path $path)] $config)
}

def plan_replace_ascii_art_mode_with_welcome_style [config: record] {
    let legacy_root_path = ["ascii"]
    let old_path = ["ascii", "mode"]
    let new_path = ["core", "welcome_style"]
    let legacy_value = (maybe_get $config $old_path)

    if $legacy_value == null {
        return null
    }

    if (has_path $config $new_path) {
        return (make_result
            "replace_ascii_art_mode_with_welcome_style"
            "manual_only"
            []
            [
                (format_path $legacy_root_path)
                (format_path $old_path)
                (format_path $new_path)
            ]
            $config
        )
    }

    let normalized = (try { $legacy_value | into string | str downcase } catch { null })
    let mapped = match $normalized {
        "static" => "random"
        "animated" => "random"
        _ => null
    }

    if $mapped == null {
        return (make_result
            "replace_ascii_art_mode_with_welcome_style"
            "manual_only"
            []
            [
                (format_path $legacy_root_path)
                (format_path $old_path)
            ]
            $config
        )
    }

    let updated_core = (($config.core? | default {}) | upsert welcome_style $mapped)
    let updated_config = (($config | reject ascii) | upsert core $updated_core)

    (make_result
        "replace_ascii_art_mode_with_welcome_style"
        "auto"
        [$"Replace [ascii].mode = \"($normalized)\" with [core].welcome_style = \"($mapped)\"."]
        [
            (format_path $legacy_root_path)
            (format_path $old_path)
        ]
        $updated_config
    )
}

def plan_rename_life_welcome_style_to_game_of_life [config: record] {
    let path = ["core", "welcome_style"]
    let value = (maybe_get $config $path)

    if $value == null {
        return null
    }

    let normalized = (try { $value | into string | str downcase } catch { null })
    if $normalized != "life" {
        return null
    }

    let updated_core = (($config.core? | default {}) | upsert welcome_style "game_of_life")
    let updated_config = ($config | upsert core $updated_core)

    (make_result
        "rename_life_welcome_style_to_game_of_life"
        "auto"
        ['Replace [core].welcome_style = "life" with "game_of_life".']
        [(format_path $path)]
        $updated_config
    )
}

def get_plan_step [rule_id: string, config: record] {
    match $rule_id {
        "remove_zellij_widget_tray_layout" => (plan_remove_zellij_widget_tray_layout $config)
        "unify_terminal_preference_list" => (plan_unify_terminal_preference_list $config)
        "remove_shell_enable_atuin" => (plan_remove_shell_enable_atuin $config)
        "move_legacy_helix_command_to_editor_command" => (plan_move_legacy_helix_command_to_editor_command $config)
        "review_legacy_cursor_trail_settings" => (plan_review_legacy_cursor_trail_settings $config)
        "review_terminal_config_mode_auto" => (plan_review_terminal_config_mode_auto $config)
        "replace_ascii_art_mode_with_welcome_style" => (plan_replace_ascii_art_mode_with_welcome_style $config)
        "rename_life_welcome_style_to_game_of_life" => (plan_rename_life_welcome_style_to_game_of_life $config)
        _ => (error make {msg: $"Unknown config migration rule id: ($rule_id)"})
    }
}

export def build_config_migration_plan_from_record [config: record, config_path: string = "<memory>"] {
    mut current_config = $config
    mut results = []

    for rule in (get_config_migration_rules) {
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

export def apply_config_migration_plan [plan: record, caller: string] {
    if not $plan.has_auto_changes {
        return {
            status: "noop"
            config_path: $plan.config_path
            backup_path: null
            applied_count: 0
            manual_count: $plan.manual_count
        }
    }

    let rewritten = ($plan.migrated_config | to toml)
    let transaction_result = (
        apply_managed_config_transaction
            $caller
            $plan.config_path
            $rewritten
    )

    $transaction_result | merge {
        applied_count: $plan.auto_count
        manual_count: $plan.manual_count
    }
}
