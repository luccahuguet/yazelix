#!/usr/bin/env nu
# Dynamic version information generator for Yazelix
# Queries actual installed tools to generate current version table

# Get version information for a tool
def get_tool_version [tool: string, version_arg: string = "--version"] {
    try {
        if (which $tool | is-empty) {
            return "❌ Not installed"
        }
        
        # Special cases for tools with different version outputs
        match $tool {
            "yazi" => {
                let version = (yazi --version | lines | first | parse "{name} {version}" | get version | first)
                $version
            }
            "ya" => {
                let version = (ya --version | lines | first | parse "{name} {version}" | get version | first)
                $version
            }
            "helix" | "hx" => {
                let helix_cmd = if (which helix | is-not-empty) { "helix" } else { "hx" }
                let version = (run-external $helix_cmd "--version" | lines | first | parse "{name} {version} ({commit})" | get version | first)
                let commit = (run-external $helix_cmd "--version" | lines | first | parse "{name} {version} ({commit})" | get commit | first)
                $"($version) \(($commit)\)"
            }
            "zellij" => {
                let version = (zellij --version | str replace "zellij " "")
                $version
            }
            "starship" => {
                let version = (starship --version | str replace "starship " "")
                $version
            }
            "lazygit" => {
                let output = (lazygit --version | lines | first)
                let version = ($output | parse "version={version}" | get version | first)
                $version
            }
            _ => {
                let output = (run-external $tool $version_arg | lines | first)
                # Try to extract version number from common patterns
                let version = ($output | parse --regex '(?P<version>\d+\.\d+\.\d+)' | get version | first? | default $output)
                $version
            }
        }
    } catch {
        "❌ Error getting version"
    }
}

# Get system information
def get_system_info [] {
    let os_info = if ($env.OS? | default "" | str contains "Windows") {
        "Windows"
    } else {
        try {
            open /etc/os-release | lines | parse "{key}={value}" | where key == "PRETTY_NAME" | get value | first | str trim --char '"'
        } catch {
            let kernel = (run-external "uname" "-s" | str trim)
            let release = (run-external "uname" "-r" | str trim)
            $"($kernel) ($release)"
        }
    }
    
    let desktop = try {
        $env.XDG_CURRENT_DESKTOP? | default "Unknown"
    } catch {
        "Unknown"
    }
    
    {os: $os_info, desktop: $desktop}
}

# Generate version table
export def main [] {
    print "🔍 Gathering Yazelix version information..."
    
    let system = get_system_info
    let timestamp = (date now | format date "%B %d, %Y")
    
    # Tools managed by Nix
    let nix_tools = [
        "zellij"
        "yazi" 
        "helix"
        "nushell"
        "zoxide"
        "starship"
        "lazygit"
        "fzf"
        "mise"
        "fish"
        "zsh"
    ]
    
    # External dependencies
    let external_tools = [
        "wezterm"
        "ghostty"
        "nix"
    ]
    
    print $"# Yazelix Version Information\n"
    print $"- Generated: ($timestamp)"
    print $"- OS: ($system.os)"
    print $"- Desktop: ($system.desktop)"
    print $"- Nix manages most tool versions automatically\n"
    
    print $"## Nix-Managed Tools\n"
    print $"| Tool | Version |"
    print $"|------|---------|"
    
    for tool in $nix_tools {
        let version = get_tool_version $tool
        print $"| ($tool) | ($version) |"
    }
    
    print $"\n## External Dependencies\n"
    print $"| Tool | Version | Status |"
    print $"|------|---------|--------|"
    
    for tool in $external_tools {
        let version = get_tool_version $tool
        let status = if ($version | str starts-with "❌") {
            "Optional/Not Required"
        } else {
            "✅ Available"
        }
        print $"| ($tool) | ($version) | ($status) |"
    }
    
    print $"\n## Notes\n"
    print $"- **Nix-managed tools**: Versions are automatically coordinated by flake.nix"
    print $"- **WezTerm**: Required terminal emulator for Yazelix"
    print $"- **Ghostty**: Alternative terminal emulator (optional)"
    print $"- **Nix**: Required for dependency management"
    print $"- Tool versions may change when you update your Nix flake"
} 