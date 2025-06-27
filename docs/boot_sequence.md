# Boot Sequence

The boot sequence of the Nix version is the following:

1. **Launch**: You run `yazelix` or `yzx` (or `~/.config/yazelix/bash/launch-yazelix.sh`)
   - The `launch-yazelix.sh` script automatically adds `yazelix` and `yzx` aliases to your shell configuration (e.g., `~/.bashrc` or `~/.zshrc`) and launches WezTerm with the Yazelix-specific configuration.

2. **WezTerm Start**: WezTerm, as configured by `~/.config/yazelix/terminal_configs/wezterm_nix/.wezterm.lua`, then executes the `~/.config/yazelix/bash/start-yazelix.sh` script.

3. **Nix Environment**: The `start-yazelix.sh` script navigates to the Yazelix project directory and runs `nix develop --impure --command ...`.

4. **Inside Nix Environment**:
   - The `flake.nix` reads `~/.config/yazelix/yazelix.nix` to determine configurations, including the `default_shell` (which defaults to `nu` but can be set to `bash` or `fish`).
   - Dependencies are installed.
   - The `shellHook` generates initializer scripts for Bash and Nushell, and exports the chosen default shell as an environment variable (`YAZELIX_DEFAULT_SHELL`). Fish users get access to all tools via PATH and can configure them manually.
   - Finally, Zellij is launched using the `YAZELIX_DEFAULT_SHELL` to set its default shell (e.g., `zellij --default-shell nu`). 