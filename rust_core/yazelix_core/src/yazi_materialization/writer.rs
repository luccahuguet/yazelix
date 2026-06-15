use crate::atomic_fs::{write_bytes_atomic, write_text_atomic};
use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use yazelix_yazi_assets::{
    YaziConfigPackRenderRequest, YaziConfigPackTemplates, YaziRenderPlanData,
    render_yazi_config_pack,
};

const RUNTIME_DIR_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_DIR__";

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct YaziManagedFileStatus {
    pub path: String,
    pub changed: bool,
}

pub(super) struct YaziConfigPackWriteRequest<'a> {
    pub source_dir: &'a Path,
    pub output_dir: &'a Path,
    pub runtime_dir: &'a Path,
    pub render_plan: &'a YaziRenderPlanData,
    pub user_yazi_config: Option<&'a toml::Table>,
    pub user_keymap: Option<&'a toml::Table>,
    pub user_init_lua: Option<&'a str>,
    pub user_plugins_dir: &'a Path,
    pub user_flavors_dir: &'a Path,
    pub semantic_keymap: &'a toml::Table,
    pub sync_static_assets: bool,
}

pub(super) struct YaziConfigPackWriteData {
    pub missing_plugins: Vec<String>,
    pub synced_static_assets: bool,
    pub user_init_appended: bool,
    pub managed_files: Vec<YaziManagedFileStatus>,
}

pub(super) fn write_yazi_config_pack(
    request: &YaziConfigPackWriteRequest<'_>,
) -> Result<YaziConfigPackWriteData, CoreError> {
    fs::create_dir_all(request.output_dir).map_err(|source| {
        CoreError::io(
            "create_yazi_output_dir",
            "Could not create the managed Yazi output directory",
            "Check permissions for the Yazelix state directory and retry.",
            request.output_dir.to_string_lossy(),
            source,
        )
    })?;

    let should_sync_static_assets = request.sync_static_assets
        || bundled_yazi_assets_missing(
            request.source_dir,
            request.output_dir,
            request.runtime_dir,
        )?;
    if should_sync_static_assets {
        sync_bundled_yazi_assets(request.source_dir, request.output_dir, request.runtime_dir)?;
    }
    for (source_root, child) in [
        (request.user_plugins_dir, "plugins"),
        (request.user_flavors_dir, "flavors"),
    ] {
        sync_named_child_directories(
            source_root,
            &request.output_dir.join(child),
            request.runtime_dir,
        )?;
    }

    let available_plugins = available_requested_plugins(
        &request.output_dir.join("plugins"),
        &request.render_plan.init_lua.load_order,
    );
    let templates =
        YaziConfigPackTemplates::bundled().map_err(super::map_yazi_config_pack_error)?;
    let runtime_dir = request.runtime_dir.to_string_lossy().to_string();
    let starship_config_path = request
        .output_dir
        .join("yazelix_starship.toml")
        .to_string_lossy()
        .to_string();
    let rendered = render_yazi_config_pack(&YaziConfigPackRenderRequest {
        templates: &templates,
        runtime_dir: &runtime_dir,
        starship_config_path: &starship_config_path,
        render_plan: request.render_plan,
        user_yazi_config: request.user_yazi_config,
        user_keymap: request.user_keymap,
        user_init_lua: request.user_init_lua,
        semantic_keymap: request.semantic_keymap,
        available_plugins: &available_plugins,
    })
    .map_err(super::map_yazi_config_pack_error)?;
    let managed_files = vec![
        write_managed_text_if_changed(&request.output_dir.join("yazi.toml"), &rendered.yazi_toml)?,
        write_managed_text_if_changed(
            &request.output_dir.join("theme.toml"),
            &rendered.theme_toml,
        )?,
        write_managed_text_if_changed(
            &request.output_dir.join("keymap.toml"),
            &rendered.keymap_toml,
        )?,
        write_managed_text_if_changed(&request.output_dir.join("init.lua"), &rendered.init_lua)?,
    ];

    Ok(YaziConfigPackWriteData {
        missing_plugins: rendered.missing_plugins,
        synced_static_assets: should_sync_static_assets,
        user_init_appended: rendered.user_init_appended,
        managed_files,
    })
}

fn available_requested_plugins(plugins_dir: &Path, plugin_names: &[String]) -> BTreeSet<String> {
    plugin_names
        .iter()
        .filter(|name| plugins_dir.join(format!("{name}.yazi")).exists())
        .cloned()
        .collect()
}

