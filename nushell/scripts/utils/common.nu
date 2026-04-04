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
    ".taplo.toml"
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

def is_valid_repo_root [candidate?: string] {
    if $candidate == null {
        return false
    }

    let candidate_path = (resolve_existing_path $candidate)
    if $candidate_path == null {
        return false
    }

    let git_marker = ($candidate_path | path join ".git")
    let devenv_nix = ($candidate_path | path join "devenv.nix")
    let default_config = ($candidate_path | path join "yazelix_default.toml")

    ($git_marker | path exists) and ($devenv_nix | path exists) and ($default_config | path exists)
}

def is_valid_runtime_dir [candidate?: string] {
    if $candidate == null {
        return false
    }

    let candidate_path = (resolve_existing_path $candidate)
    if $candidate_path == null {
        return false
    }
    let sentinel = ($candidate_path | path join "yazelix_default.toml")
    ($candidate_path | path exists) and ($sentinel | path exists)
}

def resolve_existing_path [candidate?: string] {
    if $candidate == null {
        return null
    }

    let expanded = ($candidate | path expand)
    if not ($expanded | path exists) {
        return null
    }

    try {
        let result = (^readlink -f $expanded | complete)
        if $result.exit_code == 0 {
            let resolved = ($result.stdout | str trim)
            if ($resolved | is-not-empty) and ($resolved | path exists) {
                return $resolved
            }
        }
    } catch {}

    $expanded
}

def get_inferred_runtime_dir [] {
    let candidate = (resolve_existing_path $INFERRED_RUNTIME_DIR)
    if $candidate == null {
        return null
    }
    let sentinel = ($candidate | path join "yazelix_default.toml")
    if ($candidate | path exists) and ($sentinel | path exists) {
        $candidate
    } else {
        null
    }
}

def is_nix_store_source_runtime [candidate?: string] {
    if $candidate == null {
        return false
    }

    let normalized = ($candidate | into string)
    ($normalized | str starts-with "/nix/store/") and ($normalized | str ends-with "-source")
}

def get_installed_runtime_dir [] {
    let candidate = (get_yazelix_state_dir | path join "runtime" "current")
    if (is_valid_runtime_dir $candidate) {
        resolve_existing_path $candidate
    } else {
        null
    }
}

export def get_installed_yazelix_runtime_reference_dir [] {
    get_yazelix_state_dir | path join "runtime" "current"
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
    let inferred_runtime = (get_inferred_runtime_dir)
    let configured_path = if ($configured | is-not-empty) {
        resolve_existing_path $configured
    } else {
        null
    }
    if ($configured | is-not-empty) and ($configured_path != null) and (is_valid_runtime_dir $configured_path) {
        $configured_path
    } else if ($inferred_runtime != null) and (not (is_nix_store_source_runtime $inferred_runtime)) {
        $inferred_runtime
    } else if $installed_runtime != null {
        $installed_runtime
    } else if $inferred_runtime != null {
        $inferred_runtime
    } else {
        null
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

def runtime_project_matches_runtime_root [project_root: string, runtime_root: string] {
    let project_sentinel = ($project_root | path join "devenv.nix")
    let runtime_sentinel = ($runtime_root | path join "devenv.nix")

    if not ($project_sentinel | path exists) or not ($runtime_sentinel | path exists) {
        return false
    }

    let resolved_project_sentinel = (resolve_existing_path $project_sentinel)
    let resolved_runtime_sentinel = (resolve_existing_path $runtime_sentinel)

    if ($resolved_project_sentinel == null) or ($resolved_runtime_sentinel == null) {
        return false
    }

    $resolved_project_sentinel == $resolved_runtime_sentinel
}

export def get_existing_yazelix_runtime_project_dir [] {
    let runtime_root = (get_yazelix_runtime_dir)
    if $runtime_root == null {
        return null
    }

    let project_root = (get_yazelix_runtime_project_dir)
    if (
        ($project_root | path exists)
        and (($project_root | path type) == "dir")
        and (runtime_project_matches_runtime_root $project_root $runtime_root)
    ) {
        $project_root
    } else {
        null
    }
}

def resolve_git_repo_root_from_pwd [] {
    let pwd = ($env.PWD? | default "" | into string | str trim)
    if ($pwd | is-empty) {
        return null
    }

    try {
        let result = (^git -C $pwd rev-parse --show-toplevel | complete)
        if $result.exit_code != 0 {
            return null
        }

        let candidate = ($result.stdout | str trim)
        if (is_valid_repo_root $candidate) {
            resolve_existing_path $candidate
        } else {
            null
        }
    } catch {
        null
    }
}

export def get_yazelix_repo_root [] {
    let raw_devenv_root = ($env.DEVENV_ROOT? | default null)
    let devenv_root = if $raw_devenv_root == null {
        null
    } else {
        resolve_existing_path ($raw_devenv_root | into string | str trim)
    }
    if (is_valid_repo_root $devenv_root) {
        return $devenv_root
    }

    let pwd_repo_root = (resolve_git_repo_root_from_pwd)
    if $pwd_repo_root != null {
        return $pwd_repo_root
    }

    let inferred_runtime = (get_inferred_runtime_dir)
    if (is_valid_repo_root $inferred_runtime) {
        return $inferred_runtime
    }

    null
}

export def require_yazelix_repo_root [] {
    let repo_root = (get_yazelix_repo_root)
    if $repo_root == null {
        error make {msg: "This maintainer command requires a writable Yazelix repo checkout. Run it from the repo root or a repo-local devenv shell."}
    }

    $repo_root
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

export def materialize_yazelix_runtime_project_dir [] {
    let runtime_root = (require_yazelix_runtime_dir)
    let project_root = (get_yazelix_runtime_project_dir)

    mkdir $project_root

    for entry in $RUNTIME_PROJECT_ENTRIES {
        let source = ($runtime_root | path join $entry)
        let target = ($project_root | path join $entry)
        if ($target | path exists) {
            rm -rf $target
        }
        if not ($source | path exists) {
            continue
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
    if $yazelix_dir == null {
        error make {
            msg: (
                "Could not resolve a valid Yazelix runtime root. "
                + "Set YAZELIX_RUNTIME_DIR to a real runtime tree, reinstall Yazelix, "
                + "or enter a valid repo/runtime environment."
            )
        }
    }
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
