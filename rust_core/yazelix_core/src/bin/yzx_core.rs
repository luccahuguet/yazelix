use lexopt::prelude::*;
use serde::de::DeserializeOwned;
use std::io::Write;
use std::path::PathBuf;
use yazelix_core::control_plane::{
    config_override_from_env, runtime_materialization_plan_request_from_env,
    terminal_materialization_request_from_env,
};
use yazelix_core::terminal_materialization::MarsProfile;
use yazelix_core::terminal_variant::active_terminal_from_runtime_dir;
use yazelix_core::{
    CoreError, ErrorClass, HelixMaterializationRequest,
    RuntimeMaterializationRepairEvaluateRequest, RuntimeMaterializationRepairRunData,
    RuntimeRepairDirective, TerminalMaterializationRequest, error_envelope,
    generate_helix_materialization, generate_terminal_materialization,
    repair_runtime_materialization, success_envelope,
};

const RUNTIME_MATERIALIZATION_REPAIR_COMMAND: &str = "runtime-materialization.repair";
const HELIX_MATERIALIZATION_GENERATE_COMMAND: &str = "helix-materialization.generate";
const TERMINAL_MATERIALIZATION_GENERATE_COMMAND: &str = "terminal-materialization.generate";
const UNKNOWN_COMMAND: &str = "unknown";

struct RuntimeMaterializationRepairCommand {
    request: RuntimeMaterializationRepairEvaluateRequest,
    summary: bool,
}

#[derive(Default)]
struct ConfigContractRuntimeArgs {
    config_path: Option<PathBuf>,
    default_config_path: Option<PathBuf>,
    contract_path: Option<PathBuf>,
    runtime_dir: Option<PathBuf>,
}

#[derive(Default)]
struct HelixMaterializationArgs {
    runtime_dir: Option<PathBuf>,
    config_dir: Option<PathBuf>,
    state_dir: Option<PathBuf>,
    show_splash: bool,
}

#[derive(Default)]
struct TerminalMaterializationArgs {
    paths: ConfigContractRuntimeArgs,
    state_dir: Option<PathBuf>,
    from_env: bool,
}

enum ErrorOutputMode {
    Json,
    RuntimeRepairSummary,
}

struct CommandError {
    command: String,
    error: CoreError,
    output_mode: ErrorOutputMode,
}

impl CommandError {
    fn new(command: impl Into<String>, error: CoreError) -> Box<Self> {
        Self::new_with_output_mode(command, error, ErrorOutputMode::Json)
    }

    fn runtime_repair_summary(command: impl Into<String>, error: CoreError) -> Box<Self> {
        Self::new_with_output_mode(command, error, ErrorOutputMode::RuntimeRepairSummary)
    }

    fn new_with_output_mode(
        command: impl Into<String>,
        error: CoreError,
        output_mode: ErrorOutputMode,
    ) -> Box<Self> {
        Box::new(Self {
            command: command.into(),
            error,
            output_mode,
        })
    }
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(command_error) => {
            match command_error.output_mode {
                ErrorOutputMode::Json => {
                    let envelope = error_envelope(&command_error.command, &command_error.error);
                    let _ = serde_json::to_writer(std::io::stderr(), &envelope);
                    eprintln!();
                }
                ErrorOutputMode::RuntimeRepairSummary => {
                    let _ =
                        write_runtime_materialization_repair_error_summary(&command_error.error);
                }
            }
            std::process::exit(command_error.error.class().exit_code());
        }
    }
}

enum HelperCommand {
    HelixMaterializationGenerate,
    TerminalMaterializationGenerate,
    RuntimeMaterializationRepair,
    Unsupported(String),
}

fn run() -> Result<(), Box<CommandError>> {
    let mut parser = lexopt::Parser::from_env();
    let command = parse_helper_command(&mut parser)?;
    dispatch_helper_command(command, parser)
}

fn parse_helper_command(parser: &mut lexopt::Parser) -> Result<HelperCommand, Box<CommandError>> {
    let command_name = take_helper_command_name(parser)?;
    Ok(classify_helper_command(command_name))
}

