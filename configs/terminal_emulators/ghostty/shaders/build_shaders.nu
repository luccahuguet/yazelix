#!/usr/bin/env nu
# Build script to generate cursor trail shaders from common library + variants
# This combines cursor_trail_common.glsl with each variant file to eliminate duplication

# Build cursor trail shaders from common library + variants
export def build_cursor_trail_shaders [shader_dir: path] {
    let common_file = ($shader_dir | path join "cursor_trail_common.glsl")
    let variants_dir = ($shader_dir | path join "variants")

    # Check if source files exist
    if not ($common_file | path exists) {
        print $"⚠ Shader common library not found: ($common_file)"
        return
    }

    if not ($variants_dir | path exists) {
        print $"⚠ Shader variants directory not found: ($variants_dir)"
        return
    }

    # Read the common library
    let common_code = (open $common_file)

    # Process each variant file (use glob command for dynamic patterns)
    let variants = (glob ($variants_dir | path join "*.glsl"))

    if ($variants | is-empty) {
        print $"⚠ No shader variants found in ($variants_dir)"
        return
    }

    for variant_file in $variants {
        let variant_name = ($variant_file | path basename | str replace ".glsl" "")
        let output_file = ($shader_dir | path join $"cursor_trail_($variant_name).glsl")

        # Read variant code
        let variant_code = (open $variant_file)

        # Combine common + variant
        let combined = ($common_code + "\n" + $variant_code)

        # Write to output file
        $combined | save -f $output_file
    }

    print $"✓ Built ($variants | length) cursor trail shaders from modular sources"
}

# Main entry point when run directly
def main [] {
    let shader_dir = ($env.PWD)
    print $"Building cursor trail shaders..."
    print $"Shader directory: ($shader_dir)"
    print ""

    build_cursor_trail_shaders $shader_dir
}
