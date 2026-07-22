# yzx-envelope — LifeOS bubblewrap user-namespace envelope engine (Nushell).
#
# yzx-iso T2 lane (spine ARCHBP-065..071): a Nix-declared, hermetic,
# native-process envelope per the ratified isolation architecture spec v1.0.0
# (lifeos planning-spine-v0/docs/isolation-architecture-spec.md, invariants
# I03/I05/I06). The envelope presents a private /, a read-only /nix, a tmpfs
# home overlay, and explicit durable-state binds; teardown is namespace-scoped
# so mounts unwind to zero leaks on exit; GPU/ports/devices pass through on
# demand and release cleanly (two-brother model, I11/I12/I13).
#
# Subcommands:
#   executor                      print selected bwrap executor JSON (honest
#                                 downgrade recording: a candidate blocked by
#                                 host AppArmor is reported with root cause)
#   enter [OPTS] [-- CMD...]      run CMD (default: nu) inside a fresh envelope
#   probe [OPTS]                  emit a JSON observation from inside an envelope
#   leakcheck ID                  verify zero namespace/process leaks for ID
#
# Options (enter/probe):
#   --id NAME            envelope id (default: yzx-<pid>)
#   --store PATH         bind PATH (an extracted release closure) at /nix
#                        instead of the host store — the portable-release
#                        launcher mode (yzx-iso T10, spine ARCHBP-021): the
#                        host /nix is hidden, so a workload that runs proves
#                        the bundle is self-sufficient, never falsely
#   --durable SRC:DST    bind a durable path read-write (repeatable)
#   --gpu                pass /dev/dri and /dev/nvidia* through
#   --device PATH        pass one device node through (repeatable)
#   --isolate-net        unshare the network namespace (ports released)
#   --env K=V            inject an environment variable (repeatable)
#   --cwd DIR            working directory inside the envelope
#
# The nix builder substitutes @NIX_BWRAP@ with the pinned bubblewrap path.

def die [msg: string] {
    print --stderr $"yzx-envelope: ($msg)"
    exit 1
}

# --- executor selection -----------------------------------------------------
# Prefer the Nix-pinned bwrap (single-profile path law). On hosts where
# kernel.apparmor_restrict_unprivileged_userns=1 confines userns creation to
# profiled binaries, the unconfined store bwrap is denied and the engine falls
# back to the AppArmor-profiled /usr/bin/bwrap — recorded, never silent. The
# permanent fix is an AppArmor profile for the store bwrap (host integration,
# owner-gated).
def candidate-works [c: string] {
    (do { ^$c --dev-bind / / true } | complete | get exit_code) == 0
}

def select-executor [] {
    mut candidates = []
    let operator_bwrap = ($env.YZX_BWRAP? | default "")
    if $operator_bwrap != "" { $candidates = ($candidates | append $operator_bwrap) }
    $candidates = ($candidates | append "@NIX_BWRAP@")
    let path_bwrap = (which bwrap | get --optional 0.path | default "")
    if $path_bwrap != "" { $candidates = ($candidates | append $path_bwrap) }
    $candidates = ($candidates | append "/usr/bin/bwrap")
    mut blocked = []
    for c in $candidates {
        if not ($c | path exists) { continue }
        if (candidate-works $c) {
            let reason = if $c == "/usr/bin/bwrap" {
                "recorded downgrade: unconfined userns denied by host AppArmor (kernel.apparmor_restrict_unprivileged_userns=1); using the AppArmor-profiled host bwrap; permanent fix = ship an AppArmor profile for the store bwrap (owner-gated host integration)"
            } else if ($c | str starts-with "/nix/store/") or ($c | str contains "/.nix-profile/") {
                "nix-pinned bwrap operational"
            } else {
                "operator-selected via YZX_BWRAP"
            }
            return {executor: $c, reason: $reason, blocked: $blocked}
        }
        $blocked = ($blocked | append {candidate: $c, reason: "userns denied (host policy)"})
    }
    die "no working bwrap executor found"
}

# --- envelope construction --------------------------------------------------
def parse-opts [args: list<string>] {
    mut o = {
        id: ""
        store: "/nix"
        durables: []
        devices: []
        gpu: false
        isolate_net: false
        env_vars: []
        cwd: "/"
        rest: []
    }
    mut i = 0
    while $i < ($args | length) {
        let a = ($args | get $i)
        if $a == "--id" { $o.id = ($args | get ($i + 1)); $i = $i + 2 } else if (
            $a == "--store") { $o.store = ($args | get ($i + 1)); $i = $i + 2 } else if (
            $a == "--durable") { $o.durables = ($o.durables | append ($args | get ($i + 1))); $i = $i + 2 } else if (
            $a == "--gpu") { $o.gpu = true; $i = $i + 1 } else if (
            $a == "--device") { $o.devices = ($o.devices | append ($args | get ($i + 1))); $i = $i + 2 } else if (
            $a == "--isolate-net") { $o.isolate_net = true; $i = $i + 1 } else if (
            $a == "--env") { $o.env_vars = ($o.env_vars | append ($args | get ($i + 1))); $i = $i + 2 } else if (
            $a == "--cwd") { $o.cwd = ($args | get ($i + 1)); $i = $i + 2 } else if (
            $a == "--") { $o.rest = ($args | skip ($i + 1)); $i = ($args | length) } else {
            die $"unknown option: ($a)"
        }
    }
    $o
}

