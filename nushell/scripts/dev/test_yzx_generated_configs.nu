#!/usr/bin/env nu

use ./test_yzx_helpers.nu [get_repo_config_dir repo_path]
use ../utils/launch_state.nu [get_launch_env]
use ../utils/ascii_art.nu [
    get_boids_animation_frames
    get_life_animation_frames
    get_logo_animation_frames
    get_logo_welcome_frame
    get_logo_welcome_variant
    get_max_visible_width
    get_welcome_style_random_pool
    resolve_welcome_style
]
use ../setup/yazi_config_merger.nu [generate_merged_yazi_config]
use ../setup/zellij_config_merger.nu [generate_merged_zellij_config]
use ../utils/terminal_launcher.nu [resolve_terminal_config]
use ../utils/terminal_configs.nu [
    generate_all_terminal_configs
]

def test_layout_generator_discovers_custom_top_level_layouts [] {
    print "🧪 Testing layout generator discovers custom top-level layouts..."

    let tmpdir = (^mktemp -d /tmp/yazelix_layout_generator_XXXXXX | str trim)

    let result = (try {
        let source_dir = ($tmpdir | path join "source")
        let target_dir = ($tmpdir | path join "target")
        let fragments_dir = ($source_dir | path join "fragments")
        let repo_fragments_dir = (repo_path "configs" "zellij" "layouts" "fragments")

        mkdir $source_dir
        mkdir $fragments_dir

        for fragment in [
            "zjstatus_tab_template.kdl"
            "keybinds_common.kdl"
            "swap_sidebar_open.kdl"
            "swap_sidebar_closed.kdl"
        ] {
            ^cp ($repo_fragments_dir | path join $fragment) ($fragments_dir | path join $fragment)
        }

        let custom_layout_path = ($source_dir | path join "custom_layout.kdl")
        'layout { pane }' | save --force --raw $custom_layout_path

        use ../utils/layout_generator.nu *
        generate_all_layouts $source_dir $target_dir ["editor"] "" "file:/tmp/yazelix_pane_orchestrator.wasm" $source_dir

        let generated_layout_path = ($target_dir | path join "custom_layout.kdl")
        let generated_fragments_dir = ($target_dir | path join "fragments")

        if ($generated_layout_path | path exists) and not ($generated_fragments_dir | path exists) {
            print "  ✅ Layout generator copies custom top-level layouts without copying fragments"
            true
        } else {
            print $"  ❌ Unexpected result: custom_exists=(($generated_layout_path | path exists)) fragments_copied=(($generated_fragments_dir | path exists))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_layout_generator_rewrites_runtime_paths [] {
    print "🧪 Testing layout generator rewrites runtime-root placeholders..."

    let tmpdir = (^mktemp -d /tmp/yazelix_layout_runtime_XXXXXX | str trim)

    let result = (try {
        let source_dir = ($tmpdir | path join "source")
        let target_dir = ($tmpdir | path join "target")
        let repo_layouts_dir = (repo_path "configs" "zellij" "layouts")
        let runtime_dir = ($tmpdir | path join "runtime")

        mkdir $source_dir
        mkdir $runtime_dir
        for entry in (ls $repo_layouts_dir) {
            let target_path = ($source_dir | path join ($entry.name | path basename))
            if $entry.type == dir {
                ^cp -R $entry.name $target_path
            } else {
                ^cp $entry.name $target_path
            }
        }

        use ../utils/layout_generator.nu *
        generate_all_layouts $source_dir $target_dir ["editor"] "" "file:/tmp/yazelix_pane_orchestrator.wasm" $runtime_dir

        let generated_layout = (open --raw ($target_dir | path join "yzx_side.kdl"))

        if (
            ($generated_layout | str contains $"($runtime_dir)/configs/zellij/scripts/launch_sidebar_yazi.nu")
            and ($generated_layout | str contains $"file:($runtime_dir)/configs/zellij/plugins/zjstatus.wasm")
            and ($generated_layout | str contains $"nu ($runtime_dir)/nushell/scripts/utils/zjstatus_widget.nu shell")
            and not ($generated_layout | str contains "~/.config/yazelix")
        ) {
            print "  ✅ Generated layouts stamp the configured runtime root into wrapper and widget paths"
            true
        } else {
            print "  ❌ Generated layouts still contain stale repo-shaped runtime paths"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_launch_env_omits_default_helix_runtime [] {
    print "🧪 Testing launch env omits HELIX_RUNTIME by default..."

    try {
        let cfg = {
            editor_command: "hx"
            helix_runtime_path: null
            terminals: ["ghostty"]
            default_shell: "nu"
            debug_mode: false
            enable_sidebar: true
            welcome_style: "static"
            terminal_config_mode: "yazelix"
        }
        let stdout = (get_launch_env $cfg "/tmp/yazelix-profile" | get -o HELIX_RUNTIME | default "" | str trim)

        if $stdout == "" {
            print "  ✅ HELIX_RUNTIME is omitted unless explicitly configured"
            true
        } else {
            print $"  ❌ Unexpected result: stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_launch_env_keeps_custom_helix_runtime_override [] {
    print "🧪 Testing launch env preserves custom Helix runtime override..."

    try {
        let cfg = {
            editor_command: "hx"
            helix_runtime_path: "/tmp/custom-helix-runtime"
            terminals: ["ghostty"]
            default_shell: "nu"
            debug_mode: false
            enable_sidebar: true
            welcome_style: "static"
            terminal_config_mode: "yazelix"
        }
        let stdout = (get_launch_env $cfg "/tmp/yazelix-profile" | get HELIX_RUNTIME | str trim)

        if $stdout == "/tmp/custom-helix-runtime" {
            print "  ✅ Custom helix_runtime_path is still exported"
            true
        } else {
            print $"  ❌ Unexpected result: stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_generate_all_terminal_configs_creates_override_scaffolds [] {
    print "🧪 Testing bundled terminal config generation creates Yazelix-specific override scaffolds..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_override_scaffold_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    mkdir $fake_home

    let result = (try {
        '[terminal]
terminals = ["ghostty", "kitty", "alacritty", "wezterm", "foot"]
' | save --force --raw $config_path

        with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_DIR: ($fake_home | path join ".config" "yazelix")
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            generate_all_terminal_configs $runtime_root
        }

        let override_root = ($fake_home | path join ".config" "yazelix" "terminal_overrides")
        let ghostty_override = ($override_root | path join "ghostty")
        let kitty_override = ($override_root | path join "kitty.conf")
        let alacritty_override = ($override_root | path join "alacritty.toml")
        let ghostty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "config"))
        let kitty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "kitty" "kitty.conf"))
        let alacritty_entrypoint = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "alacritty" "alacritty.toml"))
        let wezterm_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "wezterm" ".wezterm.lua"))
        let foot_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "foot" "foot.ini"))

        if (
            ($ghostty_override | path exists)
            and ($kitty_override | path exists)
            and ($alacritty_override | path exists)
            and ((open --raw $ghostty_override) | str contains "Personal Ghostty overrides")
            and ((open --raw $kitty_override) | str contains "Personal Kitty overrides")
            and ((open --raw $alacritty_override) | str contains "Personal Alacritty overrides")
            and ($ghostty_config | str contains $"config-file = ?\"($ghostty_override)\"")
            and ($kitty_config | str contains $"include ($kitty_override)")
            and ($alacritty_entrypoint | str contains $"\"($fake_home)/.local/share/yazelix/configs/terminal_emulators/alacritty/alacritty_base.toml\"")
            and ($alacritty_entrypoint | str contains $"\"($fake_home)/.config/yazelix/terminal_overrides/alacritty.toml\"")
            and not ($ghostty_config | str contains "start_yazelix.sh")
            and not ($kitty_config | str contains "start_yazelix.sh")
            and not ($alacritty_entrypoint | str contains "start_yazelix.sh")
            and not ($wezterm_config | str contains "start_yazelix.sh")
            and not ($foot_config | str contains "start_yazelix.sh")
        ) {
            print "  ✅ Terminal config generation creates override scaffolds and keeps startup out of generated terminal configs"
            true
        } else {
            print "  ❌ Override scaffold generation did not produce the expected files or imports"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_terminal_override_scaffolds_ignore_yazelix_dir_runtime_root [] {
    print "🧪 Testing terminal override scaffolds ignore YAZELIX_DIR runtime roots..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_override_path_boundary_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let fake_runtime_root = ($tmpdir | path join "runtime_root")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    let terminal_configs_script = ($runtime_root | path join "nushell" "scripts" "utils" "terminal_configs.nu")
    mkdir $fake_home
    mkdir $fake_runtime_root

    let result = (try {
        '[terminal]
terminals = ["ghostty", "kitty", "alacritty"]
' | save --force --raw $config_path

        let command_output = (with-env {
            HOME: $fake_home
            YAZELIX_DIR: $fake_runtime_root
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            ^nu -c $"use \"($terminal_configs_script)\" [generate_all_terminal_configs]; generate_all_terminal_configs \"($runtime_root)\"" | complete
        })

        let expected_override_root = ($fake_home | path join ".config" "yazelix" "terminal_overrides")
        let misplaced_override_root = ($fake_runtime_root | path join "terminal_overrides")

        if (
            ($command_output.exit_code == 0)
            and ($expected_override_root | path exists)
            and (($expected_override_root | path join "ghostty") | path exists)
            and (($expected_override_root | path join "kitty.conf") | path exists)
            and (($expected_override_root | path join "alacritty.toml") | path exists)
            and not ($misplaced_override_root | path exists)
        ) {
            print "  ✅ Override scaffolds stay under HOME/.config/yazelix even when YAZELIX_DIR points elsewhere"
            true
        } else {
            print $"  ❌ Unexpected override destinations: exit=($command_output.exit_code) expected_root_exists=(($expected_override_root | path exists)) misplaced_root_exists=(($misplaced_override_root | path exists)) expected_root=($expected_override_root) misplaced_root=($misplaced_override_root) stderr=(($command_output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_parse_yazelix_config_reads_pack_sidecar [] {
    print "🧪 Testing parse_yazelix_config reads yazelix_packs.toml as the pack source when present..."

    let tmpdir = (^mktemp -d /tmp/yazelix_pack_sidecar_parse_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let pack_path = ($tmpdir | path join "yazelix_packs.toml")

        '[core]
debug_mode = false
' | save --force --raw $config_path

        'enabled = ["git", "rust"]
user_packages = ["docker", "kubectl"]

[declarations]
git = ["gh", "prek"]
rust = ["rust_toolchain"]
' | save --force --raw $pack_path

        let parsed = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })

        if (
            ($parsed.pack_names == ["git", "rust"])
            and ($parsed.user_packages == ["docker", "kubectl"])
            and (($parsed.pack_declarations | get git) == ["gh", "prek"])
            and (($parsed.pack_declarations | get rust) == ["rust_toolchain"])
        ) {
            print "  ✅ parse_yazelix_config loads packs from the dedicated sidecar without needing [packs] in yazelix.toml"
            true
        } else {
            print $"  ❌ Unexpected parsed pack config: ($parsed | select pack_names user_packages pack_declarations | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_parse_yazelix_config_reads_core_welcome_style [] {
    print "🧪 Testing parse_yazelix_config reads core.welcome_style..."

    let tmpdir = (^mktemp -d /tmp/yazelix_welcome_style_parse_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")

        '[core]
welcome_style = "mandelbrot"
' | save --force --raw $config_path

        let parsed = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })

        if ($parsed.welcome_style == "mandelbrot") {
            print "  ✅ parse_yazelix_config reads the new core.welcome_style field"
            true
        } else {
            print $"  ❌ Unexpected parsed welcome style: ($parsed.welcome_style)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_logo_welcome_variant_adapts_to_width [] {
    print "🧪 Testing logo welcome variant selection adapts to terminal width..."

    try {
        let narrow = (get_logo_welcome_variant 36)
        let medium = (get_logo_welcome_variant 60)
        let wide = (get_logo_welcome_variant 100)
        let hero = (get_logo_welcome_variant 150)

        if ($narrow == "narrow") and ($medium == "medium") and ($wide == "wide") and ($hero == "hero") {
            print "  ✅ Logo welcome style picks narrow, medium, wide, and hero variants at the expected widths"
            true
        } else {
            print $"  ❌ Unexpected variants: narrow=($narrow) medium=($medium) wide=($wide) hero=($hero)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_logo_welcome_frame_respects_width_budget [] {
    print "🧪 Testing logo welcome frames stay within the selected width budget..."

    try {
        let narrow_frame = (get_logo_welcome_frame 36)
        let medium_frame = (get_logo_welcome_frame 60)
        let wide_frame = (get_logo_welcome_frame 100)
        let hero_frame = (get_logo_welcome_frame 150)

        let narrow_width = (get_max_visible_width $narrow_frame)
        let medium_width = (get_max_visible_width $medium_frame)
        let wide_width = (get_max_visible_width $wide_frame)
        let hero_width = (get_max_visible_width $hero_frame)

        if (
            ($narrow_width <= 36)
            and ($medium_width <= 60)
            and ($wide_width <= 100)
            and ($hero_width <= 150)
            and ($hero_width > $wide_width)
            and (($narrow_frame | length) >= 4)
        ) {
            print "  ✅ Logo welcome frames stay inside their target widths and expand meaningfully on large terminals"
            true
        } else {
            print $"  ❌ Unexpected frame widths: narrow=($narrow_width) medium=($medium_width) wide=($wide_width) hero=($hero_width)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_logo_animation_lands_on_static_resting_frame [] {
    print "🧪 Testing logo animation lands on the same final branded frame as static mode..."

    try {
        let static_frame = (get_logo_welcome_frame 60)
        let animation_frames = (get_logo_animation_frames 60)
        let final_frame = ($animation_frames | last)

        if ($final_frame == $static_frame) and (($animation_frames | length) >= 4) {
            print "  ✅ Logo animation resolves cleanly into the same resting frame static mode uses"
            true
        } else {
            print $"  ❌ Unexpected animation landing state: frames=(($animation_frames | length))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_boids_animation_stays_bounded_and_width_aware [] {
    print "🧪 Testing boids welcome frames stay bounded and fit the chosen width..."

    try {
        let narrow_frames = (get_boids_animation_frames 36)
        let medium_frames = (get_boids_animation_frames 60)
        let wide_frames = (get_boids_animation_frames 100)
        let hero_frames = (get_boids_animation_frames 150)

        let narrow_width = ($narrow_frames | each {|frame| get_max_visible_width $frame } | math max)
        let medium_width = ($medium_frames | each {|frame| get_max_visible_width $frame } | math max)
        let wide_width = ($wide_frames | each {|frame| get_max_visible_width $frame } | math max)
        let hero_width = ($hero_frames | each {|frame| get_max_visible_width $frame } | math max)

        if (
            (($narrow_frames | length) == 4)
            and (($medium_frames | length) == 4)
            and (($wide_frames | length) == 4)
            and (($hero_frames | length) == 4)
            and ($narrow_width <= 36)
            and ($medium_width <= 60)
            and ($wide_width <= 100)
            and ($hero_width <= 150)
            and ($hero_width > $wide_width)
        ) {
            print "  ✅ Boids welcome generation stays bounded and scales up on large terminals"
            true
        } else {
            print $"  ❌ Unexpected boids frame budgets: narrow_frames=(($narrow_frames | length)) narrow_width=($narrow_width) medium_width=($medium_width) wide_width=($wide_width) hero_width=($hero_width)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_boids_animation_lands_on_logo_resting_frame [] {
    print "🧪 Testing boids animation lands on the shared logo resting frame..."

    try {
        let static_logo = (get_logo_welcome_frame 60)
        let boids_frames = (get_boids_animation_frames 60)
        let final_frame = ($boids_frames | last)

        if ($final_frame == $static_logo) {
            print "  ✅ Boids animation resolves into the same readable final frame as the logo baseline"
            true
        } else {
            print "  ❌ Boids animation does not land on the shared final logo frame"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_life_animation_is_deterministic_and_evolves [] {
    print "🧪 Testing life welcome generation is deterministic and evolves before landing on the logo frame..."

    try {
        let frames_a = (get_life_animation_frames 60)
        let frames_b = (get_life_animation_frames 60)
        let frame0 = ($frames_a | get 0)
        let frame1 = ($frames_a | get 1)
        let frame2 = ($frames_a | get 2)

        if (
            ($frames_a == $frames_b)
            and ($frame0 != $frame1)
            and ($frame1 != $frame2)
            and (($frames_a | length) == 4)
        ) {
            print "  ✅ Life welcome generation is deterministic and advances through real intermediate states"
            true
        } else {
            print $"  ❌ Unexpected life animation progression: frames=(($frames_a | length)) deterministic=(($frames_a == $frames_b)) frame0_eq_frame1=(($frame0 == $frame1)) frame1_eq_frame2=(($frame1 == $frame2))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_life_animation_stays_bounded_and_width_aware [] {
    print "🧪 Testing life welcome frames stay bounded and fit the chosen width..."

    try {
        let narrow_frames = (get_life_animation_frames 36)
        let medium_frames = (get_life_animation_frames 60)
        let wide_frames = (get_life_animation_frames 100)
        let hero_frames = (get_life_animation_frames 150)

        let narrow_width = ($narrow_frames | each {|frame| get_max_visible_width $frame } | math max)
        let medium_width = ($medium_frames | each {|frame| get_max_visible_width $frame } | math max)
        let wide_width = ($wide_frames | each {|frame| get_max_visible_width $frame } | math max)
        let hero_width = ($hero_frames | each {|frame| get_max_visible_width $frame } | math max)

        if (
            (($narrow_frames | length) == 4)
            and (($medium_frames | length) == 4)
            and (($wide_frames | length) == 4)
            and (($hero_frames | length) == 4)
            and ($narrow_width <= 36)
            and ($medium_width <= 60)
            and ($wide_width <= 100)
            and ($hero_width <= 150)
            and ($hero_width > $wide_width)
        ) {
            print "  ✅ Life welcome generation stays bounded and scales up on large terminals"
            true
        } else {
            print $"  ❌ Unexpected life frame budgets: narrow_frames=(($narrow_frames | length)) narrow_width=($narrow_width) medium_width=($medium_width) wide_width=($wide_width) hero_width=($hero_width)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_life_animation_lands_on_logo_resting_frame [] {
    print "🧪 Testing life animation lands on the shared logo resting frame..."

    try {
        let static_logo = (get_logo_welcome_frame 60)
        let life_frames = (get_life_animation_frames 60)
        let final_frame = ($life_frames | last)

        if ($final_frame == $static_logo) {
            print "  ✅ Life animation resolves into the same readable final frame as the logo baseline"
            true
        } else {
            print "  ❌ Life animation does not land on the shared final logo frame"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_parse_yazelix_config_rejects_legacy_ascii_mode_with_migration_guidance [] {
    print "🧪 Testing parse_yazelix_config rejects legacy [ascii].mode with one clean migration path..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_welcome_style_legacy_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir

    let result = (try {
        '[ascii]
mode = "animated"
' | save --force --raw ($temp_config_dir | path join "yazelix.toml")

        let parser_result = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($repo_root | path join "nushell" "scripts" "utils" "config_parser.nu")\" [parse_yazelix_config]; parse_yazelix_config" | complete
        })

        let stderr = ($parser_result.stderr | str trim)

        if (
            ($parser_result.exit_code != 0)
            and ($stderr | str contains "Known migration at ascii")
            and ($stderr | str contains "Replace legacy [ascii].mode with core.welcome_style")
            and ($stderr | str contains "yzx config migrate --apply")
            and not ($stderr | str contains "Unknown config field at ascii")
        ) {
            print "  ✅ Legacy [ascii].mode now points at one clean migration path during startup"
            true
        } else {
            print $"  ❌ Unexpected parser result: exit=($parser_result.exit_code) stderr=($stderr)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_welcome_style_random_pool_excludes_static [] {
    print "🧪 Testing welcome_style random pool excludes static and only resolves animated styles..."

    try {
        let pool = (get_welcome_style_random_pool)
        let resolved = (
            0..3
            | each {|index| resolve_welcome_style "random" $index }
            | uniq
        )

        if (
            ($pool == ["logo", "boids", "life", "mandelbrot"])
            and ("static" not-in $pool)
            and ("static" not-in $resolved)
            and ($resolved | all {|style| $style in $pool })
        ) {
            print "  ✅ random welcome selection excludes static and resolves only through the animated pool"
            true
        } else {
            print $"  ❌ Unexpected pool/resolution: pool=($pool | to json -r) resolved=($resolved | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_parse_yazelix_config_bootstraps_split_default_surfaces [] {
    print "🧪 Testing parse_yazelix_config bootstraps both default config surfaces on first run..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_pack_bootstrap_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")

    let result = (try {
        let parsed = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })

        let main_exists = (($temp_config_dir | path join "yazelix.toml") | path exists)
        let pack_exists = (($temp_config_dir | path join "yazelix_packs.toml") | path exists)
        let generated_main = (if $main_exists { open --raw ($temp_config_dir | path join "yazelix.toml") } else { "" })
        let generated_packs = (if $pack_exists { open --raw ($temp_config_dir | path join "yazelix_packs.toml") } else { "" })

        if (
            $main_exists
            and $pack_exists
            and ($generated_main | str contains "Pack configuration lives in ~/.config/yazelix/yazelix_packs.toml")
            and ($generated_packs | str contains "[declarations]")
            and ((($parsed.pack_declarations | default {}) | columns | length) > 0)
        ) {
            print "  ✅ First-run bootstrap now materializes both yazelix.toml and yazelix_packs.toml from runtime defaults"
            true
        } else {
            print $"  ❌ Unexpected result: main_exists=($main_exists) pack_exists=($pack_exists) parsed=($parsed | select pack_names pack_declarations | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_parse_yazelix_config_bootstraps_welcome_style_surface [] {
    print "🧪 Testing first-run bootstrap writes welcome_style into the generated main config..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_welcome_bootstrap_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")

    let result = (try {
        let parsed = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })

        let main_path = ($temp_config_dir | path join "yazelix.toml")
        let generated_main = (if ($main_path | path exists) { open --raw $main_path } else { "" })

        if (
            ($main_path | path exists)
            and ($parsed.welcome_style == "random")
            and ($generated_main | str contains 'welcome_style = "random"')
            and not ($generated_main | str contains "[ascii]")
        ) {
            print "  ✅ First-run bootstrap writes the new welcome_style surface into yazelix.toml"
            true
        } else {
            print $"  ❌ Unexpected bootstrap result: main_exists=((($main_path | path exists))) parsed=($parsed.welcome_style) main=($generated_main)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_parse_yazelix_config_rejects_legacy_main_file_packs_with_migration_guidance [] {
    print "🧪 Testing parse_yazelix_config rejects legacy [packs] in yazelix.toml and points users at migrate..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_pack_legacy_main_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir

    let result = (try {
        '[packs]
enabled = ["git"]
user_packages = ["docker"]

[packs.declarations]
git = ["gh", "prek"]
' | save --force --raw ($temp_config_dir | path join "yazelix.toml")

        let parser_result = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            ^nu -c $"use \"($repo_root | path join "nushell" "scripts" "utils" "config_parser.nu")\" [parse_yazelix_config]; parse_yazelix_config" | complete
        })

        let stderr = ($parser_result.stderr | str trim)

        if (
            ($parser_result.exit_code != 0)
            and ($stderr | str contains "Known migration at packs")
            and ($stderr | str contains "yzx config migrate --apply")
        ) {
            print "  ✅ Legacy pack settings are now blocked with shared migration guidance"
            true
        } else {
            print $"  ❌ Unexpected parser result: exit=($parser_result.exit_code) stderr=($stderr)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

def test_parse_yazelix_config_rejects_split_pack_ownership [] {
    print "🧪 Testing parse_yazelix_config fails fast when yazelix.toml and yazelix_packs.toml both define packs..."

    let tmpdir = (^mktemp -d /tmp/yazelix_pack_sidecar_conflict_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let pack_path = ($tmpdir | path join "yazelix_packs.toml")
        let parser_script = (repo_path "nushell" "scripts" "utils" "config_parser.nu")

        '[packs]
enabled = ["git"]
' | save --force --raw $config_path

        'enabled = ["rust"]
' | save --force --raw $pack_path

        let output = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            ^nu -c $"source \"($parser_script)\"; try { parse_yazelix_config | ignore } catch {|err| print $err.msg }" | complete
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix found pack settings in both yazelix.toml and yazelix_packs.toml.")
            and ($stdout | str contains "fully owns pack settings")
            and ($stdout | str contains "Failure class: config problem.")
        ) {
            print "  ✅ parse_yazelix_config fails fast on ambiguous split pack ownership"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_user_mode_requires_real_terminal_config [] {
    print "🧪 Testing terminal.config_mode = user fails fast when the user terminal config is missing..."

    let fake_home = (^mktemp -d /tmp/yazelix_user_mode_missing_XXXXXX | str trim)

    let result = (try {
        let message = (with-env { HOME: $fake_home } {
            try {
                resolve_terminal_config "ghostty" "user"
                "unexpected-success"
            } catch {|err|
                $err.msg
            }
        })

        if ($message | str contains "terminal.config_mode = user requires a real ghostty user config") {
            print "  ✅ user mode fails clearly instead of silently falling back to Yazelix-managed config"
            true
        } else {
            print $"  ❌ Unexpected message: ($message)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fake_home
    $result
}

def test_config_schema_rejects_removed_auto_terminal_config_mode [] {
    print "🧪 Testing config schema rejects the removed terminal.config_mode = auto value..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_mode_schema_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        '[terminal]
config_mode = "auto"
' | save --force --raw $config_path

        let findings = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_schema.nu [validate_enum_values]
            validate_enum_values (open $config_path)
        })
        let mode_findings = ($findings | where path == "terminal.config_mode")

        if (
            (($mode_findings | length) == 1)
            and (($mode_findings | get 0.kind) == "invalid_enum")
        ) {
            print "  ✅ Config schema rejects the removed auto terminal config mode"
            true
        } else {
            print $"  ❌ Unexpected findings: ($mode_findings | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_launch_env_omits_yazelix_default_shell [] {
    print "🧪 Testing launch env omits YAZELIX_DEFAULT_SHELL..."

    try {
        let cfg = {
            editor_command: "hx"
            helix_runtime_path: null
            terminals: ["ghostty"]
            default_shell: "fish"
            debug_mode: false
            enable_sidebar: true
            welcome_style: "static"
            terminal_config_mode: "yazelix"
        }
        let stdout = (get_launch_env $cfg "/tmp/yazelix-profile" | get -o YAZELIX_DEFAULT_SHELL | default "" | str trim)

        if $stdout == "" {
            print "  ✅ YAZELIX_DEFAULT_SHELL is no longer part of the launch env"
            true
        } else {
            print $"  ❌ Unexpected result: stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zjstatus_widget_reads_shell_from_config [] {
    print "🧪 Testing zjstatus shell widget reads current config..."

    try {
        let widget_script = (repo_path "nushell" "scripts" "utils" "zjstatus_widget.nu")
        let tmpdir = (^mktemp -d /tmp/yazelix_widget_shell_XXXXXX | str trim)
        '[shell]
default_shell = "nu"
' | save --force --raw ($tmpdir | path join "yazelix.toml")
        let output = (with-env {
            YAZELIX_CONFIG_OVERRIDE: ($tmpdir | path join "yazelix.toml")
            YAZELIX_DEFAULT_SHELL: "fish"
        } {
            ^nu $widget_script shell | complete
        })
        rm -rf $tmpdir
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "nu") {
            print "  ✅ Shell widget ignores stale YAZELIX_DEFAULT_SHELL env"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zjstatus_widget_reads_editor_from_config [] {
    print "🧪 Testing zjstatus editor widget reads current config..."

    try {
        let widget_script = (repo_path "nushell" "scripts" "utils" "zjstatus_widget.nu")
        let tmpdir = (^mktemp -d /tmp/yazelix_widget_editor_XXXXXX | str trim)
        '[editor]
command = "nvim --headless"
' | save --force --raw ($tmpdir | path join "yazelix.toml")
        let output = (with-env {
            YAZELIX_CONFIG_OVERRIDE: ($tmpdir | path join "yazelix.toml")
            EDITOR: "fish"
        } {
            ^nu $widget_script editor | complete
        })
        rm -rf $tmpdir
        let stdout = ($output.stdout | str trim)

        if ($output.exit_code == 0) and ($stdout == "nvim") {
            print "  ✅ Editor widget ignores stale EDITOR env"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zellij_widget_tray_defaults_omit_layout [] {
    print "🧪 Testing zellij.widget_tray defaults omit the broken layout widget..."

    let tmpdir = (^mktemp -d /tmp/yazelix_widget_tray_default_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        '' | save --force --raw $config_path

        let parsed = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })
        let tray = ($parsed | get zellij_widget_tray)

        if $tray == ["editor", "shell", "term", "cpu", "ram"] {
            print "  ✅ Default widget tray no longer includes the layout widget"
            true
        } else {
            print $"  ❌ Unexpected default widget tray: ($tray | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_zjstatus_custom_text_is_trimmed_and_truncated_in_config_parser [] {
    print "🧪 Testing zjstatus custom text is normalized in config parsing..."

    let tmpdir = (^mktemp -d /tmp/yazelix_widget_custom_text_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        '[zellij]
custom_text = "  notes[]{}12345  "
' | save --force --raw $config_path

        let parsed = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })
        let normalized = ($parsed | get zellij_custom_text)

        if $normalized == "notes123" {
            print "  ✅ Config parser trims, sanitizes, and caps zjstatus custom text to 8 characters"
            true
        } else {
            print $"  ❌ Unexpected result: normalized=($normalized)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_ghostty_trail_glow_defaults_to_medium_and_reads_explicit_levels [] {
    print "🧪 Testing Ghostty trail glow defaults to medium and parses explicit levels..."

    let tmpdir = (^mktemp -d /tmp/yazelix_ghostty_glow_parse_XXXXXX | str trim)

    let result = (try {
        let default_config_path = ($tmpdir | path join "default.toml")
        let explicit_config_path = ($tmpdir | path join "explicit.toml")

        '[terminal]
ghostty_trail_color = "blaze"
' | save --force --raw $default_config_path

        '[terminal]
ghostty_trail_color = "blaze"
ghostty_trail_glow = "none"
' | save --force --raw $explicit_config_path

        let parsed_default = (with-env { YAZELIX_CONFIG_OVERRIDE: $default_config_path } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })
        let parsed_explicit = (with-env { YAZELIX_CONFIG_OVERRIDE: $explicit_config_path } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })

        if ($parsed_default.ghostty_trail_glow == "medium") and ($parsed_explicit.ghostty_trail_glow == "none") {
            print "  ✅ Ghostty trail glow defaults to medium and honors explicit enum values"
            true
        } else {
            print $"  ❌ Unexpected values: default=($parsed_default.ghostty_trail_glow) explicit=($parsed_explicit.ghostty_trail_glow)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_generate_all_terminal_configs_honors_ghostty_trail_color_none [] {
    print "🧪 Testing ghostty_trail_color = none disables the palette shader and Kitty fallback trail..."

    let tmpdir = (^mktemp -d /tmp/yazelix_ghostty_trail_none_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    let terminal_configs_script = ($runtime_root | path join "nushell" "scripts" "utils" "terminal_configs.nu")
    mkdir $fake_home

    let result = (try {
        '[terminal]
terminals = ["ghostty", "kitty"]
ghostty_trail_color = "none"
' | save --force --raw $config_path

        let findings = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_schema.nu [validate_enum_values]
            validate_enum_values (open $config_path)
        })
        let color_findings = ($findings | where path == "terminal.ghostty_trail_color")

        let command_output = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            ^nu -c $"use \"($terminal_configs_script)\" [generate_all_terminal_configs]; generate_all_terminal_configs \"($runtime_root)\"" | complete
        })

        let ghostty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "config"))
        let kitty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "kitty" "kitty.conf"))

        if (
            (($color_findings | is-empty))
            and ($command_output.exit_code == 0)
            and ($ghostty_config | str contains '# Cursor color palette: none (disable Yazelix color trail palette)')
            and (not ($ghostty_config | str contains 'cursor-color = '))
            and (not ($ghostty_config | str contains 'custom-shader = ./shaders/cursor_trail_'))
            and ($kitty_config | str contains '# cursor_trail 0  # ghostty_trail_color = none disables the fallback trail')
        ) {
            print "  ✅ ghostty_trail_color = none cleanly disables the palette shader and Kitty fallback trail"
            true
        } else {
            print $"  ❌ Unexpected result: findings=($color_findings | to json -r) exit=($command_output.exit_code) stderr=(($command_output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_config_schema_rejects_invalid_ghostty_trail_glow [] {
    print "🧪 Testing config schema rejects invalid Ghostty trail glow levels..."

    let tmpdir = (^mktemp -d /tmp/yazelix_ghostty_glow_schema_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        '[terminal]
ghostty_trail_glow = "ultra"
' | save --force --raw $config_path

        let findings = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_schema.nu [validate_enum_values]
            validate_enum_values (open $config_path)
        })
        let glow_findings = ($findings | where path == "terminal.ghostty_trail_glow")

        if (
            (($glow_findings | length) == 1)
            and (($glow_findings | get 0.kind) == "invalid_enum")
        ) {
            print "  ✅ Config schema rejects unsupported Ghostty trail glow enum values"
            true
        } else {
            print $"  ❌ Unexpected findings: ($glow_findings | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_config_schema_rejects_removed_layout_widget [] {
    print "🧪 Testing config schema rejects the removed zellij layout widget..."

    let tmpdir = (^mktemp -d /tmp/yazelix_widget_tray_schema_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        '[zellij]
widget_tray = ["layout", "editor"]
' | save --force --raw $config_path

        let findings = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_schema.nu [validate_enum_values]
            validate_enum_values (open $config_path)
        })
        let tray_findings = ($findings | where path == "zellij.widget_tray")

        if (
            (($tray_findings | length) == 1)
            and (($tray_findings | get 0.kind) == "invalid_enum")
            and ((($tray_findings | get 0.message) | str contains "layout"))
        ) {
            print "  ✅ Config schema rejects the removed layout widget entry"
            true
        } else {
            print $"  ❌ Unexpected findings: ($tray_findings | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_generate_all_terminal_configs_honors_ghostty_trail_glow [] {
    print "🧪 Testing Ghostty terminal config generation propagates trail glow into shaders..."

    let tmpdir = (^mktemp -d /tmp/yazelix_ghostty_glow_gen_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    let terminal_configs_script = ($runtime_root | path join "nushell" "scripts" "utils" "terminal_configs.nu")
    mkdir $fake_home

    let result = (try {
        '[terminal]
terminals = ["ghostty"]
ghostty_trail_color = "blaze"
ghostty_trail_effect = "sweep"
ghostty_mode_effect = "ripple"
ghostty_trail_glow = "none"
' | save --force --raw $config_path

        let command_output = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            ^nu -c $"use \"($terminal_configs_script)\" [generate_all_terminal_configs]; generate_all_terminal_configs \"($runtime_root)\"" | complete
        })

        let blaze_shader = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "shaders" "cursor_trail_blaze.glsl"))
        let sweep_shader = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "shaders" "generated_effects" "sweep.glsl"))
        let ripple_shader = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "shaders" "generated_effects" "ripple.glsl"))
        let sweep_length_matches = ($sweep_shader | parse -r 'const float TRAIL_LENGTH = (?<value>[0-9.eE-]+);')
        let sweep_length_value = if ($sweep_length_matches | is-empty) { null } else { $sweep_length_matches | get 0.value | into float }
        let blur_matches = ($ripple_shader | parse -r 'const float BLUR = (?<value>[0-9.]+);')
        let blur_value = if ($blur_matches | is-empty) { null } else { $blur_matches | get 0.value | into float }
        let radius_matches = ($ripple_shader | parse -r 'const float MAX_RADIUS = (?<value>[0-9.eE-]+);')
        let radius_value = if ($radius_matches | is-empty) { null } else { $radius_matches | get 0.value | into float }
        let ring_matches = ($ripple_shader | parse -r 'const float RING_THICKNESS = (?<value>[0-9.eE-]+);')
        let ring_value = if ($ring_matches | is-empty) { null } else { $ring_matches | get 0.value | into float }

        if (
            ($command_output.exit_code == 0)
            and ($blaze_shader | str contains 'const float YAZELIX_TRAIL_GLOW_STRENGTH = 0.0;')
            and ($blaze_shader | str contains 'const float YAZELIX_CURSOR_GLOW_STRENGTH = 0.0;')
            and ($blaze_shader | str contains 'const float YAZELIX_TRAIL_EDGE_WIDTH_SCALE = 0.0;')
            and ($blaze_shader | str contains 'const float YAZELIX_CURSOR_EDGE_WIDTH_SCALE = 0.0;')
            and ($blaze_shader | str contains 'const float YAZELIX_TRAIL_CORE_OFFSET_SCALE = 0.0;')
            and ($blaze_shader | str contains 'void renderSimpleDualColorTrail(')
            and ($blaze_shader | str contains 'renderSimpleDualColorTrail(fragColor, fragCoord, TRAIL_COLOR, TRAIL_COLOR_ACCENT, DURATION, .007, 1.5);')
            and not ($blaze_shader | str contains 'vec4 trail = mix(saturate(TRAIL_COLOR_ACCENT, 1.5), fragColor, trailGlowMask')
            and ($sweep_shader | str contains 'ghostty_trail_glow = none')
            and ($sweep_length_value != null)
            and ($sweep_length_value == 0.0)
            and ($ripple_shader | str contains 'ghostty_trail_glow = none')
            and ($blur_value != null)
            and ($blur_value < 0.5)
            and ($radius_value != null)
            and ($radius_value == 0.0)
            and ($ring_value != null)
            and ($ring_value == 0.0)
        ) {
            print "  ✅ Ghostty shader generation collapses sweep and ripple spread for glow=none instead of only lowering blur"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($command_output.exit_code) sweep=($sweep_length_value) blur=($blur_value) radius=($radius_value) ring=($ring_value) blaze_has_header=(($blaze_shader | str contains 'YAZELIX_TRAIL_GLOW_STRENGTH = 0.0;')) stderr=(($command_output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_generate_all_terminal_configs_normalizes_ghostty_medium_glow_across_variants [] {
    print "🧪 Testing Ghostty medium glow keeps mono and multicolor variants on the same spread contract..."

    let tmpdir = (^mktemp -d /tmp/yazelix_ghostty_medium_glow_variants_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    let terminal_configs_script = ($runtime_root | path join "nushell" "scripts" "utils" "terminal_configs.nu")
    mkdir $fake_home

    let result = (try {
        '[terminal]
terminals = ["ghostty"]
ghostty_trail_color = "blaze"
ghostty_trail_effect = "tail"
ghostty_mode_effect = "ripple"
ghostty_trail_glow = "medium"
' | save --force --raw $config_path

        let command_output = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            ^nu -c $"use \"($terminal_configs_script)\" [generate_all_terminal_configs]; generate_all_terminal_configs \"($runtime_root)\"" | complete
        })

        let blaze_shader = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "shaders" "cursor_trail_blaze.glsl"))
        let dusk_shader = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "shaders" "cursor_trail_dusk.glsl"))

        if (
            ($command_output.exit_code == 0)
            and ($blaze_shader | str contains 'trailGlowMask(sdfTrail, mod + 0.010, 0.035)')
            and ($dusk_shader | str contains 'trailGlowMask(sdfTrail, mod + 0.010, 0.035)')
        ) {
            print "  ✅ Mono and multicolor Ghostty variants now share the same medium outer-glow spread"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($command_output.exit_code) blaze_match=(($blaze_shader | str contains 'trailGlowMask(sdfTrail, mod + 0.010, 0.035)')) dusk_match=(($dusk_shader | str contains 'trailGlowMask(sdfTrail, mod + 0.010, 0.035)')) stderr=(($command_output.stderr | str trim))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def write_minimal_user_zellij_config [fake_home: string] {
    let zellij_config_dir = ($fake_home | path join ".config" "zellij")
    let zellij_config_path = ($zellij_config_dir | path join "config.kdl")
    mkdir $zellij_config_dir
    'keybinds { normal { bind "f1" { WriteChars "fixture"; } } }'
        | save --force --raw $zellij_config_path
}

def test_generated_zellij_layout_omits_empty_custom_text_badge [] {
    print "🧪 Testing generated Zellij layout omits empty zjstatus custom text..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_badge_empty_XXXXXX | str trim)

    let result = (try {
        let out_dir = ($tmpdir | path join "out")
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home

        let generated_layout = (with-env {
            HOME: $fake_home
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_config_dir)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "layouts" "yzx_side.kdl")
        })
        let stdout = ($generated_layout | str trim)

        if (
            ($stdout | str contains '#[fg=#00ccff,bold]YAZELIX {command_version}')
            and not ($stdout | str contains '#[fg=#ffff00,bold][')
        ) {
            print "  ✅ Empty zjstatus custom text stays invisible in the generated layout"
            true
        } else {
            print "  ❌ Unexpected result: empty custom text still rendered a badge"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_generated_zellij_layout_renders_capped_custom_text_before_branding [] {
    print "🧪 Testing generated Zellij layout renders capped zjstatus custom text before branding..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_badge_render_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let out_dir = ($tmpdir | path join "out")
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home
        '[zellij]
custom_text = "  roadmap-2026  "
' | save --force --raw $config_path

        let generated_layout = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_OVERRIDE: $config_path
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_config_dir)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "layouts" "yzx_side.kdl")
        })
        let stdout = ($generated_layout | str trim)
        let expected_segment = '#[fg=#ffff00,bold][roadmap-] #[fg=#00ccff,bold]YAZELIX {command_version}'

        if ($stdout | str contains $expected_segment) {
            print "  ✅ Rendered zjstatus custom text is capped to 8 characters and placed before YAZELIX"
            true
        } else {
            print $"  ❌ Unexpected result: expected_segment=($expected_segment)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_sidebar_state_plugin_generated [] {
    print "🧪 Testing generated Yazi init includes sidebar-state..."

    try {
        let root = (get_repo_config_dir)
        generate_merged_yazi_config $root --quiet
        let stdout = (open --raw ($env.HOME | path join ".local" "share" "yazelix" "configs" "yazi" "init.lua") | str trim)

        if ($stdout | str contains 'require("sidebar-state"):setup()') {
            print "  ✅ Generated Yazi init loads the sidebar-state core plugin"
            true
        } else {
            print "  ❌ Unexpected result: generated init is missing sidebar-state"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_zellij_default_mode_is_enforced_in_merged_config [] {
    print "🧪 Testing merged Zellij config enforces default_mode..."

    if (which zellij | is-empty) {
        print "  ℹ️  Skipping Zellij config merge test because zellij is not available"
        return true
    }

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_mode_test_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let out_dir = ($tmpdir | path join "out")
        '[zellij]
default_mode = "locked"
' | save --force --raw $config_path

        let output = (with-env {
            YAZELIX_CONFIG_OVERRIDE: $config_path
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_config_dir)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl")
        })
        let stdout = ($output | str trim)

        if ($stdout | str contains 'default_mode "locked"') {
            print "  ✅ Generated Zellij config enforces the configured default_mode"
            true
        } else {
            print "  ❌ Unexpected result: generated config is missing default_mode"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def test_zellij_horizontal_walking_is_plugin_owned [] {
    print "🧪 Testing Yazelix-owned Zellij session keybinds are emitted in merged config..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_walk_test_XXXXXX | str trim)

    let result = (try {
        let out_dir = ($tmpdir | path join "out")
        let fake_home = ($tmpdir | path join "home")
        let zellij_config_dir = ($fake_home | path join ".config" "zellij")
        let zellij_config_path = ($zellij_config_dir | path join "config.kdl")
        mkdir $zellij_config_dir
        'keybinds { normal { bind "f1" { WriteChars "fixture"; } } }'
            | save --force --raw $zellij_config_path

        let output = (with-env { HOME: $fake_home, YAZELIX_TEST_OUT_DIR: $out_dir } {
            let root = (get_repo_config_dir)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            {
                config: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl"))
                layout: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "layouts" "yzx_side.kdl"))
            }
        })
        let config_stdout = ($output.config | str trim)
        let layout_stdout = ($output.layout | str trim)

        if (
            ($config_stdout | str contains 'bind "Alt h" "Alt Left" {')
            and ($config_stdout | str contains 'name "move_focus_left_or_tab"')
            and ($config_stdout | str contains 'bind "Alt l" "Alt Right" {')
            and ($config_stdout | str contains 'name "move_focus_right_or_tab"')
            and ($config_stdout | str contains 'bind "Alt y" {')
            and ($config_stdout | str contains 'name "toggle_sidebar"')
            and ($config_stdout | str contains 'bind "Alt [" {')
            and ($config_stdout | str contains 'name "previous_family"')
            and ($config_stdout | str contains 'bind "Alt ]" {')
            and ($config_stdout | str contains 'name "next_family"')
            and ($config_stdout | str contains 'bind "Alt t" {')
            and ($config_stdout | str contains 'yzx_toggle_popup.nu')
            and ($config_stdout | str contains 'yazelix_popup_runner.wasm')
            and not ($layout_stdout | str contains 'bind "Alt h" "Alt Left" {')
            and not ($layout_stdout | str contains 'bind "Alt y" {')
            and not ($layout_stdout | str contains 'bind "Alt t" {')
        ) {
            print "  ✅ Yazelix-owned session keybinds are emitted in merged config instead of layout-local keybind blocks"
            true
        } else {
            print "  ❌ Unexpected result: merged config/layout ownership for Yazelix keybinds is wrong"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

export def run_generated_config_canonical_tests [] {
    [
        (test_layout_generator_rewrites_runtime_paths)
        (test_logo_welcome_variant_adapts_to_width)
        (test_logo_welcome_frame_respects_width_budget)
        (test_logo_animation_lands_on_static_resting_frame)
        (test_boids_animation_stays_bounded_and_width_aware)
        (test_boids_animation_lands_on_logo_resting_frame)
        (test_life_animation_is_deterministic_and_evolves)
        (test_life_animation_stays_bounded_and_width_aware)
        (test_life_animation_lands_on_logo_resting_frame)
        (test_zellij_widget_tray_defaults_omit_layout)
        (test_generate_all_terminal_configs_creates_override_scaffolds)
        (test_terminal_override_scaffolds_ignore_yazelix_dir_runtime_root)
        (test_parse_yazelix_config_reads_core_welcome_style)
        (test_parse_yazelix_config_rejects_legacy_ascii_mode_with_migration_guidance)
        (test_parse_yazelix_config_reads_pack_sidecar)
        (test_parse_yazelix_config_bootstraps_welcome_style_surface)
        (test_parse_yazelix_config_bootstraps_split_default_surfaces)
        (test_parse_yazelix_config_rejects_legacy_main_file_packs_with_migration_guidance)
        (test_parse_yazelix_config_rejects_split_pack_ownership)
        (test_welcome_style_random_pool_excludes_static)
        (test_user_mode_requires_real_terminal_config)
        (test_config_schema_rejects_removed_auto_terminal_config_mode)
        (test_ghostty_trail_glow_defaults_to_medium_and_reads_explicit_levels)
        (test_generate_all_terminal_configs_honors_ghostty_trail_color_none)
        (test_config_schema_rejects_removed_layout_widget)
        (test_config_schema_rejects_invalid_ghostty_trail_glow)
        (test_generate_all_terminal_configs_honors_ghostty_trail_glow)
        (test_generate_all_terminal_configs_normalizes_ghostty_medium_glow_across_variants)
        (test_zellij_default_mode_is_enforced_in_merged_config)
    ]
}

export def run_generated_config_noncanonical_tests [] {
    [
        (test_generated_zellij_layout_renders_capped_custom_text_before_branding)
        (test_zellij_horizontal_walking_is_plugin_owned)
    ]
}

export def run_generated_config_tests [] {
    [
        (run_generated_config_canonical_tests)
        (run_generated_config_noncanonical_tests)
    ] | flatten
}
