# YZXCONV-003 — explicit ~/.nix-profile cutover (dry-run by default).
#
# Converges the LifeOS foundation onto ~/.nix-profile as the sole selector:
#   * archives the prior ~/.nix-profile alias or generation selector,
#   * archives the retired user XDG selector and its generation links,
#   * creates a fresh explicit ~/.nix-profile with `nix profile add`,
#   * verifies the expected closure with single_profile_check.nu,
#   * archives a failed candidate and restores every prior selector link, and
#   * writes a hash-bearing migration receipt for dry-run and execute modes.
#
# WITHOUT --execute this performs NO selector mutation: it records the plan.
#
#   nu packaging/profile_migration.nu --closure /nix/store/...-lifeos-foundation-yzx \
#     [--flake-ref path:/home/flexnetos/meta/src/yazelix] \
#     [--archive-dir /home/flexnetos/.cache/flexnetos/archives/yazelix-nix-profile] \
#     [--receipt-dir DIR] [--execute]
#
# Environment overrides (fixtures/staging): YZX_PROFILE_LINK,
# YZX_LEGACY_XDG_PROFILE, YZX_STORE_PREFIX, YZX_NIX_BIN, YZX_NU_BIN,
# YZX_CHECK_SCRIPT.

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

def entry-present [path: string] {
  let probe = (read-link $path)
  $probe.ok or ($path | path exists)
}

def file-sha256 [path: string] {
  if ($path | path exists) {
    open --raw $path | hash sha256
  } else {
    ""
  }
}

def generation-links [selector: string] {
  let directory = ($selector | path dirname)
  let selector_name = ($selector | path basename)
  let prefix = $"($selector_name)-"
  if not ($directory | path exists) { return [] }
  ls -a $directory
  | where type == symlink
  | where {|entry|
      let name = ($entry.name | path basename)
      let body = ($name | str replace $prefix "")
      ($name | str starts-with $prefix) and ($body =~ '^[0-9]+-link(-[0-9]+-link)*$')
    }
  | get name
}

def refuse [msg: string] {
  print -e $"refusing: ($msg)"
  exit 2
}

def archive-entries [entries: list<record>] {
  for entry in $entries {
    if (entry-present $entry.source) {
      mkdir ($entry.archived | path dirname)
      ^mv -T $entry.source $entry.archived
    }
  }
}

def restore-entries [entries: list<record>] {
  for entry in ($entries | reverse) {
    if (entry-present $entry.archived) {
      if (entry-present $entry.source) {
        error make {msg: $"rollback destination is occupied: ($entry.source)"}
      }
      mkdir ($entry.source | path dirname)
      ^mv -T $entry.archived $entry.source
    }
  }
}

def archive-candidate [profile_link: string, destination: string] {
  let paths = ((generation-links $profile_link) | append $profile_link | uniq)
  for source in $paths {
    if (entry-present $source) {
      mkdir $destination
      ^mv -T $source ($destination | path join ($source | path basename))
    }
  }
}

