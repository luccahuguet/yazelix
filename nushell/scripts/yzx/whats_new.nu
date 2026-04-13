#!/usr/bin/env nu

use ../utils/upgrade_summary.nu [show_current_upgrade_summary]

# Show the current Yazelix upgrade summary
export def "yzx whats_new" [] {
    show_current_upgrade_summary --mark-seen | ignore
}
