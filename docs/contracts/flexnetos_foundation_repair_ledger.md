# FlexNetOS Foundation Repair Ledger

This ledger captures the live Yazelix foundation repair state from 2026-07-07.
It is intentionally about installed-runtime ownership, not raw source-tree
success. Generated runtime under `~/.local/share/yazelix` is proof only; edit
inputs under `~/.config/yazelix` or package inputs under this repository.

## Current Installed State

| Surface | Proof |
| --- | --- |
| Active frontdoor | `/home/flexnetos/.nix-profile/bin/yzx` |
| Nix profile element | `nix profile list --json` shows active `lifeos_foundation_yzx` from `path:/home/flexnetos/FlexNetOS/src/yazelix`. |
| Profile target | `/home/flexnetos/.nix-profile/bin/yzx` resolves to `/nix/store/20p5djw21m3lji2sr2chvdyd36ngmj4m-lifeos-foundation-yzx/bin/yzx`. |
| Runtime identity | clean-env `yzx status` reports runtime dir `/nix/store/20p5djw21m3lji2sr2chvdyd36ngmj4m-lifeos-foundation-yzx`; `yzx --version-full` reports `v17.9` and `yazelix_yazi_assets` revision `471073d54d4a6c9fa9e87f26134d6db3f387977e`. |
| Current shell caveat | Existing long-lived shells may still have old Yazelix store paths in inherited `PATH`; clean profile probes resolve `yzx`, `codex`, `claude`, `rtk`, and `git-kb` through `.nix-profile`. |
| Generated runtime | `yzx status` reports `Status up to date`, `Repair needed no`. |
| Health gate | `/home/flexnetos/.nix-profile/bin/yzx doctor` passes. |
| Desktop entry | `com.yazelix.Yazelix.Mars.desktop` uses `Exec="/home/flexnetos/.nix-profile/bin/yzx" desktop launch`. |
| Agent desktop entry | `com.flexnetos.Yazelix.Agent.desktop` uses `YAZELIX_LAYOUT_OVERRIDE="/home/flexnetos/.nix-profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl"` with profile `yzx`. |
| Claude URL handler | `claude-code-url-handler.desktop` uses `Exec="/home/flexnetos/.nix-profile/bin/claude" --handle-uri %u`; clean profile `claude --version` reports `2.1.202 (Claude Code)`. |
| Stale local wrapper | `~/.local/bin` contains only `archive`; no `~/.local/bin/yzx` shadow. |
| Menu/status visual capture | PTY `yzx config ui` capture reached the `status_bar` tab and showed `zellij.widget_tray` current `[9 items]` with workspace, Claude, Codex, CPU, and RAM available; generated layout and direct widget probes cover the repaired cells. A live desktop/Zellij visual pass remains the human acceptance check. |
| Source target | `.#lifeos_foundation_yzx` builds `/nix/store/xxmpnb3w6k12qwsyv7wdc7qmx1g3sb3f-lifeos-foundation-yzx`; `.#yazelix_flexnetos_foundation` is no longer exposed in the current worktree after the profile migration. |

## Repair Ledger

