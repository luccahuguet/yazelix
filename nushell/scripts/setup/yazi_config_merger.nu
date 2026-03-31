#!/usr/bin/env nu
# Yazi Configuration Generator
# Generates yazi configs from yazelix defaults + dynamic settings from yazelix.toml

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/common.nu [get_yazelix_config_dir get_yazelix_state_dir get_yazelix_user_config_dir]

# Ensure directory exists
def ensure_dir [path: string] {
    let dir = ($path | path dirname)
    if not ($dir | path exists) {
        mkdir $dir
    }
}

def get_yazi_user_config_dir [] {
    (get_yazelix_user_config_dir) | path join "yazi"
}

def get_legacy_yazi_user_config_dir [] {
    (get_yazelix_config_dir) | path join "configs" "yazi" "user"
}

def reconcile_yazi_user_file [file_name: string] {
    let current_dir = (get_yazi_user_config_dir)
    let current_path = ($current_dir | path join $file_name)
    let legacy_path = ((get_legacy_yazi_user_config_dir) | path join $file_name)
    let current_exists = ($current_path | path exists)
    let legacy_exists = ($legacy_path | path exists)

    if $current_exists and $legacy_exists {
        error make {
            msg: (
                [
                    $"Yazelix found duplicate Yazi user config files for ($file_name)."
                    $"user_configs path: ($current_path)"
                    $"legacy path: ($legacy_path)"
                    ""
                    "Keep only the user_configs copy. Move or delete the legacy configs/yazi/user file so Yazelix has one clear owner."
                ] | str join "\n"
            )
        }
    }

    if $legacy_exists {
        mkdir $current_dir
        mv $legacy_path $current_path
    }

    $current_path
}

# Deep merge two TOML records (user values override base, arrays are concatenated)
def deep_merge [base: record, user: record] {
    let base_keys = ($base | columns)
    let user_keys = ($user | columns)
    let all_keys = ($base_keys | append $user_keys | uniq)

    $all_keys | reduce --fold {} {|key, acc|
        let in_base = ($key in $base_keys)
        let in_user = ($key in $user_keys)

        let value = if $in_base and $in_user {
            let base_val = ($base | get -o $key)
            let user_val = ($user | get -o $key)
            let base_type = ($base_val | describe)
            let user_type = ($user_val | describe)
            let base_is_array = ($base_type | str starts-with "list") or ($base_type | str starts-with "table")
            let user_is_array = ($user_type | str starts-with "list") or ($user_type | str starts-with "table")
            # If both are records, merge recursively
            if ($base_type | str starts-with "record") and ($user_type | str starts-with "record") {
                deep_merge $base_val $user_val
            } else if $base_is_array and $user_is_array {
                # Concatenate arrays (base first, then user)
                $base_val | append $user_val
            } else {
                # For other types, user wins
                $user_val
            }
        } else if $in_user {
            $user | get -o $key
        } else {
            $base | get -o $key
        }

        $acc | insert $key $value
    }
}

# Copy plugins directory (preserves user-installed plugins)
def copy_plugins_directory [source_dir: string, merged_dir: string, --quiet] {
    let source_plugins = $"($source_dir)/plugins"
    let merged_plugins = $"($merged_dir)/plugins"

    if not $quiet {
        print "   📁 Copying plugins directory..."
    }

    # Ensure plugins directory exists
    if not ($merged_plugins | path exists) {
        mkdir $merged_plugins
    }

    # Copy yazelix bundled plugins (overwrites if they exist)
    # This preserves user-installed plugins that yazelix doesn't provide
    if ($source_plugins | path exists) {
        let bundled_plugins = (ls $source_plugins | where type == dir | get name)

        for plugin_path in $bundled_plugins {
            let plugin_name = ($plugin_path | path basename)
            let target = $"($merged_plugins)/($plugin_name)"

            # Remove existing yazelix plugin and copy fresh version
            if ($target | path exists) {
                rm -rf $target
            }
            cp -r $plugin_path $target
        }

        if not $quiet {
            print "     ✅ Yazelix plugins copied (user plugins preserved)"
        }
    }
}

