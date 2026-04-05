#!/usr/bin/env nu

use ../integrations/zellij.nu [toggle_sidebar_layout]

def main [] {
    let result = (toggle_sidebar_layout)

    if $result.status != "ok" {
        print $"Error: toggle sidebar failed \(status=($result.status)\)"
        exit 1
    }
}
