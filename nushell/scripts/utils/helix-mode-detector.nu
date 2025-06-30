#!/usr/bin/env nu
# Smart Helix mode detector - analyzes actual running environment
# Usage: nu helix-mode-detector.nu
# Or: helix-mode

def main [] {
    print "🔍 Analyzing actual Helix environment..."
    print "========================================"

    let analysis = analyze_helix_environment
    let actual_mode = determine_actual_mode $analysis

    print $""
    print "📊 ANALYSIS RESULTS:"
    print "==================="
    print $"Helix binary path: ($analysis.binary_path)"
    print $"Helix version: ($analysis.version)"
    print $"Binary type: ($analysis.binary_type)"
    print $"Build directory exists: ($analysis.build_dir_exists)"
    print $"Git branch: ($analysis.git_branch)"
    print $"Steel files present: ($analysis.steel_files_present)"
    print $"Steel dependencies: ($analysis.steel_dependencies)"
    print $"Patchy config exists: ($analysis.patchy_config_exists)"
    print $""
    print "🎯 DETECTED MODE:"
    print "================"
    print $"ACTUAL MODE: ($actual_mode)"
    print $""

    # Show detailed reasoning
    print "🔍 REASONING:"
    print "============="
    show_reasoning $analysis $actual_mode

    # Show config comparison if available
    let config_mode = try { get_config_mode } catch { "unknown" }
    if $config_mode != "unknown" {
        print $""
        print "⚙️  CONFIG COMPARISON:"
        print "===================="
        print $"Configured mode: ($config_mode)"
        print $"Actual mode: ($actual_mode)"
        if $config_mode != $actual_mode {
            print "⚠️  MISMATCH DETECTED!"
            print "   The configured mode differs from the actual running mode."
            print "   This usually happens when switching between modes without proper cleanup."
        } else {
            print "✅ Mode consistency: OK"
        }
    }

    print $""
    print "💡 TIPS:"
    print "========"
    if $actual_mode == "steel" {
        print "• Steel mode detected - you have Steel plugins available"
        print "• Try :hello-steel or :list-commands in Helix"
    } else if $actual_mode == "patchy" {
        print "• Patchy mode detected - you have community PRs applied"
        print "• Check the .patchy/config.toml for applied PRs"
    } else if $actual_mode == "source" {
        print "• Source mode detected - using Helix flake from repository"
        print "• You have the latest bleeding-edge features"
    } else {
        print "• Release mode detected - using nixpkgs Helix"
        print "• Fast and stable, good for first-time users"
    }
}

def analyze_helix_environment [] {
    let helix_patchy_dir = $"($env.HOME)/.config/yazelix/helix_patchy"
    let helix_config_dir = $"($env.HOME)/.config/helix"

    # Find Helix binary
    let binary_path = try {
        let which_result = which hx
        if ($which_result | is-empty) {
            "hx not found in PATH"
        } else {
            $which_result | first | get path
        }
    } catch {
        "hx not found in PATH"
    }

    # Get version
    let version = try {
        hx --version | str trim
    } catch {
        "version unknown"
    }

    # Determine binary type
    let binary_type = if ($binary_path | str contains "helix_patchy") {
        "custom build"
    } else if ($binary_path | str contains "nix/store") {
        "nixpkgs"
    } else {
        "system"
    }

    # Check build directory
    let build_dir_exists = $helix_patchy_dir | path exists

    # Check git branch
    let git_branch = if $build_dir_exists {
        try {
            cd $helix_patchy_dir
            git branch --show-current
        } catch {
            "unknown"
        }
    } else {
        "no build dir"
    }

    # Check Steel files
    let steel_files_present = try {
        ls $"($helix_config_dir)/*.scm" | length
    } catch {
        0
    }

    # Check Steel dependencies in binary
    let steel_dependencies = if ($binary_path | str contains "helix_patchy") and ($binary_path != "hx not found in PATH") {
        try {
            let ldd_output = (ldd $binary_path 2>/dev/null | str join " ")
            $ldd_output | str contains "steel"
        } catch {
            false
        }
    } else {
        false
    }

    # Check Patchy config
    let patchy_config_exists = try {
        $"($helix_patchy_dir)/.patchy/config.toml" | path exists
    } catch {
        false
    }

    {
        binary_path: $binary_path
        version: $version
        binary_type: $binary_type
        build_dir_exists: $build_dir_exists
        git_branch: $git_branch
        steel_files_present: $steel_files_present
        steel_dependencies: $steel_dependencies
        patchy_config_exists: $patchy_config_exists
    }
}

def determine_actual_mode [analysis] {
    # Steel mode detection (highest priority)
    if $analysis.steel_files_present > 0 or $analysis.steel_dependencies or ($analysis.git_branch | str contains "steel") {
        "steel"
    } else {
        # Patchy mode detection
        if $analysis.patchy_config_exists or ($analysis.git_branch | str contains "patchy") {
            "patchy"
        } else {
            # Source mode detection (custom build but not steel/patchy)
            if $analysis.build_dir_exists and ($analysis.binary_type == "custom build") {
                "source"
            } else {
                # Release mode (nixpkgs or system binary)
                "release"
            }
        }
    }
}

def show_reasoning [analysis, actual_mode] {
    if $actual_mode == "steel" {
        if $analysis.steel_files_present > 0 {
            print "• Steel configuration files found in ~/.config/helix/"
        }
        if $analysis.steel_dependencies {
            print "• Binary has Steel dependencies linked"
        }
        if ($analysis.git_branch | str contains "steel") {
            print "• Built from steel-event-system branch"
        }
    } else if $actual_mode == "patchy" {
        if $analysis.patchy_config_exists {
            print "• Patchy configuration found (.patchy/config.toml)"
        }
        if ($analysis.git_branch | str contains "patchy") {
            print "• Built from patchy branch"
        }
    } else if $actual_mode == "source" {
        print "• Custom build directory exists"
        print "• Binary is from custom build (not nixpkgs)"
        print "• No Steel/Patchy indicators found"
    } else {
        print "• Using nixpkgs or system Helix binary"
        print "• No custom build directory or Steel/Patchy indicators"
    }
}

def get_config_mode [] {
    let yazelix_config = $"($env.HOME)/.config/yazelix/yazelix.nix"
    let default_config = $"($env.HOME)/.config/yazelix/yazelix_default.nix"

    let config_file = if ($yazelix_config | path exists) { $yazelix_config } else { $default_config }

    if ($config_file | path exists) {
        try {
            let config_content = (open $config_file)
            let helix_mode_line = ($config_content | lines | where $it | str contains "helix_mode")

            if not ($helix_mode_line | is-empty) {
                $helix_mode_line | first | str replace "helix_mode = " "" | str replace "\"" "" | str replace ";" "" | str trim
            } else {
                "release"
            }
        } catch {
            "unknown"
        }
    } else {
        "unknown"
    }
}

# Export for use in other scripts
export def detect_actual_helix_mode [] {
    let analysis = analyze_helix_environment
    determine_actual_mode $analysis
}

# Run main if called directly
if ($env | get -i argv | default [] | is-empty) {
    main
}