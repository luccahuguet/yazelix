use serde::Serialize;
use std::collections::BTreeMap;
use toml::{Table, Value};

pub const MAPPED_CLASSIC_ROOT_FIELDS: &[&str] = &[
    "core.skip_welcome_screen",
    "core.welcome_style",
    "core.welcome_duration_seconds",
    "editor.command",
    "workspace.right_sidebar.command",
    "workspace.right_sidebar.args",
    "shell.default_shell",
    "zellij.widget_tray",
    "zellij.keybindings",
    "zellij.custom_popups",
];

pub const NATIVE_TRANSITION_CLASSIC_ROOT_FIELDS: &[&str] = &[
    "appearance.mode",
    "helix.external",
    "helix.steel_plugins",
    "yazi.command",
    "yazi.ya_command",
    "yazi.plugins",
    "yazi.sort_by",
    "yazi.theme",
    "yazi.keybindings",
];

pub const REMOVED_CLASSIC_ROOT_FIELDS: &[&str] = &[
    "core.debug_mode",
    "core.show_macchina_on_welcome",
    "core.game_of_life_cell_style",
    "editor.hide_sidebar_on_file_open",
    "workspace.left_sidebar.command",
    "workspace.left_sidebar.args",
    "workspace.left_sidebar.width_percent",
    "workspace.right_sidebar.width_percent",
    "zellij.support_kitty_keyboard_protocol",
    "zellij.theme",
    "zellij.widget_frame",
    "zellij.widget_separator",
    "zellij.tab_label_mode",
    "zellij.codex_usage_display",
    "zellij.codex_usage_periods",
    "zellij.claude_usage_display",
    "zellij.claude_usage_periods",
    "zellij.opencode_go_usage_display",
    "zellij.opencode_go_usage_periods",
    "zellij.custom_text",
    "zellij.popup_commands",
    "zellij.popup_width_percent",
    "zellij.popup_height_percent",
    "zellij.screen_saver_enabled",
    "zellij.screen_saver_idle_seconds",
    "zellij.screen_saver_style",
    "zellij.native_keybindings",
];

