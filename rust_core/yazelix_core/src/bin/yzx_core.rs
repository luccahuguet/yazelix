use lexopt::prelude::*;
use serde::de::DeserializeOwned;
use std::io::Write;
use std::path::PathBuf;
use yazelix_core::active_config_surface::resolve_active_config_paths;
use yazelix_core::control_plane::{
    config_dir_from_env, config_override_from_env, config_state_compute_request_from_env,
    config_state_record_request_from_env, ghostty_materialization_request_from_env,
    read_yazelix_version_from_runtime, runtime_dir_from_env, runtime_env_request_from_env,
    runtime_materialization_plan_request_from_env, state_dir_from_env,
    terminal_materialization_request_from_env,
};
use yazelix_core::terminal_materialization::YzxtermProfile;
use yazelix_core::terminal_variant::active_terminal_from_runtime_dir;
use yazelix_core::{
    ComputeConfigStateRequest, CoreError, DoctorConfigEvaluateRequest,
    DoctorRuntimeEvaluateRequest, ErrorClass, GhosttyMaterializationRequest,
    HelixDoctorEvaluateRequest, HelixMaterializationRequest, InstallOwnershipEvaluateRequest,
    LaunchMaterializationRequest, NormalizeConfigRequest, PopupSessionFactsData,
    RecordConfigStateRequest, RuntimeContractEvaluateRequest, RuntimeEnvComputeRequest,
    RuntimeMaterializationPlanRequest, RuntimeMaterializationRepairEvaluateRequest,
    RuntimeMaterializationRepairRunData, RuntimeOwnershipGraphRequest, RuntimeRepairDirective,
    SessionConfigSnapshotCreateRequest, StartupFactsData, StartupHandoffCaptureRequest,
    StartupLaunchPreflightRequest, TerminalMaterializationRequest, YaziMaterializationRequest,
    YaziRenderPlanRequest, YzxExternBridgeSyncRequest, ZellijMaterializationRequest,
    ZellijRenderPlanRequest, capture_startup_handoff_context, compute_config_state,
    compute_integration_facts_from_env, compute_popup_session_facts_from_env, compute_runtime_env,
    compute_runtime_ownership_graph, compute_startup_facts_from_env, compute_status_report,
    compute_yazi_render_plan, compute_zellij_render_plan, current_release_headline, error_envelope,
    evaluate_doctor_config_report, evaluate_doctor_runtime_report, evaluate_helix_doctor_report,
    evaluate_install_ownership_report, evaluate_runtime_contract,
    evaluate_startup_launch_preflight, generate_ghostty_materialization,
    generate_helix_materialization, generate_terminal_materialization,
    generate_yazi_materialization, generate_zellij_materialization,
    install_ownership_request_from_env, install_ownership_request_from_env_with_runtime_dir,
    launch_materialization_request_from_env, materialize_runtime_state,
    maybe_show_first_run_upgrade_summary, normalize_config, plan_runtime_materialization,
    prepare_launch_materialization, record_config_state, render_yzx_help,
    repair_runtime_materialization, success_envelope, sync_yzx_extern_bridge,
    write_session_config_snapshot_for_launch, yzx_command_metadata, yzx_command_metadata_data,
};

const CONFIG_NORMALIZE_COMMAND: &str = "config.normalize";
const CONFIG_SURFACE_RESOLVE_COMMAND: &str = "config-surface.resolve";
const CONFIG_STATE_COMPUTE_COMMAND: &str = "config-state.compute";
const CONFIG_STATE_RECORD_COMMAND: &str = "config-state.record";
const RUNTIME_CONTRACT_EVALUATE_COMMAND: &str = "runtime-contract.evaluate";
const STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND: &str = "startup-launch-preflight.evaluate";
const RUNTIME_ENV_COMPUTE_COMMAND: &str = "runtime-env.compute";
const INTEGRATION_FACTS_COMPUTE_COMMAND: &str = "integration-facts.compute";
const POPUP_SESSION_FACTS_COMPUTE_COMMAND: &str = "popup-session-facts.compute";
const STARTUP_FACTS_COMPUTE_COMMAND: &str = "startup-facts.compute";
const STARTUP_HANDOFF_CAPTURE_COMMAND: &str = "startup-handoff.capture";
const SESSION_CONFIG_SNAPSHOT_WRITE_COMMAND: &str = "session-config-snapshot.write";
const RUNTIME_MATERIALIZATION_PLAN_COMMAND: &str = "runtime-materialization.plan";
const RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND: &str = "runtime-materialization.materialize";
const RUNTIME_MATERIALIZATION_REPAIR_COMMAND: &str = "runtime-materialization.repair";
const STATUS_COMPUTE_COMMAND: &str = "status.compute";
const INSTALL_OWNERSHIP_EVALUATE_COMMAND: &str = "install-ownership.evaluate";
const DOCTOR_CONFIG_EVALUATE_COMMAND: &str = "doctor-config.evaluate";
const DOCTOR_HELIX_EVALUATE_COMMAND: &str = "doctor-helix.evaluate";
const DOCTOR_RUNTIME_EVALUATE_COMMAND: &str = "doctor-runtime.evaluate";
const ZELLIJ_RENDER_PLAN_COMPUTE_COMMAND: &str = "zellij-render-plan.compute";
const YAZI_RENDER_PLAN_COMPUTE_COMMAND: &str = "yazi-render-plan.compute";
const YAZI_MATERIALIZATION_GENERATE_COMMAND: &str = "yazi-materialization.generate";
const ZELLIJ_MATERIALIZATION_GENERATE_COMMAND: &str = "zellij-materialization.generate";
const HELIX_MATERIALIZATION_GENERATE_COMMAND: &str = "helix-materialization.generate";
const GHOSTTY_MATERIALIZATION_GENERATE_COMMAND: &str = "ghostty-materialization.generate";
const TERMINAL_MATERIALIZATION_GENERATE_COMMAND: &str = "terminal-materialization.generate";
const LAUNCH_MATERIALIZATION_PREPARE_COMMAND: &str = "launch-materialization.prepare";
const YZX_COMMAND_METADATA_LIST_COMMAND: &str = "yzx-command-metadata.list";
const YZX_COMMAND_METADATA_EXTERNS_COMMAND: &str = "yzx-command-metadata.externs";
const YZX_COMMAND_METADATA_SYNC_EXTERNS_COMMAND: &str = "yzx-command-metadata.sync-externs";
const YZX_COMMAND_METADATA_HELP_COMMAND: &str = "yzx-command-metadata.help";
const RUNTIME_OWNERSHIP_GRAPH_COMMAND: &str = "runtime-ownership.graph";
const UPGRADE_SUMMARY_HEADLINE_COMMAND: &str = "upgrade-summary.headline";
const UPGRADE_SUMMARY_FIRST_RUN_COMMAND: &str = "upgrade-summary.first-run";
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
struct ConfigStateComputeArgs {
    paths: ConfigContractRuntimeArgs,
    state_path: Option<PathBuf>,
    from_env: bool,
}

