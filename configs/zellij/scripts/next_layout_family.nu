#!/usr/bin/env nu

use ../../../nushell/scripts/integrations/zellij.nu *

let result = (next_layout_family)

if $result.status != "ok" {
    print $"Error: next layout family failed \(status=($result.status)\)"
    exit 1
}
