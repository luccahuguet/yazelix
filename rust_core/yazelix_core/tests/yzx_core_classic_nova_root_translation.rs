// Test lane: default

use std::collections::BTreeSet;
use toml::{Table, Value};
use yazelix_core::classic_nova_root_translation::{
    MAPPED_CLASSIC_ROOT_FIELDS, NATIVE_TRANSITION_CLASSIC_ROOT_FIELDS, REMOVED_CLASSIC_ROOT_FIELDS,
};
use yazelix_core::{ClassicNovaDisposition, translate_classic_root};

fn parse(source: &str) -> Table {
    toml::from_str(source).expect("valid Classic TOML fixture")
}

fn value_at<'a>(root: &'a Table, path: &str) -> Option<&'a Value> {
    let mut segments = path.split('.');
    let mut value = root.get(segments.next()?)?;
    for segment in segments {
        value = value.as_table()?.get(segment)?;
    }
    Some(value)
}

fn dispositions_for(
    translation: &yazelix_core::ClassicNovaRootTranslation,
    path: &str,
) -> Vec<ClassicNovaDisposition> {
    translation
        .report
        .iter()
        .filter(|entry| entry.source_path == path)
        .map(|entry| entry.disposition)
        .collect()
}

// Invariant: the temporary translator has one approved disposition owner for every Classic semantic root field.
#[test]
fn approved_ledger_contains_46_unique_fields_exactly_once() {
    let listed = MAPPED_CLASSIC_ROOT_FIELDS
        .iter()
        .chain(NATIVE_TRANSITION_CLASSIC_ROOT_FIELDS)
        .chain(REMOVED_CLASSIC_ROOT_FIELDS)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(listed.len(), 46);
    let listed = listed.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(listed.len(), 46);

    let contract: Table = toml::from_str(include_str!(
        "../../../config_metadata/main_config_contract.toml"
    ))
    .unwrap();
    let canonical = contract["fields"]
        .as_table()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(listed, canonical);
}

// Defends: every approved deterministic conversion produces the sparse Nova path without touching native sidecars.
#[test]
fn maps_supported_scalar_pair_list_keybinding_and_popup_values() {
    let translation = translate_classic_root(&parse(
        r#"
[core]
skip_welcome_screen = true
welcome_style = "boids"
welcome_duration_seconds = 4.0

[editor]
command = "nvim"

[shell]
default_shell = "zsh"

[workspace.right_sidebar]
command = "codex"
args = ["resume"]

[zellij]
widget_tray = ["session", "workspace", "editor", "cpu"]

[zellij.keybindings]
bottom_popup = ["Alt Shift J"]
top_popup = ["Alt Shift K"]
menu = ["Alt Shift M"]
open_codex_agent_right = ["Alt Shift L"]

[[zellij.custom_popups]]
id = " zenith "
command = ["zenith", " --basic "]
keybindings = [" Alt Shift I "]
keep_alive = true
"#,
    ));

    assert_eq!(
        value_at(&translation.root, "welcome.enabled").and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        value_at(&translation.root, "welcome.style").and_then(Value::as_str),
        Some("boids")
    );
    assert_eq!(
        value_at(&translation.root, "welcome.duration_seconds").and_then(Value::as_integer),
        Some(4)
    );
    assert_eq!(
        value_at(&translation.root, "editor.command").and_then(Value::as_str),
        Some("nvim")
    );
    assert_eq!(
        value_at(&translation.root, "shell.program").and_then(Value::as_str),
        Some("zsh")
    );
    assert_eq!(
        value_at(&translation.root, "agent.command").and_then(Value::as_str),
        Some("codex")
    );
    assert_eq!(
        value_at(&translation.root, "agent.args"),
        Some(&Value::Array(vec![Value::String("resume".into())]))
    );
    assert_eq!(
        value_at(&translation.root, "bar.widgets"),
        Some(&Value::Array(vec![
            Value::String("session".into()),
            Value::String("editor".into()),
            Value::String("cpu".into()),
        ]))
    );
    for (path, chord) in [
        ("keybindings.git", "Alt Shift J"),
        ("keybindings.config", "Alt Shift K"),
        ("keybindings.menu", "Alt Shift M"),
        ("keybindings.agent", "Alt Shift L"),
    ] {
        assert_eq!(
            value_at(&translation.root, path).and_then(Value::as_str),
            Some(chord)
        );
    }
    assert_eq!(
        value_at(&translation.root, "popups.zenith.command").and_then(Value::as_str),
        Some("zenith")
    );
    assert_eq!(
        value_at(&translation.root, "popups.zenith.args"),
        Some(&Value::Array(vec![Value::String("--basic".into())]))
    );
    assert_eq!(
        value_at(&translation.root, "popups.zenith.title").and_then(Value::as_str),
        Some("yzx_zenith")
    );
    assert_eq!(
        value_at(&translation.root, "popups.zenith.keybinding").and_then(Value::as_str),
        Some("Alt Shift I")
    );
    assert_eq!(
        value_at(&translation.root, "popups.zenith.keep_alive").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        dispositions_for(&translation, "zellij.widget_tray.workspace"),
        vec![ClassicNovaDisposition::Removed]
    );
    for path in [
        "core.skip_welcome_screen",
        "core.welcome_style",
        "core.welcome_duration_seconds",
        "editor.command",
        "shell.default_shell",
        "workspace.right_sidebar.command",
        "workspace.right_sidebar.args",
        "zellij.widget_tray",
        "zellij.keybindings.bottom_popup",
        "zellij.keybindings.top_popup",
        "zellij.keybindings.menu",
        "zellij.keybindings.open_codex_agent_right",
        "zellij.custom_popups.zenith",
    ] {
        assert_eq!(
            dispositions_for(&translation, path),
            vec![ClassicNovaDisposition::Preserved],
            "missing preserved report evidence for {path}"
        );
    }
}

