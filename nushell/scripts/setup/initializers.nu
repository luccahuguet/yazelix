#!/usr/bin/env nu
# Universal shell initializer generator for Yazelix
# Generates initializer scripts for all supported shells

def main [yazelix_dir: string, recommended: bool, shells_to_configure_str: string] {

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
        { name: "mise", required: false, init_cmd: { |shell| $"mise activate ($shell)" } }
        { name: "carapace", required: false, init_cmd: { |shell| $"carapace ($shell)" } }
    ]

    let all_shells = [
        { name: "nu", dir: "nushell", ext: "nu", tool_overrides: { zoxide: "nushell" } }
        { name: "bash", dir: "shells/bash", ext: "sh", tool_overrides: {} }
        { name: "fish", dir: "shells/fish", ext: "fish", tool_overrides: {} }
        { name: "zsh", dir: "shells/zsh", ext: "zsh", tool_overrides: {} }
    ]

    # Filter shells to only include those we want to configure
    let shells = ($all_shells | where name in $shells_to_configure)

    # Generate initializers and collect results
    let results = ($shells | each { |shell|
        let init_dir = $"($yazelix_dir)/($shell.dir)/initializers"
        mkdir $init_dir

        $tools | each { |tool|
            # Skip recommended tools if not requested
            if (not $tool.required) and (not $recommended) {
                { status: "skipped", tool: $tool.name, shell: $shell.name, reason: "recommended" }
            } else if (which $tool.name | is-empty) {
                { status: "missing", tool: $tool.name, shell: $shell.name, reason: "tool not found" }
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
                    let output_file = $"($init_dir)/($tool.name)_init.($shell.ext)"
                    $init_content | save --force $output_file
                    { status: "success", tool: $tool.name, shell: $shell.name, file: $output_file }
                } catch { |error|
                    { status: "failed", tool: $tool.name, shell: $shell.name, error: $error.msg }
                }
            }
        }
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