| component | desired_owner | current_owner | source_repo_or_package | nix_attr | exported_bin | runtime_probe | yzx_menu_cell | current_state | required_action | proof |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| yzx frontdoor | Nix profile package | Nix profile package | `src/yazelix` | `lifeos_foundation_yzx` | `yzx` | `yzx status` | n/a | Runtime healthy; profile attr migrated | Keep profile path as active frontdoor; do not restore user-local wrapper | `nix profile list --json` shows active `lifeos_foundation_yzx`; `readlink -f /home/flexnetos/.nix-profile/bin/yzx` -> `/nix/store/20p5djw21m3lji2sr2chvdyd36ngmj4m-lifeos-foundation-yzx/bin/yzx` |
| foundation name | LifeOS-owned `lifeos-foundation-yzx` target | Source and profile use LifeOS package target | `src/yazelix` | `lifeos_foundation_yzx` | n/a | `nix build .#lifeos_foundation_yzx` | n/a | Migrated | Keep the old FlexNetOS foundation attr out unless a future compatibility owner requires it | `packaging/flake_outputs.nix` defines `lifeos_foundation_yzx` with `name = "lifeos-foundation-yzx"` |
| RTK | Upstream RTK 0.43.0 | Upstream RTK release package in foundation | `github:rtk-ai/rtk/v0.43.0` via `packaging/rtk_release.nix` | foundation extra runtime package | `rtk` | `rtk --version` | agent wrapper | Repaired | Do not route foundation through `src/rtk-tokenkill` unless a real FlexNetOS delta is required | `/home/flexnetos/.nix-profile/bin/rtk --version` -> `rtk 0.43.0` |
| rtk-tokenkill fork | None unless delta exists | Separate dirty-history fork, not active foundation owner | `src/rtk-tokenkill` | none | none | `git rev-list upstream_tmp/v0.43.0...HEAD` | n/a | Not in sync; not needed for current foundation | Either archive/remove from foundation scope or rebase/sync only if a required delta is proven | fork comparison showed `3 49` divergence |
| Codex | Yazelix profile package plus RTK wrapper policy | Profile toolbin | foundation package | `lifeos_foundation_yzx` | `codex` | `yazelix_zellij_bar_widget codex ...` | `codex_usage` | Working | Keep `codex` wrapped by `rtk codex` in generated shell/session policy | clean `yzx run` resolves `/home/flexnetos/.nix-profile/bin/codex` before runtime-local tool paths |
| Claude | Yazelix profile package | Profile toolbin | `packaging/claude_code_release.nix` overriding `claude-code` to 2.1.202 | `lifeos_foundation_yzx` | `claude` | `yazelix_zellij_bar_widget claude ...` | `claude_usage` | Working; upgraded to latest known release | Keep unfree allow predicate scoped to `claude-code` and keep URL handler on profile wrapper | clean profile `claude --version` -> `2.1.202 (Claude Code)`; direct widget probe returned `claude 5h|16.4M|97% wk|16.4M|98%` |
| Codex/Claude runtime PATH | Nix profile frontdoors first | Installed profile runtime orders profile bins before runtime toolbin/bin and removes stale Yazelix store PATH entries | `rust_core/yazelix_core/src/runtime_env.rs` | `lifeos_foundation_yzx` | `yzx`, `codex`, `claude` | `yzx run command -v ...` | n/a | Repaired for fresh profile-launched sessions | Keep `.nix-profile/bin` first and treat inherited older Yazelix store paths as stale shadows; already-open sessions still need a fresh launch | focused `runtime_env` cargo tests pass; clean profile `yzx run` resolves all three through `.nix-profile/bin` first |
| Claude URL handler | Nix profile `claude` wrapper | User-local desktop handler | `~/.local/share/applications/claude-code-url-handler.desktop` | n/a | `claude` | `xdg-mime query default x-scheme-handler/claude-cli` | n/a | Repaired local handler | Keep profile `Exec`; if a profile-owned desktop handler appears later, migrate away from user-local copy instead of pinning a store private binary | `Exec="/home/flexnetos/.nix-profile/bin/claude" --handle-uri %u`; MIME default is `claude-code-url-handler.desktop` |
| OpenCode | Not active foundation until binary and data owner are defined | Missing from PATH/profile; no usage DB | nixpkgs has `opencode` candidate, not currently included | none | missing | `command -v opencode` | removed from active tray | Intentionally disabled to avoid gray/empty cell | Do not re-enable tray unless foundation owns `opencode` and `/home/flexnetos/.local/share/opencode/opencode.db` | Pure profile PATH reports `opencode` missing; `~/.config/yazelix/settings.jsonc` tray excludes `opencode_go_usage`; generated provider flag is false; direct widget probe returns empty |
| CPU widget | Yazelix generated bar widget | Profile runtime | `yazelix_zellij_bar_widget` | foundation runtime | libexec widget | returns `cpu <percent>` | `cpu` | Working | Keep enabled in active tray | direct widget probe exited 0 |
| RAM widget | Yazelix generated bar widget | Profile runtime | `yazelix_zellij_bar_widget` | foundation runtime | libexec widget | returns `ram <percent>` | `ram` | Working | Keep enabled in active tray | direct widget probe exited 0 |
| Workspace cell | Yazelix pane orchestrator/status bar | Profile generated Zellij config | `src/yazelix` | foundation runtime | generated config/plugin | layout contains `{pipe_workspace}` | `workspace` | Enabled | Verify visually after relaunch; do not edit generated layouts by hand | active config tray includes `workspace` |
| Yazi assets/plugins | Child asset repo consumed by Yazelix package | Yazelix flake input plus generated runtime output | `src/yazelix-yazi-assets` | `yazelixYaziAssets` input | generated plugin tree | plugin/flavor files under `~/.local/share/yazelix/configs/yazi` | n/a | Repaired | Keep child input pinned to published commit with smart tabs; use generated runtime only as proof | `yzx --version-full` reports yazi assets `471073d`; generated runtime includes `smart-tabs.yazi/main.lua` and 24 flavors |
| git-kb | Profile package | Profile toolbin | GitKB release packaging | foundation extra runtime package | `git-kb` | `git-kb --version` | n/a | Working | Keep release package, not local binary shim | `/home/flexnetos/.nix-profile/bin/git-kb --version` -> `git-kb 0.2.12` |
| bun | LifeOS/FlexNetOS workspace toolchain | FlexNetOS usr/bin | `/home/flexnetos/FlexNetOS/usr/bin/bun` | none in foundation | not profile exported | `bun --version` | n/a | Present outside profile by design | Keep out of `yzx` profile unless Yazelix itself needs to own JS package management; LifeOS scripts call `bun` through workspace toolchain | pure profile PATH reports missing; workspace PATH resolves `bun` -> `1.3.14`; LifeOS AGENTS says use bun for JS |
| bunx | LifeOS/FlexNetOS workspace toolchain | FlexNetOS usr/bin | `/home/flexnetos/FlexNetOS/usr/bin/bunx` | none in foundation | not profile exported | `bunx --version` | n/a | Present outside profile by design | Same as bun | pure profile PATH reports missing; workspace PATH resolves `bunx` -> `1.3.14` |
| kache | FlexNetOS workspace/control-plane toolchain | FlexNetOS usr/bin | `/home/flexnetos/FlexNetOS/usr/bin/kache` via envctl/meta | none in foundation | not profile exported | `kache --version` | n/a | Present outside profile by design | Keep meta/envctl ownership; do not recreate user-local shims or add to `yzx` profile | pure profile PATH reports missing; workspace PATH resolves `kache` -> `0.8.0`; local workaround proof says active frontdoors are meta-owned under `/home/flexnetos/FlexNetOS/usr/bin` |
| wild linker | FlexNetOS workspace/control-plane toolchain | FlexNetOS usr/bin | `/home/flexnetos/FlexNetOS/usr/bin/wild` via envctl/meta | none in foundation | not profile exported | `wild --version` | n/a | Present outside profile by design | Keep meta/envctl ownership; no `ld.wild` shim unless owner defines it | pure profile PATH reports missing; workspace PATH resolves `wild` -> `0.9.0`; `ld.wild` remains missing |
| meta | FlexNetOS workspace control plane | FlexNetOS usr/bin | `/home/flexnetos/FlexNetOS/usr/bin/meta` | none in foundation | not profile exported | `meta --version` | n/a | Present outside profile by design | Keep workspace root control-plane ownership; do not make Yazelix profile own Meta CLI without a package contract | pure profile PATH reports missing; workspace PATH resolves `meta` -> `0.2.22`; `WORKSPACE_LAYOUT.md` defines `META_ROOT` as `src/meta` |
| vue | LifeOS project-local dependency | Missing as global command | `src/lifeos/package.json` dependency | none | missing | `bun run build` path, not `command -v vue` | n/a | Not a global profile export | Keep as LifeOS dependency, not `yzx` foundation export; project uses `vue-tsc` and Vite scripts | package.json lists `vue` and build scripts; `command -v vue` missing is expected |
| vite | LifeOS project-local dependency | Missing as global command | `src/lifeos/package.json` devDependency | none | missing | `bun run dev` / `bun run build` | n/a | Not a global profile export | Keep project-local; run through Bun scripts | package.json `dev` is `vite`, `build` runs `vite build`; `tauri.conf.json` calls `bun run dev/build` |
| tauri | LifeOS project-local CLI | Missing as global command | `src/lifeos/package.json` devDependency `@tauri-apps/cli` | none | missing | `bun run tauri:*` | n/a | Not a global profile export | Keep project-local unless native installer workflow requires a profile-owned host CLI | package.json defines `tauri`, `tauri:dev`, and `tauri:build` scripts |
| wasmEdge | No active Yazelix/LifeOS foundation owner | Missing | research-only mentions outside active foundation docs | none | missing | `command -v wasmEdge` and `command -v wasmedge` | n/a | Not part of current foundation | Leave out of `yzx` profile until an owning workflow/package requires it | repo search found only current ledger/worklog plus `meta-ruvector` research examples; no active Yazelix/LifeOS consumer |
| built-in Yazelix tools | Yazelix profile runtime | Profile/runtime closure | `src/yazelix` runtime package list | foundation runtime | mostly not top-level exports | `yzx doctor` | n/a | Healthy | Do not widen profile exports without explicit owner decision | doctor reports runtime healthy; optional `mise` and `tombi` host tools unavailable only |

