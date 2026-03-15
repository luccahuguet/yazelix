#!/usr/bin/env nu
# Sync the locally built pane orchestrator wasm into the tracked repo path
# and the content-hashed runtime plugin path used by active Yazelix sessions.

use ../setup/zellij_plugin_paths.nu [
  get_tracked_pane_orchestrator_wasm_path
  sync_pane_orchestrator_runtime_wasm
]
use ../setup/zellij_config_merger.nu generate_merged_zellij_config

const built_plugin_relative_path = "rust_plugins/zellij_pane_orchestrator/target/wasm32-wasip1/release/yazelix_pane_orchestrator.wasm"

export def main [] {
  let yazelix_dir = ($env.HOME | path join ".config" "yazelix")
  let source_path = ($yazelix_dir | path join $built_plugin_relative_path)
  let repo_target_path = (get_tracked_pane_orchestrator_wasm_path $yazelix_dir)

  if not ($source_path | path exists) {
    print $"Error: built pane orchestrator wasm not found at: ($source_path)"
    print "Build it first with `yzx dev build_pane_orchestrator`."
    print "If cargo/rustc or the wasm stdlib are missing, enable the `rust_wasi` pack in yazelix.toml."
    exit 2
  }

  let byte_len = (open --raw $source_path | length)
  if $byte_len < 1024 {
    print $"Error: built pane orchestrator wasm is too small to be valid \(size=($byte_len) bytes\)"
    exit 3
  }

  cp --force $source_path $repo_target_path
  let runtime_target_path = (sync_pane_orchestrator_runtime_wasm $yazelix_dir)
  let merged_config_path = (generate_merged_zellij_config $yazelix_dir)

  print $"Updated pane orchestrator repo wasm: ($repo_target_path)"
  print $"Updated pane orchestrator runtime wasm: ($runtime_target_path)"
  print $"Updated merged Zellij config: ($merged_config_path)"
  print $"Size: ($byte_len) bytes"
  print ""
  print "Reload the plugin in the current Zellij session with:"
  print $"zellij action start-or-reload-plugin file:($runtime_target_path)"
}
