# Yazelix Subsystem Code Inventory

This document is the current code-only LOC snapshot for the subsystem families defined in [Architecture Map](./architecture_map.md).

The old flat inventory was useful as a first pass, but it was too coarse for real trim work. The current version keeps the same five-family top-level snapshot and then adds a second layer focused on the two heaviest areas: runtime control and maintainer workflow.

## Snapshot

- Snapshot date: `2026-04-12`
- Metric: `tokei` `Code` column only
- Scope: tracked repo code with Markdown excluded
- Excludes: docs prose, Markdown code fences, local scratch state, build outputs, `.git`, `.beads`, and non-source binary assets
- Covered by this inventory: `34,850` code lines

For reference, a raw `tokei .` snapshot is higher because it includes Markdown code fences. This inventory intentionally excludes those lines so the totals reflect maintained product and maintainer code only.

## Top-Level Inventory Buckets

| Subsystem family | Files | Code LOC | Share | Counted source roots |
| --- | ---: | ---: | ---: | --- |
| Runtime control plane and command surface | 87 | 14,270 | 40.9% | `nushell/scripts/core`, `nushell/scripts/setup`, `nushell/scripts/utils`, `nushell/scripts/yzx` |
| Maintainer workflow and validation | 45 | 10,420 | 29.9% | `nushell/scripts/dev`, `.github`, `maintainer_shell.nix`, `.nu-lint.toml` |
| Shipped runtime data and assets | 78 | 5,949 | 17.1% | `configs`, `config_metadata`, `user_configs`, `assets`, `nushell/config`, `.taplo.toml`, `yazelix_default.toml`, `docs/upgrade_notes.toml` |
| Workspace session orchestration | 34 | 3,209 | 9.2% | `nushell/scripts/integrations`, `nushell/scripts/zellij_wrappers`, `rust_plugins/` |
| Distribution and host integration | 20 | 1,002 | 2.9% | `home_manager`, `packaging`, `shells`, `flake.nix`, `yazelix_package.nix`, `yazelix_runtime_package.nix` |

## Runtime Detailed View

The runtime bucket is still the largest subsystem, but the useful question is no longer just "runtime is big." The useful questions are:

- how much of runtime cost is direct command surface versus helper stacks
- whether the weight lives in startup/materialization, config migration, diagnostics, or front-door UX
- how much maintainer-only logic is still shipped inside runtime paths

### Exact Runtime Path Partition

This is the literal path-based split inside the `14,270` LOC runtime bucket.

| Runtime slice | Files | Code LOC | Share of runtime |
| --- | ---: | ---: | ---: |
| `nushell/scripts/utils` | 53 | 9,347 | 65.5% |
| `nushell/scripts/setup` | 14 | 2,432 | 17.0% |
| `nushell/scripts/yzx` | 16 | 1,641 | 11.5% |
| `nushell/scripts/core` | 4 | 850 | 6.0% |

The main runtime fact remains the same: command wrappers are not the main weight. `utils` and `setup` are.

### Exact Runtime Trim Partition

This is a trim-oriented repartition of the same `14,270` runtime LOC. Each runtime file is assigned to exactly one group below.

| Runtime trim group | Files | Code LOC | Share of runtime | What it captures |
| --- | ---: | ---: | ---: | --- |
| Startup and generated-state materialization | 25 | 3,898 | 27.3% | Launch/bootstrap entrypoints, generated config writes, terminal renderers, Zellij/Yazi materialization |
| Other direct commands and shared runtime glue | 25 | 2,749 | 19.3% | Remaining public command files plus shared helpers like `constants`, `common`, `runtime_env`, `upgrade_summary`, and inline `yzx` command glue |
| Config lifecycle stack | 12 | 2,396 | 16.8% | Config parsing, schema, migration rules, migration transactions, config diagnostics, and `yzx config` |
| Runtime-shipped maintainer helpers | 9 | 1,940 | 13.6% | `yzx dev` plus the maintainer/update/release helpers that still live under shipped runtime paths |
| Doctor, ownership, and preflight checks | 10 | 1,754 | 12.3% | `doctor`, install/runtime ownership checks, nix detection, runtime contract checks, version reporting |
| Welcome and front-door UX | 6 | 1,533 | 10.7% | `ascii_art`, welcome flow, `yzx screen`, `yzx tutor`, `yzx keys`, `yzx menu` |

Two useful runtime conclusions fall straight out of that partition:

