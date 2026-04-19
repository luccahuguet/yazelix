pub mod bridge;
pub mod config_normalize;

pub use bridge::{CoreError, ErrorClass, error_envelope, success_envelope};
pub use config_normalize::{NormalizeConfigData, NormalizeConfigRequest, normalize_config};