#[derive(Default)]
struct ConfigStateRecordArgs {
    config_file: Option<String>,
    managed_config_path: Option<PathBuf>,
    state_path: Option<PathBuf>,
    config_hash: Option<String>,
    runtime_hash: Option<String>,
    from_env: bool,
}

#[derive(Default)]
struct InstallOwnershipArgs {
    request_json: Option<String>,
    from_env: bool,
    runtime_dir: Option<PathBuf>,
}

#[derive(Default)]
struct RuntimeEnvArgs {
    request_json: Option<String>,
    config_json: Option<String>,
    from_env: bool,
}

#[derive(Default)]
struct StatusComputeArgs {
    paths: ConfigContractRuntimeArgs,
    state_path: Option<PathBuf>,
    yazi_config_dir: Option<PathBuf>,
    zellij_config_dir: Option<PathBuf>,
    zellij_layout_dir: Option<PathBuf>,
    layout_override: Option<String>,
    yazelix_version: Option<String>,
    yazelix_description: Option<String>,
}

#[derive(Default)]
struct YaziMaterializationArgs {
    paths: ConfigContractRuntimeArgs,
    yazi_config_dir: Option<PathBuf>,
    sync_static_assets: bool,
}

#[derive(Default)]
struct ZellijMaterializationArgs {
    paths: ConfigContractRuntimeArgs,
    zellij_config_dir: Option<PathBuf>,
    seed_plugin_permissions: bool,
}

#[derive(Default)]
struct HelixMaterializationArgs {
    runtime_dir: Option<PathBuf>,
    config_dir: Option<PathBuf>,
    state_dir: Option<PathBuf>,
    show_splash: bool,
}