fn take_helper_command_name(parser: &mut lexopt::Parser) -> Result<String, Box<CommandError>> {
    let Some(arg) = parser
        .next()
        .map_err(|error| CommandError::new(UNKNOWN_COMMAND, CoreError::usage(error.to_string())))?
    else {
        return Err(CommandError::new(
            UNKNOWN_COMMAND,
            CoreError::usage("Missing helper command"),
        ));
    };

    match arg {
        Value(value) => value.into_string().map_err(|_| {
            CommandError::new(
                UNKNOWN_COMMAND,
                CoreError::usage("Helper command must be valid UTF-8"),
            )
        }),
        _ => Err(CommandError::new(
            UNKNOWN_COMMAND,
            CoreError::usage("First argument must be a helper command"),
        )),
    }
}

fn classify_helper_command(command_name: String) -> HelperCommand {
    match command_name.as_str() {
        RUNTIME_MATERIALIZATION_REPAIR_COMMAND => HelperCommand::RuntimeMaterializationRepair,
        HELIX_MATERIALIZATION_GENERATE_COMMAND => HelperCommand::HelixMaterializationGenerate,
        TERMINAL_MATERIALIZATION_GENERATE_COMMAND => HelperCommand::TerminalMaterializationGenerate,
        _ => HelperCommand::Unsupported(command_name),
    }
}

fn dispatch_helper_command(
    command: HelperCommand,
    parser: lexopt::Parser,
) -> Result<(), Box<CommandError>> {
    match command {
        HelperCommand::HelixMaterializationGenerate => run_helix_materialization_generate(parser)
            .map_err(|error| CommandError::new(HELIX_MATERIALIZATION_GENERATE_COMMAND, error)),
        HelperCommand::TerminalMaterializationGenerate => {
            run_terminal_materialization_generate(parser).map_err(|error| {
                CommandError::new(TERMINAL_MATERIALIZATION_GENERATE_COMMAND, error)
            })
        }
        HelperCommand::RuntimeMaterializationRepair => run_runtime_materialization_repair(
            parser,
            RUNTIME_MATERIALIZATION_REPAIR_COMMAND.to_string(),
        ),
        HelperCommand::Unsupported(command_name) => Err(CommandError::new(
            command_name.clone(),
            CoreError::usage(format!("Unsupported helper command: {command_name}")),
        )),
    }
}

fn run_helix_materialization_generate(parser: lexopt::Parser) -> Result<(), CoreError> {
    let request =
        helix_materialization_request_from_args(take_helix_materialization_args(parser)?)?;
    let data = generate_helix_materialization(&request)?;
    write_success_envelope(HELIX_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn take_helix_materialization_args(
    mut parser: lexopt::Parser,
) -> Result<HelixMaterializationArgs, CoreError> {
    let mut args = HelixMaterializationArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("runtime-dir") => args.runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("config-dir") => args.config_dir = Some(parser_path_value(&mut parser)?),
            Long("state-dir") => args.state_dir = Some(parser_path_value(&mut parser)?),
            Long("show-splash") => args.show_splash = parser_bool_value(&mut parser)?,
            _ => return Err(unexpected_argument(arg)),
        }
    }
    Ok(args)
}

fn helix_materialization_request_from_args(
    args: HelixMaterializationArgs,
) -> Result<HelixMaterializationRequest, CoreError> {
    Ok(HelixMaterializationRequest {
        runtime_dir: required_path(args.runtime_dir, "Missing --runtime-dir path")?,
        config_dir: required_path(args.config_dir, "Missing --config-dir path")?,
        state_dir: required_path(args.state_dir, "Missing --state-dir path")?,
        show_splash: args.show_splash,
    })
}

