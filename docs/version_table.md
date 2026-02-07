# Yazelix Tool Versions

Generated: 2026-02-07 17:21:22

| tool      | locked                               | runtime       |
| --------- | ------------------------------------ | ------------- |
| yazi      | nixos/nixpkgs@nixos-unstable@00c21e4 | 26.1.22       |
| zellij    | nixos/nixpkgs@nixos-unstable@00c21e4 | 0.43.1        |
| helix     | helix-editor/helix@74075bb           | 25.07.1       |
| nushell   | nixos/nixpkgs@nixos-unstable@00c21e4 | 0.110.0       |
| zoxide    | nixos/nixpkgs@nixos-unstable@00c21e4 | 0.9.9         |
| starship  | nixos/nixpkgs@nixos-unstable@00c21e4 | 1.24.2        |
| lazygit   | nixos/nixpkgs@nixos-unstable@00c21e4 | 0.58.1        |
| fzf       | nixos/nixpkgs@nixos-unstable@00c21e4 | 0.67.0        |
| wezterm   | nixos/nixpkgs@nixos-unstable@00c21e4 | not installed |
| ghostty   | nixos/nixpkgs@nixos-unstable@00c21e4 | 1.2.3         |
| nix       | nixos/nixpkgs@nixos-unstable@00c21e4 | 2.33.1        |
| devenv    | cachix/devenv@435d827                | 1.11.2        |
| kitty     | nixos/nixpkgs@nixos-unstable@00c21e4 | 0.45.0        |
| foot      | nixos/nixpkgs@nixos-unstable@00c21e4 | 1.16.2        |
| alacritty | nixos/nixpkgs@nixos-unstable@00c21e4 | 0.13.2        |
| macchina  | nixos/nixpkgs@nixos-unstable@00c21e4 | 6.4.0         |

## Usage

- **Regenerate**: `nu nushell/scripts/utils/version_info.nu --save`
- **Locked**: Flake input revisions when available (nix uses nixpkgs)
- **Runtime**: Versions resolved from current PATH