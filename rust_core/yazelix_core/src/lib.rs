pub mod active_config_surface;
pub mod bridge;
pub mod cli_render;
pub mod command_metadata;
pub mod config_commands;
pub mod config_normalize;
pub mod config_state;
pub mod control_plane;
pub mod doctor_commands;
pub mod doctor_config_report;
pub mod doctor_helix_report;
pub mod doctor_runtime_report;
pub mod edit_commands;
pub mod front_door_commands;
pub mod front_door_render;
pub mod ghostty_cursor_registry;
pub mod ghostty_materialization;
pub mod helix_materialization;
pub mod home_manager_commands;
pub mod import_commands;
pub mod initializer_commands;
pub mod install_ownership_env;
pub mod install_ownership_report;
pub mod internal_nu_runner;
pub mod keys_commands;
pub mod launch_commands;
pub mod launch_materialization;
pub mod layout_family_contract;
pub mod onboard_commands;
pub mod profile_commands;
pub mod public_command_surface;
pub mod runtime_contract;
pub mod runtime_env;
pub mod runtime_materialization;
pub mod session_config_snapshot;
pub mod session_facts;
pub mod startup_facts;
pub mod startup_handoff;
pub mod status_report;
pub mod support_commands;
pub mod terminal_materialization;
pub mod transient_pane_facts;
pub mod update_commands;
pub mod upgrade_summary;
pub mod workspace_asset_contract;
pub mod workspace_commands;
pub mod yazi_materialization;
pub mod yazi_render_plan;
pub mod zellij_commands;
pub mod zellij_materialization;
pub mod zellij_render_plan;

