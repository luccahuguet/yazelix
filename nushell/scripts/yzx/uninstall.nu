#!/usr/bin/env nu

use ../utils/common.nu [get_yazelix_state_dir]
use ../utils/install_ownership.nu [
    collect_manual_uninstall_artifacts
    get_manual_main_config_path
    get_manual_pack_config_path
    has_home_manager_managed_install
]

def format_uninstall_artifact [artifact: record] {
    $"  - ($artifact.label): ($artifact.path)"
}

def print_preserved_config_surfaces [] {
    let main_config = (get_manual_main_config_path)
    let pack_config = (get_manual_pack_config_path)
    let preserved = (
        [
            {label: "managed yazelix.toml surface", path: $main_config}
            {label: "managed yazelix_packs.toml surface", path: $pack_config}
        ]
        | where {|entry| $entry.path | path exists }
    )

    if ($preserved | is-empty) {
        return
    }

    print "Preserved by default:"
    for entry in $preserved {
        print $"  - ($entry.label): ($entry.path)"
    }
}

def remove_if_empty [path: string] {
    if ($path | path exists) and ((ls $path | is-empty)) {
        rm -rf $path
    }
}

def cleanup_empty_parents_after_uninstall [] {
    let state_dir = (get_yazelix_state_dir)
    remove_if_empty ($state_dir | path join "runtime")
    remove_if_empty ($state_dir | path join "configs")
}

export def "yzx uninstall" [
    --apply  # Remove installer-owned Yazelix artifacts
    --yes    # Skip confirmation prompt when using --apply
] {
    let artifacts = (collect_manual_uninstall_artifacts)
    let hm_managed = (has_home_manager_managed_install)

    if not $apply {
        if ($artifacts | is-empty) {
            if $hm_managed {
                print "No manual installer-owned Yazelix artifacts were found."
                print "Home Manager-managed Yazelix surfaces are present. Disable or remove the Home Manager module, then run `home-manager switch` if you want to uninstall that path."
            } else {
                print "No manual installer-owned Yazelix artifacts were found."
            }
            return
        }

        print "Yazelix uninstall preview"
        for artifact in $artifacts {
            print (format_uninstall_artifact $artifact)
        }
        print ""
        print_preserved_config_surfaces
        print ""
        print "Run `yzx uninstall --apply` to remove these installer-owned artifacts."
        return
    }

    if ($artifacts | is-empty) {
        if $hm_managed {
            print "No manual installer-owned Yazelix artifacts were found."
            print "Home Manager-managed Yazelix surfaces are present. Disable or remove the Home Manager module, then run `home-manager switch` if you want to uninstall that path."
        } else {
            print "No manual installer-owned Yazelix artifacts were found."
        }
        return
    }

    if not $yes {
        print "⚠️  This removes installer-owned Yazelix artifacts and keeps your managed Yazelix config by default."
        let confirm = try {
            (input "Continue? [y/N]: " | str downcase | str trim)
        } catch {
            "n"
        }

        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            return
        }
    }

    for artifact in $artifacts {
        rm -rf $artifact.path
    }

    cleanup_empty_parents_after_uninstall

    print "Removed installer-owned Yazelix artifacts:"
    for artifact in $artifacts {
        print (format_uninstall_artifact $artifact)
    }
    print ""
    print_preserved_config_surfaces
}
