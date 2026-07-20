# YZXCONV-003 — single-profile cutover migration (dry-run by default).
#
# Converges the LifeOS foundation onto ~/.nix-profile as the sole selector:
#   * archives (never deletes) a divergent XDG profiles/profile selector,
#   * replaces the lifeos_foundation_yzx profile element with the given closure
#     via `nix profile remove` + `nix profile install`,
#   * verifies the result with single_profile_check.nu and atomically rolls the
#     profile symlink back to the recorded prior target on any failure,
#   * writes a migration receipt (prior symlink target, new closure path,
#     rollback command) for every run.
#
# WITHOUT --execute this performs NO mutation: it only records the plan.
#
#   nu packaging/profile_migration.nu --closure /nix/store/...-lifeos-foundation-yzx \
#     [--flake-ref path:/home/flexnetos/meta/src/yazelix] [--receipt-dir DIR] [--execute]
#
# Environment overrides (fixtures/staging): YZX_PROFILE_LINK, YZX_NIX_PROFILE,
# YZX_XDG_PROFILE, YZX_STORE_PREFIX, YZX_NIX_BIN, YZX_NU_BIN, YZX_CHECK_SCRIPT.

def resolve [path: string] {
  let res = (do { ^readlink -f $path } | complete)
  if $res.exit_code == 0 { $res.stdout | str trim } else { "" }
}

def refuse [msg: string] {
  print -e $"refusing: ($msg)"
  exit 2
}

# Atomic selector restore: build the replacement link aside, rename over.
def rollback-selector [nix_profile: string, prior_target: string] {
  let tmp = $"($nix_profile).rollback-tmp"
  if ($tmp | path exists) or (($tmp | path type) == "symlink") {
    ^rm -f $tmp
  }
  ^ln -s $prior_target $tmp
  ^mv -T $tmp $nix_profile
}

def main [
  --closure: string = ""     # freshly built lifeos-foundation-yzx store path (required)
  --flake-ref: string = "path:/home/flexnetos/meta/src/yazelix"  # install source
  --receipt-dir: string = "."  # where the migration receipt is written
  --execute                  # actually mutate; the default is a read-only dry-run
] {
  let profile_link = ($env.YZX_PROFILE_LINK? | default "/home/flexnetos/.nix-profile")
  let nix_profile = ($env.YZX_NIX_PROFILE? | default "/home/flexnetos/.local/state/nix/profile")
  let xdg_profile = ($env.YZX_XDG_PROFILE? | default "/home/flexnetos/.local/state/nix/profiles/profile")
  let store_prefix = ($env.YZX_STORE_PREFIX? | default "/nix/store")
  let nix_bin = ($env.YZX_NIX_BIN? | default "nix")
  let nu_bin = ($env.YZX_NU_BIN? | default "nu")
  let check_script = ($env.YZX_CHECK_SCRIPT? | default ($env.FILE_PWD | path join "single_profile_check.nu"))

  if $closure == "" { refuse "--closure <store path of lifeos-foundation-yzx> is required" }
  if not ($closure | str starts-with $store_prefix) { refuse $"closure ($closure) is not under ($store_prefix)" }
  if not (($closure | path type) == "dir") { refuse $"closure ($closure) is not a directory" }
  if not ($check_script | path exists) { refuse $"check script not found at ($check_script)" }

  # Record the pre-change selector state.
  let prior_target = ((do { ^readlink $nix_profile } | complete).stdout | str trim)
  if $prior_target == "" { refuse $"($nix_profile) is not a symlink; no prior generation to record" }
  let prior_resolved = (resolve $nix_profile)
  let alias_resolved = (resolve $profile_link)
  if $alias_resolved != $prior_resolved {
    refuse $"($profile_link) does not resolve through ($nix_profile) — selector alias is broken"
  }

  let stamp = (^date -u +%Y%m%dT%H%M%SZ | str trim)
  let xdg_archive_path = $"($xdg_profile).archived-($stamp)"
  let xdg_state = if not ($xdg_profile | path exists) {
    "absent"
  } else if (resolve $xdg_profile) == $prior_resolved {
    "convergent"
  } else {
    "divergent"
  }
  let xdg_action = if $xdg_state == "divergent" {
    $"archive to ($xdg_archive_path)"
  } else {
    "none"
  }

  let install_commands = [
    $"($nix_bin) profile remove --profile ($nix_profile) lifeos_foundation_yzx"
    $"($nix_bin) profile install --profile ($nix_profile) '($flake_ref)#lifeos_foundation_yzx'"
  ]
  let rollback_command = $"ln -s '($prior_target)' '($nix_profile).rollback-tmp' && mv -T '($nix_profile).rollback-tmp' '($nix_profile)'"
  let mode = if $execute { "execute" } else { "dry-run" }

  mut receipt = {
    schema: "yazelix.single-profile-migration.receipt.v1"
    task: "YZXCONV-003"
    observed_at: (^date -u +%Y-%m-%dT%H:%M:%SZ | str trim)
    mode: $mode
    profile_link: $profile_link
    nix_profile: $nix_profile
    prior_profile_target: $prior_target
    prior_profile_resolved: $prior_resolved
    new_closure_path: $closure
    flake_ref: $flake_ref
    xdg_profile: $xdg_profile
    xdg_profile_state: $xdg_state
    xdg_profile_action: $xdg_action
    install_commands: $install_commands
    rollback_command: $rollback_command
    verified: null
    rollback_performed: false
  }

  mut failed = false
  if $execute {
    # 1. Take the divergent XDG selector out of play — archive, never delete.
    if $xdg_state == "divergent" {
      ^mv -T $xdg_profile $xdg_archive_path
    }

    # 2. Replace the foundation element (remove only when present).
    let manifest_path = ($prior_resolved | path join "manifest.json")
    let has_element = if ($manifest_path | path exists) {
      let manifest = (open --raw $manifest_path | from json)
      "lifeos_foundation_yzx" in (($manifest.elements? | default {}) | columns)
    } else { false }
    mut install_failed = false
    if $has_element {
      let remove = (do { ^$nix_bin profile remove --profile $nix_profile lifeos_foundation_yzx } | complete)
      if $remove.exit_code != 0 {
        print -e $remove.stderr
        $install_failed = true
      }
    }
    if not $install_failed {
      let install = (do { ^$nix_bin profile install --profile $nix_profile $"($flake_ref)#lifeos_foundation_yzx" } | complete)
      if $install.exit_code != 0 {
        print -e $install.stderr
        $install_failed = true
      }
    }

    # 3. Verify the converged selector; roll back atomically on any failure.
    let verified = if $install_failed {
      false
    } else {
      let verify = (with-env {YZX_EXPECTED_CLOSURE: $closure} {
        do { ^$nu_bin $check_script } | complete
      })
      print $verify.stdout
      $verify.exit_code == 0
    }
    $receipt.verified = $verified
    if not $verified {
      rollback-selector $nix_profile $prior_target
      $receipt.rollback_performed = true
      $failed = true
    }
  }

  let receipt_path = ($receipt_dir | path join $"single-profile-migration.receipt.($stamp).json")
  $receipt | to json --indent 2 | save --force $receipt_path
  print $"receipt: ($receipt_path)"
  print ($receipt | to json --indent 2)
  if $failed {
    print -e "cutover verification failed; selector rolled back to prior target"
    exit 1
  }
}
