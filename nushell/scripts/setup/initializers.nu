#!/usr/bin/env nu
# Universal shell initializer generator for Yazelix
# Generates initializer scripts for all supported shells

def main [yazelix_dir: string, include_optional: bool] {
    print "🔧 Generating shell initializers..."

    # Configuration for tools and shells
    let tools = [
        { name: "starship", required: true, init_cmd: { |shell| $"starship init ($shell)" } }
        { name: "zoxide", required: true, init_cmd: { |shell| $"zoxide init ($shell)" } }
        { name: "mise", required: false, init_cmd: { |shell| $"mise activate ($shell)" } }
        { name: "carapace", required: false, init_cmd: { |shell| $"carapace ($shell)" } }
    ]

    let shells = [
        { name: "nu", dir: "nushell", ext: "nu", tool_overrides: { zoxide: "nushell" } }
        { name: "bash", dir: "bash", ext: "sh", tool_overrides: {} }
        { name: "fish", dir: "fish", ext: "fish", tool_overrides: {} }
        { name: "zsh", dir: "zsh", ext: "zsh", tool_overrides: {} }
    ]

    # Generate initializers for each shell
    for $shell in $shells {
        let init_dir = $"($yazelix_dir)/($shell.dir)/initializers"
        print $"  📁 Creating ($shell.name) initializers in ($init_dir)"
        mkdir $init_dir

        for $tool in $tools {
            # Skip optional tools if not requested
            if (not $tool.required) and (not $include_optional) {
                continue
            }

            # Check if tool is available
            if (which $tool.name | is-empty) {
                print $"    ⚠️  ($tool.name) not found, skipping"
                continue
            }

            print $"    🔨 Generating ($tool.name) for ($shell.name)"

            try {
                # Use tool-specific shell name override if available
                let effective_shell_name = if ($tool.name in $shell.tool_overrides) {
                    $shell.tool_overrides | get $tool.name
                } else {
                    $shell.name
                }

                let init_content = if ($tool.name == "mise") {
                    (run-external "mise" "activate" $effective_shell_name)
                } else {
                    (run-external $tool.name "init" $effective_shell_name)
                }
                let output_file = $"($init_dir)/($tool.name)_init.($shell.ext)"
                $init_content | save --force $output_file
                print $"    ✅ Generated ($output_file)"
            } catch { |error|
                print $"    Failed to generate ($tool.name) for ($shell.name): ($error.msg)"
            }
        }
    }

    print "✨ Shell initializers generated!"
}
