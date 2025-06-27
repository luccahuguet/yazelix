#!/usr/bin/env nu
# Universal shell initializer generator
# This script generates initializers for all supported shells

def main [base_dir: string, include_optional: bool = true] {
    print "🔧 Generating shell initializers..."
    
    # Define tools and their init commands
    let tools = [
        {
            name: "starship"
            required: true
            commands: {
                nu: "starship init nu"
                bash: "starship init bash" 
                fish: "starship init fish"
            }
        }
        {
            name: "zoxide"
            required: true
            commands: {
                nu: "zoxide init nushell --cmd z"
                bash: "zoxide init bash --cmd z"
                fish: "zoxide init fish --cmd z"
            }
        }
        {
            name: "mise"
            required: false
            commands: {
                nu: "mise activate nu"
                bash: "mise activate bash"
                fish: "mise activate fish"
            }
        }
        {
            name: "carapace"
            required: false
            commands: {
                nu: "carapace _carapace nushell"
                bash: "carapace _carapace bash"
                fish: "carapace _carapace fish"
            }
        }
    ]
    
    # Define shells and their directories
    let shells = [
        { name: "nu", dir: "nushell/initializers", ext: "nu" }
        { name: "bash", dir: "bash/initializers", ext: "sh" }
        { name: "fish", dir: "fish/initializers", ext: "fish" }
    ]
    
    # Generate initializers for each shell
    for shell in $shells {
        let shell_dir = ($base_dir | path join $shell.dir)
        mkdir $shell_dir
        print $"  📁 Creating ($shell.name) initializers in ($shell_dir)"
        
        for tool in $tools {
            # Skip optional tools if not requested
            if (not $tool.required) and (not $include_optional) {
                print $"    ⏭️  Skipping ($tool.name) (optional tool disabled)"
                continue
            }
            
            let init_file = ($shell_dir | path join $"($tool.name)_init.($shell.ext)")
            let cmd = $tool.commands | get $shell.name
            
            print $"    🔨 Generating ($tool.name) for ($shell.name)"
            
            try {
                # Split command into parts and execute properly
                let cmd_parts = ($cmd | split row " ")
                let result = (run-external ($cmd_parts | first) ...(($cmd_parts | skip 1)))
                $result | save --force $init_file
                print $"    ✅ Created ($init_file)"
            } catch {
                print $"    ❌ Failed to generate ($tool.name) for ($shell.name): ($in)"
                # Create empty file as fallback
                "" | save --force $init_file
            }
        }
    }
    
    print "✨ Shell initializers generated!"
} 