const WELCOME_STYLES: &[&str] = &[
    "static",
    "logo",
    "boids",
    "boids_predator",
    "boids_schools",
    "mandelbrot",
    "game_of_life_gliders",
    "game_of_life_oscillators",
    "game_of_life_bloom",
    "random",
];
const SHARED_BAR_WIDGETS: &[&str] = &[
    "session",
    "editor",
    "shell",
    "term",
    "claude_usage",
    "codex_usage",
    "opencode_go_usage",
    "cpu",
    "ram",
];
pub(crate) const POPUP_ROLE_MAPPINGS: &[(&str, &str, &str)] = &[
    ("bottom_popup", "git", "Alt Shift J"),
    ("top_popup", "config", "Alt Shift K"),
    ("menu", "menu", "Alt Shift M"),
    ("open_codex_agent_right", "agent", "Alt Shift L"),
];
pub(crate) const PACKAGED_NON_POPUP_CHORDS: &[&str] = &[
    "Ctrl Alt g",
    "Ctrl Alt o",
    "Ctrl q",
    "Ctrl p",
    "Ctrl n",
    "Alt m",
    "Alt h",
    "Alt Left",
    "Alt l",
    "Alt Right",
    "Alt Shift F",
    "Ctrl y",
    "Alt r",
    "Ctrl t",
    "Ctrl Alt h",
    "Ctrl Alt j",
    "Ctrl Alt k",
    "Ctrl Alt l",
    "Alt Shift h",
    "Alt 1-9",
    "Alt z",
];
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassicNovaDisposition {
    Preserved,
    Removed,
    Rejected,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassicNovaReportEntry {
    pub source_path: String,
    pub disposition: ClassicNovaDisposition,
    pub target_paths: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassicNovaRootTranslation {
    pub root: Table,
    pub report: Vec<ClassicNovaReportEntry>,
}

#[derive(Debug)]
struct PopupCandidate {
    source_path: String,
    id: String,
    command: String,
    args: Vec<String>,
    title: String,
    keybinding: String,
    keep_alive: Option<bool>,
}

/// Translates a table that already passed the canonical Classic config contract.
///
/// Structural/type errors and unknown fields must fail before this call. `Rejected`
/// entries represent only the approved conditional loss budget. The transaction
/// owner must also validate the generated root against the published Nova schema
/// before writing it.
#[must_use]
pub fn translate_classic_root(classic: &Table) -> ClassicNovaRootTranslation {
    let mut translation = ClassicNovaRootTranslation {
        root: Table::new(),
        report: Vec::new(),
    };

    translate_welcome(classic, &mut translation);
    translate_editor(classic, &mut translation);
    translate_shell(classic, &mut translation);
    translate_agent(classic, &mut translation);
    translate_widgets(classic, &mut translation);
    let popup_chords = translate_popup_keybindings(classic, &mut translation);
    translate_custom_popups(classic, &popup_chords, &mut translation);

    for path in NATIVE_TRANSITION_CLASSIC_ROOT_FIELDS {
        if let Some(value) = value_at(classic, path) {
            let (disposition, detail) = native_disposition(path, value);
            translation
                .report
                .push(report(path, disposition, &[], detail));
        }
    }
    for path in REMOVED_CLASSIC_ROOT_FIELDS {
        if value_at(classic, path).is_some() {
            translation.report.push(report(
                path,
                ClassicNovaDisposition::Removed,
                &[],
                removed_detail(path),
            ));
        }
    }

    translation.report.sort_by(|left, right| {
        (&left.source_path, left.disposition, &left.detail).cmp(&(
            &right.source_path,
            right.disposition,
            &right.detail,
        ))
    });
    translation
}

fn translate_welcome(classic: &Table, translation: &mut ClassicNovaRootTranslation) {
    if let Some(value) = value_at(classic, "core.skip_welcome_screen") {
        match value.as_bool() {
            Some(skip) => preserve(
                translation,
                "core.skip_welcome_screen",
                "welcome.enabled",
                Value::Boolean(!skip),
                "inverted to Nova welcome.enabled",
            ),
            None => reject(
                translation,
                "core.skip_welcome_screen",
                "expected a boolean",
            ),
        }
    }

    if let Some(value) = value_at(classic, "core.welcome_style") {
        match value
            .as_str()
            .filter(|style| WELCOME_STYLES.contains(style))
        {
            Some(style) => preserve(
                translation,
                "core.welcome_style",
                "welcome.style",
                Value::String(style.to_string()),
                "mapped to the identical Nova style id",
            ),
            None => reject(
                translation,
                "core.welcome_style",
                "style is not in the shared Classic/Nova catalog",
            ),
        }
    }

    if let Some(value) = value_at(classic, "core.welcome_duration_seconds") {
        let duration = value
            .as_integer()
            .map(|value| value as f64)
            .or_else(|| value.as_float());
        match duration.filter(|value| value.fract() == 0.0 && (1.0..=8.0).contains(value)) {
            Some(duration) => preserve(
                translation,
                "core.welcome_duration_seconds",
                "welcome.duration_seconds",
                Value::Integer(duration as i64),
                "converted from a whole-number Classic duration",
            ),
            None => reject(
                translation,
                "core.welcome_duration_seconds",
                "choose a whole-number Nova duration; the bridge never rounds",
            ),
        }
    }
}

fn translate_editor(classic: &Table, translation: &mut ClassicNovaRootTranslation) {
    let Some(value) = value_at(classic, "editor.command") else {
        return;
    };
    match value.as_str() {
        Some("") => translation.report.push(report(
            "editor.command",
            ClassicNovaDisposition::Preserved,
            &[],
            "omitted so Nova inherits its packaged editor",
        )),
        Some(command) if executable(command) => preserve(
            translation,
            "editor.command",
            "editor.command",
            Value::String(command.to_string()),
            "mapped to the identical executable-only contract",
        ),
        _ => reject(
            translation,
            "editor.command",
            "Nova accepts one non-empty executable token without arguments",
        ),
    }
}

fn translate_shell(classic: &Table, translation: &mut ClassicNovaRootTranslation) {
    let Some(value) = value_at(classic, "shell.default_shell") else {
        return;
    };
    match value.as_str() {
        Some(shell @ ("nu" | "bash" | "fish" | "zsh")) => preserve(
            translation,
            "shell.default_shell",
            "shell.program",
            Value::String(shell.to_string()),
            "mapped to Nova shell.program",
        ),
        Some("xonsh") => reject(
            translation,
            "shell.default_shell",
            "Nova does not package xonsh; choose nu, bash, fish, or zsh manually",
        ),
        _ => reject(
            translation,
            "shell.default_shell",
            "unsupported Classic shell value",
        ),
    }
}

fn translate_agent(classic: &Table, translation: &mut ClassicNovaRootTranslation) {
    let command_path = "workspace.right_sidebar.command";
    let args_path = "workspace.right_sidebar.args";
    let command_explicit = value_at(classic, command_path);
    let args_explicit = value_at(classic, args_path);
    if command_explicit.is_none() && args_explicit.is_none() {
        return;
    }

    let command = match command_explicit {
        Some(value) => match value.as_str() {
            Some(command) => command,
            None => {
                reject(translation, command_path, "expected an executable string");
                if args_explicit.is_some() {
                    reject(
                        translation,
                        args_path,
                        "the paired command is not representable",
                    );
                }
                return;
            }
        },
        None => "yzx",
    };
    let args_value = args_explicit
        .cloned()
        .unwrap_or_else(|| Value::Array(vec![Value::String("agent".to_string())]));
    let args = strings(&args_value);
    let valid = executable(command) && args.is_some();

    if command == "yzx" && args.as_deref() == Some(&["agent".to_string()]) {
        for path in [command_path, args_path] {
            if value_at(classic, path).is_some() {
                translation.report.push(report(
                    path,
                    ClassicNovaDisposition::Preserved,
                    &[],
                    "omitted so Nova inherits agent.command = auto",
                ));
            }
        }
        return;
    }

    if !valid || matches!(command, "yzx" | "auto") {
        for path in [command_path, args_path] {
            if value_at(classic, path).is_some() {
                reject(
                    translation,
                    path,
                    "the right-sidebar pair is not an exact Nova executable-plus-argv command",
                );
            }
        }
        return;
    }

    set_value(
        &mut translation.root,
        "agent.command",
        Value::String(command.to_string()),
    );
    set_value(
        &mut translation.root,
        "agent.args",
        Value::Array(
            args.expect("validated above")
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    );
    for path in [command_path, args_path] {
        if value_at(classic, path).is_some() {
            translation.report.push(report(
                path,
                ClassicNovaDisposition::Preserved,
                &["agent.command", "agent.args"],
                "mapped as one exact executable-plus-argv pair",
            ));
        }
    }
}

fn translate_widgets(classic: &Table, translation: &mut ClassicNovaRootTranslation) {
    let path = "zellij.widget_tray";
    let Some(value) = value_at(classic, path) else {
        return;
    };
    let Some(widgets) = strings(value) else {
        reject(translation, path, "expected a string list");
        return;
    };
    if let Some(widget) = widgets.iter().find(|widget| {
        widget.as_str() != "workspace" && !SHARED_BAR_WIDGETS.contains(&widget.as_str())
    }) {
        reject(
            translation,
            path,
            &format!("widget {widget:?} is not in the shared Classic/Nova catalog"),
        );
        return;
    }

    let shared = widgets
        .iter()
        .filter(|widget| widget.as_str() != "workspace")
        .cloned()
        .collect::<Vec<_>>();
    set_value(
        &mut translation.root,
        "bar.widgets",
        Value::Array(shared.into_iter().map(Value::String).collect()),
    );
    translation.report.push(report(
        path,
        ClassicNovaDisposition::Preserved,
        &["bar.widgets"],
        "mapped shared widget ids in their original order",
    ));
    if widgets.iter().any(|widget| widget == "workspace") {
        translation.report.push(report(
            "zellij.widget_tray.workspace",
            ClassicNovaDisposition::Removed,
            &[],
            "the Classic-only workspace widget has no Nova owner",
        ));
    }
}

fn translate_popup_keybindings(
    classic: &Table,
    translation: &mut ClassicNovaRootTranslation,
) -> BTreeMap<String, String> {
    let mut effective = POPUP_ROLE_MAPPINGS
        .iter()
        .map(|(_, target, default)| ((*target).to_string(), (*default).to_string()))
        .collect::<BTreeMap<_, _>>();
    let Some(value) = value_at(classic, "zellij.keybindings") else {
        return effective;
    };
    let Some(table) = value.as_table() else {
        reject(
            translation,
            "zellij.keybindings",
            "expected a table of chord lists",
        );
        return effective;
    };

    let mut candidates = BTreeMap::<String, (String, String)>::new();
    for (source, target, _) in POPUP_ROLE_MAPPINGS {
        let Some(value) = table.get(*source) else {
            continue;
        };
        let path = format!("zellij.keybindings.{source}");
        match single_chord(value) {
            Some(chord) if valid_chord(&chord) => {
                candidates.insert((*target).to_string(), (path, chord));
            }
            _ => reject(
                translation,
                &path,
                "Nova popup roles require exactly one valid key chord",
            ),
        }
    }

    for (role, (_, chord)) in &candidates {
        effective.insert(role.clone(), chord.clone());
    }
    let frequencies = chord_frequencies(effective.values().map(String::as_str));
    for (role, (path, chord)) in candidates {
        if PACKAGED_NON_POPUP_CHORDS
            .iter()
            .any(|packaged| chord_matches(packaged, &chord))
            || frequencies
                .get(&chord.to_ascii_lowercase())
                .copied()
                .unwrap_or(0)
                > 1
        {
            let default = POPUP_ROLE_MAPPINGS
                .iter()
                .find(|(_, target, _)| *target == role)
                .map(|(_, _, default)| *default)
                .expect("candidate target comes from POPUP_ROLE_MAPPINGS");
            effective.insert(role, default.to_string());
            reject(
                translation,
                &path,
                "key chord conflicts with the Nova managed keymap",
            );
            continue;
        }
        let target = format!("keybindings.{role}");
        set_value(&mut translation.root, &target, Value::String(chord.clone()));
        translation.report.push(report(
            &path,
            ClassicNovaDisposition::Preserved,
            &[&target],
            "mapped the single non-conflicting popup chord",
        ));
    }

    for (role, _) in table {
        if POPUP_ROLE_MAPPINGS
            .iter()
            .any(|(source, _, _)| source == &role.as_str())
        {
            continue;
        }
        let path = format!("zellij.keybindings.{role}");
        translation.report.push(report(
            &path,
            ClassicNovaDisposition::Removed,
            &[],
            "Nova owns this behavior in its fixed native keymap",
        ));
    }
    effective
}

fn translate_custom_popups(
    classic: &Table,
    popup_chords: &BTreeMap<String, String>,
    translation: &mut ClassicNovaRootTranslation,
) {
    let Some(value) = value_at(classic, "zellij.custom_popups") else {
        return;
    };
    let Some(items) = value.as_array() else {
        reject(
            translation,
            "zellij.custom_popups",
            "expected an array of popup tables",
        );
        return;
    };

    let mut candidates = Vec::new();
    for (index, value) in items.iter().enumerate() {
        match popup_candidate(index, value) {
            Ok(candidate) => candidates.push(candidate),
            Err((path, detail)) => reject(translation, &path, &detail),
        }
    }
    candidates.sort_by(|left, right| left.id.cmp(&right.id));

    let id_counts = frequencies(candidates.iter().map(|candidate| candidate.id.clone()));
    let (duplicates, candidates): (Vec<_>, Vec<_>) = candidates
        .into_iter()
        .partition(|candidate| id_counts.get(&candidate.id).copied().unwrap_or(0) > 1);
    for candidate in duplicates {
        reject(
            translation,
            &candidate.source_path,
            "duplicate custom popup id",
        );
    }
    let chord_counts = chord_frequencies(
        popup_chords.values().map(String::as_str).chain(
            candidates
                .iter()
                .map(|candidate| candidate.keybinding.as_str()),
        ),
    );
    for candidate in candidates {
        if PACKAGED_NON_POPUP_CHORDS
            .iter()
            .any(|packaged| chord_matches(packaged, &candidate.keybinding))
            || chord_counts
                .get(&candidate.keybinding.to_ascii_lowercase())
                .copied()
                .unwrap_or(0)
                > 1
        {
            reject(
                translation,
                &candidate.source_path,
                "custom popup key chord conflicts with the Nova managed keymap",
            );
            continue;
        }

        let prefix = format!("popups.{}", candidate.id);
        set_value(
            &mut translation.root,
            &format!("{prefix}.command"),
            Value::String(candidate.command),
        );
        if !candidate.args.is_empty() {
            set_value(
                &mut translation.root,
                &format!("{prefix}.args"),
                Value::Array(candidate.args.into_iter().map(Value::String).collect()),
            );
        }
        set_value(
            &mut translation.root,
            &format!("{prefix}.title"),
            Value::String(candidate.title),
        );
        set_value(
            &mut translation.root,
            &format!("{prefix}.keybinding"),
            Value::String(candidate.keybinding),
        );
        if let Some(keep_alive) = candidate.keep_alive {
            set_value(
                &mut translation.root,
                &format!("{prefix}.keep_alive"),
                Value::Boolean(keep_alive),
            );
        }
        translation.report.push(report(
            &candidate.source_path,
            ClassicNovaDisposition::Preserved,
            &[&prefix],
            "mapped to one Nova custom popup with title yzx_<id>",
        ));
    }
}

fn popup_candidate(index: usize, value: &Value) -> Result<PopupCandidate, (String, String)> {
    let fallback_path = format!("zellij.custom_popups[{index}]");
    let table = value
        .as_table()
        .ok_or_else(|| (fallback_path.clone(), "expected a popup table".to_string()))?;
    let id = table
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let source_path = if id.is_empty() {
        fallback_path
    } else {
        format!("zellij.custom_popups.{id}")
    };
    if !valid_popup_id(&id) || matches!(id.as_str(), "config" | "agent" | "git" | "menu") {
        return Err((source_path, "id is invalid or reserved by Nova".to_string()));
    }
    let command = table
        .get("command")
        .and_then(normalized_strings)
        .filter(|command| !command.is_empty())
        .ok_or_else(|| {
            (
                source_path.clone(),
                "command must be a non-empty executable-plus-argv list".to_string(),
            )
        })?;
    if !executable(&command[0]) || matches!(command[0].as_str(), "yzx" | "editor") {
        return Err((
            source_path,
            "command uses unrepresentable Classic command semantics".to_string(),
        ));
    }
    let keybindings = table
        .get("keybindings")
        .and_then(normalized_strings)
        .filter(|keys| keys.len() == 1 && valid_chord(&keys[0]))
        .ok_or_else(|| {
            (
                source_path.clone(),
                "exactly one valid key chord is required".to_string(),
            )
        })?;
    let explicit_keep_alive = match table.get("keep_alive") {
        Some(value) => Some(value.as_bool().ok_or_else(|| {
            (
                source_path.clone(),
                "keep_alive must be a boolean".to_string(),
            )
        })?),
        None => None,
    };
    let keep_alive = explicit_keep_alive
        .or_else(|| (id == "zenith" && command == ["zenith".to_string()]).then_some(true));
    Ok(PopupCandidate {
        source_path,
        title: format!("yzx_{id}"),
        id,
        command: command[0].clone(),
        args: command[1..].to_vec(),
        keybinding: keybindings[0].clone(),
        keep_alive,
    })
}

fn native_disposition(path: &str, value: &Value) -> (ClassicNovaDisposition, &'static str) {
    match path {
        "appearance.mode" => (
            ClassicNovaDisposition::Manual,
            "review Mars appearance and native Yazi theme settings manually",
        ),
        "helix.external" | "helix.steel_plugins" => (
            ClassicNovaDisposition::Manual,
            "the preserved helix/ tree remains the source for manual Nova configuration",
        ),
        "yazi.command" | "yazi.ya_command" if value.as_str() == Some("") => (
            ClassicNovaDisposition::Preserved,
            "omitted so Nova uses its packaged Yazi commands",
        ),
        "yazi.command" | "yazi.ya_command" => (
            ClassicNovaDisposition::Manual,
            "Nova has no Yazi binary override; remove it or provide a complete custom package",
        ),
        "yazi.plugins" if string_list_is(value, &["git", "starship"]) => (
            ClassicNovaDisposition::Preserved,
            "omitted because Nova packages the same required integrations",
        ),
        "yazi.plugins" => (
            ClassicNovaDisposition::Manual,
            "recreate nondefault plugins through the preserved plugin directories and init.lua",
        ),
        "yazi.sort_by" if value.as_str() == Some("alphabetical") => (
            ClassicNovaDisposition::Preserved,
            "omitted so Nova inherits its packaged alphabetical sort",
        ),
        "yazi.sort_by" => (
            ClassicNovaDisposition::Manual,
            "set the desired native [mgr].sort_by value in yazi/yazi.toml",
        ),
        "yazi.theme" if value.as_str() == Some("default") => (
            ClassicNovaDisposition::Preserved,
            "omitted so Nova inherits its packaged Yazi theme",
        ),
        "yazi.theme" => (
            ClassicNovaDisposition::Manual,
            "configure preserved flavors and yazi/theme.toml manually",
        ),
        "yazi.keybindings" => (
            ClassicNovaDisposition::Manual,
            "recreate desired bindings in native yazi/keymap.toml",
        ),
        _ => unreachable!("manual field catalog is exhaustive"),
    }
}

fn removed_detail(path: &str) -> &'static str {
    match path {
        "editor.hide_sidebar_on_file_open" => {
            "Nova v1 has no equivalent; remove this Classic override"
        }
        path if path.starts_with("workspace.left_sidebar.") => {
            "Nova owns a fixed managed Yazi sidebar; remove this Classic override"
        }
        "workspace.right_sidebar.width_percent" => {
            "Nova owns agent popup geometry; choose popup cell margins manually if needed"
        }
        "zellij.popup_commands" => {
            "Nova packages its built-in popups; recreate other eligible commands under [popups.<id>]"
        }
        "zellij.popup_width_percent" | "zellij.popup_height_percent" => {
            "Nova uses popup cell margins; choose popup.side_margin and popup.vertical_margin manually"
        }
        path if path.starts_with("zellij.screen_saver_") => {
            "Nova has no automatic screen-saver setting; use yzx screen manually"
        }
        "zellij.native_keybindings" => {
            "Nova owns a fixed native keymap; do not copy this map into the semantic root"
        }
        path if path.starts_with("zellij.") => {
            "Nova packages this Zellij/bar behavior; remove the Classic override"
        }
        _ => "Nova has no replacement; remove this Classic-only override",
    }
}

fn preserve(
    translation: &mut ClassicNovaRootTranslation,
    source: &str,
    target: &str,
    value: Value,
    detail: &str,
) {
    set_value(&mut translation.root, target, value);
    translation.report.push(report(
        source,
        ClassicNovaDisposition::Preserved,
        &[target],
        detail,
    ));
}

fn reject(translation: &mut ClassicNovaRootTranslation, source: &str, detail: &str) {
    translation.report.push(report(
        source,
        ClassicNovaDisposition::Rejected,
        &[],
        detail,
    ));
}

fn report(
    source: &str,
    disposition: ClassicNovaDisposition,
    targets: &[&str],
    detail: &str,
) -> ClassicNovaReportEntry {
    ClassicNovaReportEntry {
        source_path: source.to_string(),
        disposition,
        target_paths: targets.iter().map(|target| (*target).to_string()).collect(),
        detail: detail.to_string(),
    }
}

fn value_at<'a>(root: &'a Table, path: &str) -> Option<&'a Value> {
    let mut segments = path.split('.');
    let first = segments.next()?;
    let mut value = root.get(first)?;
    for segment in segments {
        value = value.as_table()?.get(segment)?;
    }
    Some(value)
}

