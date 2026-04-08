#!/usr/bin/env nu

use config_parser.nu parse_yazelix_config
use ../setup/helix_config_merger.nu [build_managed_helix_config get_generated_helix_config_path get_managed_helix_user_config_path get_managed_reveal_command get_native_helix_config_path]

def is_helix_editor_command [editor: string] {
    let normalized = ($editor | str trim)
    ($normalized | is-empty) or ($normalized | str ends-with "/hx") or ($normalized == "hx") or ($normalized | str ends-with "/helix") or ($normalized == "helix")
}

def managed_helix_config_has_reveal_binding [config: record] {
    let normal_keys = ($config.keys? | default {} | get -o normal | default {})
    (($normal_keys | get -o "A-r" | default "") == (get_managed_reveal_command))
}

export def check_helix_runtime_conflicts [] {
    mut conflicts = []
    mut has_high_priority_conflict = false

    let user_runtime = "~/.config/helix/runtime" | path expand
    if ($user_runtime | path exists) {
        $conflicts = ($conflicts | append {
            path: $user_runtime
            priority: 2
            name: "User config runtime"
            severity: "error"
        })
        $has_high_priority_conflict = true
    }

    let helix_exe = try { (which hx | get path.0) } catch { null }
    let effective_runtime = (detect_effective_helix_runtime)
    if ($helix_exe | is-not-empty) {
        let exe_runtime = ($helix_exe | path dirname | path join "runtime")
        if ($exe_runtime | path exists) and ($exe_runtime != ($effective_runtime | default "")) {
            $conflicts = ($conflicts | append {
                path: $exe_runtime
                priority: 5
                name: "Executable sibling runtime"
                severity: "warning"
            })
        }
    }

    if ($conflicts | is-empty) {
        return {
            status: "ok"
            message: "No conflicting Helix runtime directories found"
            details: "Helix runtime search order will behave as intended"
            fix_available: false
            conflicts: []
        }
    }

    let status = if $has_high_priority_conflict { "error" } else { "warning" }
    let conflict_details = ($conflicts | each { |c|
        $"($c.name): ($c.path) \(priority ($c.priority)\)"
    } | str join ", ")
    let message = if $has_high_priority_conflict {
        "HIGH PRIORITY: ~/.config/helix/runtime will override the intended Helix runtime"
    } else {
        "Lower priority runtime directories found"
    }
    let fix_commands = if $has_high_priority_conflict {
        [
            $"# Backup and remove conflicting runtime:"
            $"mv ($user_runtime) ($user_runtime).backup"
            $"# Or if you want to delete it:"
            $"rm -rf ($user_runtime)"
        ]
    } else { [] }

    {
        status: $status
        message: $message
        details: $"Conflicting runtimes: ($conflict_details). Helix searches in priority order and will use files from higher priority directories, potentially breaking syntax highlighting."
        fix_available: true
        fix_commands: $fix_commands
        conflicts: $conflicts
    }
}

def detect_all_helix_runtimes [] {
    if (which hx | is-empty) {
        return []
    }

    try {
        let runtime_line = (
            hx --health
            | lines
            | where {|line| $line | str starts-with "Runtime directories:"}
            | first
        )
        let runtime_candidates = (
            $runtime_line
            | str replace "Runtime directories: " ""
            | split row ";"
            | each {|entry| $entry | str trim}
            | where {|entry| $entry != ""}
        )

        $runtime_candidates | where {|candidate| $candidate | path exists}
    } catch {
        []
    }
}

def detect_effective_helix_runtime [] {
    let all_runtimes = (detect_all_helix_runtimes)
    if ($all_runtimes | is-empty) {
        null
    } else {
        $all_runtimes | first
    }
}

