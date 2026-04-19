use lexopt::prelude::*;
use std::path::PathBuf;
use yazelix_core::{
    CoreError, NormalizeConfigRequest, error_envelope, normalize_config, success_envelope,
};

const CONFIG_NORMALIZE_COMMAND: &str = "config.normalize";
const UNKNOWN_COMMAND: &str = "unknown";

struct CommandError {
    command: String,
    error: CoreError,
}

impl CommandError {
    fn new(command: impl Into<String>, error: CoreError) -> Box<Self> {
        Box::new(Self {
            command: command.into(),
            error,
        })
    }
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(command_error) => {
            let envelope = error_envelope(&command_error.command, &command_error.error);
            let _ = serde_json::to_writer(std::io::stderr(), &envelope);
            eprintln!();
            std::process::exit(command_error.error.class().exit_code());
        }
    }
}

fn run() -> Result<(), Box<CommandError>> {
    let mut parser = lexopt::Parser::from_env();
    let Some(arg) = parser
        .next()
        .map_err(|error| CommandError::new(UNKNOWN_COMMAND, CoreError::usage(error.to_string())))?
    else {
        return Err(CommandError::new(
            UNKNOWN_COMMAND,
            CoreError::usage("Missing helper command"),
        ));
    };
    let command = match arg {
        Value(value) => value.into_string().map_err(|_| {
            CommandError::new(
                UNKNOWN_COMMAND,
                CoreError::usage("Helper command must be valid UTF-8"),
            )
        })?,
        _ => {
            return Err(CommandError::new(
                UNKNOWN_COMMAND,
                CoreError::usage("First argument must be a helper command"),
            ));
        }
    };

    match command.as_str() {
        CONFIG_NORMALIZE_COMMAND => {
            let command_for_error = command.clone();
            run_config_normalize(parser)
                .map_err(|error| CommandError::new(command_for_error, error))
        }
        _ => Err(CommandError::new(
            command.clone(),
            CoreError::usage(format!("Unsupported helper command: {command}")),
        )),
    }
}

fn run_config_normalize(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("config") => {
                config_path = Some(
                    parser
                        .value()
                        .map_err(|error| CoreError::usage(error.to_string()))?
                        .into(),
                );
            }
            Long("default-config") => {
                default_config_path = Some(
                    parser
                        .value()
                        .map_err(|error| CoreError::usage(error.to_string()))?
                        .into(),
                );
            }
            Long("contract") => {
                contract_path = Some(
                    parser
                        .value()
                        .map_err(|error| CoreError::usage(error.to_string()))?
                        .into(),
                );
            }
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = NormalizeConfigRequest {
        config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
        default_config_path: default_config_path
            .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
        contract_path: contract_path.ok_or_else(|| CoreError::usage("Missing --contract path"))?,
    };
    let data = normalize_config(&request)?;
    let envelope = success_envelope(CONFIG_NORMALIZE_COMMAND, data);
    serde_json::to_writer(std::io::stdout(), &envelope).map_err(|source| {
        CoreError::io(
            "write_stdout",
            "Could not write helper JSON envelope",
            "Retry the command and report this as a Yazelix internal error if it persists.",
            "<stdout>",
            source.into(),
        )
    })?;
    println!();
    Ok(())
}
