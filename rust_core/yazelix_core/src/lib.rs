pub mod active_config_surface;
pub mod bridge;
pub mod config_normalize;
pub mod config_state;
pub mod control_plane;
pub mod runtime_contract;
pub mod runtime_env;
pub mod runtime_materialization;
pub mod status_report;
pub mod yazi_render_plan;
pub mod zellij_render_plan;

pub use bridge::{error_envelope, success_envelope, CoreError, ErrorClass};
pub use config_normalize::{normalize_config, NormalizeConfigData, NormalizeConfigRequest};
pub use config_state::{
    compute_config_state, record_config_state, ComputeConfigStateRequest, ConfigStateData,
    RecordConfigStateData, RecordConfigStateRequest,
};
pub use runtime_contract::{
    evaluate_runtime_contract, GeneratedLayoutCheckRequest, LinuxGhosttyDesktopGraphicsRequest,
    RuntimeCheckData, RuntimeContractEvaluateData, RuntimeContractEvaluateRequest,
    RuntimeScriptCheckRequest, TerminalCandidate, TerminalSupportCheckRequest,
    WorkingDirCheckRequest, WorkingDirKind,
};
pub use runtime_env::{compute_runtime_env, RuntimeEnvComputeData, RuntimeEnvComputeRequest};
pub use runtime_materialization::{
    apply_runtime_materialization, plan_runtime_materialization, RuntimeArtifact,
    RuntimeMaterializationApplyData, RuntimeMaterializationApplyRequest,
    RuntimeMaterializationPlanData, RuntimeMaterializationPlanRequest,
};
pub use status_report::{compute_status_report, StatusReportData};
pub use yazi_render_plan::{compute_yazi_render_plan, YaziRenderPlanData, YaziRenderPlanRequest};
pub use zellij_render_plan::{
    compute_zellij_render_plan, ZellijRenderPlanData, ZellijRenderPlanRequest,
};
