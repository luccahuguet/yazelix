#!/usr/bin/env bash
# yzx-envelope — LifeOS bubblewrap user-namespace envelope engine.
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
set -euo pipefail

die() {
  echo "yzx-envelope: $*" >&2
  exit 1
}

json_escape() {
  local s=$1
  s=${s//\\/\\\\}
  s=${s//\"/\\\"}
  printf '%s' "$s"
}

# --- executor selection -----------------------------------------------------
# Prefer the Nix-pinned bwrap (single-profile path law). On hosts where
# kernel.apparmor_restrict_unprivileged_userns=1 confines userns creation to
# profiled binaries, the unconfined store bwrap is denied and the engine falls
# back to the AppArmor-profiled /usr/bin/bwrap — recorded, never silent. The
# permanent fix is an AppArmor profile for the store bwrap (host integration,
# owner-gated).
candidate_works() {
  "$1" --dev-bind / / true >/dev/null 2>&1
}

select_executor() {
  EXECUTOR=""
  EXECUTOR_REASON=""
  BLOCKED_JSON=""
  local candidates=()
  [ -n "${YZX_BWRAP:-}" ] && candidates+=("$YZX_BWRAP")
  local nix_bwrap
  nix_bwrap=$(command -v bwrap 2>/dev/null || true)
  [ -n "$nix_bwrap" ] && candidates+=("$nix_bwrap")
  candidates+=("/usr/bin/bwrap")
  local c
  for c in "${candidates[@]}"; do
    [ -x "$c" ] || continue
    if candidate_works "$c"; then
      EXECUTOR="$c"
      case "$c" in
        /nix/store/*|*/.nix-profile/*)
          EXECUTOR_REASON="nix-pinned bwrap operational" ;;
        /usr/bin/bwrap)
          EXECUTOR_REASON="recorded downgrade: unconfined userns denied by host AppArmor (kernel.apparmor_restrict_unprivileged_userns=1); using the AppArmor-profiled host bwrap; permanent fix = ship an AppArmor profile for the store bwrap (owner-gated host integration)" ;;
        *)
          EXECUTOR_REASON="operator-selected via YZX_BWRAP" ;;
      esac
      return 0
    fi
    BLOCKED_JSON="${BLOCKED_JSON:+$BLOCKED_JSON,}{\"candidate\":\"$(json_escape "$c")\",\"reason\":\"userns denied (host policy)\"}"
  done
  die "no working bwrap executor found"
}

cmd_executor() {
  select_executor
  printf '{"executor":"%s","reason":"%s","blocked":[%s]}\n' \
    "$(json_escape "$EXECUTOR")" "$(json_escape "$EXECUTOR_REASON")" "$BLOCKED_JSON"
}

# --- envelope construction --------------------------------------------------
ENV_ID=""
STORE_SRC="/nix"
DURABLES=()
DEVICES=()
GPU=0
ISOLATE_NET=0
ENV_VARS=()
CWD="/"

parse_opts() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --id) ENV_ID=$2; shift 2 ;;
      --store) STORE_SRC=$2; shift 2 ;;
      --durable) DURABLES+=("$2"); shift 2 ;;
      --gpu) GPU=1; shift ;;
      --device) DEVICES+=("$2"); shift 2 ;;
      --isolate-net) ISOLATE_NET=1; shift ;;
      --env) ENV_VARS+=("$2"); shift 2 ;;
      --cwd) CWD=$2; shift 2 ;;
      --) shift; REST=("$@"); return 0 ;;
      *) die "unknown option: $1" ;;
    esac
  done
  REST=()
}

build_args() {
  # A private root: tmpfs /, read-only /nix, minimal read-only /etc, private
  # /proc /dev /tmp, and a tmpfs home overlay. Everything else is absent.
  [ -d "$STORE_SRC" ] || die "store source missing: $STORE_SRC"
  BWRAP_ARGS=(
    --tmpfs /
    --ro-bind "$STORE_SRC" /nix
    --ro-bind /etc/resolv.conf /etc/resolv.conf
    --ro-bind /etc/passwd /etc/passwd
    --ro-bind /etc/group /etc/group
    --proc /proc
    --dev /dev
    --tmpfs /tmp
    --tmpfs "$HOME"
    --unshare-user --unshare-pid --unshare-ipc --unshare-uts
    --die-with-parent
    --hostname yzx-envelope
    --chdir "$CWD"
    --clearenv
    --setenv YZX_ENVELOPE_ID "$ENV_ID"
    --setenv HOME "$HOME"
    --setenv PATH "$PATH"
    --setenv TERM "${TERM:-dumb}"
  )
  [ "$ISOLATE_NET" = 1 ] && BWRAP_ARGS+=(--unshare-net)
  local d src dst
  for d in "${DURABLES[@]}"; do
    src=${d%%:*}
    dst=${d#*:}
    [ -e "$src" ] || die "durable source missing: $src"
    BWRAP_ARGS+=(--bind "$src" "$dst")
  done
  if [ "$GPU" = 1 ]; then
    [ -d /dev/dri ] && BWRAP_ARGS+=(--dev-bind /dev/dri /dev/dri)
    local n
    for n in /dev/nvidia*; do
      [ -e "$n" ] && BWRAP_ARGS+=(--dev-bind "$n" "$n")
    done
  fi
  for d in "${DEVICES[@]}"; do
    [ -e "$d" ] || die "device missing: $d"
    BWRAP_ARGS+=(--dev-bind "$d" "$d")
  done
  local kv
  for kv in ${ENV_VARS[@]+"${ENV_VARS[@]}"}; do
    BWRAP_ARGS+=(--setenv "${kv%%=*}" "${kv#*=}")
  done
}

cmd_enter() {
  parse_opts "$@"
  [ -n "$ENV_ID" ] || ENV_ID="yzx-$$"
  select_executor
  build_args
  local cmd=()
  if [ ${#REST[@]} -gt 0 ]; then
    cmd=("${REST[@]}")
  else
    # The only interactive shell inside the envelope is nu.
    local nu_path
    nu_path=$(command -v nu) || die "nu not found for the in-envelope shell"
    nu_path=$(readlink -f "$nu_path")
    cmd=("$nu_path")
  fi
  exec "$EXECUTOR" "${BWRAP_ARGS[@]}" "${cmd[@]}"
}

cmd_probe() {
  parse_opts "$@"
  [ -n "$ENV_ID" ] || ENV_ID="yzx-probe-$$"
  select_executor
  build_args
  local sh_path
  sh_path=$(readlink -f "$(command -v bash)")
  # shellcheck disable=SC2016  # the probe body must expand inside the envelope
  "$EXECUTOR" "${BWRAP_ARGS[@]}" "$sh_path" -c '
    set -eu
    mounts=$(wc -l < /proc/self/mounts)
    host_home_visible=no
    [ -e "$HOME/.claude" ] && host_home_visible=yes
    # interface count from /proc/net/dev (header is 2 lines); lo counts as 1.
    net_dev_count=$(($(wc -l < /proc/net/dev) - 2))
    dri=no; [ -d /dev/dri ] && dri=yes
    nvidia=no; [ -e /dev/nvidiactl ] && nvidia=yes
    printf "{\"id\":\"%s\",\"uid\":%s,\"pid\":%s,\"cwd\":\"%s\",\"hostname\":\"%s\",\"mounts\":%s,\"host_home_visible\":\"%s\",\"net_devices\":%s,\"gpu_dri\":\"%s\",\"gpu_nvidia\":\"%s\",\"probe_env\":\"%s\"}\n" \
      "$YZX_ENVELOPE_ID" "$(id -u)" "$$" "$(pwd)" \
      "$(cat /proc/sys/kernel/hostname)" \
      "$mounts" "$host_home_visible" "$net_dev_count" "$dri" "$nvidia" \
      "${YZX_PROBE_VAR:-unset}"
  '
}

cmd_leakcheck() {
  local id=${1:-}
  [ -n "$id" ] || die "leakcheck requires an envelope id"
  local leaks=0 p
  for p in /proc/[0-9]*/environ; do
    if { tr '\0' '\n' < "$p" | grep -qx "YZX_ENVELOPE_ID=$id"; } 2>/dev/null; then
      leaks=$((leaks + 1))
    fi
  done
  local mount_residue
  mount_residue=$(grep -c "yzx-envelope-$id" /proc/mounts 2>/dev/null || true)
  [ -z "$mount_residue" ] && mount_residue=0
  printf '{"id":"%s","leaked_processes":%s,"host_mount_residue":%s,"clean":%s}\n' \
    "$(json_escape "$id")" "$leaks" "$mount_residue" \
    "$([ "$leaks" = 0 ] && [ "$mount_residue" = 0 ] && echo true || echo false)"
  [ "$leaks" = 0 ] && [ "$mount_residue" = 0 ]
}

case "${1:-}" in
  executor) shift; cmd_executor "$@" ;;
  enter) shift; cmd_enter "$@" ;;
  probe) shift; cmd_probe "$@" ;;
  leakcheck) shift; cmd_leakcheck "$@" ;;
  *) die "usage: yzx-envelope {executor|enter|probe|leakcheck} ..." ;;
esac
