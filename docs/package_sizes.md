# Package Sizes & Runtime Surface

The current trimmed line no longer exposes dependency-group toggles like `recommended_deps`, `yazi_extensions`, or `yazi_media`. The packaged runtime ships a fixed tool stack instead of a user-managed package graph.

## What The Runtime Ships

The current runtime includes:
- the core stack: `nu`, `bash`, `fish`, `zsh`, `zellij`, `yazi`, `helix`, `neovim`
- helper tools: `fzf`, `zoxide`, `starship`, `lazygit`, `carapace`, `macchina`, `mise`, `taplo`
- preview/search helpers: `p7zip`, `jq`, `fd`, `ripgrep`, `poppler`
- system helpers required by the runtime wrappers and validators: `git`, `nix`, `coreutils`, `findutils`, `gnugrep`, `gnused`, `util-linux`
- one packaged terminal variant: WezTerm in the Linux `#yazelix` default and `#yazelix_wezterm`, or Ghostty in the macOS `#yazelix` default and `#yazelix_ghostty`

It does not ship:
- a runtime-local `devenv` binary
- pack-driven optional dependency groups
- heavy media helpers as a user-toggleable surface
- the non-selected terminal variant

## Measuring The Current Build

Use `nix path-info -S` on the actual package outputs you care about:

```bash
nix path-info -S .#runtime --extra-experimental-features "nix-command flakes"
nix path-info -S .#runtime_wezterm --extra-experimental-features "nix-command flakes"
nix path-info -S .#yazelix --extra-experimental-features "nix-command flakes"
nix path-info -S .#yazelix_wezterm --extra-experimental-features "nix-command flakes"
```

That gives you the current store size for the exact runtime/package shape on your machine and channel.
