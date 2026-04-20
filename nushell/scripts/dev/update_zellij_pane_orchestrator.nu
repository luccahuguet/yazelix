#!/usr/bin/env nu
# Sync the locally built pane orchestrator wasm into the tracked repo path
# and regenerate the Rust-owned Zellij materialization outputs.

use ../setup/zellij_config_merger.nu generate_merged_zellij_config
use ../maintainer/repo_checkout.nu require_yazelix_repo_root
use ../utils/common.nu get_yazelix_state_dir

const built_plugin_relative_path = "rust_plugins/zellij_pane_orchestrator/target/wasm32-wasip1/release/yazelix_pane_orchestrator.wasm"
const pane_orchestrator_wasm_name = "yazelix_pane_orchestrator.wasm"

def get_tracked_pane_orchestrator_wasm_path [yazelix_dir: string] {
  $yazelix_dir | path join "configs" "zellij" "plugins" $pane_orchestrator_wasm_name
}

def get_runtime_pane_orchestrator_wasm_path [] {
  get_yazelix_state_dir | path join "configs" "zellij" "plugins" $pane_orchestrator_wasm_name
}

export def main [] {
  let yazelix_dir = (require_yazelix_repo_root)
  let source_path = ($yazelix_dir | path join $built_plugin_relative_path)
  let repo_target_path = (get_tracked_pane_orchestrator_wasm_path $yazelix_dir)

  if not ($source_path | path exists) {
    print $"Error: built pane orchestrator wasm not found at: ($source_path)"
    print "Build it first with `yzx dev build_pane_orchestrator`."
    print "If cargo/rustc or the wasm stdlib are missing, run the build inside the Yazelix maintainer shell or install a wasm32-wasip1 Rust toolchain."
    exit 2
  }

  let byte_len = (open --raw $source_path | length)
  if $byte_len < 1024 {
    print $"Error: built pane orchestrator wasm is too small to be valid \(size=($byte_len) bytes\)"
    exit 3
  }

  cp --force $source_path $repo_target_path
  let merged_config_path = (generate_merged_zellij_config $yazelix_dir)
  let runtime_target_path = (get_runtime_pane_orchestrator_wasm_path)

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
