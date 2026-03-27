#!/usr/bin/env nu
# yzx menu - Interactive command palette and config opener

use ../integrations/zellij.nu [get_current_tab_workspace_root_including_bootstrap open_floating_runtime_wrapper resolve_tab_cwd_target set_tab_workspace_root]
use ../integrations/yazi.nu [sync_active_sidebar_yazi_to_directory sync_managed_editor_cwd]
use ../utils/common.nu [get_yazelix_config_dir get_yazelix_runtime_dir]
use ../utils/config_migrations.nu [apply_config_migration_plan get_config_migration_plan render_config_migration_plan validate_config_migration_rules]
use ../utils/config_parser.nu parse_yazelix_config

def classify_menu_command [cmd: string] {
    if ($cmd | str starts-with "yzx launch") or ($cmd == "yzx restart") {
        {tag: "session", color: (ansi green)}
    } else if ($cmd | str starts-with "yzx config") {
        {tag: "config", color: (ansi cyan)}
    } else if ($cmd | str starts-with "yzx update") or ($cmd | str starts-with "yzx gc") or ($cmd | str starts-with "yzx packs") or ($cmd == "yzx doctor") {
        {tag: "system", color: (ansi yellow)}
    } else if ($cmd == "yzx help") or ($cmd == "yzx why") or ($cmd == "yzx status") or ($cmd == "yzx sponsor") or ($cmd | str starts-with "yzx keys") {
        {tag: "help", color: (ansi blue)}
    } else {
        {tag: "other", color: (ansi purple)}
    }
}

def get_menu_items [] {
    help commands
    | where name =~ '^yzx( |$)'
    | where name != "yzx"
    | where name != "yzx menu"
    | where name != "yzx menu --popup"
    | where not ($it.name | str starts-with "yzx dev")
    | where $it.name != "yzx env"
    | where $it.name != "yzx run"
    | sort-by name
    | each {|row|
        let semantic = classify_menu_command $row.name
        let tag = $"($semantic.color)[($semantic.tag)](ansi reset)"
        let description = ($row.description | default "" | str replace -a "\n" " " | str trim)
        {
            id: $row.name
            label: (if ($description | is-empty) {
                $"($row.name)  ($tag)"
            } else {
                $"($row.name)  ($tag)  (ansi dark_gray)- ($description)(ansi reset)"
            })
        }
    }
}

# In popup mode, pause after most commands so output can be read before closing.
def should_pause_in_popup [cmd: string] {
    not (
        ($cmd | str starts-with "yzx launch")
        or ($cmd | str starts-with "yzx env")
        or ($cmd | str starts-with "yzx restart")
    )
}

def popup_post_action_decision [] {
    print ""
    print "Backspace: return to menu | Enter/Esc: close"
    loop {
        let event = (input listen --types [key])
        let code = ($event.code? | default "")
        if $code == "backspace" {
            clear
            return "menu"
        }
        if ($code == "enter") or ($code == "esc") {
            return "close"
        }
    }
}

def prompt_for_cwd_target [] {
    let target = (input "yzx cwd (path or zoxide query, blank=current dir)> " | str trim)
    if ($target | is-empty) { pwd } else { $target }
}

