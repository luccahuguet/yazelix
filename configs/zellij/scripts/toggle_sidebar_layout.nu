#!/usr/bin/env nu

use ~/.config/yazelix/nushell/scripts/integrations/zellij.nu *

let result = (toggle_sidebar_layout)

if $result.status != "ok" {
    print $"Error: toggle sidebar failed \(status=($result.status)\)"
    exit 1
}
