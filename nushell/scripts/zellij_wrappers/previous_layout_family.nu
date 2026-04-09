#!/usr/bin/env nu

use ../integrations/zellij.nu [run_pane_orchestrator_command_raw]

def main [] {
    let response = (run_pane_orchestrator_command_raw "previous_family")
    if $response != "ok" {
        print $"Error: previous layout family failed \(status=($response)\)"
        exit 1
    }
}
