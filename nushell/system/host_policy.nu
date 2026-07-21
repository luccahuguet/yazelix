#!/usr/bin/env nu

const PROFILE_ROOT = "/home/flexnetos/.nix-profile"
const OWNED_FILES = [
    { source: "nix.conf", target: "/etc/nix/nix.conf" }
    { source: "nix.custom.conf", target: "/etc/nix/nix.custom.conf" }
    { source: "determinate-config.json", target: "/etc/determinate/config.json" }
    { source: "shells", target: "/etc/shells" }
    { source: "nix-daemon.service", target: "/etc/systemd/system/nix-daemon.service" }
    { source: "nix-daemon.socket", target: "/etc/systemd/system/nix-daemon.socket" }
    { source: "chrome-storage.json", target: "/etc/opt/chrome/policies/managed/yazelix-storage.json" }
]
const LOG_POLICY_FILES = [
    { source: "journald-no-storage.conf", target: "/etc/systemd/journald.conf.d/10-yazelix-no-persistent.conf" }
    { source: "docker-daemon.json", target: "/etc/docker/daemon.json" }
]
const DISABLED_LOG_UNITS = [
    "rsyslog.service"
    "sysstat.service"
    "sysstat-summary.timer"
    "sysstat-collect.timer"
    "logrotate.timer"
]
const LOG_ROOTS = [
    "/var/log"
    "/nix/var/log/nix"
    "/var/crash"
    "/run/log/journal"
]
def retired-client-state [] {
    let retired_home_tree = (["." "local"] | str join)
    [
        $"/home/flexnetos/($retired_home_tree)/share/nix/trusted-settings.json"
        $"/root/($retired_home_tree)/share/nix/trusted-settings.json"
        "/usr/local/bin/determinate-nixd"
        "/nix/var/determinate"
        "/etc/systemd/system/nix-daemon.service.d/10-yazelix-host-policy.conf"
    ]
}

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

def live_target [] {
    ($env.YAZELIX_HOST_POLICY_TARGET_ROOT? | default "" | str trim | is-empty)
}

def current_login_shell [] {
    let account = (open --raw /etc/passwd | lines | where {|line| $line | str starts-with "flexnetos:"} | first)
    $account | split row ":" | get 6
}

def ensure_login_shell [] {
    if not (live_target) {
        return
    }
    let profile_nu = $"($PROFILE_ROOT)/toolbin/nu"
    if (current_login_shell) != $profile_nu {
        ^$"($PROFILE_ROOT)/bin/usermod" --shell $profile_nu flexnetos
    }
}

