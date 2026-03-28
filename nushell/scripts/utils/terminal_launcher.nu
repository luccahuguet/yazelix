#!/usr/bin/env nu
# Terminal launcher utilities for Yazelix

use constants.nu [SUPPORTED_TERMINALS, TERMINAL_CONFIG_PATHS, TERMINAL_METADATA]
use common.nu [get_yazelix_runtime_dir]

# Check if a command is available
export def command_exists [cmd: string]: nothing -> bool {
    (which $cmd | length) > 0
}

def get_startup_script_path []: nothing -> string {
    let runtime_dir = (get_yazelix_runtime_dir)
    $runtime_dir | path join "shells" "posix" "start_yazelix.sh"
}

# Resolve config path for a terminal based on mode
export def resolve_terminal_config [terminal: string, mode: string] {
    let home = $env.HOME
    let config_paths = $TERMINAL_CONFIG_PATHS | get $terminal

    if $mode == "yazelix" {
        return ($config_paths.yazelix | str replace "~" $home)
    }

    if $mode == "user" {
        let user_path = if ($config_paths | get -o user_main) != null {
            let main = $config_paths.user_main | str replace "~" $home
            let alt = $config_paths.user_alt | str replace "~" $home
            if ($main | path exists) { $main } else { $alt }
        } else {
            $config_paths.user | str replace "~" $home
        }

        if ($user_path | path exists) {
            return $user_path
        }

        error make {msg: $"terminal.config_mode = user requires a real ($terminal) user config at ($user_path)"}
    }

    error make {msg: $"Unsupported terminal.config_mode '($mode)'. Expected 'yazelix' or 'user'."}
}

export def resolve_terminal_config_from_env [terminal: string] {
    let mode = ($env.YAZELIX_TERMINAL_CONFIG_MODE? | default "yazelix" | into string | str downcase)
    resolve_terminal_config $terminal $mode
}

# Detect available terminal (wrapper or direct)
export def detect_terminal [preferred: any, prefer_wrappers: bool = true] {
    # Build list of terminals to check: use list order if provided, otherwise preferred first
    let ordered_terminals = if ($preferred | describe | str contains "list") {
        $preferred | where $it in $SUPPORTED_TERMINALS
    } else {
        let other_terminals = $SUPPORTED_TERMINALS | where $it != $preferred
        ([$preferred] | append $other_terminals)
    }
    if ($ordered_terminals | is-empty) {
        return null
    }

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

# Build a detached launch prefix for new terminal windows.
# This avoids inheriting the current Zellij client context during restart flows.
def build_detached_launch_prefix [needs_reload: bool]: nothing -> string {
    mut unset_vars = [
        "ZELLIJ"
        "ZELLIJ_SESSION_NAME"
        "ZELLIJ_PANE_ID"
        "ZELLIJ_TAB_NAME"
        "ZELLIJ_TAB_POSITION"
    ]
    if $needs_reload {
        $unset_vars = ($unset_vars | append "IN_YAZELIX_SHELL" | append "IN_NIX_SHELL")
    }

    let unset_flags = ($unset_vars | each {|name| $"-u ($name)"} | str join " ")
    let setsid_prefix = if (which setsid | is-not-empty) { "setsid " } else { "" }
    $"env ($unset_flags) ($setsid_prefix)"
}

def build_detached_background_command [prefix: string, command: string]: nothing -> string {
    $"nohup ($prefix)($command) >/dev/null 2>&1 < /dev/null &"
}

def get_working_dir_arg [terminal: string, working_dir: string]: nothing -> string {
    if ($working_dir | is-empty) {
        return ""
    }

    match $terminal {
        "ghostty" => $" --working-directory=\"($working_dir)\"",
        "wezterm" => $" --cwd \"($working_dir)\"",
        "kitty" => $" --directory=\"($working_dir)\"",
        "alacritty" => $" --working-directory \"($working_dir)\"",
        "foot" => $" --working-directory=\"($working_dir)\"",
        _ => ""
    }
}

# Build launch command for a terminal
export def build_launch_command [
    terminal_info: record
    config_path
    working_dir: string
    needs_reload: bool = true  # Whether to force environment reload
]: nothing -> string {
    let terminal = $terminal_info.terminal
    let command = $terminal_info.command
    let use_wrapper = $terminal_info.use_wrapper
    let launch_prefix = build_detached_launch_prefix $needs_reload
    let working_dir_arg = (get_working_dir_arg $terminal $working_dir)
    let startup_script = (get_startup_script_path)
    let startup_shell = $"sh -c 'exec ($startup_script)'"

    if $use_wrapper {
        # Wrappers handle config internally via environment variable
        build_detached_background_command $launch_prefix $"($command)($working_dir_arg)"
    } else {
        # Direct terminal launch with config
        # Check if nixGLIntel is available for GPU acceleration
        let nixgl_prefix = if (which nixGLIntel | is-not-empty) { "nixGLIntel " } else { "" }
        let terminal_cmd = match $terminal {
            "ghostty" => {
                $"($nixgl_prefix)ghostty --config-default-files=false --config-file=($config_path) --gtk-single-instance=false --title=\"Yazelix - Ghostty\"($working_dir_arg) -e ($startup_shell)"
            },
            "wezterm" => {
                $"($nixgl_prefix)wezterm --config-file ($config_path) start --class=com.yazelix.Yazelix($working_dir_arg) -- ($startup_shell)"
            },
            "kitty" => {
                $"($nixgl_prefix)kitty --config=($config_path) --class=com.yazelix.Yazelix --title=\"Yazelix - Kitty\"($working_dir_arg) ($startup_shell)"
            },
            "alacritty" => {
                $"($nixgl_prefix)alacritty --config-file ($config_path) --title \"Yazelix - Alacritty\"($working_dir_arg) -e ($startup_shell)"
            },
            "foot" => {
                $"($nixgl_prefix)foot --config ($config_path) --app-id com.yazelix.Yazelix($working_dir_arg) ($startup_shell)"
            },
            _ => {
                error make {msg: $"Unknown terminal: ($terminal)"}
            }
        }

        build_detached_background_command $launch_prefix $terminal_cmd
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
