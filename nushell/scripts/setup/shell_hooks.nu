#!/usr/bin/env nu
# Shell Hook Setup Module
# Generic shell hook installation and migration for all supported shells

use ../utils/constants.nu [
    SHELL_CONFIGS
    YAZELIX_START_MARKER
    YAZELIX_END_MARKER
]
use ../utils/shell_config_generation.nu [
    get_yazelix_runtime_config_path
    get_yazelix_section_content
    get_yzx_cli_path
]

def extract_yazelix_section [config_file: string] {
    if not ($config_file | path exists) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: "" }
    }

    let content = (open $config_file | lines)
    if ($content | is-empty) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: "" }
    }

    let non_empty_content = ($content | where ($it | str trim) != "")
    if ($non_empty_content | is-empty) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: ($content | str join "\n") }
    }

    let start_line_v4 = try { ($content | enumerate | where item == $YAZELIX_START_MARKER | get index | first | default (-1)) } catch { -1 }
    let end_line_v4 = try { ($content | enumerate | where item == $YAZELIX_END_MARKER | get index | first | default (-1)) } catch { -1 }

    if ($start_line_v4 != -1) and ($end_line_v4 != -1) {
        let section_content = ($content | slice ($start_line_v4 + 1)..($end_line_v4 - 1) | str join "\n")
        return {
            exists: true
            content: $section_content
            start_line: $start_line_v4
            end_line: $end_line_v4
            full_content: ($content | str join "\n")
        }
    }

    {
        exists: false
        content: ""
        start_line: -1
        end_line: -1
        full_content: ($content | str join "\n")
    }
}

def rewrite_shell_hooks [shell: string, config_file: string, yazelix_dir: string]: nothing -> record {
    if not ($config_file | path exists) {
        return { rewritten: false, reason: "config file not found" }
    }

    let section = extract_yazelix_section $config_file
    if not $section.exists {
        return { rewritten: false, reason: "no yazelix section found" }
    }

    let timestamp = (date now | format date "%Y%m%d_%H%M%S")
    let backup_file = $"($config_file).yazelix-backup-($timestamp)"

    try {
        cp $config_file $backup_file

        let content_lines = (open $config_file | lines)
        let new_yazelix_section = get_yazelix_section_content $shell $yazelix_dir
        let before_section = ($content_lines | take ($section.start_line))
        let after_section = ($content_lines | skip ($section.end_line + 1))

        let new_content = (
            $before_section
            | append ($new_yazelix_section | lines)
            | append $after_section
            | str join "\n"
        )

        $new_content | save -f $config_file

        {
            rewritten: true
            backup: $backup_file
            shell: $shell
            config: $config_file
        }
    } catch { |err|
        if ($backup_file | path exists) {
            try {
                cp $backup_file $config_file
            }
        }
        {
            rewritten: false
            reason: $"rewrite failed: ($err.msg)"
            error: $err
        }
    }
}

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

export def check_shell_hook_versions [yazelix_dir: string] {
    let configs = [
        { name: "bash", file: ($SHELL_CONFIGS.bash | str replace "~" $env.HOME), expected_source: (get_yazelix_runtime_config_path "bash" $yazelix_dir), expected_yzx_cli: (get_yzx_cli_path $yazelix_dir) }
        { name: "nushell", file: ($SHELL_CONFIGS.nushell | str replace "~" $env.HOME), expected_source: (get_yazelix_runtime_config_path "nushell" $yazelix_dir), expected_yzx_cli: (get_yzx_cli_path $yazelix_dir) }
        { name: "fish", file: ($SHELL_CONFIGS.fish | str replace "~" $env.HOME), expected_source: (get_yazelix_runtime_config_path "fish" $yazelix_dir), expected_yzx_cli: (get_yzx_cli_path $yazelix_dir) }
        { name: "zsh", file: ($SHELL_CONFIGS.zsh | str replace "~" $env.HOME), expected_source: (get_yazelix_runtime_config_path "zsh" $yazelix_dir), expected_yzx_cli: (get_yzx_cli_path $yazelix_dir) }
    ]

    $configs | each { |config|
        if not ($config.file | path exists) {
            { shell: $config.name, status: "missing", file: $config.file }
        } else {
            let section = extract_yazelix_section $config.file
            if not $section.exists {
                { shell: $config.name, status: "missing", file: $config.file }
            } else {
                let expected_source_lines = if $config.name in ["bash", "fish", "zsh"] {
                    [
                        $"source \"($config.expected_source)\""
                        $"source ($config.expected_source)"
                    ]
                } else {
                    [ $"source \"($config.expected_source)\"" ]
                }
                let expected_yzx_lines = if $config.name == "fish" {
                    [ $"    \"($config.expected_yzx_cli)\" $argv" ]
                } else if $config.name == "nushell" {
                    []
                } else {
                    [ $"    \"($config.expected_yzx_cli)\" \"$@\"" ]
                }
                let yzx_line_ok = if $config.name == "nushell" {
                    true
                } else {
                    ($expected_yzx_lines | any { |line| $section.content | str contains $line })
                }
                if (
                    ($expected_source_lines | any { |line| $section.content | str contains $line })
                    and $yzx_line_ok
                ) {
                    { shell: $config.name, status: "current", file: $config.file }
                } else {
                    { shell: $config.name, status: "outdated", file: $config.file, current_content: $section.content }
                }
            }
        }
    }
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
            msg: $"Legacy Yazelix shell hooks detected in ($shell_config).\n   Yazelix no longer auto-migrates ($legacy_generation) shell hook generations.\n   Recovery: delete the old Yazelix-managed section from ($shell_config), then rerun `yzx launch`, `yzx enter`, or your install/setup flow to generate the current hooks."
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
