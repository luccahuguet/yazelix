#!/usr/bin/env nu
# Sync the locally built pane orchestrator wasm into the tracked repo path
# and the runtime plugin path used by active Yazelix sessions.

use ../setup/zellij_plugin_paths.nu get_pane_orchestrator_wasm_path

const plugin_name = "yazelix_pane_orchestrator.wasm"
const built_plugin_relative_path = "rust_plugins/zellij_pane_orchestrator/target/wasm32-wasip1/release/yazelix_pane_orchestrator.wasm"

def atomic_copy [source_path: string, target_path: string] {
  let target_dir = ($target_path | path dirname)
  if not ($target_dir | path exists) {
    mkdir $target_dir
  }

  let tmp_path = $"($target_path).tmp"
  cp --force $source_path $tmp_path
  mv --force $tmp_path $target_path
}

export def main [] {
  let yazelix_dir = ($env.HOME | path join ".config" "yazelix")
  let source_path = ($yazelix_dir | path join $built_plugin_relative_path)
  let repo_target_path = (get_pane_orchestrator_wasm_path $yazelix_dir)
  let runtime_target_path = ($env.HOME | path join ".local" "share" "yazelix" "configs" "zellij" "plugins" $plugin_name)

  if not ($source_path | path exists) {
    print $"Error: built pane orchestrator wasm not found at: ($source_path)"
    print "Build it first with cargo build --target wasm32-wasip1 --profile release"
    exit 2
  }

  let byte_len = (open --raw $source_path | length)
  if $byte_len < 1024 {
    print $"Error: built pane orchestrator wasm is too small to be valid \(size=($byte_len) bytes\)"
    exit 3
  }

  atomic_copy $source_path $repo_target_path
  atomic_copy $source_path $runtime_target_path

  print $"Updated pane orchestrator repo wasm: ($repo_target_path)"
  print $"Updated pane orchestrator runtime wasm: ($runtime_target_path)"
  print $"Size: ($byte_len) bytes"
  print ""
  print "Reload the plugin in the current Zellij session with:"
  print $"zellij action start-or-reload-plugin file:($repo_target_path)"
}
