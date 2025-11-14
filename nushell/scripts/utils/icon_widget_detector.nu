#!/usr/bin/env nu
# Determine whether to use icon widgets in zjstatus based on config and terminal capability

use config_parser.nu parse_yazelix_config
use constants.nu *

# Check if the terminal was launched by Yazelix (not an external terminal)
# This is the only reliable way to know if Nerd Fonts are configured
export def is_yazelix_launched_terminal []: nothing -> bool {
    # YAZELIX_TERMINAL is set when launching via "yzx launch" or launch_yazelix.nu
    # If this env var is present, we know it's a Yazelix-managed launch with Nerd Fonts
    # If absent, it's either "yzx launch --here" or a manual external terminal launch
    ($env.YAZELIX_TERMINAL? | is-not-empty)
}

# Determine if icon widgets should be used based on config and terminal
export def should_use_icons []: nothing -> bool {
    # Parse the config
    let config = (parse_yazelix_config)

    # Get user preference (defaults to true if not set)
    let prefer_icons = ($config.zellij?.prefer_icon_widgets? | default true)

    # Check if terminal was launched by Yazelix
    let yazelix_launched = (is_yazelix_launched_terminal)

    # If using external terminal (not launched by Yazelix), always use ASCII regardless of preference
    # This covers both "yzx launch --here" and manual external terminal launches
    if (not $yazelix_launched) {
        return false
    }

    # If launched by Yazelix, we know Nerd Fonts are available, so respect user preference
    return $prefer_icons
}

# Main entry point - print "icons" or "ascii"
def main []: nothing -> string {
    if (should_use_icons) {
        "icons"
    } else {
        "ascii"
    }
}
