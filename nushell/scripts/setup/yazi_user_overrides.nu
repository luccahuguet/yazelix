#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_config_dir get_yazelix_user_config_dir]

def get_yazi_user_config_dir [] {
    (get_yazelix_user_config_dir) | path join "yazi"
}

def get_legacy_yazi_user_config_dir [] {
    (get_yazelix_config_dir) | path join "configs" "yazi" "user"
}

export def resolve_yazi_user_file [file_name: string] {
    let current_dir = (get_yazi_user_config_dir)
    let current_path = ($current_dir | path join $file_name)
    let legacy_path = ((get_legacy_yazi_user_config_dir) | path join $file_name)
    let current_exists = ($current_path | path exists)
    let legacy_exists = ($legacy_path | path exists)

    if $current_exists and $legacy_exists {
        error make {
            msg: (
                [
                    $"Yazelix found duplicate Yazi user config files for ($file_name)."
                    $"user_configs path: ($current_path)"
                    $"legacy path: ($legacy_path)"
                    ""
                    "Keep only the user_configs copy. Move or delete the legacy configs/yazi/user file so Yazelix has one clear owner."
                ] | str join "\n"
            )
        }
    }

    if $legacy_exists {
        error make {
            msg: (
                [
                    $"Yazelix found a legacy Yazi user config file for ($file_name)."
                    $"legacy path: ($legacy_path)"
                    $"managed path: ($current_path)"
                    ""
                    "Yazelix no longer relocates legacy Yazi overrides during normal config generation."
                    "Use `yzx import yazi` to move native or legacy overrides into `~/.config/yazelix/user_configs/yazi/`, or move the file manually."
                ] | str join "\n"
            )
        }
    }

    $current_path
}

def deep_merge [base: record, user: record] {
    let base_keys = ($base | columns)
    let user_keys = ($user | columns)
    let all_keys = ($base_keys | append $user_keys | uniq)

    $all_keys | reduce --fold {} {|key, acc|
        let in_base = ($key in $base_keys)
        let in_user = ($key in $user_keys)

        let value = if $in_base and $in_user {
            let base_val = ($base | get -o $key)
            let user_val = ($user | get -o $key)
            let base_type = ($base_val | describe)
            let user_type = ($user_val | describe)
            let base_is_array = ($base_type | str starts-with "list") or ($base_type | str starts-with "table")
            let user_is_array = ($user_type | str starts-with "list") or ($user_type | str starts-with "table")

            if ($base_type | str starts-with "record") and ($user_type | str starts-with "record") {
                deep_merge $base_val $user_val
            } else if $base_is_array and $user_is_array {
                $base_val | append $user_val
            } else {
                $user_val
            }
        } else if $in_user {
            $user | get -o $key
        } else {
            $base | get -o $key
        }

        $acc | insert $key $value
    }
}

export def merge_yazi_toml_config [base_config: record, user_config: record] {
    deep_merge $base_config $user_config
}

export def merge_yazi_keymap [base_keymap: record, user_keymap: record] {
    let sections = ($base_keymap | columns)
    $sections | reduce --fold $base_keymap {|section, acc|
        if ($section in ($user_keymap | columns)) {
            let base_section = ($acc | get -o $section)
            let user_section = ($user_keymap | get -o $section)
            let subsections = ($base_section | columns)

            let merged_section = $subsections | reduce --fold $base_section {|sub, sec_acc|
                if ($sub in ($user_section | columns)) {
                    let base_arr = ($sec_acc | get -o $sub)
                    let user_arr = ($user_section | get -o $sub)
                    $sec_acc | upsert $sub ($base_arr | append $user_arr)
                } else {
                    $sec_acc
                }
            }

            let new_subsections = ($user_section | columns | where {|s| $s not-in $subsections})
            let final_section = $new_subsections | reduce --fold $merged_section {|sub, sec_acc|
                $sec_acc | upsert $sub ($user_section | get -o $sub)
            }

            $acc | upsert $section $final_section
        } else {
            $acc
        }
    }
}