# Copy bundled flavors (themes) directory
def copy_flavors_directory [source_dir: string, merged_dir: string, --quiet] {
    let source_flavors = $"($source_dir)/flavors"
    let merged_flavors = $"($merged_dir)/flavors"

    if not $quiet {
        print "   🎨 Copying flavor themes..."
    }

    # Ensure flavors directory exists
    if not ($merged_flavors | path exists) {
        mkdir $merged_flavors
    }

    # Copy yazelix bundled flavors (overwrites if they exist)
    # This preserves user-installed flavors that yazelix doesn't provide
    if ($source_flavors | path exists) {
        let bundled_flavors = (ls $source_flavors | where type == dir | get name)

        for flavor_path in $bundled_flavors {
            let flavor_name = ($flavor_path | path basename)
            let target = $"($merged_flavors)/($flavor_name)"

            # Remove existing yazelix flavor and copy fresh version
            if ($target | path exists) {
                rm -rf $target
            }
            cp -r $flavor_path $target
        }

        if not $quiet {
            print $"     ✅ ($bundled_flavors | length) flavor themes copied \(user flavors preserved\)"
        }
    }
}

# Generate yazi.toml with dynamic settings from yazelix.toml
def generate_yazi_toml [source_dir: string, merged_dir: string, sort_by: string, user_plugins: list, --quiet] {
    let base_path = $"($source_dir)/yazelix_yazi.toml"
    let user_path = (reconcile_yazi_user_file "yazi.toml")
    let merged_path = $"($merged_dir)/yazi.toml"

    if not $quiet {
        print "   📄 Generating yazi.toml with dynamic settings..."
    }

    # Read and parse base config
    let base_config = open $base_path

    # Check for user config and merge if exists
    let has_user_config = ($user_path | path exists)
    let merged_config = if $has_user_config {
        let user_config = open $user_path
        deep_merge $base_config $user_config
    } else {
        $base_config
    }

    # Preserve Yazelix's opener.edit (critical for Zellij integration)
    # Users can customize other openers but edit must use Yazelix's integration
    let config_with_opener = if ("opener" in ($base_config | columns)) and ("edit" in ($base_config.opener | columns)) {
        let yazelix_edit = $base_config.opener.edit
        if "opener" in ($merged_config | columns) {
            $merged_config | upsert opener.edit $yazelix_edit
        } else {
            $merged_config | upsert opener { edit: $yazelix_edit }
        }
    } else {
        $merged_config
    }

    # Remove git fetchers if git plugin is not in the list
    let config_without_git_fetchers = if ("git" not-in $user_plugins) {
        $config_with_opener | reject plugin?
    } else {
        $config_with_opener
    }

    # Add dynamic settings from yazelix.toml (yazelix.toml is the source of truth)
    let final_config = ($config_without_git_fetchers | upsert manager {
        sort_by: $sort_by
    })

    # Generate header
    let timestamp = (date now | format date '%Y-%m-%d %H:%M:%S')
    let user_note = if $has_user_config {
        "#\n# User config merged from:\n#   ~/.config/yazelix/user_configs/yazi/yazi.toml\n"
    } else {
        "#\n# To add custom settings, create:\n#   ~/.config/yazelix/user_configs/yazi/yazi.toml\n"
    }
    let header = [
        "# ========================================"
        "# AUTO-GENERATED YAZI CONFIG"
        "# ========================================"
        "# This file is automatically generated by Yazelix."
        "# Do not edit directly - changes will be lost!"
        $user_note
        "# Dynamic settings from ~/.config/yazelix/user_configs/yazelix.toml:"
        "#   [yazi] sort_by, plugins"
        "#"
        $"# Generated: ($timestamp)"
        "# ========================================"
        ""
    ] | str join "\n"

    # Write final config
    let config_content = ($final_config | to toml)
    $"($header)($config_content)" | save -f $merged_path

    if not $quiet {
        let user_msg = if $has_user_config { " \(+user yazi.toml\)" } else { "" }
        print $"     ✅ yazi.toml generated with sort_by: ($sort_by)($user_msg)"
    }
}

