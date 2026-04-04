#!/usr/bin/env nu
# Terminal launcher utilities for Yazelix

use constants.nu [SUPPORTED_TERMINALS, TERMINAL_CONFIG_PATHS, TERMINAL_METADATA, YAZELIX_WINDOW_CLASS, YAZELIX_X11_INSTANCE]
use common.nu [get_yazelix_runtime_dir get_yazelix_state_dir]

# Check if a command is available
export def command_exists [cmd: string]: nothing -> bool {
    (which $cmd | length) > 0
}

def get_startup_script_path []: nothing -> string {
    let runtime_dir = (get_yazelix_runtime_dir)
    $runtime_dir | path join "shells" "posix" "start_yazelix.sh"
}

def get_terminal_title [terminal: string] {
    $"Yazelix - (($TERMINAL_METADATA | get -o $terminal | default {} | get -o name | default $terminal))"
}

def get_profile_bin_dir [profile: string] {
    if ($profile | is-empty) {
        return ""
    }

    let bin_dir = ($profile | path join "bin")
    if ($bin_dir | path exists) {
        $bin_dir
    } else {
        ""
    }
}

def get_current_profile_bin_dir [] {
    let profile = ($env.DEVENV_PROFILE? | default "" | into string | str trim)
    get_profile_bin_dir $profile
}

def resolve_nixgl_launch_prefix [] {
    let runtime_nixgl = ((get_yazelix_runtime_dir) | path join "bin" "nixGL")
    if ($runtime_nixgl | path exists) {
        return $"($runtime_nixgl) "
    }

    let runtime_nixgl_default = ((get_yazelix_runtime_dir) | path join "bin" "nixGLDefault")
    if ($runtime_nixgl_default | path exists) {
        return $"($runtime_nixgl_default) "
    }

    let profile_bin_dir = (get_current_profile_bin_dir)
    if ($profile_bin_dir | is-not-empty) {
        let profile_nixgl = ($profile_bin_dir | path join "nixGL")
        if ($profile_nixgl | path exists) {
            return $"($profile_nixgl) "
        }

        let profile_nixgl_default = ($profile_bin_dir | path join "nixGLDefault")
        if ($profile_nixgl_default | path exists) {
            return $"($profile_nixgl_default) "
        }

        let profile_nixgl_intel = ($profile_bin_dir | path join "nixGLIntel")
        if ($profile_nixgl_intel | path exists) {
            return $"($profile_nixgl_intel) "
        }
    }

    if (which nixGL | is-not-empty) {
        return "nixGL "
    }

    if (which nixGLDefault | is-not-empty) {
        return "nixGLDefault "
    }

    if (which nixGLIntel | is-not-empty) {
        return "nixGLIntel "
    }

    ""
}