#[derive(Default)]
struct GhosttyMaterializationArgs {
    runtime_dir: Option<PathBuf>,
    config_dir: Option<PathBuf>,
    state_dir: Option<PathBuf>,
    transparency: Option<String>,
    cursor_config_path: Option<PathBuf>,
    from_env: bool,
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

type StandardCommandHandlerFn = fn(lexopt::Parser) -> Result<(), CoreError>;

struct StandardCommandHandler {
    name: &'static str,
    run: StandardCommandHandlerFn,
}

enum HelperCommand {
    Standard(&'static StandardCommandHandler),
    RuntimeMaterializationRepair,
    Unsupported(String),
}

macro_rules! standard_command_handlers {
    ($(($name:expr, $run:path)),+ $(,)?) => {
        const STANDARD_COMMAND_HANDLERS: &[StandardCommandHandler] = &[
            $(StandardCommandHandler { name: $name, run: $run },)+
        ];
    };
}

standard_command_handlers!(
    (CONFIG_NORMALIZE_COMMAND, run_config_normalize),
    (CONFIG_SURFACE_RESOLVE_COMMAND, run_config_surface_resolve),
    (CONFIG_STATE_COMPUTE_COMMAND, run_config_state_compute),
    (CONFIG_STATE_RECORD_COMMAND, run_config_state_record),
    (
        RUNTIME_CONTRACT_EVALUATE_COMMAND,
        run_runtime_contract_evaluate
    ),
    (
        STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND,
        run_startup_launch_preflight_evaluate
    ),
    (RUNTIME_ENV_COMPUTE_COMMAND, run_runtime_env_compute),
    (
        INTEGRATION_FACTS_COMPUTE_COMMAND,
        run_integration_facts_compute
    ),
    (
        POPUP_SESSION_FACTS_COMPUTE_COMMAND,
        run_popup_session_facts_compute
    ),
    (STARTUP_FACTS_COMPUTE_COMMAND, run_startup_facts_compute),
    (STARTUP_HANDOFF_CAPTURE_COMMAND, run_startup_handoff_capture),
    (
        SESSION_CONFIG_SNAPSHOT_WRITE_COMMAND,
        run_session_config_snapshot_write
    ),
    (
        RUNTIME_MATERIALIZATION_PLAN_COMMAND,
        run_runtime_materialization_plan
    ),
    (
        RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND,
        run_runtime_materialization_materialize
    ),
    (STATUS_COMPUTE_COMMAND, run_status_compute),
    (
        INSTALL_OWNERSHIP_EVALUATE_COMMAND,
        run_install_ownership_evaluate
    ),
    (DOCTOR_CONFIG_EVALUATE_COMMAND, run_doctor_config_evaluate),
    (DOCTOR_HELIX_EVALUATE_COMMAND, run_doctor_helix_evaluate),
    (DOCTOR_RUNTIME_EVALUATE_COMMAND, run_doctor_runtime_evaluate),
    (
        ZELLIJ_RENDER_PLAN_COMPUTE_COMMAND,
        run_zellij_render_plan_compute
    ),
    (
        YAZI_RENDER_PLAN_COMPUTE_COMMAND,
        run_yazi_render_plan_compute
    ),
    (
        YAZI_MATERIALIZATION_GENERATE_COMMAND,
        run_yazi_materialization_generate
    ),
    (
        ZELLIJ_MATERIALIZATION_GENERATE_COMMAND,
        run_zellij_materialization_generate
    ),
    (
        HELIX_MATERIALIZATION_GENERATE_COMMAND,
        run_helix_materialization_generate
    ),
    (
        GHOSTTY_MATERIALIZATION_GENERATE_COMMAND,
        run_ghostty_materialization_generate
    ),
    (
        TERMINAL_MATERIALIZATION_GENERATE_COMMAND,
        run_terminal_materialization_generate
    ),
    (
        LAUNCH_MATERIALIZATION_PREPARE_COMMAND,
        run_launch_materialization_prepare
    ),
    (
        YZX_COMMAND_METADATA_LIST_COMMAND,
        run_yzx_command_metadata_list
    ),
    (
        YZX_COMMAND_METADATA_EXTERNS_COMMAND,
        run_yzx_command_metadata_externs
    ),
    (
        YZX_COMMAND_METADATA_SYNC_EXTERNS_COMMAND,
        run_yzx_command_metadata_sync_externs
    ),
    (
        YZX_COMMAND_METADATA_HELP_COMMAND,
        run_yzx_command_metadata_help
    ),
    (RUNTIME_OWNERSHIP_GRAPH_COMMAND, run_runtime_ownership_graph),
    (
        UPGRADE_SUMMARY_HEADLINE_COMMAND,
        run_upgrade_summary_headline
    ),
    (
        UPGRADE_SUMMARY_FIRST_RUN_COMMAND,
        run_upgrade_summary_first_run
    ),
);

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
    if command_name == RUNTIME_MATERIALIZATION_REPAIR_COMMAND {
        return HelperCommand::RuntimeMaterializationRepair;
    }

    find_standard_command_handler(&command_name)
        .map(HelperCommand::Standard)
        .unwrap_or(HelperCommand::Unsupported(command_name))
}

fn find_standard_command_handler(command_name: &str) -> Option<&'static StandardCommandHandler> {
    STANDARD_COMMAND_HANDLERS
        .iter()
        .find(|handler| handler.name == command_name)
}