- The heaviest single runtime area is still startup and materialization, not the visible command wrappers.
- There is still a meaningful `1,940` LOC seam of maintainer-oriented behavior shipped from runtime paths, which is a real delete-first target if v15 keeps narrowing.

### Direct Command Surface

The direct command files are smaller than the surrounding support stacks.

- File-based `yzx` command modules outside maintainer-only `yzx dev`: `1,455` LOC
- Inline `yzx` commands still defined in [yazelix.nu](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu): `347` LOC
- Approximate direct user-facing command surface: `1,802` LOC, about `12.6%` of runtime

That is the key inventory insight: most runtime complexity is not in the command wrappers themselves. It is in the helpers they sit on top of.

### Direct `yzx` Command Families

This table is intentionally about direct command-family LOC, not full ownership cost. It answers "how big is the wrapper itself?" not "how expensive is everything that command touches?"

| Command family | Direct LOC | Where it lives | Notes |
| --- | ---: | --- | --- |
| `yzx desktop` | 245 | [`desktop.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/desktop.nu) | Direct launcher and desktop-entry command family |
| `yzx import` | 202 | [`import.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/import.nu) | Managed-config import surface |
| `yzx dev` | 186 | [`dev.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/dev.nu) | Maintainer-only surface, counted here for visibility but not part of the `1,802` user-facing total above |
| `yzx edit` | 160 | [`edit.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/edit.nu) | Managed config editing targets |
| `yzx menu` | 156 | [`menu.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/menu.nu) | Menu wrapper; real cost also depends on front-door UX helpers |
| `yzx keys` | 151 | [`keys.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/keys.nu) | Discoverability surface |
| `yzx screen` | 95 | [`screen.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/screen.nu) | Small wrapper over the larger welcome stack |
| `yzx status` | 87 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx home_manager` | 84 | [`home_manager.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/home_manager.nu) | Home Manager takeover helper surface |
| `yzx env` | 70 | [`env.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/env.nu) | Small wrapper over bootstrap and environment helpers |
| `yzx launch` | 68 | [`launch.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/launch.nu) | Small wrapper over startup/materialization |
| `yzx cwd` | 63 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx tutor` | 61 | [`tutor.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/tutor.nu) | Front-door help surface |
| `yzx config` | 59 | [`config.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/config.nu) | After trimming, now much narrower |
| `yzx popup` | 58 | [`popup.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/popup.nu) | Popup wrapper |
| `yzx restart` | 43 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx update nix` | 42 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx enter` | 27 | [`enter.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/enter.nu) | Small wrapper over current-terminal startup |
| `yzx update home_manager` | 26 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx sponsor` | 23 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx update upstream` | 21 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx doctor` | 15 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Small wrapper over the heavier doctor stack |
| `yzx run` | 15 | [`run.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/run.nu) | Very small wrapper; any future cut is about surface clarity, not raw LOC |
| `yzx why` | 11 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx update` | 9 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline dispatcher span only |
| `yzx reveal` | 7 | [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | Inline command span only |
| `yzx whats_new` | 4 | [`whats_new.nu`](/home/lucca/pjs/yazelix/nushell/scripts/yzx/whats_new.nu) | Tiny wrapper over upgrade-summary helpers |

### Runtime Hotspot Files

These are the heaviest runtime-side files after the latest trim:

| File | Code LOC | Why it matters |
| --- | ---: | --- |
| [`ascii_art.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/ascii_art.nu) | 909 | Most of the front-door UX cost lives here |
| [`dev_update_workflow.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/dev_update_workflow.nu) | 481 | Large maintainer/update surface still shipped under runtime paths |
| [`config_migrations.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/config_migrations.nu) | 408 | Central config-migration ownership |
| [`yazelix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/core/yazelix.nu) | 399 | Inline `yzx` command definitions plus shared command glue |
| [`doctor.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/doctor.nu) | 351 | Main troubleshooting surface |
| [`test_runner.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/test_runner.nu) | 334 | Maintainer logic still living in shipped runtime paths |
| [`terminal_renderers.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/terminal_renderers.nu) | 310 | Shared terminal materialization logic |
| [`config_migration_transactions.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/config_migration_transactions.nu) | 309 | Migration transaction engine |
| [`zellij_plugin_paths.nu`](/home/lucca/pjs/yazelix/nushell/scripts/setup/zellij_plugin_paths.nu) | 295 | Zellij plugin ownership/materialization seam |
| [`doctor_helix.nu`](/home/lucca/pjs/yazelix/nushell/scripts/utils/doctor_helix.nu) | 274 | Helix-specific diagnostic surface |

## Maintainer Detailed View

The maintainer bucket is now just under one third of the repo. The useful question here is not "tests are big." The useful question is which maintainer responsibilities still cost real LOC and where overlap is most likely.

The path split is almost trivial now:

| Maintainer slice | Files | Code LOC | Share of maintainer bucket |
| --- | ---: | ---: | ---: |
| `nushell/scripts/dev` | 44 | 10,360 | 99.4% |
| `maintainer_shell.nix` | 1 | 60 | 0.6% |
| `.github` and `.nu-lint.toml` | 0 counted by `tokei` | 0 | 0.0% |

That means the real maintainer inventory has to be functional, not path-based.

### Exact Maintainer Partition

This is an exact repartition of the same `10,420` LOC maintainer bucket.

| Maintainer group | Files | Code LOC | Share of maintainer bucket | What it captures |
| --- | ---: | ---: | ---: | --- |
| Runtime behavior tests | 9 | 5,140 | 49.3% | Core runtime/workspace command tests and generated-config behavior tests |
| Validators | 13 | 1,645 | 15.8% | Contract validators, syntax/spec guards, traceability and install validators |
| Maintainer update, release, and build flows | 6 | 1,289 | 12.4% | Maintainer update/build flows plus maintainer command regression coverage |
| Config and upgrade tests | 6 | 1,270 | 12.2% | Managed-config, stale-config, and upgrade-note behavior tests |
| Sweeps and demos | 8 | 839 | 8.1% | Config sweeps and demo recorders |
| Test harness and maintainer shell glue | 3 | 237 | 2.3% | Lightweight helpers plus the maintainer shell definition |

The main maintainer fact is hard to miss: nearly half of the bucket is runtime behavior tests. If the maintainer subsystem needs another serious trim, that is still the first place to audit for overlap.

### Maintainer Hotspot Files

These are the heaviest maintainer-side files after the latest trim:

| File | Code LOC | Why it matters |
| --- | ---: | --- |
| [`test_yzx_generated_configs.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_generated_configs.nu) | 1,431 | Largest single maintainer file; generated-config coverage is a major cost center |
| [`test_yzx_workspace_commands.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_workspace_commands.nu) | 1,142 | Workspace behavior is still one of the heaviest defended surfaces |
| [`test_yzx_core_commands.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_core_commands.nu) | 1,104 | Core command coverage remains a large maintenance surface |
| [`test_yzx_maintainer.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_maintainer.nu) | 955 | Maintainer/update/build surface is concentrated here |
| [`test_shell_managed_config_contracts.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_shell_managed_config_contracts.nu) | 522 | Shell-config contract coverage is still large |
| [`test_yzx_doctor_commands.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_doctor_commands.nu) | 427 | Doctor coverage cost after the recent narrowing |
| [`test_yzx_popup_commands.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_popup_commands.nu) | 407 | Popup behavior remains a substantial test surface |
| [`validate_flake_install.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/validate_flake_install.nu) | 347 | Biggest validator file |
| [`validate_upgrade_contract.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/validate_upgrade_contract.nu) | 337 | Upgrade contract guard remains comparatively heavy |
| [`test_yzx_yazi_commands.nu`](/home/lucca/pjs/yazelix/nushell/scripts/dev/test_yzx_yazi_commands.nu) | 295 | Sidebar/Yazi integration still costs real test LOC |

## What The Detailed Inventory Says Now

- The repo is still runtime-heavy, but runtime is helper-heavy much more than command-wrapper-heavy.
- Inside runtime, `startup and generated-state materialization` is the largest exact trim partition at `3,898` LOC.
- The config lifecycle stack is still substantial at `2,396` LOC even after recent command-surface trimming.
- The front-door UX cluster is still `1,533` LOC even though its visible command wrappers are small.
- Runtime still ships `1,940` LOC of maintainer-oriented helper logic in runtime paths. That is one of the clearest remaining delete-first seams.
- Inside maintainer code, almost half the budget is runtime behavior tests. If the next trim needs to cut maintainer LOC without dropping product features, duplicate or overly coupled behavior coverage is still the first area to audit.

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
        or p.startswith("./.github/")
        or p in {"./maintainer_shell.nix", "./.nu-lint.toml"}
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

The deeper runtime and maintainer partitions in this document use the same `tokei` JSON snapshot but with explicit curated file assignment aimed at trim planning rather than broad path prefixes.

## Traceability

- Bead: `yazelix-mdqo`

- Bead: `yazelix-mdqo`
