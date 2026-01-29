#!/usr/bin/env nu
# yzx packs - Show enabled packs and their sizes

use ../utils/config_parser.nu parse_yazelix_config

# Format bytes to human readable
def format_size [bytes: int] {
    if $bytes < 1024 {
        $"($bytes) B"
    } else if $bytes < (1024 * 1024) {
        $"(($bytes / 1024) | math round --precision 1) KiB"
    } else if $bytes < (1024 * 1024 * 1024) {
        $"(($bytes / 1024 / 1024) | math round --precision 1) MiB"
    } else {
        $"(($bytes / 1024 / 1024 / 1024) | math round --precision 2) GiB"
    }
}

# Get the devenv profile path
def get_devenv_profile [] {
    let yazelix_dir = $env.YAZELIX_DIR? | default ("~/.config/yazelix" | path expand)

    # Try to get from environment first (when inside devenv shell)
    if ($env.DEVENV_PROFILE? | is-not-empty) {
        return $env.DEVENV_PROFILE
    }

    # Use the .devenv/profile symlink
    let profile_link = ($yazelix_dir | path join ".devenv/profile")
    if ($profile_link | path exists) {
        # Resolve symlink to get the actual nix store path
        let resolved = (^readlink -f $profile_link | str trim)
        if ($resolved | path exists) {
            return $resolved
        }
    }

    null
}

# Get closure size for a store path
def get_closure_size [path: string] {
    let result = (do { ^nix path-info --closure-size $path } | complete)
    if $result.exit_code == 0 {
        # Output format: "/nix/store/...\t<size>"
        let parts = $result.stdout | str trim | split row "\t"
        if ($parts | length) >= 2 {
            $parts | last | str trim | into int
        } else {
            0
        }
    } else {
        0
    }
}

# Get all requisites (dependencies) of a store path
def get_requisites [path: string] {
    let result = (do { ^nix-store -qR $path } | complete)
    if $result.exit_code == 0 {
        $result.stdout | lines | where { |l| ($l | str trim) != "" }
    } else {
        []
    }
}

# Show packs and their sizes
#
# Examples:
#   yzx packs              # Show enabled packs summary
#   yzx packs --expand     # Show packages within each pack
#   yzx packs --all        # Show all declared packs (even disabled)
export def "yzx packs" [
    --expand (-e)    # Show individual packages in each pack
    --all (-a)       # Show all declared packs, not just enabled
] {
    let yazelix_dir = $env.YAZELIX_DIR? | default ("~/.config/yazelix" | path expand)
    let config = parse_yazelix_config

    let enabled_packs = $config.pack_names? | default []
    let declarations = $config.pack_declarations? | default {}

    if ($declarations | is-empty) {
        print "No packs declared in yazelix.toml"
        return
    }

    let packs_to_show = if $all {
        $declarations | columns
    } else {
        $enabled_packs
    }

    if ($packs_to_show | is-empty) {
        print "No packs enabled. Enable packs in yazelix.toml under [packs].enabled"
        return
    }

    # Get total shell size
    print "Calculating sizes..."
    let profile = get_devenv_profile
    let total_size = if ($profile | is-not-empty) and ($profile | path exists) {
        get_closure_size $profile
    } else {
        0
    }

    print ""
    print $"(ansi green_bold)Yazelix Shell(ansi reset)"
    print $"Total closure size: (ansi cyan)(format_size $total_size)(ansi reset)"
    print ""

    # Show packs
    print $"(ansi green_bold)Packs(ansi reset) \(($enabled_packs | length) enabled\)"
    print ""

    for pack_name in $packs_to_show {
        let packages = $declarations | get -o $pack_name | default []
        let is_enabled = $pack_name in $enabled_packs
        let status = if $is_enabled { $"(ansi green)●(ansi reset)" } else { $"(ansi yellow)○(ansi reset)" }

        print $"($status) (ansi white_bold)($pack_name)(ansi reset) \(($packages | length) packages\)"

        if $expand and ($packages | length) > 0 {
            for pkg in $packages {
                print $"    - ($pkg)"
            }
            print ""
        }
    }

    if not $expand {
        print ""
        print $"(ansi dark_gray_dimmed)Use --expand to see packages in each pack(ansi reset)"
    }
}
