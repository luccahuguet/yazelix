#!/usr/bin/env nu

const RUNTIME_PATHS_MODULE_PATH = (path self)
const INFERRED_RUNTIME_DIR = (
    $RUNTIME_PATHS_MODULE_PATH
    | path dirname
    | path join ".." ".." ".."
    | path expand
)

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

export def expand_user_path_string [value: string] {
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

export def get_yazelix_runtime_dir [] {
    let configured = (
        $env.YAZELIX_RUNTIME_DIR?
        | default ""
        | into string
        | str trim
    )
    if ($configured | is-not-empty) {
        let configured_path = (resolve_existing_path $configured)
        if (is_valid_runtime_dir $configured_path) {
            return $configured_path
        }
    }

    let inferred_runtime = (get_inferred_runtime_dir)
    if $inferred_runtime != null {
        return $inferred_runtime
    }

    null
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
