#!/usr/bin/env nu
# Yazelix Configuration Manager
# Utilities for reading, updating, and managing yazelix configuration sections in user shell configs

# Extract yazelix configuration section from a shell config file
export def extract_yazelix_section [config_file: string] {
    use ./constants.nu *

    if not ($config_file | path exists) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: "" }
    }

    let start_marker = $YAZELIX_START_MARKER
    let end_marker = $YAZELIX_END_MARKER
    let content = (open $config_file | lines)

        # Handle empty content
    if ($content | is-empty) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: "" }
    }

    # Handle content with only whitespace/empty lines
    let non_empty_content = ($content | where ($it | str trim) != "")
    if ($non_empty_content | is-empty) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: ($content | str join "\n") }
    }

    let start_line = try { ($content | enumerate | where item == $start_marker | get index | first | default (-1)) } catch { -1 }
    let end_line = try { ($content | enumerate | where item == $end_marker | get index | first | default (-1)) } catch { -1 }

    if ($start_line == -1) or ($end_line == -1) {
        return { exists: false, content: "", start_line: -1, end_line: -1, full_content: ($content | str join "\n") }
    }

    let section_content = ($content | slice ($start_line + 1)..($end_line - 1) | str join "\n")

    {
        exists: true
        content: $section_content
        start_line: $start_line
        end_line: $end_line
        full_content: ($content | str join "\n")
    }
}

# Check if yazelix configuration sections are up to date
export def check_config_versions [yazelix_dir: string] {
    use ./constants.nu *

    let configs = [
        { name: "bash", file: ($SHELL_CONFIGS.bash | str replace "~" $env.HOME), expected_source: ($YAZELIX_CONFIG_FILES.bash) }
        { name: "nushell", file: ($SHELL_CONFIGS.nushell | str replace "~" $env.HOME), expected_source: ($YAZELIX_CONFIG_FILES.nushell) }
        { name: "fish", file: ($SHELL_CONFIGS.fish | str replace "~" $env.HOME), expected_source: ($YAZELIX_CONFIG_FILES.fish) }
        { name: "zsh", file: ($SHELL_CONFIGS.zsh | str replace "~" $env.HOME), expected_source: ($YAZELIX_CONFIG_FILES.zsh) }
    ]

    let results = ($configs | each { |config|
        if not ($config.file | path exists) {
            { shell: $config.name, status: "missing", file: $config.file }
        } else {
            let section = extract_yazelix_section $config.file
            if not $section.exists {
                { shell: $config.name, status: "missing", file: $config.file }
            } else {
                let expected_source_lines = if $config.name in ["bash", "fish", "zsh"] {
                    [
                        $"source \"($config.expected_source | str replace '~' $env.HOME)\""
                        $"source ($config.expected_source | str replace '~' $env.HOME)"
                        $"source \"$HOME/($config.expected_source | str replace '~/.config/' '.config/')\""
                        $"source $HOME/($config.expected_source | str replace '~/.config/' '.config/')"
                        $"source \"~($config.expected_source | str replace '~' '')\""
                        $"source ~($config.expected_source | str replace '~' '')"
                        $"source \"($config.expected_source)\""
                        $"source ($config.expected_source)"
                    ]
                } else {
                    [ $"source \"($config.expected_source)\"" ]
                }
                if ($expected_source_lines | any { |line| $section.content | str contains $line }) {
                    { shell: $config.name, status: "current", file: $config.file }
                } else {
                    { shell: $config.name, status: "outdated", file: $config.file, current_content: $section.content }
                }
            }
        }
    })

    $results
}

# Show configuration status for all shells
export def show_config_status [yazelix_dir: string] {
    let status = check_config_versions $yazelix_dir

    print "=== Yazelix Configuration Status ==="

    for $result in $status {
        if $result.status == "missing" {
            print $"ℹ️  ($result.shell): Not configured \(optional\)"
        } else if $result.status == "current" {
            print $"✅ ($result.shell): Configuration is current"
        } else if $result.status == "outdated" {
            print $"⚠️  ($result.shell): Configuration may be outdated in ($result.file)"
        }
    }

    print "==================================="
    $status
}