# Resolve config path for a terminal based on mode
export def resolve_terminal_config [terminal: string, mode: string] {
    let home = $env.HOME
    let config_paths = ($TERMINAL_CONFIG_PATHS | get -o $terminal)
    if $config_paths == null {
        error make {msg: $"Unsupported terminal config lookup: ($terminal)"}
    }

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

# Compatibility shim for older generated terminal wrappers that still read the
# config mode from ambient env instead of using the wrapper's baked default.
export def resolve_terminal_config_from_env [terminal: string] {
    let mode = (
        $env.YAZELIX_TERMINAL_CONFIG_MODE?
        | default "yazelix"
        | into string
        | str trim
    )
    resolve_terminal_config $terminal $mode
}

export def detect_terminal_candidates [preferred: any, prefer_wrappers: bool = true] {
    # Build list of terminals to check: use list order if provided, otherwise preferred first
    let ordered_terminals = if ($preferred | describe | str contains "list") {
        $preferred | where $it in $SUPPORTED_TERMINALS
    } else {
        let other_terminals = $SUPPORTED_TERMINALS | where $it != $preferred
        ([$preferred] | append $other_terminals)
    }
    if ($ordered_terminals | is-empty) {
        return []
    }

    let terminals_to_check = (
        $ordered_terminals
        | each {|t|
            if $prefer_wrappers {
                [
                    {terminal: $t, use_wrapper: true}
                    {terminal: $t, use_wrapper: false}
                ]
            } else {
                [{terminal: $t, use_wrapper: false}]
            }
        }
        | flatten
    )

    mut available = []
    for term_check in $terminals_to_check {
        let terminal = $term_check.terminal
        let use_wrapper = $term_check.use_wrapper
        let term_meta = ($TERMINAL_METADATA | get -o $terminal | default {})

        let command = if $use_wrapper { $term_meta.wrapper } else { $terminal }

        if (command_exists $command) {
            $available = ($available | append {
                terminal: $terminal
                name: $term_meta.name
                command: $command
                use_wrapper: $use_wrapper
            })
        }
    }

    $available
}

export def detect_terminal_wrapper_candidates_from_profile [preferred: any, profile_path: string] {
    let profile_bin_dir = (get_profile_bin_dir $profile_path)
    if ($profile_bin_dir | is-empty) {
        return []
    }

    let ordered_terminals = if ($preferred | describe | str contains "list") {
        $preferred | where $it in $SUPPORTED_TERMINALS
    } else {
        let other_terminals = $SUPPORTED_TERMINALS | where $it != $preferred
        ([$preferred] | append $other_terminals)
    }
    if ($ordered_terminals | is-empty) {
        return []
    }

    $ordered_terminals
    | each {|terminal|
        let term_meta = ($TERMINAL_METADATA | get -o $terminal | default {})
        let wrapper = ($term_meta.wrapper? | default "")
        let wrapper_path = if ($wrapper | is-not-empty) {
            $profile_bin_dir | path join $wrapper
        } else {
            ""
        }
        if ($wrapper_path | is-not-empty) and ($wrapper_path | path exists) {
            {
                terminal: $terminal
                name: $term_meta.name
                command: $wrapper_path
                use_wrapper: true
            }
        } else {
            null
        }
    }
    | compact
}

export def detect_terminal_wrapper_candidates [preferred: any] {
    let current_profile = ($env.DEVENV_PROFILE? | default "" | into string | str trim)
    detect_terminal_wrapper_candidates_from_profile $preferred $current_profile
}

# Detect first available terminal (wrapper or direct)
export def detect_terminal [preferred: any, prefer_wrappers: bool = true] {
    let candidates = (detect_terminal_candidates $preferred $prefer_wrappers)
    if ($candidates | is-empty) {
        null
    } else {
        $candidates | first
    }
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

def quote_for_bash_single_string [value: string] {
    "'" + ($value | str replace -a "'" "'\"'\"'") + "'"
}

def get_launch_probe_log_path [terminal_name: string] {
    let timestamp = (date now | format date "%Y%m%d_%H%M%S_%3f")
    let sanitized_terminal = (
        $terminal_name
        | str downcase
        | str replace -ra '[^a-z0-9]+' "_"
        | str trim -c "_"
    )
    let log_dir = (get_yazelix_state_dir | path join "logs" "terminal_launch")
    mkdir $log_dir
    ($log_dir | path join $"($sanitized_terminal)_($timestamp).log")
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

# Build launch command for a terminal. The returned command is a foreground
# terminal exec; detached/background handling is applied by run_detached_terminal_launch.
export def build_launch_command [
    terminal_info: record
    config_path
    terminal_config_mode: string
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
    let title = (get_terminal_title $terminal)

    if $use_wrapper {
        # Managed wrapper binaries already bake the canonical terminal binary,
        # config mode, startup script, and nixGL path.
        $"($launch_prefix)($command)($working_dir_arg)"
    } else {
        # Direct terminal launch with config
        # Prefer the generic nixGL wrapper when available. Fall back to the
        # older Intel-specific name only if the default wrapper is absent.
        let nixgl_prefix = (resolve_nixgl_launch_prefix)
        let terminal_cmd = match $terminal {
            "ghostty" => {
                $"($nixgl_prefix)ghostty --config-default-files=false --config-file=($config_path) --gtk-single-instance=false --class=\"($YAZELIX_WINDOW_CLASS)\" --x11-instance-name=\"($YAZELIX_X11_INSTANCE)\" --title=\"($title)\"($working_dir_arg) -e ($startup_shell)"
            },
            "wezterm" => {
                $"($nixgl_prefix)wezterm --config-file ($config_path) start --class=($YAZELIX_WINDOW_CLASS)($working_dir_arg) -- ($startup_shell)"
            },
            "kitty" => {
                $"($nixgl_prefix)kitty --config=($config_path) --class=($YAZELIX_WINDOW_CLASS) --title=\"($title)\"($working_dir_arg) ($startup_shell)"
            },
            "alacritty" => {
                $"($nixgl_prefix)alacritty --config-file ($config_path) --class \"($YAZELIX_WINDOW_CLASS)\" --title \"($title)\"($working_dir_arg) -e ($startup_shell)"
            },
            "foot" => {
                $"($nixgl_prefix)foot --config ($config_path) --app-id ($YAZELIX_WINDOW_CLASS)($working_dir_arg) ($startup_shell)"
            },
            _ => {
                error make {msg: $"Unknown terminal: ($terminal)"}
            }
        }

        $"($launch_prefix)($terminal_cmd)"
    }
}

# Get display name for terminal
export def get_terminal_display_name [terminal_info: record]: nothing -> string {
    let name = $terminal_info.name
    if $terminal_info.use_wrapper {
        $"Yazelix - ($name)"
    } else {
        $"($name)"
    }
}

export def run_detached_terminal_launch [launch_cmd: string, terminal_name: string, --verbose] {
    if (which bash | is-empty) {
        error make {msg: $"Cannot launch ($terminal_name): bash is not available in PATH.\nYazelix uses bash to detach new terminal windows."}
    }

    let launch_log = (get_launch_probe_log_path $terminal_name)
    let quoted_launch_cmd = (quote_for_bash_single_string $launch_cmd)
    let quoted_launch_log = (quote_for_bash_single_string $launch_log)
    let probe_script = [
        $"launch_log=($quoted_launch_log)"
        ': > "$launch_log"'
        $"nohup bash -lc ($quoted_launch_cmd) >\"$launch_log\" 2>&1 < /dev/null &"
        'pid=$!'
        'sleep 1'
        'if kill -0 "$pid" 2>/dev/null; then'
        '  rm -f "$launch_log"'
        '  exit 0'
        'fi'
        'wait "$pid"'
        'status=$?'
        'echo "$launch_log"'
        'exit "$status"'
    ] | str join "\n"

    let output = (^bash -lc $probe_script | complete)
    if $output.exit_code != 0 {
        let logged_path = ($output.stdout | lines | last | default "" | str trim)
        let log_tail = if ($logged_path | is-not-empty) and ($logged_path | path exists) {
            open --raw $logged_path
            | default ""
            | lines
            | last 10
            | str join "\n"
            | str trim
        } else {
            ""
        }
        let details = if ($log_tail | is-empty) {
            "No terminal stderr was captured."
        } else {
            $log_tail
        }
        error make {msg: $"Failed to launch ($terminal_name) \(exit code: ($output.exit_code)\)\n($details)"}
    }

    if $verbose {
        print $"✅ Launch request sent to ($terminal_name)"
    }
}
