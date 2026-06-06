# Yzxterm Package Boundary

## Summary

Yazelix Terminal is a child-owned Rio-derived terminal package. Main Yazelix may
select that package, include it in one terminal runtime variant, expose a narrow
Home Manager override for dogfooding, and consume stable package metadata. Main
Yazelix must not parse yzxterm shader files, infer package profiles from store
names, or make other terminal variants depend on yzxterm internals.

## Ownership

| Concern | Owner |
| --- | --- |
| yzxterm binary wrapper behavior | `yazelix-terminal` child package |
| yzxterm profile config templates | `yazelix-terminal` child package |
| yzxterm shader asset layout and ABI | `yazelix-terminal` child package |
| yzxterm package metadata schema and values | `yazelix-terminal` child package |
| runtime variant selection | main Yazelix Nix package builders |
| Home Manager terminal selection | main Yazelix Home Manager module |
| Home Manager yzxterm package override | main Yazelix Home Manager module, yzxterm-only |
| runtime identity enrichment from yzxterm metadata | main Yazelix runtime package assembly |
| generic terminal materialization requests | main Yazelix Rust runtime control plane |

## Contract

- A yzxterm runtime package must receive a terminal package that exposes
  `passthru.yzxtermPackageMetadata`
- The metadata must include schema version, package name, package profile,
  checked/release status, metadata path, wrapper commands, and config roots
- Main Yazelix derives the yzxterm launch command and runtime identity package
  fields from that metadata, not from terminal package names or child config
  files
- `programs.yazelix.yzxterm_package` overrides only the yzxterm child package.
  It must not require replacing `programs.yazelix.package`
- The override is invalid unless the active terminal or an extra terminal
  launcher includes `yzxterm`
- Non-yzxterm terminal variants must ignore the yzxterm override and must not
  require yzxterm metadata
- Fast/local yzxterm packages must be visibly detectable through
  `runtime_identity.json`; release packages must also report whether the child
  package was checked

## Verification

- `yzx_repo_validator validate-nix-customization-api`
- `yzx_repo_validator validate-config-surface-contract`
- focused Nix eval of yzxterm runtime-tool registry metadata handling
- focused Home Manager eval for yzxterm package override assertions and extra
  terminal launcher behavior
- `yzx dev rust test terminal_materialization` when Rust yzxterm
  materialization logic changes
