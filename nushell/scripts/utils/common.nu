#!/usr/bin/env nu

# Utility functions for Yazelix

const COMMON_MODULE_PATH = (path self)
const INFERRED_RUNTIME_DIR = (
    $COMMON_MODULE_PATH
    | path dirname
    | path join ".." ".." ".."
    | path expand
)

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
        | default ""
        | into string
        | str trim
    )
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

export def resolve_yazelix_nu_bin [] {
    let explicit_nu = (normalize_command_candidate ($env.YAZELIX_NU_BIN? | default null))
    if $explicit_nu != null {
        return $explicit_nu
    }

    let runtime_dir = (get_yazelix_runtime_dir)
    if $runtime_dir != null {
        let runtime_nu = ($runtime_dir | path join "bin" "nu")
        if ($runtime_nu | path exists) {
            return $runtime_nu
        }
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

export def resolve_zellij_default_shell [yazelix_dir: string, default_shell: string] {
    let shell_name = ($default_shell | str downcase)
    if $shell_name == "nu" {
        ($yazelix_dir | path join "shells" "posix" "yazelix_nu.sh")
    } else {
        $default_shell
    }
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
