#!/usr/bin/env nu
# Shell Hook Setup Module
# Generic shell hook installation and migration for all supported shells

use ../utils/constants.nu *
use ../utils/config_manager.nu migrate_shell_hooks

# Setup yazelix hooks for a specific shell with automatic v1->v2 migration
export def setup_shell_hooks [
    shell: string
    yazelix_dir: string
    quiet_mode: bool = false
]: nothing -> nothing {
    # Get shell-specific paths
    let shell_config = ($SHELL_CONFIGS | get $shell | str replace "~" $env.HOME)
    let yazelix_config = if $shell == "nushell" {
        $"($yazelix_dir)/nushell/config/config.nu"
    } else {
        $"($yazelix_dir)/shells/($shell)/yazelix_($shell)_config.($shell)"
    }
    let section_content = get_yazelix_section_content $shell $yazelix_dir

    # Check if yazelix config file exists (skip for optional shells)
    if not ($yazelix_config | path exists) {
        if not $quiet_mode {
            print $"⚠️  ($shell | str capitalize) config not found, skipping ($shell) setup"
        }
        return
    }

    # Ensure shell config directory exists
    mkdir ($shell_config | path dirname)

    # Create shell config if it doesn't exist (for nushell)
    if not ($shell_config | path exists) {
        if $shell == "nushell" {
            if not $quiet_mode {
                print $"📝 Creating new Nushell config: ($shell_config)"
            }
            "# Nushell user configuration (created by Yazelix setup)" | save $shell_config
        } else {
            touch $shell_config
        }
    }

    let config_content = (open $shell_config)

    # Check if v2 hooks already exist
    if ($config_content | str contains $YAZELIX_START_MARKER) {
        if not $quiet_mode {
            print $"✅ ($shell | str capitalize) config already sourced"
        }
        return
    }

    # Check for v1 hooks and migrate
    if ($config_content | str contains $YAZELIX_START_MARKER_V1) {
        let migration = migrate_shell_hooks $shell $shell_config $yazelix_dir
        if $migration.migrated {
            if not $quiet_mode {
                print $"🔄 Migrated ($shell | str capitalize) hooks to v2 \(backup: ($migration.backup)\)"
            }
        } else if not $quiet_mode {
            print $"⚠️  Migration skipped: ($migration.reason)"
        }
        return
    }

    # No existing hooks, add new v2 hooks
    if not $quiet_mode {
        print $"🐚 Adding Yazelix ($shell | str capitalize) config to ($shell_config)"
    }
    $"\n\n($section_content)" | save --append $shell_config
}
