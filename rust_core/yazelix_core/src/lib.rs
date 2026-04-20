pub mod active_config_surface;
pub mod bridge;
pub mod command_metadata;
pub mod config_normalize;
pub mod config_state;
pub mod control_plane;
pub mod doctor_config_report;
pub mod doctor_helix_report;
pub mod doctor_runtime_report;
pub mod helix_materialization;
pub mod install_ownership_report;
pub mod runtime_contract;
pub mod runtime_env;
pub mod runtime_materialization;
pub mod status_report;
pub mod update_commands;
pub mod yazi_materialization;
pub mod yazi_render_plan;
pub mod zellij_materialization;
pub mod zellij_render_plan;

pub use bridge::{error_envelope, success_envelope, CoreError, ErrorClass};
pub use command_metadata::{
    render_yzx_externs, render_yzx_help, yzx_command_metadata, yzx_command_metadata_data,
    YzxCommandCategory, YzxCommandMetadata, YzxCommandMetadataData, YzxCommandParameter,
    YzxParameterKind,
};
pub use config_normalize::{normalize_config, NormalizeConfigData, NormalizeConfigRequest};
pub use config_state::{
    compute_config_state, record_config_state, ComputeConfigStateRequest, ConfigStateData,
    RecordConfigStateData, RecordConfigStateRequest,
};
pub use doctor_config_report::{
    evaluate_doctor_config_report, DoctorConfigEvaluateData, DoctorConfigEvaluateRequest,
};
pub use doctor_helix_report::{
    evaluate_helix_doctor_report, HelixDoctorEvaluateData, HelixDoctorEvaluateRequest,
};
pub use doctor_runtime_report::{
    evaluate_doctor_runtime_report, DoctorRuntimeEvaluateData, DoctorRuntimeEvaluateRequest,
};
pub use helix_materialization::{
    generate_helix_materialization, HelixImportNotice, HelixMaterializationData,
    HelixMaterializationRequest,
};
pub use install_ownership_report::{
    evaluate_install_ownership_report, DoctorInstallResult, HomeManagerPrepareArtifact,
    InstallOwnershipEvaluateData, InstallOwnershipEvaluateRequest,
};
pub use runtime_contract::{
    evaluate_runtime_contract, evaluate_startup_launch_preflight, GeneratedLayoutCheckRequest,
    LaunchPreflightPayload, LinuxGhosttyDesktopGraphicsRequest, PreflightKind, RuntimeCheckData,
    RuntimeContractEvaluateData, RuntimeContractEvaluateRequest, RuntimeScriptCheckRequest,
    StartupLaunchPreflightData, StartupLaunchPreflightRequest, StartupPreflightPayload,
    TerminalCandidate, TerminalSupportCheckRequest, WorkingDirCheckRequest, WorkingDirKind,
};
pub use runtime_env::{compute_runtime_env, RuntimeEnvComputeData, RuntimeEnvComputeRequest};
pub use runtime_materialization::{
    apply_runtime_materialization, evaluate_runtime_materialization_repair,
    plan_runtime_materialization, RuntimeArtifact, RuntimeMaterializationApplyData,
    RuntimeMaterializationApplyRequest, RuntimeMaterializationPlanData,
    RuntimeMaterializationPlanRequest, RuntimeMaterializationRepairEvaluateData,
    RuntimeMaterializationRepairEvaluateRequest, RuntimeRepairDirective,
};
pub use status_report::{compute_status_report, StatusReportData};
pub use yazi_materialization::{
    generate_yazi_materialization, YaziManagedFileStatus, YaziMaterializationData,
    YaziMaterializationRequest,
};
pub use yazi_render_plan::{compute_yazi_render_plan, YaziRenderPlanData, YaziRenderPlanRequest};
pub use zellij_materialization::{
    generate_zellij_materialization, ZellijMaterializationData, ZellijMaterializationRequest,
};
pub use zellij_render_plan::{
    compute_zellij_render_plan, ZellijRenderPlanData, ZellijRenderPlanRequest,
};
