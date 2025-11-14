#!/usr/bin/env nu
# Script to create macOS .icns icon from PNG images
# Run this script on macOS to generate the yazelix.icns icon file

def main [] {
    # Get script directory and related paths
    let script_dir = $env.FILE_PWD
    let icon_dir = $"($script_dir)/../icons"
    let iconset_dir = $"($script_dir)/yazelix.iconset"
    let app_resources = $"($script_dir)/Yazelix.app/Contents/Resources"

    print "Creating iconset directory..."
    mkdir $iconset_dir

    # Copy and rename PNG files to the iconset format required by macOS
    # macOS iconset requires specific naming: icon_<size>x<size>.png and icon_<size>x<size>@2x.png
    print "Copying icon files..."

    # Map of source files to destination names with comments
    let icon_mappings = [
        {src: "48x48/yazelix.png", dst: "icon_24x24@2x.png", note: "48x48 for 24pt @2x"},
        {src: "64x64/yazelix.png", dst: "icon_32x32@2x.png", note: "64x64 for 32pt @2x"},
        {src: "128x128/yazelix.png", dst: "icon_128x128.png", note: "128x128 for 128pt"},
        {src: "256x256/yazelix.png", dst: "icon_128x128@2x.png", note: "256x256 for 128pt @2x"},
        {src: "256x256/yazelix.png", dst: "icon_256x256.png", note: "256x256 for 256pt"},
        {src: "48x48/yazelix.png", dst: "icon_16x16@2x.png", note: "Alternative usage"}
    ]

    # Copy each icon file
    for mapping in $icon_mappings {
        cp $"($icon_dir)/($mapping.src)" $"($iconset_dir)/($mapping.dst)"
    }

    print "Converting iconset to icns..."
    ^iconutil -c icns $iconset_dir -o $"($app_resources)/yazelix.icns"

    print "Cleaning up..."
    rm -rf $iconset_dir

    print $"✅ Icon created successfully at: ($app_resources)/yazelix.icns"
    print ""
    print "Note: This icon is optional. The .app will work without it,"
    print "but having it provides a better visual experience in Finder and Spotlight."
}
