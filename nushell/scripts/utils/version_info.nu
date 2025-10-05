#!/usr/bin/env nu
# Simple version information for Yazelix tools

# Get version for a tool with simple fallback
def get_version [tool: string] {
    try {
        match $tool {
            "yazi" => {
                if (which yazi | is-empty) { return "not installed" }
                try { (yazi --version | lines | first | split column " " | get column2 | str replace --all '[' '' | str replace --all ']' '') } catch { "error" }
            }
            "zellij" => {
                if (which zellij | is-empty) { return "not installed" }
                try { (zellij --version | str replace "zellij " "") } catch { "error" }
            }
            "helix" => {
                # Check if EDITOR is actually Helix before using it
                let editor = $env.EDITOR
                let is_helix = ($editor | str ends-with "/hx") or ($editor == "hx") or ($editor | str ends-with "/helix") or ($editor == "helix")
                if $is_helix {
                    try { (^$editor --version | lines | first | split column " " | get column2 | str replace --all '[' '' | str replace --all ']' '') } catch { "error" }
                } else {
                    # Fallback to 'hx' for non-Helix editors
                    try { (hx --version | lines | first | split column " " | get column2 | str replace --all '[' '' | str replace --all ']' '') } catch { "not available" }
                }
            }
            "nushell" => {
                if (which nu | is-empty) { return "not installed" }
                try { (nu --version) } catch { "error" }
            }
            "zoxide" => {
                if (which zoxide | is-empty) { return "not installed" }
                try { (zoxide --version | split column " " | get column2 | str replace --all '[' '' | str replace --all ']' '') } catch { "error" }
            }
            "starship" => {
                if (which starship | is-empty) { return "not installed" }
                try { (starship --version | lines | first | split column " " | get column2 | str replace --all '[' '' | str replace --all ']' '') } catch { "error" }
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
                try { (fzf --version | split column " " | get column1 | str replace --all '[' '' | str replace --all ']' '') } catch { "error" }
            }
            "wezterm" => {
                if (which wezterm | is-empty) { return "not installed" }
                try { (wezterm --version | split column " " | get column2) } catch { "error" }
            }
            "ghostty" => {
                if (which ghostty | is-empty) { return "not installed" }
                try { (ghostty --version | lines | first | split column " " | get column2) } catch { "error" }
            }
            "nix" => {
                if (which nix | is-empty) { return "not installed" }
                try { (nix --version | split column " " | get column3 | str replace --all '[' '' | str replace --all ']' '') } catch { "error" }
            }
            "kitty" => {
                if (which kitty | is-empty) { return "not installed" }
                try { (kitty --version | split column " " | get column2) } catch { "error" }
            }
            "foot" => {
                if (which foot | is-empty) { return "not installed" }
                try { (foot --version | split column " " | get column2?) } catch { "error" }
            }
            "alacritty" => {
                if (which alacritty | is-empty) { return "not installed" }
                try { (alacritty --version | split column " " | get column2) } catch { "error" }
            }
            "macchina" => {
                if (which macchina | is-empty) { return "not installed" }
                try {
                    (macchina -v | lines | first | split row ' ' | where $it != '' | get 1)
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

# Main function - markdown table output
export def main [--save(-s)] {
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

    # Collect tool information
    let tool_data = ($tools | each { |tool|
        let version = get_version $tool
        {tool: $tool, version: $version}
    })

    let header = [
        "# Yazelix Tool Versions"
        ""
        $"Generated: (date now | format date '%Y-%m-%d %H:%M:%S')"
        ""
    ]

    let table_md = ($tool_data | to md --pretty)

    let notes = [
        ""
        "## Usage"
        ""
        "- **Regenerate**: `nu nushell/scripts/utils/version_info.nu --save`"
        "- **View only**: `nu nushell/scripts/utils/version_info.nu`"
    ]

    let full_output = ([$header [$table_md] $notes] | flatten | str join "\n")

    if $save {
        let file_path = "docs/version_table.md"
        $full_output | save $file_path --force
        print $"✅ Version table saved to ($file_path)"
    } else {
        print $full_output
    }
}
