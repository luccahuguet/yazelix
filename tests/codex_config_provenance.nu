# Runtime provenance gate for Codex config and durable rules authorship.
#
# Usage:
#   nu tests/codex_config_provenance.nu <repo-root>
#   nu tests/codex_config_provenance.nu <repo-root> --config-home <staged-profile-runtime>
#
# Every clause is evaluated. The gate exits nonzero if any clause fails.

def main [
    root: path
    --config-home: path = "/run/user/1001/yazelix/profile-runtime/codex"
    --deployed-config-src: path = "/home/flexnetos/.nix-profile/share/yazelix/agent_configs/codex/config.toml.src"
    --deployed-rules-src: path = "/home/flexnetos/.nix-profile/share/yazelix/agent_configs/codex/RULES.md.src"
] {
    let materializer = ($root | path join "nushell/scripts/materialize_codex_config.nu")
    let repo_config_src = ($root | path join "agent_configs/codex/config.toml.src")
    let repo_rules_src = ($root | path join "agent_configs/codex/RULES.md.src")
    let live_config = ($config_home | path join "config.toml")
    let live_rules = ($config_home | path join "RULES.md")
    mut failures = []

    let profile_codex = "/home/flexnetos/.nix-profile/bin/codex"
    let codex_bin = (which codex | get path.0? | default "")
    let selector_real = (if ($codex_bin | is-empty) { "" } else { $codex_bin | path expand --strict })
    let profile_real = (if ($profile_codex | path exists) { $profile_codex | path expand --strict } else { "" })
    if ($codex_bin != $profile_codex) or ($profile_real | is-empty) or ($selector_real | is-empty) or ($selector_real != $profile_real) {
        $failures = ($failures | append "binary-profile-owned")
        print $"clause binary-profile-owned: FAIL lexical=($codex_bin | to nuon) required=($profile_codex | to nuon) resolved=($selector_real | to nuon)"
    } else {
        print $"clause binary-profile-owned: ok lexical=($codex_bin) payload=($selector_real)"
    }

    let required_inputs = [
        {name: "deployed-config", path: $deployed_config_src}
        {name: "deployed-rules", path: $deployed_rules_src}
        {name: "review-config", path: $repo_config_src}
        {name: "review-rules", path: $repo_rules_src}
    ]
    for input in $required_inputs {
        if ($input.path | path exists) {
            print $"clause input-($input.name): ok ($input.path)"
        } else {
            $failures = ($failures | append $"input-($input.name)")
            print $"clause input-($input.name): FAIL missing ($input.path)"
        }
    }

    for pair in [
        {name: "config", deployed: $deployed_config_src, review: $repo_config_src}
        {name: "rules", deployed: $deployed_rules_src, review: $repo_rules_src}
    ] {
        if ($pair.deployed | path exists) and ($pair.review | path exists) {
            let deployed_hash = (open --raw $pair.deployed | hash sha256)
            let repo_hash = (open --raw $pair.review | hash sha256)
            if $deployed_hash == $repo_hash {
                print $"clause input-review-sync-($pair.name): ok sha256=($deployed_hash)"
            } else {
                $failures = ($failures | append $"input-review-sync-($pair.name)")
                print $"clause input-review-sync-($pair.name): FAIL deployed=($deployed_hash) repo=($repo_hash)"
            }
        } else {
            $failures = ($failures | append $"input-review-sync-($pair.name)")
            print $"clause input-review-sync-($pair.name): FAIL missing input"
        }
    }

    let prerequisites = [
        $deployed_config_src
        $deployed_rules_src
        $live_config
        $live_rules
        $materializer
    ]
    if ($prerequisites | all {|value| $value | path exists }) {
        let workdir = (mktemp --directory --tmpdir "codex-provenance-gate.XXXXXX")
        let expected_config = ($workdir | path join "config.toml")
        let expected_rules = ($workdir | path join "RULES.md")
        # Seed the expected output with the live file so the materializer's
        # live-only runtime-table preservation is part of the byte proof.
        cp $live_config $expected_config
        let result = (do { ^$nu.current-exe $materializer $deployed_config_src $expected_config $deployed_rules_src $expected_rules } | complete)
        if $result.exit_code != 0 {
            $failures = ($failures | append "runtime-provenance")
            print $"clause runtime-provenance: FAIL materializer error: ($result.stderr | str trim)"
        } else {
            for pair in [
                {name: "config", expected: $expected_config, live: $live_config}
                {name: "rules", expected: $expected_rules, live: $live_rules}
            ] {
                let expected_hash = (open --raw $pair.expected | hash sha256)
                let live_hash = (open --raw $pair.live | hash sha256)
                if $expected_hash == $live_hash {
                    print $"clause runtime-provenance-($pair.name): ok sha256=($live_hash)"
                } else {
                    $failures = ($failures | append $"runtime-provenance-($pair.name)")
                    print $"clause runtime-provenance-($pair.name): FAIL live=($live_hash) expected=($expected_hash)"
                }
            }
        }
    } else {
        $failures = ($failures | append "runtime-provenance")
        print "clause runtime-provenance: FAIL prerequisites missing"
    }

    if ($live_config | path exists) {
        let raw = (open --raw $live_config)
        let parses = (try { $raw | from toml | ignore; true } catch { false })
        let marked = ($raw | str contains "GENERATED by yazelix codex config materializer")
        let source_hash = (if ($deployed_config_src | path exists) { open --raw $deployed_config_src | hash sha256 } else { "" })
        let hashed = (not ($source_hash | is-empty)) and ($raw | str contains $"# source_sha256 = ($source_hash)")
        let mode = (ls -l $live_config | get mode.0)
        if $parses and $marked and $hashed and ($mode == "rw-r--r--") {
            print "clause config-parseable-generated: ok"
        } else {
            $failures = ($failures | append "config-parseable-generated")
            print $"clause config-parseable-generated: FAIL parses=($parses) marker=($marked) source-hash=($hashed) mode=($mode)"
        }
    } else {
        $failures = ($failures | append "config-parseable-generated")
        print $"clause config-parseable-generated: FAIL missing ($live_config)"
    }

    if ($live_config | path exists) and ($deployed_config_src | path exists) {
        let live_parsed = (try { open --raw $live_config | from toml } catch { {} })
        let source_parsed = (try { open --raw $deployed_config_src | from toml } catch { {} })
        let source_keys = ($source_parsed | columns)
        let authored_matches = ($source_keys | all {|key|
            ($live_parsed | get --optional $key) == ($source_parsed | get $key)
        })
        if $authored_matches {
            print $"clause config-authored-projection: ok keys=($source_keys | str join ',')"
        } else {
            $failures = ($failures | append "config-authored-projection")
            print $"clause config-authored-projection: FAIL keys=($source_keys | str join ',')"
        }

        let live_keys = ($live_parsed | columns)
        let replaced_keys = ($source_keys | where {|key| $key in $live_keys })
        let runtime_projection = if ($replaced_keys | is-empty) {
            $live_parsed
        } else {
            $live_parsed | reject ...$replaced_keys
        }
        let runtime_hash = ($runtime_projection | to toml | hash sha256)
        let runtime_hashed = ((open --raw $live_config) | str contains $"# runtime_projection_sha256 = ($runtime_hash)")
        if $runtime_hashed {
            print $"clause config-runtime-projection: ok sha256=($runtime_hash)"
        } else {
            $failures = ($failures | append "config-runtime-projection")
            print $"clause config-runtime-projection: FAIL sha256=($runtime_hash)"
        }
    }

    if ($live_rules | path exists) {
        let raw = (open --raw $live_rules)
        let marked = ($raw | str contains "GENERATED by yazelix codex rules materializer")
        let headed = ($raw | str contains "# FlexNetOS Codex Durable Rules")
        let source_hash = (if ($deployed_rules_src | path exists) { open --raw $deployed_rules_src | hash sha256 } else { "" })
        let hashed = (not ($source_hash | is-empty)) and ($raw | str contains $"<!-- source_sha256 = ($source_hash) -->")
        let mode = (ls -l $live_rules | get mode.0)
        if $marked and $headed and $hashed and ($mode == "rw-r--r--") {
            print "clause rules-generated-headed: ok"
        } else {
            $failures = ($failures | append "rules-generated-headed")
            print $"clause rules-generated-headed: FAIL marker=($marked) heading=($headed) source-hash=($hashed) mode=($mode)"
        }
    } else {
        $failures = ($failures | append "rules-generated-headed")
        print $"clause rules-generated-headed: FAIL missing ($live_rules)"
    }

    if ($live_config | path exists) {
        let raw = (open --raw $live_config)
        let retired_home_tree = (["." "local"] | str join)
        let forbidden = [
            "/home/flexnetos/FlexNetOS"
            "/nix/store/"
            "/nix/var/nix/profiles/"
            $retired_home_tree
        ]
        let hits = ($forbidden | where {|pattern| $raw | str contains $pattern })
        if ($hits | is-empty) {
            print "clause no-retired-or-inactive-config-paths: ok"
        } else {
            $failures = ($failures | append "no-retired-or-inactive-config-paths")
            print $"clause no-retired-or-inactive-config-paths: FAIL ($hits | str join ', ')"
        }
    }

    if ($live_config | path exists) {
        let parsed = (try { open --raw $live_config | from toml } catch { {} })
        let icm_command = ($parsed.mcp_servers?.icm?.command? | default "")
        if $icm_command == "/home/flexnetos/.nix-profile/bin/icm" {
            print $"clause icm-profile-selector: ok ($icm_command)"
        } else {
            $failures = ($failures | append "icm-profile-selector")
            print $"clause icm-profile-selector: FAIL ($icm_command | to nuon)"
        }
    }

    if ($failures | is-empty) {
        print $"ok codex provenance gate: ($live_config) and ($live_rules) are Yazelix-authored"
    } else {
        print --stderr $"codex provenance gate: FAIL clauses: ($failures | uniq | str join ', ')"
        exit 1
    }
}
