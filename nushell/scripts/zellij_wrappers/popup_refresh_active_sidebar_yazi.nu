#!/usr/bin/env nu

use ../integrations/yazi.nu [refresh_active_sidebar_yazi]

def main [] {
    # Zellij reports popup closure before focus restoration fully settles.
    # Give the sidebar instance a moment to become active again before emitting into Yazi.
    sleep 150ms
    refresh_active_sidebar_yazi | ignore
}