fn dispatch_helper_command(
    command: HelperCommand,
    parser: lexopt::Parser,
) -> Result<(), Box<CommandError>> {
    match command {
        HelperCommand::Standard(handler) => {
            (handler.run)(parser).map_err(|error| CommandError::new(handler.name, error))
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

fn run_upgrade_summary_headline(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let runtime_dir = runtime_dir_from_env()?;
    let version = read_yazelix_version_from_runtime(&runtime_dir)?;
    let headline = current_release_headline(&runtime_dir, &version)?;
    write_success_envelope(
        UPGRADE_SUMMARY_HEADLINE_COMMAND,
        serde_json::json!({
            "version": version,
            "headline": headline,
        }),
    )
}

fn run_upgrade_summary_first_run(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let runtime_dir = runtime_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let version = read_yazelix_version_from_runtime(&runtime_dir)?;
    let data = maybe_show_first_run_upgrade_summary(&runtime_dir, &state_dir, &version)?;
    write_success_envelope(UPGRADE_SUMMARY_FIRST_RUN_COMMAND, data)
}

fn run_yzx_command_metadata_list(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    write_success_envelope(
        YZX_COMMAND_METADATA_LIST_COMMAND,
        yzx_command_metadata_data(),
    )
}

fn run_yzx_command_metadata_externs(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    write_success_envelope(
        YZX_COMMAND_METADATA_EXTERNS_COMMAND,
        yzx_command_metadata_data(),
    )
}

fn run_yzx_command_metadata_sync_externs(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut runtime_dir: Option<PathBuf> = None;
    let mut state_dir: Option<PathBuf> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("state-dir") => state_dir = Some(parser_path_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = YzxExternBridgeSyncRequest {
        runtime_dir: runtime_dir.ok_or_else(|| CoreError::usage("Missing --runtime-dir path"))?,
        state_dir: state_dir.ok_or_else(|| CoreError::usage("Missing --state-dir path"))?,
    };
    let data = sync_yzx_extern_bridge(&request)?;
    write_success_envelope(YZX_COMMAND_METADATA_SYNC_EXTERNS_COMMAND, data)
}

fn run_yzx_command_metadata_help(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    print!("{}", render_yzx_help(&yzx_command_metadata()));
    Ok(())
}

fn run_runtime_ownership_graph(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut runtime_dir: Option<PathBuf> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = RuntimeOwnershipGraphRequest {
        runtime_dir: match runtime_dir {
            Some(path) => path,
            None => runtime_dir_from_env()?,
        },
    };
    let data = compute_runtime_ownership_graph(&request)?;
    write_success_envelope(RUNTIME_OWNERSHIP_GRAPH_COMMAND, data)
}

fn run_config_normalize(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut config_path: Option<PathBuf> = None;
    let mut default_config_path: Option<PathBuf> = None;
    let mut contract_path: Option<PathBuf> = None;
    let mut include_missing = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("config") => config_path = Some(parser_path_value(&mut parser)?),
            Long("default-config") => default_config_path = Some(parser_path_value(&mut parser)?),
            Long("contract") => contract_path = Some(parser_path_value(&mut parser)?),
            Long("include-missing") => include_missing = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request = NormalizeConfigRequest {
        config_path: config_path.ok_or_else(|| CoreError::usage("Missing --config path"))?,
        default_config_path: default_config_path
            .ok_or_else(|| CoreError::usage("Missing --default-config path"))?,
        contract_path: contract_path.ok_or_else(|| CoreError::usage("Missing --contract path"))?,
        include_missing,
    };
    let data = normalize_config(&request)?;
    write_success_envelope(CONFIG_NORMALIZE_COMMAND, data)
}

fn run_config_surface_resolve(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut runtime_dir: Option<PathBuf> = None;
    let mut config_dir: Option<PathBuf> = None;
    let mut config_override: Option<String> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("runtime-dir") => runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("config-dir") => config_dir = Some(parser_path_value(&mut parser)?),
            Long("config-override") => config_override = Some(parser_string_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let runtime_dir = match runtime_dir {
        Some(path) => path,
        None => runtime_dir_from_env()?,
    };
    let config_dir = match config_dir {
        Some(path) => path,
        None => config_dir_from_env()?,
    };
    let config_override = config_override.or_else(config_override_from_env);
    let data = resolve_active_config_paths(&runtime_dir, &config_dir, config_override.as_deref())?;
    write_success_envelope(CONFIG_SURFACE_RESOLVE_COMMAND, data)
}

fn run_config_state_compute(parser: lexopt::Parser) -> Result<(), CoreError> {
    let request = config_state_compute_request_from_args(take_config_state_compute_args(parser)?)?;
    let data = compute_config_state(&request)?;
    write_success_envelope(CONFIG_STATE_COMPUTE_COMMAND, data)
}

fn take_config_state_compute_args(
    mut parser: lexopt::Parser,
) -> Result<ConfigStateComputeArgs, CoreError> {
    let mut args = ConfigStateComputeArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        let option = parsed_long_option(arg)?;
        if parse_config_contract_runtime_option(&option.name, &mut parser, &mut args.paths)? {
            continue;
        }
        match option.name.as_str() {
            "state-path" => args.state_path = Some(parser_path_value(&mut parser)?),
            "from-env" => args.from_env = true,
            _ => return Err(option.unexpected_error()),
        }
    }
    Ok(args)
}

fn config_state_compute_request_from_args(
    args: ConfigStateComputeArgs,
) -> Result<ComputeConfigStateRequest, CoreError> {
    let explicit_args_present =
        config_contract_runtime_args_present(&args.paths) || args.state_path.is_some();

    if args.from_env {
        if explicit_args_present {
            return Err(CoreError::usage(
                "Use either --from-env or explicit config-state.compute paths, not both.",
            ));
        }
        return config_state_compute_request_from_env(config_override_from_env().as_deref());
    }

    Ok(ComputeConfigStateRequest {
        config_path: required_path(args.paths.config_path, "Missing --config path")?,
        default_config_path: required_path(
            args.paths.default_config_path,
            "Missing --default-config path",
        )?,
        contract_path: required_path(args.paths.contract_path, "Missing --contract path")?,
        runtime_dir: required_path(args.paths.runtime_dir, "Missing --runtime-dir path")?,
        state_path: required_path(args.state_path, "Missing --state-path path")?,
    })
}

fn run_config_state_record(parser: lexopt::Parser) -> Result<(), CoreError> {
    let request = config_state_record_request_from_args(take_config_state_record_args(parser)?)?;
    let data = record_config_state(&request)?;
    write_success_envelope(CONFIG_STATE_RECORD_COMMAND, data)
}

fn take_config_state_record_args(
    mut parser: lexopt::Parser,
) -> Result<ConfigStateRecordArgs, CoreError> {
    let mut args = ConfigStateRecordArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("config-file") => args.config_file = Some(parser_string_value(&mut parser)?),
            Long("managed-config") => {
                args.managed_config_path = Some(parser_path_value(&mut parser)?)
            }
            Long("state-path") => args.state_path = Some(parser_path_value(&mut parser)?),
            Long("config-hash") => args.config_hash = Some(parser_string_value(&mut parser)?),
            Long("runtime-hash") => args.runtime_hash = Some(parser_string_value(&mut parser)?),
            Long("from-env") => args.from_env = true,
            _ => return Err(unexpected_argument(arg)),
        }
    }
    Ok(args)
}

fn config_state_record_request_from_args(
    args: ConfigStateRecordArgs,
) -> Result<RecordConfigStateRequest, CoreError> {
    let explicit_args_present = args.managed_config_path.is_some() || args.state_path.is_some();

    let config_file = required_string(args.config_file, "Missing --config-file path")?;
    let config_hash = required_string(args.config_hash, "Missing --config-hash value")?;
    let runtime_hash = required_string(args.runtime_hash, "Missing --runtime-hash value")?;

    if args.from_env {
        if explicit_args_present {
            return Err(CoreError::usage(
                "Use either --from-env or explicit config-state.record paths, not both.",
            ));
        }
        return config_state_record_request_from_env(config_file, config_hash, runtime_hash);
    }

    Ok(RecordConfigStateRequest {
        config_file,
        managed_config_path: required_path(
            args.managed_config_path,
            "Missing --managed-config path",
        )?,
        state_path: required_path(args.state_path, "Missing --state-path path")?,
        config_hash,
        runtime_hash,
    })
}

fn run_runtime_contract_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut request_json: Option<String> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(&mut parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    let request_json =
        request_json.ok_or_else(|| CoreError::usage("Missing --request-json payload"))?;
    let request: RuntimeContractEvaluateRequest =
        serde_json::from_str(&request_json).map_err(|error| {
            CoreError::classified(
                ErrorClass::Usage,
                "invalid_request_json",
                format!("Invalid runtime-contract request JSON: {error}"),
                "Pass one valid JSON payload via --request-json.",
                serde_json::json!({}),
            )
        })?;
    let data = evaluate_runtime_contract(&request)?;
    write_success_envelope(RUNTIME_CONTRACT_EVALUATE_COMMAND, data)
}

fn run_startup_launch_preflight_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: StartupLaunchPreflightRequest =
        deserialize_json_request(&request_json, "startup-launch-preflight")?;
    let data = evaluate_startup_launch_preflight(&request)?;
    write_success_envelope(STARTUP_LAUNCH_PREFLIGHT_EVALUATE_COMMAND, data)
}

fn run_doctor_helix_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: HelixDoctorEvaluateRequest =
        deserialize_json_request(&request_json, "doctor-helix")?;
    let data = evaluate_helix_doctor_report(&request);
    write_success_envelope(DOCTOR_HELIX_EVALUATE_COMMAND, data)
}

