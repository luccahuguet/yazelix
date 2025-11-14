#!/usr/bin/env nu
# Build script to generate cursor trail shaders from common library + variants
# This combines cursor_trail_common.glsl with each variant file to eliminate duplication

const SHADER_DIR = ($env.PWD)
const COMMON_FILE = ($SHADER_DIR | path join "cursor_trail_common.glsl")
const VARIANTS_DIR = ($SHADER_DIR | path join "variants")

# Read the common library
let common_code = (open $COMMON_FILE)

print $"Building cursor trail shaders from variants..."
print $"Common library: ($COMMON_FILE)"
print $"Variants directory: ($VARIANTS_DIR)"
print ""

# Process each variant file
let variants = (ls ($VARIANTS_DIR | path join "*.glsl") | get name)

for variant_file in $variants {
    let variant_name = ($variant_file | path basename | str replace ".glsl" "")
    let output_file = ($SHADER_DIR | path join $"cursor_trail_($variant_name).glsl")

    print $"Building ($variant_name)..."

    # Read variant code
    let variant_code = (open $variant_file)

    # Combine common + variant
    let combined = ($common_code + "\n" + $variant_code)

    # Write to output file
    $combined | save -f $output_file

    print $"  ✓ Generated: ($output_file)"
}

print ""
print $"✓ Build complete! Generated (length $variants) shader files."
