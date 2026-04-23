#!/usr/bin/env nu

use runtime_paths.nu [expand_user_path_string get_yazelix_runtime_dir]

export def normalize_path_entries [value: any] {
    let described = ($value | describe)

    if ($described | str starts-with "list") {
        $value | each {|entry| $entry | into string }
    } else {
        let text = ($value | into string | str trim)
        if ($text | is-empty) {
            []
        } else {
            $text | split row (char esep)
        }
    }
}

export def get_runtime_platform_name []: nothing -> string {
    (
        $env.YAZELIX_TEST_OS?
        | default $nu.os-info.name
        | into string
        | str trim
        | str downcase
    )
}

export def resolve_external_command_path [command_name: string] {
    let matches = (which $command_name | where type == "external")
    if ($matches | is-empty) {
        null
    } else {
        $matches | get -o 0.path
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

export def resolve_yazelix_nu_bin [] {
    let explicit_nu = (normalize_command_candidate ($env.YAZELIX_NU_BIN? | default null))
    if $explicit_nu != null {
        return $explicit_nu
    }

    let runtime_dir = (get_yazelix_runtime_dir)
    if $runtime_dir != null {
        let runtime_nu = ($runtime_dir | path join "libexec" "nu")
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

    error make {msg: "Could not resolve a usable Nushell binary for Yazelix. Checked YAZELIX_NU_BIN, runtime-local libexec/nu, PATH, and $nu.current-exe."}
}

export def resolve_zellij_default_shell [yazelix_dir: string, default_shell: string] {
    let shell_name = ($default_shell | str downcase)
    if $shell_name == "nu" {
        ($yazelix_dir | path join "shells" "posix" "yazelix_nu.sh")
    } else {
        $default_shell
    }
}
