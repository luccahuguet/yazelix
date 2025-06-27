# Boot Sequence


## 1.A: WezTerm Auto-Launch 
- **Setup**: Copy WezTerm config: `cp ~/.config/yazelix/terminal_configs/wezterm_nix/.wezterm.lua ~/.wezterm.lua`
- **Launch**: Open WezTerm and WezTerm's `default_prog` automatically executes `bash -c ~/.config/yazelix/bash/start-yazelix.sh`

## 1.B: Terminal Commands  
- **Setup**: See [Terminal Setup Guide](./terminal_setup.md) for `yazelix` and `yzx` alias configuration
- **Launch**: Run `yazelix` or `yzx` from any terminal and it will execute `~/.config/yazelix/bash/launch-yazelix.sh`
- That launches WezTerm with specific config: `nohup wezterm --config-file ~/.config/yazelix/terminal_configs/wezterm_nix/.wezterm.lua`
- WezTerm's `default_prog` automatically executes `bash -c ~/.config/yazelix/bash/start-yazelix.sh`

## 2. **Nix Environment**: Changes to `~/.config/yazelix` and runs:
   ```bash
   nix develop --impure --command bash -c "zellij --config-dir ~/.config/yazelix/zellij options --default-cwd $HOME --default-layout yazelix --default-shell $YAZELIX_DEFAULT_SHELL"
   ```

## 3. **Nix Dependencies**: 
   - Reads `~/.config/yazelix/yazelix.nix` configuration (creates from `yazelix_default.nix` if missing)
   - Installs dependencies based on config flags
   - Sets environment variables (`YAZELIX_DIR`, `YAZELIX_DEFAULT_SHELL`, etc.)

## 4. **shellHook Execution**: Nix shellHook runs `nushell/scripts/setup/environment.nu`:
   - **Initializer Generation**: Creates shell initializers for Nu, Bash, Fish, Zsh  
   - **Shell Configuration**: Adds Yazelix config sourcing to user shell configs
   - **Editor Setup**: Sets `EDITOR` to `helix` or `hx`
   - **Permissions**: Makes scripts executable
   - **Logging**: Creates timestamped logs and auto-trims old ones

## 5. **Zellij Launch**: Starts Zellij with yazelix layout and configured default shell 