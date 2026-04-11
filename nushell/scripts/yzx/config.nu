#!/usr/bin/env nu
# yzx config - Show, migrate, and reset the Yazelix main config surface

use ../utils/config_migration_preview.nu [get_config_migration_plan render_config_migration_plan]
use ../utils/config_migrations.nu apply_config_migration_plan
use ../utils/config_migration_rules.nu validate_config_migration_rules
use ../utils/config_migration_transactions.nu recover_stale_managed_config_transactions
use ../utils/config_surfaces.nu [copy_default_config_surfaces get_primary_config_paths load_active_config_surface reconcile_primary_config_surfaces]

# Show the active Yazelix configuration
export def "yzx config" [
    --full   # Show the complete main config record
    --path   # Print the resolved config path
] {
    let config_surface = (load_active_config_surface)
    let config_path = $config_surface.config_file

    if $path {
        $config_path
    } else {
        $config_surface.merged_config
    }
}

def resolve_config_migration_context [] {
    let paths = get_primary_config_paths
    let user_exists = ($paths.user_config | path exists)
    let legacy_exists = ($paths.legacy_user_config | path exists)

    if $user_exists and $legacy_exists {
        error make {
            msg: (
                [
                    "Yazelix found duplicate config surfaces in both the repo root and user_configs."
                    $"user_configs main: ($paths.user_config)"
                    $"legacy main: ($paths.legacy_user_config)"
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
            preview_pack_path: null
            relocation_needed: false
        }
    }

    if $legacy_exists {
        return {
            paths: $paths
            preview_config_path: $paths.legacy_user_config
            preview_pack_path: null
            relocation_needed: true
        }
    }

    error make {msg: $"User config not found: ($paths.user_config)"}
}

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
        print $"  Target main: ($context.paths.user_config)"
        print "  Change: Move the legacy root-level managed config file into user_configs before applying any safe rewrites."
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
            applied_count: 0
            manual_count: $apply_plan.manual_count
        }
    }

    print ""
    if $had_path_relocation {
        print $"✅ Relocated managed config into: ($context.paths.user_config)"
    }
    if ($apply_result.backup_path? | is-not-empty) {
        print $"✅ Backed up previous config to: ($apply_result.backup_path)"
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
    let paths = get_primary_config_paths
    let user_config_exists = ($paths.user_config | path exists)
    let removed_without_backup = ($no_backup and $user_config_exists)

    if not ($paths.default_config | path exists) {
        error make {msg: $"Default config not found: ($paths.default_config)"}
    }

    if not $yes {
        print "⚠️  This replaces yazelix.toml with a fresh shipped template."
        if $user_config_exists and not $no_backup {
            print "   Your current yazelix.toml will be backed up first."
        }
        if $user_config_exists and $no_backup {
            print "   Your current yazelix.toml will be removed without a backup."
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

    copy_default_config_surfaces $paths.default_config $paths.user_config | ignore

    if ($backup_path | is-not-empty) {
        print $"✅ Backed up previous config to: ($backup_path)"
    }
    print $"✅ Replaced yazelix.toml with a fresh template: ($paths.user_config)"
    if $removed_without_backup {
        print "⚠️  Previous config surface was removed without backup."
    }
}
