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
    required: bool = false  # If true, error on missing config; if false, skip silently
]: nothing -> nothing {
    # Get shell-specific paths
    let shell_config = ($SHELL_CONFIGS | get $shell | str replace "~" $env.HOME)

    # Map shell to correct file extension
    let shell_ext = match $shell {
        "bash" => "sh"
        "zsh" => "zsh"
        "fish" => "fish"
        "nushell" => "nu"
    }

    let yazelix_config = if $shell == "nushell" {
        $"($yazelix_dir)/nushell/config/config.nu"
    } else {
        $"($yazelix_dir)/shells/($shell)/yazelix_($shell)_config.($shell_ext)"
    }
    let section_content = get_yazelix_section_content $shell $yazelix_dir

    # Check if yazelix config file exists
    if not ($yazelix_config | path exists) {
        if $required {
            # Required shells (bash, nushell) must have config files
            error make {
                msg: $"❌ Required ($shell) config file not found: ($yazelix_config)"
                label: {
                    text: "This is a critical error - yazelix cannot function without bash and nushell configs"
                    span: (metadata $shell).span
                }
            }
        } else {
            # Optional shells (fish, zsh) skip silently
            return
        }
    }

    # Check if shell config file exists
    if not ($shell_config | path exists) {
        if $required {
            # Required shells must have config files
            let help_message = if $shell == "nushell" {
                $"Run Nushell once to create config: nu"
            } else if $shell == "bash" {
                $"Create your bash config: touch ($shell_config)"
            } else {
                $"Create your ($shell) config file first"
            }

            error make {
                msg: $"❌ Required ($shell) config file not found: ($shell_config)\n   ($help_message)"
                label: {
                    text: $"($shell) config file is required for Yazelix"
                    span: (metadata $shell).span
                }
            }
        } else {
            # Optional shells skip silently
            return
        }
    }

    let config_content = (open $shell_config)

    # Check if v4 hooks already exist (current version)
    if ($config_content | str contains $YAZELIX_START_MARKER) {
        if not $quiet_mode {
            print $"✅ ($shell | str capitalize) config already sourced"
        }
        return
    }

    # Check for v3 hooks and migrate to v4 (v4 includes direnv)
    if ($config_content | str contains $YAZELIX_START_MARKER_V3) {
        let migration = migrate_shell_hooks $shell $shell_config $yazelix_dir
        if $migration.migrated {
            if not $quiet_mode {
                print $"🔄 Migrated ($shell | str capitalize) hooks from v($migration.from_version) to v($migration.to_version) \(now includes direnv, backup: ($migration.backup)\)"
            }
        } else if not $quiet_mode {
            print $"⚠️  Migration skipped: ($migration.reason)"
        }
        return
    }

    # Check for v2 hooks and migrate to v4
    if ($config_content | str contains $YAZELIX_START_MARKER_V2) {
        let migration = migrate_shell_hooks $shell $shell_config $yazelix_dir
        if $migration.migrated {
            if not $quiet_mode {
                print $"🔄 Migrated ($shell | str capitalize) hooks from v($migration.from_version) to v($migration.to_version) \(backup: ($migration.backup)\)"
            }
        } else if not $quiet_mode {
            print $"⚠️  Migration skipped: ($migration.reason)"
        }
        return
    }

    # Check for v1 hooks and migrate to v4
    if ($config_content | str contains $YAZELIX_START_MARKER_V1) {
        let migration = migrate_shell_hooks $shell $shell_config $yazelix_dir
        if $migration.migrated {
            if not $quiet_mode {
                print $"🔄 Migrated ($shell | str capitalize) hooks from v($migration.from_version) to v($migration.to_version) \(backup: ($migration.backup)\)"
            }
        } else if not $quiet_mode {
            print $"⚠️  Migration skipped: ($migration.reason)"
        }
        return
    }

    # No existing hooks, add new v4 hooks
    if not $quiet_mode {
        print $"🐚 Adding Yazelix ($shell | str capitalize) config to ($shell_config)"
    }
    $"\n\n($section_content)" | save --append $shell_config
}

# Setup direnv hook for a specific shell
export def setup_direnv_hook [
    shell: string
    quiet_mode: bool = false
]: nothing -> nothing {
    # Skip if direnv is not available
    if (which direnv | is-empty) {
        return
    }

    # Get shell-specific config path
    let shell_config = ($SHELL_CONFIGS | get $shell | str replace "~" $env.HOME)

    # Skip if shell config doesn't exist
    if not ($shell_config | path exists) {
        return
    }

    let config_content = (open $shell_config)

    # Check if direnv hook already exists
    if ($config_content | str contains $DIRENV_START_MARKER) {
        return
    }

    # Check if direnv hook might already be manually configured
    if ($shell == "bash" or $shell == "zsh") and ($config_content | str contains "direnv hook") {
        return
    } else if $shell == "fish" and ($config_content | str contains "direnv hook fish") {
        return
    } else if ($shell == "nu" or $shell == "nushell") and ($config_content | str contains "direnv export json") {
        return
    }

    # Add direnv hook
    let section_content = get_direnv_section_content $shell
    if not $quiet_mode {
        print $"⚡ Adding direnv hook to ($shell | str capitalize) config for 40x faster launches"
    }

    # For nushell, prepend direnv hook at the top (before Yazelix section)
    # For other shells, direnv is integrated into v4 section (this shouldn't be called)
    if $shell == "nushell" or $shell == "nu" {
        let existing_content = (open $shell_config)
        $"($section_content)\n\n($existing_content)" | save -f $shell_config
    } else {
        # Fallback: append (though this shouldn't happen for v4)
        $"\n\n($section_content)" | save --append $shell_config
    }
}
