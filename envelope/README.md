# yzx-envelope — LifeOS user-namespace envelope

The Nix-declared bubblewrap envelope for the yzx-iso T2 lane (spine
ARCHBP-065..071), conforming to the ratified isolation architecture spec
v1.0.0 (`lifeos planning-spine-v0/docs/isolation-architecture-spec.md`).

## Build & run

```sh
nix build .#yzx-envelope          # hermetic; pinned bubblewrap input
nix run .#yzx-envelope -- executor
```

## Subcommands

| Command | Purpose |
|---|---|
| `executor` | Print the selected bwrap executor as JSON. On hosts with `kernel.apparmor_restrict_unprivileged_userns=1` the unconfined store bwrap is denied; the engine records the downgrade to the AppArmor-profiled `/usr/bin/bwrap` with root cause and the owner-gated permanent fix (an AppArmor profile for the store bwrap). Never silent. |
| `enter [opts] [-- CMD…]` | Run CMD (default: `nu`, the only in-envelope shell) inside a fresh envelope: private tmpfs `/`, read-only `/nix`, minimal read-only `/etc`, private `proc`/`dev`/`tmp`, tmpfs home overlay, user/pid/ipc/uts unshare, `--die-with-parent`, `--clearenv` plus declared env. |
| `probe [opts]` | Emit a JSON observation from inside an envelope (uid, pid, cwd, mount count, home-overlay opacity, net interface count, GPU visibility, injected env). |
| `leakcheck ID` | Prove zero leaked processes and zero host mount residue for envelope ID after exit. |

## Options (enter/probe)

`--id NAME` · `--durable SRC:DST` (repeatable, read-write durable bind) ·
`--gpu` (`/dev/dri` + `/dev/nvidia*`) · `--device PATH` (repeatable) ·
`--isolate-net` (unshare net = release ports) · `--env K=V` (repeatable) ·
`--cwd DIR`

## Invariant conformance (T1 ledger)

- **I03** — native processes in a user-namespace envelope; no hypervisor or
  container daemon in the hot path (probe: private PID ns, `pid=2`).
- **I05/I06** — durable state enters only via explicit `--durable` binds to
  LifeOS-owned durable paths; nothing durable is created on host `/run`.
- **I11/I12** — GPU/ports/devices pass through only on demand and release
  cleanly (net isolation restores port ownership; teardown is
  namespace-scoped with `leakcheck`-proven zero leaks).
- **I13** — the host is never modified: no host installs, no `/etc` writes,
  host shell untouched.

Multiple envelopes coexist on one shared kernel with no cross-session
`/tmp` leakage and a safe shared durable plane (flock-safe appends).
