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
        { name: "nu", dir: "nushell", ext: "nu" }
        { name: "bash", dir: "bash", ext: "sh" } 
        { name: "fish", dir: "fish", ext: "fish" }
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
                let init_content = (run-external $tool.name "init" $shell.name)
                let output_file = $"($init_dir)/($tool.name)_init.($shell.ext)"
                $init_content | save $output_file
                print $"    ✅ Created ($output_file)"
            } catch { |error|
                print $"    ❌ Failed to generate ($tool.name) for ($shell.name): ($error.msg)"
            }
        }
    }
    
    print "✨ Shell initializers generated!"
} 