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
| yzxterm emoji fallback presets and bundled font paths | `yazelix-terminal` child package |
| yzxterm shader asset layout and ABI | `yazelix-terminal` child package |
| yzxterm package metadata schema and values | `yazelix-terminal` child package |
| runtime variant selection | main Yazelix Nix package builders |
| Home Manager terminal selection | main Yazelix Home Manager module |
| Home Manager yzxterm package override | main Yazelix Home Manager module, yzxterm-only |
| runtime identity enrichment from yzxterm metadata | main Yazelix runtime package assembly |
| generic terminal materialization requests | main Yazelix Rust runtime control plane |
| yzxterm generated-config adapter for stable Yazelix inputs | main Yazelix Rust runtime control plane, yzxterm-only branch |

## Contract

- A yzxterm runtime package must receive a terminal package that exposes
  `passthru.yzxtermPackageMetadata`
- The metadata must include schema version, package name, package profile,
  checked/release status, metadata path, wrapper commands, config roots,
  supported emoji fallback presets, and the emoji-font wrapper env name
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
- Main Yazelix may read the selected packaged yzxterm profile TOML as a
  template, validate its table shape, and apply stable Yazelix inputs to it.
  This is a yzxterm-only generated-config adapter, not a generic Rio config
  owner and not package identity inference
- Main Yazelix stable inputs for that adapter are limited to the selected
  yzxterm profile, selected `terminal.emoji_style` yzxterm emoji fallback preset, terminal order,
  runtime and state directories, terminal transparency, active cursor color,
  and generated cursor shader snapshot paths
- The child package remains the owner of profile template roots, wrapper
  behavior, emoji font fallback roots, shader ABI, shader file layout, and the
  meaning of package metadata
- If the child package later exposes a stable config-composition API, it may
  replace the main-side adapter. Until then, the main-side adapter is the
  supported boundary and must fail clearly when packaged profile TOML is missing
  or malformed

## Materialization Inventory

| Terminal | Main Yazelix materialization role | Terminal/package-owned role |
| --- | --- | --- |
| Ghostty | writes generated Ghostty config, cursor palette includes, and shader references from Yazelix cursor state | Ghostty owns config semantics and shader runtime behavior |
| Kitty | writes generated Kitty config and optional user override include | Kitty owns config semantics and terminal-native cursor behavior |
| WezTerm | writes generated `.wezterm.lua` from stable Yazelix settings | WezTerm owns Lua config semantics |
| Rio | writes generated upstream Rio config from stable Yazelix settings | Rio owns config semantics |
| Ratty | writes generated Ratty config and launch argv | Ratty owns config semantics and RGP/GPU behavior |
| Foot | writes generated Linux-only `foot.ini` | Foot owns config semantics |
| yzxterm | reads the selected child-owned profile and emoji-fallback template, then applies stable Yazelix transparency, cursor color, and generated shader snapshot paths | `yazelix-terminal` owns wrapper behavior, profile templates, emoji fallback presets, shader ABI, shader asset layout, and package metadata |

## Verification

- `yzx_repo_validator validate-nix-customization-api`
- `yzx_repo_validator validate-config-surface-contract`
- focused Nix eval of yzxterm runtime-tool registry metadata handling
- focused Home Manager eval for yzxterm package override assertions and extra
  terminal launcher behavior
- `yzx dev rust test terminal_materialization` when Rust yzxterm
  materialization logic changes
- `yzx_repo_validator validate-contracts` when this boundary text changes
