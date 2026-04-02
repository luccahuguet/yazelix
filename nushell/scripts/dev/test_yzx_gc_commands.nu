#!/usr/bin/env nu
# Test lane: maintainer

def write_executable [path: string, body: string] {
    $body | save --force --raw $path
    ^chmod +x $path
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
# Defends: yzx gc surfaces phase-by-phase progress and completion feedback.
def test_yzx_gc_surfaces_phase_feedback [] {
    print "🧪 Testing yzx gc surfaces start/finish feedback for each phase..."

    let tmpdir = (^mktemp -d /tmp/yazelix_gc_feedback_XXXXXX | str trim)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        mkdir $fake_bin
        let nu_bin = (which nu | get 0.path)
        ^ln -s $nu_bin ($fake_bin | path join "nu")

        write_executable ($fake_bin | path join "du") '#!/bin/sh
printf "1048576\t/nix/store\n"
'
        write_executable ($fake_bin | path join "devenv") '#!/bin/sh
if [ "$1" = "gc" ]; then
  echo "devenv cleaned"
  exit 0
fi
echo "unexpected devenv args: $*" >&2
exit 1
'
        write_executable ($fake_bin | path join "nix-collect-garbage") '#!/bin/sh
echo "garbage cleaned"
exit 0
'

        let gc_script = ("nushell/scripts/yzx/gc.nu" | path expand)
        let output = with-env { PATH: ([$fake_bin] | append $env.PATH) } {
            ^nu -c $"source \"($gc_script)\"; yzx gc" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Measuring current Nix store size...")
            and ($stdout | str contains "Cleaning devenv generations... this can take a while")
            and ($stdout | str contains "Collecting garbage... this can take a while")
            and ($stdout | str contains "Re-measuring Nix store size...")
            and ($stdout | str contains "Done in")
            and ($stdout | str contains "devenv cleaned")
            and ($stdout | str contains "garbage cleaned")
            and ($stdout | str contains "Nix Store")
        ) {
            print "  ✅ yzx gc reports progress and completion for both phases"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Strength: defect=1 behavior=2 resilience=1 cost=1 uniqueness=1 total=6/10
# Regression: yzx gc still uses a valid du total even when du reports transient missing-path errors.
def test_yzx_gc_accepts_du_totals_even_with_transient_errors [] {
    print "🧪 Testing yzx gc accepts a valid du total even when du exits nonzero..."

    let tmpdir = (^mktemp -d /tmp/yazelix_gc_du_partial_XXXXXX | str trim)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        mkdir $fake_bin
        let nu_bin = (which nu | get 0.path)
        ^ln -s $nu_bin ($fake_bin | path join "nu")

        write_executable ($fake_bin | path join "du") '#!/bin/sh
echo "du: cannot access '\''/nix/store/missing-path'\'': No such file or directory" >&2
printf "148547536542\t/nix/store\n"
exit 1
'

        write_executable ($fake_bin | path join "devenv") '#!/bin/sh
if [ "$1" = "gc" ]; then
  echo "devenv cleaned"
  exit 0
fi
echo "unexpected devenv args: $*" >&2
exit 1
'
        write_executable ($fake_bin | path join "nix-collect-garbage") '#!/bin/sh
echo "garbage cleaned"
exit 0
'

        let gc_script = ("nushell/scripts/yzx/gc.nu" | path expand)
        let output = with-env { PATH: ([$fake_bin] | append $env.PATH) } {
            ^nu -c $"source \"($gc_script)\"; yzx gc" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Current size:")
            and not ($stdout | str contains "Current size: 0 B")
            and ($stdout | str contains "devenv cleaned")
            and ($stdout | str contains "garbage cleaned")
        ) {
            print "  ✅ yzx gc uses the reported du total even when du emits transient missing-path errors"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
# Defends: yzx gc fails loudly and stops early when devenv gc fails.
def test_yzx_gc_fails_loudly_when_devenv_gc_fails [] {
    print "🧪 Testing yzx gc fails loudly when devenv gc fails..."

    let tmpdir = (^mktemp -d /tmp/yazelix_gc_failure_XXXXXX | str trim)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        mkdir $fake_bin
        let nu_bin = (which nu | get 0.path)
        ^ln -s $nu_bin ($fake_bin | path join "nu")

        write_executable ($fake_bin | path join "du") '#!/bin/sh
printf "1048576\t/nix/store\n"
'
        write_executable ($fake_bin | path join "devenv") '#!/bin/sh
echo "devenv exploded" >&2
exit 7
'
        write_executable ($fake_bin | path join "nix-collect-garbage") '#!/bin/sh
echo "should not run"
exit 0
'

        let gc_script = ("nushell/scripts/yzx/gc.nu" | path expand)
        let output = with-env { PATH: ([$fake_bin] | append $env.PATH) } {
            ^nu -c $"source \"($gc_script)\"; yzx gc" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code != 0)
            and ($stdout | str contains "Cleaning devenv generations... this can take a while")
            and ($stdout | str contains "Failed after")
            and ($stdout | str contains "devenv exploded")
            and not ($stdout | str contains "Collecting garbage... this can take a while")
        ) {
            print "  ✅ yzx gc surfaces devenv gc failures instead of looking silent"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

export def run_gc_tests [] {
    [
        (test_yzx_gc_accepts_du_totals_even_with_transient_errors)
        (test_yzx_gc_surfaces_phase_feedback)
        (test_yzx_gc_fails_loudly_when_devenv_gc_fails)
    ]
}
