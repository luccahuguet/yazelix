#!/usr/bin/env nu
# Internal Nu entrypoint for the Rust-owned public `yzx` root.

use ./yazelix.nu [
    "yzx cwd"
    "yzx doctor"
    "yzx restart"
    "yzx reveal"
    "yzx sponsor"
    "yzx status"
    "yzx why"
]
use ../yzx/config.nu ["yzx config" "yzx config reset"]
use ../yzx/desktop.nu [
    "yzx desktop install"
    "yzx desktop launch"
    "yzx desktop uninstall"
]
use ../yzx/dev.nu [
    "yzx dev"
    "yzx dev build_pane_orchestrator"
    "yzx dev bump"
    "yzx dev lint_nu"
    "yzx dev profile"
    "yzx dev sync_issues"
    "yzx dev test"
    "yzx dev update"
]
use ../yzx/edit.nu ["yzx edit" "yzx edit config"]
use ../yzx/enter.nu ["yzx enter"]
use ../yzx/home_manager.nu ["yzx home_manager" "yzx home_manager prepare"]
use ../yzx/import.nu [
    "yzx import"
    "yzx import helix"
    "yzx import yazi"
    "yzx import zellij"
]
use ../yzx/keys.nu [
    "yzx keys"
    "yzx keys helix"
    "yzx keys hx"
    "yzx keys nu"
    "yzx keys nushell"
    "yzx keys yazi"
    "yzx keys yzx"
]
use ../yzx/launch.nu ["yzx launch"]
use ../yzx/menu.nu ["yzx menu"]
use ../yzx/popup.nu ["yzx popup"]
use ../yzx/screen.nu ["yzx screen"]
use ../yzx/tutor.nu [
    "yzx tutor"
    "yzx tutor helix"
    "yzx tutor hx"
    "yzx tutor nu"
    "yzx tutor nushell"
]
use ../yzx/whats_new.nu ["yzx whats_new"]

def first_rest [rest: list<string>] {
    $rest | get -o 0 | default ""
}

def route_config [rest: list<string>] {
    if (first_rest $rest) == "reset" {
        yzx config reset ...($rest | skip 1)
    } else {
        yzx config ...$rest
    }
}

def route_desktop [rest: list<string>] {
    match (first_rest $rest) {
        "install" => { yzx desktop install ...($rest | skip 1) }
        "launch" => { yzx desktop launch ...($rest | skip 1) }
        "uninstall" => { yzx desktop uninstall ...($rest | skip 1) }
        _ => {
            error make {msg: "yzx desktop requires one of: install, launch, uninstall"}
        }
    }
}

def route_dev [rest: list<string>] {
    match (first_rest $rest) {
        "" | "help" | "-h" | "--help" => { yzx dev }
        "build_pane_orchestrator" => { yzx dev build_pane_orchestrator ...($rest | skip 1) }
        "bump" => { yzx dev bump ...($rest | skip 1) }
        "lint_nu" => { yzx dev lint_nu ...($rest | skip 1) }
        "profile" => { yzx dev profile ...($rest | skip 1) }
        "sync_issues" => { yzx dev sync_issues ...($rest | skip 1) }
        "test" => { yzx dev test ...($rest | skip 1) }
        "update" => { yzx dev update ...($rest | skip 1) }
        _ => {
            error make {msg: "Unknown yzx dev subcommand"}
        }
    }
}

def route_edit [rest: list<string>] {
    if (first_rest $rest) == "config" {
        yzx edit config ...($rest | skip 1)
    } else {
        yzx edit ...$rest
    }
}

def route_home_manager [rest: list<string>] {
    if (first_rest $rest) == "prepare" {
        yzx home_manager prepare ...($rest | skip 1)
    } else {
        yzx home_manager ...$rest
    }
}

def route_import [rest: list<string>] {
    match (first_rest $rest) {
        "" | "help" | "-h" | "--help" => { yzx import }
        "helix" => { yzx import helix ...($rest | skip 1) }
        "yazi" => { yzx import yazi ...($rest | skip 1) }
        "zellij" => { yzx import zellij ...($rest | skip 1) }
        _ => {
            error make {msg: "Unknown yzx import target"}
        }
    }
}

def route_keys [rest: list<string>] {
    match (first_rest $rest) {
        "" => { yzx keys }
        "help" | "-h" | "--help" => { yzx keys ...$rest }
        "helix" => { yzx keys helix }
        "hx" => { yzx keys hx }
        "nu" => { yzx keys nu }
        "nushell" => { yzx keys nushell }
        "yazi" => { yzx keys yazi }
        "yzx" => { yzx keys yzx }
        _ => {
            error make {msg: "Unknown yzx keys target"}
        }
    }
}

def route_tutor [rest: list<string>] {
    match (first_rest $rest) {
        "" => { yzx tutor }
        "help" | "-h" | "--help" => { yzx tutor ...$rest }
        "helix" => { yzx tutor helix }
        "hx" => { yzx tutor hx }
        "nu" => { yzx tutor nu }
        "nushell" => { yzx tutor nushell }
        _ => {
            error make {msg: "Unknown yzx tutor target"}
        }
    }
}

def main [route: string, ...rest: string] {
    match $route {
        "config" => { route_config $rest }
        "cwd" => { yzx cwd ...$rest }
        "desktop" => { route_desktop $rest }
        "dev" => { route_dev $rest }
        "doctor" => { yzx doctor ...$rest }
        "edit" => { route_edit $rest }
        "enter" => { yzx enter ...$rest }
        "home_manager" => { route_home_manager $rest }
        "import" => { route_import $rest }
        "keys" => { route_keys $rest }
        "launch" => { yzx launch ...$rest }
        "menu" => { yzx menu ...$rest }
        "popup" => { yzx popup ...$rest }
        "restart" => { yzx restart ...$rest }
        "reveal" => { yzx reveal ...$rest }
        "screen" => { yzx screen ...$rest }
        "sponsor" => { yzx sponsor ...$rest }
        "status" => { yzx status ...$rest }
        "tutor" => { route_tutor $rest }
        "whats_new" => { yzx whats_new ...$rest }
        "why" => { yzx why ...$rest }
        _ => {
            error make {msg: $"Unknown internal yzx route: ($route)"}
        }
    }
}
