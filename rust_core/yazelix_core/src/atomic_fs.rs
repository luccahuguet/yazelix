use crate::bridge::{CoreError, ErrorClass};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn write_text_atomic(path: &Path, content: &str) -> Result<(), CoreError> {
    write_bytes_atomic(path, content.as_bytes())
}

pub(crate) fn write_text_atomic_create_new(path: &Path, content: &str) -> Result<(), CoreError> {
    write_bytes_atomic_inner(path, content.as_bytes(), None, false)
}

pub(crate) fn write_text_atomic_with_permissions(
    path: &Path,
    content: &str,
    permissions: &fs::Permissions,
) -> Result<(), CoreError> {
    write_bytes_atomic_inner(path, content.as_bytes(), Some(permissions), true)
}

pub(crate) fn write_text_atomic_create_new_with_permissions(
    path: &Path,
    content: &str,
    permissions: &fs::Permissions,
) -> Result<(), CoreError> {
    write_bytes_atomic_inner(path, content.as_bytes(), Some(permissions), false)
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
    write_bytes_atomic_inner(path, content, None, true)
}

fn write_bytes_atomic_inner(
    path: &Path,
    content: &[u8],
    permissions: Option<&fs::Permissions>,
    replace: bool,
) -> Result<(), CoreError> {
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

    let write_result = permissions
        .map_or(Ok(()), |permissions| {
            temp_file.set_permissions(permissions.clone())
        })
        .and_then(|()| temp_file.write_all(content))
        .and_then(|()| temp_file.sync_all());
    drop(temp_file);
    if let Err(source) = write_result {
        let source = cleanup_error(&temp_path, source);
        return Err(CoreError::io(
            "atomic_write_content",
            format!("Could not write temporary file {}.", temp_path.display()),
            "Check permissions and disk space, then retry.",
            temp_path.to_string_lossy(),
            source,
        ));
    }

    if replace {
        return fs::rename(&temp_path, path).map_err(|source| {
            let source = cleanup_error(&temp_path, source);
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
        });
    }

    fs::hard_link(&temp_path, path).map_err(|source| {
        let source = cleanup_error(&temp_path, source);
        CoreError::io(
            "atomic_write_create_new",
            format!("Could not atomically create {}.", path.display()),
            "Preserve the existing path if it appeared concurrently, or fix directory permissions, then retry.",
            path.display().to_string(),
            source,
        )
    })?;
    sync_parent_directory(parent).map_err(|source| {
        let source = cleanup_error(&temp_path, source);
        CoreError::io(
            "atomic_write_parent_sync",
            format!(
                "Created {} but could not make its directory entry durable.",
                path.display()
            ),
            "The target is complete; preserve it and retry after fixing the filesystem error.",
            parent.to_string_lossy(),
            source,
        )
    })?;
    fs::remove_file(&temp_path).map_err(|source| {
        CoreError::io(
            "atomic_write_temp_cleanup",
            format!(
                "Created {} but could not remove temporary link {}.",
                path.display(),
                temp_path.display()
            ),
            "The target is complete; remove the reported temporary link before retrying.",
            temp_path.display().to_string(),
            source,
        )
    })
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> io::Result<()> {
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> io::Result<()> {
    Ok(())
}

fn cleanup_error(temp_path: &Path, source: std::io::Error) -> std::io::Error {
    match fs::remove_file(temp_path) {
        Ok(()) => source,
        Err(cleanup) if cleanup.kind() == std::io::ErrorKind::NotFound => source,
        Err(cleanup) => std::io::Error::new(
            source.kind(),
            format!(
                "{source}; also could not remove {}: {cleanup}",
                temp_path.display()
            ),
        ),
    }
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
