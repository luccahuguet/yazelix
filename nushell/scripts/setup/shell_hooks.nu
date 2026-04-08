#!/usr/bin/env nu
# Shell Hook Setup Module
# Generic shell hook installation and migration for all supported shells

use ../utils/constants.nu [
    SHELL_CONFIGS
    YAZELIX_START_MARKER
]
use ../utils/shell_config_generation.nu [get_yazelix_section_content]
use ../utils/config_manager.nu [extract_yazelix_section rewrite_shell_hooks]

def get_legacy_yazelix_shell_hook_generation [config_content: string] {
    let legacy_markers = [
        {generation: "v1", marker: "# YAZELIX START - Yazelix managed configuration (do not modify this comment)"}
        {generation: "v2", marker: "# YAZELIX START v2 - Yazelix managed configuration (do not modify this comment)"}
        {generation: "v3", marker: "# YAZELIX START v3 - Yazelix managed configuration (do not modify this comment)"}
    ]

    $legacy_markers
    | where {|entry| $config_content | str contains $entry.marker}
    | get -o 0.generation
    | default null
}

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

    if ($config_content | str contains $YAZELIX_START_MARKER) {
        if $existing_section.exists and ($config_content | str contains $section_content) {
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

    let legacy_generation = (get_legacy_yazelix_shell_hook_generation $config_content)
    if $legacy_generation != null {
        error make {
            msg: $"Legacy Yazelix shell hooks detected in ($shell_config).\n   Yazelix no longer auto-migrates ($legacy_generation) shell hook generations.\n   Recovery: delete the old Yazelix-managed section from ($shell_config), then rerun `yzx refresh` to generate the current hooks."
            label: {
                text: $"Manual shell-hook cleanup required for ($legacy_generation)"
                span: (metadata $shell).span
            }
        }
    }

    if not $quiet_mode {
        print $"🐚 Adding Yazelix ($shell | str capitalize) config to ($shell_config)"
    }
    $"\n\n($section_content)" | save --append $shell_config
}
