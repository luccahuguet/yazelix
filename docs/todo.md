# Backlog (Ordered)

- [ ] Add testing (#189): baseline tests for core flows (launch, layouts, keybindings, zjstatus).
- [ ] Packaging: make yazelix a single package (#232): reduce complexity and drift.
- [ ] Profile boot/initialization end-to-end (#257): instrument startup and log timings.
- [ ] Faster boot with included Ghostty (nixGL) (#258): reduce cold/warm start time.
- [ ] Plugin hygiene: remove hardcoded yazi plugins or add update script (#205).
- [ ] Core IPC: use pipes instead of `zellij action write-chars` (#49).
- [ ] Yazi UX: remove extra step after zoxide jump (#30).
- [ ] Keybindings: fix Alt-( and Alt-) in Helix inside Zellij (#176).
- [ ] VS Code integration: use `yzx env` via VS Code task syntax (#238).
- [ ] Desktop entry: Ghostty window shows blank icon/name; align with Yazelix (#259).
- [ ] Packs: LSP/formatter/linter packs for Go, Rust, Kotlin, TS/JS, Python; link each to Helix languages.toml config (#199).
- [ ] Language servers: Tailwind LSP pack option (#195).
- [ ] Language servers: postgres-language-server support (#239).
- [ ] Yazi: open multiple files from selection (#158).
- [ ] Platform: verify Windows support on WSL (#140).
- [ ] Terminal support: include terminal emulators (Alacritty/Kitty/Wezterm) bundled via nixGL (#247).
- [ ] Comparative table: terminal emulators (#234).
- [ ] Benchmark: clean install using Nix (#92).
- [ ] zjstatus: user‑configurable strings (#237).
- [ ] Integrated theming (#26).
- [ ] Yazi → system file manager: open native file manager on Linux (#242).
- [ ] UX: option to use a floating pane with lazygit / or an AI TUI of choice (#73).
- [ ] Zellij plugin: create a yazelix plugin (#167).
- [ ] UX experiment: job freeze/unfreeze to remove sidebar? (nushell freeze/unfreeze; no‑sidebar flow) (#240).
- [ ] Emoji support (#224).
- [ ] Integration: further Claude Code integration (#172).
- [ ] Evaluate Nushell AI integration (#223).
- [ ] Static preview: refresh/update preview assets (#250).
- [ ] Repo hygiene: filter large GIFs from history (#249).
- [ ] Media optimization: split demo GIFs into smaller pieces (#228).

## Notes and Rationale

- Stabilize the core (P1–P5): tests, install flexibility, plugin hygiene, nix warnings, and IPC.
- Unblock daily workflows (P6–P9): editor integration, language enablement via packs, keybindings, and small-screen UX.
- Docs and assets (P10–P17): reduce support and improve onboarding.
- Broader support/features (P18–P24): platforms, terminals, zjstatus customization, benchmarks, integrations.
- Experiments and nice‑to‑haves (P25+): keep separate to avoid rework.

## Decisions from Clarifications

- #244 zjstatus binary: duplication already solved; just verify and close.
- #199 Packs: scope is LSPs/formatters/linters for Go, Rust, Kotlin, TS/JS, Python; each pack should link to the relevant Helix languages.toml config.
- #190 yzx why: a concise elevator pitch; reuse same text in README.
- #238 VS Code: implement via calling `yzx env` from VS Code task syntax.
- #240 Freeze/unfreeze: use Nushell job freeze/unfreeze to reuse the same pane (no‑sidebar flow candidate).
- #247 Terminal emulators: bundle via nix with nixGL similar to Ghostty.
- #242 File manager from Yazi: Linux only to start.

## Acceptance Criteria — Packaging (#232)

- Publish as a Nix flake output installable via `nix profile install <src>#yazelix` and as a Home Manager module.
- Entrypoints on PATH: `yazelix` (main) and `yzx` (helper).
- No repo clone required; works from any cwd.
- XDG-compliant paths:
  - Config under `$XDG_CONFIG_HOME/yazelix`
  - State under `$XDG_STATE_HOME/yazelix`
  - Cache under `$XDG_CACHE_HOME/yazelix`
- No runtime writes to the Nix store; do not assume writable installation paths.
- Respect and allow user overrides/extensions under `$XDG_CONFIG_HOME/yazelix/*` (layouts, packs, themes).
- Docs: clear install instructions (Nix profile + Home Manager) and migration from repo-clone installs.
- Build/test: flake outputs build in CI; smoke test that `yazelix --version` runs and a minimal session starts.
- Out of scope: bundling terminal emulators via nixGL (#247) and other integrations (tracked separately).

## Done

- [x] Nix deprecation: replace `mesa.drivers` with `mesa` (#254)
- [x] Layout/UX: zjstatus looks good on 13” half‑width splits (#255)
- [x] Docs: POSIX and XDG usage and paths (#206)
- [x] Docs: notes on using yazelix via SSH (#253)
- [x] Docs: add “yzx why” elevator pitch and README section (#190)
