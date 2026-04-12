#!/usr/bin/env nu

use ../utils/constants.nu [ZELLIJ_CONFIG_PATHS]

export def extract_semantic_config_blocks [config_content: string] {
    mut stripped_lines = []
    mut load_plugin_lines = []
    mut plugin_lines = []
    mut keybind_lines = []
    mut ui_lines = []
    mut active_block = ""
    mut brace_depth = 0

    for line in ($config_content | lines) {
        let trimmed = ($line | str trim)
        let open_braces = (($line | split chars | where {|char| $char == "{"}) | length)
        let close_braces = (($line | split chars | where {|char| $char == "}"}) | length)

        if ($active_block | is-empty) {
            let matched_block = (
                ["load_plugins", "plugins", "keybinds", "ui"]
                | where {|block_name| $trimmed | str starts-with $block_name }
                | get 0?
                | default ""
            )

            if ($matched_block | is-not-empty) {
                $active_block = $matched_block
                $brace_depth = ($open_braces - $close_braces)

                if $brace_depth <= 0 {
                    let inline_body = (
                        $trimmed
                        | str replace -r $"^($matched_block)\\s*\\{" ""
                        | str replace -r "\\}\\s*$" ""
                        | str trim
                    )
                    if ($inline_body | is-not-empty) {
                        match $matched_block {
                            "load_plugins" => {
                                $load_plugin_lines = ($load_plugin_lines | append $inline_body)
                            }
                            "plugins" => {
                                $plugin_lines = ($plugin_lines | append $inline_body)
                            }
                            "keybinds" => {
                                $keybind_lines = ($keybind_lines | append $inline_body)
                            }
                            "ui" => {
                                $ui_lines = ($ui_lines | append $inline_body)
                            }
                        }
                    }
                    $active_block = ""
                    $brace_depth = 0
                }
            } else {
                $stripped_lines = ($stripped_lines | append $line)
            }
        } else {
            $brace_depth = ($brace_depth + $open_braces - $close_braces)
            if $brace_depth > 0 {
                match $active_block {
                    "load_plugins" => {
                        $load_plugin_lines = ($load_plugin_lines | append $line)
                    }
                    "plugins" => {
                        $plugin_lines = ($plugin_lines | append $line)
                    }
                    "keybinds" => {
                        $keybind_lines = ($keybind_lines | append $line)
                    }
                    "ui" => {
                        $ui_lines = ($ui_lines | append $line)
                    }
                }
            } else {
                $active_block = ""
            }
        }
    }

    {
        config_without_semantic_blocks: ($stripped_lines | str join "\n")
        load_plugin_lines: $load_plugin_lines
        plugin_lines: $plugin_lines
        keybind_lines: $keybind_lines
        ui_lines: $ui_lines
    }
}

export def build_yazelix_load_plugins_block [
    existing_load_plugin_lines: list<string>
    pane_orchestrator_alias: string
] {
    mut merged_plugin_lines = (
        $existing_load_plugin_lines
        | flatten
        | where {|line|
            not (
                ($line | str contains "yazelix_popup_runner.wasm")
                or ($line | str contains "yazelix_popup_runner")
            )
        }
    )
    let pane_orchestrator_entry = $"  ($pane_orchestrator_alias)"
    let pane_orchestrator_present = ($merged_plugin_lines | any {|line| ($line | str trim) == $pane_orchestrator_alias })
    if not $pane_orchestrator_present {
        $merged_plugin_lines = ($merged_plugin_lines | append $pane_orchestrator_entry)
    }

    (
        [
            "load_plugins {"
            ...($merged_plugin_lines | flatten)
            "}"
        ]
        | str join "\n"
    )
}

export def build_yazelix_plugins_block [
    existing_plugin_lines: list<string>
    pane_orchestrator_alias: string
    pane_orchestrator_wasm_path: string
    widget_tray_segment: string
    custom_text_segment: string
    sidebar_width_percent: int
    runtime_dir: string
    popup_width_percent: int
    popup_height_percent: int
] {
    let escaped_widget_tray = ($widget_tray_segment | to json -r)
    let escaped_custom_text = ($custom_text_segment | to json -r)
    let escaped_sidebar_width_percent = ($sidebar_width_percent | into string | to json -r)
    let escaped_runtime_dir = ($runtime_dir | path expand | to json -r)
    let escaped_popup_width_percent = ($popup_width_percent | into string | to json -r)
    let escaped_popup_height_percent = ($popup_height_percent | into string | to json -r)
    let pane_alias_present = ($existing_plugin_lines | any {|line| $line | str contains $"($pane_orchestrator_alias) location=" })
    mut merged_plugin_lines = $existing_plugin_lines

    if not $pane_alias_present {
        $merged_plugin_lines = ($merged_plugin_lines | append [
            $"    ($pane_orchestrator_alias) location=\"file:($pane_orchestrator_wasm_path)\" {"
            $"        widget_tray_segment ($escaped_widget_tray)"
            $"        custom_text_segment ($escaped_custom_text)"
            $"        sidebar_width_percent ($escaped_sidebar_width_percent)"
            $"        runtime_dir ($escaped_runtime_dir)"
            $"        popup_width_percent ($escaped_popup_width_percent)"
            $"        popup_height_percent ($escaped_popup_height_percent)"
            "    }"
        ])
    }

    if ($merged_plugin_lines | is-empty) {
        ""
    } else {
        (
            [
                "plugins {"
                ...($merged_plugin_lines | flatten)
                "}"
            ]
            | str join "\n"
        )
    }
}

export def build_merged_keybinds_block [
    existing_keybind_lines: list<string>
    yazelix_keybind_lines: list<string>
] {
    let merged_keybind_lines = ($existing_keybind_lines | append $yazelix_keybind_lines | flatten)
    if ($merged_keybind_lines | is-empty) {
        ""
    } else {
        (
            [
                "keybinds {"
                ...$merged_keybind_lines
                "}"
            ]
            | str join "\n"
        )
    }
}

export def read_yazelix_override_keybinds [
    yazelix_dir: string
    pane_orchestrator_plugin_url: string
]: nothing -> list<string> {
    let overrides_path = ($yazelix_dir | path join $ZELLIJ_CONFIG_PATHS.yazelix_overrides)

    if not ($overrides_path | path exists) {
        error make {msg: $"Missing Yazelix Zellij overrides file: ($overrides_path)"}
    }

    let runtime_ref = ($yazelix_dir | path expand)
    let resolved_overrides = (
        (open $overrides_path)
        | str replace -a "__YAZELIX_PANE_ORCHESTRATOR_PLUGIN_URL__" $pane_orchestrator_plugin_url
        | str replace -a "__YAZELIX_RUNTIME_DIR__" $runtime_ref
    )
    let extracted_blocks = (extract_semantic_config_blocks $resolved_overrides)
    $extracted_blocks.keybind_lines
}
