#!/usr/bin/env nu

use ../utils/upgrade_summary.nu [show_current_upgrade_summary]

export def "yzx whats_new" [] {
    show_current_upgrade_summary --mark-seen | ignore
}
