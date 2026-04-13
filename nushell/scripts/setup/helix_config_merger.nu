#!/usr/bin/env nu
# Generate a Yazelix-managed Helix config.toml for Yazelix-managed Helix sessions.

use ../utils/atomic_writes.nu write_text_atomic
use ../utils/common.nu [get_yazelix_runtime_dir get_yazelix_state_dir get_yazelix_user_config_dir]

const MANAGED_REVEAL_COMMAND = ':sh yzx reveal "%{buffer_name}"'

export def get_managed_reveal_command [] {
    $MANAGED_REVEAL_COMMAND
}

def get_helix_template_path [] {
    (get_yazelix_runtime_dir) | path join "configs" "helix" "yazelix_config.toml"
}

def get_managed_helix_user_config_dir [] {
    (get_yazelix_user_config_dir) | path join "helix"
}

export def get_managed_helix_user_config_path [] {
    (get_managed_helix_user_config_dir) | path join "config.toml"
}

export def get_native_helix_config_path [] {
    let xdg_config_home = (
        $env.XDG_CONFIG_HOME?
        | default ($env.HOME | path join ".config")
        | into string
        | str trim
        | path expand
    )
    ($xdg_config_home | path join "helix" "config.toml")
}

def get_generated_helix_config_dir [] {
    (get_yazelix_state_dir) | path join "configs" "helix"
}

export def get_generated_helix_config_path [] {
    (get_generated_helix_config_dir) | path join "config.toml"
}

def get_helix_notice_state_dir [] {
    (get_yazelix_state_dir) | path join "state" "helix"
}

def get_helix_import_notice_marker_path [] {
    (get_helix_notice_state_dir) | path join "import_notice_seen"
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

            if ($base_type | str starts-with "record") and ($user_type | str starts-with "record") {
                deep_merge $base_val $user_val
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

def enforce_reveal_binding [config: record] {
    let keys_config = ($config | get -o keys | default {})
    let normal_keys = ($keys_config | get -o normal | default {})
    let updated_normal_keys = ($normal_keys | upsert "A-r" $MANAGED_REVEAL_COMMAND)
    let updated_keys = ($keys_config | upsert normal $updated_normal_keys)
    $config | upsert keys $updated_keys
}

export def build_managed_helix_config [] {
    let template_path = (get_helix_template_path)
    if not ($template_path | path exists) {
        error make {msg: $"Missing Yazelix Helix template at: ($template_path)"}
    }

    let user_config_path = (get_managed_helix_user_config_path)
    let base_config = (open $template_path)
    let merged_config = if ($user_config_path | path exists) {
        let user_config = (open $user_config_path)
        deep_merge $base_config $user_config
    } else {
        $base_config
    }

    enforce_reveal_binding $merged_config
}

def maybe_show_helix_import_notice [] {
    let native_config_path = (get_native_helix_config_path)
    let managed_user_config_path = (get_managed_helix_user_config_path)
    let notice_marker_path = (get_helix_import_notice_marker_path)

    if not ($native_config_path | path exists) {
        return
    }

    if ($managed_user_config_path | path exists) {
        return
    }

    if ($notice_marker_path | path exists) {
        return null
    }

    {
        marker_path: $notice_marker_path
        lines: [
            "ℹ️  Yazelix is using its managed Helix config."
            $"   Personal Helix config detected at: ($native_config_path)"
            "   If you want Yazelix-managed Helix sessions to reuse it, run: yzx import helix"
        ]
    }
}

export def generate_managed_helix_config [] {
    let output_path = (get_generated_helix_config_path)
    let final_config = (build_managed_helix_config)
    let notice = (maybe_show_helix_import_notice)
    write_text_atomic $output_path ($final_config | to toml) --raw | ignore

    if $notice != null {
        write_text_atomic $notice.marker_path "" --raw | ignore
        for line in $notice.lines {
            print --stderr $line
        }
    }

    $output_path
}

export def main [--print-path] {
    let output_path = (generate_managed_helix_config)
    if $print_path {
        print $output_path
    }
}