fn run_terminal_materialization_generate(parser: lexopt::Parser) -> Result<(), CoreError> {
    let request =
        terminal_materialization_request_from_args(take_terminal_materialization_args(parser)?)?;
    let data = generate_terminal_materialization(&request)?;
    write_success_envelope(TERMINAL_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn take_terminal_materialization_args(
    mut parser: lexopt::Parser,
) -> Result<TerminalMaterializationArgs, CoreError> {
    let mut args = TerminalMaterializationArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        let option = parsed_long_option(arg)?;
        if parse_config_contract_runtime_option(&option.name, &mut parser, &mut args.paths)? {
            continue;
        }
        match option.name.as_str() {
            "from-env" => args.from_env = true,
            "state-dir" => args.state_dir = Some(parser_path_value(&mut parser)?),
            _ => return Err(option.unexpected_error()),
        }
    }
    Ok(args)
}

fn terminal_materialization_request_from_args(
    args: TerminalMaterializationArgs,
) -> Result<TerminalMaterializationRequest, CoreError> {
    let explicit_args_present =
        config_contract_runtime_args_present(&args.paths) || args.state_dir.is_some();

    if args.from_env {
        if explicit_args_present {
            return Err(CoreError::usage(
                "Use either --from-env or explicit terminal-materialization.generate paths, not both.",
            ));
        }
        return terminal_materialization_request_from_env(config_override_from_env().as_deref());
    }

    let runtime_dir = required_path(args.paths.runtime_dir, "Missing --runtime-dir path")?;
    let terminal = active_terminal_from_runtime_dir(&runtime_dir)?;
    let config_path = required_path(args.paths.config_path, "Missing --config path")?;
    Ok(TerminalMaterializationRequest {
        cursor_config_path: config_path.clone(),
        config_path,
        default_config_path: required_path(
            args.paths.default_config_path,
            "Missing --default-config path",
        )?,
        contract_path: required_path(args.paths.contract_path, "Missing --contract path")?,
        runtime_dir,
        state_dir: required_path(args.state_dir, "Missing --state-dir path")?,
        terminals: vec![terminal],
        mars_emoji_font: None,
        mars_profile: MarsProfile::Full,
    })
}

fn run_runtime_materialization_repair(
    mut parser: lexopt::Parser,
    command_name: String,
) -> Result<(), Box<CommandError>> {
    let command = take_runtime_materialization_repair_command(&mut parser)
        .map_err(|error| CommandError::new(command_name.clone(), error))?;
    let data = repair_runtime_materialization(&command.request).map_err(|error| {
        if command.summary {
            CommandError::runtime_repair_summary(command_name.clone(), error)
        } else {
            CommandError::new(command_name.clone(), error)
        }
    })?;
    if command.summary {
        write_runtime_materialization_repair_summary(&data)
            .map_err(|error| CommandError::new(command_name.clone(), error))
    } else {
        write_success_envelope(RUNTIME_MATERIALIZATION_REPAIR_COMMAND, data)
            .map_err(|error| CommandError::new(command_name.clone(), error))
    }
}

fn take_runtime_materialization_repair_command(
    parser: &mut lexopt::Parser,
) -> Result<RuntimeMaterializationRepairCommand, CoreError> {
    let mut request_json: Option<String> = None;
    let mut from_env = false;
    let mut force = false;
    let mut summary = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(parser)?),
            Long("from-env") => from_env = true,
            Long("force") => force = true,
            Long("summary") => summary = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = match (from_env, request_json) {
        (true, None) => Ok(RuntimeMaterializationRepairEvaluateRequest {
            plan: runtime_materialization_plan_request_from_env(
                config_override_from_env().as_deref(),
            )?,
            force,
        }),
        (false, Some(raw)) => {
            if force {
                return Err(CoreError::usage(
                    "Use --force only with --from-env for runtime materialization repair.",
                ));
            }
            deserialize_json_request(&raw, "runtime-materialization.repair")
        }
        (true, Some(_)) => Err(CoreError::usage(
            "Use either --from-env or --request-json for runtime materialization repair, not both.",
        )),
        (false, None) => Err(CoreError::usage(
            "Missing --request-json payload or --from-env for runtime materialization repair.",
        )),
    }?;

    Ok(RuntimeMaterializationRepairCommand { request, summary })
}

fn deserialize_json_request<T: DeserializeOwned>(raw: &str, kind: &str) -> Result<T, CoreError> {
    serde_json::from_str(raw).map_err(|error| {
        CoreError::classified(
            ErrorClass::Usage,
            "invalid_request_json",
            format!("Invalid {kind} request JSON: {error}"),
            "Pass one valid JSON payload via --request-json.",
            serde_json::json!({}),
        )
    })
}

fn parser_path_value(parser: &mut lexopt::Parser) -> Result<PathBuf, CoreError> {
    Ok(parser
        .value()
        .map_err(|error| CoreError::usage(error.to_string()))?
        .into())
}

fn parser_string_value(parser: &mut lexopt::Parser) -> Result<String, CoreError> {
    parser
        .value()
        .map_err(|error| CoreError::usage(error.to_string()))?
        .into_string()
        .map_err(|_| CoreError::usage("Argument value must be valid UTF-8"))
}

