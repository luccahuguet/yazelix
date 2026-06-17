# Mars Package Boundary

## Summary

Mars is a child-owned Rio-derived terminal package. Main Yazelix may select that
package, include it in one terminal runtime variant, expose a narrow Home
Manager override for dogfooding, and consume stable package metadata. Main
Yazelix must not parse Mars shader files, infer package profiles from store
names, or make other terminal variants depend on Mars internals.

## Ownership

| Concern | Owner |
| --- | --- |
| Mars binary wrapper behavior | `yazelix-terminal` child package |
| Mars profile config templates | `yazelix-terminal` child package |
| Mars emoji fallback presets and bundled font paths | `yazelix-terminal` child package |
| Mars shader asset layout and ABI | `yazelix-terminal` child package |
| Mars dark/light themes and adaptive appearance behavior | `yazelix-terminal` child package |
| Mars package metadata schema and values | `yazelix-terminal` child package |
| runtime variant selection | main Yazelix Nix package builders |
| Home Manager terminal selection | main Yazelix Home Manager module |
| Home Manager yzxterm package override | main Yazelix Home Manager module, Mars-only |
| runtime identity enrichment from Mars metadata | main Yazelix runtime package assembly |
| generic terminal materialization requests | main Yazelix Rust runtime control plane |
| Mars generated-config adapter for stable Yazelix inputs | main Yazelix Rust runtime control plane, yzxterm-only branch |

## Contract

- A yzxterm runtime package must receive a terminal package that exposes
  `passthru.marsPackageMetadata`
- The metadata must include schema version, package name, package profile,
  checked/release status, metadata path, wrapper commands, config roots,
  supported emoji fallback presets, supported appearance modes, default
  appearance mode, the appearance wrapper env name, and the emoji-font wrapper
  env name
- Main Yazelix derives the Mars launch command and runtime identity package
  fields from that metadata, not from terminal package names or child config
  files
- Main Yazelix requires Mars metadata to advertise `dark`, `light`, and `auto`
  appearance support before exposing Mars as a first-class target for
  global `appearance.mode`
- `programs.yazelix.yzxterm_package` overrides only the Mars child package.
  It must not require replacing `programs.yazelix.package`
- The override is invalid unless the active terminal or an extra terminal
  launcher includes `yzxterm`
- Non-yzxterm terminal variants must ignore the yzxterm override and must not
  require Mars metadata
- Fast/local Mars packages must be visibly detectable through
  `runtime_identity.json`; release packages must also report whether the child
  package was checked
- Main Yazelix may read the selected packaged Mars profile TOML as a
  template, validate its table shape, and apply stable Yazelix inputs to it.
  This is a Mars-only generated-config adapter, not a generic Rio config
  owner and not package identity inference
- Main Yazelix stable inputs for that adapter are limited to the selected Mars
  profile, selected `terminal.emoji_style` Mars emoji fallback preset, terminal
  order, runtime and state directories, terminal transparency, global
  appearance mode, Mars cell-opacity policy, active cursor color, and generated
  cursor shader snapshot paths
- The child package remains the owner of profile template roots, wrapper
  behavior, emoji font fallback roots, dark/light themes, adaptive appearance
  behavior, shader ABI, shader file layout, and the meaning of package metadata
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
| yzxterm | reads the selected child-owned Mars profile and emoji-fallback template, copies child-owned dark/light themes into the generated config root, then applies stable Yazelix transparency, appearance selection, cell-opacity policy, cursor color, and generated shader snapshot paths | `yazelix-terminal` owns wrapper behavior, profile templates, emoji fallback presets, dark/light theme palettes, adaptive appearance behavior, shader ABI, shader asset layout, and package metadata |

## Verification

- `yzx_repo_validator validate-nix-customization-api`
- `yzx_repo_validator validate-config-surface-contract`
- focused Nix eval of yzxterm runtime-tool registry metadata handling
- focused Home Manager eval for yzxterm package override assertions and extra
  terminal launcher behavior
- `yzx dev rust test terminal_materialization` when Rust yzxterm
  materialization logic changes
- `yzx_repo_validator validate-contracts` when this boundary text changes