fn run_doctor_config_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: DoctorConfigEvaluateRequest =
        deserialize_json_request(&request_json, "doctor-config")?;
    let data = evaluate_doctor_config_report(&request);
    write_success_envelope(DOCTOR_CONFIG_EVALUATE_COMMAND, data)
}

fn run_doctor_runtime_evaluate(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: DoctorRuntimeEvaluateRequest =
        deserialize_json_request(&request_json, "doctor-runtime")?;
    let data = evaluate_doctor_runtime_report(&request);
    write_success_envelope(DOCTOR_RUNTIME_EVALUATE_COMMAND, data)
}

fn run_install_ownership_evaluate(parser: lexopt::Parser) -> Result<(), CoreError> {
    let request = install_ownership_request_from_args(take_install_ownership_args(parser)?)?;
    let data = evaluate_install_ownership_report(&request);
    write_success_envelope(INSTALL_OWNERSHIP_EVALUATE_COMMAND, data)
}

fn take_install_ownership_args(
    mut parser: lexopt::Parser,
) -> Result<InstallOwnershipArgs, CoreError> {
    let mut args = InstallOwnershipArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => args.request_json = Some(parser_string_value(&mut parser)?),
            Long("from-env") => args.from_env = true,
            Long("runtime-dir") => args.runtime_dir = Some(parser_path_value(&mut parser)?),
            _ => return Err(unexpected_argument(arg)),
        }
    }
    Ok(args)
}

fn install_ownership_request_from_args(
    args: InstallOwnershipArgs,
) -> Result<InstallOwnershipEvaluateRequest, CoreError> {
    match (args.from_env, args.request_json) {
        (true, Some(_)) => Err(CoreError::usage(
            "Use either --from-env or --request-json for install-ownership.evaluate, not both.",
        )),
        (true, None) => Ok(match args.runtime_dir {
            Some(runtime_dir) => install_ownership_request_from_env_with_runtime_dir(runtime_dir)?,
            None => install_ownership_request_from_env()?,
        }),
        (false, Some(request_json)) => {
            if args.runtime_dir.is_some() {
                return Err(CoreError::usage(
                    "Use --runtime-dir only with --from-env for install-ownership.evaluate.",
                ));
            }
            deserialize_json_request(&request_json, "install-ownership")
        }
        (false, None) => Err(CoreError::usage(
            "Missing --request-json payload or --from-env for install-ownership.evaluate.",
        )),
    }
}

fn run_zellij_render_plan_compute(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: ZellijRenderPlanRequest =
        deserialize_json_request(&request_json, "zellij-render-plan")?;
    let data = compute_zellij_render_plan(&request)?;
    write_success_envelope(ZELLIJ_RENDER_PLAN_COMPUTE_COMMAND, data)
}

fn run_yazi_render_plan_compute(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: YaziRenderPlanRequest =
        deserialize_json_request(&request_json, "yazi-render-plan")?;
    let data = compute_yazi_render_plan(&request)?;
    write_success_envelope(YAZI_RENDER_PLAN_COMPUTE_COMMAND, data)
}

