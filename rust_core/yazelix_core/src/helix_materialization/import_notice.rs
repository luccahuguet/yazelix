use super::{HelixImportNotice, HelixMaterializationRequest};
use crate::bridge::CoreError;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn build_import_notice(
    request: &HelixMaterializationRequest,
    user_config_path: &Path,
) -> Result<Option<HelixImportNotice>, CoreError> {
    let native_config_path = resolve_native_helix_config_path();

    if !native_config_path.exists() {
        return Ok(None);
    }

    if user_config_path.exists() {
        return Ok(None);
    }

    let notice_dir = request.state_dir.join("state").join("helix");
    fs::create_dir_all(&notice_dir).map_err(|source| {
        CoreError::io(
            "create_helix_notice_dir",
            "Could not create the Helix notice state directory",
            "Check permissions for the Yazelix state directory and retry.",
            notice_dir.to_string_lossy(),
            source,
        )
    })?;

    let marker_path = notice_dir.join("import_notice_seen");
    if marker_path.exists() {
        return Ok(None);
    }

    fs::write(&marker_path, "").map_err(|source| {
        CoreError::io(
            "write_helix_notice_marker",
            "Could not write the Helix import notice marker",
            "Check permissions for the Yazelix state directory and retry.",
            marker_path.to_string_lossy(),
            source,
        )
    })?;

    Ok(Some(HelixImportNotice {
        marker_path: marker_path.to_string_lossy().into_owned(),
        lines: vec![
            "ℹ️  Yazelix is using its managed Helix config.".into(),
            format!(
                "   Personal Helix config detected at: {}",
                native_config_path.display()
            ),
            "   If you want Yazelix-managed Helix sessions to reuse it, run: yzx import helix"
                .into(),
        ],
    }))
}

fn resolve_native_helix_config_path() -> PathBuf {
    let xdg_config_home = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.into())
            }
        })
        .unwrap_or_else(|| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".config"))
                .unwrap_or_else(|_| PathBuf::from("."))
        });

    xdg_config_home.join("helix").join("config.toml")
}
