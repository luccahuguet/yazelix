#!/usr/bin/env nu

# Utility functions for Yazelix

export def get_yazelix_config_dir [] {
    let configured = (
        $env.YAZELIX_CONFIG_DIR?
        | default ""
        | into string
        | str trim
    )
    if ($configured | is-not-empty) {
        $configured | path expand
    } else if (($env.XDG_CONFIG_HOME? | default "" | into string | str trim) | is-not-empty) {
        ($env.XDG_CONFIG_HOME | path join "yazelix")
    } else if (($env.HOME? | default "" | into string | str trim) | is-not-empty) {
        ($env.HOME | path join ".config" "yazelix")
    } else {
        "~/.config/yazelix" | path expand
    }
}

export def get_yazelix_user_config_dir [config_root?: string] {
    let root = if $config_root == null {
        get_yazelix_config_dir
    } else {
        $config_root | path expand
    }
    ($root | path join "user_configs")
}

export def get_yazelix_runtime_dir [] {
    let configured = (
        $env.YAZELIX_RUNTIME_DIR?
        | default ($env.YAZELIX_DIR? | default "")
        | into string
        | str trim
    )
    if ($configured | is-not-empty) {
        $configured | path expand
    } else {
        get_yazelix_config_dir
    }
}

export def get_yazelix_state_dir [] {
    let configured = (
        $env.YAZELIX_STATE_DIR?
        | default ""
        | into string
        | str trim
    )
    if ($configured | is-not-empty) {
        $configured | path expand
    } else if (($env.XDG_DATA_HOME? | default "" | into string | str trim) | is-not-empty) {
        ($env.XDG_DATA_HOME | path join "yazelix")
    } else if (($env.HOME? | default "" | into string | str trim) | is-not-empty) {
        ($env.HOME | path join ".local" "share" "yazelix")
    } else {
        "~/.local/share/yazelix" | path expand
    }
}

export def get_yazelix_dir [] {
    get_yazelix_runtime_dir
}

export def require_yazelix_config_dir [] {
    let yazelix_dir = (get_yazelix_config_dir)
    if not ($yazelix_dir | path exists) {
        error make {msg: $"Cannot find Yazelix config directory at ($yazelix_dir)"}
    }
    $yazelix_dir
}

export def require_yazelix_runtime_dir [] {
    let yazelix_dir = (get_yazelix_runtime_dir)
    if not ($yazelix_dir | path exists) {
        error make {msg: $"Cannot find Yazelix runtime directory at ($yazelix_dir)"}
    }
    $yazelix_dir
}

export def require_yazelix_dir [] {
    require_yazelix_runtime_dir
}

def get_total_cores [] {
    let total_cores = (sys cpu | length)
    if $total_cores > 0 { $total_cores } else { 1 }
}

export def get_yazelix_nix_config [] {
    [
        "warn-dirty = false"
        "extra-substituters = https://cache.numtide.com"
        "extra-trusted-public-keys = niks3.numtide.com-1:DTx8wZduET09hRmMtKdQDxNNthLQETkc/yaX7M4qK0g="
    ] | str join "\n"
}

def parse_parallelism_setting [setting_value: string, default_value: string, kind: string] {
    let total_cores = get_total_cores
    let resolved_value = if ($setting_value | is-not-empty) {
        $setting_value
    } else {
        $default_value
    }

    match $resolved_value {
        "auto" => {
            if $kind != "max_jobs" {
                error make {msg: "Invalid build_cores value 'auto'. Allowed symbolic values: max, max_minus_one, half, quarter, or a positive integer."}
            }
            if $total_cores >= 4 { (($total_cores / 4) | math floor | into int) } else { 1 }
        }
        "max" => $total_cores,
        "max_minus_one" => (if $total_cores > 1 { $total_cores - 1 } else { 1 }),
        "half" => (if $total_cores >= 2 { (($total_cores / 2) | math floor | into int) } else { 1 }),
        "quarter" => (if $total_cores >= 4 { (($total_cores / 4) | math floor | into int) } else { 1 }),
        _ => {
            let parsed = (try { $resolved_value | into int } catch { null })
            if $parsed == null {
                if $kind == "max_jobs" {
                    error make {msg: $"Invalid max_jobs value '($resolved_value)'. Allowed symbolic values: auto, max, max_minus_one, half, quarter, or a positive integer."}
                } else {
                    error make {msg: $"Invalid build_cores value '($resolved_value)'. Allowed symbolic values: max, max_minus_one, half, quarter, or a positive integer."}
                }
            }
            if $parsed < 1 {
                error make {msg: $"Invalid ($kind) value '($resolved_value)'. Expected a positive integer."}
            }
            $parsed
        }
    }
}

export def get_max_jobs [max_jobs_config?: string] {
    parse_parallelism_setting ($max_jobs_config | default "") "half" "max_jobs"
}

# Get the number of CPU cores to use per build based on configuration
export def get_max_cores [build_cores_config?: string] {
    parse_parallelism_setting ($build_cores_config | default "") "2" "build_cores"
}

export def describe_build_parallelism [build_cores_config?: string, max_jobs_config?: string] {
    let resolved_build_cores = if ($build_cores_config | is-not-empty) {
        $build_cores_config
    } else {
        "2"
    }
    let resolved_max_jobs = if ($max_jobs_config | is-not-empty) {
        $max_jobs_config
    } else {
        "half"
    }
    let per_job_cores = (get_max_cores $resolved_build_cores)
    let max_jobs = (get_max_jobs $resolved_max_jobs)
    let total_budget = ($per_job_cores * $max_jobs)

    if ($resolved_build_cores == ($per_job_cores | into string)) and ($resolved_max_jobs == ($max_jobs | into string)) {
        $"($max_jobs) jobs x ($per_job_cores) cores/job \(~($total_budget) total\)"
    } else {
        $"($max_jobs) jobs x ($per_job_cores) cores/job \(~($total_budget) total, max_jobs=($resolved_max_jobs), build_cores=($resolved_build_cores)\)"
    }
}

# Check if Helix (hx or helix) is running in a Zellij pane based on client output
export def is_hx_running [list_clients_output: string] {
    let cmd = $list_clients_output | str trim | str downcase
    let parts = $cmd | split row " "
    let has_hx_paths = ($parts | any {|part| $part | str ends-with "/hx"})
    let has_helix_paths = ($parts | any {|part| $part | str ends-with "/helix"})
    let is_hx_cmd = ($parts | any {|part| $part == "hx"})
    let is_helix_cmd = ($parts | any {|part| $part == "helix"})

    $has_hx_paths or $has_helix_paths or $is_hx_cmd or $is_helix_cmd
}
