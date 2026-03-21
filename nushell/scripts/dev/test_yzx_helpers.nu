#!/usr/bin/env nu

export const CLEAN_ZELLIJ_ENV_PREFIX = "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID -u ZELLIJ_TAB_NAME -u ZELLIJ_TAB_POSITION"

export def get_repo_root [] {
    pwd
}

export def get_repo_config_dir [] {
    ($env.YAZELIX_DIR? | default "~/.config/yazelix") | path expand
}

export def repo_path [...parts: string] {
    $parts | reduce -f (get_repo_config_dir) {|part, acc| $acc | path join $part }
}

export def setup_test_home [] {
    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_test_home_XXXXXX | str trim)
    let config_parent = ($tmp_home | path join ".config")
    let config_dir = ($config_parent | path join "yazelix")

    mkdir $config_parent
    mkdir $config_dir

    for entry in (ls $repo_root | where name != ($repo_root | path join ".git") and name != ($repo_root | path join "yazelix.toml")) {
        let name = ($entry.name | path basename)
        ^ln -s $entry.name ($config_dir | path join $name)
    }

    cp ($repo_root | path join "yazelix_default.toml") ($config_dir | path join "yazelix.toml")

    {
        repo_root: $repo_root
        tmp_home: $tmp_home
        config_dir: $config_dir
    }
}