// Regression: explicit Classic values equal to old defaults remain explicit, while only the two approved semantic omissions disappear.
#[test]
fn preserves_explicit_default_intent_without_equality_inference() {
    let translation = translate_classic_root(&parse(
        r#"
[core]
skip_welcome_screen = false
welcome_style = "random"
welcome_duration_seconds = 4.0
[editor]
command = ""
[shell]
default_shell = "nu"
[workspace.right_sidebar]
command = "yzx"
args = ["agent"]
"#,
    ));

    assert_eq!(
        value_at(&translation.root, "welcome.enabled").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        value_at(&translation.root, "welcome.style").and_then(Value::as_str),
        Some("random")
    );
    assert_eq!(
        value_at(&translation.root, "welcome.duration_seconds").and_then(Value::as_integer),
        Some(4)
    );
    assert_eq!(
        value_at(&translation.root, "shell.program").and_then(Value::as_str),
        Some("nu")
    );
    assert!(value_at(&translation.root, "editor.command").is_none());
    assert!(value_at(&translation.root, "agent.command").is_none());
    for path in [
        "editor.command",
        "workspace.right_sidebar.command",
        "workspace.right_sidebar.args",
    ] {
        assert_eq!(
            dispositions_for(&translation, path),
            vec![ClassicNovaDisposition::Preserved]
        );
    }
}

// Regression: fractional durations, command strings, xonsh, ambiguous agent pairs, and conflicting popup controls are reported instead of approximated.
#[test]
fn rejects_every_unrepresentable_conversion_family() {
    let translation = translate_classic_root(&parse(
        r#"
[core]
welcome_duration_seconds = 2.5
[editor]
command = "nvim --clean"
[shell]
default_shell = "xonsh"
[workspace.right_sidebar]
command = "yzx"
args = ["config"]
[zellij]
widget_tray = ["editor", "future_widget"]
[zellij.keybindings]
bottom_popup = ["Alt Shift J", "Alt Shift B"]
top_popup = ["Alt m"]

[[zellij.custom_popups]]
id = "agent"
command = ["btm"]
keybindings = ["Alt Shift B"]

[[zellij.custom_popups]]
id = "legacy"
command = ["yzx", "menu"]
keybindings = ["Alt Shift C"]

[[zellij.custom_popups]]
id = "conflict"
command = ["btm"]
keybindings = ["Alt Shift L"]
"#,
    ));

    for path in [
        "core.welcome_duration_seconds",
        "editor.command",
        "shell.default_shell",
        "workspace.right_sidebar.command",
        "workspace.right_sidebar.args",
        "zellij.widget_tray",
        "zellij.keybindings.bottom_popup",
        "zellij.keybindings.top_popup",
        "zellij.custom_popups.agent",
        "zellij.custom_popups.legacy",
        "zellij.custom_popups.conflict",
    ] {
        assert_eq!(
            dispositions_for(&translation, path),
            vec![ClassicNovaDisposition::Rejected],
            "missing rejection for {path}"
        );
    }
    assert!(translation.root.is_empty());
}

// Regression: malformed paired command state and Nova's reserved auto sentinel never become a plausible agent command.
#[test]
fn rejects_invalid_or_reserved_agent_commands() {
    for source in [
        "[workspace.right_sidebar]\ncommand = 42\nargs = [\"agent\"]\n",
        "[workspace.right_sidebar]\ncommand = \"auto\"\nargs = []\n",
    ] {
        let translation = translate_classic_root(&parse(source));
        assert!(value_at(&translation.root, "agent.command").is_none());
        assert_eq!(
            dispositions_for(&translation, "workspace.right_sidebar.command"),
            vec![ClassicNovaDisposition::Rejected]
        );
    }
}

// Regression: agent argv follows the shared Classic/Nova raw string-array contract rather than custom-popup nonempty rules.
#[test]
fn preserves_empty_and_whitespace_agent_arguments() {
    let translation = translate_classic_root(&parse(
        "[workspace.right_sidebar]\ncommand = \"command\"\nargs = [\"\", \" two \"]\n",
    ));
    assert_eq!(
        value_at(&translation.root, "agent.args"),
        Some(&Value::Array(vec![
            Value::String(String::new()),
            Value::String(" two ".into())
        ]))
    );
}

