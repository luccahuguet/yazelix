pub mod active_config_surface;
pub mod bridge;
pub mod config_normalize;
pub mod config_state;
pub mod control_plane;
pub mod doctor_config_report;
pub mod doctor_helix_report;
pub mod doctor_runtime_report;
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

pub use bridge::{CoreError, ErrorClass, error_envelope, success_envelope};
pub use config_normalize::{NormalizeConfigData, NormalizeConfigRequest, normalize_config};
pub use config_state::{
    ComputeConfigStateRequest, ConfigStateData, RecordConfigStateData, RecordConfigStateRequest,
    compute_config_state, record_config_state,
};
pub use doctor_config_report::{
    DoctorConfigEvaluateData, DoctorConfigEvaluateRequest, evaluate_doctor_config_report,
};
pub use doctor_helix_report::{
    HelixDoctorEvaluateData, HelixDoctorEvaluateRequest, evaluate_helix_doctor_report,
};
pub use doctor_runtime_report::{
    DoctorRuntimeEvaluateData, DoctorRuntimeEvaluateRequest, evaluate_doctor_runtime_report,
};
pub use install_ownership_report::{
    DoctorInstallResult, HomeManagerPrepareArtifact, InstallOwnershipEvaluateData,
    InstallOwnershipEvaluateRequest, evaluate_install_ownership_report,
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
    RuntimeArtifact, RuntimeMaterializationApplyData, RuntimeMaterializationApplyRequest,
    RuntimeMaterializationPlanData, RuntimeMaterializationPlanRequest,
    RuntimeMaterializationRepairEvaluateData, RuntimeMaterializationRepairEvaluateRequest,
    RuntimeRepairDirective, apply_runtime_materialization, evaluate_runtime_materialization_repair,
    plan_runtime_materialization,
};
pub use status_report::{StatusReportData, compute_status_report};
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