## Verification

Concrete verification path for this contract:

```text
nix develop --accept-flake-config .#ci -c cargo build --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -p yazelix_core --bin yzx_core
nix develop --accept-flake-config .#ci -c rust_core/target/debug/yzx_repo_validator validate-contracts
nix develop --accept-flake-config .#ci -c rust_core/target/debug/yzx_repo_validator validate-child-release-transaction
nix build --accept-flake-config --no-write-lock-file .#checks.x86_64-linux.lifeos_foundation_yzx_runtime_release_contracts --no-link --print-out-paths --log-format raw
```

## Validation Already Run

### 2026-07-07 Final Profile Repair and Claude 2.1.202

```text
env -u FLEXNETOS_GIT_KB_PATH -u FLEXNETOS_RTK_PATH -u NIXPKGS_ALLOW_UNFREE nix build --accept-flake-config --no-write-lock-file .#lifeos_foundation_yzx --no-link --print-out-paths --log-format raw
-> /nix/store/xxmpnb3w6k12qwsyv7wdc7qmx1g3sb3f-lifeos-foundation-yzx

env -u FLEXNETOS_GIT_KB_PATH -u FLEXNETOS_RTK_PATH -u NIXPKGS_ALLOW_UNFREE nix build --accept-flake-config --no-write-lock-file .#checks.x86_64-linux.lifeos_foundation_yzx_runtime_release_contracts --no-link --print-out-paths --log-format raw
-> /nix/store/2wvb76r3w06h5li73id0g35h27864yd5-yazelix-runtime-release-contracts

env -u FLEXNETOS_GIT_KB_PATH -u FLEXNETOS_RTK_PATH -u NIXPKGS_ALLOW_UNFREE nix build --accept-flake-config --no-write-lock-file .#claude --no-link --print-out-paths --log-format raw
-> /nix/store/fha66lcq86lkj8qf1dl6vign6cw7a93c-claude-code-2.1.202

/home/flexnetos/.nix-profile/bin/yzx update upstream
-> ✅ Yazelix profile updated.

/home/flexnetos/.nix-profile/bin/yzx doctor --fix
-> Generated runtime state repaired.

/home/flexnetos/.nix-profile/bin/yzx desktop install
-> Installed Yazelix desktop entry: /home/flexnetos/.local/share/applications/com.yazelix.Yazelix.Mars.desktop

readlink -f /home/flexnetos/.nix-profile/bin/yzx
-> /nix/store/20p5djw21m3lji2sr2chvdyd36ngmj4m-lifeos-foundation-yzx/bin/yzx

/home/flexnetos/.nix-profile/bin/yzx status
-> Generated Runtime State: Status up to date; Repair needed no

/home/flexnetos/.nix-profile/bin/yzx doctor
-> All checks passed.

TERM=xterm-256color YAZELIX_ZELLIJ_SESSION_NAME=codex_visual_proof_20260707 YAZELIX_LAYOUT_OVERRIDE=/home/flexnetos/.nix-profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl YAZELIX_STARTUP_PROFILE_SKIP_WELCOME=1 /home/flexnetos/.nix-profile/bin/yzx enter --home --setup-only --with core.skip_welcome_screen=true
-> Generated 6 shell initializers; Setup complete; optional tools not found: atuin, mise.
```

