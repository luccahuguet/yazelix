#!/usr/bin/env nu
# Lightweight internal support commands that still live in Nushell.

def has_external_command [command_name: string] {
    (which $command_name | where type == "external" | is-not-empty)
}

# Elevator pitch: Why Yazelix
export def "yzx why" [] {
    print "Yazelix is a reproducible terminal IDE (Yazi + Zellij + Helix) with:"
    print "• Zero‑conflict keybindings, zjstatus, smooth Yazi↔editor flows"
    print "• Top terminals (Ghostty/WezTerm/Kitty/Alacritty) and shells (Bash/Zsh/Fish/Nushell)"
    print "• One‑file config (Nix) with sane defaults and curated packs"
    print "• Remote‑ready over SSH; same superterminal on barebones hosts"
    print "• Git and tooling preconfigured (lazygit, starship, zoxide, carapace)"
    print "Get everything running in <10 minutes. No extra deps, only Nix."
    print "Install once, get the same environment everywhere."
}

# Open the Yazelix sponsor page or print its URL
export def "yzx sponsor" [] {
    let sponsor_url = "https://github.com/sponsors/luccahuguet"

    if (has_external_command "xdg-open") {
        let result = (^xdg-open $sponsor_url | complete)
        if $result.exit_code == 0 {
            print "Opened sponsor page."
            return
        }
    }

    if (has_external_command "open") {
        let result = (^open $sponsor_url | complete)
        if $result.exit_code == 0 {
            print "Opened sponsor page."
            return
        }
    }

    print "Support Yazelix:"
    print $sponsor_url
}