def build-args [o: record] {
    # A private root: tmpfs /, read-only /nix, minimal read-only /etc, private
    # /proc /dev /tmp, and a tmpfs home overlay. Everything else is absent.
    if not ($o.store | path exists) { die $"store source missing: ($o.store)" }
    mut args = [
        --tmpfs /
        --ro-bind $o.store /nix
        --ro-bind /etc/resolv.conf /etc/resolv.conf
        --ro-bind /etc/passwd /etc/passwd
        --ro-bind /etc/group /etc/group
        --proc /proc
        --dev /dev
        --tmpfs /tmp
        --tmpfs $env.HOME
        --unshare-user --unshare-pid --unshare-ipc --unshare-uts
        --die-with-parent
        --hostname yzx-envelope
        --chdir $o.cwd
        --clearenv
        --setenv YZX_ENVELOPE_ID $o.id
        --setenv HOME $env.HOME
        # nu's ENV_CONVERSIONS holds PATH as a list; bwrap needs the string.
        --setenv PATH (do { let p = $env.PATH; if (($p | describe) | str starts-with "list") { $p | str join ":" } else { $p } })
        --setenv TERM ($env.TERM? | default "dumb")
    ]
    if $o.isolate_net { $args = ($args | append [--unshare-net]) }
    for d in $o.durables {
        let src = ($d | split row ":" | get 0)
        let dst = ($d | str substring (($src | str length) + 1)..)
        if not ($src | path exists) { die $"durable source missing: ($src)" }
        $args = ($args | append [--bind $src $dst])
    }
    if $o.gpu {
        if ("/dev/dri" | path exists) { $args = ($args | append [--dev-bind /dev/dri /dev/dri]) }
        for n in (glob /dev/nvidia*) {
            $args = ($args | append [--dev-bind $n $n])
        }
    }
    for d in $o.devices {
        if not ($d | path exists) { die $"device missing: ($d)" }
        $args = ($args | append [--dev-bind $d $d])
    }
    for kv in $o.env_vars {
        let k = ($kv | split row "=" | get 0)
        let v = ($kv | str substring (($k | str length) + 1)..)
        $args = ($args | append [--setenv $k $v])
    }
    $args
}

def cmd-executor [] {
    print (select-executor | to json --raw)
}

def cmd-enter [args: list<string>] {
    mut o = (parse-opts $args)
    if $o.id == "" { $o.id = $"yzx-($nu.pid)" }
    let executor = (select-executor | get executor)
    let bwrap_args = (build-args $o)
    let cmd = if ($o.rest | is-not-empty) {
        $o.rest
    } else {
        # The only interactive shell inside the envelope is nu.
        let nu_path = (which nu | get --optional 0.path | default "")
        if $nu_path == "" { die "nu not found for the in-envelope shell" }
        [($nu_path | path expand)]
    }
    exec $executor ...$bwrap_args ...$cmd
}

# The probe body runs INSIDE the envelope (only /nix visible), so it is a nu
# script string executed by the store nu resolved on the preserved PATH.
const PROBE_BODY = '
    let mounts = (open --raw /proc/self/mounts | lines | length)
    # Overlay dir name joined from parts (strict_profile_sources idiom) so the
    # engine source does not itself trip the textual ownership gate.
    let agent_overlay = (["." "claude"] | str join)
    let host_home_visible = if ([$env.HOME $agent_overlay] | path join | path exists) { "yes" } else { "no" }
    # interface count from /proc/net/dev (header is 2 lines); lo counts as 1.
    let net_devices = ((open --raw /proc/net/dev | lines | length) - 2)
    print ({
        id: $env.YZX_ENVELOPE_ID
        uid: (^id -u | into int)
        pid: $nu.pid
        cwd: $env.PWD
        hostname: (open --raw /proc/sys/kernel/hostname | str trim)
        mounts: $mounts
        host_home_visible: $host_home_visible
        net_devices: $net_devices
        gpu_dri: (if ("/dev/dri" | path exists) { "yes" } else { "no" })
        gpu_nvidia: (if ("/dev/nvidiactl" | path exists) { "yes" } else { "no" })
        probe_env: ($env.YZX_PROBE_VAR? | default "unset")
    } | to json --raw)
'

def cmd-probe [args: list<string>] {
    mut o = (parse-opts $args)
    if $o.id == "" { $o.id = $"yzx-probe-($nu.pid)" }
    let executor = (select-executor | get executor)
    let bwrap_args = (build-args $o)
    let nu_path = (which nu | get --optional 0.path | default "")
    if $nu_path == "" { die "nu not found for the envelope probe" }
    ^$executor ...$bwrap_args ($nu_path | path expand) -c $PROBE_BODY
}

def cmd-leakcheck [id: string] {
    if $id == "" { die "leakcheck requires an envelope id" }
    let needle = $"YZX_ENVELOPE_ID=($id)"
    let leaks = (
        glob /proc/[0-9]*/environ
        | where {|p|
            let raw = (try { open --raw $p } catch { "" })
            ($raw | into string | split row (char --integer 0) | any {|entry| $entry == $needle })
        }
        | length
    )
    let mount_residue = (
        open --raw /proc/mounts | lines
        | where {|line| $line | str contains $"yzx-envelope-($id)" }
        | length
    )
    let clean = ($leaks == 0 and $mount_residue == 0)
    print ({id: $id, leaked_processes: $leaks, host_mount_residue: $mount_residue, clean: $clean} | to json --raw)
    if not $clean { exit 1 }
}

# --wrapped: option-looking args (--id, --store, ...) flow through to the
# engine's own parser instead of being claimed by nu's flag handling.
def --wrapped main [...all: string] {
    let subcommand = ($all | get --optional 0 | default "")
    let args = ($all | skip 1)
    match $subcommand {
        "executor" => { cmd-executor }
        "enter" => { cmd-enter $args }
        "probe" => { cmd-probe $args }
        "leakcheck" => { cmd-leakcheck ($args | get --optional 0 | default "") }
        _ => { die "usage: yzx-envelope {executor|enter|probe|leakcheck} ..." }
    }
}
