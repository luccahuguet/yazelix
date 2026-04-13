# Yazelix Subsystem Code Inventory

This document is the current code-only LOC snapshot for the subsystem families defined in [Architecture Map](./architecture_map.md).

The old flat inventory was useful as a first pass, but it was too coarse for real trim work. The current version keeps the same five-family top-level snapshot and then adds a second layer focused on the two heaviest areas: runtime control and maintainer workflow.

## Snapshot

- Snapshot date: `2026-04-12`
- Metric: `tokei` `Code` column only
- Scope: tracked repo code with Markdown excluded
- Excludes: docs prose, Markdown code fences, local scratch state, build outputs, `.git`, `.beads`, and non-source binary assets
- Covered by this inventory: `35,617` code lines

For reference, a raw `tokei .` snapshot is higher because it includes Markdown code fences. This inventory intentionally excludes those lines so the totals reflect maintained product and maintainer code only.

## Peak Comparison

- Current `HEAD`: `35,617` code LOC
- Code-only historical peak measured across the full commit graph: `42,170` code LOC at commit `7f0d3c2` on `2026-04-09`
- Delta from peak to current: `-6,553` code LOC, a `15.5%` reduction
- `v14` itself was almost the same size at `41,733` code LOC, so the current trim is effectively `6.1k` lines below the `v14` release line too

That means the repo has regrown slightly since the first post-trim inventory refresh, but the broader delete-first pass still holds: Yazelix remains materially smaller than its peak.

## Top-Level Inventory Buckets

| Subsystem family | Files | Code LOC | Share | Counted source roots |
| --- | ---: | ---: | ---: | --- |
| Runtime control plane and command surface | 80 | 12,582 | 35.3% | `nushell/scripts/core`, `nushell/scripts/setup`, `nushell/scripts/utils`, `nushell/scripts/yzx` |
| Maintainer workflow and validation | 53 | 12,547 | 35.2% | `nushell/scripts/dev`, `nushell/scripts/maintainer`, `scripts`, `maintainer_shell.nix`, `.github`, `.nu-lint.toml` |
| Shipped runtime data and assets | 75 | 5,907 | 16.6% | `configs`, `config_metadata`, `user_configs`, `assets`, `nushell/config`, `.taplo.toml`, `yazelix_default.toml`, `docs/upgrade_notes.toml` |
| Workspace session orchestration | 30 | 3,579 | 10.0% | `nushell/scripts/integrations`, `nushell/scripts/zellij_wrappers`, `rust_plugins/` |
| Distribution and host integration | 20 | 1,002 | 2.8% | `home_manager`, `packaging`, `shells`, `flake.nix`, `yazelix_package.nix`, `yazelix_runtime_package.nix` |

The top-level picture changed in one important way: runtime and maintainer code are now effectively tied. Runtime is still the largest bucket, but only by `35` LOC.

## Runtime Detailed View

Runtime is still the biggest product-facing subsystem, but it is no longer overwhelmingly larger than everything else. The useful questions are still:

- how much of runtime cost is direct command surface versus helper stacks
- whether the weight lives in startup/materialization, config migration, diagnostics, or front-door UX
- how much maintainer-only logic is still shipped in runtime paths

### Exact Runtime Path Partition

This is the literal path-based split inside the `12,582` LOC runtime bucket.

| Runtime slice | Files | Code LOC | Share of runtime |
| --- | ---: | ---: | ---: |
| `nushell/scripts/utils` | 46 | 7,629 | 60.6% |
| `nushell/scripts/setup` | 14 | 2,429 | 19.3% |
| `nushell/scripts/yzx` | 16 | 1,674 | 13.3% |
| `nushell/scripts/core` | 4 | 850 | 6.8% |

The main runtime fact still holds: command wrappers are not the main weight. `utils` and `setup` are.

### Exact Runtime Trim Partition

This is a trim-oriented repartition of the same `12,582` runtime LOC. Each runtime file is assigned to exactly one group below.

