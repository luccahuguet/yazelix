#!/usr/bin/env nu
# Update zjstatus plugin (Zellij status bar) for Yazelix
# Refreshes the vendored zjstatus wasm from the pinned `zjstatus` flake input.

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)

def copy_zjstatus_from_store [store_root: string, target_dir: string] {
  let store_path = ($store_root | path join "bin" "zjstatus.wasm")
  if not ($store_path | path exists) {
    print $"Error: zjstatus wasm not found at: ($store_path)"
    exit 5
  }

  let byte_len = (open --raw $store_path | length)
  if $byte_len < 1024 {
    print $"Error: Nix-provided wasm is too small to be valid \(size=($byte_len) bytes\)"
    exit 6
  }

  let target_path = ($target_dir | path join "zjstatus.wasm")
  let tmp_path = $"($target_path).tmp"
  try { cp --force $store_path $tmp_path } catch {|err|
    print $"Error writing temporary file: ($err.msg)"
    exit 7
  }
  mv --force $tmp_path $target_path

  {
    target_path: $target_path
    byte_len: $byte_len
  }
}

export def main [] {
  if not ($REPO_ROOT | path exists) {
    print $"Error: Cannot resolve repo root from script path: ($REPO_ROOT)"
    exit 1
  }

  let target_dir = ($REPO_ROOT | path join "configs" "zellij" "plugins")
  let lock_path = ($REPO_ROOT | path join "flake.lock")

  if not ($lock_path | path exists) {
    print $"Error: flake.lock not found at: ($lock_path)"
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

  # Prepare target directory
  if not ($target_dir | path exists) { mkdir $target_dir }

  let zjstatus = (copy_zjstatus_from_store $store_root $target_dir)
  print $"Updated vendored zjstatus at: ($zjstatus.target_path) \(size=($zjstatus.byte_len) bytes, source=($flake_ref)\)"
}