fn set_value(root: &mut Table, path: &str, value: Value) {
    fn set(table: &mut Table, segments: &[&str], value: Value) {
        if let [leaf] = segments {
            table.insert((*leaf).to_string(), value);
            return;
        }
        let child = table
            .entry(segments[0].to_string())
            .or_insert_with(|| Value::Table(Table::new()));
        set(
            child
                .as_table_mut()
                .expect("translator target paths never collide"),
            &segments[1..],
            value,
        );
    }
    set(root, &path.split('.').collect::<Vec<_>>(), value);
}

fn strings(value: &Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|value| value.as_str().map(str::to_string))
        .collect()
}

fn normalized_strings(value: &Value) -> Option<Vec<String>> {
    strings(value)?
        .into_iter()
        .map(|value| {
            let value = value.trim();
            (!value.is_empty() && !value.contains(['\n', '\r'])).then(|| value.to_string())
        })
        .collect()
}

fn string_list_is(value: &Value, expected: &[&str]) -> bool {
    strings(value).is_some_and(|values| {
        values
            .iter()
            .map(String::as_str)
            .eq(expected.iter().copied())
    })
}

fn single_chord(value: &Value) -> Option<String> {
    let values = strings(value)?;
    (values.len() == 1).then(|| values[0].trim().to_string())
}

