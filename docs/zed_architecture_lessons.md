# Zed Architecture Lessons For Yazelix

This note records the useful findings from a shallow inspection of `zed-industries/zed` for bead `yazelix-qi71`.

Source inspected: `zed@62507a1`, cloned at `/home/lucca/pjs/open_source/yazelix_related/zed` on 2026-05-05

## Boundary

Use Zed as product and architecture inspiration, not as code to vendor. The Zed repo root carries `LICENSE-AGPL`, `LICENSE-GPL`, and `LICENSE-APACHE`, and the relevant surfaces are deeply tied to GPUI and a GUI editor runtime. Direct reuse would need a separate per-file license and dependency review, and it is not justified for the current Yazelix terminal-native config/action track.

## Useful Patterns

| Zed surface | Repo references | Useful Yazelix lesson |
| --- | --- | --- |
| Typed settings inventory | `crates/settings_content/src/settings_content.rs`, `crates/settings/src/settings_store.rs` | Treat settings as typed product state, not loose file text. Yazelix should keep `settings.jsonc` plus schema metadata as the semantic inventory, then derive UI, validation, docs, and defaults from that inventory |
| Curated settings UI pages | `crates/settings_ui/src/page_data.rs`, `crates/settings_ui/src/settings_ui.rs` | Do not render the raw schema tree directly. Build user-intent tabs with titles, descriptions, field pick/write functions, and explicit per-file support |
| Default versus user state | `SettingsStore::raw_default_settings`, `get_value_from_file`, `get_value_up_to_file`; `settings_ui.rs` reset and "Modified in" rendering | The Yazelix config UI should show whether a value is explicit, defaulted/unset, overridden by another owner, or read-only, and should offer reset only when the current owner can safely change it |
| Settings search | `settings_ui.rs` search-index construction around page, section, title, description, and JSON path | The first read-only UI should include search from day one. Search candidates should include tab, section, label, help text, and `settings.jsonc` path |
| Comment-preserving writes | `crates/settings_json/src/settings_json.rs` | Zed updates JSON text by path while preserving unrelated comments and formatting. This validates `yazelix-ryx4.2`: Yazelix should not parse-and-pretty-print user `settings.jsonc` as the default save path |
| Action registry | `crates/gpui/src/action.rs`, `crates/zed_actions/src/lib.rs`, `script/generate-action-metadata` | Actions deserve names, namespaces, docs, optional argument schemas, deprecation metadata, and generated docs. Yazelix should define its own small action registry before offering friendly remaps |
| Command palette | `crates/command_palette/src/command_palette.rs`, `crates/command_palette/src/persistence.rs`, `docs/src/command-palette.md` | A palette should be action-backed, searchable by human names and action names, show active keybindings, remember recent queries, and allow aliases. This is a better model than a hand-maintained menu list |
| Keymap model and diagnostics | `crates/gpui/src/keymap.rs`, `crates/gpui/src/keymap/context.rs`, `crates/keymap_editor/src/keymap_editor.rs`, `crates/language_tools/src/key_context_view.rs`, `docs/src/key-bindings.md` | Keybindings need context, precedence, unbind/no-action semantics, conflict diagnostics, and a "what context am I in?" debugging view. Yazelix should not stop at generating KDL/TOML snippets |
| Keymap editor UX | `crates/keymap_editor/src/keymap_editor.rs`, `crates/keymap_editor/src/action_completion_provider.rs`, command palette footer `Change Keybinding...` / `Add Keybinding...` | Users should remap semantic actions from the action list, not learn Zellij/Yazi/Helix implementation details first. Native sidecars remain available for advanced use |

## Yazelix Shape

### Config UI

The Zed settings UI confirms the existing Yazelix direction:

- `settings.jsonc` remains the durable file surface
- schema and contract metadata should describe all semantic settings, including values absent from the user's sparse file
- the UI should be curated by user intent, not raw storage sections
- each field needs label, help text, JSON path, default source, owner, editor kind, restart/apply requirement, and write support metadata
- Home Manager-owned settings should render read-only with the declarative owner path
- Advanced should show sidecar status and open/import/validate actions, not pretend to round-trip arbitrary KDL, Lua, shell, or terminal-native grammars
- writes should wait for a comment-preserving JSONC patcher; unsafe edits should require preview-confirmed rewrite

This maps directly to `yazelix-ryx4.1`, `yazelix-ryx4.2`, and `yazelix-ryx4.3`.

### Action Keymap

Zed's strongest lesson for `yazelix-hg3a` is that keybindings should hang from an action registry:

- define Yazelix-owned actions first, such as `workspace.toggle_sidebar`, `workspace.focus_sidebar`, `workspace.open_selected_in_editor`, `workspace.zoxide_to_editor`, `workspace.toggle_popup`, `workspace.toggle_menu`, and `zellij.unlock`
- give every action a stable id, label, description, owning subsystem, native backend steps, default binding, and optional arguments
- generate Zellij, Yazi, and Helix-facing bindings from semantic actions where possible
- keep native sidecars as escape hatches, including Zellij `keybinds clear-defaults=true`
- expose conflicts as product diagnostics: duplicate key, shadowed key, missing required backend action, context mismatch, and backend unsupported
- add a small context/debug view before broad editing, showing which Yazelix pane/tool/context owns the currently focused keybinding layer

Do not clone Zed's full context-expression language now. Yazelix can start with coarse contexts such as `workspace`, `zellij`, `yazi_mgr`, `editor`, `shell`, `popup`, and `sidebar`, then add precision only when a real conflict needs it.

### Command Palette

The current `yzx menu` should eventually be generated from the same action registry:

- every palette item should correspond to a typed action
- search should match action id, label, aliases, and description
- results should show the current keybinding when one exists
- usage history and query aliases are useful later, but not required for the first registry slice
- palette-only commands should still be registered actions, not separate menu-only strings

## Avoid

- Do not import GPUI architecture into Yazelix. Ratatui plus Yazelix's terminal runtime is the right UI layer
- Do not make the config UI a raw schema browser. Zed uses curated pages even with typed settings underneath
- Do not make action remapping a thin wrapper around native Zellij/Yazi/Helix config. That preserves the problem users reported
- Do not add long-lived legacy key names or config aliases by default. Zed supports deprecated aliases, but Yazelix's command-surface policy is stricter unless a pushed release or user-support reason justifies compatibility
- Do not attempt a complete keybinding editor before the action registry, conflict model, and read-only diagnostics exist

## Follow-Up Links

- `yazelix-ryx4.1`: expand schema UI metadata with curated labels, paths, default/owner state, editor kind, and sidecar action metadata
- `yazelix-ryx4.2`: implement a comment-preserving JSONC patcher before mutable config UI writes
- `yazelix-ryx4.3`: build the first read-only Ratatui browser with search, explicit/defaulted/read-only states, and sidecar status
- `yazelix-hg3a`: design the semantic action registry and action-keymap surface that can generate Zellij/Yazi/Helix bindings
- `yazelix-7rn8`: consume the action model for robust Yazi-to-editor open behavior instead of depending on fragile editor command-mode keys