fn parser_bool_value(parser: &mut lexopt::Parser) -> Result<bool, CoreError> {
    match parser_string_value(parser)?.as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        other => Err(CoreError::usage(format!(
            "Boolean argument value must be true or false, got `{other}`"
        ))),
    }
}

struct ParsedLongOption {
    name: String,
    unexpected_message: String,
}

impl ParsedLongOption {
    fn unexpected_error(&self) -> CoreError {
        CoreError::usage(self.unexpected_message.clone())
    }
}

fn parsed_long_option(arg: lexopt::Arg<'_>) -> Result<ParsedLongOption, CoreError> {
    let unexpected_message = format!("Unexpected argument: {arg:?}");
    match arg {
        Long(name) => Ok(ParsedLongOption {
            name: name.to_string(),
            unexpected_message,
        }),
        _ => Err(CoreError::usage(unexpected_message)),
    }
}

fn unexpected_argument(arg: lexopt::Arg) -> CoreError {
    CoreError::usage(format!("Unexpected argument: {arg:?}"))
}

fn required_path(value: Option<PathBuf>, message: &'static str) -> Result<PathBuf, CoreError> {
    value.ok_or_else(|| CoreError::usage(message))
}

fn parse_config_contract_runtime_option(
    option: &str,
    parser: &mut lexopt::Parser,
    paths: &mut ConfigContractRuntimeArgs,
) -> Result<bool, CoreError> {
    match option {
        "config" => paths.config_path = Some(parser_path_value(parser)?),
        "default-config" => paths.default_config_path = Some(parser_path_value(parser)?),
        "contract" => paths.contract_path = Some(parser_path_value(parser)?),
        "runtime-dir" => paths.runtime_dir = Some(parser_path_value(parser)?),
        _ => return Ok(false),
    }
    Ok(true)
}

fn config_contract_runtime_args_present(paths: &ConfigContractRuntimeArgs) -> bool {
    paths.config_path.is_some()
        || paths.default_config_path.is_some()
        || paths.contract_path.is_some()
        || paths.runtime_dir.is_some()
}

