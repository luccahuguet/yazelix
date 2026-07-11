# Active Runtime Identity Contract

## Summary

Yazelix status and doctor surfaces must explain which runtime is active, who owns updates, and which generated-state surfaces were produced by that runtime.

## Behavior

- Runtime identity has these fields:
  - `runtime_root`: active `YAZELIX_RUNTIME_DIR`
  - `config_root`: active `YAZELIX_CONFIG_DIR`
  - `state_root`: active `YAZELIX_STATE_DIR`
  - `install_owner`: one of `home-manager`, `profile`, or `manual`
  - `stable_yzx_wrapper`: the profile or manual launcher that future desktop/update commands should target, when known
  - `desktop_launcher_path`: the launcher expected in desktop entries
  - `runtime_variant`: the packaged runtime label when the runtime exposes `runtime_variant`; current Yazelix packages expose `kitty`
  - `session_terminal_label`: the terminal emulator actually hosting the current Yazelix session when known; `yzx enter` derives this from explicit Yazelix session env, managed-launch hints, and conservative terminal-specific host env before falling back to `unknown`
  - `runtime_features`: marker files under `runtime_features/`, such as `zellij_kitty_passthrough`
  - `generated_state_checks`: derived-state checks for layouts, shell initializers, workspace assets, and launch logs
- `runtime_variant` and `session_terminal_label` are separate facts. The packaged runtime label can be `kitty` while `yzx enter` runs inside WezTerm, Ghostty, Foot, Rio, or an unknown host terminal.
- The integrated status bar terminal widget displays the session terminal label, not the packaged runtime label, for current-terminal `yzx enter` sessions.
- Generic terminal identifiers such as `xterm-256color` are not guessed as a specific emulator; unsupported or ambiguous host terminals report `unknown`.
- Home Manager ownership is detected when either:
  - `settings.jsonc` is a Home Manager-owned profile symlink, or
  - the default profile contains a Home Manager path, `~/.nix-profile/bin/yzx`, and the Home Manager profile desktop entry
- `manage_config = false` keeps `settings.jsonc` mutable without changing the install owner away from Home Manager
- Profile ownership means the default Nix profile directly owns a Yazelix package entry
- Manual ownership means no supported profile or Home Manager owner was found
- Owner update commands:
  - Home Manager: `yzx update home_manager`, then the printed `home-manager switch`
  - Profile: `yzx update upstream`
  - Manual: install into a profile or enable Home Manager before relying on update ownership
- Generated shell initializers must not retain deleted transient `result*` runtime paths
- Home Manager activation regenerates shell initializers from the active Home Manager runtime after linking the generation
- Doctor reports deleted transient initializer references with a clear regeneration command

## Non-Goals

- making status or doctor infer release notes without a known runtime identity
- treating a live shell `PATH` entry as install ownership by itself
- silently moving user config files between native app config roots and Yazelix-managed roots
- making Home Manager own `settings.jsonc` when `manage_config = false`

## Verification

- `yzx dev rust test install_ownership_report::tests::evaluate_install_ownership_detects_home_manager_profile_without_managed_config`
- `yzx dev rust test doctor_runtime_report::tests::shell_initializer_finding_warns_on_deleted_transient_runtime_path`
- `yzx dev rust test launch_commands::tests::desktop_deferred_launch_helper_records_lifetime_status`
- `yzx dev rust test launch_commands::tests::launch_probe_log_path_uses_command_basename`
- `yzx dev rust test terminal_variant::tests::detects_session_terminal_from_explicit_and_common_host_env`
- `yzx dev rust test --test yzx_control_runtime_surface yzx_enter_uses_detected_host_terminal_for_status_bar_label`
- `yzx_repo_validator validate-contracts`

## Traceability

- Defended by: `rust_core/yazelix_core/src/install_ownership_report.rs`
- Defended by: `rust_core/yazelix_core/src/doctor_runtime_report.rs`
- Defended by: `shells/posix/desktop_deferred_launch_probe.sh`
