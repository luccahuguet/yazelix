#!/usr/bin/env nu
# Simple version information for Yazelix tools

# Get version for a tool with simple fallback
def get_version [tool: string] {
    try {
        match $tool {
            "yazi" => {
                if (which yazi | is-empty) { return "not installed" }
                try { (yazi --version | lines | first | split column " " | get column2) } catch { "error" }
            }
            "zellij" => {
                if (which zellij | is-empty) { return "not installed" }
                try { (zellij --version | str replace "zellij " "") } catch { "error" }
            }
            "helix" => {
                if (which hx | is-empty) { return "not installed" }
                try { (hx --version | lines | first | split column " " | get column2) } catch { "error" }
            }
            "nushell" => {
                if (which nu | is-empty) { return "not installed" }
                try { (nu --version) } catch { "error" }
            }
            "zoxide" => {
                if (which zoxide | is-empty) { return "not installed" }
                try { (zoxide --version | split column " " | get column2) } catch { "error" }
            }
            "starship" => {
                if (which starship | is-empty) { return "not installed" }
                try { (starship --version | lines | first | split column " " | get column2) } catch { "error" }
            }
                        "lazygit" => {
                if (which lazygit | is-empty) { return "not installed" }
                try {
                    let output = (lazygit --version | lines | first)
                    ($output | parse --regex 'version=([^,]+)' | get capture0 | first)
                } catch { "error" }
            }
            "fzf" => {
                if (which fzf | is-empty) { return "not installed" }
                try { (fzf --version | split column " " | get column1) } catch { "error" }
            }
            "wezterm" => {
                if (which wezterm | is-empty) { return "not installed" }
                try { (wezterm --version | split column " " | get column2) } catch { "error" }
            }
            "nix" => {
                if (which nix | is-empty) { return "not installed" }
                try { (nix --version | split column " " | get column3) } catch { "error" }
            }
            _ => {
                if (which $tool | is-empty) { return "not installed" }
                try {
                    let output = (run-external $tool "--version" | lines | first)
                    ($output | parse --regex '\d+\.\d+\.\d+' | first | values | first? | default "unknown")
                } catch { "error" }
            }
        }
    } catch {
        "not available"
    }
}

# Main function - simple version listing
export def main [] {
    print "Yazelix Tool Versions"
    print "====================="

    let tools = [
        "yazi"
        "zellij"
        "helix"
        "nushell"
        "zoxide"
        "starship"
        "lazygit"
        "fzf"
        "wezterm"
        "nix"
    ]

    for tool in $tools {
        let version = get_version $tool
        print $"($tool): ($version)"
    }

    print ""
    print $"Generated: (date now | format date '%Y-%m-%d %H:%M:%S')"
}