def main [
  --closure: string = ""     # freshly built lifeos-foundation-yzx store path (required)
  --flake-ref: string = "path:/home/flexnetos/meta/src/yazelix"  # install source
  --archive-dir: string = "/home/flexnetos/.cache/flexnetos/archives/yazelix-nix-profile"
  --receipt-dir: string = "."  # where the migration receipt is written
  --execute                    # actually mutate; default is a read-only dry-run
] {
  let profile_link = ($env.YZX_PROFILE_LINK? | default "/home/flexnetos/.nix-profile")
  let retired_home_tree = (["." "local"] | str join)
  let legacy_xdg_profile = (
    $env.YZX_LEGACY_XDG_PROFILE?
    | default $"/home/flexnetos/($retired_home_tree)/state/nix/profile"
  )
  let store_prefix = ($env.YZX_STORE_PREFIX? | default "/nix/store")
  let nix_bin = ($env.YZX_NIX_BIN? | default "nix")
  let nu_bin = ($env.YZX_NU_BIN? | default "nu")
  let check_script = ($env.YZX_CHECK_SCRIPT? | default ($env.FILE_PWD | path join "single_profile_check.nu"))

  if $closure == "" { refuse "--closure <store path of lifeos-foundation-yzx> is required" }
  if not ($closure | str starts-with $store_prefix) { refuse $"closure ($closure) is not under ($store_prefix)" }
  if not (($closure | path type) == "dir") { refuse $"closure ($closure) is not a directory" }
  if not ($check_script | path exists) { refuse $"check script not found at ($check_script)" }
  if not (entry-present $profile_link) { refuse $"active profile selector is absent: ($profile_link)" }

  let prior_profile_target = (read-link $profile_link | get target)
  let prior_profile_resolved = (resolve $profile_link)
  if $prior_profile_resolved == "" { refuse $"active profile does not resolve: ($profile_link)" }
  let prior_manifest = ($prior_profile_resolved | path join "manifest.json")
  if not ($prior_manifest | path exists) { refuse $"active profile manifest is absent: ($prior_manifest)" }

  let stamp = (^date -u +%Y%m%dT%H%M%S%NZ | str trim)
  let archive_path = ($archive_dir | path join $stamp)
  let profile_paths = ((generation-links $profile_link) | append $profile_link)
  let legacy_paths = if (entry-present $legacy_xdg_profile) {
    (generation-links $legacy_xdg_profile) | append $legacy_xdg_profile
  } else { [] }
  let prior_paths = ($profile_paths | append $legacy_paths | uniq)
  let prior_entries = ($prior_paths | each {|source|
    let probe = (read-link $source)
    {
      source: $source
      archived: ($archive_path | path join "prior" | path join ($source | path basename))
      target: $probe.target
      resolved: (resolve $source)
    }
  })

  let install_command = $"($nix_bin) profile add --profile '($profile_link)' '($flake_ref)#lifeos_foundation_yzx'"
  let mode = if $execute { "execute" } else { "dry-run" }
  mut receipt = {
    schema: "yazelix.single-profile-migration.receipt.v2"
    task: "YZXCONV-003"
    observed_at: (^date -u +%Y-%m-%dT%H:%M:%SZ | str trim)
    mode: $mode
    profile_link: $profile_link
    legacy_xdg_profile: $legacy_xdg_profile
    prior_profile_target: $prior_profile_target
    prior_profile_resolved: $prior_profile_resolved
    prior_manifest_sha256: (file-sha256 $prior_manifest)
    new_closure_path: $closure
    flake_ref: $flake_ref
    archive_path: $archive_path
    archive_entries: $prior_entries
    install_command: $install_command
    install_exit_code: null
    rollback_actions: ($prior_entries | reverse | each {|entry| {from: $entry.archived, to: $entry.source}})
    new_profile_resolved: null
    new_manifest_sha256: null
    verification_exit_code: null
    verification_stdout_sha256: null
    verified: null
    rollback_performed: false
    failed_candidate_archive: null
    failure_stage: null
    errors: []
  }

  mut failed = false
  if $execute {
    let archive_result = (try {
      mkdir $archive_path
      archive-entries $prior_entries
      {ok: true, error: null}
    } catch {|err|
      {ok: false, error: ($err.msg? | default ($err | to json --raw))}
    })

    if not $archive_result.ok {
      $receipt.failure_stage = "archive-prior"
      $receipt.errors = ($receipt.errors | append $archive_result.error)
      let restore_result = (try {
        restore-entries $prior_entries
        {ok: true, error: null}
      } catch {|err|
        {ok: false, error: ($err.msg? | default ($err | to json --raw))}
      })
      $receipt.rollback_performed = $restore_result.ok
      if not $restore_result.ok {
        $receipt.errors = ($receipt.errors | append $restore_result.error)
      }
      $receipt.verified = false
      $failed = true
    } else {
      let install = (do {
        ^$nix_bin profile add --profile $profile_link $"($flake_ref)#lifeos_foundation_yzx"
      } | complete)
      $receipt.install_exit_code = $install.exit_code
      if $install.exit_code != 0 {
        print -e $install.stderr
        $receipt.failure_stage = "install-profile"
        let install_error = ($install.stderr | str trim)
        $receipt.errors = ($receipt.errors | append (
          if $install_error == "" { $"profile install exited ($install.exit_code)" } else { $install_error }
        ))
      }

      let verified = if $install.exit_code != 0 {
        false
      } else {
        let verify = (with-env {
          YZX_PROFILE_LINK: $profile_link
          YZX_LEGACY_XDG_PROFILE: $legacy_xdg_profile
          YZX_STORE_PREFIX: $store_prefix
          YZX_EXPECTED_CLOSURE: $closure
        } {
          do { ^$nu_bin $check_script } | complete
        })
        $receipt.verification_exit_code = $verify.exit_code
        $receipt.verification_stdout_sha256 = ($verify.stdout | hash sha256)
        print $verify.stdout
        if $verify.exit_code != 0 {
          print -e $verify.stderr
          $receipt.failure_stage = "verify-profile"
          let verify_error = ($verify.stderr | str trim)
          $receipt.errors = ($receipt.errors | append (
            if $verify_error == "" { $"profile verification exited ($verify.exit_code)" } else { $verify_error }
          ))
        }
        $verify.exit_code == 0
      }
      $receipt.verified = $verified

      if $verified {
        let new_profile = (resolve $profile_link)
        $receipt.new_profile_resolved = $new_profile
        $receipt.new_manifest_sha256 = (file-sha256 ($new_profile | path join "manifest.json"))
      } else {
        let failed_archive = ($archive_path | path join "failed-candidate")
        let candidate_result = (try {
          archive-candidate $profile_link $failed_archive
          {ok: true, error: null}
        } catch {|err|
          {ok: false, error: ($err.msg? | default ($err | to json --raw))}
        })
        if not $candidate_result.ok {
          $receipt.errors = ($receipt.errors | append $candidate_result.error)
        }
        let restore_result = (try {
          restore-entries $prior_entries
          {ok: true, error: null}
        } catch {|err|
          {ok: false, error: ($err.msg? | default ($err | to json --raw))}
        })
        if not $restore_result.ok {
          $receipt.errors = ($receipt.errors | append $restore_result.error)
        }
        $receipt.rollback_performed = $restore_result.ok
        $receipt.failed_candidate_archive = $failed_archive
        $failed = true
      }
    }
  }

  mkdir $receipt_dir
  let receipt_path = ($receipt_dir | path join $"single-profile-migration.receipt.($stamp).json")
  $receipt | to json --indent 2 | save --force $receipt_path
  print $"receipt: ($receipt_path)"
  print ($receipt | to json --indent 2)
  if $failed {
    if $receipt.rollback_performed {
      print -e "cutover failed; every prior selector link was restored"
    } else {
      print -e $"cutover failed and automatic recovery was incomplete; follow receipt rollback_actions: ($receipt_path)"
    }
    exit 1
  }
}
