# Package Sizes & Runtime Surface

The packaged runtime stays batteries-included by default, with granular storage controls exposed through Home Manager and `lib.${system}.mkYazelix` instead of a large public package matrix.

## What The Runtime Ships

The default runtime includes:

- the core stack: `nu`, `bash`, `fish`, `zsh`, `zellij`, `yazi`, `helix`, `neovim`
- Helix Steel authoring tools: `steel`, `steel-language-server`, `forge`, `cargo-steel-lib`, `repl-connect`
- helper tools: `fzf`, `zoxide`, `starship`, `lazygit`, `zenith`, `carapace`, `macchina`
- host-managed helper integrations: `mise`, `tombi`
- preview/search helpers: `p7zip`, `jq`, `fd`, `ripgrep`, `poppler`, `resvg`
- system helpers required by runtime wrappers and validators: `git`, `nix`, `coreutils`, `findutils`, `gnugrep`, `gnused`, `util-linux`
- one packaged terminal variant: Ghostty in `#yazelix` and `#yazelix_ghostty`, vanilla Rio in `#yazelix_rio`, WezTerm in `#yazelix_wezterm`, Kitty in `#yazelix_kitty`, Linux Foot in `#yazelix_foot`, or experimental Linux Ratty in `#yazelix_ratty`
- `tokenusage` for the default Codex and Claude status widgets

It does not ship:

- a runtime-local `devenv` binary
- pack-driven optional dependency groups
- every terminal variant in one package
- a public flake package for every possible host/off tool combination

## Runtime Size Reporter

Use the repo-local reporter when investigating package size. It measures an already-realized output by default and prints total closure size, largest individual NAR paths, direct-reference closure sizes, and duplicate store basenames:

```bash
shells/posix/yazelix_runtime_size_report.sh "$(command -v yzx)"
shells/posix/yazelix_runtime_size_report.sh /nix/store/...-yazelix
```

Only use `--build` when the command should first realize a flake output:

```bash
shells/posix/yazelix_runtime_size_report.sh --build .#yazelix
shells/posix/yazelix_runtime_size_report.sh --build .#yazelix_wezterm --top 40 --direct-top 60
```

The reporter depends only on normal maintainer/runtime shell tools: `nix`, `nix-store`, `jq`, `awk`, `sort`, `head`, `wc`, `sed`, `tr`, `readlink`, and `mktemp`. It uses `numfmt` when available.

For a quick total-only check, `nix path-info -S` is still useful:

```bash
nix path-info -S .#yazelix --extra-experimental-features "nix-command flakes"
nix path-info -S .#yazelix_rio --extra-experimental-features "nix-command flakes"
nix path-info -S .#yazelix_wezterm --extra-experimental-features "nix-command flakes"
nix path-info -S .#yazelix_kitty --extra-experimental-features "nix-command flakes"
nix path-info -S .#yazelix_foot --extra-experimental-features "nix-command flakes"
nix path-info -S .#yazelix_ratty --extra-experimental-features "nix-command flakes"
```

## Current x86_64-linux Findings

Measurements below are local NAR/closure measurements from June 2, 2026. They are not exact Cachix billed bytes because Cachix checks the upstream NixOS cache first and uploads compressed paths.

The current `git+file://` default Ghostty package measured:

| Shape | Build target | Closure size | Paths | Notes |
| --- | --- | ---: | ---: | --- |
| Default `#yazelix` | `.#packages.x86_64-linux.yazelix` | 3.0 GiB | 816 | Full runtime with 64-bit-only nixGL wrappers; `mise` and `tombi` are host-sourced by default |
| Lean package-builder profile | `lib.${system}.mkYazelix` | 2.2 GiB | 445 | Host-sources editor/sidebar/helper tools, disables optional helpers, omits cursor and screen components |

The lean profile measurement predates the default host-sourcing of `mise` and `tombi`, but still represents the smaller supported shape for users who want a profile-owned runtime. It includes about 1.1 GiB of Linux `nixGLMesa` closure, so graphics wrapper ownership remains a major remaining Linux storage question.

Largest default direct references by closure:

| Direct reference | Closure |
| --- | ---: |
| runtime tree | 3.1 GiB |
| Ghostty | 1.1 GiB |
| `nixGLMesa` | 1.1 GiB |
| Yazi wrapper | 504 MiB |
| `git` | 374 MiB |
| Yazelix Helix | 328 MiB |
| `fish` | 302 MiB |
| `neovim` | 254 MiB |

`mise` and `tombi` no longer appear as direct references in the measured default runtime because their default source mode is `host`.

Disabling 32-bit nixGL support removes the large duplicate Mesa/LLVM families from the default package closure. The remaining duplicate store basenames in the measured default package are small compared with the old 32-bit wrapper duplication.

