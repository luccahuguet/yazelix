#!/usr/bin/env nu
# Universal shell initializer generator for Yazelix
# Generates initializer scripts for all supported shells

def strip_nu_starship_right_prompt [] {
    let lines = ($in | split row "\n")
    mut filtered = []
    mut skipping = false
    mut skipping_config = false

    for $line in $lines {
        if (not $skipping) and ($line | str contains "PROMPT_COMMAND_RIGHT: {||") {
            $skipping = true
        } else if (not $skipping_config) and ($line | str contains "config: ($env.config? | default {} | merge {") {
            $skipping_config = true
        } else if $skipping {
            if $line == "    }" {
                $skipping = false
            }
        } else if $skipping_config {
            if $line == "    })" {
                $skipping_config = false
            }
        } else {
            $filtered = ($filtered | append $line)
        }
    }

    $filtered | str join "\n"
}

def normalize_initializer_content [shell_name: string, init_content: string] {
    if $shell_name == "nu" {
        $init_content
        | str replace -a "get $field --ignore-errors" "get --optional $field"
        # Starship's right prompt triggers repeated cursor-position queries in
        # interactive Nushell panes, which makes new Yazelix panes feel frozen.
        | strip_nu_starship_right_prompt
    } else {
        $init_content
    }
}

def describe_initializer_issue [result: record] {
    if ("reason" in ($result | columns)) {
        $result.reason
    } else if ("error" in ($result | columns)) {
        $result.error
    } else {
        "unknown failure"
    }
}

def make_initializer_result [
    status: string
    tool: string
    shell: string
    reason: string = ""
    error: string = ""
    file: string = ""
] {
    mut result = {
        status: $status
        tool: $tool
        shell: $shell
    }

    if ($reason | is-not-empty) {
        $result = ($result | upsert reason $reason)
    }
    if ($error | is-not-empty) {
        $result = ($result | upsert error $error)
    }
    if ($file | is-not-empty) {
        $result = ($result | upsert file $file)
    }

    $result
}

