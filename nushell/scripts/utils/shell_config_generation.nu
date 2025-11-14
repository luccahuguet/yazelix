#!/usr/bin/env nu
# Shell Configuration Generation Functions
# Helper functions for generating yazelix shell hook configurations

use constants.nu [
    YAZELIX_START_MARKER
    YAZELIX_END_MARKER
    YAZELIX_REGENERATE_COMMENT
    YAZELIX_CONFIG_FILES
]

# Get the full start comment with regeneration instruction
export def get_yazelix_start_comment [] {
    $YAZELIX_START_MARKER + "\n" + $YAZELIX_REGENERATE_COMMENT
}

# Get the complete yazelix section content for a shell
export def get_yazelix_section_content [shell: string, yazelix_dir: string] {
    let config_file = $YAZELIX_CONFIG_FILES | get $shell

    # Generate shell-specific conditional loading + yzx function (always available)
    let section_body = if $shell == "bash" or $shell == "zsh" {
        let home_file = ($config_file | str replace "~" "$HOME")
        [
            $"if [ -n \"$IN_YAZELIX_SHELL\" ]; then"
            $"  source \"($home_file)\""
            "fi"
            "# yzx command - always available for launching/managing yazelix"
            "yzx() {"
            "    nu -c \"use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx $*\""
            "}"
        ] | str join "\n"
    } else if $shell == "fish" {
        let home_file = ($config_file | str replace "~" "$HOME")
        [
            "if test -n \"$IN_YAZELIX_SHELL\""
            $"  source \"($home_file)\""
            "end"
            "# yzx command - always available for launching/managing yazelix"
            "function yzx --description \"Yazelix command suite\""
            "    nu -c \"use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx $argv\""
            "end"
        ] | str join "\n"
    } else {
        # Nushell - always source, conditional is inside the config file itself
        # This works because sourcing inside an if block doesn't export aliases properly
        [
            $"source \"($config_file)\""
            "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *"
        ] | str join "\n"
    }

    (get_yazelix_start_comment) + "\n" + $section_body + "\n" + $YAZELIX_END_MARKER
}
