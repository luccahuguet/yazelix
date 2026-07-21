# Profile-owned Codex/Claude launcher.
#
# This file is rendered twice by flake.nix. The installed wrapper is the only
# supported frontdoor; it owns mutable state at the approved profile-owned state
# home — volatile under the runtime root for Codex, durable under Meta for
# Claude — and never through a home-root compatibility directory. The state home
# is passed in whole (not derived from a shared parent) so the frontdoor secures
# exactly that directory and never reaches up to chmod a shared data root.

const AGENT = "@agent@"
const STATE_HOME = "@stateHome@"
const PAYLOAD = "@payload@"
const MATERIALIZER = "@materializer@"
const CHMOD = "@chmod@"

def fail [message: string] {
    print --stderr $"profile-owned ($AGENT) frontdoor: ($message)"
    exit 1
}

def ensure-runtime [] {
    let state_home = $STATE_HOME
    mkdir $state_home
    let mode = (do { ^$CHMOD 0700 $state_home } | complete)
    if $mode.exit_code != 0 {
        fail $"unable to secure runtime directory: ($state_home)"
    }
    $state_home
}

def reject-competing-owner [name: string, expected: path] {
    let inherited = ($env | get --optional $name | default "")
    if ($inherited | is-not-empty) and $inherited != ($expected | into string) {
        fail $"($name) selects a competing owner: ($inherited)"
    }
}

def materialize-reviewed-config [] {
    if ($MATERIALIZER | is-empty) {
        fail "reviewed configuration materializer is missing"
    }
    let materialized = (do { ^$MATERIALIZER } | complete)
    if $materialized.exit_code != 0 {
        print --stderr ($materialized.stderr | str trim)
        fail "reviewed configuration could not be materialized"
    }
}

def --wrapped main [...args] {
    let state_home = (ensure-runtime)
    match $AGENT {
        "codex" => {
            reject-competing-owner "CODEX_HOME" $state_home
            $env.CODEX_HOME = ($state_home | into string)
            materialize-reviewed-config
        }
        "claude" => {
            reject-competing-owner "CLAUDE_CONFIG_DIR" $state_home
            $env.CLAUDE_CONFIG_DIR = ($state_home | into string)
            materialize-reviewed-config
        }
        _ => { fail $"unsupported agent identity: ($AGENT)" }
    }
    exec $PAYLOAD ...$args
}
