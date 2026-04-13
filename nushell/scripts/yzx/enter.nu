#!/usr/bin/env nu
# yzx enter command - Start Yazelix in the current terminal

use ../core/start_yazelix.nu [start_yazelix_session]

# Start Yazelix in the current terminal
export def "yzx enter" [
    --path(-p): string # Start in specific directory
    --home             # Start in home directory
    --verbose          # Enable verbose logging
] {
    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 yzx enter: verbose mode enabled"
    }

    $env.YAZELIX_ENV_ONLY = "false"

    let requested_path = ($path | default "")
    let cwd_override = if $home {
        $env.HOME
    } else if ($requested_path | is-not-empty) {
        $requested_path
    } else {
        null
    }

    if ($cwd_override != null) {
        start_yazelix_session $cwd_override --verbose=$verbose_mode
    } else {
        start_yazelix_session --verbose=$verbose_mode
    }
}
