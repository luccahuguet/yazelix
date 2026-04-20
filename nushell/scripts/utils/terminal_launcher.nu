#!/usr/bin/env nu
# Terminal launcher utilities for Yazelix

use constants.nu [SUPPORTED_TERMINALS, TERMINAL_CONFIG_PATHS, TERMINAL_METADATA, YAZELIX_WINDOW_CLASS, YAZELIX_X11_INSTANCE]
use common.nu [get_yazelix_runtime_dir get_yazelix_state_dir get_runtime_platform_name]
use startup_profile.nu [profile_startup_step]

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

def get_runtime_nixgl_wrapper_candidates [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    if $runtime_dir == null {
        return []
    }

    [
        {command: "nixGL", path: ($runtime_dir | path join "libexec" "nixGL")}
        {command: "nixGLDefault", path: ($runtime_dir | path join "libexec" "nixGLDefault")}
        {command: "nixGLMesa", path: ($runtime_dir | path join "libexec" "nixGLMesa")}
        {command: "nixGLIntel", path: ($runtime_dir | path join "libexec" "nixGLIntel")}
        {command: "nixGLMesa", path: ($runtime_dir | path join "bin" "nixGLMesa")}
        {command: "nixGLIntel", path: ($runtime_dir | path join "bin" "nixGLIntel")}
    ]
}

export def resolve_nixgl_launch_context [] {
    for candidate in (get_runtime_nixgl_wrapper_candidates) {
        if ($candidate.path | path exists) {
            return {
                source: "runtime"
                command: $candidate.command
                path: $candidate.path
                prefix: $"($candidate.path) "
            }
        }
    }

    for command_name in ["nixGL" "nixGLDefault" "nixGLMesa" "nixGLIntel"] {
        if (which $command_name | is-not-empty) {
            return {
                source: "host_path"
                command: $command_name
                path: $command_name
                prefix: $"($command_name) "
            }
        }
    }

    {
        source: "none"
        command: null
        path: null
        prefix: ""
    }
}

def resolve_nixgl_launch_prefix [] {
    (resolve_nixgl_launch_context).prefix
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

export def detect_terminal_candidates [preferred: any] {
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

    mut available = []
    for terminal in $ordered_terminals {
        let term_meta = ($TERMINAL_METADATA | get -o $terminal | default {})
        if (command_exists $terminal) {
            $available = ($available | append {
                terminal: $terminal
                name: $term_meta.name
                command: $terminal
            })
        }
    }

    $available
}

# Detect first available terminal.
export def detect_terminal [preferred: any] {
    let candidates = (detect_terminal_candidates $preferred)
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

def get_ghostty_env_wrapper_path []: nothing -> string {
    let runtime_dir = (get_yazelix_runtime_dir)
    $runtime_dir | path join "shells" "posix" "yazelix_ghostty.sh"
}

def build_ghostty_launch_command [
    command: string
    config_path: string
    title: string
    working_dir_arg: string
    startup_shell: string
]: nothing -> string {
    let platform_name = (get_runtime_platform_name)

    if $platform_name == "macos" {
        return $"($command) --config-default-files=false --config-file=($config_path) --title=\"($title)\"($working_dir_arg) -e ($startup_shell)"
    }

    let nixgl_prefix = (resolve_nixgl_launch_prefix)
    let ghostty_env_wrapper = (quote_for_bash_single_string (get_ghostty_env_wrapper_path))
    $"($ghostty_env_wrapper) ($nixgl_prefix)($command) --config-default-files=false --config-file=($config_path) --gtk-single-instance=false --class=\"($YAZELIX_WINDOW_CLASS)\" --x11-instance-name=\"($YAZELIX_X11_INSTANCE)\" --title=\"($title)\"($working_dir_arg) -e ($startup_shell)"
}

# Build launch command for a terminal. The returned command is a foreground
# terminal exec; detached/background handling is applied by run_detached_terminal_launch.
export def build_launch_command [
    terminal_info: record
    config_path
    working_dir: string
    needs_reload: bool = true  # Whether to force environment reload
]: nothing -> string {
    let terminal = $terminal_info.terminal
    let command = $terminal_info.command
    let launch_prefix = build_detached_launch_prefix $needs_reload
    let working_dir_arg = (get_working_dir_arg $terminal $working_dir)
    let startup_script = (get_startup_script_path)
    let startup_shell = $"sh -c 'exec ($startup_script)'"
    let title = (get_terminal_title $terminal)

    # Prefer the generic nixGL wrapper when available. Fall back to the
    # older Intel-specific name only if the default wrapper is absent.
    let nixgl_prefix = (resolve_nixgl_launch_prefix)
    let terminal_cmd = match $terminal {
        "ghostty" => {
            build_ghostty_launch_command $command $config_path $title $working_dir_arg $startup_shell
        },
        "wezterm" => {
            $"($nixgl_prefix)($command) --config-file ($config_path) start --class=($YAZELIX_WINDOW_CLASS)($working_dir_arg) -- ($startup_shell)"
        },
        "kitty" => {
            $"($nixgl_prefix)($command) --config=($config_path) --class=($YAZELIX_WINDOW_CLASS) --title=\"($title)\"($working_dir_arg) ($startup_shell)"
        },
        "alacritty" => {
            $"($nixgl_prefix)($command) --config-file ($config_path) --class \"($YAZELIX_WINDOW_CLASS)\" --title \"($title)\"($working_dir_arg) -e ($startup_shell)"
        },
        "foot" => {
            $"($nixgl_prefix)($command) --config ($config_path) --app-id ($YAZELIX_WINDOW_CLASS)($working_dir_arg) ($startup_shell)"
        },
        _ => {
            error make {msg: $"Unknown terminal: ($terminal)"}
        }
    }

    $"($launch_prefix)($terminal_cmd)"
}

# Get display name for terminal
export def get_terminal_display_name [terminal_info: record]: nothing -> string {
    $terminal_info.name
}

export def run_detached_terminal_launch [launch_cmd: string, terminal_name: string, --verbose] {
    if (which bash | is-empty) {
        error make {msg: $"Cannot launch ($terminal_name): bash is not available in PATH.\nYazelix uses bash to detach new terminal windows."}
    }

    let launch_log = (get_launch_probe_log_path $terminal_name)
    let probe_script = '
launch_log="$1"
launch_cmd="$2"

: > "$launch_log"
nohup bash -lc "$launch_cmd" >"$launch_log" 2>&1 < /dev/null &
pid=$!

for i in 1 2 3 4 5 6; do
  sleep 0.05
  if ! kill -0 "$pid" 2>/dev/null; then
    wait "$pid"
    status=$?
    echo "$launch_log"
    exit "$status"
  fi
done

rm -f "$launch_log"
exit 0
'

    let output = (profile_startup_step "terminal_launcher" "detached_launch_probe" {
        ^bash -c $probe_script bash $launch_log $launch_cmd | complete
    } {
        terminal: $terminal_name
    })
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
