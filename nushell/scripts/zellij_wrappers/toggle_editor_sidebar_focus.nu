#!/usr/bin/env nu

use ../integrations/zellij.nu [toggle_editor_sidebar_focus]
use ../integrations/yazi.nu [refresh_active_sidebar_yazi]

def main [] {
    let result = (toggle_editor_sidebar_focus)

    if $result.status in ["missing" "not_ready"] {
        exit 0
    }

    if $result.status != "ok" {
        print $\"Error: toggle editor/sidebar focus failed \\(status=($result.status)\\)\"
        exit 1
    }

    if (($result.target? | default "") == "sidebar") {
        refresh_active_sidebar_yazi | ignore
    }
}
