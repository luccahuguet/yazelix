# Yazelix Rust Migration Plan

This document outlines the incremental migration of Yazelix from Nushell to Rust for core application logic, while keeping Nushell for shell integration and CLI dispatch.

## Goals

- **Maintainability:** Type safety, compile-time checks, better refactoring tools
- **Performance:** Faster config parsing, merging, and startup
- **Reliability:** Eliminate regex parsing fragility, better error handling
- **Distribution:** Single binary distribution via Nix binary cache
- **Constraint:** All shell scripts remain `.nu` files (no bash/sh)

## Architecture Overview

```
yazelix/
├── Cargo.toml               # Single crate (simpler!)
├── src/
│   ├── main.rs              # Entry point, CLI dispatch
│   ├── cli.rs               # Clap command definitions
│   ├── config/              # Config parsing, merging, validation
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   ├── merger.rs
│   │   ├── validator.rs
│   │   └── types.rs
│   ├── launcher/            # Terminal detection, launch commands
│   │   ├── mod.rs
│   │   ├── terminal.rs
│   │   └── command.rs
│   ├── doctor/              # Health checks, diagnostics
│   │   ├── mod.rs
│   │   ├── checks.rs
│   │   └── fixes.rs
│   └── nix/                 # Nix environment detection
│       ├── mod.rs
│       └── detector.rs
├── nushell/scripts/         # Thin .nu wrappers + integration hooks
└── flake.nix                # Rust build configuration
```

**Why single crate?** Simpler to learn, faster to develop, easier to navigate. Multiple crates are overkill for a CLI tool. Modules give you the same organization without the complexity.

---

## Milestone 1: Project Setup & Foundation

### 1.1 Create Rust Project

- [ ] Create `Cargo.toml` at project root with binary configuration
- [ ] Add basic dependencies: `clap`, `anyhow`, `serde`, `toml`, `kdl`
- [ ] Create `src/` directory
- [ ] Create `src/main.rs` with "Hello, Yazelix!" test
- [ ] Set up `.gitignore` for Rust (`target/`, `Cargo.lock`)
- [ ] Test basic compilation: `cargo build` and `cargo run`

### 1.2 Configure Nix Integration

- [ ] Update `flake.nix` to include Rust toolchain
- [ ] Add `rustPlatform.buildRustPackage` configuration
- [ ] Test basic Rust compilation via `nix build`
- [ ] Verify Rust binary is available in `nix develop` environment
- [ ] Document Nix + Rust setup in this file

### 1.3 Set Up CI/CD for Binary Caching

