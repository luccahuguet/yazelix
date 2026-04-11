#!/usr/bin/env nu

const CONFIG_MIGRATION_RETIREMENT_POLICIES = [
    "demote_to_explicit_then_delete"
    "review_then_delete_or_keep"
]

const CONFIG_MIGRATION_RULES = [
    {
        id: "remove_zellij_widget_tray_layout"
        title: "Remove the broken layout widget from zellij.widget_tray"
        kind: "removed_value"
        introduced_in: null
        introduced_after_version: "v13.7"
        introduced_on: "2026-03-27"
        review_after_days: 180
        last_reviewed_on: null
        retirement_policy: "demote_to_explicit_then_delete"
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
        last_reviewed_on: null
        retirement_policy: "demote_to_explicit_then_delete"
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
        last_reviewed_on: null
        retirement_policy: "demote_to_explicit_then_delete"
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
        last_reviewed_on: null
        retirement_policy: "demote_to_explicit_then_delete"
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
        last_reviewed_on: null
        retirement_policy: "review_then_delete_or_keep"
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
        last_reviewed_on: null
        retirement_policy: "review_then_delete_or_keep"
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
        last_reviewed_on: null
        retirement_policy: "demote_to_explicit_then_delete"
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
        last_reviewed_on: null
        retirement_policy: "demote_to_explicit_then_delete"
        auto_apply: true
        user_visible: true
        guarded_paths: ["core.welcome_style"]
        rationale: "Yazelix now uses the clearer game_of_life welcome-style name instead of the shorter life alias."
        manual_fix: "Replace core.welcome_style = \"life\" with \"game_of_life\"."
    }
]

def parse_rule_date [value: any] {
    let normalized = ($value | default "" | into string | str trim)
    if ($normalized | is-empty) {
        return null
    }

    try { $normalized | into datetime } catch { null }
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
        "last_reviewed_on"
        "retirement_policy"
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

            if $field == "last_reviewed_on" {
                if $value == null {
                    continue
                }

                let normalized = ($value | into string | str trim)
                if ($normalized | is-empty) {
                    continue
                }

                let parsed = (try { $normalized | into datetime } catch { null })
                if $parsed == null {
                    $errors = ($errors | append $"Config migration rule ($rule.id) has invalid last_reviewed_on date: ($normalized)")
                }
                continue
            }

            if $field == "retirement_policy" {
                let normalized = ($value | default "" | into string | str trim)
                if not ($normalized in $CONFIG_MIGRATION_RETIREMENT_POLICIES) {
                    $errors = ($errors | append $"Config migration rule ($rule.id) has invalid retirement_policy: ($normalized)")
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

        if $rule.auto_apply and ($rule.retirement_policy != "demote_to_explicit_then_delete") {
            $errors = ($errors | append $"Config migration rule ($rule.id) must use retirement_policy = demote_to_explicit_then_delete because it still auto-applies")
        }

        if (not $rule.auto_apply) and ($rule.retirement_policy != "review_then_delete_or_keep") {
            $errors = ($errors | append $"Config migration rule ($rule.id) must use retirement_policy = review_then_delete_or_keep because it is manual-only")
        }

        if (($rule.introduced_in | is-empty) and ($rule.introduced_after_version | is-empty)) {
            $errors = ($errors | append $"Config migration rule ($rule.id) must declare introduced_in or introduced_after_version")
        }

        let introduced_on = (try {
            parse_rule_date $rule.introduced_on
        } catch { null })
        let last_reviewed_on = (try {
            parse_rule_date $rule.last_reviewed_on
        } catch { null })

        if (($rule.introduced_on | default "" | into string | str trim | is-not-empty) and ($introduced_on == null)) {
            $errors = ($errors | append $"Config migration rule ($rule.id) has invalid introduced_on date: ($rule.introduced_on)")
        }

        let normalized_last_reviewed_on = ($rule.last_reviewed_on | default "" | into string | str trim)
        if (($normalized_last_reviewed_on | is-not-empty) and ($last_reviewed_on == null)) {
            $errors = ($errors | append $"Config migration rule ($rule.id) has invalid last_reviewed_on date: ($normalized_last_reviewed_on)")
        }

        if ($introduced_on != null) and ($last_reviewed_on != null) and ($last_reviewed_on < $introduced_on) {
            $errors = ($errors | append $"Config migration rule ($rule.id) cannot have last_reviewed_on earlier than introduced_on")
        }

        if $introduced_on != null {
            let review_anchor = if $last_reviewed_on != null { $last_reviewed_on } else { $introduced_on }
            let elapsed_days = ((((date now) - $review_anchor) / 1day) | into int)
            let review_after_days = ($rule.review_after_days | into int)
            if $elapsed_days >= $review_after_days {
                let overdue_days = ($elapsed_days - $review_after_days)
                let anchor_label = if $last_reviewed_on != null { "last_reviewed_on" } else { "introduced_on" }
                $errors = ($errors | append $"Config migration rule ($rule.id) is overdue for retirement review by ($overdue_days) day\(s\) \(anchor: ($anchor_label)=($review_anchor | format date '%Y-%m-%d'); policy: ($rule.retirement_policy)\)")
            }
        }
    }

    $errors
}
