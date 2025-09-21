#!/usr/bin/env nu
# Universal shell initializer generator for Yazelix
# Generates initializer scripts for all supported shells

def main [yazelix_dir: string, recommended: bool, shells_to_configure_str: string] {
    # Import constants for XDG paths
    use ../utils/constants.nu *

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
        { 
            name: "nu" 
            dir: ($SHELL_INITIALIZER_DIRS.nushell | str replace "~" $env.HOME)
            ext: "nu" 
            tool_overrides: { zoxide: "nushell" } 
        }
        { 
            name: "bash" 
            dir: ($SHELL_INITIALIZER_DIRS.bash | str replace "~" $env.HOME)
            ext: "sh" 
            tool_overrides: {} 
        }
        { 
            name: "fish" 
            dir: ($SHELL_INITIALIZER_DIRS.fish | str replace "~" $env.HOME)
            ext: "fish" 
            tool_overrides: {} 
        }
        { 
            name: "zsh" 
            dir: ($SHELL_INITIALIZER_DIRS.zsh | str replace "~" $env.HOME)
            ext: "zsh" 
            tool_overrides: {} 
        }
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

            # Skip recommended tools if not requested; remove any stale initializer
            if (not $tool.required) and (not $recommended) {
                if ($output_file | path exists) { rm $output_file }
                { status: "skipped", tool: $tool.name, shell: $shell.name, reason: "recommended" }
            } else if (which $tool.name | is-empty) {
                # Tool not found: record and remove any previous output
                if $tool.required {
                    if ($output_file | path exists) { rm $output_file }
                    { status: "required-missing", tool: $tool.name, shell: $shell.name, reason: "tool not found" }
                } else {
                    if ($output_file | path exists) { rm $output_file }
                    { status: "missing", tool: $tool.name, shell: $shell.name, reason: "tool not found" }
                }
            } else {
                try {
                    # Use tool-specific shell name override if available
                    let effective_shell_name = if ($tool.name in $shell.tool_overrides) {
                        $shell.tool_overrides | get $tool.name
                    } else {
                        $shell.name
                    }

                    let init_content = if ($tool.name == "mise") {
                        (run-external "mise" "activate" $effective_shell_name)
                    } else if ($tool.name == "carapace" and $shell.name == "nu") {
                        (run-external "carapace" "_carapace" "nushell")
                    } else {
                        (run-external $tool.name "init" $effective_shell_name)
                    }
                    $init_content | save --force $output_file
                    { status: "success", tool: $tool.name, shell: $shell.name, file: $output_file }
                } catch { |error|
                    # On failure, record and remove any previous output
                    if ($output_file | path exists) { rm $output_file }
                    if $tool.required {
                        { status: "required-failed", tool: $tool.name, shell: $shell.name, error: $error.msg }
                    } else {
                        { status: "failed", tool: $tool.name, shell: $shell.name, error: $error.msg }
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
            | each {|r| $"# WARNING: required initializer not generated for ($r.tool): (($r.reason? | default $r.error))\n" }
            | str join ""
        )

        ($header + $required_issues + $aggregate_content + "\n") | save --force $aggregate_file

        # Return both per-tool results and the aggregate file info
        $tool_results | append [{ status: "aggregate", shell: $shell.name, file: $aggregate_file }]
    } | flatten)

    # Show concise summary
    let successful = ($results | where status == "success")
    let failed = ($results | where status == "failed")
    let missing = ($results | where status == "missing" | get tool | uniq)

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
