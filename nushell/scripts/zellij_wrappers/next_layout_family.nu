#!/usr/bin/env nu

use ../integrations/zellij.nu [next_layout_family]

def main [] {
    let result = (next_layout_family)
    if $result.status != "ok" {
        print $"Error: next layout family failed \(status=($result.status)\)"
        exit 1
    }
}