fn run_yazi_materialization_generate(parser: lexopt::Parser) -> Result<(), CoreError> {
    let request = yazi_materialization_request_from_args(take_yazi_materialization_args(parser)?)?;
    let data = generate_yazi_materialization(&request)?;
    write_success_envelope(YAZI_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn take_yazi_materialization_args(
    mut parser: lexopt::Parser,
) -> Result<YaziMaterializationArgs, CoreError> {
    let mut args = YaziMaterializationArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        let option = parsed_long_option(arg)?;
        if parse_config_contract_runtime_option(&option.name, &mut parser, &mut args.paths)? {
            continue;
        }
        match option.name.as_str() {
            "yazi-config-dir" => args.yazi_config_dir = Some(parser_path_value(&mut parser)?),
            "sync-static-assets" => args.sync_static_assets = true,
            _ => return Err(option.unexpected_error()),
        }
    }
    Ok(args)
}

fn yazi_materialization_request_from_args(
    args: YaziMaterializationArgs,
) -> Result<YaziMaterializationRequest, CoreError> {
    Ok(YaziMaterializationRequest {
        config_path: required_path(args.paths.config_path, "Missing --config path")?,
        default_config_path: required_path(
            args.paths.default_config_path,
            "Missing --default-config path",
        )?,
        contract_path: required_path(args.paths.contract_path, "Missing --contract path")?,
        runtime_dir: required_path(args.paths.runtime_dir, "Missing --runtime-dir path")?,
        yazi_config_dir: required_path(args.yazi_config_dir, "Missing --yazi-config-dir path")?,
        sync_static_assets: args.sync_static_assets,
    })
}

fn run_zellij_materialization_generate(parser: lexopt::Parser) -> Result<(), CoreError> {
    let request =
        zellij_materialization_request_from_args(take_zellij_materialization_args(parser)?)?;
    let data = generate_zellij_materialization(&request)?;
    write_success_envelope(ZELLIJ_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn take_zellij_materialization_args(
    mut parser: lexopt::Parser,
) -> Result<ZellijMaterializationArgs, CoreError> {
    let mut args = ZellijMaterializationArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        let option = parsed_long_option(arg)?;
        if parse_config_contract_runtime_option(&option.name, &mut parser, &mut args.paths)? {
            continue;
        }
        match option.name.as_str() {
            "zellij-config-dir" => args.zellij_config_dir = Some(parser_path_value(&mut parser)?),
            "seed-plugin-permissions" => args.seed_plugin_permissions = true,
            _ => return Err(option.unexpected_error()),
        }
    }

    Ok(args)
}

fn zellij_materialization_request_from_args(
    args: ZellijMaterializationArgs,
) -> Result<ZellijMaterializationRequest, CoreError> {
    Ok(ZellijMaterializationRequest {
        config_path: required_path(args.paths.config_path, "Missing --config path")?,
        default_config_path: required_path(
            args.paths.default_config_path,
            "Missing --default-config path",
        )?,
        contract_path: required_path(args.paths.contract_path, "Missing --contract path")?,
        runtime_dir: required_path(args.paths.runtime_dir, "Missing --runtime-dir path")?,
        zellij_config_dir: required_path(
            args.zellij_config_dir,
            "Missing --zellij-config-dir path",
        )?,
        seed_plugin_permissions: args.seed_plugin_permissions,
    })
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

fn run_ghostty_materialization_generate(parser: lexopt::Parser) -> Result<(), CoreError> {
    let request =
        ghostty_materialization_request_from_args(take_ghostty_materialization_args(parser)?)?;
    let data = generate_ghostty_materialization(&request)?;
    write_success_envelope(GHOSTTY_MATERIALIZATION_GENERATE_COMMAND, data)
}

fn take_ghostty_materialization_args(
    mut parser: lexopt::Parser,
) -> Result<GhosttyMaterializationArgs, CoreError> {
    let mut args = GhosttyMaterializationArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("from-env") => args.from_env = true,
            Long("runtime-dir") => args.runtime_dir = Some(parser_path_value(&mut parser)?),
            Long("config-dir") => args.config_dir = Some(parser_path_value(&mut parser)?),
            Long("state-dir") => args.state_dir = Some(parser_path_value(&mut parser)?),
            Long("transparency") => args.transparency = Some(parser_string_value(&mut parser)?),
            Long("cursor-config") => {
                args.cursor_config_path = Some(parser_path_value(&mut parser)?)
            }
            _ => return Err(unexpected_argument(arg)),
        }
    }
    Ok(args)
}

fn ghostty_materialization_request_from_args(
    args: GhosttyMaterializationArgs,
) -> Result<GhosttyMaterializationRequest, CoreError> {
    let explicit_args_present = args.runtime_dir.is_some()
        || args.config_dir.is_some()
        || args.state_dir.is_some()
        || args.transparency.is_some()
        || args.cursor_config_path.is_some();

    if args.from_env {
        if explicit_args_present {
            return Err(CoreError::usage(
                "Use either --from-env or explicit ghostty-materialization.generate flags, not both.",
            ));
        }
        return ghostty_materialization_request_from_env(config_override_from_env().as_deref());
    }

    Ok(GhosttyMaterializationRequest {
        runtime_dir: required_path(args.runtime_dir, "Missing --runtime-dir path")?,
        config_dir: required_path(args.config_dir, "Missing --config-dir path")?,
        state_dir: required_path(args.state_dir, "Missing --state-dir path")?,
        transparency: required_string(args.transparency, "Missing --transparency")?,
        cursor_config_path: required_path(args.cursor_config_path, "Missing --cursor-config path")?,
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
        yzxterm_emoji_font: None,
        yzxterm_profile: YzxtermProfile::Full,
    })
}

fn run_launch_materialization_prepare(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let mut desktop_fast_path = false;
    let mut from_env = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("from-env") => from_env = true,
            Long("desktop-fast-path") => desktop_fast_path = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    if !from_env {
        return Err(CoreError::usage(
            "launch-materialization.prepare currently requires --from-env.",
        ));
    }

    let request: LaunchMaterializationRequest = launch_materialization_request_from_env(
        desktop_fast_path,
        false,
        config_override_from_env().as_deref(),
    )?;
    let data = prepare_launch_materialization(&request)?;
    write_success_envelope(LAUNCH_MATERIALIZATION_PREPARE_COMMAND, data)
}

fn run_runtime_env_compute(parser: lexopt::Parser) -> Result<(), CoreError> {
    let request = runtime_env_request_from_args(take_runtime_env_args(parser)?)?;
    let data = compute_runtime_env(&request)?;
    write_success_envelope(RUNTIME_ENV_COMPUTE_COMMAND, data)
}

fn take_runtime_env_args(mut parser: lexopt::Parser) -> Result<RuntimeEnvArgs, CoreError> {
    let mut args = RuntimeEnvArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => args.request_json = Some(parser_string_value(&mut parser)?),
            Long("config-json") => args.config_json = Some(parser_string_value(&mut parser)?),
            Long("from-env") => args.from_env = true,
            _ => return Err(unexpected_argument(arg)),
        }
    }
    Ok(args)
}

