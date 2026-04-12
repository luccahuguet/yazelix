# Yazelix Subsystem Code Inventory

This document is the current code-only LOC snapshot for the subsystem families defined in [Architecture Map](./architecture_map.md).

The important rule for this inventory is simple: each counted source file belongs to exactly one subsystem family. This avoids the drift and double-counting problems that happen when broad directories and conceptual layers are mixed loosely.

## Snapshot

- Snapshot date: `2026-04-12`
- Metric: `tokei` `Code` column only
- Scope: tracked repo code with Markdown excluded
- Excludes: docs prose, Markdown code fences, local scratch state, build outputs, `.git`, `.beads`, and non-source binary assets

## Inventory Buckets

| Subsystem family | Files | Code LOC | Share | Counted source roots |
| --- | ---: | ---: | ---: | --- |
| Runtime control plane and command surface | 88 | 14,736 | 40.9% | `nushell/scripts/core`, `nushell/scripts/setup`, `nushell/scripts/utils`, `nushell/scripts/yzx` |
| Maintainer workflow and validation | 48 | 11,125 | 30.9% | `nushell/scripts/dev`, `.github`, `maintainer_shell.nix`, `.nu-lint.toml` |
| Shipped runtime data and assets | 78 | 5,949 | 16.5% | `configs`, `config_metadata`, `user_configs`, `assets`, `nushell/config`, `.taplo.toml`, `yazelix_default.toml`, `docs/upgrade_notes.toml` |
| Workspace session orchestration | 34 | 3,209 | 8.9% | `nushell/scripts/integrations`, `nushell/scripts/zellij_wrappers`, `rust_plugins/` |
| Distribution and host integration | 20 | 1,002 | 2.8% | `home_manager`, `packaging`, `shells`, `flake.nix`, `yazelix_package.nix`, `yazelix_runtime_package.nix` |

Covered by this inventory: `36,021` code lines.

For reference, a raw `tokei .` snapshot currently reports `36,544` code lines. The extra `523` lines come from embedded code blocks inside Markdown, which this inventory intentionally excludes.

## What The Numbers Mean

- The runtime control plane is still the largest single subsystem. If Yazelix feels too heavy, the main slimming budget is still there.
- Maintainer workflow and validation is still almost one third of the repo. The project has a real maintenance and verification surface, not just shipped product logic.
- Shipped runtime data and assets are a substantial subsystem on their own. Yazelix carries meaningful tracked product logic in TOML, Lua, GLSL, shell config, and release metadata.
- Workspace session orchestration is smaller than many people expect. Many workspace bugs come from ownership seams, not from overwhelming raw workspace LOC.
- Distribution and host integration is now comparatively small. That is a healthy sign for the v15 trim: packaging and launcher adaptation no longer dominate the repo shape.

## Counting Rules

The bucket model follows the current architecture map exactly:

1. Runtime control plane and command surface
2. Workspace session orchestration
3. Distribution and host integration
4. Shipped runtime data and assets
5. Maintainer workflow and validation

Each source file is assigned to exactly one of those five families. Top-level files like `maintainer_shell.nix`, `.taplo.toml`, `yazelix_default.toml`, and `docs/upgrade_notes.toml` are assigned explicitly so the totals still reconcile.

## Reproduce The Snapshot

Use file-level assignment instead of ad hoc directory totals. The following script reproduces the exact current bucket totals:

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

## Traceability

- Bead: `yazelix-mtqj`