fn write_success_envelope<T: serde::Serialize>(command: &str, data: T) -> Result<(), CoreError> {
    let envelope = success_envelope(command, data);
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

fn write_runtime_materialization_repair_summary(
    data: &RuntimeMaterializationRepairRunData,
) -> Result<(), CoreError> {
    let summary = match &data.repair {
        RuntimeRepairDirective::Noop { lines } => lines
            .first()
            .map(String::as_str)
            .unwrap_or("✅ Yazelix generated state is already up to date."),
        RuntimeRepairDirective::Regenerate { success_lines, .. } => success_lines
            .first()
            .map(String::as_str)
            .unwrap_or("✅ Generated runtime state repaired."),
    };

    writeln!(std::io::stdout(), "{summary}").map_err(|source| {
        CoreError::io(
            "write_stdout",
            "Could not write helper summary output",
            "Retry the command and report this as a Yazelix internal error if it persists.",
            "<stdout>",
            source.into(),
        )
    })?;
    Ok(())
}

fn write_runtime_materialization_repair_error_summary(error: &CoreError) -> Result<(), CoreError> {
    let summary = render_runtime_materialization_repair_error_summary(error);
    std::io::stderr()
        .write_all(summary.as_bytes())
        .map_err(|source| {
            CoreError::io(
                "write_stderr",
                "Could not write helper error summary output",
                "Retry the command and report this as a Yazelix internal error if it persists.",
                "<stderr>",
                source,
            )
        })
}

fn render_runtime_materialization_repair_error_summary(error: &CoreError) -> String {
    let mut lines = vec![
        "Yazelix generated runtime repair failed".to_string(),
        format!("Reason: {}", clean_human_error_line(&error.message())),
    ];

    if error.code() == "unsupported_config" {
        append_unsupported_config_summary(error, &mut lines);
    } else {
        lines.push(format!(
            "Recovery: {}",
            clean_human_error_line(&error.remediation())
        ));
    }

    lines.push(String::new());
    lines.join("\n")
}

fn append_unsupported_config_summary(error: &CoreError, lines: &mut Vec<String>) {
    let details = error.details();
    let diagnostics = details
        .get("blocking_diagnostics")
        .and_then(serde_json::Value::as_array)
        .or_else(|| {
            details
                .get("schema_diagnostics")
                .and_then(serde_json::Value::as_array)
        })
        .cloned()
        .unwrap_or_default();
    let blocking_count = details
        .get("blocking_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(diagnostics.len() as u64);

    lines.push(format!("Blocking config issues: {blocking_count}"));
    for diagnostic in diagnostics.iter().take(8) {
        let headline = diagnostic
            .get("headline")
            .and_then(serde_json::Value::as_str)
            .or_else(|| diagnostic.get("path").and_then(serde_json::Value::as_str))
            .unwrap_or("Unsupported config entry");
        lines.push(format!("- {}", clean_human_error_line(headline)));
    }
    if diagnostics.len() > 8 {
        lines.push(format!("- and {} more", diagnostics.len() - 8));
    }

    let mut next_steps = Vec::new();
    for diagnostic in &diagnostics {
        let Some(detail_lines) = diagnostic
            .get("detail_lines")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for detail in detail_lines
            .iter()
            .filter_map(serde_json::Value::as_str)
            .filter_map(|line| line.trim().strip_prefix("Next: "))
        {
            let cleaned = clean_human_error_line(detail);
            if !next_steps.contains(&cleaned) {
                next_steps.push(cleaned);
            }
        }
    }

    if next_steps.is_empty() {
        lines.push(format!(
            "Recovery: {}",
            clean_human_error_line(&error.remediation())
        ));
    } else {
        lines.push("Next:".to_string());
        for step in next_steps.iter().take(5) {
            lines.push(format!("- {step}"));
        }
    }
}

fn clean_human_error_line(value: &str) -> String {
    let mut cleaned = value.trim().to_string();
    while cleaned.ends_with('.') {
        cleaned.pop();
    }
    cleaned
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: private bridge command parsing maps a live helper name to its typed dispatcher path.
    #[test]
    fn parses_live_helper_command() {
        let mut parser = lexopt::Parser::from_args([TERMINAL_MATERIALIZATION_GENERATE_COMMAND]);
        let command = match parse_helper_command(&mut parser) {
            Ok(command) => command,
            Err(error) => panic!("unexpected parse error: {}", error.error.message()),
        };

        match command {
            HelperCommand::TerminalMaterializationGenerate => {}
            HelperCommand::HelixMaterializationGenerate
            | HelperCommand::RuntimeMaterializationRepair
            | HelperCommand::Unsupported(_) => panic!("expected terminal helper command"),
        }
    }

    // Defends: runtime materialization repair keeps its summary-aware error-output path outside the standard JSON dispatcher.
    #[test]
    fn classifies_runtime_repair_as_special_dispatch() {
        match classify_helper_command(RUNTIME_MATERIALIZATION_REPAIR_COMMAND.to_string()) {
            HelperCommand::RuntimeMaterializationRepair => {}
            HelperCommand::HelixMaterializationGenerate
            | HelperCommand::TerminalMaterializationGenerate
            | HelperCommand::Unsupported(_) => {
                panic!("expected runtime repair special dispatch")
            }
        }
    }

    // Defends: unsupported helper commands preserve the raw command name in the command error envelope.
    #[test]
    fn unsupported_helper_command_preserves_error_command() {
        let error = match dispatch_helper_command(
            classify_helper_command("missing.helper".to_string()),
            lexopt::Parser::from_args(Vec::<&str>::new()),
        ) {
            Ok(()) => panic!("unsupported helper unexpectedly succeeded"),
            Err(error) => error,
        };

        assert_eq!(error.command, "missing.helper");
        assert_eq!(error.error.class().as_str(), "usage");
        assert_eq!(
            error.error.message(),
            "Unsupported helper command: missing.helper"
        );
    }

    // Defends: terminal-materialization.generate still rejects mixed from-env and explicit path modes.
    #[test]
    fn terminal_materialization_rejects_from_env_with_explicit_paths() {
        let error = terminal_materialization_request_from_args(TerminalMaterializationArgs {
            paths: ConfigContractRuntimeArgs {
                runtime_dir: Some(PathBuf::from("/tmp/runtime")),
                ..Default::default()
            },
            from_env: true,
            ..Default::default()
        })
        .expect_err("from-env with explicit path should fail");

        assert_eq!(
            error.message(),
            "Use either --from-env or explicit terminal-materialization.generate paths, not both."
        );
    }
}