# Generate theme.toml with flavor configuration from yazelix.toml
def generate_theme_toml [source_dir: string, merged_dir: string, theme: string, --quiet] {
    let source_path = $"($source_dir)/yazelix_theme.toml"
    let merged_path = $"($merged_dir)/theme.toml"

    if not $quiet {
        print "   📄 Generating theme.toml with flavor configuration..."
    }

    # Read base theme config (if it exists for custom overrides)
    let base_theme = if ($source_path | path exists) {
        open $source_path
    } else {
        {}
    }

    # Add flavor configuration
    # Only set flavor if theme is not "default" (Yazi's built-in default)
    let flavor_config = if $theme != "default" and $theme != "random" {
        { flavor: { dark: $theme } }
    } else if $theme == "default" {
        {} # Don't set flavor for default theme
    } else {
        {} # Random was already resolved to actual theme name
    }

    # Merge base theme with flavor config (flavor takes precedence)
    let final_config = ($base_theme | merge $flavor_config)

    # Generate header
    let timestamp = (date now | format date '%Y-%m-%d %H:%M:%S')
    let header = [
        "# ========================================"
        "# AUTO-GENERATED YAZI THEME CONFIG"
        "# ========================================"
        "# This file is automatically generated by Yazelix."
        "# Do not edit directly - changes will be lost!"
        "#"
        "# To customize theme, edit:"
        "#   ~/.config/yazelix/user_configs/yazelix.toml"
        "#   [yazi] theme = \"...\""
        "#"
        $"# Current theme: ($theme)"
        $"# Generated: ($timestamp)"
        "# ========================================"
        ""
    ] | str join "\n"

    # Write final config
    let config_content = if ($final_config | is-empty) {
        ""  # Empty file if using default theme
    } else {
        $final_config | to toml
    }
    $"($header)($config_content)" | save -f $merged_path

    if not $quiet {
        print $"     ✅ theme.toml generated with flavor: ($theme)"
    }
}

# Generate keymap.toml with optional user keymap merging
def generate_keymap_toml [source_dir: string, merged_dir: string, --quiet] {
    let base_path = $"($source_dir)/yazelix_keymap.toml"
    let user_path = (reconcile_yazi_user_file "keymap.toml")
    let merged_path = $"($merged_dir)/keymap.toml"

    if not $quiet {
        print "   📄 Generating keymap.toml..."
    }

    # Read base keymap
    let base_keymap = open $base_path

    # Check for user keymap and merge if exists
    let has_user_keymap = ($user_path | path exists)
    let final_keymap = if $has_user_keymap {
        let user_keymap = open $user_path

        # Merge keymaps by concatenating arrays in each section
        # Structure: { mgr: { append_keymap: [...], prepend_keymap: [...] }, ... }
        let sections = ($base_keymap | columns)
        $sections | reduce --fold $base_keymap {|section, acc|
            if ($section in ($user_keymap | columns)) {
                let base_section = ($acc | get -o $section)
                let user_section = ($user_keymap | get -o $section)
                let subsections = ($base_section | columns)

                let merged_section = $subsections | reduce --fold $base_section {|sub, sec_acc|
                    if ($sub in ($user_section | columns)) {
                        let base_arr = ($sec_acc | get -o $sub)
                        let user_arr = ($user_section | get -o $sub)
                        $sec_acc | upsert $sub ($base_arr | append $user_arr)
                    } else {
                        $sec_acc
                    }
                }

                # Also add any new subsections from user
                let new_subsections = ($user_section | columns | where {|s| $s not-in $subsections})
                let final_section = $new_subsections | reduce --fold $merged_section {|sub, sec_acc|
                    $sec_acc | upsert $sub ($user_section | get -o $sub)
                }

                $acc | upsert $section $final_section
            } else {
                $acc
            }
        }
    } else {
        $base_keymap
    }

    # Generate header
    let timestamp = (date now | format date '%Y-%m-%d %H:%M:%S')
    let header = [
        "# ========================================"
        "# AUTO-GENERATED YAZI KEYMAP"
        "# ========================================"
        "# This file is automatically generated by Yazelix."
        "# Do not edit directly - changes will be lost!"
        "#"
        "# To add custom keybindings, create:"
        "#   ~/.config/yazelix/user_configs/yazi/keymap.toml"
        "#"
        $"# Generated: ($timestamp)"
        "# ========================================"
        ""
    ] | str join "\n"

    # Write final keymap
    let keymap_content = ($final_keymap | to toml)
    $"($header)($keymap_content)" | save -f $merged_path

    if not $quiet {
        let user_msg = if $has_user_keymap { " \(+user keymap\)" } else { "" }
        print $"     ✅ keymap.toml generated($user_msg)"
    }
}

