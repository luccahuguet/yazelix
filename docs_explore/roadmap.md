# Yazelix Roadmap (Proposed)

As of 2026-03-09

This is shared planning for sequencing major work.

It is not user-facing documentation and it is not a promise of delivery dates.

## Main Sequence

1. [ ] Harden config and build errors `#387`
   Make lock bumps and rebuilds deterministic, strengthen validation, and improve error quality first.
2. [ ] Packaging-readiness refactor `#401`
   Separate shipped assets, user config, and generated runtime state so Yazelix stops depending on a repo checkout as the runtime model.
3. [ ] Rollback system `#388`
   Make it possible to recover quickly from broken updates or config changes.
4. [ ] Packaging and distribution follow-on
   Sequence this after packaging-readiness:
   - umbrella `#232`
   - Home Manager rework `#403`
   - first-party flake interface `#404`
   - nixpkgs package `#402`
5. [ ] Docs experience pass
   Improve onboarding, structure docs around workflows, and keep commands and visuals current.
6. [ ] Website launch at `yazelix.com` `#408`
   Treat the website as a focused project surface, separate from the broader docs restructuring pass.
7. [ ] Workspace UX and Yazi ergonomics
   Focus on small but real workflow wins:
   - Yazi UX `#30`
   - keybinding ergonomics `#405`
   - tab name and cwd behavior `#406`
   - new panes opening at project path `#369`
   - bottom bar replacement or hideable bar `#407`
8. [ ] Support a separate user-managed `devenv` file `#396`
   Power-user opt-in, without replacing `yazelix.toml` as the default path.
9. [ ] Agent usage widgets in the Zellij widget tray `#400`
   Surface `ai_agents` and `ai_tools` more directly in the workspace.
10. [ ] Broader Rust/clap rewrite only after boundaries are proven
   Related exploration: `docs_explore/rust_rewrite.md`

## Always-On Maintenance Lane

This is not a numbered milestone.

Keep a strict WIP limit and only pull from here when it does not derail the main sequence.

Highest-friction current issues:
- desktop launcher with fish `#359`
- `yzx launch --here --path` bug `#357`
- Yazi user config not taken into account `#363`
- Yazi opening full white `#392`
- stale starship Yazi plugin state `#395`
- status bar not fitting the screen `#349`

## Completed

1. [x] Sidebar toggle/orchestration
   Current state: deterministic sidebar toggle, editor/sidebar focus, and managed editor targeting are shipped in `v13`.
