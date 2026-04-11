# User Configs

This directory is the Yazelix-owned boundary for user-managed configuration.

Live surfaces:
- `yazelix.toml`
- `terminal/`
- `yazi/`
- `zellij/`

How to reason about this:
- edit files under `user_configs/` when you want to customize a Yazelix-managed tool surface
- Yazelix generates runtime config under `~/.local/share/yazelix/`
- tracked defaults stay at the repo root and are copied into `user_configs/` on first use or reset

Tracked defaults stay at the repo root:
- `yazelix_default.toml`

Generated runtime state belongs in:
- `~/.local/share/yazelix/`
