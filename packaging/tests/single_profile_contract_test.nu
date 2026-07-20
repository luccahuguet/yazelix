# YZXCONV-003 — single-profile closure contract tests.
#
# Exercises packaging/single_profile_check.nu and packaging/profile_migration.nu
# against hermetic fixtures (no live profile is read or mutated). Run:
#
#   nu packaging/tests/single_profile_contract_test.nu packaging
#
# Also wired into `nix flake check` via checks.<system>.single_profile_contract.

def expect [cond: bool, msg: string] {
  if not $cond {
    print -e $"FAIL: ($msg)"
    exit 1
  }
  print $"ok: ($msg)"
}

def make-exec [path: string] {
  mkdir ($path | path dirname)
  let nu_bin = (which nu | get path.0)
  [$"#!($nu_bin)" "exit 0" ""] | str join "\n" | save --force $path
  ^chmod +x $path
}

# A fake foundation closure with the contract binaries.
def make-foundation [store: string, name: string] {
  let foundation = ($store | path join $name)
  for b in [yzx codex claude rtk] {
    make-exec ($foundation | path join "bin" | path join $b)
  }
  make-exec ($foundation | path join "toolbin" | path join "nu")
  $foundation
}

# A fake `nix profile` output directory: manifest.json + bin/toolbin links.
def make-profile-dir [store: string, name: string, foundation: string] {
  let profile_dir = ($store | path join $name)
  mkdir $profile_dir
  ^ln -s ($foundation | path join "bin") ($profile_dir | path join "bin")
  ^ln -s ($foundation | path join "toolbin") ($profile_dir | path join "toolbin")
  {
    version: 3
    elements: {
      lifeos_foundation_yzx: {
        active: true
        attrPath: "packages.x86_64-linux.lifeos_foundation_yzx"
        originalUrl: "path:."
        outputs: null
        priority: 5
        storePaths: [$foundation]
        url: "path:."
      }
    }
  } | to json | save --force ($profile_dir | path join "manifest.json")
  $profile_dir
}

# Full convergent selector fixture:
#   <root>/store/aaaa-lifeos-foundation-yzx   fake closure
#   <root>/store/bbbb-profile                 fake profile output (manifest v3)
#   <root>/state/profile-1-link -> profile output
#   <root>/state/profile        -> profile-1-link
#   <root>/state/profiles/                    (XDG selector dir, empty)
#   <root>/home/.nix-profile    -> state/profile
def make-fixture [] {
  let root = (^mktemp -d | str trim)
  let store = ($root | path join "store")
  let foundation = (make-foundation $store "aaaa-lifeos-foundation-yzx")
  let profile_dir = (make-profile-dir $store "bbbb-profile" $foundation)
  let state = ($root | path join "state")
  mkdir ($state | path join "profiles")
  ^ln -s $profile_dir ($state | path join "profile-1-link")
  ^ln -s "profile-1-link" ($state | path join "profile")
  let home = ($root | path join "home")
  mkdir $home
  ^ln -s ($state | path join "profile") ($home | path join ".nix-profile")
  {
    root: $root
    store: $store
    foundation: $foundation
    profile_dir: $profile_dir
    state: $state
    home: $home
  }
}

def fixture-env [fx: record] {
  {
    YZX_PROFILE_LINK: ($fx.home | path join ".nix-profile")
    YZX_NIX_PROFILE: ($fx.state | path join "profile")
    YZX_XDG_PROFILE: ($fx.state | path join "profiles" | path join "profile")
    YZX_STORE_PREFIX: $fx.store
  }
}

def run-check [check_script: string, fx: record, extra: record] {
  with-env ((fixture-env $fx) | merge $extra) {
    do { ^nu $check_script } | complete
  }
}

def run-migrate [migrate_script: string, check_script: string, fx: record, extra: record, args: list<string>] {
  let base = ((fixture-env $fx) | merge {YZX_CHECK_SCRIPT: $check_script})
  with-env ($base | merge $extra) {
    do { ^nu $migrate_script ...$args } | complete
  }
}

def read-receipt [receipt_dir: string] {
  let files = (ls $receipt_dir | where name =~ "single-profile-migration.receipt" | get name)
  expect (($files | length) == 1) $"exactly one receipt written in ($receipt_dir)"
  open --raw ($files | first) | from json
}