Permission-regression checks:

```text
nix develop --accept-flake-config .#ci -c cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core readonly -- --nocapture
-> initializer, Zellij, config override, and Yazi readonly repair tests passed.

nix develop --accept-flake-config .#ci -c cargo fmt --manifest-path rust_core/Cargo.toml --all -- --check
-> passed

nix develop --accept-flake-config .#ci -c cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core widget_tray_picker_marks_host_selected_status_widgets_checked -- --nocapture
-> 1 passed; host-selected workspace, Claude, CPU, and RAM tray values render checked while OpenCode remains unchecked.
```

Clean profile versions:

```text
claude=2.1.202 (Claude Code)
codex=codex-cli 0.143.0-alpha.35
rtk=rtk 0.43.0
git-kb=git-kb 0.2.12
```

### 2026-07-07 Codex Profile Path Repair

```text
desktop-file-validate /home/flexnetos/.local/share/applications/claude-code-url-handler.desktop /home/flexnetos/.local/share/applications/com.flexnetos.Yazelix.Agent.desktop /home/flexnetos/.local/share/applications/com.yazelix.Yazelix.Mars.desktop
nix develop --accept-flake-config --no-write-lock-file --command cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-installed-runtime-contract
nix develop --accept-flake-config --no-write-lock-file --command cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_maintainer --bin yzx_repo_validator -- validate-flake-interface
nix build --accept-flake-config --no-write-lock-file .#checks.x86_64-linux.lifeos_foundation_yzx_runtime_release_contracts .#checks.x86_64-linux.runtime_release_contracts .#checks.x86_64-linux.kgp_package_contracts --no-link --print-out-paths --log-format raw
nix build --accept-flake-config --no-write-lock-file .#lifeos_foundation_yzx --no-link --print-out-paths --log-format raw
nix build --accept-flake-config --no-write-lock-file .#codex .#claude --no-link --print-out-paths --log-format raw
nix flake check --accept-flake-config --no-write-lock-file --no-build --log-format raw
nix develop --accept-flake-config --no-write-lock-file --command cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core runtime_env -- --nocapture
nix develop --accept-flake-config --no-write-lock-file --command cargo fmt --all --manifest-path rust_core/Cargo.toml --check
YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1 /nix/store/znvz5frvs5aw3dmr6gwlshmwn7njdabd-lifeos-foundation-yzx/bin/yzx run sh -c 'command -v yzx; command -v codex; command -v claude'
env -i HOME=/home/flexnetos USER=flexnetos LOGNAME=flexnetos SHELL=/bin/sh TERM=xterm-256color PATH=/home/flexnetos/.nix-profile/bin:/home/flexnetos/.local/state/nix/profile/bin:/nix/var/nix/profiles/default/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin /home/flexnetos/.nix-profile/bin/yzx run sh -c 'command -v yzx; command -v codex; command -v claude'
env -i HOME=/home/flexnetos USER=flexnetos LOGNAME=flexnetos SHELL=/bin/sh TERM=xterm-256color PATH=/home/flexnetos/.nix-profile/bin:/home/flexnetos/.local/state/nix/profile/bin:/nix/var/nix/profiles/default/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin /home/flexnetos/.nix-profile/bin/yzx status
/home/flexnetos/.nix-profile/bin/yzx doctor
```

