use std::io::{self, Read};

use yazelix_zellij_config_pack::{RENDERER_SCHEMA_VERSION, render_zellij_config_pack};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [] => render_stdin(),
        [flag] if flag == "--schema-version" => {
            println!("{RENDERER_SCHEMA_VERSION}");
            Ok(())
        }
        [flag] if flag == "--help" || flag == "-h" => {
            println!("usage: yazelix_zellij_config_pack [--schema-version] < request.json");
            Ok(())
        }
        _ => Err("unexpected arguments; use --help".to_string()),
    }
}

fn render_stdin() -> Result<(), String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("could not read request JSON from stdin: {error}"))?;
    let request = serde_json::from_str(&input)
        .map_err(|error| format!("could not parse request JSON: {error}"))?;
    let output = render_zellij_config_pack(&request)?;
    serde_json::to_writer_pretty(io::stdout(), &output)
        .map_err(|error| format!("could not write output JSON: {error}"))?;
    println!();
    Ok(())
}
