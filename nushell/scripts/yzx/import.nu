#!/usr/bin/env nu
use ../utils/common.nu [get_yazelix_user_config_dir]
use ../setup/helix_config_merger.nu [get_managed_helix_user_config_path get_native_helix_config_path]

def get_xdg_config_home [] {
    let configured = (
        $env.XDG_CONFIG_HOME?
        | default ""
        | into string
        | str trim
    )

    if ($configured | is-not-empty) {
        $configured | path expand
    } else if (($env.HOME? | default "" | into string | str trim) | is-not-empty) {
        ($env.HOME | path join ".config")
    } else {
        "~/.config" | path expand
    }
}

def get_native_zellij_config_path [] {
    (get_xdg_config_home) | path join "zellij" "config.kdl"
}

def get_managed_zellij_config_path [] {
    (get_yazelix_user_config_dir) | path join "zellij" "config.kdl"
}

def get_native_yazi_config_dir [] {
    (get_xdg_config_home) | path join "yazi"
}

def get_managed_yazi_config_dir [] {
    (get_yazelix_user_config_dir) | path join "yazi"
}

def get_import_entries [target: string] {
    match $target {
        "zellij" => [
            {
                name: "config.kdl"
                source: (get_native_zellij_config_path)
                destination: (get_managed_zellij_config_path)
            }
        ]
        "yazi" => {
            let source_dir = (get_native_yazi_config_dir)
            let destination_dir = (get_managed_yazi_config_dir)
            [
                {
                    name: "yazi.toml"
                    source: ($source_dir | path join "yazi.toml")
                    destination: ($destination_dir | path join "yazi.toml")
                }
                {
                    name: "keymap.toml"
                    source: ($source_dir | path join "keymap.toml")
                    destination: ($destination_dir | path join "keymap.toml")
                }
                {
                    name: "init.lua"
                    source: ($source_dir | path join "init.lua")
                    destination: ($destination_dir | path join "init.lua")
                }
            ]
        }
        "helix" => [
            {
                name: "config.toml"
                source: (get_native_helix_config_path)
                destination: (get_managed_helix_user_config_path)
            }
        ]
        _ => {
            error make {msg: $"Unknown import target: ($target)"}
        }
    }
}

def get_existing_source_entries [entries: list<record>] {
    $entries | where {|entry| $entry.source | path exists }
}

def get_missing_source_entries [entries: list<record>] {
    $entries | where {|entry| not ($entry.source | path exists) }
}

def fail_if_no_import_sources [target: string, entries: list<record>] {
    if not (($entries | is-empty)) {
        return
    }

    if $target == "zellij" {
        let source_path = (get_native_zellij_config_path)
        error make {
            msg: (
                [
                    $"Native Zellij config not found at: ($source_path)"
                    ""
                    "Create it first, or copy the settings you want manually into ~/.config/yazelix/user_configs/zellij/config.kdl."
                ] | str join "\n"
            )
        }
    }

    if $target == "helix" {
        let source_path = (get_native_helix_config_path)
        error make {
            msg: (
                [
                    $"Native Helix config not found at: ($source_path)"
                    ""
                    "Create it first, or copy the settings you want manually into ~/.config/yazelix/user_configs/helix/config.toml."
                ] | str join "\n"
            )
        }
    }

    let source_dir = (get_native_yazi_config_dir)
    error make {
        msg: (
            [
                $"No native Yazi config files found under: ($source_dir)"
                "Expected at least one of: yazi.toml, keymap.toml, init.lua"
                ""
                "Create the native Yazi files first, or add the managed overrides directly under ~/.config/yazelix/user_configs/yazi/."
            ] | str join "\n"
        )
    }
}

def fail_if_managed_destinations_exist [target: string, entries: list<record>] {
    let conflicts = ($entries | where {|entry| $entry.destination | path exists })
    if ($conflicts | is-empty) {
        return
    }

    let conflict_lines = ($conflicts | each {|entry| $"- ($entry.destination)" } | str join "\n")
    error make {
        msg: (
            [
                $"Managed destination files already exist for `yzx import ($target)`:
($conflict_lines)"
                ""
                $"Use `yzx import ($target) --force` to overwrite them after writing backups."
            ] | str join "\n"
        )
    }
}

def backup_path_for [path: string, timestamp: string] {
    $"($path).backup-($timestamp)"
}

def import_entries [target: string, entries: list<record>, force: bool] {
    let existing_sources = (get_existing_source_entries $entries)
    let missing_sources = (get_missing_source_entries $entries)

    fail_if_no_import_sources $target $existing_sources

    if not $force {
        fail_if_managed_destinations_exist $target $existing_sources
    }

    let timestamp = (date now | format date "%Y%m%d_%H%M%S")
    mut backup_records = []

    for entry in $existing_sources {
        mkdir ($entry.destination | path dirname)

        if $force and ($entry.destination | path exists) {
            let backup_path = (backup_path_for $entry.destination $timestamp)
            mv $entry.destination $backup_path
            $backup_records = ($backup_records | append {
                name: $entry.name
                backup_path: $backup_path
            })
        }

        cp $entry.source $entry.destination
    }

    match $target {
        "zellij" => {
            let entry = ($existing_sources | first)
            print $"✅ Imported native Zellij config into: ($entry.destination)"
            print $"   Source: ($entry.source)"
        }
        "helix" => {
            let entry = ($existing_sources | first)
            print $"✅ Imported native Helix config into: ($entry.destination)"
            print $"   Source: ($entry.source)"
        }
        "yazi" => {
            print $"✅ Imported native Yazi config into: (get_managed_yazi_config_dir)"
            print $"   Imported files: (($existing_sources | get name | str join ', '))"
        }
    }

    if not ($backup_records | is-empty) {
        print "   Backup files:"
        for backup in $backup_records {
            print $"   - ($backup.name): ($backup.backup_path)"
        }
    }

    if not ($missing_sources | is-empty) {
        print $"   Skipped missing native files: (($missing_sources | get name | str join ', '))"
    }
}

# Import native config files into Yazelix-managed override paths.
export def "yzx import" [] {
    help "yzx import"
}

# Import the native Zellij config into Yazelix-managed overrides.
export def "yzx import zellij" [
    --force  # Overwrite the managed destination after writing a backup
] {
    import_entries "zellij" (get_import_entries "zellij") $force
}

# Import native Yazi config files into Yazelix-managed overrides.
export def "yzx import yazi" [
    --force  # Overwrite managed destination files after writing backups
] {
    import_entries "yazi" (get_import_entries "yazi") $force
}

# Import the native Helix config into Yazelix-managed overrides.
export def "yzx import helix" [
    --force  # Overwrite the managed destination after writing a backup
] {
    import_entries "helix" (get_import_entries "helix") $force
}
