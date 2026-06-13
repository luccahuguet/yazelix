# Initializer Scripts

Yazelix generates shell initializer scripts under `~/.local/share/yazelix/initializers/` during environment setup.

Generated shell directories:
- `nushell/`
- `bash/`
- `fish/`
- `zsh/`
- `xonsh/`

Each directory gets a `yazelix_init.*` aggregate initializer and any available tool initializers:
- `starship_init.*`: runs `starship init <shell>`
- `zoxide_init.*`: runs `zoxide init <shell>`
- `atuin_init.*`: runs `atuin init <shell>` when host `atuin` is available
- `mise_init.*`: runs `mise activate <shell>` when host `mise` is available
- `carapace_init.*`: runs `carapace _carapace <shell>` when `carapace` is available

Nushell uses the upstream `nushell` shell name for zoxide and carapace, so its generated commands use `zoxide init nushell` and `carapace _carapace nushell`.

The shipped managed Yazelix Bash, Fish, Zsh, and Nushell startup files source their matching aggregate initializers when those shells run inside Yazelix. Xonsh is host-owned: Yazelix accepts `shell.default_shell = "xonsh"` and generates `xonsh/yazelix_init.xsh` plus `~/.config/yazelix/shell_xonsh.xsh`, but it does not install xonsh. The host must provide `xonsh` on `PATH`. To use the generated xonsh initializers from native xonsh startup, source `~/.config/yazelix/shell_xonsh.xsh` from `~/.xonshrc` or `~/.config/xonsh/rc.xsh`.

These files are regenerated whenever Yazelix refreshes its managed runtime state or Home Manager activates the runtime. Do not edit files under `~/.local/share/yazelix/initializers/` manually; use Yazelix sidecars under `~/.config/yazelix/` or tool-specific configs such as `~/.config/starship.toml` for host-owned customization.
