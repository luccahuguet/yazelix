#!/usr/bin/env nu

use ../integrations/zellij.nu [previous_layout_family]

def main [] {
    let result = (previous_layout_family)

    if $result.status != "ok" {
        print $"Error: previous layout family failed \(status=($result.status)\)"
        exit 1
    }
}
