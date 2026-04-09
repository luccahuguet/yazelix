#!/usr/bin/env nu

use ../integrations/zellij.nu [run_pane_orchestrator_command_raw]

def main [] {
    let response = (run_pane_orchestrator_command_raw "next_family")
    if $response != "ok" {
        print $"Error: next layout family failed \(status=($response)\)"
        exit 1
    }
}
