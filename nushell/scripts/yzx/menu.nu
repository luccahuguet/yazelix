#!/usr/bin/env nu
# yzx menu - Interactive command palette and config opener

use ../integrations/zellij.nu [get_current_tab_workspace_root_including_bootstrap open_floating_runtime_wrapper resolve_tab_cwd_target set_tab_workspace_root]
use ../integrations/yazi.nu [sync_active_sidebar_yazi_to_directory sync_managed_editor_cwd]
use ../utils/common.nu [get_yazelix_config_dir get_yazelix_runtime_dir get_yazelix_user_config_dir]
use ../utils/config_migrations.nu [apply_config_migration_plan get_config_migration_plan render_config_migration_plan validate_config_migration_rules]
use ../utils/config_migration_transactions.nu [recover_stale_managed_config_transactions]
use ../utils/editor_launch_context.nu [resolve_editor_launch_context]
use ../utils/config_surfaces.nu [resolve_active_config_paths get_primary_config_paths reconcile_primary_config_surfaces]
use ../setup/helix_config_merger.nu [get_generated_helix_config_path get_managed_helix_user_config_path]

def classify_menu_command [cmd: string] {
    if ($cmd | str starts-with "yzx launch") or ($cmd | str starts-with "yzx enter") or ($cmd == "yzx restart") {
        {tag: "session", color: (ansi green)}
    } else if (
        ($cmd | str starts-with "yzx config")
        or ($cmd | str starts-with "yzx import")
        or ($cmd | str starts-with "yzx open")
        or ($cmd | str starts-with "yzx edit")
    ) {
        {tag: "config", color: (ansi cyan)}
    } else if ($cmd | str starts-with "yzx update") or ($cmd | str starts-with "yzx gc") or ($cmd | str starts-with "yzx packs") or ($cmd == "yzx doctor") {
        {tag: "system", color: (ansi yellow)}
    } else if ($cmd == "yzx help") or ($cmd == "yzx why") or ($cmd == "yzx status") or ($cmd == "yzx sponsor") or ($cmd == "yzx whats_new") or ($cmd | str starts-with "yzx keys") or ($cmd | str starts-with "yzx tutor") {
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
        or ($cmd | str starts-with "yzx enter")
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

    let yazelix_module = ((get_yazelix_runtime_dir) | path join "nushell" "scripts" "core" "yazelix.nu")
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
    use ../utils/config_surfaces.nu [load_active_config_surface]
    let config_surface = (load_active_config_surface)
    let config_path = $config_surface.config_file

    if $path {
        $config_path
    } else {
        let active_config = $config_surface.merged_config
        if $full { $active_config } else { $active_config | reject packs }
    }
}

def show_config_section [section: string] {
    let yazi_config_path = ("~/.local/share/yazelix/configs/yazi/yazi.toml" | path expand)
    let zellij_config_path = ("~/.local/share/yazelix/configs/zellij/config.kdl" | path expand)
    let helix_config_path = (get_managed_helix_user_config_path)
    let generated_helix_config_path = (get_generated_helix_config_path)

    match $section {
        "hx" => {
            {
                config_path: $helix_config_path
                config: (if ($helix_config_path | path exists) { open $helix_config_path } else { null })
                generated_config_path: $generated_helix_config_path
                generated_config: (if ($generated_helix_config_path | path exists) { open $generated_helix_config_path } else { null })
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

export def "yzx open hx" [] {
    show_config_section "hx"
}

export def "yzx open yazi" [] {
    show_config_section "yazi"
}

export def "yzx open zellij" [] {
    show_config_section "zellij"
}

def open_config_surface_in_editor [config_path: string, --print] {
    if $print {
        $config_path
    } else {
        let editor_context = (resolve_editor_launch_context)
        mkdir ($config_path | path dirname)
        clear
        if ($editor_context.launch_env | columns | is-empty) {
            exec $editor_context.editor $config_path
        } else {
            with-env $editor_context.launch_env {
                exec $editor_context.editor $config_path
            }
        }
    }
}

def get_edit_targets [] {
    let paths = get_primary_config_paths
    let user_root = (get_yazelix_user_config_dir)
    let helix_path = (get_managed_helix_user_config_path)
    let zellij_path = ($user_root | path join "zellij" "config.kdl")
    let yazi_toml_path = ($user_root | path join "yazi" "yazi.toml")

    [
        {
            id: "config"
            label: $"config  (ansi dark_gray)- main Yazelix config → ($paths.user_config)(ansi reset)"
            aliases: ["config", "main", "yazelix.toml"]
            search: "config main yazelix yazelix.toml"
            path: $paths.user_config
        }
        {
            id: "packs"
            label: $"packs  (ansi dark_gray)- pack declarations → ($paths.user_pack_config)(ansi reset)"
            aliases: ["packs", "pack", "yazelix_packs.toml"]
            search: "packs pack declarations yazelix_packs.toml"
            path: $paths.user_pack_config
        }
        {
            id: "helix"
            label: $"helix  (ansi dark_gray)- managed Helix user config → ($helix_path)(ansi reset)"
            aliases: ["helix", "hx", "editor"]
            search: "helix hx editor config config.toml"
            path: $helix_path
        }
        {
            id: "zellij"
            label: $"zellij  (ansi dark_gray)- managed Zellij user config → ($zellij_path)(ansi reset)"
            aliases: ["zellij", "terminal", "config.kdl"]
            search: "zellij terminal config.kdl multiplexer"
            path: $zellij_path
        }
        {
            id: "yazi"
            label: $"yazi  (ansi dark_gray)- managed Yazi main config \(yazi.toml\) → ($yazi_toml_path)(ansi reset)"
            aliases: ["yazi", "yazi.toml", "file-manager"]
            search: "yazi yazi.toml file-manager file manager"
            path: $yazi_toml_path
        }
    ]
}

def resolve_edit_target_by_id [target_id: string] {
    get_edit_targets | where id == $target_id | first
}

def filter_edit_targets [targets: list<record>, query_text: string] {
    let normalized = ($query_text | str downcase | str trim)
    if ($normalized | is-empty) {
        return $targets
    }

    let exact = (
        $targets | where {|target|
            (
                (($target.id | str downcase) == $normalized)
                or (($target.aliases? | default []) | any {|alias| ($alias | str downcase) == $normalized })
            )
        }
    )
    if not ($exact | is-empty) {
        return $exact
    }

    let tokens = ($normalized | split row " " | where {|token| not ($token | is-empty) })
    $targets | where {|target|
        let haystack = (
            [
                $target.id
                ...($target.aliases? | default [])
                ($target.search? | default "")
            ]
            | str join " "
            | str downcase
        )
        $tokens | all {|token| $haystack | str contains $token }
    }
}

def render_edit_target_choices [targets: list<record>] {
    $targets | get label
}

def render_edit_target_error_choices [targets: list<record>] {
    $targets
    | each {|target| $"  - ($target.id): ($target.path)" }
    | str join "\n"
}

def choose_edit_target [targets: list<record>, prompt: string] {
    let selected = (render_edit_target_choices $targets | input list --fuzzy $prompt)
    if ($selected | is-empty) {
        return null
    }

    $targets | where label == $selected | first
}

export def "yzx edit" [
    ...query: string  # Optional managed config surface name or alias
    --print  # Print the resolved config path without opening
] {
    let targets = (get_edit_targets)
    let query_text = ($query | str join " " | str trim)

    if ($query_text | is-empty) {
        if $print {
            error make {msg: $"yzx edit --print requires a target query. Supported managed surfaces:\n(render_edit_target_error_choices $targets)"}
        }

        let selected = (choose_edit_target $targets "yzx edit \(Esc to cancel\)> ")
        if $selected == null {
            return
        }

        return (open_config_surface_in_editor $selected.path)
    }

    let matches = (filter_edit_targets $targets $query_text)

    if ($matches | is-empty) {
        error make {msg: $"No managed Yazelix config surface matched `($query_text)`. Supported surfaces:\n(render_edit_target_error_choices $targets)"}
    }

    if (($matches | length) == 1) {
        let match = ($matches | first)
        return (open_config_surface_in_editor $match.path --print=$print)
    }

    if $print {
        error make {msg: $"Query `($query_text)` matched multiple managed config surfaces. Refine it to one of:\n(render_edit_target_error_choices $matches)"}
    }

    let selected = (choose_edit_target $matches $"yzx edit \((query_text)\)> ")
    if $selected == null {
        return
    }

    open_config_surface_in_editor $selected.path
}

export def "yzx edit config" [
    --print  # Print the config path without opening
] {
    let target = (resolve_edit_target_by_id "config")
    open_config_surface_in_editor $target.path --print=$print
}

export def "yzx edit packs" [
    --print  # Print the config path without opening
] {
    let target = (resolve_edit_target_by_id "packs")
    open_config_surface_in_editor $target.path --print=$print
}

def resolve_config_migration_context [] {
    let paths = get_primary_config_paths
    let user_exists = ($paths.user_config | path exists)
    let user_pack_exists = ($paths.user_pack_config | path exists)
    let legacy_exists = ($paths.legacy_user_config | path exists)
    let legacy_pack_exists = ($paths.legacy_pack_config | path exists)

    if ($user_exists or $user_pack_exists) and ($legacy_exists or $legacy_pack_exists) {
        error make {
            msg: (
                [
                    "Yazelix found duplicate config surfaces in both the repo root and user_configs."
                    $"user_configs main: ($paths.user_config)"
                    $"user_configs packs: ($paths.user_pack_config)"
                    $"legacy main: ($paths.legacy_user_config)"
                    $"legacy packs: ($paths.legacy_pack_config)"
                    ""
                    "Keep only the user_configs copies. Move or delete the legacy root-level config files so Yazelix has one clear config owner."
                ] | str join "\n"
            )
        }
    }

    if $user_exists {
        return {
            paths: $paths
            preview_config_path: $paths.user_config
            preview_pack_path: $paths.user_pack_config
            relocation_needed: false
        }
    }

    if $legacy_exists {
        return {
            paths: $paths
            preview_config_path: $paths.legacy_user_config
            preview_pack_path: $paths.legacy_pack_config
            relocation_needed: true
        }
    }

    error make {msg: $"User config not found: ($paths.user_config)"}
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

    let context = (resolve_config_migration_context)
    let pre_apply_recovery = if $apply and (not $context.relocation_needed) {
        recover_stale_managed_config_transactions $context.paths.user_config
    } else {
        {
            recovered_count: 0
            transaction_ids: []
        }
    }
    let preview_plan = (get_config_migration_plan $context.preview_config_path)
    if $context.relocation_needed {
        print "Yazelix config path migration preview"
        print $"[AUTO] relocate_root_config_surfaces_into_user_configs"
        print $"  Legacy main: ($context.paths.legacy_user_config)"
        if ($context.paths.legacy_pack_config | path exists) {
            print $"  Legacy packs: ($context.paths.legacy_pack_config)"
        }
        print $"  Target main: ($context.paths.user_config)"
        print $"  Target packs: ($context.paths.user_pack_config)"
        print "  Change: Move the legacy root-level managed config files into user_configs before applying any safe rewrites."
        print ""
    }
    print (render_config_migration_plan $preview_plan)

    if not $apply {
        return
    }

    let had_path_relocation = $context.relocation_needed
    if $had_path_relocation {
        with-env { YAZELIX_ACCEPT_USER_CONFIG_RELOCATION: "true" } {
            reconcile_primary_config_surfaces | ignore
        }
    }

    let recovery = if $had_path_relocation {
        recover_stale_managed_config_transactions $context.paths.user_config
    } else {
        $pre_apply_recovery
    }
    if $recovery.recovered_count > 0 {
        print ""
        print $"ℹ️  Recovered ($recovery.recovered_count) interrupted managed-config transaction\(s\) before applying new migrations."
    }

    let apply_plan = (get_config_migration_plan $context.paths.user_config)

    if (not $apply_plan.has_auto_changes) and (not $had_path_relocation) {
        print ""
        print "No safe config rewrites to apply."
        return
    }

    if not $yes {
        print ""
        print "⚠️  This rewrites yazelix.toml from parsed TOML."
        if $had_path_relocation {
            print "   It will also move legacy root-level config files into user_configs."
        }
        print "   It may also create or rewrite yazelix_packs.toml when packs are migrated."
        print "   Any rewritten file will be backed up first."
        print "   Comments and key ordering may be normalized."
        let confirm = try {
            (input "Apply the safe migrations? [y/N]: " | str downcase)
        } catch { "n" }
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            return
        }
    }

    let apply_result = if $apply_plan.has_auto_changes {
        apply_config_migration_plan $apply_plan "config_migrate"
    } else {
        {
            status: "relocated"
            config_path: $context.paths.user_config
            backup_path: null
            pack_config_path: $context.paths.user_pack_config
            pack_backup_path: null
            applied_count: 0
            manual_count: $apply_plan.manual_count
        }
    }

    print ""
    if $had_path_relocation {
        print $"✅ Relocated managed config into: ($context.paths.user_config)"
        if ($context.paths.user_pack_config | path exists) {
            print $"✅ Using managed pack config at: ($context.paths.user_pack_config)"
        }
    }
    if ($apply_result.backup_path? | is-not-empty) {
        print $"✅ Backed up previous config to: ($apply_result.backup_path)"
    }
    if ($apply_result.pack_backup_path? | is-not-empty) {
        print $"✅ Backed up previous pack config to: ($apply_result.pack_backup_path)"
    }
    if ($apply_result.pack_config_path? | is-not-empty) and ($apply_result.pack_backup_path? | is-empty) and (($apply_result.pack_config_path | path exists)) {
        print $"✅ Wrote pack config to: ($apply_result.pack_config_path)"
    }
    if $apply_result.applied_count > 0 {
        print $"✅ Applied ($apply_result.applied_count) config migration\(s\) to: ($apply_result.config_path)"
        print "ℹ️  Comments and key ordering were normalized because Yazelix rewrote the file from parsed TOML."
    } else if $had_path_relocation {
        print "ℹ️  No additional TOML rewrites were needed after relocating the managed config surfaces."
    }

    if $apply_result.manual_count > 0 {
        print $"ℹ️  ($apply_result.manual_count) manual migration item\(s\) still need follow-up."
    }
}

export def "yzx config reset" [
    --yes        # Skip confirmation prompt
    --no-backup  # Replace config surfaces without writing timestamped backups first
] {
    use ../utils/config_surfaces.nu [copy_default_config_surfaces]
    let paths = get_primary_config_paths
    let user_config_exists = ($paths.user_config | path exists)
    let user_pack_config_exists = ($paths.user_pack_config | path exists)
    let removed_without_backup = ($no_backup and ($user_config_exists or $user_pack_config_exists))

    if not ($paths.default_config | path exists) {
        error make {msg: $"Default config not found: ($paths.default_config)"}
    }

    if not $yes {
        print "⚠️  This replaces yazelix.toml and yazelix_packs.toml with fresh shipped templates."
        if $user_config_exists and not $no_backup {
            print "   Your current yazelix.toml will be backed up first."
        }
        if $user_config_exists and $no_backup {
            print "   Your current yazelix.toml will be removed without a backup."
        }
        if $user_pack_config_exists and not $no_backup {
            print "   Your current yazelix_packs.toml will be backed up first."
        }
        if $user_pack_config_exists and $no_backup {
            print "   Your current yazelix_packs.toml will be removed without a backup."
        }
        let confirm = try {
            (input "Continue? [y/N]: " | str downcase)
        } catch { "n" }
        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            return
        }
    }

    let backup_path = if $user_config_exists and not $no_backup {
        let timestamp = (date now | format date "%Y%m%d_%H%M%S")
        let path = $"($paths.user_config).backup-($timestamp)"
        mv $paths.user_config $path
        $path
    } else if $user_config_exists and $no_backup {
        rm $paths.user_config
        null
    } else {
        null
    }

    let pack_backup_path = if $user_pack_config_exists and not $no_backup {
        let timestamp = (date now | format date "%Y%m%d_%H%M%S")
        let path = $"($paths.user_pack_config).backup-($timestamp)"
        mv $paths.user_pack_config $path
        $path
    } else if $user_pack_config_exists and $no_backup {
        rm $paths.user_pack_config
        null
    } else {
        null
    }

    let copy_result = (copy_default_config_surfaces $paths.default_config $paths.user_config)

    if ($backup_path | is-not-empty) {
        print $"✅ Backed up previous config to: ($backup_path)"
    }
    if ($pack_backup_path | is-not-empty) {
        print $"✅ Backed up previous pack config to: ($pack_backup_path)"
    }
    print $"✅ Replaced yazelix.toml with a fresh template: ($paths.user_config)"
    if $copy_result.pack_config_copied {
        print $"✅ Replaced yazelix_packs.toml with a fresh template: ($copy_result.pack_config_path)"
    }
    if $removed_without_backup {
        print "⚠️  Previous config surfaces were removed without backup."
    }
}