export def check_helix_runtime_health [] {
    let all_runtimes = (detect_all_helix_runtimes)
    let primary_runtime = (detect_effective_helix_runtime)

    if ($primary_runtime | is-empty) {
        return {
            status: "error"
            message: "Helix runtime could not be resolved"
            details: "Helix did not report any valid runtime directory in `hx --health`"
            fix_available: false
        }
    }

    let required_dirs = ["grammars", "queries", "themes"]
    let missing_dirs = ($required_dirs | where {|required_dir|
        let found_in_any = ($all_runtimes | any {|runtime_path|
            $"($runtime_path)/($required_dir)" | path exists
        })
        not $found_in_any
    })

    if not ($missing_dirs | is-empty) {
        return {
            status: "error"
            message: $"Missing required directories: ($missing_dirs | str join ', ')"
            details: $"The effective Helix runtime at ($primary_runtime) is incomplete (note: Nix may split runtime across multiple paths)"
            fix_available: false
        }
    }

    let grammar_count = ($all_runtimes | each {|runtime_path|
        try {
            (ls $"($runtime_path)/grammars" | length)
        } catch {
            0
        }
    } | math sum)

    if ($grammar_count < 200) {
        return {
            status: "warning"
            message: $"Only ($grammar_count) grammar files found (expected 200+)"
            details: "Some languages may not have syntax highlighting"
            fix_available: false
        }
    }

    let tutor_exists = ($all_runtimes | any {|runtime_path|
        $"($runtime_path)/tutor" | path exists
    })

    if not $tutor_exists {
        return {
            status: "warning"
            message: "Helix tutor file missing"
            details: "Tutorial will not be available"
            fix_available: false
        }
    }

    {
        status: "ok"
        message: $"Helix runtime healthy with ($grammar_count) grammars"
        details: $"Primary runtime directory: ($primary_runtime)"
        fix_available: false
    }
}

export def check_managed_helix_integration [] {
    let config = (try {
        parse_yazelix_config
    } catch {
        return []
    })

    let configured_editor = ($config.editor_command? | default "" | into string | str trim)
    if not (is_helix_editor_command $configured_editor) {
        return []
    }

    mut results = []

    let managed_user_config = (get_managed_helix_user_config_path)
    let native_helix_config = (get_native_helix_config_path)
    if (not ($managed_user_config | path exists)) and ($native_helix_config | path exists) {
        $results = ($results | append {
            status: "info"
            message: "Personal Helix config has not been imported into Yazelix-managed Helix"
            details: $"Native config: ($native_helix_config)\nManaged config: ($managed_user_config)\nRun `yzx import helix` if you want Yazelix-managed Helix sessions to reuse that personal config."
            fix_available: false
        })
    }

    let expected_config_result = (try {
        {
            config: (build_managed_helix_config)
            error: null
        }
    } catch {|err|
        {
            config: null
            error: $err.msg
        }
    })
    if ($expected_config_result.error | is-not-empty) {
        return ($results | append {
            status: "error"
            message: "Managed Helix config contract could not be built"
            details: $expected_config_result.error
            fix_available: false
        })
    }
    let expected_config = $expected_config_result.config

    if not (managed_helix_config_has_reveal_binding $expected_config) {
        return ($results | append {
            status: "error"
            message: "Managed Helix config contract lost the Yazelix reveal binding"
            details: "The expected managed Helix config no longer enforces `A-r = :sh yzx reveal \"%{buffer_name}\"`."
            fix_available: false
        })
    }

    let generated_config_path = (get_generated_helix_config_path)
    if not ($generated_config_path | path exists) {
        return ($results | append {
            status: "info"
            message: "Managed Helix config has not been materialized yet"
            details: $"Expected generated config: ($generated_config_path)\nThis is normal before the first managed Helix launch. Yazelix will generate it on demand."
            fix_available: false
        })
    }

    let generated_config_result = (try {
        {
            config: (open $generated_config_path)
            error: null
        }
    } catch {|err|
        {
            config: null
            error: $err.msg
        }
    })
    if ($generated_config_result.error | is-not-empty) {
        return ($results | append {
            status: "warning"
            message: "Managed Helix generated config could not be read"
            details: $"Generated config: ($generated_config_path)\nUnderlying error: ($generated_config_result.error)"
            fix_available: false
        })
    }
    let generated_config = $generated_config_result.config

    if not (managed_helix_config_has_reveal_binding $generated_config) {
        return ($results | append {
            status: "warning"
            message: "Managed Helix generated config is stale or invalid"
            details: $"Generated config: ($generated_config_path)\nExpected `A-r` to run `yzx reveal`.\nLaunch a managed Helix session again to regenerate it."
            fix_available: false
        })
    }

    $results | append {
        status: "ok"
        message: "Managed Helix reveal integration is healthy"
        details: $"Generated config: ($generated_config_path)"
        fix_available: false
    }
}

export def fix_helix_runtime_conflicts [conflicts: list] {
    mut success = true

    for $conflict in $conflicts {
        if $conflict.severity == "error" {
            let backup_path = $"($conflict.path).backup"

            let move_result = try {
                mv $conflict.path $backup_path
                print $"✅ Moved ($conflict.name) from ($conflict.path) to ($backup_path)"
                true
            } catch {
                print $"❌ Failed to move ($conflict.name) from ($conflict.path)"
                false
            }

            if not $move_result {
                $success = false
            }
        }
    }

    $success
}
