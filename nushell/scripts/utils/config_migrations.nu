#!/usr/bin/env nu
# Shared migration registry and preview/apply engine for Yazelix config migrations

use config_migration_transactions.nu apply_managed_config_transaction

const CONFIG_MIGRATION_RULES = [
    {
        id: "remove_zellij_widget_tray_layout"
        title: "Remove the broken layout widget from zellij.widget_tray"
        kind: "removed_value"
        introduced_in: null
        introduced_after_version: "v13.7"
        introduced_on: "2026-03-27"
        review_after_days: 180
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
        review_after_days: 180
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
        review_after_days: 180
        auto_apply: true
        user_visible: true
        guarded_paths: ["shell.enable_atuin"]
        rationale: "Yazelix no longer uses a shell-level enable_atuin toggle and now relies on direct tool availability."
        manual_fix: "Remove shell.enable_atuin from [shell]."
    }
    {
        id: "move_legacy_helix_command_to_editor_command"
        title: "Move legacy helix.command to editor.command"
        kind: "field_reshape"
        introduced_in: null
        introduced_after_version: "v13.10"
        introduced_on: "2026-03-30"
        review_after_days: 180
        auto_apply: true
        user_visible: true
        guarded_paths: ["helix.command", "editor.command"]
        rationale: "Yazelix now owns editor selection under [editor].command, while [helix] only keeps Helix-specific settings like mode and runtime_path."
        manual_fix: "Move helix.command into [editor].command. If both fields exist with different values, keep the one you intend to use and remove the other manually."
    }
    {
        id: "review_legacy_cursor_trail_settings"
        title: "Review legacy Ghostty cursor-trail settings manually"
        kind: "manual_only"
        introduced_in: "v13.2"
        introduced_after_version: null
        introduced_on: "2026-03-14"
        review_after_days: 365
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
    {
        id: "review_terminal_config_mode_auto"
        title: "Review the removed terminal.config_mode = auto setting manually"
        kind: "manual_only"
        introduced_in: "v13.8"
        introduced_after_version: null
        introduced_on: "2026-03-28"
        review_after_days: 365
        auto_apply: false
        user_visible: true
        guarded_paths: ["terminal.config_mode"]
        rationale: "Yazelix no longer has an ambient fallback mode for terminal configs. Users must now choose either Yazelix-managed configs or their terminal's real native config."
        manual_fix: "Replace terminal.config_mode = \"auto\" with either \"yazelix\" or \"user\" after deciding which config owner you want."
    }
    {
        id: "replace_ascii_art_mode_with_welcome_style"
        title: "Replace legacy [ascii].mode with core.welcome_style"
        kind: "field_reshape"
        introduced_in: "v13.8"
        introduced_after_version: null
        introduced_on: "2026-03-29"
        review_after_days: 180
        auto_apply: true
        user_visible: true
        guarded_paths: ["ascii", "ascii.mode", "core.welcome_style"]
        rationale: "Yazelix now uses one welcome_style selector instead of a separate ASCII-art mode field. Legacy ascii.mode values now collapse into the random welcome-style pool."
        manual_fix: "Replace [ascii].mode with [core].welcome_style = \"random\"."
    }
    {
        id: "rename_life_welcome_style_to_game_of_life"
        title: "Rename core.welcome_style = life to game_of_life"
        kind: "field_reshape"
        introduced_in: "v13.8"
        introduced_after_version: null
        introduced_on: "2026-03-29"
        review_after_days: 180
        auto_apply: true
        user_visible: true
        guarded_paths: ["core.welcome_style"]
        rationale: "Yazelix now uses the clearer game_of_life welcome-style name instead of the shorter life alias."
        manual_fix: "Replace core.welcome_style = \"life\" with \"game_of_life\"."
    }
    {
        id: "split_legacy_pack_config_surface"
        title: "Move legacy [packs] out of yazelix.toml into yazelix_packs.toml"
        kind: "field_reshape"
        introduced_in: "v13.8"
        introduced_after_version: null
        introduced_on: "2026-03-28"
        review_after_days: 180
        auto_apply: true
        user_visible: true
        guarded_paths: ["packs"]
        rationale: "Yazelix now keeps pack declarations in a dedicated yazelix_packs.toml sidecar so the main config stays focused and pack ownership is unambiguous."
        manual_fix: "Move [packs] out of yazelix.toml into yazelix_packs.toml, then re-run with only the sidecar owning pack settings."
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

def plan_split_legacy_pack_config_surface [config: record, pack_config: any, pack_config_path: string] {
    let path = ["packs"]
    let packs = (maybe_get $config $path)

    if $packs == null {
        return null
    }

    if not (($packs | describe) | str contains "record") {
        return (make_result "split_legacy_pack_config_surface" "manual_only" [] [(format_path $path)] $config)
    }

    if $pack_config != null {
        return (make_result
            "split_legacy_pack_config_surface"
            "manual_only"
            []
            [
                (format_path $path)
                $pack_config_path
            ]
            $config
        )
    }

    let updated_config = ($config | reject packs)

    (
        make_result
            "split_legacy_pack_config_surface"
            "auto"
            [
                "Move [packs] out of yazelix.toml into yazelix_packs.toml."
            ]
            [(format_path $path)]
            $updated_config
        | upsert pack_config_after $packs
    )
}

def get_plan_step [rule_id: string, config: record, pack_config: any = null, pack_config_path: string = "yazelix_packs.toml"] {
    match $rule_id {
        "remove_zellij_widget_tray_layout" => (plan_remove_zellij_widget_tray_layout $config)
        "unify_terminal_preference_list" => (plan_unify_terminal_preference_list $config)
        "remove_shell_enable_atuin" => (plan_remove_shell_enable_atuin $config)
        "move_legacy_helix_command_to_editor_command" => (plan_move_legacy_helix_command_to_editor_command $config)
        "review_legacy_cursor_trail_settings" => (plan_review_legacy_cursor_trail_settings $config)
        "review_terminal_config_mode_auto" => (plan_review_terminal_config_mode_auto $config)
        "replace_ascii_art_mode_with_welcome_style" => (plan_replace_ascii_art_mode_with_welcome_style $config)
        "rename_life_welcome_style_to_game_of_life" => (plan_rename_life_welcome_style_to_game_of_life $config)
        "split_legacy_pack_config_surface" => (plan_split_legacy_pack_config_surface $config $pack_config $pack_config_path)
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
        "review_after_days"
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

            let value = ($rule | get -o $field)
            if ($field == "guarded_paths") and (($value | describe) | str contains "list") {
                if ($value | is-empty) {
                    $errors = ($errors | append $"Config migration rule ($rule.id) must declare at least one guarded path")
                }
                continue
            }

            if $field == "review_after_days" {
                let parsed = (try { $value | into int } catch { null })
                if ($parsed == null) or ($parsed < 1) {
                    $errors = ($errors | append $"Config migration rule ($rule.id) must declare a positive review_after_days value")
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

        if (($rule.auto_apply and ($rule.kind == "manual_only")) or ((not $rule.auto_apply) and ($rule.kind != "manual_only") and ($rule.id != "review_legacy_cursor_trail_settings"))) {
            $errors = ($errors | append $"Config migration rule ($rule.id) has inconsistent kind/auto_apply metadata")
        }

        if (($rule.introduced_in | is-empty) and ($rule.introduced_after_version | is-empty)) {
            $errors = ($errors | append $"Config migration rule ($rule.id) must declare introduced_in or introduced_after_version")
        }
    }

    $errors
}

export def build_config_migration_plan_from_record [config: record, config_path: string = "<memory>", pack_config: any = null, pack_config_path: string = "yazelix_packs.toml"] {
    mut current_config = $config
    mut current_pack_config = $pack_config
    mut results = []

    for rule in $CONFIG_MIGRATION_RULES {
        let step = (get_plan_step $rule.id $current_config $current_pack_config $pack_config_path)
        if $step == null {
            continue
        }

        $results = ($results | append $step)

        if $step.status == "auto" {
            $current_config = $step.config_after
            if ("pack_config_after" in ($step | columns)) {
                $current_pack_config = $step.pack_config_after
            }
        }
    }

    let auto_results = ($results | where status == "auto")
    let manual_results = ($results | where status == "manual_only")

    {
        config_path: $config_path
        pack_config_path: $pack_config_path
        original_config: $config
        migrated_config: $current_config
        original_pack_config: $pack_config
        migrated_pack_config: $current_pack_config
        results: $results
        auto_results: $auto_results
        manual_results: $manual_results
        auto_count: ($auto_results | length)
        manual_count: ($manual_results | length)
        has_auto_changes: (not ($auto_results | is-empty))
        has_manual_items: (not ($manual_results | is-empty))
        has_pack_config_change: ($current_pack_config != $pack_config)
    }
}

export def get_config_migration_plan [config_path: string] {
    let config = open $config_path
    use config_surfaces.nu get_pack_sidecar_path

    let pack_config_path = (get_pack_sidecar_path $config_path)
    let pack_config = if ($pack_config_path | path exists) {
        open $pack_config_path
    } else {
        null
    }

    build_config_migration_plan_from_record $config $config_path $pack_config $pack_config_path
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

export def apply_config_migration_plan [plan: record, caller: string] {
    if not $plan.has_auto_changes {
        return {
            status: "noop"
            config_path: $plan.config_path
            backup_path: null
            pack_config_path: ($plan.pack_config_path? | default null)
            pack_backup_path: null
            applied_count: 0
            manual_count: $plan.manual_count
        }
    }

    let pack_config_path = ($plan.pack_config_path? | default null)
    let rewritten = ($plan.migrated_config | to toml)
    let pack_rewritten = if $plan.has_pack_config_change {
        $plan.migrated_pack_config | to toml
    } else {
        null
    }
    let transaction_result = (
        apply_managed_config_transaction
            $caller
            $plan.config_path
            $rewritten
            $pack_config_path
            $pack_rewritten
    )

    $transaction_result | merge {
        applied_count: $plan.auto_count
        manual_count: $plan.manual_count
    }
}
