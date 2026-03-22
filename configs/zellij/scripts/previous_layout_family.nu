#!/usr/bin/env nu

use ../../../nushell/scripts/integrations/zellij.nu *

let result = (previous_layout_family)

if $result.status != "ok" {
    print $"Error: previous layout family failed \(status=($result.status)\)"
    exit 1
}