The clean profile runtime proof resolves `yzx`, `codex`, and `claude` through
`/home/flexnetos/.nix-profile/bin` before runtime-local tool paths. The
already-open Codex shell still carries stale
`fqzbpni171j0q86gbgirgj49npq7fzbq-yazelix-flexnetos-foundation` environment
entries from its launch-time session; treat those as inherited session state,
not as the active owner for fresh launches.

```text
env -u FLEXNETOS_GIT_KB_PATH -u FLEXNETOS_RTK_PATH -u NIXPKGS_ALLOW_UNFREE nix build --accept-flake-config .#lifeos_foundation_yzx --no-link --print-out-paths --log-format raw
env -u FLEXNETOS_GIT_KB_PATH -u FLEXNETOS_RTK_PATH -u NIXPKGS_ALLOW_UNFREE nix build --accept-flake-config .#checks.x86_64-linux.lifeos_foundation_yzx_runtime_release_contracts --no-link --print-out-paths --log-format raw
nix develop --accept-flake-config .#ci -c cargo fmt --manifest-path rust_core/Cargo.toml --all -- --check
cargo test -p yazelix_core resolves_rtk_for_codex_agent
cargo test -p yazelix_core codex_without_rtk_is_rejected
cargo test -p yazelix_zellij_config_pack wraps_direct_codex_right_sidebar_with_rtk
/home/flexnetos/.nix-profile/bin/yzx update upstream
/home/flexnetos/.nix-profile/bin/yzx doctor --fix
/home/flexnetos/.nix-profile/bin/yzx desktop install
/home/flexnetos/.nix-profile/bin/yzx doctor
```

