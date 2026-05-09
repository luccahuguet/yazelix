# Zellij bar rename compatibility

The standalone Zellij bar package is renamed from `yazelix-bar` to `yazelix-zellij-bar`.

The compatibility policy is delete-first:

| Old surface | New surface | Compatibility decision |
| --- | --- | --- |
| GitHub repository `luccahuguet/yazelix-bar` | `luccahuguet/yazelix-zellij-bar` | Rely on GitHub's repository rename redirect |
| Flake input `github:luccahuguet/yazelix-bar` | `github:luccahuguet/yazelix-zellij-bar` | No old input in Yazelix main repo |
| Flake package `yazelix_bar` / `yazelix-bar` | `yazelix_zellij_bar` / `yazelix-zellij-bar` | No compatibility package aliases |
| Rust crate `yazelix_bar` | `yazelix_zellij_bar` | No compatibility crate alias |
| Binary `yazelix_bar_generate` | none | Generator removed; no compatibility binary alias |
| Binary `yazelix_bar_widget` | `yazelix_zellij_bar_widget` | No compatibility binary alias |
| Install path `share/yazelix_bar` | `share/yazelix_zellij_bar` | No compatibility install path |
| Install path `share/doc/yazelix_bar` | `share/doc/yazelix_zellij_bar` | No compatibility install path |
| Config/cache examples under `yazelix_bar` | `yazelix_zellij_bar` | No compatibility example path |

## Rationale

The old names were pushed but are still part of an active extraction/rename sequence, not a stabilized release contract. Keeping aliases would make the standalone story harder to understand and would leave stale command surfaces that conflict with the Zellij plugin naming contract.

The only compatibility retained is GitHub's repository redirect because it is external infrastructure behavior and does not add runtime or package surface area.

## Verification

The rename is complete when audits across the main repo and child repo find no active `yazelix-bar`, `yazelix_bar`, `yazelix_bar_widget`, `yazelix_bar_generate`, or `share/yazelix_bar` surfaces outside this compatibility document and historical records.
