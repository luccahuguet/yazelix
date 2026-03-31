#!/usr/bin/env nu
# Shell Configuration Generation Functions
# Helper functions for generating yazelix shell hook configurations

use constants.nu [
    YAZELIX_START_MARKER
    YAZELIX_END_MARKER
    YAZELIX_REGENERATE_COMMENT
    YAZELIX_CONFIG_FILES
]
use common.nu [get_yazelix_runtime_reference_dir]

# Get the full start comment with regeneration instruction
export def get_yazelix_start_comment [] {
    $YAZELIX_START_MARKER + "\n" + $YAZELIX_REGENERATE_COMMENT
}

export def get_yazelix_runtime_config_path [shell: string, yazelix_dir: string] {
    let relative_path = ($YAZELIX_CONFIG_FILES | get -o $shell)
    if $relative_path == null {
        error make {msg: $"Unsupported shell config path lookup: ($shell)"}
    }
    ($yazelix_dir | path join $relative_path)
}

export def get_yzx_cli_path [] {
    ($env.HOME | path join ".local" "bin" "yzx")
}

# Get the complete yazelix section content for a shell
export def get_yazelix_section_content [shell: string, yazelix_dir: string] {
    let runtime_ref = (get_yazelix_runtime_reference_dir)
    let config_file = (get_yazelix_runtime_config_path $shell $runtime_ref)
    let yzx_core_path = ($runtime_ref | path join "nushell" "scripts" "core" "yazelix.nu")
    let yzx_cli_path = (get_yzx_cli_path)

    # Generate shell-specific conditional loading + yzx function (always available)
    let section_body = if $shell == "bash" or $shell == "zsh" {
        [
            $"if [ -n \"$IN_YAZELIX_SHELL\" ]; then"
            $"  source \"($config_file)\""
            "fi"
            "# yzx command - always available for launching/managing yazelix"
            "yzx() {"
            $"    \"($yzx_cli_path)\" \"$@\""
            "}"
        ] | str join "\n"
    } else if $shell == "fish" {
        [
            "if test -n \"$IN_YAZELIX_SHELL\""
            $"  source \"($config_file)\""
            "end"
            "# yzx command - always available for launching/managing yazelix"
            "function yzx --description \"Yazelix command suite\""
            $"    \"($yzx_cli_path)\" $argv"
            "end"
        ] | str join "\n"
    } else {
        # Nushell - always source, conditional is inside the config file itself
        # This works because sourcing inside an if block doesn't export aliases properly
        [
            $"source \"($config_file)\""
            $"use ($yzx_core_path) *"
        ] | str join "\n"
    }

    (get_yazelix_start_comment) + "\n" + $section_body + "\n" + $YAZELIX_END_MARKER
}
