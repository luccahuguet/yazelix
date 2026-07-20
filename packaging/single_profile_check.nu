# YZXCONV-003 — single-profile closure contract check (read-only).
#
# Verifies that exactly one Nix profile selector owns the LifeOS foundation:
#   1. selector_resolves          ~/.nix-profile resolves through the nix-managed
#                                 profile link to a profile with a manifest.json
#   2. single_foundation_element  the manifest holds exactly one element, named
#                                 lifeos_foundation_yzx, with store-backed paths
#   3. xdg_selector_convergent    the XDG profiles/profile selector is inactive
#                                 or resolves to the same closure
#   4. foundation_binaries_resolve yzx/codex/claude/rtk (bin) and nu (toolbin)
#                                 are executable and store-backed
#   5. path_single_owner          (YZX_CHECK_PATH=1) every PATH resolution of the
#                                 foundation binaries realpaths into the profile
#   6. closure_matches_expected   (YZX_EXPECTED_CLOSURE set) the manifest element
#                                 storePaths equal exactly [expected closure]
#
# Prints a JSON report to stdout; exits 0 only if every evaluated clause holds.
# Environment overrides (used by fixtures, staging, and the flake check):
#   YZX_PROFILE_LINK   default /home/flexnetos/.nix-profile
#   YZX_NIX_PROFILE    default /home/flexnetos/.local/state/nix/profile
#   YZX_XDG_PROFILE    default /home/flexnetos/.local/state/nix/profiles/profile
#   YZX_STORE_PREFIX   default /nix/store
#   YZX_EXPECTED_CLOSURE, YZX_CHECK_PATH  optional clause activators

def resolve [path: string] {
  let res = (do { ^readlink -f $path } | complete)
  if $res.exit_code == 0 { $res.stdout | str trim } else { "" }
}

def is-executable [path: string] {
  if not ($path | path exists) { return false }
  (ls -l $path | get 0.mode | str contains "x")
}

def main [] {
  let profile_link = ($env.YZX_PROFILE_LINK? | default "/home/flexnetos/.nix-profile")
  let nix_profile = ($env.YZX_NIX_PROFILE? | default "/home/flexnetos/.local/state/nix/profile")
  let xdg_profile = ($env.YZX_XDG_PROFILE? | default "/home/flexnetos/.local/state/nix/profiles/profile")
  let store_prefix = ($env.YZX_STORE_PREFIX? | default "/nix/store")
  let expected = ($env.YZX_EXPECTED_CLOSURE? | default "")
  let check_path = (($env.YZX_CHECK_PATH? | default "") == "1")

  # 1. selector resolves: interactive link and nix profile link converge on one
  #    profile directory that carries a nix profile manifest.
  let resolved_link = (resolve $profile_link)
  let resolved_nix = (resolve $nix_profile)
  let selector_resolves = (
    $resolved_link != ""
    and $resolved_link == $resolved_nix
    and ($resolved_link | path type) == "dir"
    and (($resolved_link | path join "manifest.json") | path exists)
  )

  # 2. exactly one foundation element in the profile manifest.
  mut single_element = false
  mut element_paths = []
  if $selector_resolves {
    let manifest = (open --raw ($resolved_link | path join "manifest.json") | from json)
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

  # 3. XDG selector inactive or convergent.
  let xdg_selector_convergent = if not ($xdg_profile | path exists) {
    true
  } else {
    (resolve $xdg_profile) == $resolved_link and $selector_resolves
  }

  # 4. foundation binaries executable and store-backed.
  let bin_specs = [[dir name]; [bin yzx] [bin codex] [bin claude] [bin rtk] [toolbin nu]]
  let binary_reports = if $selector_resolves {
    $bin_specs | each {|s|
      let p = ($resolved_link | path join $s.dir | path join $s.name)
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

  # 5. optional: every PATH resolution realpaths into this profile.
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

  # 6. optional: manifest element pinned to exactly the expected closure.
  let closure_matches_expected = if $expected == "" {
    null
  } else {
    $single_element and $element_paths == [$expected]
  }

  let clauses = {
    selector_resolves: $selector_resolves
    single_foundation_element: $single_element
    xdg_selector_convergent: $xdg_selector_convergent
    foundation_binaries_resolve: $foundation_binaries_resolve
    path_single_owner: $path_single_owner
    closure_matches_expected: $closure_matches_expected
  }
  let pass = ($clauses | values | all {|v| $v == null or $v == true })

  let report = {
    schema: "yazelix.single-profile-check.v1"
    task: "YZXCONV-003"
    observed_at: (^date -u +%Y-%m-%dT%H:%M:%SZ | str trim)
    profile_link: $profile_link
    nix_profile: $nix_profile
    xdg_profile: $xdg_profile
    resolved_profile: $resolved_link
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
