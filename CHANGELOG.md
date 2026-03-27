# CHANGELOG

Short, upgrade-facing release notes live here. The longer narrative history remains in [docs/history.md](./docs/history.md).

## Unreleased

Upgrade contract, config migrations, and stale-config diagnostics.

Upgrade impact: migration available

Highlights:
- Added `yzx config migrate` with a shared migration registry, preview-first behavior, backup-first apply mode, and explicit manual-only cases.
- Startup, refresh, and `yzx doctor` now surface migration-aware config diagnostics instead of collapsing stale config into generic failures.
- `yzx doctor --fix` can now apply the same safe config rewrites as `yzx config migrate --apply`.

Migration notes:
- Removed the broken `layout` value from `zellij.widget_tray`; safe migration is available.
- Removed the obsolete `shell.enable_atuin` toggle; safe migration is available.
- Legacy cursor-trail settings still require manual review because the old intent is not always deterministic.

## v13.7 - 2026-03-26

Popup polish, Ghostty controls, and stronger validation.

Highlights:
- Added configurable popup sizing and a configurable popup program for Zellij floating panes.
- Added Ghostty trail glow controls plus a `ghostty_trail_color = "none"` option.
- Tightened config-schema validation, ignored Yazelix config backup files, and improved test/spec governance.

See also:
- Full narrative history: [docs/history.md](./docs/history.md)
