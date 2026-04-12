#!/usr/bin/env nu
# Test lane: maintainer

def write_executable [path: string, body: string] {
    $body | save --force --raw $path
    ^chmod +x $path
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
# Defends: yzx gc surfaces Nix-store progress and completion feedback without reviving removed GC ownership.
def test_yzx_gc_surfaces_phase_feedback [] {
    print "🧪 Testing yzx gc surfaces Nix-store start/finish feedback..."

    let tmpdir = (^mktemp -d /tmp/yazelix_gc_feedback_XXXXXX | str trim)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        mkdir $fake_bin
        let nu_bin = (which nu | get 0.path)
        ^ln -s $nu_bin ($fake_bin | path join "nu")

        write_executable ($fake_bin | path join "du") '#!/bin/sh
printf "1048576\t/nix/store\n"
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
            and ($stdout | str contains "Collecting garbage... this can take a while")
            and ($stdout | str contains "Re-measuring Nix store size...")
            and ($stdout | str contains "done in")
            and ($stdout | str contains "garbage cleaned")
            and ($stdout | str contains "Nix Store")
            and ($stdout | str contains "Collecting")
        ) {
            print "  ✅ yzx gc reports progress and completion for the Nix-store phase"
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
            and ($stdout | str contains "garbage cleaned")
            and ($stdout | str contains "Done")
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
# Defends: yzx gc fails loudly and stops before re-measuring when nix-collect-garbage fails.
def test_yzx_gc_fails_loudly_when_nix_gc_fails [] {
    print "🧪 Testing yzx gc fails loudly when nix-collect-garbage fails..."

    let tmpdir = (^mktemp -d /tmp/yazelix_gc_failure_XXXXXX | str trim)

    let result = (try {
        let fake_bin = ($tmpdir | path join "bin")
        mkdir $fake_bin
        let nu_bin = (which nu | get 0.path)
        ^ln -s $nu_bin ($fake_bin | path join "nu")

        write_executable ($fake_bin | path join "du") '#!/bin/sh
printf "1048576\t/nix/store\n"
'
        write_executable ($fake_bin | path join "nix-collect-garbage") '#!/bin/sh
echo "nix gc exploded" >&2
exit 7
'

        let gc_script = ("nushell/scripts/yzx/gc.nu" | path expand)
        let output = with-env { PATH: ([$fake_bin] | append $env.PATH) } {
            ^nu -c $"source \"($gc_script)\"; yzx gc" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code != 0)
            and ($stdout | str contains "Collecting garbage... this can take a while")
            and ($stdout | str contains "Failed after")
            and ($stdout | str contains "nix gc exploded")
            and not ($stdout | str contains "Re-measuring Nix store size")
            and ($stdout | str contains "Done")
        ) {
            print "  ✅ yzx gc surfaces nix-collect-garbage failures instead of looking silent"
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
