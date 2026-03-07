#!/usr/bin/env nu

const pane_orchestrator_wasm_name = "yazelix_pane_orchestrator.wasm"

export def get_pane_orchestrator_wasm_path [yazelix_dir: string = "~/.config/yazelix"] {
    let root = ($yazelix_dir | path expand)
    $root | path join "configs" "zellij" "plugins" $pane_orchestrator_wasm_name
}
