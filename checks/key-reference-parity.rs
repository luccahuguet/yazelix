use std::{env, fs, process};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, catalog_path, config_path] = args.as_slice() else {
        panic!("usage: key-reference-parity-check <catalog.rs> <config.kdl>");
    };
    let catalog = fs::read_to_string(catalog_path).unwrap();
    let config = fs::read_to_string(config_path).unwrap();
    let chords = catalog
        .lines()
        .filter_map(config_kdl_chord)
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

    if !failures.is_empty() {
        eprintln!("{}", failures.join("\n"));
        process::exit(1);
    }
}

fn config_kdl_chord(line: &str) -> Option<&str> {
    let mut quoted = line
        .trim()
        .strip_prefix("key!(")?
        .split('"')
        .skip(1)
        .step_by(2);
    quoted.next()?;
    let chord = quoted.next()?;
    quoted.next()?;
    quoted.next()?;
    (quoted.next()? == "config.kdl").then_some(chord)
}

fn config_has_chord(config: &str, chord: &str) -> bool {
    match chord {
        "Alt h / Alt Left" => config.contains(r#"bind "Alt h" "Alt Left""#),
        "Alt l / Alt Right" => config.contains(r#"bind "Alt l" "Alt Right""#),
        "Alt Shift L" => {
            config.contains(r#"bind "Alt Shift L""#) || config.contains(r#"bind "@agentKey@""#)
        }
        "Alt 1-9" => {
            (1..=9).all(|tab| config.contains(&format!(r#"bind "Alt {tab}" {{ GoToTab {tab}; }}"#)))
        }
        "n in tab mode" => config.contains(r#"bind "n" { NewTab"#),
        chord => config.contains(&format!(r#"bind "{chord}""#)),
    }
}