def run_menu_cwd_action [] {
    if ($env.ZELLIJ? | is-empty) {
        error make {msg: "yzx cwd only works inside Zellij. Start Yazelix first, then run it from the tab you want to update."}
    }

    let resolved_dir = try {
        resolve_tab_cwd_target (prompt_for_cwd_target)
    } catch {|err|
        error make {msg: $err.msg}
    }

    let result = (set_tab_workspace_root $resolved_dir "yzx_menu_cwd.log")

    match $result.status {
        "ok" => {
            let editor_sync_result = (sync_managed_editor_cwd $result.workspace_root "yzx_menu_cwd.log")
            let sidebar_sync_result = (sync_active_sidebar_yazi_to_directory $result.workspace_root "yzx_menu_cwd.log")
            print $"✅ Updated current tab workspace directory to: ($result.workspace_root)"
            print $"   Tab renamed to: ($result.tab_name)"
            print "   Existing panes keep their current working directories."
            print "   New managed actions will use the updated tab directory."
            if $editor_sync_result.status == "ok" {
                print "   Managed editor cwd synced to the updated directory."
            }
            if $sidebar_sync_result.status == "ok" {
                print "   Sidebar Yazi synced to the updated directory."
            }
        }
        "not_ready" => {
            error make {msg: "Yazelix tab state is not ready yet. Wait a moment for the pane orchestrator plugin to finish loading, then try again."}
        }
        "permissions_denied" => {
            error make {msg: "The Yazelix pane orchestrator plugin is missing required Zellij permissions. Reload the Yazelix session and try again."}
        }
        _ => {
            let reason = ($result.reason? | default "unknown error")
            error make {msg: $"Failed to update the current tab workspace directory: ($reason)"}
        }
    }
}

def run_menu_action [cmd: string] {
    if $cmd == "yzx cwd" {
        run_menu_cwd_action
        return
    }

    let yazelix_module = $"($env.HOME)/.config/yazelix/nushell/scripts/core/yazelix.nu"
    ^nu -c $"use ($yazelix_module) *; ($cmd)"
}

# Interactive command palette for Yazelix
export def "yzx menu" [
    --popup  # Open menu in a Zellij floating pane
] {
    if $popup {
        if ($env.ZELLIJ? | is-empty) {
            error make {msg: "Not in a Zellij session; run `yzx menu` directly or start Yazelix/Zellij first."}
        }

        let popup_cwd = ((get_current_tab_workspace_root_including_bootstrap) | default (pwd))
        open_floating_runtime_wrapper "yzx_menu" "yzx_menu_popup.nu" $popup_cwd
        return
    }

    let in_popup = ($env.ZELLIJ_PANE_ID? | is-not-empty) and ($env.YAZELIX_MENU_POPUP? == "true")
    let items = get_menu_items

    if $in_popup {
        loop {
            let selected = ($items | get label | input list --fuzzy "yzx menu \(Esc to cancel\)> ")
            if ($selected | is-empty) {
                return
            }

            let entry = ($items | where label == $selected | first)
            run_menu_action $entry.id

            if (should_pause_in_popup $entry.id) {
                if (popup_post_action_decision) == "menu" {
                    continue
                }
            }

            return
        }
    } else {
        let selected = ($items | get label | input list --fuzzy "yzx menu \(Esc to cancel\)> ")
        if ($selected | is-empty) {
            return
        }
        let entry = ($items | where label == $selected | first)
        run_menu_action $entry.id
    }
}

# Show the active Yazelix configuration
export def "yzx config" [
    --full   # Include the packs section
    --path   # Print the resolved config path
] {
    let config = parse_yazelix_config
    let config_path = $config.config_file

    if $path {
        $config_path
    } else {
        let raw_config = (open $config_path)
        if $full { $raw_config } else { $raw_config | reject packs }
    }
}

def get_primary_config_paths [] {
    let config_dir = (get_yazelix_config_dir)
    let runtime_dir = (get_yazelix_runtime_dir)
    {
        yazelix_dir: $config_dir
        user_config: ($config_dir | path join "yazelix.toml")
        default_config: ($runtime_dir | path join "yazelix_default.toml")
    }
}

def show_config_section [section: string] {
    let yazi_config_path = ("~/.local/share/yazelix/configs/yazi/yazi.toml" | path expand)
    let zellij_config_path = ("~/.local/share/yazelix/configs/zellij/config.kdl" | path expand)
    let helix_config_path = ("~/.config/helix/config.toml" | path expand)
    let helix_languages_path = ("~/.config/helix/languages.toml" | path expand)

    match $section {
        "hx" => {
            {
                config_path: $helix_config_path
                config: (if ($helix_config_path | path exists) { open $helix_config_path } else { null })
                languages_path: $helix_languages_path
                languages: (if ($helix_languages_path | path exists) { open $helix_languages_path } else { null })
            }
        }
        "yazi" => {
            if not ($yazi_config_path | path exists) {
                error make {msg: $"Yazi config not found at ($yazi_config_path). Launch Yazelix once to generate it."}
            }
            open $yazi_config_path
        }
        "zellij" => {
            if not ($zellij_config_path | path exists) {
                error make {msg: $"Zellij config not found at ($zellij_config_path). Launch Yazelix once to generate it."}
            }
            open --raw $zellij_config_path
        }
        _ => (error make {msg: $"Unknown config section: ($section)"})
    }
}