fn runtime_env_request_from_args(
    args: RuntimeEnvArgs,
) -> Result<RuntimeEnvComputeRequest, CoreError> {
    if args.from_env {
        if args.request_json.is_some() {
            return Err(CoreError::usage(
                "Use either --from-env or --request-json for runtime-env.compute, not both.",
            ));
        }
        return runtime_env_request_from_env(
            args.config_json.as_deref(),
            config_override_from_env().as_deref(),
        );
    }

    if args.config_json.is_some() {
        return Err(CoreError::usage(
            "runtime-env.compute only accepts --config-json together with --from-env.",
        ));
    }
    let request_json = required_string(args.request_json, "Missing --request-json payload")?;
    deserialize_json_request(&request_json, "runtime-env")
}

fn run_integration_facts_compute(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let data = compute_integration_facts_from_env()?;
    write_success_envelope(INTEGRATION_FACTS_COMPUTE_COMMAND, data)
}

fn run_popup_session_facts_compute(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let data: PopupSessionFactsData = compute_popup_session_facts_from_env()?;
    write_success_envelope(POPUP_SESSION_FACTS_COMPUTE_COMMAND, data)
}

fn run_startup_facts_compute(parser: lexopt::Parser) -> Result<(), CoreError> {
    ensure_no_args(parser)?;
    let data: StartupFactsData = compute_startup_facts_from_env()?;
    write_success_envelope(STARTUP_FACTS_COMPUTE_COMMAND, data)
}

fn run_startup_handoff_capture(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: StartupHandoffCaptureRequest =
        deserialize_json_request(&request_json, "startup-handoff.capture")?;
    let data = capture_startup_handoff_context(&request)?;
    write_success_envelope(STARTUP_HANDOFF_CAPTURE_COMMAND, data)
}

fn run_session_config_snapshot_write(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request_json = take_request_json(&mut parser)?;
    let request: SessionConfigSnapshotCreateRequest =
        deserialize_json_request(&request_json, "session-config-snapshot.write")?;
    let version = read_yazelix_version_from_runtime(&request.runtime_dir)?;
    let data = write_session_config_snapshot_for_launch(&request, &version)?;
    write_success_envelope(SESSION_CONFIG_SNAPSHOT_WRITE_COMMAND, data)
}

fn run_runtime_materialization_plan(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request =
        take_runtime_materialization_plan_request(&mut parser, "runtime-materialization.plan")?;
    let data = plan_runtime_materialization(&request)?;
    write_success_envelope(RUNTIME_MATERIALIZATION_PLAN_COMMAND, data)
}

fn run_runtime_materialization_materialize(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    let request = take_runtime_materialization_plan_request(
        &mut parser,
        "runtime-materialization.materialize",
    )?;
    let data = materialize_runtime_state(&request)?;
    write_success_envelope(RUNTIME_MATERIALIZATION_MATERIALIZE_COMMAND, data)
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

fn run_status_compute(parser: lexopt::Parser) -> Result<(), CoreError> {
    let args = take_status_compute_args(parser)?;
    let version =
        required_nonempty_string(&args.yazelix_version, "Missing --yazelix-version")?.to_string();
    let description =
        required_nonempty_string(&args.yazelix_description, "Missing --yazelix-description")?
            .to_string();
    let request = status_compute_request_from_args(args)?;
    let data = compute_status_report(&request, &version, &description)?;
    write_success_envelope(STATUS_COMPUTE_COMMAND, data)
}

fn take_status_compute_args(mut parser: lexopt::Parser) -> Result<StatusComputeArgs, CoreError> {
    let mut args = StatusComputeArgs::default();
    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        let option = parsed_long_option(arg)?;
        if parse_config_contract_runtime_option(&option.name, &mut parser, &mut args.paths)? {
            continue;
        }
        match option.name.as_str() {
            "state-path" => args.state_path = Some(parser_path_value(&mut parser)?),
            "yazi-config-dir" => args.yazi_config_dir = Some(parser_path_value(&mut parser)?),
            "zellij-config-dir" => args.zellij_config_dir = Some(parser_path_value(&mut parser)?),
            "zellij-layout-dir" => args.zellij_layout_dir = Some(parser_path_value(&mut parser)?),
            "layout-override" => args.layout_override = Some(parser_string_value(&mut parser)?),
            "yazelix-version" => args.yazelix_version = Some(parser_string_value(&mut parser)?),
            "yazelix-description" => {
                args.yazelix_description = Some(parser_string_value(&mut parser)?)
            }
            _ => return Err(option.unexpected_error()),
        }
    }
    Ok(args)
}

fn status_compute_request_from_args(
    args: StatusComputeArgs,
) -> Result<RuntimeMaterializationPlanRequest, CoreError> {
    into_runtime_plan_request(
        args.paths,
        args.state_path,
        args.yazi_config_dir,
        args.zellij_config_dir,
        args.zellij_layout_dir,
        args.layout_override,
    )
}

