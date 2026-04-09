#!/usr/bin/env nu

use ./zellij.nu get_active_sidebar_yazi_state_from_plugin

export def get_active_sidebar_state [] {
    if ($env.ZELLIJ? | is-empty) {
        return null
    }

    get_active_sidebar_yazi_state_from_plugin
}
