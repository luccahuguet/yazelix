# Contract check for the Codex config and rules materializer (YZXCONV-004)
#
# Proves:
#   1. both reviewed inputs exist and satisfy their format contracts
#   2. source-only materialization is deterministic for config.toml and RULES.md
#   3. both outputs carry exact source hashes, generated markers, and mode 0644
#   4. every input and both staged outputs validate before either live file changes
#   5. forbidden config paths, invalid TOML, malformed rules, and missing inputs
#      fail closed
#   6. reviewed top-level tables replace stale live tables while live-only runtime
#      tables survive, and a second-publication failure restores the prior pair

def fail [message: string] {
    print --stderr $"codex materializer contract: ($message)"
    exit 1
}

def invoke-materializer [materializer: path, config_src: path, config_out: path, rules_src: path, rules_out: path] {
    do { ^$nu.current-exe $materializer $config_src $config_out $rules_src $rules_out } | complete
}

def expect-config-reject [materializer: path, workdir: path, rules_src: path, label: string, content: string] {
    let config_src = ($workdir | path join $"reject-config-($label).toml.src")
    $content | save --force --raw $config_src
    let config_out = ($workdir | path join $"reject-config-($label).toml")
    let rules_out = ($workdir | path join $"reject-config-($label).md")
    let result = (invoke-materializer $materializer $config_src $config_out $rules_src $rules_out)
    if $result.exit_code == 0 {
        fail $"materializer accepted forbidden config input: ($label)"
    }
    if ($config_out | path exists) or ($rules_out | path exists) {
        fail $"materializer wrote output despite rejecting config input: ($label)"
    }
}

def expect-rules-reject [materializer: path, workdir: path, config_src: path, label: string, content: string] {
    let rules_src = ($workdir | path join $"reject-rules-($label).md.src")
    $content | save --force --raw $rules_src
    let config_out = ($workdir | path join $"reject-rules-($label).toml")
    let rules_out = ($workdir | path join $"reject-rules-($label).md")
    let result = (invoke-materializer $materializer $config_src $config_out $rules_src $rules_out)
    if $result.exit_code == 0 {
        fail $"materializer accepted malformed rules input: ($label)"
    }
    if ($config_out | path exists) or ($rules_out | path exists) {
        fail $"materializer wrote output despite rejecting rules input: ($label)"
    }
}