def assert_bundle [] {
    for owned in ($OWNED_FILES | append $LOG_POLICY_FILES) {
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

def check_log_targets [] {
    assert_bundle
    for owned in $LOG_POLICY_FILES {
        let source = (source_path $owned.source)
        let target = (target_path $owned.target)
        if not (same_file $source $target) {
            error make {msg: $"host logging policy drift: ($target)"}
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
    for retired in (retired-client-state) {
        let target = (target_path $retired)
        if ($target | path exists) {
            error make {msg: $"retired Nix cache authority exists: ($target)"}
        }
    }
    if (live_target) and ((current_login_shell) != $"($PROFILE_ROOT)/toolbin/nu") {
        error make {msg: "flexnetos login shell is not profile-owned Nushell"}
    }
    if (live_target) {
        let systemctl = $"($PROFILE_ROOT)/bin/systemctl"
        let determinate = (^$systemctl is-enabled determinate-nixd.socket | complete)
        if ($determinate.stdout | str trim) != "masked" {
            error make {msg: "Determinate Nix cache injector socket is not masked"}
        }
        let daemon_unit = (^$systemctl cat nix-daemon.service | complete)
        if not ($daemon_unit.stdout | str contains "ExecStart=@/home/flexnetos/.nix-profile/bin/nix-daemon nix-daemon --daemon") {
            error make {msg: "Nix daemon is not profile-owned"}
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
    if (live_target) {
        let systemctl = $"($PROFILE_ROOT)/bin/systemctl"
        ^$systemctl disable --now determinate-nixd.socket
        ^$systemctl stop nix-daemon.socket
        ^$systemctl stop nix-daemon.service
    }
    for owned in $OWNED_FILES {
        apply_owned_file (source_path $owned.source) (target_path $owned.target)
    }
    for retired in (retired-client-state) {
        let target = (target_path $retired)
        if ($target | path exists) {
            rm --recursive --force $target
        }
    }
    if (live_target) {
        let systemctl = $"($PROFILE_ROOT)/bin/systemctl"
        for path in [
            "/etc/systemd/system/determinate-nixd.socket"
            "/etc/systemd/system/sockets.target.wants/determinate-nixd.socket"
        ] {
            if (($path | path type) != "") {
                rm --force $path
            }
        }
        ^$systemctl daemon-reload
        ^$systemctl mask determinate-nixd.socket
    }
    ensure_login_shell
    check_targets
    if (live_target) {
        let systemctl = $"($PROFILE_ROOT)/bin/systemctl"
        ^$systemctl enable nix-daemon.service nix-daemon.socket
        ^$systemctl reset-failed nix-daemon.service nix-daemon.socket
        ^$systemctl restart nix-daemon.socket
    }
}

def purge_logs [] {
    for root in $LOG_ROOTS {
        let target = (target_path $root)
        if ($target | path exists) {
            for entry in (ls -a $target) {
                rm --recursive --force $entry.name
            }
        } else {
            mkdir $target
        }
    }
    if (live_target) {
        for log in (glob "/var/lib/docker/containers/**/*-json.log") {
            rm --force $log
        }
    }
}

def check_logs [] {
    check_log_targets
    if not (live_target) {
        return
    }
    let systemctl = $"($PROFILE_ROOT)/bin/systemctl"
    for unit in $DISABLED_LOG_UNITS {
        let enabled = (^$systemctl is-enabled $unit | complete)
        if ($enabled.stdout | str trim) != "masked" {
            error make {msg: $"persistent log writer is not masked: ($unit)"}
        }
        let active = (^$systemctl is-active $unit | complete)
        if ($active.stdout | str trim) == "active" {
            error make {msg: $"persistent log writer is active: ($unit)"}
        }
    }
    for root in $LOG_ROOTS {
        if ($root | path exists) and ((ls -a $root) | is-not-empty) {
            error make {msg: $"log root is not empty: ($root)"}
        }
    }
    let docker_logs = (glob "/var/lib/docker/containers/**/*-json.log")
    if ($docker_logs | is-not-empty) {
        error make {msg: $"Docker JSON logs survive: ($docker_logs | to nuon)"}
    }
}

def apply_logs [] {
    assert_bundle
    for owned in $LOG_POLICY_FILES {
        apply_owned_file (source_path $owned.source) (target_path $owned.target)
    }
    if (live_target) {
        let systemctl = $"($PROFILE_ROOT)/bin/systemctl"
        for unit in $DISABLED_LOG_UNITS {
            let result = (^$systemctl mask --now $unit | complete)
            if $result.exit_code != 0 {
                error make {msg: $"failed to mask persistent log writer ($unit): ($result.stderr | str trim)"}
            }
        }
        ^$systemctl restart systemd-journald.service
        ^$systemctl restart docker.service
    }
    purge_logs
    check_logs
}

def main [command: string = "check"] {
    match $command {
        "check-bundle" => { assert_bundle }
        "check-files" => { check_targets }
        "check-log-files" => { check_log_targets }
        "apply-nix" => { apply_nix }
        "apply-logs" => { apply_logs }
        "purge-logs" => { purge_logs; check_logs }
        "check-logs" => { check_logs }
        "check" => { check_targets; check_effective }
        _ => { error make {msg: $"unknown host policy command: ($command)"} }
    }
}