def main [yazelix_dir: string, shells_to_configure_str: string] {
    # Import constants for XDG paths
    use ../utils/constants.nu *

    if not ($yazelix_dir | path exists) {
        error make {msg: $"Yazelix directory does not exist: ($yazelix_dir)"}
    }

    # Parse shells to configure from comma-separated string
    let shells_to_configure = if ($shells_to_configure_str | is-empty) {
        ["nu", "bash", "fish", "zsh"]
    } else {
        $shells_to_configure_str | split row "," | where $it != ""
    }

    # Configuration for tools and shells
    let tools = [
        { name: "starship", required: true, init_cmd: { |shell| $"starship init ($shell)" } }
        { name: "zoxide", required: true, init_cmd: { |shell| $"zoxide init ($shell)" } }
        { name: "atuin", required: false, init_cmd: { |shell| $"atuin init ($shell)" } }
        { name: "mise", required: false, init_cmd: { |shell| $"mise activate ($shell)" } }
        { name: "carapace", required: false, init_cmd: { |shell| $"carapace ($shell)" } }
    ]

    # Use XDG-compliant state directories for initializers
    let all_shells = [
        [name dir ext tool_overrides];
        ["nu" ($SHELL_INITIALIZER_DIRS.nushell | str replace "~" $env.HOME) "nu" { zoxide: "nushell" }]
        ["bash" ($SHELL_INITIALIZER_DIRS.bash | str replace "~" $env.HOME) "sh" {}]
        ["fish" ($SHELL_INITIALIZER_DIRS.fish | str replace "~" $env.HOME) "fish" {}]
        ["zsh" ($SHELL_INITIALIZER_DIRS.zsh | str replace "~" $env.HOME) "zsh" {}]
    ]

    # Filter shells to only include those we want to configure
    let shells = ($all_shells | where name in $shells_to_configure)

    # Generate initializers and collect results
    let results = ($shells | each { |shell|
        let init_dir = $shell.dir
        mkdir $init_dir

        # Collect per-tool generation results for this shell
        let tool_results = ($tools | each { |tool|
            # Compute expected output path for this tool/shell
            let output_file = $"($init_dir)/($tool.name)_init.($shell.ext)"

            if (which $tool.name | is-empty) {
                # Tool not found: record and remove any previous output
                if $tool.required {
                    if ($output_file | path exists) { rm $output_file }
                    make_initializer_result "required-missing" $tool.name $shell.name "tool not found"
                } else {
                    if ($output_file | path exists) { rm $output_file }
                    make_initializer_result "missing" $tool.name $shell.name "tool not found"
                }
            } else {
                try {
                    # Use tool-specific shell name override if available
                    let effective_shell_name = if ($tool.name in $shell.tool_overrides) {
                        $shell.tool_overrides | get -o $tool.name
                    } else {
                        $shell.name
                    }

                    let raw_init_content = if ($tool.name == "mise") {
                        (run-external "mise" "activate" $effective_shell_name)
                    } else if ($tool.name == "carapace" and $shell.name == "nu") {
                        (run-external "carapace" "_carapace" "nushell")
                    } else {
                        (run-external $tool.name "init" $effective_shell_name)
                    }
                    let init_content = (normalize_initializer_content $shell.name $raw_init_content)
                    $init_content | save --force $output_file
                    make_initializer_result "success" $tool.name $shell.name "" "" $output_file
                } catch { |error|
                    # On failure, record and remove any previous output
                    if ($output_file | path exists) { rm $output_file }
                    if $tool.required {
                        make_initializer_result "required-failed" $tool.name $shell.name "" $error.msg
                    } else {
                        make_initializer_result "failed" $tool.name $shell.name "" $error.msg
                    }
                }
            }
        })

        # After per-tool generation, build an aggregate initializer that always exists
        let aggregate_file = $"($init_dir)/yazelix_init.($shell.ext)"
        let header = $"# Yazelix aggregate initializer for ($shell.name)\n# Concatenates generated initializers for available tools.\n"

        # Determine inclusion order: required first, then optional successes
        let included = (
            $tool_results
            | where status == "success"
            | sort-by {|r| (if ($tools | where name == $r.tool | first).required { 0 } else { 1 }) }
        )

        let aggregate_content = (
            $included
            | each {|r| open $r.file }
            | str join "\n"
        )

        let required_issues = (
            $tool_results | where status in ["required-missing", "required-failed"]
            | each {|r| $"# WARNING: required initializer not generated for ($r.tool): (describe_initializer_issue $r)\n" }
            | str join ""
        )

        # For Nushell: add PATH preservation to prevent devenv paths from being lost
        let nushell_path_preservation = if ($shell.name == "nu") {
            [
                ""
                "# Preserve devenv-provided PATH (includes packs like python, rust, etc.)"
                "let initial_path = $env.PATH"
                ""
                "# --- Tool initializers below ---"
                ""
            ] | str join "\n"
        } else {
            ""
        }

        let nushell_path_restoration = if ($shell.name == "nu") {
            [
                ""
                "# --- Tool initializers above ---"
                ""
                "# Restore any PATH entries lost during initialization without letting stale saved entries outrank the current shell PATH"
                "let current_path = $env.PATH"
                "$env.PATH = ($current_path | append $initial_path | uniq)"
                ""
            ] | str join "\n"
        } else {
            ""
        }

        ($header + $required_issues + $nushell_path_preservation + $aggregate_content + $nushell_path_restoration + "\n") | save --force $aggregate_file

        # Return both per-tool results and the aggregate file info
        $tool_results | append [{ status: "aggregate", shell: $shell.name, file: $aggregate_file }]
    } | flatten)

    # Show concise summary (unless in quiet mode)
    let quiet_mode = ($env.YAZELIX_QUIET_MODE? == "true")
    let successful = ($results | where status == "success")
    let failed = ($results | where status == "failed")
    let missing = ($results | where status == "missing" | get tool | uniq)

    if not $quiet_mode {
        if ($failed | is-empty) and ($missing | is-empty) {
            print $"✅ Generated (($successful | length)) shell initializers successfully"
        } else {
            print $"✅ Generated (($successful | length)) shell initializers"
            if (not ($missing | is-empty)) {
                print $"⚠️  Tools not found: (($missing | str join ', '))"
            }
            if (not ($failed | is-empty)) {
                print "❌ Failed to generate:"
                for $failure in $failed {
                    print $"   ($failure.tool) for ($failure.shell): ($failure.error)"
                }
            }
        }
    }
}
