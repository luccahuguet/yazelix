#!/usr/bin/env nu
# Helpers for nixless system mode

use terminal_launcher.nu command_exists

def resolve_command_name [cmd: string]: nothing -> string {
    if ($cmd | str contains " ") {
        $cmd | split row " " | first
    } else {
        $cmd
    }
}

def command_available [cmd: string]: nothing -> bool {
    let name = resolve_command_name $cmd
    if ($name | str contains "/") {
        $name | path exists
    } else {
        command_exists $name
    }
}

export def require_command [cmd: string, label: string] {
    let name = resolve_command_name $cmd
    if ($name | is-empty) {
        print $"Error: Missing ($label) command"
        exit 1
    }
    if not (command_available $name) {
        print $"Error: Missing ($label) command: ($name)"
        exit 1
    }
}

export def assert_no_packs [config: record] {
    let mode = ($config.environment_mode? | default "nix")
    if $mode != "system" {
        return
    }

    let has_enabled = (($config.packs_enabled? | default []) | length) > 0
    let has_user = (($config.packs_user_packages? | default []) | length) > 0
    if $has_enabled or $has_user {
        print "Error: environment.mode = \"system\" does not support packs."
        print "Remove [packs] entries or switch environment.mode to \"nix\"."
        exit 1
    }
}
