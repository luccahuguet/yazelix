#!/usr/bin/env nu

export const CLEAN_ZELLIJ_ENV_PREFIX = "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID -u ZELLIJ_TAB_NAME -u ZELLIJ_TAB_POSITION"

export def get_repo_root [] {
    pwd
}

export def get_repo_config_dir [] {
    get_repo_root
}

export def repo_path [...parts: string] {
    $parts | reduce -f (get_repo_root) {|part, acc| $acc | path join $part }
}

export def resolve_test_yzx_control_bin [] {
    let explicit = ($env.YAZELIX_YZX_CONTROL_BIN? | default "" | into string | str trim)
    if ($explicit | is-not-empty) and (($explicit | path expand) | path exists) {
        return ($explicit | path expand)
    }

    for candidate in [
        (repo_path "rust_core" "target" "release" "yzx_control")
        (repo_path "rust_core" "target" "debug" "yzx_control")
    ] {
        if ($candidate | path exists) {
            return $candidate
        }
    }

    error make {
        msg: "Yazelix tests need a built yzx_control binary. Enter the maintainer shell or set YAZELIX_YZX_CONTROL_BIN."
    }
}

export def resolve_test_yzx_bin [] {
    let explicit = ($env.YAZELIX_YZX_BIN? | default "" | into string | str trim)
    if ($explicit | is-not-empty) and (($explicit | path expand) | path exists) {
        return ($explicit | path expand)
    }

    for candidate in [
        (repo_path "rust_core" "target" "release" "yzx")
        (repo_path "rust_core" "target" "debug" "yzx")
    ] {
        if ($candidate | path exists) {
            return $candidate
        }
    }

    error make {
        msg: "Yazelix tests need a built Rust yzx root helper. Enter the maintainer shell or set YAZELIX_YZX_BIN."
    }
}

export def resolve_test_yzx_core_bin [] {
    let explicit = ($env.YAZELIX_YZX_CORE_BIN? | default "" | into string | str trim)
    if ($explicit | is-not-empty) and (($explicit | path expand) | path exists) {
        return ($explicit | path expand)
    }

    for candidate in [
        (repo_path "rust_core" "target" "release" "yzx_core")
        (repo_path "rust_core" "target" "debug" "yzx_core")
    ] {
        if ($candidate | path exists) {
            return $candidate
        }
    }

    error make {
        msg: "Yazelix tests need a built yzx_core helper. Enter the maintainer shell or set YAZELIX_YZX_CORE_BIN to a built yzx_core binary."
    }
}

export def setup_test_home [] {
    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_test_home_XXXXXX | str trim)
    let config_parent = ($tmp_home | path join ".config")
    let config_dir = ($config_parent | path join "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")

    mkdir $config_parent
    mkdir $config_dir
    mkdir $user_config_dir

    for entry in (
        ls $repo_root
        | where name != ($repo_root | path join ".git")
        | where name != ($repo_root | path join "user_configs")
        | where name != ($repo_root | path join "yazelix.toml")
        | where name != ($repo_root | path join "yazelix_packs.toml")
    ) {
        let name = ($entry.name | path basename)
        ^ln -s $entry.name ($config_dir | path join $name)
    }
    ^ln -s ($repo_root | path join ".taplo.toml") ($config_dir | path join ".taplo.toml")

    cp ($repo_root | path join "yazelix_default.toml") ($user_config_dir | path join "yazelix.toml")

    {
        repo_root: $repo_root
        tmp_home: $tmp_home
        config_dir: $config_dir
    }
}

export def test_profiling_enabled [--profile] {
    if $profile {
        return true
    }

    let raw_value = ($env.YAZELIX_TEST_PROFILE? | default "false" | into string | str downcase | str trim)
    $raw_value in ["1", "true", "yes", "on"]
}

export def format_test_profile_report [records: list<record>, title: string] {
    let sorted = ($records | sort-by elapsed_ms --reverse)
    let lines = (
        $sorted
        | each {|record|
            let seconds = (($record.elapsed_ms | into float) / 1000.0 | into string | str substring 0..4)
            $"  - ($record.name): ($seconds)s"
        }
    )

    [
        $title
        ...$lines
    ] | str join "\n"
}

export def setup_managed_config_fixture [
    label: string
    raw_toml: string
] {
    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d $"/tmp/($label)_XXXXXX" | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")

    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir

    let config_path = ($user_config_dir | path join "yazelix.toml")

    $raw_toml | save --force --raw $config_path

    {
        repo_root: $repo_root
        tmp_home: $tmp_home
        config_dir: $config_dir
        user_config_dir: $user_config_dir
        config_path: $config_path
        yzx_script: ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
    }
}

export def add_fixture_log [fixture: record, log_file_name: string] {
    let log_file = ($fixture.tmp_home | path join $log_file_name)
    "" | save --force --raw $log_file
    $fixture | merge { log_file: $log_file }
}

export def log_line [log_file: string, line: string] {
    print $line
    $"($line)\n" | save --append --raw $log_file
}

export def log_block [log_file: string, title: string, content: string] {
    log_line $log_file $"=== ($title) ==="
    if ($content | is-empty) {
        log_line $log_file "<empty>"
    } else {
        for line in ($content | lines) {
            log_line $log_file $line
        }
    }
    log_line $log_file ""
}
