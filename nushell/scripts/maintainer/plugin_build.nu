#!/usr/bin/env nu

use ../utils/runtime_paths.nu get_yazelix_state_dir
use ../utils/yzx_core_bridge.nu [
    build_default_yzx_core_error_surface
    build_record_yzx_core_error_surface
    resolve_active_config_surface_via_yzx_core
    run_yzx_core_json_command
]

const ZELLIJ_MATERIALIZATION_COMMAND = "zellij-materialization.generate"

def require_yazelix_repo_root [] {
    let repo_root = ($env.YAZELIX_REPO_ROOT? | default "" | path expand)
    if ($repo_root | is-empty) or (not ($repo_root | path exists)) {
        error make {msg: "This maintainer command requires YAZELIX_REPO_ROOT to point at a writable Yazelix repo checkout."}
    }
    $repo_root
}

def get_pane_orchestrator_paths [] {
    let yazelix_dir = require_yazelix_repo_root
    let crate_dir = ($yazelix_dir | path join "rust_plugins" "zellij_pane_orchestrator")
    let build_target = "wasm32-wasip1"
    let wasm_path = ($crate_dir | path join "target" $build_target "release" "yazelix_pane_orchestrator.wasm")

    {
        yazelix_dir: $yazelix_dir
        crate_dir: $crate_dir
        build_target: $build_target
        wasm_path: $wasm_path
    }
}

const pane_orchestrator_wasm_name = "yazelix_pane_orchestrator.wasm"

def get_tracked_pane_orchestrator_wasm_path [yazelix_dir: string] {
    $yazelix_dir | path join "configs" "zellij" "plugins" $pane_orchestrator_wasm_name
}

def get_runtime_pane_orchestrator_wasm_path [] {
    get_yazelix_state_dir | path join "configs" "zellij" "plugins" $pane_orchestrator_wasm_name
}

def print_rust_wasi_enable_hint [] {
    print "   Install a WASI-capable Rust toolchain in your maintainer environment."
    print "   Example: run the build inside the repo's maintainer shell, or use `rustup target add wasm32-wasip1`."
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

def generate_merged_zellij_config [yazelix_dir: string] {
    let config_surface = (resolve_active_config_surface_via_yzx_core $yazelix_dir)
    let merged_config_dir = (get_yazelix_state_dir | path join "configs" "zellij")
    let helper_args = [
        $ZELLIJ_MATERIALIZATION_COMMAND
        "--config"
        $config_surface.config_file
        "--default-config"
        $config_surface.default_config_path
        "--contract"
        ($yazelix_dir | path join "config_metadata" "main_config_contract.toml")
        "--runtime-dir"
        $yazelix_dir
        "--zellij-config-dir"
        $merged_config_dir
    ]

    let result = (run_yzx_core_json_command
        $yazelix_dir
        (build_record_yzx_core_error_surface {config_file: $config_surface.config_file})
        $helper_args
        "Yazelix Rust zellij-materialization helper returned invalid JSON.")

    $result.merged_config_path
}

def sync_built_wasm [paths: record, label: string] {
    print $"🔄 Syncing ($label) wasm into Yazelix..."
    let repo_target_path = (get_tracked_pane_orchestrator_wasm_path $paths.yazelix_dir)
    cp --force $paths.wasm_path $repo_target_path
    let merged_config_path = (generate_merged_zellij_config $paths.yazelix_dir)
    let runtime_target_path = (get_runtime_pane_orchestrator_wasm_path)
    let runtime_target_dir = ($runtime_target_path | path dirname)
    if not ($runtime_target_dir | path exists) {
        mkdir $runtime_target_dir
    }
    cp --force $paths.wasm_path $runtime_target_path
    let byte_len = (open --raw $paths.wasm_path | length)

    print $"Updated pane orchestrator repo wasm: ($repo_target_path)"
    print $"Updated pane orchestrator runtime wasm: ($runtime_target_path)"
    print $"Updated merged Zellij config: ($merged_config_path)"
    print $"Size: ($byte_len) bytes"
    print ""
    print "Safest next step:"
    print "Restart Yazelix or open a fresh Yazelix window so Zellij loads the updated plugin cleanly."
    print "In-place plugin reloads can leave the current session in a broken permission state."
    print ""
    print "If you are already stuck in a blank/permission-limbo session, recover with:"
    print "zellij delete-all-sessions -f -y"
}

export def build_pane_orchestrator_wasm [sync: bool = false] {
    let paths = get_pane_orchestrator_paths
    run_wasm_build $paths "pane orchestrator"

    if $sync {
        sync_built_wasm $paths "pane orchestrator"
    }
}
