#!/usr/bin/env nu
# yzx config - Show and reset the Yazelix main config surface

use ../utils/config_surfaces.nu [copy_default_config_surfaces get_primary_config_paths load_active_config_surface]

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
