#!/usr/bin/env nu
# Terminal launcher utilities for Yazelix

use constants.nu [TERMINAL_CONFIG_PATHS, YAZELIX_WINDOW_CLASS, YAZELIX_X11_INSTANCE]
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

def get_terminal_title [terminal_info: record] {
    let display_name = ($terminal_info.name? | default $terminal_info.terminal)
    $"Yazelix - ($display_name)"
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

def resolve_nixgl_launch_context [] {
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

def render_shell_arg [value: string] {
    if ($value | is-empty) {
        return "''"
    }

    if (($value | str replace -ra '[A-Za-z0-9_./:=,@+-]' '') | is-empty) {
        return $value
    }

    if ($value | str contains "'") {
        '"' + ($value | str replace -a '"' '\"') + '"'
    } else {
        "'" + $value + "'"
    }
}

export def render_launch_command_argv [launch_argv: list<string>] {
    $launch_argv | each {|arg| render_shell_arg $arg } | str join " "
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

def get_working_dir_args [terminal: string, working_dir: string]: nothing -> list<string> {
    if ($working_dir | is-empty) {
        return []
    }

    match $terminal {
        "ghostty" => [$"--working-directory=($working_dir)"],
        "wezterm" => ["--cwd", $working_dir],
        "kitty" => [$"--directory=($working_dir)"],
        "alacritty" => ["--working-directory", $working_dir],
        "foot" => [$"--working-directory=($working_dir)"],
        _ => []
    }
}

def get_ghostty_env_wrapper_path []: nothing -> string {
    let runtime_dir = (get_yazelix_runtime_dir)
    $runtime_dir | path join "shells" "posix" "yazelix_ghostty.sh"
}

def get_detached_launch_probe_helper_path []: nothing -> string {
    let runtime_dir = (get_yazelix_runtime_dir)
    $runtime_dir | path join "shells" "posix" "detached_launch_probe.sh"
}

def get_startup_command_argv []: nothing -> list<string> {
    let startup_script = (get_startup_script_path)
    ["sh", "-c", $"exec ($startup_script)"]
}

def prepend_launch_wrapper [argv: list<string>, wrapper: string] {
    if ($wrapper | str trim | is-empty) {
        $argv
    } else {
        [$wrapper] | append $argv
    }
}

def build_ghostty_launch_argv [
    command: string
    config_path: string
    title: string
    working_dir_args: list<string>
    startup_argv: list<string>
]: nothing -> list<string> {
    let platform_name = (get_runtime_platform_name)

    if $platform_name == "macos" {
        return (
            [$command, "--config-default-files=false", $"--config-file=($config_path)", $"--title=($title)"]
            | append $working_dir_args
            | append ["-e"]
            | append $startup_argv
        )
    }

    let ghostty_argv = (
        [$command, "--config-default-files=false", $"--config-file=($config_path)", "--gtk-single-instance=false", $"--class=($YAZELIX_WINDOW_CLASS)", $"--x11-instance-name=($YAZELIX_X11_INSTANCE)", $"--title=($title)"]
        | append $working_dir_args
        | append ["-e"]
        | append $startup_argv
    )
    let nixgl_wrapper = ((resolve_nixgl_launch_context).path? | default "")
    prepend_launch_wrapper (prepend_launch_wrapper $ghostty_argv $nixgl_wrapper) (get_ghostty_env_wrapper_path)
}

export def build_launch_command_argv [
    terminal_info: record
    config_path
    working_dir: string
    needs_reload: bool = true  # Whether to force environment reload
]: nothing -> list<string> {
    let terminal = $terminal_info.terminal
    let command = $terminal_info.command
    let working_dir_args = (get_working_dir_args $terminal $working_dir)
    let startup_argv = (get_startup_command_argv)
    let title = (get_terminal_title $terminal_info)

    let nixgl_wrapper = ((resolve_nixgl_launch_context).path? | default "")
    let terminal_argv = match $terminal {
        "ghostty" => {
            build_ghostty_launch_argv $command $config_path $title $working_dir_args $startup_argv
        },
        "wezterm" => {
            prepend_launch_wrapper (
                [$command, "--config-file", $config_path, "start", $"--class=($YAZELIX_WINDOW_CLASS)"]
                | append $working_dir_args
                | append ["--"]
                | append $startup_argv
            ) $nixgl_wrapper
        },
        "kitty" => {
            prepend_launch_wrapper (
                [$command, $"--config=($config_path)", $"--class=($YAZELIX_WINDOW_CLASS)", $"--title=($title)"]
                | append $working_dir_args
                | append $startup_argv
            ) $nixgl_wrapper
        },
        "alacritty" => {
            prepend_launch_wrapper (
                [$command, "--config-file", $config_path, "--class", $YAZELIX_WINDOW_CLASS, "--title", $title]
                | append $working_dir_args
                | append ["-e"]
                | append $startup_argv
            ) $nixgl_wrapper
        },
        "foot" => {
            prepend_launch_wrapper (
                [$command, "--config", $config_path, "--app-id", $YAZELIX_WINDOW_CLASS]
                | append $working_dir_args
                | append $startup_argv
            ) $nixgl_wrapper
        },
        _ => {
            error make {msg: $"Unknown terminal: ($terminal)"}
        }
    }

    $terminal_argv
}

# Build launch command for display and tests. Detached/background handling is applied by run_detached_terminal_launch.
export def build_launch_command [
    terminal_info: record
    config_path
    working_dir: string
    needs_reload: bool = true
]: nothing -> string {
    render_launch_command_argv (build_launch_command_argv $terminal_info $config_path $working_dir $needs_reload)
}

export def run_detached_terminal_launch [launch_argv: list<string>, terminal_name: string, needs_reload: bool = true, --verbose] {
    let probe_helper = (get_detached_launch_probe_helper_path)
    if not ($probe_helper | path exists) {
        error make {msg: $"Cannot launch ($terminal_name): detached launch helper is missing at ($probe_helper).\nRestore shells/posix/detached_launch_probe.sh or reinstall Yazelix."}
    }

    let launch_log = (get_launch_probe_log_path $terminal_name)
    let helper_args = if $needs_reload {
        [$launch_log, "--reload", "--"] | append $launch_argv
    } else {
        [$launch_log, "--"] | append $launch_argv
    }
    let output = (profile_startup_step "terminal_launcher" "detached_launch_probe" {
        ^$probe_helper ...$helper_args | complete
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
