# Runtime provenance gate for Codex config authorship (YZXCONV-004)
#
# Verifies that the config.toml inside a CODEX_HOME is exactly the
# materialized output of the reviewed Yazelix-owned editable input, that the
# deploy-surface input matches the repo review copy, and that the generated
# config carries no retired-workspace or profile-bypassing paths.
#
# Usage:
#   nu tests/codex_config_provenance.nu <repo-root>
#   nu tests/codex_config_provenance.nu <repo-root> --config-home <staged-codex-home>
#
# Against the live home (/home/flexnetos/.codex) this stays RED until the
# mission RESET cutover materializes config.toml from the reviewed input with
# a checksum-backed backup. Pointing --config-home at a staged copy proves the
# gate goes green post-cutover.
#
# Every clause is evaluated and reported; the gate exits nonzero if any fails.

def main [
    root: path
    --config-home: path = "/home/flexnetos/.codex"
    --deployed-src: path = "/home/flexnetos/.config/yazelix/agents/codex/config.toml.src"
] {
    let materializer = ($root | path join "nushell/scripts/materialize_codex_config.nu")
    let repo_src = ($root | path join "agent_configs/codex/config.toml.src")
    let live = ($config_home | path join "config.toml")
    mut failures = []

    # clause binary-profile-owned: the codex on PATH is the profile's payload.
    # (Sole-selector enforcement — PATH resolving through ~/.nix-profile itself —
    # is YZXCONV-003 scope; recorded here as info only.)
    let profile_codex = "/home/flexnetos/.nix-profile/bin/codex"
    let codex_bin = (which codex | get path.0? | default "")
    let selector_real = (if ($codex_bin | is-empty) { "" } else { $codex_bin | path expand })
    let profile_real = (if ($profile_codex | path exists) { $profile_codex | path expand } else { "" })
    if ($profile_real | is-empty) or ($selector_real | is-empty) or ($selector_real != $profile_real) {
        $failures = ($failures | append "binary-profile-owned")
        print $"clause binary-profile-owned: FAIL selector=($codex_bin | to nuon) profile=($profile_codex | to nuon)"
    } else {
        print $"clause binary-profile-owned: ok payload=($selector_real)"
        if not ($codex_bin | str starts-with "/home/flexnetos/.nix-profile/") {
            print $"info: PATH selector is ($codex_bin), not the profile symlink — YZXCONV-003 sole-selector scope"
        }
    }

    # clause input-deployed: the deploy-surface editable input exists
    if ($deployed_src | path exists) {
        print $"clause input-deployed: ok ($deployed_src)"
    } else {
        $failures = ($failures | append "input-deployed")
        print $"clause input-deployed: FAIL missing ($deployed_src)"
    }

    # clause input-review-sync: deploy surface matches the repo review copy
    if ($deployed_src | path exists) and ($repo_src | path exists) {
        let deployed_hash = (open --raw $deployed_src | hash sha256)
        let repo_hash = (open --raw $repo_src | hash sha256)
        if $deployed_hash == $repo_hash {
            print $"clause input-review-sync: ok sha256=($deployed_hash)"
        } else {
            $failures = ($failures | append "input-review-sync")
            print $"clause input-review-sync: FAIL deployed=($deployed_hash) repo=($repo_hash)"
        }
    } else {
        $failures = ($failures | append "input-review-sync")
        print $"clause input-review-sync: FAIL missing input \(deployed=($deployed_src | path exists) repo=($repo_src | path exists)\)"
    }

    # clause config-provenance: generated config checksum-matches the
    # materialized output of the deployed input
    if ($deployed_src | path exists) and ($live | path exists) and ($materializer | path exists) {
        let workdir = (mktemp --directory --tmpdir "codex-provenance-gate.XXXXXX")
        let expected_out = ($workdir | path join "config.toml")
        let result = (do { ^$nu.current-exe $materializer $deployed_src $expected_out } | complete)
        if $result.exit_code != 0 {
            $failures = ($failures | append "config-provenance")
            print $"clause config-provenance: FAIL materializer error: ($result.stderr | str trim)"
        } else {
            let expected_hash = (open --raw $expected_out | hash sha256)
            let live_hash = (open --raw $live | hash sha256)
            if $expected_hash == $live_hash {
                print $"clause config-provenance: ok sha256=($live_hash)"
            } else {
                $failures = ($failures | append "config-provenance")
                print $"clause config-provenance: FAIL live=($live_hash) expected=($expected_hash)"
            }
        }
    } else {
        $failures = ($failures | append "config-provenance")
        print $"clause config-provenance: FAIL prerequisites missing \(input=($deployed_src | path exists) config=($live | path exists) materializer=($materializer | path exists)\)"
    }

    # clause config-parseable-generated: generated config parses as TOML and
    # carries the generated marker
    if ($live | path exists) {
        let live_raw = (open --raw $live)
        let parses = (try { $live_raw | from toml | ignore; true } catch { false })
        let marked = ($live_raw | str contains "GENERATED by yazelix codex config materializer")
        if $parses and $marked {
            print "clause config-parseable-generated: ok"
        } else {
            $failures = ($failures | append "config-parseable-generated")
            print $"clause config-parseable-generated: FAIL parses=($parses) generated-marker=($marked)"
        }
    } else {
        $failures = ($failures | append "config-parseable-generated")
        print $"clause config-parseable-generated: FAIL config missing ($live)"
    }

    # clause no-retired-or-inactive-paths: generated config references no
    # retired workspace mirror, raw store pin, or profile-generation pin
    if ($live | path exists) {
        let live_raw = (open --raw $live)
        let forbidden = [
            "/home/flexnetos/FlexNetOS"
            "/nix/store/"
            "/nix/var/nix/profiles/"
            ".local/state/nix/profiles/profile-"
        ]
        let hits = ($forbidden | where {|pattern| $live_raw | str contains $pattern })
        if ($hits | is-empty) {
            print "clause no-retired-or-inactive-paths: ok"
        } else {
            $failures = ($failures | append "no-retired-or-inactive-paths")
            print $"clause no-retired-or-inactive-paths: FAIL ($hits | str join ', ')"
        }
    } else {
        $failures = ($failures | append "no-retired-or-inactive-paths")
        print $"clause no-retired-or-inactive-paths: FAIL config missing ($live)"
    }

    if ($failures | is-empty) {
        print $"ok codex provenance gate: ($live) is authored by ($deployed_src)"
    } else {
        print --stderr $"codex provenance gate: FAIL clauses: ($failures | str join ', ')"
        exit 1
    }
}
