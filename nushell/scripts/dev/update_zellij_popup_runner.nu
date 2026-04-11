#!/usr/bin/env nu
# Sync the locally built popup runner wasm into the tracked repo path
# and the stable runtime plugin path used by active Yazelix sessions.

use ../setup/zellij_plugin_paths.nu [
  get_tracked_popup_runner_wasm_path
  sync_popup_runner_runtime_wasm
]
use ../setup/zellij_config_merger.nu generate_merged_zellij_config

const built_plugin_relative_path = "rust_plugins/zellij_popup_runner/target/wasm32-wasip1/release/yazelix_popup_runner.wasm"

export def main [] {
  let yazelix_dir = ($env.HOME | path join ".config" "yazelix")
  let source_path = ($yazelix_dir | path join $built_plugin_relative_path)
  let repo_target_path = (get_tracked_popup_runner_wasm_path $yazelix_dir)

  if not ($source_path | path exists) {
    print $"Error: built popup runner wasm not found at: ($source_path)"
    print "Build it first with `yzx dev build_popup_plugin`."
    print "If cargo/rustc or the wasm stdlib are missing, run the build inside the Yazelix maintainer shell or install a wasm32-wasip1 Rust toolchain."
    exit 2
  }

  let byte_len = (open --raw $source_path | length)
  if $byte_len < 1024 {
    print $"Error: built popup runner wasm is too small to be valid \(size=($byte_len) bytes\)"
    exit 3
  }

  cp --force $source_path $repo_target_path
  let runtime_target_path = (sync_popup_runner_runtime_wasm $yazelix_dir)
  let merged_config_path = (generate_merged_zellij_config $yazelix_dir)

  print $"Updated popup runner repo wasm: ($repo_target_path)"
  print $"Updated popup runner runtime wasm: ($runtime_target_path)"
  print $"Updated merged Zellij config: ($merged_config_path)"
  print $"Size: ($byte_len) bytes"
}
