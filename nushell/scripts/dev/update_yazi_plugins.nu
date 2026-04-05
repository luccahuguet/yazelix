#!/usr/bin/env nu
# Refresh vendored Yazi plugin runtime files from pinned upstream sources.

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const DEFAULT_MANIFEST = ($REPO_ROOT | path join "config_metadata" "vendored_yazi_plugins.toml")

def fail [message: string, exit_code: int = 1] {
  print --stderr $"Error: ($message)"
  exit $exit_code
}

def ensure_git_available [] {
  if (which git | is-empty) {
    fail "git is not available in PATH" 2
  }
}

def load_manifest [manifest_path: string] {
  if not ($manifest_path | path exists) {
    fail $"Vendored Yazi plugin manifest not found: ($manifest_path)" 3
  }

  let manifest = (open --raw $manifest_path | from toml)
  let plugins = ($manifest.plugins? | default [])

  if ($plugins | is-empty) {
    fail $"Vendored Yazi plugin manifest has no plugin entries: ($manifest_path)" 4
  }

  {
    manifest: $manifest
    plugins: $plugins
  }
}

def ensure_clean_managed_targets [repo_root: string, entry: record] {
  let target_dir = ($repo_root | path join ($entry.target_dir | into string))
  let managed_files = ($entry.managed_files? | default [])
  if ($managed_files | is-empty) {
    fail $"Vendored Yazi plugin entry has no managed files: ($entry.name)" 16
  }
  let target_paths = (
    $managed_files
    | each {|rel| $target_dir | path join ($rel | into string) }
  )

  let result = (^git -C $repo_root status --porcelain -- ...$target_paths | complete)
  if $result.exit_code != 0 {
    fail $"Failed to inspect git status for managed Yazi plugin targets under ($target_dir): (($result.stderr | str trim))" 7
  }

  let dirty = ($result.stdout | lines | where {|line| ($line | str trim) != "" })
  if ($dirty | is-not-empty) {
    let target_name = ($entry.name | into string)
    let dirty_text = ($dirty | str join "; ")
    fail $"Local changes detected in managed vendored plugin files for ($target_name): ($dirty_text). Commit, stash, or revert them before refreshing vendored Yazi plugins." 8
  }
}

def clone_repo_at_rev [repo_url: string, rev: string, temp_dir: string] {
  let result = (^git clone --quiet $repo_url $temp_dir | complete)
  if $result.exit_code != 0 {
    fail $"Failed to clone ($repo_url): (($result.stderr | str trim))" 9
  }

  let checkout = (^git -C $temp_dir checkout --quiet $rev | complete)
  if $checkout.exit_code != 0 {
    fail $"Failed to checkout revision ($rev) for ($repo_url): (($checkout.stderr | str trim))" 10
  }
}

def copy_managed_files [source_root: string, stage_root: string, managed_files: list<any>] {
  for rel_path in $managed_files {
    let relative = ($rel_path | into string)
    let source_path = ($source_root | path join $relative)
    if not ($source_path | path exists) {
      fail $"Managed vendored file not found in upstream source: ($source_path)" 11
    }

    let target_path = ($stage_root | path join $relative)
    let parent_dir = ($target_path | path dirname)
    if not ($parent_dir | path exists) {
      mkdir $parent_dir
    }

    cp --force $source_path $target_path
  }
}

def apply_patch_overlay [repo_root: string, stage_root: string, patch_file: string] {
  if (($patch_file | str trim) == "") {
    return
  }

  let patch_path = ($repo_root | path join $patch_file)
  if not ($patch_path | path exists) {
    fail $"Vendored Yazi plugin patch not found: ($patch_path)" 12
  }

  let result = (do {
    cd $stage_root
    ^git apply $patch_path
  } | complete)
  if $result.exit_code != 0 {
    fail $"Failed to apply vendored Yazi plugin patch ($patch_file): (($result.stderr | str trim))" 13
  }
}

def install_staged_files [stage_root: string, target_root: string, managed_files: list<any>] {
  if not ($target_root | path exists) {
    mkdir $target_root
  }

  for rel_path in $managed_files {
    let relative = ($rel_path | into string)
    let source_path = ($stage_root | path join $relative)
    let target_path = ($target_root | path join $relative)
    let parent_dir = ($target_path | path dirname)
    if not ($parent_dir | path exists) {
      mkdir $parent_dir
    }
    cp --force $source_path $target_path
  }
}

def refresh_plugin [repo_root: string, entry: record, rev: string, --quiet] {
  let checkout_dir = (^mktemp -d /tmp/yazelix_vendored_yazi_checkout_XXXXXX | str trim)
  let stage_dir = (^mktemp -d /tmp/yazelix_vendored_yazi_stage_XXXXXX | str trim)

  try {
    clone_repo_at_rev ($entry.upstream_repo | into string) $rev $checkout_dir

    let source_root = if (($entry.source_subdir | default "." | into string) == ".") {
      $checkout_dir
    } else {
      $checkout_dir | path join ($entry.source_subdir | into string)
    }
    if not ($source_root | path exists) {
      fail $"Source subdir missing for vendored Yazi plugin ($entry.name): ($source_root)" 14
    }

    let managed_files = ($entry.managed_files? | default [])
    copy_managed_files $source_root $stage_dir $managed_files
    apply_patch_overlay $repo_root $stage_dir ($entry.patch_file? | default "")
    install_staged_files $stage_dir ($repo_root | path join ($entry.target_dir | into string)) $managed_files

    if not $quiet {
      print $"Updated vendored Yazi plugin runtime files for ($entry.name) from ($rev)"
    }
  } finally {
    rm -rf $checkout_dir
    rm -rf $stage_dir
  }
}

export def main [
  --repo-root: string = $REPO_ROOT
  --manifest: string = $DEFAULT_MANIFEST
  --quiet
] {
  ensure_git_available

  let repo_root = ($repo_root | path expand)
  let manifest_path = ($manifest | path expand)
  let loaded = (load_manifest $manifest_path)
  let plugins = $loaded.plugins

  for entry in $plugins {
    ensure_clean_managed_targets $repo_root $entry

    let chosen_rev = ($entry.pinned_rev | into string)

    if ($chosen_rev | str trim | is-empty) {
      fail $"Vendored Yazi plugin entry is missing a pinned revision: ($entry.name)" 15
    }

    refresh_plugin $repo_root $entry $chosen_rev --quiet=$quiet
  }

  if not $quiet {
    let names = ($plugins | each {|entry| $entry.name | into string } | str join ", ")
    print $"Vendored Yazi plugin runtime files are in sync: ($names)"
  }
}