| Runtime trim group | Files | Code LOC | Share of runtime | What it captures |
| --- | ---: | ---: | ---: | --- |
| Startup and generated-state materialization | 23 | 3,715 | 29.5% | Launch/bootstrap entrypoints, generated config writes, shell hooks, Zellij and Yazi materialization, startup profile, and render-time state |
| Config lifecycle stack | 14 | 2,758 | 21.9% | Config parsing, schema, diagnostics, migration engine, and the `yzx config` / `edit` / `import` surfaces |
| Command surface and shared runtime glue | 23 | 2,389 | 19.0% | Direct command glue, shared runtime helpers, terminal launch transport, and the remaining shipped `yzx dev` entrypoint |
| Front-door UX and transient workspace tools | 11 | 1,972 | 15.7% | Welcome animation, command palette, popup pane, tutorial/keys/screen flows, and release-note rendering |
| Doctor, ownership, and runtime validation | 9 | 1,748 | 13.9% | Health checks, install/runtime ownership, nix detection, version reporting, and runtime-contract checks |

Two useful runtime conclusions fall straight out of that partition:

- The heaviest single runtime area is still startup and generated-state materialization, not the visible command wrappers.
- The old broad "runtime-shipped maintainer helpers" seam is mostly gone. The obvious shipped maintainer command surface left in runtime paths is now basically `yzx dev` at `181` LOC, not a multi-file helper cluster.

### Direct Command Surface

The direct command files are still smaller than the surrounding support stacks.

- File-based `yzx` command modules outside maintainer-only `yzx dev`: `1,493` LOC
- Inline `yzx` commands still defined in [yazelix.nu](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu): `347` LOC
- Approximate direct user-facing command surface: `1,840` LOC, about `14.6%` of runtime

That is still the key inventory insight: most runtime complexity is not in the command wrappers themselves. It is in the helpers they sit on top of.

### Direct `yzx` Command Families

This table is intentionally about direct command-family LOC, not full ownership cost. It answers "how big is the wrapper itself?" not "how expensive is everything that command touches?"

