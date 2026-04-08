#!/usr/bin/env nu
# Shell Hook Setup Module
# Generic shell hook installation and migration for all supported shells

use ../utils/constants.nu [
    SHELL_CONFIGS
    YAZELIX_START_MARKER
    YAZELIX_START_MARKER_V1
    YAZELIX_START_MARKER_V2
    YAZELIX_START_MARKER_V3
]
use ../utils/shell_config_generation.nu [get_yazelix_section_content]
use ../utils/config_manager.nu [extract_yazelix_section migrate_shell_hooks rewrite_shell_hooks]

# Setup yazelix hooks for a specific shell with automatic v1->v2 migration
export def setup_shell_hooks [
    shell: string
    yazelix_dir: string
    quiet_mode: bool = false
    required: bool = false  # If true, error on missing config; if false, skip silently
]: nothing -> nothing {
    # Get shell-specific paths
    let shell_config = (($SHELL_CONFIGS | get -o $shell | default "") | str replace "~" $env.HOME)

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
    let existing_section = extract_yazelix_section $shell_config

    # Check if v4 hooks already exist and still match the current generated content
    if ($config_content | str contains $YAZELIX_START_MARKER) {
        if $existing_section.exists and ($existing_section.version == 4) and ($config_content | str contains $section_content) {
            if not $quiet_mode {
                print $"✅ ($shell | str capitalize) config already sourced"
            }
            return
        }

        let rewrite = rewrite_shell_hooks $shell $shell_config $yazelix_dir
        if $rewrite.rewritten {
            if not $quiet_mode {
                print $"🔄 Refreshed stale ($shell | str capitalize) hooks \(backup: ($rewrite.backup)\)"
            }
        } else if not $quiet_mode {
            print $"⚠️  Refresh skipped: ($rewrite.reason)"
        }
        return
    }

    # Check for v3 hooks and migrate to v4
    if ($config_content | str contains $YAZELIX_START_MARKER_V3) {
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
