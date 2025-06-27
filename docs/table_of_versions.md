# Yazelix Version Information

> **Note**: This file is now generated dynamically! 
> 
> To get current version information for your Yazelix installation, run:
> ```bash
> nu ~/.config/yazelix/nushell/scripts/generate-version-table.nu
> ```
> 
> Or to update this file:
> ```bash
> nu ~/.config/yazelix/nushell/scripts/generate-version-table.nu --save
> ```

Since Yazelix v7 uses Nix for dependency management, most tool versions are automatically coordinated by `flake.nix`. The only external dependencies you need to manage are:

- **WezTerm** (required terminal emulator)
- **Nix** (for dependency management)
- Your operating system

All other tools (Zellij, Yazi, Helix, Nushell, etc.) are managed by Nix and their versions are guaranteed to be compatible with each other.

## Last Static Version Table (Deprecated)

The information below is from the last manual update and may be outdated:

| Component          | Version                  |
|--------------------|--------------------------|
| OS                 | Pop!_OS 24.04            |
| DE                 | COSMIC                   |
| Zellij             | 0.42.1                   |
| Helix (from source)| helix 25.01.1 (0efa8207) |
| Nushell            | 0.104.0                  |
| Zoxide             | 0.9.7                    |
| Yazi               | 25.4.8                   |
| WezTerm            | 20240203-110809-5046fc22 |
| Ghostty            | 1.1.2                    |
| ya (from yazi-cli) | 25.4.8                   |
| lazygit            | 0.44.1                   |
| Starship           | 1.20.1                   |
