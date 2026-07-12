# Package Sizes & Runtime Surface

The complete `#yazelix` package is intentionally batteries-included. The main flake does not expose a granular package builder, runtime-only package, or component matrix.

## What The Runtime Ships

The default runtime includes:

- the core stack: `nu`, `bash`, `fish`, `zsh`, `zellij`, `yazi`, `helix`, `neovim`
- Helix Steel authoring tools: `steel`, `steel-language-server`, `forge`, `cargo-steel-lib`, `repl-connect`
- helper tools: `fzf`, `zoxide`, `starship`, `lazygit`, `zenith`, `carapace`, `macchina`
- host-managed helper integrations: `mise`, `tombi`
- preview/search helpers: `p7zip`, `jq`, `fd`, `ripgrep`, `poppler`, `resvg`
- system helpers required by runtime wrappers and validators: `git`, `nix`, `coreutils`, `findutils`, `gnugrep`, `gnused`, `util-linux`
- one packaged terminal: Mars in `#yazelix`
- `tokenusage` for the default Codex and Claude status widgets

It does not ship:

- a runtime-local `devenv` binary
- pack-driven optional dependency groups
- every host terminal in one package
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
```

The reporter depends only on normal maintainer/runtime shell tools: `nix`, `nix-store`, `jq`, `awk`, `sort`, `head`, `wc`, `sed`, `tr`, `readlink`, and `mktemp`. It uses `numfmt` when available.

For a quick total-only check, `nix path-info -S` is still useful:

```bash
nix path-info -S .#yazelix --extra-experimental-features "nix-command flakes"
```

## Last Recorded x86_64-linux Findings

Measurements below are local NAR/closure measurements from June 2, 2026, before Mars became the default terminal. They are not exact Cachix billed bytes because Cachix checks the upstream NixOS cache first and uploads compressed paths.

The recorded `git+file://` default Ghostty package measured:

| Shape | Build target | Closure size | Paths | Notes |
| --- | --- | ---: | ---: | --- |
| Default `#yazelix` | `.#packages.x86_64-linux.yazelix` | 3.0 GiB | 816 | Full runtime with 64-bit-only nixGL wrappers; `mise` and `tombi` are host-sourced by default |
The retired lean builder measurement is historical evidence rather than a supported package shape. Users who need a different closure must build a complete compatible package outside the main flake.

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

The Linux runtime registry imports `nixGL` when the platform is Linux and the flake input is present, with `enable32bits = false` and `enableIntelX86Extensions = false`. The Mars package path may add a 64-bit-only Vulkan wrapper for the packaged terminal launcher.

Current launch behavior:

- Mars launch commands may prepend a graphics wrapper
- terminals outside packaged Mars keep their own graphics-wrapper policy and run Yazelix with `yzx enter`

Graphics-wrapper selection remains package-owned. Home Manager accepts only a complete package override and does not expose a second wrapper-selection language.

## Yazi Package Shape

The runtime includes both a Yazi wrapper path and a Yazi unwrapped path. This is the normal nixpkgs Yazi package shape, not accidental duplicate ownership.

Observed current shape:

- `libexec/yazi` points to the wrapped Yazi package
- `libexec/ya` points to the unwrapped Yazi helper package
- the wrapper references the unwrapped binary plus preview helpers such as `ffmpeg-headless`, `file`, `chafa`, `jq`, `poppler-utils`, `imagemagick`, and `fd`
- a base nixpkgs Yazi package from the same flake input also has wrapper plus unwrapped paths

There is no obvious safe deletion in the wrapper/unwrapped pair while preserving upstream Yazi image-preview behavior. Home Manager installs the complete package rather than replacing its Yazi dependency independently.

## Home Manager package ownership

Home Manager installs one complete package. Closure-size tradeoffs belong to the selected package, not to Home Manager per-tool or component options.

## Cachix Publish Size

The publish workflow builds the supported `x86_64-linux` product package and representative Home Manager closure. Child dependencies enter the cache through that package graph rather than public main-flake mirrors.

| Output | Closure | Incremental unique NAR | Decision |
| --- | ---: | ---: | --- |
| `yazelix` | 3.1 GiB | 2.7 GiB | Keep: main supported install path |

Further storage relief should come from shrinking the complete runtime closure and from Cachix retention policy, not from multiplying public package shapes.

The Darwin lane builds the same complete product package and representative Home Manager closure. Expand it only after measuring runner time, disk pressure, and Cachix storage churn.

Cachix-side policy from the current docs:

- [pushing runtime closure](https://docs.cachix.org/pushing#pushing-runtime-closure) is the documented flake path for selected packages
- [garbage collection](https://docs.cachix.org/garbage-collection) first checks the upstream NixOS cache when pushing, then deletes oldest paths by last access, or creation date when never accessed, once the cache reaches its storage limit
- [pins](https://docs.cachix.org/pins) can protect important release paths from garbage collection and can retain only the last N revisions or last X days

Use pins for deliberate release retention and leave day-to-day CI outputs to normal Cachix garbage collection unless a release needs a public long-lived substitute.
