#!/usr/bin/env nu
# Update zjstatus plugin (Zellij status bar) for Yazelix
# Refreshes the vendored zjstatus wasm from the pinned `zjstatus` input in devenv.lock.

export def main [] {
  let home = $env.HOME
  if ($home | is-empty) or (not ($home | path exists)) {
    print "Error: Cannot resolve HOME directory"
    exit 1
  }

  let target_dir = $"($home)/.config/yazelix/configs/zellij/plugins"
  let target_path = $"($target_dir)/zjstatus.wasm"
  let lock_path = $"($home)/.config/yazelix/devenv.lock"

  if not ($lock_path | path exists) {
    print $"Error: devenv.lock not found at: ($lock_path)"
    exit 2
  }

  if (which nix | is-empty) {
    print "Error: nix is not available in PATH"
    exit 3
  }

  let lock = (open --raw $lock_path | from json)
  let locked_zjstatus = ($lock | get nodes.zjstatus.locked)
  let owner = ($locked_zjstatus | get owner)
  let repo = ($locked_zjstatus | get repo)
  let rev = ($locked_zjstatus | get rev)
  let system = (^nix eval --impure --raw --expr "builtins.currentSystem" | str trim)
  if ($system | is-empty) {
    print "Error: Failed to resolve current Nix system"
    exit 4
  }

  let flake_ref = $"github:($owner)/($repo)/($rev)#packages.($system).default"
  let store_root = (^nix build --no-link --print-out-paths $flake_ref | str trim)
  let store_path = ($store_root | path join "bin" "zjstatus.wasm")
  if not ($store_path | path exists) {
    print $"Error: zjstatus wasm not found at: ($store_path)"
    exit 5
  }

  # Minimal validation: check file size and extension hint
  let byte_len = (open --raw $store_path | length)
  if $byte_len < 1024 {
    print $"Error: Nix-provided wasm is too small to be valid \(size=($byte_len) bytes\)"
    exit 6
  }

  # Prepare target directory
  if not ($target_dir | path exists) { mkdir $target_dir }

  # No backup: overwrite atomically via temp file

  # Atomic write: temp then move
  let tmp_path = $"($target_path).tmp"
  try { cp --force $store_path $tmp_path } catch {|err|
    print $"Error writing temporary file: ($err.msg)"
    exit 7
  }
  mv --force $tmp_path $target_path
  print $"Updated vendored zjstatus at: ($target_path) \(size=($byte_len) bytes, source=($flake_ref)\)"
}
