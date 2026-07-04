use std::{collections::BTreeSet, env, fs, process};

struct KeyBinding {
    chord: String,
    source: String,
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, catalog_path, config_path, tutor_path] = args.as_slice() else {
        panic!("usage: key-reference-parity-check <catalog.rs> <config.kdl> <yzn-tutor/main.rs>");
    };
    let catalog = fs::read_to_string(catalog_path).unwrap();
    let config = fs::read_to_string(config_path).unwrap();
    let tutor = fs::read_to_string(tutor_path).unwrap();
    let bindings = catalog
        .lines()
        .filter_map(catalog_key_binding)
        .collect::<Vec<_>>();
    let chords = bindings
        .iter()
        .filter(|binding| binding.source == "config.kdl")
        .map(|binding| binding.chord.as_str())
        .collect::<Vec<_>>();

    let mut failures = Vec::new();
    for chord in &chords {
        if !config_has_chord(&config, chord) {
            failures.push(format!(
                "KEY_BINDINGS row for {chord} is not backed by config.kdl"
            ));
        }
    }

    for chord in ["Ctrl Alt h", "Ctrl Alt j", "Ctrl Alt k", "Ctrl Alt l"] {
        if !chords.contains(&chord) {
            failures.push(format!(
                "KEY_BINDINGS is missing packaged movement key {chord}"
            ));
        }
    }

    failures.extend(tutor_key_hint_failures(&tutor, &bindings));

    if !failures.is_empty() {
        eprintln!("{}", failures.join("\n"));
        process::exit(1);
    }
}

fn catalog_key_binding(line: &str) -> Option<KeyBinding> {
    let mut fields = line
        .trim()
        .strip_prefix("key!(")?
        .split('"')
        .skip(1)
        .step_by(2);
    fields.next()?;
    let chord = fields.next()?.to_string();
    fields.next()?;
    fields.next()?;
    let source = fields.next()?.to_string();
    Some(KeyBinding { chord, source })
}

fn config_has_chord(config: &str, chord: &str) -> bool {
    match chord {
        "Alt h / Alt Left" => config.contains(r#"bind "Alt h" "Alt Left""#),
        "Alt l / Alt Right" => config.contains(r#"bind "Alt l" "Alt Right""#),
        "Alt Shift J" => {
            config.contains(r#"bind "Alt Shift J""#) || config.contains(r#"bind "@gitKey@""#)
        }
        "Alt Shift K" => {
            config.contains(r#"bind "Alt Shift K""#) || config.contains(r#"bind "@configKey@""#)
        }
        "Alt Shift L" => {
            config.contains(r#"bind "Alt Shift L""#) || config.contains(r#"bind "@agentKey@""#)
        }
        "Alt Shift M" => {
            config.contains(r#"bind "Alt Shift M""#) || config.contains(r#"bind "@menuKey@""#)
        }
        "Alt 1-9" => {
            (1..=9).all(|tab| config.contains(&format!(r#"bind "Alt {tab}" {{ GoToTab {tab}; }}"#)))
        }
        "n in tab mode" => config.contains(r#"bind "n" { NewTab"#),
        chord => config.contains(&format!(r#"bind "{chord}""#)),
    }
}

fn tutor_key_hint_failures(tutor: &str, bindings: &[KeyBinding]) -> Vec<String> {
    let catalog_aliases = catalog_chord_aliases(bindings);
    let mut failures = Vec::new();
    for (name, chord) in tutor_key_constants(tutor) {
        if !catalog_aliases.contains(&chord) {
            failures.push(format!(
                "yzn-tutor {name} key hint `{chord}` is not backed by KEY_BINDINGS"
            ));
        }
    }

    for hint in inline_key_hints(tutor) {
        if !catalog_aliases.contains(&hint) && !tool_native_key_hint(&hint) {
            failures.push(format!(
                "yzn-tutor inline key hint `{hint}` is neither backed by KEY_BINDINGS nor marked tool-native"
            ));
        }
    }
    failures
}

fn catalog_chord_aliases(bindings: &[KeyBinding]) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    for binding in bindings {
        aliases.insert(binding.chord.clone());
        for alias in binding.chord.split(" / ") {
            aliases.insert(alias.to_string());
        }
    }
    aliases
}

fn tutor_key_constants(tutor: &str) -> Vec<(&str, String)> {
    tutor
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let name = line.strip_prefix("const KEY_")?.split(':').next()?;
            let value = line.split('"').nth(1)?.to_string();
            Some((name, value))
        })
        .collect()
}

fn inline_key_hints(tutor: &str) -> BTreeSet<String> {
    let mut hints = BTreeSet::new();
    for value in tutor.split('`').skip(1).step_by(2) {
        if key_like_hint(value) {
            hints.insert(value.to_string());
        }
    }
    hints
}

fn key_like_hint(value: &str) -> bool {
    matches!(value, "Enter" | "Esc" | "q" | ":q")
        || value.starts_with("Alt ")
        || value.starts_with("Ctrl ")
        || value.starts_with("Shift ")
}

fn tool_native_key_hint(value: &str) -> bool {
    matches!(value, "Enter" | "q" | "Esc" | ":q" | "Ctrl d")
}
