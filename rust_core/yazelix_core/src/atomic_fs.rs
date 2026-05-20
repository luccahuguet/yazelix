use crate::bridge::{CoreError, ErrorClass};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn write_text_atomic(path: &Path, content: &str) -> Result<(), CoreError> {
    write_bytes_atomic(path, content.as_bytes())
}

pub(crate) fn write_bytes_atomic(path: &Path, content: &[u8]) -> Result<(), CoreError> {
    let parent = path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "atomic_write_no_parent",
            format!(
                "Cannot atomically write path without a parent: {}",
                path.display()
            ),
            "Use a path inside the Yazelix state directory.",
            serde_json::json!({ "path": path.to_string_lossy() }),
        )
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        CoreError::io(
            "atomic_write_mkdir",
            format!("Could not create parent directory {}.", parent.display()),
            "Check permissions for the Yazelix state directory and retry.",
            parent.to_string_lossy(),
            source,
        )
    })?;

    let temp_path = create_temp_file_path(path);
    let mut temp_file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
    {
        Ok(file) => file,
        Err(source) => {
            return Err(CoreError::io(
                "atomic_write_create",
                format!("Could not create temporary file {}.", temp_path.display()),
                "Check permissions for the Yazelix state directory and retry.",
                temp_path.to_string_lossy(),
                source,
            ));
        }
    };

    let write_result = temp_file
        .write_all(content)
        .and_then(|()| temp_file.sync_all());
    drop(temp_file);
    if let Err(source) = write_result {
        let _ = fs::remove_file(&temp_path);
        return Err(CoreError::io(
            "atomic_write_content",
            format!("Could not write temporary file {}.", temp_path.display()),
            "Check permissions and disk space, then retry.",
            temp_path.to_string_lossy(),
            source,
        ));
    }

    fs::rename(&temp_path, path).map_err(|source| {
        let _ = fs::remove_file(&temp_path);
        CoreError::io(
            "atomic_write_rename",
            format!(
                "Could not replace {} with temporary file {}.",
                path.display(),
                temp_path.display()
            ),
            "Check permissions for the Yazelix state directory and retry.",
            format!("{} -> {}", temp_path.display(), path.display()),
            source,
        )
    })
}

fn create_temp_file_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("yazelix-generated");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    path.with_file_name(format!(
        ".{file_name}.yazelix-tmp-{}-{nanos}",
        std::process::id()
    ))
}
