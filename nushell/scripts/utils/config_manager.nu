#!/usr/bin/env nu
# Yazelix Configuration Manager
# Utilities for reading, updating, and managing yazelix configuration sections in user shell configs

# Extract yazelix configuration section from a shell config file
export def extract_yazelix_section [config_file: string] {
    use ./constants_with_helpers.nu *

    if not ($config_file | path exists) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: "", version: 0 }
    }

    let content = (open $config_file | lines)

    # Handle empty content
    if ($content | is-empty) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: "", version: 0 }
    }

    # Handle content with only whitespace/empty lines
    let non_empty_content = ($content | where ($it | str trim) != "")
    if ($non_empty_content | is-empty) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: ($content | str join "\n"), version: 0 }
    }

    # Try v4 markers first (current version)
    let start_line_v4 = try { ($content | enumerate | where item == $YAZELIX_START_MARKER | get index | first | default (-1)) } catch { -1 }
    let end_line_v4 = try { ($content | enumerate | where item == $YAZELIX_END_MARKER | get index | first | default (-1)) } catch { -1 }

    # If v4 found, return it
    if ($start_line_v4 != -1) and ($end_line_v4 != -1) {
        let section_content = ($content | slice ($start_line_v4 + 1)..($end_line_v4 - 1) | str join "\n")
        return {
            exists: true
            content: $section_content
            start_line: $start_line_v4
            end_line: $end_line_v4
            full_content: ($content | str join "\n")
            version: 4
        }
    }

    # Try v3 markers
    let start_line_v3 = try { ($content | enumerate | where item == $YAZELIX_START_MARKER_V3 | get index | first | default (-1)) } catch { -1 }
    let end_line_v3 = try { ($content | enumerate | where item == $YAZELIX_END_MARKER_V3 | get index | first | default (-1)) } catch { -1 }

    # If v3 found, return it
    if ($start_line_v3 != -1) and ($end_line_v3 != -1) {
        let section_content = ($content | slice ($start_line_v3 + 1)..($end_line_v3 - 1) | str join "\n")
        return {
            exists: true
            content: $section_content
            start_line: $start_line_v3
            end_line: $end_line_v3
            full_content: ($content | str join "\n")
            version: 3
        }
    }

    # Try v2 markers
    let start_line_v2 = try { ($content | enumerate | where item == $YAZELIX_START_MARKER_V2 | get index | first | default (-1)) } catch { -1 }
    let end_line_v2 = try { ($content | enumerate | where item == $YAZELIX_END_MARKER_V2 | get index | first | default (-1)) } catch { -1 }

    # If v2 found, return it
    if ($start_line_v2 != -1) and ($end_line_v2 != -1) {
        let section_content = ($content | slice ($start_line_v2 + 1)..($end_line_v2 - 1) | str join "\n")
        return {
            exists: true
            content: $section_content
            start_line: $start_line_v2
            end_line: $end_line_v2
            full_content: ($content | str join "\n")
            version: 2
        }
    }

    # Try v1 markers as fallback
    let start_line_v1 = try { ($content | enumerate | where item == $YAZELIX_START_MARKER_V1 | get index | first | default (-1)) } catch { -1 }
    let end_line_v1 = try { ($content | enumerate | where item == $YAZELIX_END_MARKER_V1 | get index | first | default (-1)) } catch { -1 }

    if ($start_line_v1 != -1) and ($end_line_v1 != -1) {
        let section_content = ($content | slice ($start_line_v1 + 1)..($end_line_v1 - 1) | str join "\n")
        return {
            exists: true
            content: $section_content
            start_line: $start_line_v1
            end_line: $end_line_v1
            full_content: ($content | str join "\n")
            version: 1
        }
    }

    # No yazelix section found
    {
        exists: false
        content: ""
        start_line: -1
        end_line: -1
        full_content: ($content | str join "\n")
        version: 0
    }
}

# Check if yazelix configuration sections are up to date
export def check_config_versions [yazelix_dir: string] {
    use ./constants_with_helpers.nu *

    let configs = [
        { name: "bash", file: ($SHELL_CONFIGS.bash | str replace "~" $env.HOME), expected_source: (get_yazelix_runtime_config_path "bash" $yazelix_dir), expected_yzx_core: ($yazelix_dir | path join "nushell" "scripts" "core" "yazelix.nu"), expected_yzx_cli: (get_yzx_cli_path) }
        { name: "nushell", file: ($SHELL_CONFIGS.nushell | str replace "~" $env.HOME), expected_source: (get_yazelix_runtime_config_path "nushell" $yazelix_dir), expected_yzx_core: ($yazelix_dir | path join "nushell" "scripts" "core" "yazelix.nu"), expected_yzx_cli: (get_yzx_cli_path) }
        { name: "fish", file: ($SHELL_CONFIGS.fish | str replace "~" $env.HOME), expected_source: (get_yazelix_runtime_config_path "fish" $yazelix_dir), expected_yzx_core: ($yazelix_dir | path join "nushell" "scripts" "core" "yazelix.nu"), expected_yzx_cli: (get_yzx_cli_path) }
        { name: "zsh", file: ($SHELL_CONFIGS.zsh | str replace "~" $env.HOME), expected_source: (get_yazelix_runtime_config_path "zsh" $yazelix_dir), expected_yzx_core: ($yazelix_dir | path join "nushell" "scripts" "core" "yazelix.nu"), expected_yzx_cli: (get_yzx_cli_path) }
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

export def rewrite_shell_hooks [shell: string, config_file: string, yazelix_dir: string]: nothing -> record {
    use ./constants_with_helpers.nu *

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
            from_version: $section.version
            to_version: 4
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

# Safely migrate hooks to latest version with backup
export def migrate_shell_hooks [shell: string, config_file: string, yazelix_dir: string]: nothing -> record {
    use ./constants_with_helpers.nu *

    if not ($config_file | path exists) {
        return { migrated: false, reason: "config file not found" }
    }

    # Extract current section
    let section = extract_yazelix_section $config_file

    # Only migrate if hooks exist
    if not $section.exists {
        return { migrated: false, reason: "no yazelix section found" }
    }

    # Check if already on latest version (v4)
    if $section.version == 4 {
        return { migrated: false, reason: "already on v4" }
    }

    # Only migrate v1, v2, and v3
    if $section.version not-in [1, 2, 3] {
        return { migrated: false, reason: "unknown version" }
    }

    let rewrite = rewrite_shell_hooks $shell $config_file $yazelix_dir
    if $rewrite.rewritten {
        {
            migrated: true
            backup: $rewrite.backup
            shell: $rewrite.shell
            config: $rewrite.config
            from_version: $rewrite.from_version
            to_version: $rewrite.to_version
        }
    } else {
        {
            migrated: false
            reason: $rewrite.reason
            error: ($rewrite.error? | default null)
        }
    }
}