fn take_request_json(parser: &mut lexopt::Parser) -> Result<String, CoreError> {
    let mut request_json: Option<String> = None;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(parser)?),
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    request_json.ok_or_else(|| CoreError::usage("Missing --request-json payload"))
}

fn take_runtime_materialization_plan_request(
    parser: &mut lexopt::Parser,
    kind: &str,
) -> Result<RuntimeMaterializationPlanRequest, CoreError> {
    let mut request_json: Option<String> = None;
    let mut from_env = false;

    while let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        match arg {
            Long("request-json") => request_json = Some(parser_string_value(parser)?),
            Long("from-env") => from_env = true,
            _ => return Err(CoreError::usage(format!("Unexpected argument: {arg:?}"))),
        }
    }

    match (from_env, request_json) {
        (true, None) => {
            runtime_materialization_plan_request_from_env(config_override_from_env().as_deref())
        }
        (false, Some(raw)) => deserialize_json_request(&raw, kind),
        (true, Some(_)) => Err(CoreError::usage(
            "Use either --from-env or --request-json for runtime materialization, not both.",
        )),
        (false, None) => Err(CoreError::usage(
            "Missing --request-json payload or --from-env for runtime materialization.",
        )),
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

fn ensure_no_args(mut parser: lexopt::Parser) -> Result<(), CoreError> {
    if let Some(arg) = parser
        .next()
        .map_err(|error| CoreError::usage(error.to_string()))?
    {
        return Err(CoreError::usage(format!("Unexpected argument: {arg:?}")));
    }
    Ok(())
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

fn required_string(value: Option<String>, message: &'static str) -> Result<String, CoreError> {
    value.ok_or_else(|| CoreError::usage(message))
}

fn required_nonempty_string<'a>(
    value: &'a Option<String>,
    message: &'static str,
) -> Result<&'a str, CoreError> {
    value
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| CoreError::usage(message))
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

fn into_runtime_plan_request(
    paths: ConfigContractRuntimeArgs,
    state_path: Option<PathBuf>,
    yazi_config_dir: Option<PathBuf>,
    zellij_config_dir: Option<PathBuf>,
    zellij_layout_dir: Option<PathBuf>,
    layout_override: Option<String>,
) -> Result<RuntimeMaterializationPlanRequest, CoreError> {
    Ok(RuntimeMaterializationPlanRequest {
        config_path: required_path(paths.config_path, "Missing --config path")?,
        default_config_path: required_path(
            paths.default_config_path,
            "Missing --default-config path",
        )?,
        contract_path: required_path(paths.contract_path, "Missing --contract path")?,
        runtime_dir: required_path(paths.runtime_dir, "Missing --runtime-dir path")?,
        state_path: required_path(state_path, "Missing --state-path path")?,
        yazi_config_dir: required_path(yazi_config_dir, "Missing --yazi-config-dir path")?,
        zellij_config_dir: required_path(zellij_config_dir, "Missing --zellij-config-dir path")?,
        zellij_layout_dir: required_path(zellij_layout_dir, "Missing --zellij-layout-dir path")?,
        zellij_permissions_cache_path: None,
        layout_override,
    })
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

    // Defends: private bridge command parsing maps a known helper name to the typed standard dispatcher path.
    #[test]
    fn parses_standard_helper_command() {
        let mut parser = lexopt::Parser::from_args([STATUS_COMPUTE_COMMAND]);
        let command = match parse_helper_command(&mut parser) {
            Ok(command) => command,
            Err(error) => panic!("unexpected parse error: {}", error.error.message()),
        };

        match command {
            HelperCommand::Standard(handler) => assert_eq!(handler.name, STATUS_COMPUTE_COMMAND),
            HelperCommand::RuntimeMaterializationRepair | HelperCommand::Unsupported(_) => {
                panic!("expected standard helper command")
            }
        }
    }

    // Defends: runtime materialization repair keeps its summary-aware error-output path outside the standard JSON dispatcher.
    #[test]
    fn classifies_runtime_repair_as_special_dispatch() {
        match classify_helper_command(RUNTIME_MATERIALIZATION_REPAIR_COMMAND.to_string()) {
            HelperCommand::RuntimeMaterializationRepair => {}
            HelperCommand::Standard(_) | HelperCommand::Unsupported(_) => {
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

    // Defends: status.compute reports the same required identity error after parser/request-builder split.
    #[test]
    fn status_compute_requires_version_identity() {
        let error = run_status_compute(lexopt::Parser::from_args([
            "--config",
            "/tmp/settings.jsonc",
            "--default-config",
            "/tmp/default.jsonc",
            "--contract",
            "/tmp/contract.toml",
            "--runtime-dir",
            "/tmp/runtime",
            "--state-path",
            "/tmp/state.json",
            "--yazi-config-dir",
            "/tmp/yazi",
            "--zellij-config-dir",
            "/tmp/zellij",
            "--zellij-layout-dir",
            "/tmp/layouts",
            "--yazelix-description",
            "Yazelix",
        ]))
        .expect_err("missing version should fail before status computation");

        assert_eq!(error.message(), "Missing --yazelix-version");
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

    // Defends: runtime-env.compute keeps --config-json scoped to the from-env request path.
    #[test]
    fn runtime_env_rejects_config_json_without_from_env() {
        let error = runtime_env_request_from_args(RuntimeEnvArgs {
            config_json: Some("{}".to_string()),
            ..Default::default()
        })
        .expect_err("config-json without from-env should fail");

        assert_eq!(
            error.message(),
            "runtime-env.compute only accepts --config-json together with --from-env."
        );
    }
}
