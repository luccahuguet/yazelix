#!/usr/bin/env nu

# Utility functions for Yazelix

# Get the number of CPU cores to use for builds based on configuration
export def get_max_cores [] {
    let total_cores = (sys cpu | length)

    # Try to read from environment variable (set by devenv.nix)
    let build_cores_config = ($env.YAZELIX_BUILD_CORES? | default "max_minus_one")

    # Parse configuration
    match $build_cores_config {
        "max" => $total_cores,
        "max_minus_one" => (if $total_cores > 1 { $total_cores - 1 } else { 1 }),
        "half" => (($total_cores / 2) | math floor | into int),
        _ => {
            # Try to parse as a number
            try {
                $build_cores_config | into int
            } catch {
                # Fallback to max_minus_one if invalid
                if $total_cores > 1 { $total_cores - 1 } else { 1 }
            }
        }
    }
}

# Check if Helix (hx or helix) is running in a Zellij pane based on client output
export def is_hx_running [list_clients_output: string] {
    let cmd = $list_clients_output | str trim | str downcase
    let parts = $cmd | split row " "
    let has_hx_paths = ($parts | any {|part| $part | str ends-with "/hx"})
    let has_helix_paths = ($parts | any {|part| $part | str ends-with "/helix"})
    let is_hx_cmd = ($parts | any {|part| $part == "hx"})
    let is_helix_cmd = ($parts | any {|part| $part == "helix"})

    $has_hx_paths or $has_helix_paths or $is_hx_cmd or $is_helix_cmd
}
