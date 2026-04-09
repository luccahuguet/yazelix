#!/usr/bin/env nu

use ../utils/common.nu get_yazelix_user_config_dir

def get_zellij_defaults [] {
    try { zellij setup --dump-config } catch {|err|
        print $"❌ CRITICAL ERROR: Cannot fetch Zellij defaults: ($err.msg)"
        print "   Zellij must be available in PATH for Yazelix to work properly."
        print "   This indicates the merger is running outside the Nix environment."
        print "   Yazelix cannot function without proper Zellij configuration."
        exit 1
    }
}

def get_zellij_user_config_path [] {
    (get_yazelix_user_config_dir) | path join "zellij" "config.kdl"
}

def get_legacy_native_zellij_config_path [] {
    ($env.HOME | path join ".config" "zellij" "config.kdl")
}

def read_text_if_exists [path_value: string] {
    if not ($path_value | path exists) {
        return ""
    }

    open --raw $path_value
}

export def resolve_base_config_source [] {
    let managed_path = (get_zellij_user_config_path)
    let native_path = (get_legacy_native_zellij_config_path)

    if ($managed_path | path exists) {
        {
            source: "managed"
            path: $managed_path
            content: (read_text_if_exists $managed_path)
        }
    } else if ($native_path | path exists) {
        {
            source: "native"
            path: $native_path
            content: (read_text_if_exists $native_path)
        }
    } else {
        {
            source: "defaults"
            path: ""
            content: (get_zellij_defaults)
        }
    }
}

export def describe_base_config_source [resolved: record] {
    match $resolved.source {
        "managed" => {
            print $"📥 Using managed Zellij config from ($resolved.path)"
        }
        "native" => {
            print $"📥 Using native Zellij config from ($resolved.path)"
        }
        _ => {
            print "📥 No user Zellij config found, fetching defaults..."
        }
    }
}