pub(crate) fn executable(value: &str) -> bool {
    !value.is_empty() && !value.chars().any(char::is_whitespace)
}

pub(crate) fn valid_popup_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

pub(crate) fn valid_chord(value: &str) -> bool {
    value.rsplit_once(' ').is_some_and(|(modifiers, key)| {
        matches!(
            modifiers,
            "Ctrl" | "Alt" | "Shift" | "Ctrl Alt" | "Ctrl Shift" | "Alt Shift" | "Ctrl Alt Shift"
        ) && (matches!(key.as_bytes(), [ch] if ch.is_ascii_alphanumeric())
            || matches!(
                key,
                "Left"
                    | "Right"
                    | "Up"
                    | "Down"
                    | "Enter"
                    | "Esc"
                    | "Tab"
                    | "Backspace"
                    | "Space"
                    | "Home"
                    | "End"
                    | "PageUp"
                    | "PageDown"
            ))
    })
}

pub(crate) fn chord_matches(pattern: &str, chord: &str) -> bool {
    pattern.eq_ignore_ascii_case(chord)
        || matches!(
            (pattern, chord.strip_prefix("Alt ")),
            (
                "Alt 1-9",
                Some("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            )
        )
}

fn chord_frequencies<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    frequencies(values.map(str::to_ascii_lowercase))
}

fn frequencies<T: Ord>(values: impl Iterator<Item = T>) -> BTreeMap<T, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
}
