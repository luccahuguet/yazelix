#!/usr/bin/env bash
# Build script to generate cursor trail shaders from common library + variants
# This combines cursor_trail_common.glsl with each variant file to eliminate duplication

set -euo pipefail

SHADER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMMON_FILE="$SHADER_DIR/cursor_trail_common.glsl"
VARIANTS_DIR="$SHADER_DIR/variants"

echo "Building cursor trail shaders from variants..."
echo "Common library: $COMMON_FILE"
echo "Variants directory: $VARIANTS_DIR"
echo ""

# Read the common library
COMMON_CODE=$(cat "$COMMON_FILE")

# Process each variant file
for variant_file in "$VARIANTS_DIR"/*.glsl; do
    variant_name=$(basename "$variant_file" .glsl)
    output_file="$SHADER_DIR/cursor_trail_${variant_name}.glsl"

    echo "Building $variant_name..."

    # Combine common + variant
    {
        echo "$COMMON_CODE"
        echo ""
        cat "$variant_file"
    } > "$output_file"

    echo "  ✓ Generated: $output_file"
done

echo ""
echo "✓ Build complete! Generated $(ls "$VARIANTS_DIR"/*.glsl | wc -l) shader files."