// Defends: two explicitly swapped popup-role chords are validated as one final target keymap, not rejected against stale defaults.
#[test]
fn accepts_nonconflicting_role_chord_swaps() {
    let translation = translate_classic_root(&parse(
        r#"
[zellij.keybindings]
bottom_popup = ["Alt Shift K"]
top_popup = ["Alt Shift J"]
"#,
    ));
    assert_eq!(
        value_at(&translation.root, "keybindings.git").and_then(Value::as_str),
        Some("Alt Shift K")
    );
    assert_eq!(
        value_at(&translation.root, "keybindings.config").and_then(Value::as_str),
        Some("Alt Shift J")
    );
}

// Invariant: every approved native/manual and removed family produces its exact default-value disposition.
#[test]
fn reports_all_native_transition_and_removed_field_families() {
    let mut classic = parse(include_str!("../../../config_default.toml"));
    classic.insert(
        "helix".into(),
        Value::Table(parse(
            r#"
[external]
binary = "hx"
runtime = "/tmp/runtime"
[steel_plugins]
enabled = ["splash"]
extra = []
"#,
        )),
    );
    let translation = translate_classic_root(&classic);

    for path in [
        "appearance.mode",
        "helix.external",
        "helix.steel_plugins",
        "yazi.keybindings",
    ] {
        assert_eq!(
            dispositions_for(&translation, path),
            vec![ClassicNovaDisposition::Manual],
            "missing manual report for {path}"
        );
    }
    for path in [
        "yazi.command",
        "yazi.ya_command",
        "yazi.plugins",
        "yazi.sort_by",
        "yazi.theme",
    ] {
        assert_eq!(
            dispositions_for(&translation, path),
            vec![ClassicNovaDisposition::Preserved],
            "missing approved target omission for {path}"
        );
    }
    for path in REMOVED_CLASSIC_ROOT_FIELDS {
        assert_eq!(
            dispositions_for(&translation, path),
            vec![ClassicNovaDisposition::Removed],
            "missing removal report for {path}"
        );
    }
}

// Defends: nondefault Yazi root policy remains only in the backup and receives manual native-tree guidance.
#[test]
fn reports_nondefault_yazi_policy_as_manual() {
    let translation = translate_classic_root(&parse(
        r#"
[yazi]
command = "yazi-custom"
ya_command = "ya-custom"
plugins = ["git"]
sort_by = "modified"
theme = "random"
[yazi.keybindings]
open_directory_as_workspace_pane = ["<A-o>"]
"#,
    ));
    for path in NATIVE_TRANSITION_CLASSIC_ROOT_FIELDS
        .iter()
        .copied()
        .filter(|path| path.starts_with("yazi."))
    {
        assert_eq!(
            dispositions_for(&translation, path),
            vec![ClassicNovaDisposition::Manual],
            "missing manual Yazi report for {path}"
        );
    }
}

// Invariant: semantically identical popup lists produce byte-order-independent target and report data.
#[test]
fn output_and_report_order_are_deterministic() {
    let first = translate_classic_root(&parse(
        r#"
[[zellij.custom_popups]]
id = "zeta"
command = ["zeta"]
keybindings = ["Alt Shift Z"]
keep_alive = false
[[zellij.custom_popups]]
id = "alpha"
command = ["alpha"]
keybindings = ["Alt Shift A"]
keep_alive = true
"#,
    ));
    let second = translate_classic_root(&parse(
        r#"
[[zellij.custom_popups]]
id = "alpha"
command = ["alpha"]
keybindings = ["Alt Shift A"]
keep_alive = true
[[zellij.custom_popups]]
id = "zeta"
command = ["zeta"]
keybindings = ["Alt Shift Z"]
keep_alive = false
"#,
    ));
    assert_eq!(first, second);
    assert_eq!(
        toml::to_string(&first.root).unwrap(),
        toml::to_string(&second.root).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&first.report).unwrap(),
        serde_json::to_string(&second.report).unwrap()
    );
}

// Regression: discarded duplicate ids do not reserve their chords and reject an otherwise valid popup.
#[test]
fn duplicate_popup_ids_do_not_pollute_surviving_chord_validation() {
    let translation = translate_classic_root(&parse(
        r#"
[[zellij.custom_popups]]
id = "duplicate"
command = ["one"]
keybindings = ["Alt Shift B"]
[[zellij.custom_popups]]
id = "duplicate"
command = ["two"]
keybindings = ["Alt Shift C"]
[[zellij.custom_popups]]
id = "valid"
command = ["valid"]
keybindings = ["Alt Shift B"]
"#,
    ));

    assert_eq!(
        value_at(&translation.root, "popups.valid.command").and_then(Value::as_str),
        Some("valid")
    );
    assert_eq!(
        dispositions_for(&translation, "zellij.custom_popups.duplicate"),
        vec![
            ClassicNovaDisposition::Rejected,
            ClassicNovaDisposition::Rejected
        ]
    );
}