# Generate init.lua dynamically based on plugin configuration
def generate_init_lua [merged_dir: string, user_plugins: list, --quiet] {
    let plugins_dir = $"($merged_dir)/plugins"

    # Core plugins - always loaded, cannot be disabled
    let core_plugins = ["sidebar-status", "auto-layout", "sidebar-state"]

    # Combine core + user plugins
    let all_plugins = ($core_plugins | append $user_plugins | uniq)

    # Check for missing plugins and warn
    let missing = ($all_plugins | where {|p|
        not ($"($plugins_dir)/($p).yazi" | path exists)
    })

    if ($missing | is-not-empty) {
        print $"⚠️  Warning: Missing plugins in yazelix.toml: ($missing | str join ', ')"
        print "   Install with: ya pkg add <owner/repo>"
        print "   Or remove from yazelix.toml [yazi] plugins list"
    }

    # Only load plugins that exist
    let valid_plugins = ($all_plugins | where {|p|
        ($"($plugins_dir)/($p).yazi" | path exists)
    })

    # Generate require\(\) statements with safe setup\(\) check
    # Some plugins don't have a setup\(\) function, so we check before calling
    let requires = ($valid_plugins | each {|name|
        if ($name in $core_plugins) {
            # Core plugins - we know they have setup\(\)
            $"-- Core plugin \(always loaded\)\nrequire\(\"($name)\"\):setup\(\)"
        } else if ($name == "starship") {
            # Starship plugin with custom sidebar-optimized config
            $"-- User plugin \(from yazelix.toml\)\nrequire\(\"starship\"\):setup\({\n    config_file = \"~/.config/yazelix/configs/yazi/yazelix_starship.toml\"\n}\)"
        } else {
            # User plugins - check if setup\(\) exists before calling
            $"-- User plugin \(from yazelix.toml\)\nlocal _($name | str replace -a '-' '_') = require\(\"($name)\"\)\nif type\(_($name | str replace -a '-' '_').setup\) == \"function\" then _($name | str replace -a '-' '_'):setup\(\) end"
        }
    } | str join "\n\n")

    # Generate final init.lua content
    let timestamp = (date now | format date '%Y-%m-%d %H:%M:%S')
    let header = [
        "-- ========================================"
        "-- AUTO-GENERATED YAZI INIT.LUA"
        "-- ========================================"
        "-- This file is automatically generated by Yazelix."
        "-- Do not edit directly - changes will be lost!"
        "--"
        "-- To customize plugins, edit:"
        "--   ~/.config/yazelix/user_configs/yazelix.toml"
        "--   [yazi] plugins = [...]"
        "--"
        "-- For custom Lua code, create:"
        "--   ~/.config/yazelix/user_configs/yazi/init.lua"
        "--"
        $"-- Generated: ($timestamp)"
        "-- ========================================"
        ""
    ] | str join "\n"

    let init_content = $"($header)($requires)\n"

    # Check for user custom init.lua and append if exists
    let user_init_path = (reconcile_yazi_user_file "init.lua")
    let final_content = if ($user_init_path | path exists) {
        let user_init = open $user_init_path --raw
        let user_section = [
            ""
            "-- ========================================"
            "-- USER CUSTOM CODE"
            "-- ========================================"
            "-- From: ~/.config/yazelix/user_configs/yazi/init.lua"
            "-- ========================================"
            ""
            $user_init
        ] | str join "\n"
        $"($init_content)($user_section)"
    } else {
        $init_content
    }

    # Write init.lua
    let init_path = $"($merged_dir)/init.lua"
    $final_content | save -f $init_path

    if not $quiet {
        let user_msg = if ($user_init_path | path exists) { " \(+user init.lua\)" } else { "" }
        print $"   ✅ Generated init.lua with ($valid_plugins | length) plugins($user_msg)"
    }
}

