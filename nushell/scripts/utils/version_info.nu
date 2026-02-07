#!/usr/bin/env nu
# Simple version information for Yazelix tools

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

def short_rev [rev: string] {
    if ($rev | is-empty) {
        "unknown"
    } else {
        $rev | str substring 0..6
    }
}

def format_locked_entry [node: record] {
    if ($node | is-empty) {
        return "not locked"
    }

    let locked = ($node | get -o locked)
    if ($locked | is-empty) {
        return "not locked"
    }

    let owner = ($locked | get -o owner | default "unknown")
    let repo = ($locked | get -o repo | default "unknown")
    let rev = (short_rev ($locked | get -o rev | default ""))
    let ref = ($node | get -o original | get -o ref | default "")

    if ($ref | is-empty) {
        $"($owner)/($repo)@($rev)"
    } else {
        $"($owner)/($repo)@($ref)@($rev)"
    }
}

def load_lockfile [] {
    let lock_path = $"($env.HOME)/.config/yazelix/devenv.lock"
    if not ($lock_path | path exists) {
        return null
    }

    try {
        open --raw $lock_path | from json
    } catch {
        null
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
                # Check if EDITOR is actually Helix before using it
                let editor = $env.EDITOR
                let is_helix = ($editor | str ends-with "/hx") or ($editor == "hx") or ($editor | str ends-with "/helix") or ($editor == "helix")
                if $is_helix {
                    try { (^$editor --version | lines | first | extract_first_semver) } catch { "error" }
                } else {
                    # Fallback to 'hx' for non-Helix editors
                    try { (hx --version | lines | first | extract_first_semver) } catch { "not available" }
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
                try { (wezterm --version | split column " " | get column2) } catch { "error" }
            }
            "ghostty" => {
                if (which ghostty | is-empty) { return "not installed" }
                try { (ghostty --version | lines | first | extract_first_semver) } catch { "error" }
            }
            "nix" => {
                if (which nix | is-empty) { return "not installed" }
                try { (nix --version | lines | first | extract_last_semver) } catch { "error" }
            }
            "devenv" => {
                if (which devenv | is-empty) { return "not installed" }
                try { (devenv --version | lines | first | extract_first_semver) } catch { "error" }
            }
            "kitty" => {
                if (which kitty | is-empty) { return "not installed" }
                try { (kitty --version | lines | first | extract_first_semver) } catch { "error" }
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

def get_locked_version [tool: string, lockfile: record] {
    if ($lockfile | is-empty) {
        return "not locked"
    }

    let nodes = ($lockfile | get -o nodes)
    if ($nodes | is-empty) {
        return "not locked"
    }

    let nixpkgs_locked = (format_locked_entry ($nodes | get -o nixpkgs))

    match $tool {
        "devenv" => (format_locked_entry ($nodes | get -o devenv))
        "helix" => (format_locked_entry ($nodes | get -o helix))
        "nix" => $nixpkgs_locked
        _ => $nixpkgs_locked
    }
}

# Main function - markdown table output
export def main [
    --save(-s)
] {
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
        "devenv"
        "kitty"
        "foot"
        "alacritty"
        "macchina"
    ]

    # Collect tool information
    let lockfile = load_lockfile
    let tool_data = ($tools | each { |tool|
        let locked = get_locked_version $tool $lockfile
        let runtime = get_version $tool
        {tool: $tool, locked: $locked, runtime: $runtime}
    })

    if $save {
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
            "- **Locked**: Flake input revisions when available (nix uses nixpkgs)"
            "- **Runtime**: Versions resolved from current PATH"
        ]

        let full_output = ([$header [$table_md] $notes] | flatten | str join "\n")

        let file_path = "docs/version_table.md"
        $full_output | save $file_path --force
        print $"✅ Version table saved to ($file_path)"
    } else {
        print "Yazelix Tool Versions"
        print $"Generated: (date now | format date '%Y-%m-%d %H:%M:%S')"
        print ($tool_data | table)
        print ""
        print "Run with --save to write Markdown output."
    }
}