def main [root: path] {
    let materializer = ($root | path join "nushell/scripts/materialize_codex_config.nu")
    let config_src = ($root | path join "agent_configs/codex/config.toml.src")
    let rules_src = ($root | path join "agent_configs/codex/RULES.md.src")
    for required in [$materializer $config_src $rules_src] {
        if not ($required | path exists) {
            fail $"required source missing: ($required)"
        }
    }

    let config_raw = (open --raw $config_src)
    try { $config_raw | from toml | ignore } catch {
        fail "reviewed config input is not valid TOML"
    }
    let forbidden = [
        "/home/flexnetos/FlexNetOS"
        "/nix/store/"
        "/nix/var/nix/profiles/"
        ".local/state/nix/profiles/profile-"
        ".local/bin"
    ]
    for pattern in $forbidden {
        if ($config_raw | str contains $pattern) {
            fail $"reviewed config input contains forbidden path ($pattern)"
        }
    }

    let rules_raw = (open --raw $rules_src)
    if not ($rules_raw | str starts-with "# FlexNetOS Codex Durable Rules\n") {
        fail "reviewed rules input lacks the durable rules heading"
    }

    let workdir = (mktemp --directory --tmpdir "codex-materializer-check.XXXXXX")
    let config_out_a = ($workdir | path join "a" "config.toml")
    let config_out_b = ($workdir | path join "b" "config.toml")
    let rules_out_a = ($workdir | path join "a" "RULES.md")
    let rules_out_b = ($workdir | path join "b" "RULES.md")
    for pair in [
        {config: $config_out_a, rules: $rules_out_a}
        {config: $config_out_b, rules: $rules_out_b}
    ] {
        let result = (invoke-materializer $materializer $config_src $pair.config $rules_src $pair.rules)
        if $result.exit_code != 0 {
            fail $"materializer failed on reviewed inputs: ($result.stderr)"
        }
    }

    let config_hash_a = (open --raw $config_out_a | hash sha256)
    let config_hash_b = (open --raw $config_out_b | hash sha256)
    let rules_hash_a = (open --raw $rules_out_a | hash sha256)
    let rules_hash_b = (open --raw $rules_out_b | hash sha256)
    if $config_hash_a != $config_hash_b {
        fail $"non-deterministic config output: ($config_hash_a) vs ($config_hash_b)"
    }
    if $rules_hash_a != $rules_hash_b {
        fail $"non-deterministic rules output: ($rules_hash_a) vs ($rules_hash_b)"
    }

    let rendered_config = (open --raw $config_out_a)
    try { $rendered_config | from toml | ignore } catch {
        fail "materialized config output is not valid TOML"
    }
    if not ($rendered_config | str contains "GENERATED by yazelix codex config materializer") {
        fail "materialized config output lacks its generated marker"
    }
    let source_config_hash = ($config_raw | hash sha256)
    if not ($rendered_config | str contains $"# source_sha256 = ($source_config_hash)") {
        fail "materialized config output lacks its exact source hash"
    }
    if not ($rendered_config | str contains "# runtime_projection_sha256 = ") {
        fail "materialized config output lacks its runtime projection hash"
    }
    let rendered_rules = (open --raw $rules_out_a)
    if not ($rendered_rules | str contains "GENERATED by yazelix codex rules materializer") {
        fail "materialized rules output lacks its generated marker"
    }
    if not ($rendered_rules | str contains "# FlexNetOS Codex Durable Rules") {
        fail "materialized rules output lacks its durable rules body"
    }
    let source_rules_hash = ($rules_raw | hash sha256)
    if not ($rendered_rules | str contains $"<!-- source_sha256 = ($source_rules_hash) -->") {
        fail "materialized rules output lacks its exact source hash"
    }
    for output in [$config_out_a $rules_out_a $config_out_b $rules_out_b] {
        let mode = (ls -l $output | get mode.0)
        if $mode != "rw-r--r--" {
            fail $"materialized output mode is not 0644: ($output) mode=($mode)"
        }
    }

    expect-config-reject $materializer $workdir $rules_src "retired-workspace" ('[projects."/home/flexnetos/FlexNetOS"]' + "\ntrust_level = \"trusted\"\n")
    expect-config-reject $materializer $workdir $rules_src "raw-store-pin" ("[mcp_servers.icm]\ncommand = \"/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-icm-0.0.1/bin/icm\"\n")
    expect-config-reject $materializer $workdir $rules_src "system-profile-pin" ("[mcp_servers.icm]\ncommand = \"/nix/var/nix/profiles/default/bin/icm\"\n")
    expect-config-reject $materializer $workdir $rules_src "user-profile-generation-pin" ("[mcp_servers.icm]\ncommand = \"/home/flexnetos/.local/state/nix/profiles/profile-4-link/bin/icm\"\n")
    expect-config-reject $materializer $workdir $rules_src "user-local-shadow" ("[mcp_servers.icm]\ncommand = \"/home/flexnetos/.local/bin/icm\"\n")
    expect-config-reject $materializer $workdir $rules_src "tilde-user-local-shadow" ("[mcp_servers.icm]\ncommand = \"~/.local/bin/icm\"\n")
    expect-config-reject $materializer $workdir $rules_src "home-variable-user-local-shadow" ("[mcp_servers.icm]\ncommand = \"$HOME/.local/bin/icm\"\n")
    expect-config-reject $materializer $workdir $rules_src "bare-icm-shadow" ("[mcp_servers.icm]\ncommand = \"icm\"\n")
    expect-config-reject $materializer $workdir $rules_src "system-icm-shadow" ("[mcp_servers.icm]\ncommand = \"/usr/bin/icm\"\n")
    expect-config-reject $materializer $workdir $rules_src "invalid-toml" "[unterminated\n"
    expect-rules-reject $materializer $workdir $config_src "missing-heading" "# Different Rules\n"
    expect-rules-reject $materializer $workdir $config_src "generated-input" "# FlexNetOS Codex Durable Rules\nGENERATED by yazelix codex rules materializer\n"

    let missing_config = (invoke-materializer $materializer ($workdir | path join "missing.toml.src") ($workdir | path join "never-config.toml") $rules_src ($workdir | path join "never-rules.md"))
    if $missing_config.exit_code == 0 {
        fail "materializer accepted a missing config input"
    }
    let missing_rules = (invoke-materializer $materializer $config_src ($workdir | path join "never-config-2.toml") ($workdir | path join "missing-rules.md.src") ($workdir | path join "never-rules-2.md"))
    if $missing_rules.exit_code == 0 {
        fail "materializer accepted a missing rules input"
    }

    let sentinel_config = ($workdir | path join "sentinel" "config.toml")
    let sentinel_rules = ($workdir | path join "sentinel" "RULES.md")
    mkdir ($sentinel_config | path dirname)
    "preserve-config" | save --raw $sentinel_config
    "preserve-rules" | save --raw $sentinel_rules
    let invalid_rules = ($workdir | path join "invalid-rules.md.src")
    "# Wrong heading\n" | save --raw $invalid_rules
    let failed_update = (invoke-materializer $materializer $config_src $sentinel_config $invalid_rules $sentinel_rules)
    if $failed_update.exit_code == 0 {
        fail "materializer accepted invalid rules during an update"
    }
    if (open --raw $sentinel_config) != "preserve-config" or (open --raw $sentinel_rules) != "preserve-rules" {
        fail "materializer changed an existing output before all inputs validated"
    }

    let split_config = ($workdir | path join "split-a" "config.toml")
    let split_rules = ($workdir | path join "split-b" "RULES.md")
    let split_result = (invoke-materializer $materializer $config_src $split_config $rules_src $split_rules)
    if $split_result.exit_code == 0 {
        fail "materializer accepted outputs in different directories"
    }
    if ($split_config | path exists) or ($split_rules | path exists) {
        fail "materializer changed output while rejecting a split-directory transaction"
    }

    let merge_dir = ($workdir | path join "merge-existing")
    mkdir $merge_dir
    let merge_config = ($merge_dir | path join "config.toml")
    let merge_rules = ($merge_dir | path join "RULES.md")
    [
        'approvals_reviewer = "runtime-stale"'
        ''
        '[runtime_only]'
        'token = "keep"'
        ''
        '[projects."/home/flexnetos/FlexNetOS"]'
        'trust_level = "trusted"'
        ''
        '[hooks.state."fixture"]'
        'trusted_hash = "sha256:keep"'
        ''
    ] | str join "\n" | save --raw $merge_config
    "prior rules\n" | save --raw $merge_rules
    let merge_result = (invoke-materializer $materializer $config_src $merge_config $rules_src $merge_rules)
    if $merge_result.exit_code != 0 {
        fail $"materializer failed while preserving runtime-only config: ($merge_result.stderr)"
    }
    let merged = (open --raw $merge_config | from toml)
    if $merged.approvals_reviewer != "user" {
        fail "reviewed root preference did not replace stale runtime value"
    }
    if $merged.runtime_only.token != "keep" or $merged.hooks.state.fixture.trusted_hash != "sha256:keep" {
        fail "live-only runtime tables were not preserved"
    }
    if ($merged.projects | columns | any {|name| $name == "/home/flexnetos/FlexNetOS" }) {
        fail "reviewed projects table did not replace the retired live table"
    }

    let rollback_dir = ($workdir | path join "rollback-pair")
    mkdir $rollback_dir
    let rollback_config = ($rollback_dir | path join "config.toml")
    let rollback_rules = ($rollback_dir | path join "RULES.md")
    [
        'approvals_reviewer = "prior"'
        ''
        '[runtime_only]'
        'token = "prior"'
        ''
    ] | str join "\n" | save --raw $rollback_config
    "prior rules\n" | save --raw $rollback_rules
    let prior_config_hash = (open --raw $rollback_config | hash sha256)
    let prior_rules_hash = (open --raw $rollback_rules | hash sha256)
    let interrupted_result = (with-env {YAZELIX_TEST_CRASH_AFTER_CONFIG_REPLACE: "1"} {
        invoke-materializer $materializer $config_src $rollback_config $rules_src $rollback_rules
    })
    if $interrupted_result.exit_code != 86 {
        fail $"injected publication interruption returned unexpected status: ($interrupted_result.exit_code)"
    }
    let journal = ($rollback_dir | path join ".yazelix-codex-transaction.json")
    if not ($journal | path exists) {
        fail "interrupted publication did not retain its durable recovery journal"
    }
    let recovery_result = (do {
        ^$nu.current-exe $materializer $config_src $rollback_config $rules_src $rollback_rules --recover-only
    } | complete)
    if $recovery_result.exit_code != 0 {
        fail $"durable recovery failed: ($recovery_result.stderr)"
    }
    if (open --raw $rollback_config | hash sha256) != $prior_config_hash or (open --raw $rollback_rules | hash sha256) != $prior_rules_hash {
        fail "interruption recovery did not restore the exact prior pair"
    }
    let leftovers = (ls -a $rollback_dir | where {|entry| $entry.name | str contains ".yazelix-" })
    if not ($leftovers | is-empty) {
        fail "interruption recovery left a journal, stage, or backup file"
    }

    print "ok codex materializer: deterministic source, runtime preservation, interruption recovery, provenance, fail-closed"
}