# Main function: Generate Yazi configuration
export def generate_merged_yazi_config [yazelix_dir: string, --quiet] {
    # Parse yazelix config to get settings
    let config = parse_yazelix_config
    let user_plugins = $config.yazi_plugins

    # Yazi flavor themes (25 total: 1 default + 19 dark + 5 light)
    # See: https://github.com/yazi-rs/flavors
    let yazi_themes_dark = [
        "catppuccin-mocha", "catppuccin-frappe", "catppuccin-macchiato",
        "dracula", "gruvbox-dark", "tokyo-night",
        "kanagawa", "kanagawa-dragon",
        "rose-pine", "rose-pine-moon",
        "flexoki-dark", "bluloco-dark",
        "ayu-dark", "everforest-medium", "ashen", "neon", "nord", "synthwave84", "monokai"
    ]

    let yazi_themes_light = [
        "catppuccin-latte",
        "kanagawa-lotus",
        "rose-pine-dawn",
        "flexoki-light", "bluloco-light"
    ]

    let theme_config = $config.yazi_theme
    let theme = if $theme_config == "random-dark" {
        $yazi_themes_dark | shuffle | first
    } else if $theme_config == "random-light" {
        $yazi_themes_light | shuffle | first
    } else {
        $theme_config
    }

    let sort_by = $config.yazi_sort_by

    # Define paths
    let state_dir = (get_yazelix_state_dir)
    let merged_config_dir = $"($state_dir)/configs/yazi"
    let source_config_dir = $"($yazelix_dir)/configs/yazi"

    if not $quiet {
        print "🔄 Generating Yazi configuration..."
    }

    # Ensure output directory exists
    ensure_dir $"($merged_config_dir)/yazi.toml"

    # Generate yazi.toml with dynamic settings from yazelix.toml
    generate_yazi_toml $source_config_dir $merged_config_dir $sort_by $user_plugins --quiet=$quiet

    # Generate theme.toml with flavor configuration from yazelix.toml
    generate_theme_toml $source_config_dir $merged_config_dir $theme --quiet=$quiet

    # Generate keymap.toml with optional user keymap merging
    generate_keymap_toml $source_config_dir $merged_config_dir --quiet=$quiet

    # Copy plugins directory
    copy_plugins_directory $source_config_dir $merged_config_dir --quiet=$quiet

    # Copy flavors (themes) directory
    copy_flavors_directory $source_config_dir $merged_config_dir --quiet=$quiet

    # Generate init.lua dynamically based on plugin configuration
    generate_init_lua $merged_config_dir $user_plugins --quiet=$quiet

    if not $quiet {
        print $"✅ Yazi configuration generated successfully!"
        print $"   📁 Config saved to: ($merged_config_dir)"
    }

    $merged_config_dir
}

# Export main function for external use
export def main [yazelix_dir: string, --quiet] {
    generate_merged_yazi_config $yazelix_dir --quiet=$quiet | ignore
}
