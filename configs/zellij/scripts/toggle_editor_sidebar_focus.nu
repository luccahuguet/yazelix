#!/usr/bin/env nu

use ~/.config/yazelix/nushell/scripts/integrations/zellij.nu *

let result = (toggle_editor_sidebar_focus)

if $result.status in ["missing" "not_ready"] {
    exit 0
}

if $result.status != "ok" {
    print $"Error: toggle editor/sidebar focus failed \(status=($result.status)\)"
    exit 1
}
