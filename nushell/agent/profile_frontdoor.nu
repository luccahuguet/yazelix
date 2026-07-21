# Profile-owned Codex/Claude launcher.
#
# This file is rendered twice by flake.nix. The installed wrapper is the only
# supported frontdoor; its mutable state is reached through the profile's
# runtime link and never through a home-root compatibility directory.

const AGENT = "@agent@"
const PROFILE_ROOT = "@profileRoot@"
const RUNTIME_TARGET = "@runtimeTarget@"
const PAYLOAD = "@payload@"
const MATERIALIZER = "@materializer@"
const CHMOD = "@chmod@"
const READLINK = "@readlink@"

def fail [message: string] {
    print --stderr $"profile-owned ($AGENT) frontdoor: ($message)"
    exit 1
}

def ensure-runtime [] {
    let profile_runtime = ($PROFILE_ROOT | path join "runtime")
    if (($profile_runtime | path type) != "symlink") {
        fail $"profile runtime link is missing: ($profile_runtime)"
    }

    mkdir $RUNTIME_TARGET
    let resolved = (do { ^$READLINK -f $profile_runtime } | complete)
    if $resolved.exit_code != 0 {
        fail $"profile runtime link cannot be resolved: ($profile_runtime)"
    }
    let actual_target = ($resolved.stdout | str trim)
    let expected_target = (do { ^$READLINK -f $RUNTIME_TARGET } | complete)
    if $expected_target.exit_code != 0 or $actual_target != ($expected_target.stdout | str trim) {
        fail $"profile runtime link does not select ($RUNTIME_TARGET)"
    }

    let state_home = ($profile_runtime | path join $AGENT)
    mkdir $state_home
    for directory in [$RUNTIME_TARGET $state_home] {
        let mode = (do { ^$CHMOD 0700 $directory } | complete)
        if $mode.exit_code != 0 {
            fail $"unable to secure runtime directory: ($directory)"
        }
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
