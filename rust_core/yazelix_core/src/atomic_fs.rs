// Test lane: maintainer
use crate::bridge::{CoreError, ErrorClass};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn write_text_atomic(path: &Path, content: &str) -> Result<(), CoreError> {
    write_bytes_atomic(path, content.as_bytes())
}

pub(crate) fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub(crate) fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

pub(crate) fn write_bytes_atomic(path: &Path, content: &[u8]) -> Result<(), CoreError> {
    if let Ok(existing) = fs::read(path) {
        if existing == content {
            return Ok(());
        }
    }

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

#[cfg(test)]
mod tests {
    use super::write_bytes_atomic;
    use std::fs;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(unix)]
    // Regression: unchanged generated files do not require write access to their parent directory.
    #[test]
    fn skips_matching_content_in_read_only_directory() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("generated.toml");
        fs::write(&target, b"same").unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();

        let result = write_bytes_atomic(&target, b"same");

        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.is_ok());
        assert_eq!(fs::read(&target).unwrap(), b"same");
    }

    #[cfg(unix)]
    // Regression: changed generated files still report read-only runtime output as an error.
    #[test]
    fn still_errors_when_read_only_directory_needs_rewrite() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("generated.toml");
        fs::write(&target, b"old").unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();

        let result = write_bytes_atomic(&target, b"new");

        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.is_err());
        assert_eq!(fs::read(&target).unwrap(), b"old");
    }
}
