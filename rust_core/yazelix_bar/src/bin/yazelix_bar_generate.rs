use std::env;
use std::process;

use yazelix_bar::{
    StandaloneCommandWidget, StandalonePresetOptions, generate_standalone_preset,
    standalone_part_from_token,
};

fn main() {
    match run(env::args().skip(1).collect()) {
        Ok(output) => {
            print!("{output}");
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    let mut options = StandalonePresetOptions::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return Ok(help_text()),
            "--wasm-url" => options.wasm_url = take_value(&args, &mut index, "--wasm-url")?,
            "--brand-label" => {
                options.brand_label = take_value(&args, &mut index, "--brand-label")?
            }
            "--brand-color" => {
                options.brand_color = take_value(&args, &mut index, "--brand-color")?
            }
            "--session-color" => {
                options.session_color = take_value(&args, &mut index, "--session-color")?
            }
            "--datetime-color" => {
                options.datetime_color = take_value(&args, &mut index, "--datetime-color")?
            }
            "--datetime-format" => {
                options.datetime_format = take_value(&args, &mut index, "--datetime-format")?
            }
            "--tab-label-mode" => {
                options.tab_label_mode = take_value(&args, &mut index, "--tab-label-mode")?
            }
            "--left" => {
                options.format_left = parse_parts(&take_value(&args, &mut index, "--left")?)?
            }
            "--center" => {
                options.format_center = parse_parts(&take_value(&args, &mut index, "--center")?)?
            }
            "--right" => {
                options.format_right = parse_parts(&take_value(&args, &mut index, "--right")?)?
            }
            "--command" => {
                let (name, command) =
                    split_named_value(&take_value(&args, &mut index, "--command")?)?;
                options
                    .command_widgets
                    .push(StandaloneCommandWidget::new(name, command));
            }
            "--command-format" => {
                let (name, format) =
                    split_named_value(&take_value(&args, &mut index, "--command-format")?)?;
                let widget = find_command_widget_mut(&mut options.command_widgets, &name)?;
                widget.format = format;
            }
            "--command-interval" => {
                let (name, interval) =
                    split_named_value(&take_value(&args, &mut index, "--command-interval")?)?;
                let widget = find_command_widget_mut(&mut options.command_widgets, &name)?;
                widget.interval = interval;
            }
            other => return Err(format!("unexpected argument: {other}")),
        }
        index += 1;
    }

    generate_standalone_preset(&options).map_err(|error| format!("{error:?}"))
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn split_named_value(value: &str) -> Result<(String, String), String> {
    let Some((name, value)) = value.split_once('=') else {
        return Err("expected NAME=VALUE".to_string());
    };
    let name = name.trim();
    if name.is_empty() {
        return Err("command widget name cannot be empty".to_string());
    }
    Ok((name.to_string(), value.to_string()))
}

fn find_command_widget_mut<'a>(
    widgets: &'a mut [StandaloneCommandWidget],
    name: &str,
) -> Result<&'a mut StandaloneCommandWidget, String> {
    widgets
        .iter_mut()
        .find(|widget| widget.name == name)
        .ok_or_else(|| {
            format!("--command-format/--command-interval references unknown command: {name}")
        })
}

fn parse_parts(value: &str) -> Result<Vec<yazelix_bar::StandalonePresetPart>, String> {
    if value.trim().is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|token| {
            standalone_part_from_token(token)
                .ok_or_else(|| format!("unknown format token: {}", token.trim()))
        })
        .collect()
}

fn help_text() -> String {
    [
        "Usage: yazelix_bar_generate [options]",
        "",
        "Options:",
        "  --wasm-url URL                 zjstatus wasm URL used in the plugin block",
        "  --brand-label TEXT             right-side brand text",
        "  --brand-color #RRGGBB          brand text color",
        "  --session-color #RRGGBB        session widget color",
        "  --datetime-color #RRGGBB       datetime widget color",
        "  --datetime-format FORMAT       strftime-style datetime format",
        "  --tab-label-mode full|compact  tab label renderer",
        "  --left TOKENS                  comma tokens: mode,tabs,session,datetime,brand,command:name",
        "  --center TOKENS                same token format",
        "  --right TOKENS                 same token format",
        "  --command NAME=COMMAND         add a generic zjstatus command widget",
        "  --command-format NAME=FORMAT   override command widget format",
        "  --command-interval NAME=SECS   override command widget interval",
        "",
        "Example:",
        "  yazelix_bar_generate --right session,datetime,command:host,brand --command 'host=hostname -s'",
        "",
    ]
    .join("\n")
}