pub(super) fn bundled_yazi_assets_missing(
    source_dir: &Path,
    output_dir: &Path,
    runtime_dir: &Path,
) -> Result<bool, CoreError> {
    let source_plugins = source_dir.join("plugins");
    let output_plugins = output_dir.join("plugins");
    let source_flavors = source_dir.join("flavors");
    let output_flavors = output_dir.join("flavors");
    let source_starship = source_dir.join("yazelix_starship.toml");
    let output_starship = output_dir.join("yazelix_starship.toml");

    Ok(
        asset_tree_missing_targets(&source_plugins, &output_plugins, runtime_dir)?
            || asset_tree_missing_targets(&source_flavors, &output_flavors, runtime_dir)?
            || (source_starship.exists()
                && asset_file_needs_sync(&source_starship, &output_starship, runtime_dir)?),
    )
}

pub(super) fn asset_tree_missing_targets(
    source_root: &Path,
    target_root: &Path,
    runtime_dir: &Path,
) -> Result<bool, CoreError> {
    if !source_root.exists() {
        return Ok(false);
    }
    if !target_root.exists() {
        return Ok(true);
    }

    let mut stack = vec![source_root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path).map_err(|source| {
            CoreError::io(
                "read_yazi_asset_source_dir",
                "Could not inspect bundled Yazi assets",
                "Reinstall Yazelix so the runtime includes a readable Yazi asset tree.",
                path.to_string_lossy(),
                source,
            )
        })? {
            let entry = entry.map_err(|source| {
                CoreError::io(
                    "read_yazi_asset_source_entry",
                    "Could not inspect bundled Yazi asset entries",
                    "Reinstall Yazelix so the runtime includes a readable Yazi asset tree.",
                    path.to_string_lossy(),
                    source,
                )
            })?;
            let source_path = entry.path();
            let source_metadata = fs::metadata(&source_path).map_err(|source| {
                CoreError::io(
                    "inspect_yazi_asset_source_entry",
                    "Could not inspect a bundled Yazi asset entry",
                    "Reinstall Yazelix so the runtime includes readable Yazi assets.",
                    path.to_string_lossy(),
                    source,
                )
            })?;
            if path == source_root && !source_metadata.is_dir() {
                continue;
            }
            let relative = source_path.strip_prefix(source_root).map_err(|_| {
                CoreError::classified(
                    ErrorClass::Internal,
                    "invalid_yazi_asset_relative_path",
                    "Could not resolve a bundled Yazi asset relative path",
                    "Report this as a Yazelix internal error.",
                    json!({
                        "source_root": source_root.to_string_lossy(),
                        "source_path": source_path.to_string_lossy(),
                    }),
                )
            })?;
            let target_path = target_root.join(relative);
            if source_metadata.is_dir() {
                if !target_path.is_dir() {
                    return Ok(true);
                }
                stack.push(source_path);
            } else if asset_file_needs_sync(&source_path, &target_path, runtime_dir)? {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn asset_file_needs_sync(
    source_path: &Path,
    target_path: &Path,
    runtime_dir: &Path,
) -> Result<bool, CoreError> {
    if !target_path.is_file() {
        return Ok(true);
    }
    let source_content = fs::read(source_path).map_err(|source| {
        CoreError::io(
            "read_yazi_asset_file",
            "Could not read a bundled Yazi asset file",
            "Reinstall Yazelix so the runtime includes readable Yazi assets.",
            source_path.to_string_lossy(),
            source,
        )
    })?;
    let expected = render_asset_content(&source_content, runtime_dir);
    Ok(fs::read(target_path)
        .map(|actual| actual != expected)
        .unwrap_or(true))
}

fn sync_bundled_yazi_assets(
    source_dir: &Path,
    output_dir: &Path,
    runtime_dir: &Path,
) -> Result<(), CoreError> {
    for child in ["plugins", "flavors"] {
        sync_named_child_directories(
            &source_dir.join(child),
            &output_dir.join(child),
            runtime_dir,
        )?;
    }
    sync_starship_config(
        &source_dir.join("yazelix_starship.toml"),
        &output_dir.join("yazelix_starship.toml"),
    )?;
    Ok(())
}

fn sync_named_child_directories(
    source_root: &Path,
    target_root: &Path,
    runtime_dir: &Path,
) -> Result<(), CoreError> {
    if !source_root.exists() {
        return Ok(());
    }
    fs::create_dir_all(target_root).map_err(|source| {
        CoreError::io(
            "create_yazi_asset_target_dir",
            "Could not create the managed Yazi asset directory",
            "Check permissions for the Yazelix state directory and retry.",
            target_root.to_string_lossy(),
            source,
        )
    })?;

    for entry in fs::read_dir(source_root).map_err(|source| {
        CoreError::io(
            "read_yazi_asset_root",
            "Could not inspect a Yazi asset directory",
            "Check permissions for the Yazelix config and runtime directories, then retry.",
            source_root.to_string_lossy(),
            source,
        )
    })? {
        let entry = entry.map_err(|source| {
            CoreError::io(
                "read_yazi_asset_entry",
                "Could not inspect a Yazi asset entry",
                "Check permissions for the Yazelix config and runtime directories, then retry.",
                source_root.to_string_lossy(),
                source,
            )
        })?;
        let source_path = entry.path();
        let source_metadata = fs::metadata(&source_path).map_err(|source| {
            CoreError::io(
                "inspect_yazi_asset_entry",
                "Could not inspect a Yazi asset entry",
                "Check permissions for the Yazelix config and runtime directories, then retry.",
                source_path.to_string_lossy(),
                source,
            )
        })?;
        if !source_metadata.is_dir() {
            continue;
        }
        let target_path = target_root.join(entry.file_name());
        if target_path.exists() {
            relax_permissions_recursively(&target_path)?;
            remove_path_recursively(&target_path)?;
        }
        copy_path_recursive(&source_path, &target_path, runtime_dir)?;
        ensure_writable_recursively(&target_path)?;
    }

    Ok(())
}

pub(super) fn sync_starship_config(
    source_path: &Path,
    target_path: &Path,
) -> Result<(), CoreError> {
    if !source_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Io,
            "missing_yazi_starship_config",
            format!(
                "Missing bundled Yazi Starship config at: {}",
                source_path.to_string_lossy()
            ),
            "Reinstall Yazelix so the runtime includes configs/yazi/yazelix_starship.toml.",
            json!({ "path": source_path.to_string_lossy() }),
        ));
    }

    let content = fs::read(source_path).map_err(|source| {
        CoreError::io(
            "read_yazi_starship_config",
            "Could not read the bundled Yazi Starship config",
            "Reinstall Yazelix so the runtime includes a readable Yazi Starship config.",
            source_path.to_string_lossy(),
            source,
        )
    })?;
    write_bytes_atomic(target_path, &content)?;
    set_writable(target_path, false)?;
    Ok(())
}

