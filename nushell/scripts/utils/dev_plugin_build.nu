#!/usr/bin/env nu

use repo_checkout.nu require_yazelix_repo_root

def get_pane_orchestrator_paths [] {
    let yazelix_dir = require_yazelix_repo_root
    let crate_dir = ($yazelix_dir | path join "rust_plugins" "zellij_pane_orchestrator")
    let build_target = "wasm32-wasip1"
    let wasm_path = ($crate_dir | path join "target" $build_target "release" "yazelix_pane_orchestrator.wasm")
    let sync_script = ($yazelix_dir | path join "nushell" "scripts" "dev" "update_zellij_pane_orchestrator.nu")

    {
        yazelix_dir: $yazelix_dir
        crate_dir: $crate_dir
        build_target: $build_target
        wasm_path: $wasm_path
        sync_script: $sync_script
    }
}

def get_popup_runner_paths [] {
    let yazelix_dir = require_yazelix_repo_root
    let crate_dir = ($yazelix_dir | path join "rust_plugins" "zellij_popup_runner")
    let build_target = "wasm32-wasip1"
    let wasm_path = ($crate_dir | path join "target" $build_target "release" "yazelix_popup_runner.wasm")
    let sync_script = ($yazelix_dir | path join "nushell" "scripts" "dev" "update_zellij_popup_runner.nu")

    {
        yazelix_dir: $yazelix_dir
        crate_dir: $crate_dir
        build_target: $build_target
        wasm_path: $wasm_path
        sync_script: $sync_script
    }
}

def print_rust_wasi_enable_hint [] {
    print "   Enable the `rust_wasi` pack in ~/.config/yazelix/user_configs/yazelix_packs.toml to get the pinned WASI-capable Rust toolchain."
    print '   Example: enabled = ["rust_wasi"]'
}

def ensure_build_tools_available [] {
    let missing_tools = (
        ["cargo" "rustc"]
        | where { |tool| (which $tool | is-empty) }
    )
    if ($missing_tools | is-not-empty) {
        print $"❌ Missing Rust tool(s): ($missing_tools | str join ', ')"
        print_rust_wasi_enable_hint
        exit 1
    }
}

def run_wasm_build [paths: record, label: string] {
    if not ($paths.crate_dir | path exists) {
        print $"❌ ($label) crate not found: ($paths.crate_dir)"
        exit 1
    }

    ensure_build_tools_available

    print $"🦀 Building ($label) for target ($paths.build_target)..."
    let result = (do {
        cd $paths.crate_dir
        ^cargo build --target $paths.build_target --profile release | complete
    })

    if ($result.stdout | default "" | str trim | is-not-empty) {
        print ($result.stdout | str trim)
    }

    if $result.exit_code != 0 {
        let stderr_text = ($result.stderr | default "" | str trim)
        if ($stderr_text | is-not-empty) {
            print $stderr_text
        }
        if (
            ($stderr_text | str contains "can't find crate for `core`")
            or ($stderr_text | str contains "can't find crate for `std`")
            or ($stderr_text | str contains "target may not be installed")
        ) {
            print ""
            print "❌ The wasm target stdlib is not available in the current Rust toolchain."
            print_rust_wasi_enable_hint
        } else {
            print ""
            print $"❌ ($label) build failed."
        }
        exit $result.exit_code
    }

    if not ($paths.wasm_path | path exists) {
        print $"❌ Build reported success, but wasm output was not found at: ($paths.wasm_path)"
        exit 1
    }

    print $"✅ Built ($label) wasm: ($paths.wasm_path)"
}

def sync_built_wasm [paths: record, label: string] {
    if not ($paths.sync_script | path exists) {
        print $"❌ Sync helper not found: ($paths.sync_script)"
        exit 1
    }
    print $"🔄 Syncing ($label) wasm into Yazelix..."
    ^nu $paths.sync_script
}

export def build_pane_orchestrator_wasm [sync: bool = false] {
    let paths = get_pane_orchestrator_paths
    run_wasm_build $paths "pane orchestrator"

    if $sync {
        sync_built_wasm $paths "pane orchestrator"
    }
}

export def build_popup_plugin_wasm [sync: bool = false] {
    let paths = get_popup_runner_paths
    run_wasm_build $paths "popup runner"

    if $sync {
        sync_built_wasm $paths "popup runner"
    }
}
