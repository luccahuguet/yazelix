pub mod active_config_surface;
pub mod bridge;
pub mod command_metadata;
pub mod config_commands;
pub mod config_normalize;
pub mod config_state;
pub mod control_plane;
pub mod doctor_commands;
pub mod doctor_config_report;
pub mod doctor_helix_report;
pub mod doctor_runtime_report;
pub mod ghostty_materialization;
pub mod helix_materialization;
pub mod home_manager_commands;
pub mod install_ownership_env;
pub mod install_ownership_report;
pub mod internal_nu_runner;
pub mod keys_commands;
pub mod public_command_surface;
pub mod runtime_contract;
pub mod runtime_env;
pub mod runtime_materialization;
pub mod status_report;
pub mod support_commands;
pub mod terminal_materialization;
pub mod update_commands;
pub mod yazi_materialization;
pub mod yazi_render_plan;
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
pub use ghostty_materialization::{
    GhosttyCursorState, GhosttyMaterializationData, GhosttyMaterializationRequest,
    generate_ghostty_materialization,
};
pub use helix_materialization::{
    HelixImportNotice, HelixMaterializationData, HelixMaterializationRequest,
    generate_helix_materialization,
};
pub use home_manager_commands::run_yzx_home_manager;
pub use install_ownership_report::{
    DoctorInstallResult, HomeManagerPrepareArtifact, InstallOwnershipEvaluateData,
    InstallOwnershipEvaluateRequest, evaluate_install_ownership_report,
};
pub use keys_commands::run_yzx_keys;
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
pub use status_report::{StatusReportData, compute_status_report};
pub use support_commands::{run_yzx_sponsor, run_yzx_why};
pub use terminal_materialization::{
    TerminalGeneratedConfig, TerminalMaterializationData, TerminalMaterializationRequest,
    generate_terminal_materialization,
};
pub use yazi_materialization::{
    YaziManagedFileStatus, YaziMaterializationData, YaziMaterializationRequest,
    generate_yazi_materialization,
};
pub use yazi_render_plan::{YaziRenderPlanData, YaziRenderPlanRequest, compute_yazi_render_plan};
pub use zellij_materialization::{
    ZellijMaterializationData, ZellijMaterializationRequest, generate_zellij_materialization,
};
pub use zellij_render_plan::{
    ZellijRenderPlanData, ZellijRenderPlanRequest, compute_zellij_render_plan,
};
