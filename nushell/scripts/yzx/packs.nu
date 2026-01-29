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

# Get the devenv shell derivation path (reliable source of truth)
# Uses .devenv/gc/shell symlink which is the authoritative GC root for the current shell
def get_devenv_shell [] {
    let yazelix_dir = $env.YAZELIX_DIR? | default ("~/.config/yazelix" | path expand)
    let shell_link = ($yazelix_dir | path join ".devenv/gc/shell")
    
    if ($shell_link | path exists) {
        let resolved = (^readlink -f $shell_link | str trim)
        if ($resolved | path exists) {
            return $resolved
        }
    }
    
    # Fallback to .devenv/profile symlink
    let profile_link = ($yazelix_dir | path join ".devenv/profile")
    if ($profile_link | path exists) {
        let resolved = (^readlink -f $profile_link | str trim)
        if ($resolved | path exists) {
            return $resolved
        }
    }
    
    null
}

# Get all nix store paths from PATH that belong to this yazelix session
# This captures all packages added by devenv, including packs
def get_session_nix_paths [] {
    # Only include /nix/store paths from PATH (devenv adds packages this way)
    $env.PATH | where { |p| $p | str starts-with "/nix/store/" } | each { |p|
        # Convert /nix/store/xxx/bin to /nix/store/xxx
        $p | path dirname
    } | uniq
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

# Get NAR size (just this path, not closure) for a store path
def get_nar_size [path: string] {
    let result = (do { ^nix path-info --size $path } | complete)
    if $result.exit_code == 0 {
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

# Get all store paths in profile with their sizes
def get_profile_paths_with_sizes [profile: string] {
    # Get all paths in the closure with their sizes
    let result = (do { ^nix path-info -rS $profile } | complete)
    if $result.exit_code == 0 {
        $result.stdout | lines | where { |l| ($l | str trim) != "" } | each { |line|
            let parts = $line | str trim | split row "\t"
            if ($parts | length) >= 2 {
                {
                    path: ($parts | first),
                    size: ($parts | last | str trim | into int)
                }
            } else {
                null
            }
        } | where { |x| $x != null }
    } else {
        []
    }
}

# Extract package name from store path (e.g., /nix/store/xxx-ripgrep-14.1.0 -> ripgrep)
def extract_pkg_name [store_path: string] {
    # Get the basename after the hash
    let basename = $store_path | path basename
    # Remove the hash prefix (32 chars + dash)
    let name_with_version = if ($basename | str length) > 33 {
        $basename | str substring 33..
    } else {
        $basename
    }
    # Remove version suffix (everything after last dash followed by digit)
    $name_with_version | parse --regex '^(?<name>.+?)-\d' | get -o 0.name | default $name_with_version
}

# Normalize package name for matching (handle prefixes like nodePackages., python3Packages.)
def normalize_pkg_name [pkg: string] {
    $pkg
    | str replace 'nodePackages.' ''
    | str replace 'python3Packages.' ''
    | str replace -a '-' ''
    | str replace -a '_' ''
    | str downcase
}

# Check if a store path matches a package name
def matches_pkg [store_name: string, pkg: string] {
    let normalized_store = normalize_pkg_name $store_name
    let normalized_pkg = normalize_pkg_name $pkg

    # Direct match
    if $normalized_store == $normalized_pkg { return true }

    # Store name contains the normalized package name
    if ($normalized_store | str contains $normalized_pkg) { return true }

    # Handle common package name mappings
    let mappings = {
        "jujutsu": "jj",
        "typescript-language-server": "typescript",
        "kotlin-language-server": "kotlin",
        "yaml-language-server": "yaml",
    }

    let mapped = $mappings | get -o $pkg | default null
    if $mapped != null and ($normalized_store | str contains (normalize_pkg_name $mapped)) {
        return true
    }

    false
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

    # Get total shell size and all paths with sizes
    print "Calculating sizes..."
    let shell_path = get_devenv_shell
    
    # Get total closure size from the shell derivation
    let total_size = if ($shell_path | is-not-empty) and ($shell_path | path exists) {
        get_closure_size $shell_path
    } else {
        0
    }
    
    # Get package sizes from both the shell closure AND session PATH
    # The shell closure has core packages; PATH has pack packages added by devenv
    let shell_paths = if ($shell_path | is-not-empty) and ($shell_path | path exists) {
        get_profile_paths_with_sizes $shell_path
    } else {
        []
    }
    
    # Get NAR sizes for packages in PATH (these are the pack packages)
    let session_paths = get_session_nix_paths | each { |p|
        let result = (do { ^nix path-info -S $p } | complete)
        if $result.exit_code == 0 {
            let parts = $result.stdout | str trim | split row "\t"
            if ($parts | length) >= 2 {
                { path: ($parts | first), size: ($parts | last | str trim | into int) }
            } else {
                null
            }
        } else {
            null
        }
    } | where { |x| $x != null }
    
    # Combine both sources, preferring session paths for duplicates
    let all_paths = $shell_paths | append $session_paths | uniq-by path

    # Build a lookup from package name to size
    let pkg_sizes = $all_paths | each { |p|
        let name = extract_pkg_name $p.path
        { name: $name, size: $p.size, path: $p.path }
    }

    print ""
    print $"(ansi green_bold)Yazelix Shell(ansi reset)"
    print $"Total closure size: (ansi cyan)(format_size $total_size)(ansi reset)"
    print ""

    # Get /nix/store size
    let nix_store_result = (do { ^du -sb /nix/store } | complete)
    let nix_store_size = if $nix_store_result.exit_code == 0 {
        $nix_store_result.stdout | split row "\t" | first | into int
    } else {
        0
    }
    print $"(ansi green_bold)Nix Store(ansi reset)"
    print $"Total /nix/store:   (ansi cyan)(format_size $nix_store_size)(ansi reset)"
    print ""

    # Show packs with sizes
    print $"(ansi green_bold)Packs(ansi reset) \(($enabled_packs | length) enabled\)"
    print ""

    for pack_name in $packs_to_show {
        let packages = $declarations | get -o $pack_name | default []
        let is_enabled = $pack_name in $enabled_packs
        let status = if $is_enabled { $"(ansi green)●(ansi reset)" } else { $"(ansi yellow)○(ansi reset)" }

        # Calculate pack size by matching package names
        let pack_size = $packages | each { |pkg|
            # Find matching store paths
            let matches = $pkg_sizes | where { |ps| matches_pkg $ps.name $pkg }
            if ($matches | is-empty) {
                0
            } else {
                # Take the largest match (usually the main package)
                $matches | get size | math max
            }
        } | math sum

        let size_str = if $pack_size > 0 {
            $"(ansi cyan)(format_size $pack_size)(ansi reset)"
        } else {
            $"(ansi dark_gray)~(ansi reset)"
        }

        print $"($status) (ansi white_bold)($pack_name)(ansi reset) \(($packages | length) packages\) ($size_str)"

        if $expand and ($packages | length) > 0 {
            for pkg in $packages {
                # Find size for this specific package
                let matches = $pkg_sizes | where { |ps| matches_pkg $ps.name $pkg }
                let pkg_size = if ($matches | is-empty) { 0 } else { $matches | get size | math max }
                let pkg_size_str = if $pkg_size > 0 {
                    $" (ansi dark_gray)\((format_size $pkg_size)\)(ansi reset)"
                } else {
                    ""
                }
                print $"    - ($pkg)($pkg_size_str)"
            }
            print ""
        }
    }

    if not $expand {
        print ""
        print $"(ansi dark_gray_dimmed)Use --expand to see packages in each pack(ansi reset)"
    }

    # Show yzx repo size
    print ""
    print $"(ansi green_bold)Local(ansi reset)"
    let repo_size = (^du -sb $yazelix_dir | split row "\t" | first | into int)
    print $"yzx repo: (ansi cyan)(format_size $repo_size)(ansi reset)"
}
