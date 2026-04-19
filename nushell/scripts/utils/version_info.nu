#!/usr/bin/env nu
# Simple version information for Yazelix tools

use helix_mode.nu [get_helix_binary]

def extract_first_semver [] {
    let matches = ($in | parse --regex '(\d+\.\d+\.\d+)' | get -o capture0)
    if ($matches | is-empty) {
        "unknown"
    } else {
        $matches | first
    }
}

def extract_last_semver [] {
    let matches = ($in | parse --regex '(\d+\.\d+\.\d+)' | get -o capture0)
    if ($matches | is-empty) {
        "unknown"
    } else {
        $matches | last
    }
}

# Get version for a tool with simple fallback
def get_version [tool: string] {
    try {
        match $tool {
            "yazi" => {
                if (which yazi | is-empty) { return "not installed" }
                try { (yazi --version | lines | first | extract_first_semver) } catch { "error" }
            }
            "zellij" => {
                if (which zellij | is-empty) { return "not installed" }
                try { (zellij --version | lines | first | extract_first_semver) } catch { "error" }
            }
            "helix" => {
                let helix_binary = (get_helix_binary)
                if $helix_binary == "hx" {
                    try { (hx --version | lines | first | extract_first_semver) } catch { "not available" }
                } else {
                    try { (^$helix_binary --version | lines | first | extract_first_semver) } catch { "error" }
                }
            }
            "nushell" => {
                if (which nu | is-empty) { return "not installed" }
                try { (nu --version) } catch { "error" }
            }
            "zoxide" => {
                if (which zoxide | is-empty) { return "not installed" }
                try { (zoxide --version | lines | first | extract_first_semver) } catch { "error" }
            }
            "starship" => {
                if (which starship | is-empty) { return "not installed" }
                try { (starship --version | lines | first | extract_first_semver) } catch { "error" }
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
                try { (fzf --version | lines | first | extract_first_semver) } catch { "error" }
            }
            "wezterm" => {
                if (which wezterm | is-empty) { return "not installed" }
                try {
                    let result = (^wezterm --version | complete)
                    if $result.exit_code != 0 { "error" } else { ($result.stdout | split column " " | get column2) }
                } catch { "error" }
            }
            "ghostty" => {
                if (which ghostty | is-empty) { return "not installed" }
                try {
                    let result = (^ghostty --version | complete)
                    if $result.exit_code != 0 { "error" } else { ($result.stdout | lines | first | extract_first_semver) }
                } catch { "error" }
            }
            "nix" => {
                if (which nix | is-empty) { return "not installed" }
                try {
                    let result = (^nix --version | complete)
                    if $result.exit_code != 0 { "error" } else { ($result.stdout | lines | first | extract_last_semver) }
                } catch { "error" }
            }
            "kitty" => {
                if (which kitty | is-empty) { return "not installed" }
                try {
                    let result = (^kitty --version | complete)
                    if $result.exit_code != 0 { "error" } else { ($result.stdout | lines | first | extract_first_semver) }
                } catch { "error" }
            }
            "foot" => {
                if (which foot | is-empty) { return "not installed" }
                try { (foot --version | lines | first | extract_first_semver) } catch { "error" }
            }
            "alacritty" => {
                if (which alacritty | is-empty) { return "not installed" }
                try { (alacritty --version | lines | first | extract_first_semver) } catch { "error" }
            }
            "macchina" => {
                if (which macchina | is-empty) { return "not installed" }
                try {
                    (macchina -v | lines | first | extract_first_semver)
                } catch { "error" }
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

export def collect_version_info [] {
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
        "ghostty"
        "nix"
        "kitty"
        "foot"
        "alacritty"
        "macchina"
    ]

    let tool_data = ($tools | each { |tool|
        let runtime = get_version $tool
        {tool: $tool, runtime: $runtime}
    })

    {
        title: "Yazelix Tool Versions"
        generated_at: (date now | format date '%Y-%m-%d %H:%M:%S')
        tools: $tool_data
    }
}

export def render_version_info [version_report: record] {
    print ($version_report.title? | default "Yazelix Tool Versions")
    print $"Generated: ($version_report.generated_at? | default '')"
    print (($version_report.tools? | default []) | table)
}

# Main function - markdown table output
export def print_version_info [] {
    render_version_info (collect_version_info)
}

export def main [] {
    print_version_info
}
