#!/usr/bin/env nu

use ../core/yazelix.nu *

def test_yzx_status [] {
    print "🧪 Testing yzx status..."

    try {
        yzx status | ignore
        print "  ✅ yzx status runs successfully"
        true
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_view [] {
    print "🧪 Testing yzx config..."

    try {
        let output = (
            ^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config | columns | str join ','" | complete
        ).stdout | str trim

        if ($output | str contains "core") and ($output | str contains "terminal") and not ($output | str contains "packs") {
            print "  ✅ yzx config hides packs by default"
            true
        } else {
            print $"  ❌ Unexpected output: ($output)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_sections [] {
    print "🧪 Testing yzx config section views..."

    try {
        ^nu -c 'use ~/.config/yazelix/nushell/scripts/setup/yazi_config_merger.nu *; let root = ($env.HOME | path join ".config" "yazelix"); generate_merged_yazi_config $root --quiet | ignore' | complete | ignore
        let hx_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config hx | columns | str join ','" | complete).stdout | str trim
        let yazi_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config yazi | columns | str join ','" | complete).stdout | str trim
        if (which zellij | is-empty) {
            if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") {
                print "  ℹ️  Skipping zellij config section check because zellij is not available"
                print "  ✅ yzx config section commands return focused sections"
                return true
            }
        }

        ^nu -c 'use ~/.config/yazelix/nushell/scripts/setup/zellij_config_merger.nu *; let root = ($env.HOME | path join ".config" "yazelix"); generate_merged_zellij_config $root | ignore' | complete | ignore
        let zellij_output = (^nu -c "use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config zellij" | complete).stdout | str trim

        if ($hx_output | str contains "config_path") and ($yazi_output | str contains "manager") and ($zellij_output | str contains "default_layout") {
            print "  ✅ yzx config section commands return focused sections"
            true
        } else {
            print $"  ❌ Unexpected section output: hx=($hx_output) yazi=($yazi_output) zellij=($zellij_output)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_yzx_config_reset_replaces_with_backup [] {
    print "🧪 Testing yzx config reset replaces the config with backup..."

    let repo_root = ("~/.config/yazelix" | path expand)
    let tmp_home = (^mktemp -d /tmp/yazelix_config_reset_XXXXXX | str trim)
    let temp_yazelix_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir $temp_yazelix_dir

    let result = (try {
        ^ln -s ($repo_root | path join "nushell") ($temp_yazelix_dir | path join "nushell")
        cp ($repo_root | path join "yazelix_default.toml") ($temp_yazelix_dir | path join "yazelix_default.toml")
        '[shell]
default_shell = "bash"
' | save --force --raw ($temp_yazelix_dir | path join "yazelix.toml")

        let output = with-env { HOME: $tmp_home } {
            ^nu -c 'use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *; yzx config reset --yes' | complete
        }
        let stdout = ($output.stdout | str trim)
        let new_config = (open --raw ($temp_yazelix_dir | path join "yazelix.toml"))
        let default_config = (open --raw ($temp_yazelix_dir | path join "yazelix_default.toml"))
        let backups = (
            ls $temp_yazelix_dir
            | where name =~ 'yazelix\.toml\.backup-'
        )
        let backup_content = if ($backups | is-empty) { "" } else { open --raw (($backups | first).name) }

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Backed up previous config")
            and ($stdout | str contains "Replaced yazelix.toml with a fresh template")
            and ($new_config == $default_config)
            and ($backup_content | str contains 'default_shell = "bash"')
        ) {
            print "  ✅ yzx config reset backs up the current config and restores the template"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) backups=(($backups | length))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

export def run_core_tests [] {
    [
        (test_yzx_status)
        (test_yzx_config_view)
        (test_yzx_config_sections)
        (test_yzx_config_reset_replaces_with_backup)
    ]
}
