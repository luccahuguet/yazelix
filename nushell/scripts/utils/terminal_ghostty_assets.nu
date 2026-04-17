#!/usr/bin/env nu

use safe_remove.nu remove_path_within_root

export def sync_generated_ghostty_shader_assets [
    runtime_dir: string
    ghostty_dir: string
    glow_level: string
    effect_color_literal: string = "iCurrentCursorColor"
] {
    let resolved_runtime_dir = ($runtime_dir | path expand)
    let shaders_src = ($resolved_runtime_dir | path join "configs" "terminal_emulators" "ghostty" "shaders")
    let shaders_dest = ($ghostty_dir | path join "shaders")

    if ($shaders_dest | path exists) {
        let chmod_result = (^chmod -R u+w $shaders_dest | complete)
        if ($chmod_result.exit_code != 0) and (($chmod_result.stderr | str trim) | is-not-empty) {
            print $"⚠ Failed to relax Ghostty shader permissions before cleanup: ($chmod_result.stderr | str trim)"
        }
        try {
            remove_path_within_root $shaders_dest $ghostty_dir "generated Ghostty shaders" --recursive
        } catch {|err|
            error make {msg: $"Failed to remove previous Ghostty shader assets at ($shaders_dest): ($err.msg)"}
        }
    }

    mkdir $shaders_dest
    if ($shaders_src | path exists) {
        let copy_result = (^cp -R $"($shaders_src)/." $shaders_dest | complete)
        if $copy_result.exit_code != 0 {
            error make {msg: $"Failed to copy Ghostty shader assets from ($shaders_src) to ($shaders_dest): ($copy_result.stderr | str trim)"}
        }
        let chmod_result = (^chmod -R u+w $shaders_dest | complete)
        if $chmod_result.exit_code != 0 {
            error make {msg: $"Failed to make generated Ghostty shader assets writable at ($shaders_dest): ($chmod_result.stderr | str trim)"}
        }
    }

    let build_script = ($shaders_src | path join "build_shaders.nu")
    if ($build_script | path exists) {
        nu -c $"use '($build_script)' [build_cursor_trail_shaders build_ghostty_cursor_effect_shaders]; build_cursor_trail_shaders '($shaders_dest)' '($glow_level)'; build_ghostty_cursor_effect_shaders '($shaders_dest)' '($glow_level)' '($shaders_dest)' '($effect_color_literal)'"
    }
}
