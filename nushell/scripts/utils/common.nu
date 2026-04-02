#!/usr/bin/env nu

# Utility functions for Yazelix

const COMMON_MODULE_PATH = (path self)
const INFERRED_RUNTIME_DIR = (
    $COMMON_MODULE_PATH
    | path dirname
    | path join ".." ".." ".."
    | path expand
)
const RUNTIME_PROJECT_ENTRIES = [
    "assets"
    "config_metadata"
    "configs"
    "docs"
    "nushell"
    "rust_plugins"
    "shells"
    "CHANGELOG.md"
    "devenv.lock"
    "devenv.nix"
    "devenv.yaml"
    "yazelix_default.toml"
    "yazelix_packs_default.toml"
]

def is_valid_runtime_dir [candidate?: string] {
    if $candidate == null {
        return false
    }

    let candidate_path = ($candidate | path expand)
    let sentinel = ($candidate_path | path join "yazelix_default.toml")
    ($candidate_path | path exists) and ($sentinel | path exists)
}

def get_inferred_runtime_dir [] {
    let candidate = ($INFERRED_RUNTIME_DIR | path expand)
    let sentinel = ($candidate | path join "yazelix_default.toml")
    if ($candidate | path exists) and ($sentinel | path exists) {
        $candidate
    } else {
        null
    }
}

def get_installed_runtime_dir [] {
    let candidate = (get_yazelix_state_dir | path join "runtime" "current")
    if (is_valid_runtime_dir $candidate) {
        $candidate | path expand
    } else {
        null
    }
}

export def get_installed_yazelix_runtime_dir [] {
    get_installed_runtime_dir
}

export def require_installed_yazelix_runtime_dir [] {
    let runtime_dir = (get_installed_runtime_dir)
    if $runtime_dir == null {
        error make {msg: $"Cannot find installed Yazelix runtime at ((get_yazelix_state_dir | path join 'runtime' 'current'))"}
    }
    $runtime_dir
}

def expand_user_path_string [value: string] {
    let trimmed = ($value | str trim)
    if ($trimmed | is-empty) {
        return $trimmed
    }

    let home_dir = ($env.HOME? | default "" | into string)
    let expanded_home = if ($home_dir | is-not-empty) and ($trimmed | str starts-with "$HOME/") {
        $trimmed | str replace "$HOME" $home_dir
    } else if ($home_dir | is-not-empty) and ($trimmed == "$HOME") {
        $home_dir
    } else {
        $trimmed
    }

    $expanded_home | path expand
}

export def resolve_external_command_path [command_name: string] {
    let matches = (which $command_name | where type == "external")
    if ($matches | is-empty) {
        null
    } else {
        $matches | get 0.path
    }
}

def normalize_command_candidate [candidate?: string] {
    if $candidate == null {
        return null
    }

    let trimmed = ($candidate | into string | str trim)
    if ($trimmed | is-empty) {
        return null
    }

    let expanded = (expand_user_path_string $trimmed)
    if ($expanded | path exists) {
        $expanded
    } else {
        resolve_external_command_path $trimmed
    }
}

export def get_yazelix_config_dir [] {
    let configured = (
        $env.YAZELIX_CONFIG_DIR?
        | default ""
        | into string
        | str trim
    )
    if ($configured | is-not-empty) {
        expand_user_path_string $configured
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
    let installed_runtime = (get_installed_runtime_dir)
    let configured_path = if ($configured | is-not-empty) { $configured | path expand } else { "" }

    if ($configured | is-not-empty) and ($configured_path | str starts-with "/nix/store/") and ($installed_runtime != null) and ($configured_path != $installed_runtime) {
        $installed_runtime
    } else if ($configured | is-not-empty) and (is_valid_runtime_dir $configured) {
        $configured | path expand
    } else if $installed_runtime != null {
        $installed_runtime
    } else if ((get_inferred_runtime_dir) != null) {
        get_inferred_runtime_dir
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
        expand_user_path_string $configured
    } else if (($env.XDG_DATA_HOME? | default "" | into string | str trim) | is-not-empty) {
        ($env.XDG_DATA_HOME | path join "yazelix")
    } else if (($env.HOME? | default "" | into string | str trim) | is-not-empty) {
        ($env.HOME | path join ".local" "share" "yazelix")
    } else {
        "~/.local/share/yazelix" | path expand
    }
}

export def get_yazelix_runtime_project_dir [] {
    (get_yazelix_state_dir | path join "runtime" "project")
}

export def get_yazelix_runtime_reference_dir [] {
    let state_runtime = (get_yazelix_state_dir | path join "runtime" "current")
    if (is_valid_runtime_dir $state_runtime) {
        $state_runtime
    } else {
        get_yazelix_runtime_dir
    }
}

export def resolve_yazelix_nu_bin [] {
    let explicit_nu = (normalize_command_candidate ($env.YAZELIX_NU_BIN? | default null))
    if $explicit_nu != null {
        return $explicit_nu
    }

    let runtime_nu = ((get_yazelix_runtime_reference_dir) | path join "bin" "nu")
    if ($runtime_nu | path exists) {
        return $runtime_nu
    }

    let path_nu = (resolve_external_command_path "nu")
    if $path_nu != null {
        return $path_nu
    }

    let current_nu = (normalize_command_candidate ($nu.current-exe? | default null))
    if $current_nu != null {
        return $current_nu
    }

    error make {msg: "Could not resolve a usable Nushell binary for Yazelix. Checked YAZELIX_NU_BIN, runtime-local bin/nu, PATH, and $nu.current-exe."}
}

export def ensure_yazelix_runtime_project_dir [] {
    let runtime_root = (get_yazelix_runtime_dir)
    let project_root = (get_yazelix_runtime_project_dir)

    mkdir $project_root

    for entry in $RUNTIME_PROJECT_ENTRIES {
        let source = ($runtime_root | path join $entry)
        if not ($source | path exists) {
            continue
        }

        let target = ($project_root | path join $entry)
        if ($target | path exists) {
            rm -rf $target
        }
        ^ln -s $source $target
    }

    $project_root
}

export def get_yazelix_dir [] {
    get_yazelix_runtime_dir
}

export def resolve_zellij_default_shell [yazelix_dir: string, default_shell: string] {
    let shell_name = ($default_shell | str downcase)
    if $shell_name == "nu" {
        ($yazelix_dir | path join "shells" "posix" "yazelix_nu.sh")
    } else {
        $default_shell
    }
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