| Command family | Direct LOC | Where it lives | Notes |
| --- | ---: | --- | --- |
| `yzx desktop` | 245 | [`desktop.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/desktop.nu) | Direct launcher and desktop-entry command family |
| `yzx import` | 202 | [`import.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/import.nu) | Managed-config import surface |
| `yzx menu` | 205 | [`menu.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/menu.nu) | Command-palette wrapper on top of the shared transient-pane path |
| `yzx dev` | 181 | [`dev.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/dev.nu) | Maintainer-only surface, still shipped but much smaller than the old helper cluster |
| `yzx edit` | 160 | [`edit.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/edit.nu) | Managed config editing targets |
| `yzx keys` | 151 | [`keys.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/keys.nu) | Discoverability surface |
| `yzx screen` | 95 | [`screen.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/screen.nu) | Small wrapper over the larger welcome stack |
| `yzx status` | 87 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline status command span only |
| `yzx home_manager` | 84 | [`home_manager.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/home_manager.nu) | Home Manager takeover helper surface |
| `yzx env` | 70 | [`env.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/env.nu) | Small wrapper over bootstrap and environment helpers |
| `yzx launch` | 68 | [`launch.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/launch.nu) | Small wrapper over startup/materialization |
| `yzx cwd` | 63 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx tutor` | 61 | [`tutor.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/tutor.nu) | Front-door help surface |
| `yzx config` | 59 | [`config.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/config.nu) | Narrow config management surface after v15 trims |
| `yzx popup` | 47 | [`popup.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/popup.nu) | Popup wrapper on the same transient-pane mechanism as `yzx menu --popup` |
| `yzx restart` | 43 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx update nix` | 42 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx enter` | 27 | [`enter.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/enter.nu) | Small wrapper over current-terminal startup |
| `yzx update home_manager` | 26 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx sponsor` | 23 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx update upstream` | 21 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx doctor` | 15 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Small wrapper over the heavier doctor stack |
| `yzx run` | 15 | [`run.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/run.nu) | Thin passthrough wrapper; any future cut is about surface clarity, not raw LOC |
| `yzx why` | 11 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx update` | 9 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline dispatcher span only |
| `yzx reveal` | 7 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx whats_new` | 4 | [`whats_new.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/whats_new.nu) | Tiny wrapper over upgrade-summary helpers |

### Runtime Hotspot Files

These are the heaviest runtime-side files after the latest trim.

| File | Code LOC | Why it matters |
| --- | ---: | --- |
| [`ascii_art.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/ascii_art.nu) | 909 | Most of the front-door UX cost still lives here |
| [`config_migrations.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/config_migrations.nu) | 408 | Central config-migration ownership |
| [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | 399 | Inline `yzx` command definitions plus shared command glue |
| [`doctor.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/doctor.nu) | 351 | Main troubleshooting surface |
| [`terminal_renderers.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/terminal_renderers.nu) | 310 | Shared terminal materialization logic |
| [`config_migration_transactions.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/config_migration_transactions.nu) | 309 | Migration transaction engine |
| [`zellij_plugin_paths.nu`](/home/lucca/pjs/yazelix/nushell/scripts/setup/zellij_plugin_paths.nu) | 289 | Zellij plugin ownership and permission-repair seam |
| [`doctor_helix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/doctor_helix.nu) | 274 | Helix-specific diagnostic surface |
| [`upgrade_summary.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/upgrade_summary.nu) | 273 | Runtime-side release and upgrade rendering |
| [`yazi_config_merger.nu`](/home/lucca/pjs/yazelix/nushell/scripts/setup/yazi_config_merger.nu) | 266 | Yazi materialization remains a dense setup seam |

## Maintainer Detailed View

The maintainer bucket is now effectively tied with runtime. That changes the framing: the useful question is no longer just "tests are big." The useful questions are which maintainer responsibilities still cost real LOC and where overlap is most likely.

### Maintainer Path Partition

The path split is now still simple, but no longer trivial:

| Maintainer slice | Files | Code LOC | Share of maintainer bucket |
| --- | ---: | ---: | ---: |
| `nushell/scripts/dev` | 43 | 10,628 | 84.7% |
| `nushell/scripts/maintainer` | 8 | 1,706 | 13.6% |
| `scripts` plus `maintainer_shell.nix` | 2 | 213 | 1.7% |
| `.github` and `.nu-lint.toml` | 0 counted by `tokei` | 0 | 0.0% |

That means the real maintainer inventory still has to be functional, not just path-based.

### Exact Maintainer Partition

This is an exact repartition of the same `12,547` LOC maintainer bucket.

| Maintainer group | Files | Code LOC | Share of maintainer bucket | What it captures |
| --- | ---: | ---: | ---: | --- |
| Runtime behavior and workspace tests | 9 | 4,092 | 32.6% | Core runtime, workspace, popup, doctor, sidebar, and plugin behavior coverage |
| Maintainer update, release, and issue flows | 12 | 2,723 | 21.7% | Maintainer-command coverage plus issue sync, readme/build/update flows, version bumping, and Beads/GitHub contract helpers |
| Generated config and managed-config coverage | 4 | 2,442 | 19.5% | Generated config coverage, shell/Helix managed-config contracts, and config sweep behavior |
| Validators and install-contract checks | 14 | 1,710 | 13.6% | Traceability, flake/install, syntax/spec, readme, config-surface, and upgrade validators |
| Sweep, demo, and harness glue | 10 | 1,082 | 8.6% | Demo recorders, sweep runners, shared test helpers, the maintainer test runner, and the maintainer shell |
| Upgrade and migration behavior tests | 4 | 498 | 4.0% | Historical upgrade-note, stale-config, and upgrade-summary behavior tests |

The main maintainer fact now is more nuanced:

- Runtime behavior tests are still the largest single maintainer group.
- Maintainer update/release/issue machinery is now the second-largest group, which reflects the recent movement of maintainer-only ownership out of shipped runtime paths and into dedicated maintainer surfaces.

### Maintainer Hotspot Files

These are the heaviest maintainer-side files after the latest trim:

| File | Code LOC | Why it matters |
| --- | ---: | --- |
| [`test_yzx_generated_configs.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_generated_configs.nu) | 1,401 | Largest single maintainer file; generated-config coverage is still a major cost center |
| [`test_yzx_core_commands.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_core_commands.nu) | 1,173 | Core command coverage remains a large maintenance surface |
| [`test_yzx_workspace_commands.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_workspace_commands.nu) | 1,142 | Workspace behavior is still one of the heaviest defended surfaces |
| [`test_yzx_maintainer.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_maintainer.nu) | 956 | Maintainer/update/build surface is concentrated here |
| [`test_shell_managed_config_contracts.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_shell_managed_config_contracts.nu) | 535 | Shell-config contract coverage is still large |
| [`test_yzx_popup_commands.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_popup_commands.nu) | 530 | Popup and menu behavior still carry substantial regression cost |
| [`update_workflow.nu`](/home/lucca/pjs/yazelix/nushell/scripts/maintainer/update_workflow.nu) | 481 | The maintainer update path is now the largest non-test maintainer file |
| [`test_yzx_doctor_commands.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_doctor_commands.nu) | 427 | Doctor coverage remains expensive even after command-surface trims |
| [`validate_flake_install.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/validate_flake_install.nu) | 347 | Biggest validator file |
| [`validate_upgrade_contract.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/validate_upgrade_contract.nu) | 337 | Upgrade contract guard remains comparatively heavy |

## What The Detailed Inventory Says Now

- Runtime and maintainer code are now effectively tied at the top level: `12,582` versus `12,547` LOC.
- The repo is still materially smaller than its code-only peak: `35,617` now versus `42,170` at peak, a `15.5%` reduction.
- Inside runtime, `startup and generated-state materialization` is still the largest exact trim partition at `3,715` LOC.
- The config lifecycle stack is still substantial at `2,758` LOC even after recent command-surface trimming.
- The front-door UX and transient workspace cluster is now `1,972` LOC, which reflects the retained welcome surface plus the unified popup/menu transient path.
- The old broad maintainer-helper seam inside runtime has shrunk sharply. What remains is mostly direct shipped surface, not a hidden maintainer sublayer.
- Inside maintainer code, runtime behavior tests are still the largest bucket, but the second-place maintainer update/release/issue group at `2,723` LOC is now big enough that maintainer trim can no longer be framed as "just delete tests."

## Counting Rules

The five top-level subsystem families still follow the architecture map exactly:

1. Runtime control plane and command surface
2. Workspace session orchestration
3. Distribution and host integration
4. Shipped runtime data and assets
5. Maintainer workflow and validation

Each counted source file belongs to exactly one of those five families.

The detailed runtime and maintainer sections also use exact single-owner assignment within their parent subsystem. The command-family table is different on purpose: it is a direct wrapper view, not a full ownership accounting model.

## Reproduce The Snapshot

The top-level five-family totals are reproducible with `tokei --exclude '*.md' -o json .` plus the same explicit path assignment used below:

```bash
python3 - <<'PY'
import json
import subprocess
from collections import defaultdict

raw = subprocess.check_output(
    ["tokei", "--exclude", "*.md", "-o", "json", "."],
    text=True,
)
data = json.loads(raw)

categories = {
    "Runtime control plane and command surface": lambda p: (
        p.startswith("./nushell/scripts/core/")
        or p.startswith("./nushell/scripts/setup/")
        or p.startswith("./nushell/scripts/utils/")
        or p.startswith("./nushell/scripts/yzx/")
    ),
    "Workspace session orchestration": lambda p: (
        p.startswith("./nushell/scripts/integrations/")
        or p.startswith("./nushell/scripts/zellij_wrappers/")
        or p.startswith("./rust_plugins/")
    ),
    "Distribution and host integration": lambda p: (
        p.startswith("./home_manager/")
        or p.startswith("./packaging/")
        or p.startswith("./shells/")
        or p in {
            "./flake.nix",
            "./yazelix_package.nix",
            "./yazelix_runtime_package.nix",
        }
    ),
    "Shipped runtime data and assets": lambda p: (
        p.startswith("./configs/")
        or p.startswith("./config_metadata/")
        or p.startswith("./user_configs/")
        or p.startswith("./assets/")
        or p.startswith("./nushell/config/")
        or p in {
            "./.taplo.toml",
            "./yazelix_default.toml",
            "./docs/upgrade_notes.toml",
        }
    ),
    "Maintainer workflow and validation": lambda p: (
        p.startswith("./nushell/scripts/dev/")
        or p.startswith("./nushell/scripts/maintainer/")
        or p.startswith("./scripts/")
        or p.startswith("./.github/")
        or p in {
            "./maintainer_shell.nix",
            "./.nu-lint.toml",
        }
    ),
}

counts = defaultdict(lambda: {"files": 0, "code": 0})
for language, info in data.items():
    if language == "Total":
        continue
    for report in info.get("reports", []):
        path = report["name"]
        code = report["stats"]["code"]
        matches = [name for name, pred in categories.items() if pred(path)]
        if len(matches) != 1:
            raise SystemExit(f"Expected exactly one bucket for {path}, got {matches}")
        bucket = matches[0]
        counts[bucket]["files"] += 1
        counts[bucket]["code"] += code

total = data["Total"]["code"]
for name in categories:
    files = counts[name]["files"]
    code = counts[name]["code"]
    share = (code / total) * 100 if total else 0
    print(f"{name}: files={files} code={code} share={share:.1f}%")
print(f"Total code={total}")
PY
```

The historical peak comparison in this document was measured with the same `tokei --exclude '*.md' -o json .` method inside a temporary git worktree across the full commit graph.

## Traceability

- Bead: `yazelix-mdqo`
