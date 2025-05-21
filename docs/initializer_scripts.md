# Initializer Scripts

Yazelix generates Nushell initializer scripts in `~/.config/yazelix/nushell/initializers/` during the Nix environment setup (`nix develop --impure`):
- `mise_init.nu`: Runs `mise activate nu` (only if `include_optional_deps = true` in `yazelix.toml`).
- `starship_init.nu`: Runs `starship init nu`.
- `zoxide_init.nu`: Runs `zoxide init nushell --cmd z`.
These are sourced in `~/.config/yazelix/nushell/config/config.nu` and **regenerated each time you open WezTerm** to reflect the current tool versions. Do not edit these files manually, as they will be overwritten. For custom configurations, use `~/.config/nushell/config.nu` or tool-specific configs (e.g., `~/.config/starship.toml`).

For Cargo-based setups with Bash/Zsh, manually generate equivalent scripts:
```bash
mise activate bash > ~/.config/yazelix/nushell/initializers/mise_init.bash
starship init bash > ~/.config/yazelix/nushell/initializers/starship_init.bash
zoxide init bash --cmd z > ~/.config/yazelix/nushell/initializers/zoxide_init.bash
source ~/.config/yazelix/nushell/initializers/mise_init.bash
source ~/.config/yazelix/nushell/initializers/starship_init.bash
source ~/.config/yazelix/nushell/initializers/zoxide_init.bash
```