pub use bridge::{CoreError, ErrorClass, error_envelope, success_envelope};
pub use command_metadata::{
    YzxCommandMetadataData, YzxExternBridgeSyncData, YzxExternBridgeSyncRequest,
    YzxExternBridgeSyncStatus, render_yzx_externs, render_yzx_help, sync_yzx_extern_bridge,
    yzx_command_metadata_data,
};
pub use config_commands::run_yzx_config;
pub use config_normalize::{NormalizeConfigData, NormalizeConfigRequest, normalize_config};
pub use config_state::{
    ComputeConfigStateRequest, ConfigStateData, RecordConfigStateData, RecordConfigStateRequest,
    compute_config_state, record_config_state,
};
pub use doctor_commands::{DoctorReportData, DoctorReportSummary, run_yzx_doctor};
pub use doctor_config_report::{
    DoctorConfigEvaluateData, DoctorConfigEvaluateRequest, evaluate_doctor_config_report,
};
pub use doctor_helix_report::{
    HelixDoctorEvaluateData, HelixDoctorEvaluateRequest, evaluate_helix_doctor_report,
};
pub use doctor_runtime_report::{
    DoctorRuntimeEvaluateData, DoctorRuntimeEvaluateRequest, evaluate_doctor_runtime_report,
};
pub use edit_commands::{run_yzx_edit, run_yzx_edit_config};
pub use front_door_commands::{
    run_internal_welcome_screen, run_yzx_screen, run_yzx_tutor, run_yzx_whats_new,
};
pub use ghostty_cursor_registry::{
    DEFAULT_GHOSTTY_TRAIL_DURATION, GHOSTTY_TRAIL_DURATION_MAX, GHOSTTY_TRAIL_DURATION_MIN,
};
pub use ghostty_materialization::{
    GhosttyCursorState, GhosttyMaterializationData, GhosttyMaterializationRequest,
    generate_ghostty_materialization,
};
pub use helix_materialization::{
    HelixImportNotice, HelixMaterializationData, HelixMaterializationRequest,
    generate_helix_materialization,
};
pub use home_manager_commands::run_yzx_home_manager;
pub use import_commands::run_yzx_import;
pub use initializer_commands::run_generate_shell_initializers;
pub use install_ownership_env::{
    install_ownership_request_from_env, install_ownership_request_from_env_with_runtime_dir,
};
pub use install_ownership_report::{
    DoctorInstallResult, HomeManagerPrepareArtifact, InstallOwnershipEvaluateData,
    InstallOwnershipEvaluateRequest, evaluate_install_ownership_report,
};
pub use keys_commands::run_yzx_keys;
pub use launch_commands::{run_yzx_desktop, run_yzx_enter, run_yzx_launch, run_yzx_restart};
pub use launch_materialization::{
    LaunchMaterializationData, LaunchMaterializationRequest,
    launch_materialization_request_from_env, prepare_launch_materialization,
};
pub use onboard_commands::run_yzx_onboard;
pub use profile_commands::{
    run_profile_create_run, run_profile_load_report, run_profile_print_report,
    run_profile_record_step, run_profile_wait_step,
};
pub use public_command_surface::{
    YzxCommandCategory, YzxCommandMetadata, YzxCommandParameter, YzxInternalNuRoutePlan,
    YzxMenuCategory, YzxParameterKind, YzxPublicRootRoute, classify_yzx_root_route,
    yzx_command_metadata,
};
pub use runtime_contract::{
    GeneratedLayoutCheckRequest, LaunchPreflightPayload, LinuxGhosttyDesktopGraphicsRequest,
    PreflightKind, RuntimeCheckData, RuntimeContractEvaluateData, RuntimeContractEvaluateRequest,
    RuntimeScriptCheckRequest, StartupLaunchPreflightData, StartupLaunchPreflightRequest,
    StartupPreflightPayload, TerminalCandidate, TerminalSupportCheckRequest,
    WorkingDirCheckRequest, WorkingDirKind, evaluate_runtime_contract,
    evaluate_startup_launch_preflight,
};
pub use runtime_env::{RuntimeEnvComputeData, RuntimeEnvComputeRequest, compute_runtime_env};
pub use runtime_materialization::{
    RuntimeMaterializationApplyData, RuntimeMaterializationPlanData,
    RuntimeMaterializationPlanRequest, RuntimeMaterializationRepairEvaluateRequest,
    RuntimeMaterializationRepairRunData, RuntimeMaterializationRunData, RuntimeRepairDirective,
    materialize_runtime_state, plan_runtime_materialization, repair_runtime_materialization,
};
pub use session_config_snapshot::{
    SESSION_CONFIG_SNAPSHOT_FILE_NAME, SESSION_CONFIG_SNAPSHOT_PATH_ENV,
    SESSION_CONFIG_SNAPSHOT_SCHEMA_VERSION, SessionConfigSnapshotCreateRequest,
    SessionConfigSnapshotData, SessionConfigSnapshotWriteData, SessionConfigSnapshotWriteRequest,
    load_session_config_snapshot_from_env, load_session_config_snapshot_from_path,
    load_session_facts_from_snapshot_path, session_config_snapshot_path,
    session_config_snapshot_path_from_env, write_session_config_snapshot,
    write_session_config_snapshot_for_launch,
};
pub use session_facts::{SessionFactsData, compute_session_facts_from_env};
pub use startup_facts::{StartupFactsData, compute_startup_facts_from_env};
pub use startup_handoff::{
    StartupHandoffArtifact, StartupHandoffCaptureData, StartupHandoffCaptureRequest,
    capture_startup_handoff_context,
};
pub use status_report::{StatusReportData, compute_status_report, session_config_snapshot_summary};
pub use support_commands::{run_yzx_sponsor, run_yzx_why};
pub use terminal_materialization::{
    TerminalGeneratedConfig, TerminalMaterializationData, TerminalMaterializationRequest,
    generate_terminal_materialization,
};
pub use transient_pane_facts::{TransientPaneFactsData, compute_transient_pane_facts_from_env};
pub use upgrade_summary::{
    UpgradeSummaryDisplayResult, UpgradeSummaryReport, build_upgrade_summary_report,
    current_release_headline, maybe_show_first_run_upgrade_summary, show_current_upgrade_summary,
};
pub use workspace_commands::{
    IntegrationFactsData, compute_integration_facts_from_env, run_yzx_cwd, run_yzx_popup,
    run_yzx_reveal,
};
pub use yazi_materialization::{
    YaziManagedFileStatus, YaziMaterializationData, YaziMaterializationRequest,
    generate_yazi_materialization,
};
pub use yazi_render_plan::{YaziRenderPlanData, YaziRenderPlanRequest, compute_yazi_render_plan};
pub use zellij_commands::{
    run_zellij_get_workspace_root, run_zellij_inspect_session, run_zellij_open_editor,
    run_zellij_open_editor_cwd, run_zellij_open_terminal, run_zellij_pipe, run_zellij_retarget,
    run_zellij_status_bus, run_zellij_status_bus_ai_activity, run_zellij_status_bus_token_budget,
    run_zellij_status_bus_workspace,
};
pub use zellij_materialization::{
    ZellijMaterializationData, ZellijMaterializationRequest, generate_zellij_materialization,
};
pub use zellij_render_plan::{
    ZellijRenderPlanData, ZellijRenderPlanRequest, compute_zellij_render_plan,
    managed_sidebar_layout_name,
};