fn copy_path_recursive(source: &Path, target: &Path, runtime_dir: &Path) -> Result<(), CoreError> {
    let source_metadata = fs::metadata(source).map_err(|source_err| {
        CoreError::io(
            "inspect_yazi_asset_source",
            "Could not inspect a Yazi asset path",
            "Check permissions for the Yazelix config and runtime directories, then retry.",
            source.to_string_lossy(),
            source_err,
        )
    })?;

    if source_metadata.is_dir() {
        fs::create_dir_all(target).map_err(|source_err| {
            CoreError::io(
                "create_yazi_asset_dir",
                "Could not create a managed Yazi asset directory",
                "Check permissions for the Yazelix state directory and retry.",
                target.to_string_lossy(),
                source_err,
            )
        })?;
        for entry in fs::read_dir(source).map_err(|source_err| {
            CoreError::io(
                "read_yazi_asset_dir",
                "Could not read a Yazi asset directory",
                "Check permissions for the Yazelix config and runtime directories, then retry.",
                source.to_string_lossy(),
                source_err,
            )
        })? {
            let entry = entry.map_err(|source_err| {
                CoreError::io(
                    "read_yazi_asset_dir_entry",
                    "Could not read a Yazi asset entry",
                    "Check permissions for the Yazelix config and runtime directories, then retry.",
                    source.to_string_lossy(),
                    source_err,
                )
            })?;
            copy_path_recursive(&entry.path(), &target.join(entry.file_name()), runtime_dir)?;
        }
        return Ok(());
    }

    let bytes = fs::read(source).map_err(|source_err| {
        CoreError::io(
            "read_yazi_asset_file",
            "Could not read a Yazi asset file",
            "Check permissions for the Yazelix config and runtime directories, then retry.",
            source.to_string_lossy(),
            source_err,
        )
    })?;
    let rendered = render_asset_content(&bytes, runtime_dir);
    write_bytes_atomic(target, &rendered)?;
    Ok(())
}

fn render_asset_content(bytes: &[u8], runtime_dir: &Path) -> Vec<u8> {
    match std::str::from_utf8(bytes) {
        Ok(text) => render_runtime_root_placeholders(text, runtime_dir).into_bytes(),
        Err(_) => bytes.to_vec(),
    }
}

pub(super) fn render_runtime_root_placeholders(content: &str, runtime_dir: &Path) -> String {
    content.replace(
        RUNTIME_DIR_PLACEHOLDER,
        runtime_dir.to_string_lossy().as_ref(),
    )
}