- [ ] Create `.github/workflows/build.yml` for Rust builds
- [ ] Configure builds for: `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, `aarch64-darwin`
- [ ] Set up Cachix or configure binary cache
- [ ] Test cache push/pull workflow
- [ ] Document caching strategy

**Completion Criteria:** `yazelix --version` runs successfully via `nix develop`

---

## Milestone 2: CLI Framework & Basic Commands

### 2.1 Implement CLI Structure with Clap

- [ ] Create `src/cli.rs` with `clap` subcommands:
  - `yazelix --version`
  - `yazelix --help`
  - `yazelix launch`
  - `yazelix env`
  - `yazelix doctor`
  - `yazelix bench`
  - `yazelix test`
  - `yazelix info`
  - `yazelix versions`
- [ ] Add `--verbose` and `--quiet` flags
- [ ] Implement basic subcommand dispatch (empty implementations)

### 2.2 Simple Commands (Info & Versions)

- [ ] Implement `yazelix --version` (show Yazelix version)
- [ ] Implement `yazelix versions` (show Helix, Zellij, Yazi, Nix versions)
- [ ] Implement `yazelix info` (show system info, paths, environment)
- [ ] Add JSON output option (`--json`) for scriptability

### 2.3 Update Nushell CLI Wrapper

- [ ] Modify `nushell/scripts/core/yazelix.nu` to call Rust binary
- [ ] Keep Nushell wrapper for backward compatibility
- [ ] Test all subcommands work via both `yazelix` and `yzx` aliases
- [ ] Verify help text matches between Rust and Nushell

**Completion Criteria:** All info commands work identically to Nushell version

---

## Milestone 3: Config Parsing & Validation

### 3.1 Create Config Module

- [ ] Create `src/config/` directory
- [ ] Create `src/config/mod.rs` (module entry point)
- [ ] Define `YazelixConfig` struct in `src/config/types.rs` with all fields from `yazelix_default.nix`
- [ ] Add `serde` derives for serialization
- [ ] Implement `Default` trait with yazelix_default.nix defaults

### 3.2 Implement Nix Config Parser

- [ ] Create `src/config/parser.rs` with proper parser (consider `pest` or hand-rolled)
- [ ] Parse `yazelix.nix` into `YazelixConfig` struct
- [ ] Handle all config option types:
  - Booleans (`enable = true`)
  - Strings (`shell = "nu"`)
  - Lists (`extra_packages = [ ... ]`)
  - Nested structures (packs)
- [ ] Add comprehensive error messages for parse failures
- [ ] Write unit tests for parser edge cases

### 3.3 Config Validation

- [ ] Create `src/config/validator.rs`
- [ ] Validate config options:
  - Shell must be one of: bash, zsh, fish, nu
  - Editor must be: null, "nvim", or custom path
  - Terminal must be valid terminal name
  - Paths must exist where required
- [ ] Add helpful error messages with suggestions
- [ ] Write unit tests for validation rules

### 3.4 Expose Config Commands

- [ ] Implement `yazelix config parse` - parse and validate yazelix.nix
- [ ] Implement `yazelix config show` - display current config
- [ ] Implement `yazelix config validate` - validate without side effects
- [ ] Add `--json` output for programmatic use
- [ ] Test against real yazelix.nix files

**Completion Criteria:** Config parsing matches Nushell behavior, better error messages

---

## Milestone 4: Config Merging (TOML & KDL)

### 4.1 TOML Config Merger (Yazi)

- [ ] Create `src/config/merger.rs`
- [ ] Implement 3-layer TOML merging:
  - Layer 1: `configs/yazi/yazelix_*.toml`
  - Layer 2: `configs/yazi/personal/*.toml`
  - Layer 3: Generated configs
- [ ] Use `toml` crate for parsing
- [ ] Implement deep merge algorithm (nested tables)
- [ ] Write merged config to `configs/yazi/user/yazi.toml`
- [ ] Add atomic file writes (write to temp, then rename)
- [ ] Write unit tests for merge logic

### 4.2 KDL Config Merger (Zellij)

- [ ] Implement KDL config merging using `kdl` crate
- [ ] 3-layer merging:
  - Layer 1: `configs/zellij/yazelix_overrides.kdl`
  - Layer 2: `configs/zellij/personal/*.kdl`
  - Layer 3: Generated configs
- [ ] Handle Zellij-specific KDL structure
- [ ] Write merged config to `configs/zellij/user/config.kdl`
- [ ] Add atomic file writes
- [ ] Write unit tests for KDL merge logic

### 4.3 Config Change Detection

- [ ] Implement SHA256 hashing for config files
- [ ] Cache hash in `~/.cache/yazelix/config.hash`
- [ ] Skip merge if config unchanged (performance optimization)
- [ ] Add `--force-rebuild` flag to bypass cache
- [ ] Test cache invalidation works correctly

### 4.4 Expose Merge Commands

- [ ] Implement `yazelix config merge-yazi` - merge Yazi configs
- [ ] Implement `yazelix config merge-zellij` - merge Zellij configs
- [ ] Implement `yazelix config merge-all` - merge all configs
- [ ] Add `--dry-run` to show what would be merged without writing
- [ ] Update `start_yazelix.nu` to call Rust for merging

**Completion Criteria:** Config merging produces identical output to Nushell version, ~2-5x faster

---

## Milestone 5: Terminal Detection & Launcher

### 5.1 Create Launcher Module

- [ ] Create `src/launcher/` directory
- [ ] Create `src/launcher/mod.rs` (module entry point)
- [ ] Define `TerminalInfo` struct in `src/launcher/terminal.rs` with terminal metadata
- [ ] Create terminal detection logic in `src/launcher/terminal.rs`

### 5.2 Terminal Detection Logic

- [ ] Implement terminal detection for:
  - Ghostty
  - WezTerm
  - Kitty
  - Alacritty
  - Foot
- [ ] Check `YAZELIX_TERMINAL` environment variable first
- [ ] Detect installed terminals via command existence
- [ ] Respect user preference from config
- [ ] Implement fallback priority list
- [ ] Add helpful errors when no terminal found

### 5.3 Terminal Config Path Resolution

- [ ] Implement config path resolution for each terminal
- [ ] Check user configs first (`~/.config/ghostty/`, etc.)
- [ ] Check Yazelix-managed configs second
- [ ] Add `--config-mode` option: `yazelix-managed`, `user-managed`
- [ ] Test path resolution for all terminals

### 5.4 Launch Command Building

- [ ] Create `src/launcher/command.rs`
- [ ] Build launch commands for each terminal:
  - Ghostty: `ghostty -e <command>`
  - WezTerm: `wezterm start -- <command>`
  - Kitty: `kitty -e <command>`
  - Alacritty: `alacritty -e <command>`
  - Foot: `foot <command>`
- [ ] Handle config file arguments
- [ ] Handle transparency, cursor trails, etc.
- [ ] Add environment variable passing

### 5.5 Expose Launcher Commands

- [ ] Implement `yazelix launch` - full launch workflow
- [ ] Implement `yazelix detect-terminal` - show detected terminal
- [ ] Update `nushell/scripts/core/launch_yazelix.nu` to call Rust
- [ ] Test launches from all supported terminals
- [ ] Verify config modes work correctly

**Completion Criteria:** Terminal detection/launching works identically, cleaner code

---

## Milestone 6: Doctor & Diagnostics

### 6.1 Create Doctor Module

- [ ] Create `src/doctor/` directory
- [ ] Create `src/doctor/mod.rs` (module entry point)
- [ ] Define health check result types in `src/doctor/checks.rs`
- [ ] Create diagnostic checks in `src/doctor/checks.rs`

### 6.2 Implement Health Checks

- [ ] Check runtime conflicts (Helix/Neovim vs Zellij keybindings)
- [ ] Check required directories exist
- [ ] Check config file validity
- [ ] Check Nix environment availability
- [ ] Check log file sizes
- [ ] Check grammar installations (Helix)
- [ ] Check Yazi plugins exist
- [ ] Check Zellij plugins exist
- [ ] Add detailed diagnostic output

### 6.3 Implement Auto-Fix Logic

- [ ] Create `src/doctor/fixes.rs`
- [ ] Implement backup before fixes
- [ ] Auto-fix runtime conflicts:
  - Detect conflicting keybindings
  - Comment out conflicts in Helix config
  - Add explanatory comments
- [ ] Clean up old logs
- [ ] Recreate missing directories
- [ ] Add `--fix` flag to apply fixes
- [ ] Add `--dry-run` to preview fixes
- [ ] Implement rollback on failure

### 6.4 Expose Doctor Commands

- [ ] Implement `yazelix doctor` - run all checks
- [ ] Implement `yazelix doctor --fix` - run checks and auto-fix
- [ ] Implement `yazelix doctor --check <specific>` - run one check
- [ ] Add colorful output with ✓/✗ indicators
- [ ] Update `nushell/scripts/utils/doctor.nu` to call Rust

**Completion Criteria:** Doctor checks match Nushell version, safer auto-fix with rollback

---

## Milestone 7: Nix Environment Detection

### 7.1 Create Nix Module

- [ ] Create `src/nix/` directory
- [ ] Create `src/nix/mod.rs` (module entry point)
- [ ] Create `src/nix/detector.rs`

### 7.2 Nix Detection Logic

- [ ] Check for Nix in PATH
- [ ] Check common Nix installation locations:
  - `/nix/store`
  - `~/.nix-profile`
  - `/run/current-system/sw` (NixOS)
- [ ] Detect Nix installation type (multi-user, single-user, NixOS)
- [ ] Validate Nix version compatibility
- [ ] Add clear error messages for missing Nix

### 7.3 Environment Setup

- [ ] Generate `nix develop` command with correct flake path
- [ ] Set up environment variables
- [ ] Handle GitHub URL vs local path
- [ ] Add `--flake` option to specify custom flake
- [ ] Test on NixOS and non-NixOS systems

### 7.4 Expose Nix Commands

- [ ] Implement `yazelix nix detect` - show Nix installation info
- [ ] Implement `yazelix nix check` - validate Nix environment
- [ ] Implement `yazelix env` - enter Nix environment
- [ ] Update `start_yazelix.nu` to call Rust for detection
- [ ] Remove complex `nix_detector.nu` (320 lines → cleaner Rust)

**Completion Criteria:** Nix detection works reliably, simpler than 320-line Nushell version

---

## Milestone 8: Integration & Testing

### 8.1 Update Nushell Scripts

- [ ] Update `nushell/scripts/core/start_yazelix.nu`:
  - Call `yazelix config merge-all`
  - Call `yazelix nix detect`
  - Keep Zellij launch as Nushell
- [ ] Update `nushell/scripts/core/launch_yazelix.nu`:
  - Call `yazelix launch`
- [ ] Update `nushell/scripts/core/yazelix.nu`:
  - Dispatch all commands to Rust binary
  - Keep as thin wrapper for backward compatibility
- [ ] Keep all integration hooks as `.nu` files (no changes needed)

### 8.2 Comprehensive Testing

- [ ] Test all shells: bash, zsh, fish, nu
- [ ] Test all terminals: Ghostty, WezTerm, Kitty, Alacritty, Foot
- [ ] Test config merging with various yazelix.nix configurations
- [ ] Test doctor checks and auto-fix
- [ ] Test Nix detection on NixOS and non-NixOS
- [ ] Test performance improvements (benchmark startup time)
- [ ] Test on clean system (no existing configs)

### 8.3 Performance Benchmarking

- [ ] Implement `yazelix bench` in Rust
- [ ] Benchmark config parsing (Rust vs Nushell)
- [ ] Benchmark config merging (Rust vs Nushell)
- [ ] Benchmark startup time (full launch)
- [ ] Add `--compare` flag to show Rust vs Nushell comparison
- [ ] Document performance improvements

### 8.4 Error Handling & UX

- [ ] Ensure all errors have helpful messages
- [ ] Add suggestions for common issues
- [ ] Test error messages for usability
- [ ] Add progress indicators for long operations
- [ ] Add `--verbose` for debugging output
- [ ] Add `--quiet` for minimal output

**Completion Criteria:** All functionality works via Rust, comprehensive test coverage

---

## Milestone 9: Documentation & Polish

### 9.1 Update Documentation

- [ ] Update `README.md` with Rust migration notes
- [ ] Update `docs/installation.md` if needed
- [ ] Update `docs/yzx_cli.md` with new Rust-powered CLI
- [ ] Document new `--json` output options
- [ ] Add troubleshooting section for Rust-specific issues
- [ ] Document binary caching setup for contributors

### 9.2 Migration Guide

- [ ] Create `docs/nushell_to_rust_migration.md` for users
- [ ] Document what changed for users (mostly transparent)
- [ ] Document what changed for contributors (Rust development)
- [ ] Add examples of calling Rust from Nushell
- [ ] Document new capabilities (better errors, performance, etc.)

### 9.3 Contributor Documentation

- [ ] Create `CONTRIBUTING.md` with Rust development guide
- [ ] Document module structure and responsibilities
- [ ] Add examples of adding new features
- [ ] Document testing strategy
- [ ] Add debugging tips for Rust development

### 9.4 Final Polish

- [ ] Add shell completions (bash, zsh, fish, nu) via `clap_complete`
- [ ] Add man page generation via `clap_mangen`
- [ ] Optimize binary size (strip symbols, LTO)
- [ ] Run `cargo clippy` and fix all warnings
- [ ] Run `cargo fmt` for consistent style
- [ ] Update `CLAUDE.md` with Rust conventions if needed

**Completion Criteria:** Documentation complete, polished user and contributor experience

---

## Milestone 10: Release & Distribution

### 10.1 Nix Flake Updates

- [ ] Update `flake.nix` with final Rust build configuration
- [ ] Set up outputs for all platforms
- [ ] Test `nix build` on all platforms
- [ ] Test `nix develop` includes Rust binary
- [ ] Document flake structure

### 10.2 Binary Caching

- [ ] Verify CI/CD pushes to cache correctly
- [ ] Test cache downloads on clean system
- [ ] Document cache setup for forks
- [ ] Monitor cache hit rates

### 10.3 Version Bump & Release

- [ ] Update version to reflect Rust rewrite (e.g., v11.0)
- [ ] Create comprehensive changelog
- [ ] Tag release in git
- [ ] Create GitHub release with notes
- [ ] Announce migration in README

### 10.4 Post-Release Monitoring

- [ ] Monitor GitHub issues for migration problems
- [ ] Gather performance feedback
- [ ] Document any rough edges discovered
- [ ] Plan for Phase 2 features if desired

**Completion Criteria:** Yazelix v11.0 released with Rust core, users successfully migrated

---

## Future Enhancements (Phase 2+)

### Optional Advanced Migrations

- [ ] Migrate Zellij pane management to Rust (more reliable pane detection)
- [ ] Migrate shell config management to Rust (safer file modifications)
- [ ] Add Yazelix plugin system in Rust
- [ ] Create Yazelix language server for config editing
- [ ] Explore WASM for cross-platform without Nix

### Performance Optimizations

- [ ] Parallel config merging
- [ ] Incremental config caching
- [ ] Lazy loading of heavy dependencies

### New Features Enabled by Rust

- [ ] Better error recovery and suggestions
- [ ] Interactive TUI for configuration
- [ ] Built-in update mechanism
- [ ] Telemetry (opt-in) for usage insights

---

## Progress Tracking

### Current Status

- **Milestone 1:** ⬜ Not Started
- **Milestone 2:** ⬜ Not Started
- **Milestone 3:** ⬜ Not Started
- **Milestone 4:** ⬜ Not Started
- **Milestone 5:** ⬜ Not Started
- **Milestone 6:** ⬜ Not Started
- **Milestone 7:** ⬜ Not Started
- **Milestone 8:** ⬜ Not Started
- **Milestone 9:** ⬜ Not Started
- **Milestone 10:** ⬜ Not Started

### Estimated Timeline

- **Milestone 1-2:** 1-2 weeks (setup + basic CLI)
- **Milestone 3-4:** 2-3 weeks (config parsing + merging)
- **Milestone 5:** 1-2 weeks (launcher)
- **Milestone 6-7:** 1-2 weeks (doctor + Nix detection)
- **Milestone 8:** 1-2 weeks (integration + testing)
- **Milestone 9-10:** 1 week (docs + release)

**Total:** ~10-14 weeks for complete migration

---

## Notes

- All shell scripts remain `.nu` files - no bash/sh allowed
- Nushell integration hooks stay as Nushell (called by Yazi/Zellij)
- Rust handles complex data processing, Nushell handles shell integration
- Hybrid approach: gradual migration, maintain backward compatibility
- Users download pre-built Rust binaries via Nix cache (no Rust installation needed)
