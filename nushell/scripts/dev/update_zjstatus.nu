#!/usr/bin/env nu
# Update zjstatus plugin (Zellij status bar) for Yazelix
# Syncs the Nix-provided zjstatus wasm to
# ~/.config/yazelix/configs/zellij/zjstatus.wasm (no backup).

export def main [] {
  let home = $env.HOME
  if ($home | is-empty) or (not ($home | path exists)) {
    print "Error: Cannot resolve HOME directory"
    exit 1
  }

  let target_dir = $"($home)/.config/yazelix/configs/zellij"
  let target_path = $"($target_dir)/zjstatus.wasm"

  # Resolve wasm from Nix (exported by devenv.nix)
  let store_path = ($env.YAZELIX_ZJSTATUS_WASM? | default "")
  if ($store_path | is-empty) {
    print "Error: YAZELIX_ZJSTATUS_WASM is not set"
    exit 2
  }
  if not ($store_path | path exists) {
    print $"Error: zjstatus wasm not found at: ($store_path)"
    exit 3
  }

  # Minimal validation: check file size and extension hint
  let byte_len = (open --raw $store_path | length)
  if $byte_len < 1024 {
    print $"Error: Nix-provided wasm is too small to be valid \(size=($byte_len) bytes\)"
    exit 5
  }

  # Prepare target directory
  if not ($target_dir | path exists) { mkdir $target_dir }

  # No backup: overwrite atomically via temp file

  # Atomic write: temp then move
  let tmp_path = $"($target_path).tmp"
  try { cp --force $store_path $tmp_path } catch {|err|
    print $"Error writing temporary file: ($err.msg)"
    exit 6
  }
  mv --force $tmp_path $target_path
  print $"Updated zjstatus at: ($target_path) \(size=($byte_len) bytes\)"
}
