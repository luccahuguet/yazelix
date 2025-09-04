#!/usr/bin/env nu
# Update zjstatus plugin (Zellij status bar) for Yazelix
# Fetches the latest GitHub release wasm and installs it to
# ~/.config/yazelix/configs/zellij/zjstatus.wasm (no backup).

export def main [] {
  let home = $env.HOME
  if ($home | is-empty) or (not ($home | path exists)) {
    print "Error: Cannot resolve HOME directory"
    exit 1
  }

  let target_dir = $"($home)/.config/yazelix/configs/zellij"
  let target_path = $"($target_dir)/zjstatus.wasm"

  # Determine latest release asset URL from GitHub API using Nushell's HTTP client
  let api = "https://api.github.com/repos/dj95/zjstatus/releases/latest"
  let release = (try { http get $api } catch {|err|
    print $"Error fetching latest release info: ($err.msg)"
    exit 2
  })
  let asset_url = ($release.assets | where name == "zjstatus.wasm" | get browser_download_url | first)
  if ($asset_url | is-empty) {
    print "Error: Could not find zjstatus.wasm in latest release assets"
    exit 3
  }

  # Download the wasm using Nushell's HTTP client
  print $"Downloading: ($asset_url)"
  let data = (try { http get --raw $asset_url } catch {|err|
    print $"Error downloading zjstatus.wasm: ($err.msg)"
    exit 3
  })

  # Minimal validation: check file size and extension hint
  let byte_len = ($data | bytes length)
  if $byte_len < 1024 {
    print $"Error: Downloaded/provided file is too small to be a valid wasm (size=($byte_len) bytes)"
    exit 5
  }

  # Prepare target directory
  if not ($target_dir | path exists) { mkdir $target_dir }

  # No backup: overwrite atomically via temp file

  # Atomic write: temp then move
  let tmp_path = $"($target_path).tmp"
  try { $data | save --force $tmp_path } catch {|err|
    print $"Error writing temporary file: ($err.msg)"
    exit 6
  }
  mv -f $tmp_path $target_path
  print $"Updated zjstatus at: ($target_path) (size=($byte_len) bytes)"
}
