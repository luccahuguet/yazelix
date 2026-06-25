use std::{collections::BTreeSet, env, fs, process::ExitCode};

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    let [_, layout_path, swap_path] = args.as_slice() else {
        eprintln!("usage: zellij-layout <layout.kdl> <layout.swap.kdl>");
        return ExitCode::FAILURE;
    };

    let layout = read(layout_path);
    let templates = layout
        .lines()
        .filter_map(tab_template)
        .collect::<BTreeSet<_>>();
    let swap = read(swap_path);
    let mut depth = 0i32;
    let mut ok = true;

    for (index, line) in swap.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("swap_tiled_layout ") {
            depth = 1;
            continue;
        }
        if depth == 1 && trimmed.ends_with('{') {
            let name = trimmed.split_whitespace().next().unwrap_or_default();
            if !templates.contains(name) {
                eprintln!("{swap_path}:{}: missing tab_template {name}", index + 1);
                ok = false;
            }
        }
        if depth > 0 {
            depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        }
    }

    ExitCode::from((!ok) as u8)
}

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("could not read {}: {}", path, error))
}

fn tab_template(line: &str) -> Option<String> {
    line.trim()
        .strip_prefix("tab_template name=\"")?
        .split('"')
        .next()
        .map(str::to_owned)
}