## New Session Prompt

```text
You are continuing the FlexNetOS/Yazelix foundation repair from
/home/flexnetos/FlexNetOS/src/yazelix.

Read and obey:
- /home/flexnetos/.codex/RTK.md
- /home/flexnetos/.codex/AGENTS.rtk.md
- /home/flexnetos/FlexNetOS/AGENTS.md
- /home/flexnetos/FlexNetOS/src/yazelix/AGENTS.md
- /home/flexnetos/FlexNetOS/src/lifeos/AGENTS.md if touching LifeOS ownership
- /home/flexnetos/FlexNetOS/src/yazelix-yazi-assets/AGENTS.md if touching Yazi assets

Runtime ownership rules:
- Editable input: /home/flexnetos/.config/yazelix/
- Generated proof: /home/flexnetos/.local/share/yazelix/
- Active frontdoor: /home/flexnetos/.nix-profile/bin/yzx
- Do not hand-edit generated runtime under ~/.local/share/yazelix.
- Do not restore ~/.local/bin/yzx or stale user-local desktop launchers as ownership layers.
- Do not run yzx restart.

Start by verifying current state, not by assuming this prompt is still fresh:
1. cd /home/flexnetos/FlexNetOS/src/yazelix
2. nix run --accept-flake-config .#br -- show yazelix-xig9o
3. git status --short --branch
4. /home/flexnetos/.nix-profile/bin/yzx status
5. /home/flexnetos/.nix-profile/bin/yzx doctor
6. readlink -f /home/flexnetos/.nix-profile/bin/yzx
7. nix profile list --json
8. /home/flexnetos/.nix-profile/bin/yzx --version-full
9. env -i HOME=/home/flexnetos USER=flexnetos LOGNAME=flexnetos SHELL=/bin/sh TERM=xterm-256color PATH=/home/flexnetos/.nix-profile/bin:/home/flexnetos/.local/state/nix/profile/bin:/nix/var/nix/profiles/default/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin /bin/sh -lc 'command -v yzx && readlink -f $(command -v yzx)'

Known repaired state as of 2026-07-07:
- Source build of .#lifeos_foundation_yzx passes without NIXPKGS_ALLOW_UNFREE
  and returns a lifeos-foundation-yzx store path.
- .#yazelix_flexnetos_foundation is no longer exposed in the current worktree
  after the active profile migrated to lifeos_foundation_yzx.
- Source runtime-release contracts pass through
  lifeos_foundation_yzx_runtime_release_contracts and require smart-tabs.yazi.
- Profile update through /home/flexnetos/.nix-profile/bin/yzx update upstream succeeded.
- nix profile list --json shows active lifeos_foundation_yzx from path:/home/flexnetos/FlexNetOS/src/yazelix.
- readlink -f /home/flexnetos/.nix-profile/bin/yzx reports /nix/store/20p5djw21m3lji2sr2chvdyd36ngmj4m-lifeos-foundation-yzx/bin/yzx.
- yzx --version-full reports v17.9 and yazi assets revision 471073d54d4a6c9fa9e87f26134d6db3f387977e.
- clean profile claude --version reports 2.1.202 (Claude Code).
- Profile RTK is upstream rtk 0.43.0.
- Profile git-kb is 0.2.12.
- yzx doctor passes after doctor --fix.
- yzx enter --home --setup-only --with core.skip_welcome_screen=true succeeds
  after generated initializer and session override permission repair.
- desktop install was rerun after config updates.
- claude-code-url-handler.desktop uses /home/flexnetos/.nix-profile/bin/claude.
- Source now defines .#lifeos_foundation_yzx with derivation/runtime names
  lifeos-foundation-yzx and lifeos-foundation-yzx-runtime.
- Mars desktop Exec uses /home/flexnetos/.nix-profile/bin/yzx desktop launch.
- Agent desktop Exec uses YAZELIX_LAYOUT_OVERRIDE=/home/flexnetos/.nix-profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl and profile yzx.
- ~/.local/bin/yzx is absent.
- Active widget_tray is ["session","editor","shell","term","workspace","claude_usage","codex_usage","cpu","ram"].
- OpenCode usage is intentionally removed from the active tray because opencode is missing and ~/.local/share/opencode/opencode.db is absent.
- Current running Codex shell may still have an old store path ahead of .nix-profile; treat that as inherited session state and verify fresh-shell PATH before changing packages.

Resolved ownership decisions:
1. Keep the active profile target on lifeos_foundation_yzx. Do not re-add the
   old yazelix_flexnetos_foundation attr unless a future compatibility owner
   proves it is still required.
2. Keep the active foundation on upstream RTK 0.43.0. The rtk-tokenkill fork is
   not in sync with upstream v0.43.0 and is not part of the active foundation
   path unless a real FlexNetOS delta is later proven.
3. Keep bun and bunx as LifeOS/FlexNetOS workspace toolchain commands under
   /home/flexnetos/FlexNetOS/usr/bin, not Yazelix profile exports.
4. Keep kache, wild, and meta as meta/envctl-owned workspace frontdoors under
   /home/flexnetos/FlexNetOS/usr/bin, not Yazelix profile exports.
5. Keep vue, vite, and tauri project-local to LifeOS and invoked through Bun
   scripts, not global yzx profile commands.
6. Keep wasmEdge/wasmedge out of the foundation until an owning Yazelix/LifeOS
   workflow appears. Current active repo search found no foundation consumer.
7. Keep OpenCode disabled in the tray until both the binary owner and
   /home/flexnetos/.local/share/opencode/opencode.db owner are defined.
8. Keep Yazi assets pinned to the published child commit that includes
   smart-tabs.yazi, and keep runtime contracts checking that plugin.

Before claiming success, run and report exact proof:
- nix build --accept-flake-config .#checks.x86_64-linux.lifeos_foundation_yzx_runtime_release_contracts --no-link --print-out-paths --log-format raw
- nix build --accept-flake-config .#lifeos_foundation_yzx --no-link --print-out-paths --log-format raw
- /home/flexnetos/.nix-profile/bin/yzx update upstream
- /home/flexnetos/.nix-profile/bin/yzx doctor --fix
- /home/flexnetos/.nix-profile/bin/yzx desktop install
- /home/flexnetos/.nix-profile/bin/yzx status
- /home/flexnetos/.nix-profile/bin/yzx doctor
- command probes for every required exported tool
- desktop Exec proof
- no ~/.local/bin/yzx shadow
- generated runtime proof only, no manual edits under ~/.local/share/yazelix
- live visual proof for any remaining gray/unchecked menu cells; a bounded
  noninteractive yzx menu capture only produced terminal control sequences and
  is not sufficient as acceptance proof.

Do not mark the task done if any gray cell/tool ownership is only hidden by a
workaround. Surface it in the repair ledger with current_state, required_action,
and proof.
```
