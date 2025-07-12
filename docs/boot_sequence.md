# Boot Sequence


## 1.A: Terminal Auto-Launch 
- **Setup**: Copy terminal config:
  - **Ghostty**: `cp ~/.config/yazelix/terminal_configs/ghostty/config ~/.config/ghostty/config`
  - **WezTerm**: `cp ~/.config/yazelix/terminal_configs/wezterm/.wezterm.lua ~/.wezterm.lua`
- **Launch**: Open your terminal and it automatically executes `bash -c ~/.config/yazelix/bash/start_yazelix.sh`

## 1.B: Terminal Commands  
- **Setup**: `yazelix` and `yzx` aliases are automatically available when shell configs are sourced
- **Launch**: Run `yazelix` or `yzx` from any terminal and it will execute `~/.config/yazelix/nushell/scripts/launch_yazelix.nu`
- That launches your preferred terminal (WezTerm by default, or Ghostty) with specific config
- The terminal automatically executes `nu ~/.config/yazelix/nushell/scripts/start_yazelix.nu`

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
   - **Shell Configuration**: Adds Yazelix config sourcing to user shell configs with managed sections
   - **Editor Setup**: Sets `EDITOR` to `hx`
   - **Permissions**: Makes scripts executable
   - **Logging**: Creates timestamped logs and auto-trims old ones

## 5. **Zellij Launch**: Starts Zellij with yazelix layout and configured default shell

## Configuration Management
Yazelix now uses managed configuration sections in user shell configs:
- **Start/End Markers**: `# YAZELIX START/END` comments clearly mark yazelix-managed sections
- **Configuration Commands**: Use `yazelix get_config`, `yazelix check_config`, or `yazelix config_status` to check configurations
- **Extract Sections**: Use `yazelix extract_config <shell>` to view specific shell configurations
- **Future Features**: Enables automatic updates and version compatibility warnings
- **User Safety**: Clear markers prevent accidental modification of yazelix-managed sections 