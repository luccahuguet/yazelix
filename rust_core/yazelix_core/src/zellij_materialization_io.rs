use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn write_text_atomic(path: &Path, content: &str) -> Result<(), CoreError> {
    write_bytes_atomic(path, content.as_bytes())
}

pub(crate) fn write_bytes_atomic(path: &Path, content: &[u8]) -> Result<(), CoreError> {
    let parent = path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "invalid_zellij_output_path",
            "Generated Zellij output path has no parent directory",
            "Report this as a Yazelix internal error.",
            json!({ "path": path.to_string_lossy() }),
        )
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        CoreError::io(
            "create_zellij_output_parent",
            "Could not create parent directory for generated Zellij output",
            "Check permissions for the Yazelix state directory and retry.",
            parent.to_string_lossy(),
            source,
        )
    })?;
    let temporary_path = path.with_file_name(format!(
        ".{}.yazelix-tmp-{}-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("zellij"),
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    fs::write(&temporary_path, content).map_err(|source| {
        CoreError::io(
            "write_zellij_output_temp",
            "Could not write temporary generated Zellij output",
            "Check permissions for the Yazelix state directory and retry.",
            temporary_path.to_string_lossy(),
            source,
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|source| {
        CoreError::io(
            "rename_zellij_output_temp",
            "Could not replace generated Zellij output",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })
}

pub(crate) fn read_text(path: &Path, code: &str) -> Result<String, CoreError> {
    fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            code,
            "Could not read a Zellij materialization input",
            "Check permissions or reinstall Yazelix if a runtime input is missing.",
            path.to_string_lossy(),
            source,
        )
    })
}

pub(crate) fn read_text_if_exists(path: &Path) -> Result<String, CoreError> {
    if path.exists() {
        read_text(path, "read_zellij_optional_input")
    } else {
        Ok(String::new())
    }
}

pub(crate) fn hash_file(path: &Path) -> Result<String, CoreError> {
    let bytes = fs::read(path).map_err(|source| {
        CoreError::io(
            "hash_zellij_input",
            "Could not hash a Zellij materialization input",
            "Check permissions or reinstall Yazelix if a runtime input is missing.",
            path.to_string_lossy(),
            source,
        )
    })?;
    Ok(hash_bytes(&bytes))
}

pub(crate) fn hash_text(value: &str) -> String {
    hash_bytes(value.as_bytes())
}

fn hash_bytes(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}
