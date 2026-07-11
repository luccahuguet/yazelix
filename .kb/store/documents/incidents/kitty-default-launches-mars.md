---
id: 019f5105-3a93-7261-b6bd-900f0331f544
slug: incidents/kitty-default-launches-mars
title: "Kitty default still launches Mars"
type: incident
status: investigating
priority: high
---

## Symptom

The installed Yazelix runtime reports the `kitty` runtime variant and reads
`/home/flexnetos/.config/yazelix/settings.jsonc`, but the visible/default
desktop launch still opens Mars with Kitty-oriented generated configuration.

## Expected behavior

Kitty is the selected default terminal. The profile-owned desktop launcher,
runtime package variant, generated terminal configuration, and launched
terminal process must all agree on Kitty.

## Actual behavior

Mars remains the effective default terminal despite the runtime reporting the
Kitty variant.

## Investigation scope

- Prove the active profile frontdoor and immutable package runtime root.
- Inspect the user config without editing generated state.
- Trace desktop entry selection and launch argv to the owning source.
- Distinguish the current live session from next-launch behavior.
- Repair the owning config/package/desktop generation path rather than patching
  `/home/flexnetos/.local/share/yazelix` by hand.

## Acceptance criteria

- [ ] `yzx inspect --json` reports a coherent Kitty runtime and config.
- [ ] The visible/default desktop entry launches the profile-owned `yzx`.
- [ ] The resulting terminal process is Kitty, not Mars.
- [ ] No stale Mars or duplicate desktop entry shadows the selected launcher.
- [ ] Source/package tests cover the regression if a product defect is found.
- [ ] A fresh user-launched window confirms the fix before any non-trivial push.

## Initial evidence

- Profile frontdoor: `/home/flexnetos/.nix-profile/bin/yzx`
- Reported runtime: `v17.9`, variant `kitty`
- Reported config: `/home/flexnetos/.config/yazelix/settings.jsonc`
- Reported install owner: default Nix profile

## Findings

- The profile runtime and `yzx status --json` correctly selected Kitty.
- `launch_materialization.rs` still sent every active terminal through
  `generate_terminal_materialization`, but that materializer only accepts Mars.
  A Kitty desktop launch therefore failed before spawning Kitty.
- Native-config diagnostics advertised
  `~/.local/share/yazelix/configs/terminal_emulators/kitty` even though the
  Kitty contract keeps native configuration user-owned and writes no generated
  Kitty config.
- Doctor surfaced stale Mars launch logs even for a Kitty runtime because it
  only checked the runtime variant when the old log directory was empty.
- The FlexNetOS Agent desktop entry used the correct profile `yzx` command and
  layout override but retained Mars comment and WM-class metadata.
- The first installed Kitty launch reached the correct packaged Kitty binary,
  then exited by signal 11 after GLFW reported `EGL: Failed to initialize EGL`.
  The package runtime already shipped `libexec/nixGLMesa`, but the Rust launch
  argv no longer prepended it. The earlier Nushell launch implementation had
  applied NixGL to Kitty; that behavior was lost during the Rust ownership cut.

## Progress Log

### 2026-07-11

- Archived pre-repair runtime and source evidence under
  `/home/flexnetos/archive/yazelix-kitty-repair-20260711T1159Z/`.
- Repaired generated Yazelix state through `yzx doctor --fix`; subsequent
  profile doctor/status reports are healthy and select Kitty.
- Corrected the FlexNetOS Agent desktop entry metadata to Kitty and validated
  both desktop files with `desktop-file-validate`.
- Implemented source fixes so Kitty skips Mars-only materialization, native
  config status remains user-owned, and stale Mars logs are ignored.
- Archived the two failed installed-runtime launch logs under the existing
  repair archive.
- Implemented the follow-up in an isolated clean worktree because another
  session had uncommitted changes in the original checkout. Linux Kitty launch
  now prepends the absolute runtime-owned NixGL helper with structured argv,
  fails loudly if the packaged helper is missing, preserves Kitty-named launch
  logs, and self-describes `nixGLMesa` as a private required bundled command.
- Kept the fix inside the main Rust launch and Nix package-assembly owners, in
  line with the Rust/Nushell bridge, runtime self-description, terminal support,
  and terminal config-pack contracts.

## Verification

- `cargo fmt --manifest-path rust_core/Cargo.toml --all -- --check`
- `cargo check --manifest-path rust_core/Cargo.toml -p yazelix_core`
- Kitty regression filter: 11 passed
- Mars launch-log regression filter: 3 passed
- Full `yazelix_core --lib`: 442 passed; one pre-existing unrelated
  `config_state::tests::computes_default_rebuild_hash_without_recorded_state`
  failure remains
- `yzx_repo_validator validate-contracts`
- `nix build .#runtime_kitty --no-link --no-write-lock-file`

## Installed-runtime proof

- `yzx update local_source` upgraded the profile-owned package to
  `/nix/store/6j5i7cx0q863lhw85ihmd681p2y0hgp1-lifeos-foundation-yzx`.
- `yzx status --json` reports `terminal = "kitty"`, current generated state,
  and no repair requirement.
- `yzx doctor --json` reports zero errors and zero warnings.
- Doctor reports no Mars launch-log findings and no generated Kitty config;
  Kitty correctly uses built-in defaults because
  `~/.config/kitty/kitty.conf` is absent.
- Both desktop entries target `/home/flexnetos/.nix-profile/bin/yzx`, use the
  Kitty WM class, and pass desktop-file validation.

## Remaining gate

- Launch the built/fixed Kitty runtime in a fresh desktop window and obtain the
  maintainer's manual confirmation before pushing the non-trivial change.
