#!/usr/bin/env nu
# yzx edit - Open managed Yazelix config surfaces in the configured editor

use ../utils/common.nu [get_yazelix_user_config_dir]
use ../utils/editor_launch_context.nu [resolve_editor_launch_context]
use ../utils/config_surfaces.nu reconcile_primary_config_surfaces
use ../setup/helix_config_merger.nu get_managed_helix_user_config_path

def open_config_surface_in_editor [config_path: string, --print] {
    if $print {
        $config_path
    } else {
        let editor_context = (resolve_editor_launch_context)
        mkdir ($config_path | path dirname)
        clear
        if ($editor_context.launch_env | columns | is-empty) {
            exec $editor_context.editor $config_path
        } else {
            with-env $editor_context.launch_env {
                exec $editor_context.editor $config_path
            }
        }
    }
}

def get_edit_targets [] {
    let paths = reconcile_primary_config_surfaces
    let user_root = (get_yazelix_user_config_dir)
    let helix_path = (get_managed_helix_user_config_path)
    let zellij_path = ($user_root | path join "zellij" "config.kdl")
    let yazi_toml_path = ($user_root | path join "yazi" "yazi.toml")
    let yazi_keymap_path = ($user_root | path join "yazi" "keymap.toml")
    let yazi_init_path = ($user_root | path join "yazi" "init.lua")

    [
        {
            id: "config"
            label: $"config  (ansi dark_gray)- main Yazelix config → ($paths.user_config)(ansi reset)"
            aliases: ["config", "main", "yazelix.toml"]
            search: "config main yazelix yazelix.toml"
            path: $paths.user_config
        }
        {
            id: "helix"
            label: $"helix  (ansi dark_gray)- managed Helix user config → ($helix_path)(ansi reset)"
            aliases: ["helix", "hx", "editor"]
            search: "helix hx editor config config.toml"
            path: $helix_path
        }
        {
            id: "zellij"
            label: $"zellij  (ansi dark_gray)- managed Zellij user config → ($zellij_path)(ansi reset)"
            aliases: ["zellij", "terminal", "config.kdl"]
            search: "zellij terminal config.kdl multiplexer"
            path: $zellij_path
        }
        {
            id: "yazi"
            label: $"yazi  (ansi dark_gray)- managed Yazi main config \(yazi.toml\) → ($yazi_toml_path)(ansi reset)"
            aliases: ["yazi", "yazi.toml", "file-manager"]
            search: "yazi yazi.toml file-manager file manager"
            path: $yazi_toml_path
        }
        {
            id: "yazi-keymap"
            label: $"yazi-keymap  (ansi dark_gray)- managed Yazi keymap \(keymap.toml\) → ($yazi_keymap_path)(ansi reset)"
            aliases: ["yazi-keymap", "keymap", "keymap.toml", "yazi keymap"]
            search: "yazi keymap keymap.toml file-manager bindings"
            path: $yazi_keymap_path
        }
        {
            id: "yazi-init"
            label: $"yazi-init  (ansi dark_gray)- managed Yazi init.lua → ($yazi_init_path)(ansi reset)"
            aliases: ["yazi-init", "init", "init.lua", "yazi init", "lua"]
            search: "yazi init init.lua lua file-manager plugins"
            path: $yazi_init_path
        }
    ]
}

def resolve_edit_target_by_id [target_id: string] {
    get_edit_targets | where id == $target_id | first
}

def filter_edit_targets [targets: list<record>, query_text: string] {
    let normalized = ($query_text | str downcase | str trim)
    if ($normalized | is-empty) {
        return $targets
    }

    let exact = (
        $targets | where {|target|
            (
                (($target.id | str downcase) == $normalized)
                or (($target.aliases? | default []) | any {|alias| ($alias | str downcase) == $normalized })
            )
        }
    )
    if not ($exact | is-empty) {
        return $exact
    }

    let tokens = ($normalized | split row " " | where {|token| not ($token | is-empty) })
    $targets | where {|target|
        let haystack = (
            [
                $target.id
                ...($target.aliases? | default [])
                ($target.search? | default "")
            ]
            | str join " "
            | str downcase
        )
        $tokens | all {|token| $haystack | str contains $token }
    }
}

def render_edit_target_choices [targets: list<record>] {
    $targets | get label
}

def render_edit_target_error_choices [targets: list<record>] {
    $targets
    | each {|target| $"  - ($target.id): ($target.path)" }
    | str join "\n"
}

def choose_edit_target [targets: list<record>, prompt: string] {
    let selected = (render_edit_target_choices $targets | input list --fuzzy $prompt)
    if ($selected | is-empty) {
        return null
    }

    $targets | where label == $selected | first
}

# Open a Yazelix-managed config surface in the configured editor
export def "yzx edit" [
    ...query: string  # Optional managed config surface name or alias
    --print  # Print the resolved config path without opening
] {
    let targets = (get_edit_targets)
    let query_text = ($query | str join " " | str trim)

    if ($query_text | is-empty) {
        if $print {
            error make {msg: $"yzx edit --print requires a target query. Supported managed surfaces:\n(render_edit_target_error_choices $targets)"}
        }

        let selected = (choose_edit_target $targets "yzx edit \(Esc to cancel\)> ")
        if $selected == null {
            return
        }

        return (open_config_surface_in_editor $selected.path)
    }

    let matches = (filter_edit_targets $targets $query_text)

    if ($matches | is-empty) {
        error make {msg: $"No managed Yazelix config surface matched `($query_text)`. Supported surfaces:\n(render_edit_target_error_choices $targets)"}
    }

    if (($matches | length) == 1) {
        let match = ($matches | first)
        return (open_config_surface_in_editor $match.path --print=$print)
    }

    if $print {
        error make {msg: $"Query `($query_text)` matched multiple managed config surfaces. Refine it to one of:\n(render_edit_target_error_choices $matches)"}
    }

    let selected = (choose_edit_target $matches $"yzx edit \((query_text)\)> ")
    if $selected == null {
        return
    }

    open_config_surface_in_editor $selected.path
}

# Open the main Yazelix config in the configured editor
export def "yzx edit config" [
    --print  # Print the config path without opening
] {
    let target = (resolve_edit_target_by_id "config")
    open_config_surface_in_editor $target.path --print=$print
}
