# YZXCONV-003 — single-profile closure contract check (read-only).
#
# Verifies that exactly one Nix profile selector owns the LifeOS foundation:
#   1. direct_profile_selector     ~/.nix-profile is the explicit selector and
#                                  points to its own .nix-profile-N-link
#   2. selector_resolves           the selected profile has manifest.json
#   3. single_foundation_element   the manifest contains only
#                                  lifeos_foundation_yzx
#   4. legacy_xdg_inactive         ~/.local/state/nix/profile is absent, even
#                                  when it would resolve to the same closure
#   5. foundation_binaries_resolve yzx/codex/claude/rtk/br/bun/git-kb/icm/nix
#                                  (bin) and nu (toolbin) are executable and
#                                  store-backed
#   6. path_single_owner           (YZX_CHECK_PATH=1) every PATH resolution of
#                                  the foundation binaries resolves identically
#   7. closure_matches_expected    (YZX_EXPECTED_CLOSURE set) the manifest
#                                  element storePaths equal [expected closure]
#
# Prints a JSON report to stdout; exits 0 only if every evaluated clause holds.
# Environment overrides (used by fixtures, staging, and the flake check):
#   YZX_PROFILE_LINK        default /home/flexnetos/.nix-profile
#   YZX_LEGACY_XDG_PROFILE  default /home/flexnetos/.local/state/nix/profile
#   YZX_STORE_PREFIX        default /nix/store
#   YZX_EXPECTED_CLOSURE, YZX_CHECK_PATH  optional clause activators

def resolve [path: string] {
  let res = (do { ^readlink -f $path } | complete)
  if $res.exit_code == 0 { $res.stdout | str trim } else { "" }
}

def read-link [path: string] {
  let res = (do { ^readlink $path } | complete)
  {
    ok: ($res.exit_code == 0)
    target: (if $res.exit_code == 0 { $res.stdout | str trim } else { "" })
  }
}

def is-executable [path: string] {
  if not ($path | path exists) { return false }
  (ls -l $path | get 0.mode | str contains "x")
}

def main [] {
  let profile_link = ($env.YZX_PROFILE_LINK? | default "/home/flexnetos/.nix-profile")
  let legacy_xdg_profile = (
    $env.YZX_LEGACY_XDG_PROFILE? | default "/home/flexnetos/.local/state/nix/profile"
  )
  let store_prefix = ($env.YZX_STORE_PREFIX? | default "/nix/store")
  let expected = ($env.YZX_EXPECTED_CLOSURE? | default "")
  let check_path = (($env.YZX_CHECK_PATH? | default "") == "1")

  let selector_link = (read-link $profile_link)
  let profile_name = ($profile_link | path basename)
  let generation_prefix = $"($profile_name)-"
  let generation_body = ($selector_link.target | str replace $generation_prefix "")
  let direct_profile_selector = (
    $selector_link.ok
    and not ($selector_link.target | str contains "/")
    and ($selector_link.target | str starts-with $generation_prefix)
    and ($generation_body =~ '^[0-9]+-link$')
  )

  let resolved_profile = (resolve $profile_link)
  let selector_resolves = (
    $direct_profile_selector
    and $resolved_profile != ""
    and ($resolved_profile | path type) == "dir"
    and (($resolved_profile | path join "manifest.json") | path exists)
  )

  mut single_element = false
  mut element_paths = []
  if $selector_resolves {
    let manifest = (open --raw ($resolved_profile | path join "manifest.json") | from json)
    if ($manifest.version? | default 0) == 3 {
      let names = ($manifest.elements | columns)
      if $names == ["lifeos_foundation_yzx"] {
        let paths = ($manifest.elements.lifeos_foundation_yzx.storePaths? | default [])
        if ($paths | length) == 1 and ($paths | all {|p| $p | str starts-with $store_prefix }) {
          $single_element = true
          $element_paths = $paths
        }
      }
    }
  }

  let legacy_probe = (read-link $legacy_xdg_profile)
  let legacy_xdg_inactive = (
    not $legacy_probe.ok and not ($legacy_xdg_profile | path exists)
  )

  let bin_specs = [[dir name];
    [bin yzx]
    [bin codex]
    [bin claude]
    [bin rtk]
    [bin br]
    [bin bun]
    [bin git-kb]
    [bin icm]
    [bin nix]
    [toolbin nu]
  ]
  let binary_reports = if $selector_resolves {
    $bin_specs | each {|s|
      let p = ($resolved_profile | path join $s.dir | path join $s.name)
      let rp = (resolve $p)
      {
        name: $s.name
        path: $p
        realpath: $rp
        ok: ($rp != "" and ($rp | str starts-with $store_prefix) and (is-executable $rp))
      }
    }
  } else { [] }
  let foundation_binaries_resolve = (
    $selector_resolves and ($binary_reports | all {|b| $b.ok })
  )

  let path_single_owner = if not $check_path {
    null
  } else if not $foundation_binaries_resolve {
    false
  } else {
    $binary_reports | all {|b|
      let hits = (which --all $b.name | where type == "external" | get path)
      ($hits | length) > 0 and ($hits | all {|h| (resolve $h) == $b.realpath })
    }
  }

  let closure_matches_expected = if $expected == "" {
    null
  } else {
    $single_element and $element_paths == [$expected]
  }

  let clauses = {
    direct_profile_selector: $direct_profile_selector
    selector_resolves: $selector_resolves
    single_foundation_element: $single_element
    legacy_xdg_inactive: $legacy_xdg_inactive
    foundation_binaries_resolve: $foundation_binaries_resolve
    path_single_owner: $path_single_owner
    closure_matches_expected: $closure_matches_expected
  }
  let pass = ($clauses | values | all {|v| $v == null or $v == true })

  let report = {
    schema: "yazelix.single-profile-check.v2"
    task: "YZXCONV-003"
    observed_at: (^date -u +%Y-%m-%dT%H:%M:%SZ | str trim)
    profile_link: $profile_link
    profile_link_target: $selector_link.target
    legacy_xdg_profile: $legacy_xdg_profile
    legacy_xdg_target: $legacy_probe.target
    resolved_profile: $resolved_profile
    element_store_paths: $element_paths
    binaries: $binary_reports
    clauses: $clauses
    pass: $pass
  }
  print ($report | to json --indent 2)
  if not $pass {
    exit 1
  }
}
