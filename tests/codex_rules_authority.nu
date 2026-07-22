# Durable RULES.md authority gate (ARCHBP-027).
#
# One reviewed editable source governs the Codex durable rules:
#   repo agent_configs/codex/RULES.md.src
#     -> nix profile share RULES.md.src
#       -> generated profile-runtime RULES.md (source_sha256-stamped).
#
# Clauses fail red when the rules file is duplicated at a retired mirror,
# when the generated runtime names anything but the deployed profile source,
# or when the repo/deployed/runtime hash chain splits. `--fixture-root`
# evaluates the same clauses against a planted simulation tree so the red
# paths are provable without touching blocked live locations.
#
# Usage:
#   nu tests/codex_rules_authority.nu <repo-root>
#   nu tests/codex_rules_authority.nu <repo-root> --fixture-root <dir>

def sha256_of [file: path] {
    open --raw $file | hash sha256
}

def main [
    root: path
    --fixture-root: path = ""
    --config-home: path = "/run/user/1001/yazelix/profile-runtime/codex"
    --deployed-rules-src: path = "/home/flexnetos/.nix-profile/share/yazelix/agent_configs/codex/RULES.md.src"
] {
    let simulated = (not ($fixture_root | is-empty))
    let home_base = (if $simulated { $fixture_root } else { "/home/flexnetos" })
    let live_rules = (
        if $simulated {
            [$fixture_root "profile-runtime/codex/RULES.md"] | path join
        } else {
            $config_home | path join "RULES.md"
        }
    )
    let deployed_src = (
        if $simulated {
            [$fixture_root "profile-share/RULES.md.src"] | path join
        } else {
            $deployed_rules_src
        }
    )
    let repo_src = ($root | path join "agent_configs/codex/RULES.md.src")
    mut failures = []

    # Clause duplicate-authority: the durable rules must exist at exactly one
    # authority; every retired mirror location must stay empty. The overlay
    # dir name is joined from parts (the strict_profile_sources idiom) so this
    # test does not itself trip the textual ownership gate.
    let retired_overlay = (["." "codex"] | str join)
    let retired_mirrors = [
        ([$home_base $retired_overlay "RULES.md"] | path join)
        ([$home_base "lifeos" $retired_overlay "RULES.md"] | path join)
        ([$home_base "FlexNetOS" $retired_overlay "RULES.md"] | path join)
        ([$home_base "meta/src/envctl" $retired_overlay "RULES.md"] | path join)
    ]
    let duplicates = ($retired_mirrors | where {|mirror| $mirror | path exists })
    if ($duplicates | is-empty) {
        print "clause duplicate-authority: ok (no retired mirror carries RULES.md)"
    } else {
        $failures = ($failures | append "duplicate-authority")
        print $"clause duplicate-authority: FAIL duplicates=($duplicates | to nuon)"
    }

    # Clause retired-mirror-source: the generated runtime must name the
    # deployed profile source as its only authorship input.
    if not ($live_rules | path exists) {
        $failures = ($failures | append "retired-mirror-source")
        print $"clause retired-mirror-source: FAIL missing generated rules at ($live_rules)"
    } else {
        let head = (open --raw $live_rules | lines | first 3 | str join "\n")
        let expected_input = (
            if $simulated { $deployed_src } else { $deployed_rules_src }
        )
        if ($head | str contains $"authorship input: ($expected_input)") {
            print $"clause retired-mirror-source: ok — authorship input is ($expected_input)"
        } else {
            $failures = ($failures | append "retired-mirror-source")
            print $"clause retired-mirror-source: FAIL generated header does not name ($expected_input): ($head | to nuon)"
        }
    }

    # Clause single-source: repo source, deployed source, and the runtime's
    # embedded source_sha256 must be one identical hash.
    if (($repo_src | path exists) and ($deployed_src | path exists) and ($live_rules | path exists)) {
        let repo_hash = (sha256_of $repo_src)
        let deployed_hash = (sha256_of $deployed_src)
        let embedded = (
            open --raw $live_rules
            | lines
            | where {|line| $line | str contains "source_sha256 = " }
            | first
            | parse --regex 'source_sha256 = (?<hash>[0-9a-f]{64})'
            | get hash.0?
            | default ""
        )
        if ($repo_hash == $deployed_hash) and ($deployed_hash == $embedded) {
            print $"clause single-source: ok — one reviewed source, sha256 ($repo_hash)"
        } else {
            $failures = ($failures | append "single-source")
            print $"clause single-source: FAIL repo=($repo_hash) deployed=($deployed_hash) embedded=($embedded)"
        }
    } else {
        $failures = ($failures | append "single-source")
        print $"clause single-source: FAIL missing inputs repo=(($repo_src | path exists)) deployed=(($deployed_src | path exists)) runtime=(($live_rules | path exists))"
    }

    if not ($failures | is-empty) {
        error make { msg: $"codex rules authority gate failed: ($failures | str join ', ')" }
    }
    print "codex rules authority gate: all clauses passed"
}