## Linux Graphics Wrappers

The Linux runtime registry currently imports `nixGL` when the platform is Linux and the flake input is present, with `enable32bits = false` and `enableIntelX86Extensions = false`. It adds 64-bit-only `nixgl_mesa` for Linux runtimes and 64-bit-only `nixvulkan_mesa` for Ratty package variants.

Current launch behavior:

- Ghostty, WezTerm, Kitty, and Ratty launch commands may prepend a graphics wrapper
- Ratty prefers the Vulkan wrapper because its renderer needs a Vulkan-capable adapter

The option surface should be explicit instead of hidden behind `runtime_tool_sources`: a future package option should choose a graphics wrapper source such as bundled, host, or none, with launch rendering and doctor diagnostics tested per terminal variant. Default behavior should not change until desktop launch reliability is preserved.

## Yazi Package Shape

The runtime includes both a Yazi wrapper path and a Yazi unwrapped path. This is the normal nixpkgs Yazi package shape, not accidental duplicate ownership.

Observed current shape:

- `libexec/yazi` points to the wrapped Yazi package
- `libexec/ya` points to the unwrapped Yazi helper package
- the wrapper references the unwrapped binary plus preview helpers such as `ffmpeg-headless`, `file`, `chafa`, `jq`, `poppler-utils`, `imagemagick`, and `fd`
- a base nixpkgs Yazi package from the same flake input also has wrapper plus unwrapped paths

There is no obvious safe deletion in the wrapper/unwrapped pair while preserving upstream Yazi image-preview behavior. Users who prefer storage savings over runtime-packaged Yazi can set `runtime_tool_sources.yazi = "host"` and then rely on host Yazi support.

## Lean Home Manager Profile

Home Manager is the recommended storage-saving surface. See [home_manager/README.md](../home_manager/README.md#lean-runtime-profile) for the exact profile.

The measured lean profile host-sources large leaf tools, disables optional helpers, removes cursor/screen components, and omits `tokenusage` after removing Codex/Claude widgets from the status tray. Feature losses are explicit:

- host-sourced commands must exist on the inherited `PATH`
- host Yazi may lose Yazelix's bundled KGP preview behavior
- host Helix may not match the Yazelix Steel fork behavior
- disabled `steel` removes Steel authoring commands
- disabled `p7zip`, `poppler`, and `resvg` reduce archive/PDF/SVG preview helpers
- disabled `screen` removes `yzx screen` and requires skipping welcome/screen-saver behavior
- disabled `cursors` removes Yazelix cursor shader assets and hides cursor fields from the config UI
- `agent_usage_programs = []` is only correct when Codex and Claude usage widgets are removed or intentionally host-provided

## Cachix Publish Size

The publish workflow builds selected `x86_64-linux` outputs. Local incremental NAR measurements in workflow order:

| Output | Closure | Incremental unique NAR | Decision |
| --- | ---: | ---: | --- |
| `yazelix_kgp_zellij` | 102 MiB | 56 MiB | Keep publishing explicitly for the expensive KGP Zellij output |
| `yazelix_helix` | 328 MiB | 282 MiB | Keep publishing explicitly for the expensive Helix fork output |
| `yazelix` | 3.1 GiB | 2.7 GiB | Keep: main supported install path |
| `yazelix_wezterm` | 2.8 GiB | 232 MiB | Keep: supported alternate runtime with real unique closure |

The previous workflow also listed `yazelix_ghostty`, `yazelix_agent_tools`, and `yazelix_screen`; those measured as zero incremental unique NAR in workflow order and are no longer explicit publish targets. Further storage relief should come from shrinking the default runtime closure, especially host-tool-manager references, and from Cachix retention policy.

The publish workflow also builds a selective `aarch64-darwin` lane for macOS users:

- `yazelix_kgp_zellij`
- `yazelix_helix`
- `yazelix`

The Darwin lane intentionally starts with the default supported install path and expensive editor/Zellij package outputs rather than every terminal variant or dev/check output. Expand it only after measuring runner time, disk pressure, and Cachix storage churn.

Cachix-side policy from the current docs:

- [pushing runtime closure](https://docs.cachix.org/pushing#pushing-runtime-closure) is the documented flake path for selected packages
- [garbage collection](https://docs.cachix.org/garbage-collection) first checks the upstream NixOS cache when pushing, then deletes oldest paths by last access, or creation date when never accessed, once the cache reaches its storage limit
- [pins](https://docs.cachix.org/pins) can protect important release paths from garbage collection and can retain only the last N revisions or last X days

Use pins for deliberate release retention and leave day-to-day CI outputs to normal Cachix garbage collection unless a release needs a public long-lived substitute.