fn write_managed_text_if_changed(
    path: &Path,
    content: &str,
) -> Result<YaziManagedFileStatus, CoreError> {
    let changed = write_text_atomic_if_changed(path, content)?;
    Ok(YaziManagedFileStatus {
        path: path.to_string_lossy().to_string(),
        changed,
    })
}

fn write_text_atomic_if_changed(path: &Path, content: &str) -> Result<bool, CoreError> {
    if fs::read_to_string(path).ok().as_deref() == Some(content) {
        return Ok(false);
    }
    write_text_atomic(path, content)?;
    Ok(true)
}

fn remove_path_recursively(path: &Path) -> Result<(), CoreError> {
    let file_type = fs::symlink_metadata(path).map_err(|source| {
        CoreError::io(
            "inspect_yazi_remove_target",
            "Could not inspect an existing managed Yazi asset path",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })?;
    if file_type.is_dir() {
        fs::remove_dir_all(path).map_err(|source| {
            CoreError::io(
                "remove_yazi_asset_dir",
                "Could not remove an existing managed Yazi asset directory",
                "Check permissions for the Yazelix state directory and retry.",
                path.to_string_lossy(),
                source,
            )
        })
    } else {
        fs::remove_file(path).map_err(|source| {
            CoreError::io(
                "remove_yazi_asset_file",
                "Could not remove an existing managed Yazi asset file",
                "Check permissions for the Yazelix state directory and retry.",
                path.to_string_lossy(),
                source,
            )
        })
    }
}

fn relax_permissions_recursively(path: &Path) -> Result<(), CoreError> {
    let file_type = fs::symlink_metadata(path).map_err(|source| {
        CoreError::io(
            "inspect_yazi_permission_target",
            "Could not inspect an existing managed Yazi asset path",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })?;
    if file_type.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| {
            CoreError::io(
                "read_yazi_permission_dir",
                "Could not inspect an existing managed Yazi asset directory",
                "Check permissions for the Yazelix state directory and retry.",
                path.to_string_lossy(),
                source,
            )
        })? {
            let entry = entry.map_err(|source| {
                CoreError::io(
                    "read_yazi_permission_entry",
                    "Could not inspect an existing managed Yazi asset entry",
                    "Check permissions for the Yazelix state directory and retry.",
                    path.to_string_lossy(),
                    source,
                )
            })?;
            relax_permissions_recursively(&entry.path())?;
        }
    }
    set_writable(path, file_type.is_dir())
}

fn ensure_writable_recursively(path: &Path) -> Result<(), CoreError> {
    let file_type = fs::symlink_metadata(path).map_err(|source| {
        CoreError::io(
            "inspect_yazi_written_asset",
            "Could not inspect a managed Yazi asset path after writing it",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })?;
    if file_type.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| {
            CoreError::io(
                "read_yazi_written_asset_dir",
                "Could not inspect a managed Yazi asset directory after writing it",
                "Check permissions for the Yazelix state directory and retry.",
                path.to_string_lossy(),
                source,
            )
        })? {
            let entry = entry.map_err(|source| {
                CoreError::io(
                    "read_yazi_written_asset_entry",
                    "Could not inspect a managed Yazi asset entry after writing it",
                    "Check permissions for the Yazelix state directory and retry.",
                    path.to_string_lossy(),
                    source,
                )
            })?;
            ensure_writable_recursively(&entry.path())?;
        }
    }
    set_writable(path, file_type.is_dir())
}

fn set_writable(path: &Path, is_dir: bool) -> Result<(), CoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if is_dir { 0o755 } else { 0o644 };
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|source| {
            CoreError::io(
                "set_yazi_permissions",
                "Could not adjust permissions on a managed Yazi path",
                "Check permissions for the Yazelix state directory and retry.",
                path.to_string_lossy(),
                source,
            )
        })?;
    }
    #[cfg(not(unix))]
    {
        let mut permissions = fs::metadata(path)
            .map_err(|source| {
                CoreError::io(
                    "read_yazi_permissions",
                    "Could not inspect permissions on a managed Yazi path",
                    "Check permissions for the Yazelix state directory and retry.",
                    path.to_string_lossy(),
                    source,
                )
            })?
            .permissions();
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions).map_err(|source| {
            CoreError::io(
                "set_yazi_permissions",
                "Could not adjust permissions on a managed Yazi path",
                "Check permissions for the Yazelix state directory and retry.",
                path.to_string_lossy(),
                source,
            )
        })?;
    }
    Ok(())
}
