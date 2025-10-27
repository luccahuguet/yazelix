#!/usr/bin/env nu
# Terminal launcher utilities for Yazelix

use constants.nu [SUPPORTED_TERMINALS, TERMINAL_CONFIG_PATHS, TERMINAL_METADATA]

# Check if a command is available
export def command_exists [cmd: string]: nothing -> bool {
    (which $cmd | length) > 0
}

# Resolve config path for a terminal based on mode
export def resolve_terminal_config [terminal: string, mode: string]: nothing -> string {
    let home = $env.HOME
    let config_paths = $TERMINAL_CONFIG_PATHS | get $terminal

    if $mode == "yazelix" {
        $config_paths.yazelix | str replace "~" $home
    } else {
        # user or auto mode - prefer user config if it exists
        let user_path = if ($config_paths | get -o user_main) != null {
            # Special case for wezterm which has two possible user paths
            let main = $config_paths.user_main | str replace "~" $home
            let alt = $config_paths.user_alt | str replace "~" $home
            if ($main | path exists) { $main } else { $alt }
        } else {
            $config_paths.user | str replace "~" $home
        }

        let yazelix_path = $config_paths.yazelix | str replace "~" $home

        if ($user_path | path exists) { $user_path } else { $yazelix_path }
    }
}

# Detect available terminal (wrapper or direct)
export def detect_terminal [preferred: string, prefer_wrappers: bool = true] {
    # Build list of terminals to check: preferred first, then others
    let other_terminals = $SUPPORTED_TERMINALS | where $it != $preferred
    let ordered_terminals = ([$preferred] | append $other_terminals)

    let terminals_to_check = if $prefer_wrappers {
        # Check wrappers first, then direct
        let wrappers = $ordered_terminals | each {|t| {terminal: $t, use_wrapper: true}}
        let direct = $ordered_terminals | each {|t| {terminal: $t, use_wrapper: false}}
        $wrappers | append $direct
    } else {
        # Direct terminal only
        $ordered_terminals | each {|t| {terminal: $t, use_wrapper: false}}
    }

    # Find first available terminal
    for term_check in $terminals_to_check {
        let terminal = $term_check.terminal
        let use_wrapper = $term_check.use_wrapper
        let term_meta = $TERMINAL_METADATA | get $terminal

        let command = if $use_wrapper { $term_meta.wrapper } else { $terminal }

        if (command_exists $command) {
            return {
                terminal: $terminal
                name: $term_meta.name
                command: $command
                use_wrapper: $use_wrapper
            }
        }
    }

    # No terminal found
    null
}

# Build launch command for a terminal
export def build_launch_command [
    terminal_info: record
    config_path
    terminal_config_mode: string
    needs_reload: bool = true  # Whether to force environment reload
]: nothing -> string {
    let terminal = $terminal_info.terminal
    let command = $terminal_info.command
    let use_wrapper = $terminal_info.use_wrapper

    # Smart environment reload: only unset vars if config changed
    # This makes launches ~4s faster when config hasn't changed (uses inherited nix shell)
    # When config changed, we clear vars to force fresh nix develop and pick up changes
    let env_prefix = if $needs_reload {
        "env -u IN_YAZELIX_SHELL -u IN_NIX_SHELL "
    } else {
        ""
    }

    if $use_wrapper {
        # Wrappers handle config internally via environment variable
        $"nohup ($env_prefix)($command) >/dev/null 2>&1 &"
    } else {
        # Direct terminal launch with config
        match $terminal {
            "ghostty" => {
                $"nohup ($env_prefix)ghostty --config-file=($config_path) --title=\"Yazelix - Ghostty\" >/dev/null 2>&1 &"
            },
            "wezterm" => {
                $"nohup ($env_prefix)wezterm --config-file ($config_path) start --class=com.yazelix.Yazelix >/dev/null 2>&1 &"
            },
            "kitty" => {
                $"nohup ($env_prefix)kitty --config=($config_path) --class=com.yazelix.Yazelix --title=\"Yazelix - Kitty\" >/dev/null 2>&1 &"
            },
            "alacritty" => {
                $"nohup ($env_prefix)alacritty --config-file ($config_path) --title \"Yazelix - Alacritty\" >/dev/null 2>&1 &"
            },
            "foot" => {
                $"nohup ($env_prefix)foot --config ($config_path) --app-id com.yazelix.Yazelix >/dev/null 2>&1 &"
            },
            _ => {
                error make {msg: $"Unknown terminal: ($terminal)"}
            }
        }
    }
}

# Get display name for terminal
export def get_terminal_display_name [terminal_info: record]: nothing -> string {
    let name = $terminal_info.name
    if $terminal_info.use_wrapper {
        $"Yazelix - ($name) \(with GPU acceleration\)"
    } else {
        $"($name)"
    }
}