export def "yzx config hx" [] {
    show_config_section "hx"
}

export def "yzx config yazi" [] {
    show_config_section "yazi"
}

export def "yzx config zellij" [] {
    show_config_section "zellij"
}

# Open the active Yazelix configuration file in your editor
export def "yzx config open" [
    --print  # Print the config path without opening
] {
    let config = parse_yazelix_config
    let config_path = $config.config_file

    if $print {
        $config_path
    } else if ($env.EDITOR? | is-empty) {
        error make {msg: $"EDITOR is not set. Set it in yazelix.toml under [editor] command, or export EDITOR in your shell.\nConfig path: ($config_path)"}
    } else {
        ^$env.EDITOR $config_path
    }
}

# Preview or apply known Yazelix config migrations
export def "yzx config migrate" [
    --apply  # Write safe migrations back to yazelix.toml
    --yes    # Skip confirmation prompt when using --apply
] {
    let metadata_errors = (validate_config_migration_rules)
    if not ($metadata_errors | is-empty) {
        let details = ($metadata_errors | each {|line| $" - ($line)" } | str join "\n")
        error make {msg: $"Config migration rules are invalid:\n($details)"}
    }

    let paths = get_primary_config_paths

    if not ($paths.user_config | path exists) {
        error make {msg: $"User config not found: ($paths.user_config)"}
    }

    let plan = (get_config_migration_plan $paths.user_config)
    print (render_config_migration_plan $plan)

    if not $apply {
        return
    }

    if not $plan.has_auto_changes {
        print ""
        print "No safe config rewrites to apply."
        return
    }

    if not $yes {
        print ""
        print "⚠️  This rewrites yazelix.toml from parsed TOML."
        print "   The original file will be backed up first."
        print "   Comments and key ordering may be normalized."
        let confirm = try {
            (input "Apply the safe migrations? [y/N]: " | str downcase)
        } catch { "n" }
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            return
        }
    }

    let apply_result = (apply_config_migration_plan $plan)

    print ""
    print $"✅ Backed up previous config to: ($apply_result.backup_path)"
    print $"✅ Applied ($apply_result.applied_count) config migration\(s\) to: ($apply_result.config_path)"
    print "ℹ️  Comments and key ordering were normalized because Yazelix rewrote the file from parsed TOML."

    if $apply_result.manual_count > 0 {
        print $"ℹ️  ($apply_result.manual_count) manual migration item\(s\) still need follow-up."
    }
}

export def "yzx config reset" [
    --yes  # Skip confirmation prompt
] {
    let paths = get_primary_config_paths

    if not ($paths.default_config | path exists) {
        error make {msg: $"Default config not found: ($paths.default_config)"}
    }

    if not $yes {
        print "⚠️  This replaces yazelix.toml with a fresh copy of yazelix_default.toml."
        if ($paths.user_config | path exists) {
            print "   Your current yazelix.toml will be backed up first."
        }
        let confirm = try {
            (input "Continue? [y/N]: " | str downcase)
        } catch { "n" }
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            return
        }
    }

    let backup_path = if ($paths.user_config | path exists) {
        let timestamp = (date now | format date "%Y%m%d_%H%M%S")
        let path = $"($paths.user_config).backup-($timestamp)"
        mv $paths.user_config $path
        $path
    } else {
        null
    }

    cp $paths.default_config $paths.user_config

    if ($backup_path | is-not-empty) {
        print $"✅ Backed up previous config to: ($backup_path)"
    }
    print $"✅ Replaced yazelix.toml with a fresh template: ($paths.user_config)"
}
