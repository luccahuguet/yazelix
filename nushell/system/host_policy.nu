#!/usr/bin/env nu

const PROFILE_ROOT = "/home/flexnetos/.nix-profile"
const OWNED_FILES = [
    { source: "nix.conf", target: "/etc/nix/nix.conf" }
    { source: "nix.custom.conf", target: "/etc/nix/nix.custom.conf" }
    { source: "determinate-config.json", target: "/etc/determinate/config.json" }
]
const RETIRED_CLIENT_STATE = [
    "/home/flexnetos/.local/share/nix/trusted-settings.json"
    "/root/.local/share/nix/trusted-settings.json"
]

def policy_root [] {
    $env.YAZELIX_HOST_POLICY_ROOT? | default $"($PROFILE_ROOT)/share/yazelix/host-policy"
}

def source_path [name: string] {
    $"(policy_root)/($name)"
}

def target_path [absolute: string] {
    let root = ($env.YAZELIX_HOST_POLICY_TARGET_ROOT? | default "" | str trim --right --char "/")
    if ($root | is-empty) {
        $absolute
    } else {
        $root | path join ($absolute | str trim --left --char "/")
    }
}

def assert_bundle [] {
    for owned in $OWNED_FILES {
        let source = (source_path $owned.source)
        if not ($source | path exists) {
            error make {msg: $"profile-owned host policy input is missing: ($source)"}
        }
    }
    let determinate = (open $"(policy_root)/determinate-config.json")
    if ($determinate.telemetry.sentry.endpoint? != null) {
        error make {msg: "Determinate Sentry endpoint must be null"}
    }
    let nix_policy = (open --raw $"(policy_root)/nix.conf")
    for required in [
        "substitute = false"
        "substituters ="
        "trusted-substituters ="
        "keep-build-log = false"
        "compress-build-log = false"
    ] {
        if not ($nix_policy | lines | any {|line| ($line | str trim) == $required}) {
            error make {msg: $"Nix host policy is missing: ($required)"}
        }
    }
}

def same_file [source: string, target: string] {
    if not ($target | path exists) {
        return false
    }
    (open --raw $source) == (open --raw $target)
}

def apply_owned_file [source: string, target: string] {
    if (same_file $source $target) {
        return
    }
    let parent = ($target | path dirname)
    mkdir $parent
    let staged = $"($target).yazelix-new"
    if ($staged | path exists) {
        rm --force $staged
    }
    cp $source $staged
    mv --force $staged $target
}

def check_targets [] {
    assert_bundle
    for owned in $OWNED_FILES {
        let source = (source_path $owned.source)
        let target = (target_path $owned.target)
        if not (same_file $source $target) {
            error make {msg: $"host policy drift: ($target)"}
        }
    }
    for retired in $RETIRED_CLIENT_STATE {
        let target = (target_path $retired)
        if ($target | path exists) {
            error make {msg: $"retired Nix cache authority exists: ($target)"}
        }
    }
}

def check_effective [] {
    let nix = $"($PROFILE_ROOT)/bin/nix"
    let config = (^$nix config show --json | from json)
    let violations = [
        {name: "substitute", actual: $config.substitute.value, expected: false}
        {name: "substituters", actual: $config.substituters.value, expected: []}
        {name: "trusted-substituters", actual: $config.trusted-substituters.value, expected: []}
        {name: "keep-build-log", actual: $config.keep-build-log.value, expected: false}
        {name: "compress-build-log", actual: $config.compress-build-log.value, expected: false}
        {name: "always-allow-substitutes", actual: $config.always-allow-substitutes.value, expected: false}
    ] | where {|row| $row.actual != $row.expected}
    if not ($violations | is-empty) {
        error make {msg: $"effective Nix cache policy violations: ($violations | to nuon)"}
    }
}

def apply_nix [] {
    assert_bundle
    for owned in $OWNED_FILES {
        apply_owned_file (source_path $owned.source) (target_path $owned.target)
    }
    for retired in $RETIRED_CLIENT_STATE {
        let target = (target_path $retired)
        if ($target | path exists) {
            rm --recursive --force $target
        }
    }
    check_targets
}

def main [command: string = "check"] {
    match $command {
        "check-bundle" => { assert_bundle }
        "check-files" => { check_targets }
        "apply-nix" => { apply_nix }
        "check" => { check_targets; check_effective }
        _ => { error make {msg: $"unknown host policy command: ($command)"} }
    }
}