def main [packaging_dir: string] {
  let pdir = ($packaging_dir | path expand)
  let check = ($pdir | path join "single_profile_check.nu")
  let migrate = ($pdir | path join "profile_migration.nu")
  expect ($check | path exists) $"check script exists: ($check)"
  expect ($migrate | path exists) $"migration script exists: ($migrate)"

  # --- 1. convergent single-selector fixture passes every clause ---
  let fx1 = (make-fixture)
  let r1 = (run-check $check $fx1 {})
  expect ($r1.exit_code == 0) "convergent fixture: exit 0"
  let j1 = ($r1.stdout | from json)
  expect ($j1.pass == true) "convergent fixture: pass true"
  expect ($j1.clauses.selector_resolves == true) "convergent fixture: selector_resolves"
  expect ($j1.clauses.single_foundation_element == true) "convergent fixture: single_foundation_element"
  expect ($j1.clauses.xdg_selector_convergent == true) "convergent fixture: xdg_selector_convergent"
  expect ($j1.clauses.foundation_binaries_resolve == true) "convergent fixture: foundation_binaries_resolve"

  # --- 2. divergent XDG selector (the EVIDENCE.md two-selector gap) fails ---
  let fx2 = (make-fixture)
  let other2 = ($fx2.store | path join "cccc-second-owner")
  mkdir $other2
  ^ln -s $other2 ($fx2.state | path join "profiles" | path join "profile")
  let r2 = (run-check $check $fx2 {})
  expect ($r2.exit_code != 0) "divergent XDG selector: nonzero exit"
  let j2 = ($r2.stdout | from json)
  expect ($j2.clauses.xdg_selector_convergent == false) "divergent XDG selector: clause false"

  # --- 3. two manifest elements violate the single-element contract ---
  let fx3 = (make-fixture)
  let mpath3 = ($fx3.profile_dir | path join "manifest.json")
  let m3 = (open --raw $mpath3 | from json)
  $m3
  | update elements ($m3.elements | insert second_element {storePaths: [$fx3.foundation]})
  | to json | save --force $mpath3
  let r3 = (run-check $check $fx3 {})
  expect ($r3.exit_code != 0) "two manifest elements: nonzero exit"
  let j3 = ($r3.stdout | from json)
  expect ($j3.clauses.single_foundation_element == false) "two manifest elements: clause false"

  # --- 4. missing foundation binary fails ---
  let fx4 = (make-fixture)
  let claude4 = ($fx4.foundation | path join "bin" | path join "claude")
  ^mv $claude4 $"($claude4).disabled"
  let r4 = (run-check $check $fx4 {})
  expect ($r4.exit_code != 0) "missing claude binary: nonzero exit"
  let j4 = ($r4.stdout | from json)
  expect ($j4.clauses.foundation_binaries_resolve == false) "missing claude binary: clause false"

  # --- 5. expected-closure mismatch fails (the cutover-pending clause) ---
  let fx5 = (make-fixture)
  let other5 = (make-foundation $fx5.store "dddd-lifeos-foundation-new")
  let r5 = (run-check $check $fx5 {YZX_EXPECTED_CLOSURE: $other5})
  expect ($r5.exit_code != 0) "expected-closure mismatch: nonzero exit"
  let j5 = ($r5.stdout | from json)
  expect ($j5.clauses.closure_matches_expected == false) "expected-closure mismatch: clause false"

  # --- 6. expected-closure match passes ---
  let r6 = (run-check $check $fx5 {YZX_EXPECTED_CLOSURE: $fx5.foundation})
  expect ($r6.exit_code == 0) "expected-closure match: exit 0"
  let j6 = ($r6.stdout | from json)
  expect ($j6.clauses.closure_matches_expected == true) "expected-closure match: clause true"

  # --- 7. migration defaults to dry-run: receipt written, nothing mutated ---
  let fx7 = (make-fixture)
  let newf7 = (make-foundation $fx7.store "dddd-lifeos-foundation-new")
  let rdir7 = ($fx7.root | path join "receipts")
  mkdir $rdir7
  let r7 = (run-migrate $migrate $check $fx7 {} [--closure $newf7 --receipt-dir $rdir7])
  expect ($r7.exit_code == 0) "dry-run migration: exit 0"
  let link7 = (^readlink ($fx7.state | path join "profile") | str trim)
  expect ($link7 == "profile-1-link") "dry-run migration: profile symlink unchanged"
  let rec7 = (read-receipt $rdir7)
  expect ($rec7.schema == "yazelix.single-profile-migration.receipt.v1") "receipt: schema"
  expect ($rec7.mode == "dry-run") "receipt: mode dry-run"
  expect ($rec7.prior_profile_target == "profile-1-link") "receipt: prior symlink target recorded"
  expect ($rec7.prior_profile_resolved == ($fx7.profile_dir | path expand)) "receipt: prior resolved store path recorded"
  expect ($rec7.new_closure_path == $newf7) "receipt: new closure path recorded"
  expect (($rec7.install_commands | length) == 2) "receipt: install commands recorded"
  expect (($rec7.rollback_command | str contains "profile-1-link") and ($rec7.rollback_command | str contains "mv -T")) "receipt: atomic rollback command recorded"
  expect ($rec7.verified == null) "receipt: dry-run leaves verified null"
  expect ($rec7.rollback_performed == false) "receipt: dry-run performed no rollback"

  # --- 8. --execute without --closure is refused ---
  let fx8 = (make-fixture)
  let rdir8 = ($fx8.root | path join "receipts")
  mkdir $rdir8
  let r8 = (run-migrate $migrate $check $fx8 {} [--execute --receipt-dir $rdir8])
  expect ($r8.exit_code != 0) "--execute without --closure: refused"

  # --- 9. dry-run never archives a divergent XDG selector, only plans it ---
  let fx9 = (make-fixture)
  let other9 = ($fx9.store | path join "cccc-second-owner")
  mkdir $other9
  let xdg9 = ($fx9.state | path join "profiles" | path join "profile")
  ^ln -s $other9 $xdg9
  let newf9 = (make-foundation $fx9.store "dddd-lifeos-foundation-new")
  let rdir9 = ($fx9.root | path join "receipts")
  mkdir $rdir9
  let r9 = (run-migrate $migrate $check $fx9 {} [--closure $newf9 --receipt-dir $rdir9])
  expect ($r9.exit_code == 0) "dry-run with divergent XDG: exit 0"
  expect ($xdg9 | path exists) "dry-run with divergent XDG: selector untouched"
  let rec9 = (read-receipt $rdir9)
  expect ($rec9.xdg_profile_state == "divergent") "receipt: divergent XDG state recorded"
  expect ($rec9.xdg_profile_action | str contains "archive") "receipt: XDG archive action planned"

  # --- 10. execute rehearsal (stub nix): atomic cutover + archive + verify ---
  let fx10 = (make-fixture)
  let newf10 = (make-foundation $fx10.store "dddd-lifeos-foundation-new")
  let newprof10 = (make-profile-dir $fx10.store "eeee-profile-next" $newf10)
  # patch the new manifest to point at the new closure
  let mpath10 = ($newprof10 | path join "manifest.json")
  let m10 = (open --raw $mpath10 | from json)
  $m10
  | update elements ($m10.elements | update lifeos_foundation_yzx ($m10.elements.lifeos_foundation_yzx | update storePaths [$newf10]))
  | to json | save --force $mpath10
  ^ln -s $newprof10 ($fx10.state | path join "profile-2-link")
  let other10 = ($fx10.store | path join "cccc-second-owner")
  mkdir $other10
  let xdg10 = ($fx10.state | path join "profiles" | path join "profile")
  ^ln -s $other10 $xdg10
  let stub10 = ($fx10.root | path join "stub-nix")
  let nu_bin10 = (which nu | get path.0)
  [$"#!($nu_bin10)" '^ln -sfn $env.STUB_TARGET $env.STUB_PROFILE' 'exit 0' ''] | str join "\n" | save --force $stub10
  ^chmod +x $stub10
  let rdir10 = ($fx10.root | path join "receipts")
  mkdir $rdir10
  let env10 = {
    YZX_NIX_BIN: $stub10
    STUB_TARGET: "profile-2-link"
    STUB_PROFILE: ($fx10.state | path join "profile")
  }
  let r10 = (run-migrate $migrate $check $fx10 $env10 [--closure $newf10 --receipt-dir $rdir10 --execute])
  expect ($r10.exit_code == 0) "execute rehearsal: exit 0"
  let link10 = (^readlink ($fx10.state | path join "profile") | str trim)
  expect ($link10 == "profile-2-link") "execute rehearsal: profile flipped to new generation"
  expect (not ($xdg10 | path exists)) "execute rehearsal: divergent XDG selector no longer active"
  let archived10 = (ls ($fx10.state | path join "profiles") | where name =~ "profile.archived-" | length)
  expect ($archived10 == 1) "execute rehearsal: divergent XDG selector archived, not deleted"
  let rec10 = (read-receipt $rdir10)
  expect ($rec10.mode == "execute") "execute receipt: mode execute"
  expect ($rec10.verified == true) "execute receipt: verified true"
  expect ($rec10.rollback_performed == false) "execute receipt: no rollback needed"

  # --- 11. execute rehearsal failure: auto-rollback to the prior selector ---
  let fx11 = (make-fixture)
  let newf11 = (make-foundation $fx11.store "dddd-lifeos-foundation-new")
  let broken11 = ($fx11.state | path join "profile-broken")
  mkdir $broken11
  ^ln -s $broken11 ($fx11.state | path join "profile-3-link")
  let stub11 = ($fx11.root | path join "stub-nix")
  let nu_bin11 = (which nu | get path.0)
  [$"#!($nu_bin11)" '^ln -sfn $env.STUB_TARGET $env.STUB_PROFILE' 'exit 0' ''] | str join "\n" | save --force $stub11
  ^chmod +x $stub11
  let rdir11 = ($fx11.root | path join "receipts")
  mkdir $rdir11
  let env11 = {
    YZX_NIX_BIN: $stub11
    STUB_TARGET: "profile-3-link"
    STUB_PROFILE: ($fx11.state | path join "profile")
  }
  let r11 = (run-migrate $migrate $check $fx11 $env11 [--closure $newf11 --receipt-dir $rdir11 --execute])
  expect ($r11.exit_code != 0) "failed cutover: nonzero exit"
  let link11 = (^readlink ($fx11.state | path join "profile") | str trim)
  expect ($link11 == "profile-1-link") "failed cutover: profile atomically rolled back"
  let rec11 = (read-receipt $rdir11)
  expect ($rec11.verified == false) "failed-cutover receipt: verified false"
  expect ($rec11.rollback_performed == true) "failed-cutover receipt: rollback recorded"

  print "ok: all single-profile contract tests passed"
}
