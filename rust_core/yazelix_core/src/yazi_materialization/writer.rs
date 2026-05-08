use super::YaziManagedFileStatus;
use crate::bridge::{CoreError, ErrorClass};
use crate::yazi_render_plan::{ThemeFlavorPlan, YaziRenderPlanData};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use toml::Value as TomlValue;

const RUNTIME_DIR_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_DIR__";

pub(super) struct YaziConfigPackWriteRequest<'a> {
    pub source_dir: &'a Path,
    pub output_dir: &'a Path,
    pub runtime_dir: &'a Path,
    pub render_plan: &'a YaziRenderPlanData,
    pub user_yazi_config: Option<&'a toml::Table>,
    pub user_keymap: Option<&'a toml::Table>,
    pub user_init_lua: Option<&'a str>,
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

    let yazi_toml_status = write_generated_yazi_toml(
        request.source_dir,
        request.output_dir,
        request.runtime_dir,
        request.render_plan,
        request.user_yazi_config,
    )?;
    let theme_toml_status =
        write_generated_theme_toml(request.source_dir, request.output_dir, request.render_plan)?;
    let keymap_status = write_generated_keymap_toml(
        request.source_dir,
        request.output_dir,
        request.runtime_dir,
        request.user_keymap,
        request.semantic_keymap,
    )?;

    let should_sync_static_assets = request.sync_static_assets
        || bundled_yazi_assets_missing(request.source_dir, request.output_dir)?;
    if should_sync_static_assets {
        sync_bundled_yazi_assets(request.source_dir, request.output_dir, request.runtime_dir)?;
    }

    let (init_status, missing_plugins, user_init_appended) = write_generated_init_lua(
        request.output_dir,
        request.runtime_dir,
        request.render_plan,
        request.user_init_lua,
    )?;

    Ok(YaziConfigPackWriteData {
        missing_plugins,
        synced_static_assets: should_sync_static_assets,
        user_init_appended,
        managed_files: vec![
            yazi_toml_status,
            theme_toml_status,
            keymap_status,
            init_status,
        ],
    })
}

fn write_generated_yazi_toml(
    source_dir: &Path,
    output_dir: &Path,
    runtime_dir: &Path,
    render_plan: &YaziRenderPlanData,
    user_config: Option<&toml::Table>,
) -> Result<YaziManagedFileStatus, CoreError> {
    let base_path = source_dir.join("yazelix_yazi.toml");
    let base_config = read_required_toml_table(
        &base_path,
        "read_yazi_base_config",
        "Could not read the bundled Yazi base config",
        "Reinstall Yazelix so the runtime includes configs/yazi/yazelix_yazi.toml.",
    )?;

    let mut final_config = if let Some(user_config) = user_config {
        merge_yazi_toml_config(base_config.clone(), user_config.clone())
    } else {
        base_config.clone()
    };

    preserve_yazelix_edit_opener(&base_config, &mut final_config);
    if !render_plan.git_plugin_enabled {
        final_config.remove("plugin");
    }
    upsert_nested_string(
        &mut final_config,
        &["manager"],
        "sort_by",
        &render_plan.sort_by,
    );

    let user_note = if user_config.is_some() {
        "User config merged from:"
    } else {
        "To add custom settings, create:"
    };
    let header = generated_header(
        "#",
        "AUTO-GENERATED YAZI CONFIG",
        &[
            "",
            user_note,
            "  ~/.config/yazelix/yazi.toml",
            "Dynamic settings from ~/.config/yazelix/settings.jsonc:",
            "  [yazi] sort_by, plugins",
            "",
        ],
    );
    let config_content = render_runtime_root_placeholders(
        &toml_to_string_pretty(&TomlValue::Table(final_config))?,
        runtime_dir,
    );
    let target = output_dir.join("yazi.toml");
    let changed = write_text_atomic_if_changed(&target, &format!("{header}{config_content}"))?;
    Ok(YaziManagedFileStatus {
        path: target.to_string_lossy().to_string(),
        changed,
    })
}

fn write_generated_theme_toml(
    source_dir: &Path,
    output_dir: &Path,
    render_plan: &YaziRenderPlanData,
) -> Result<YaziManagedFileStatus, CoreError> {
    let source_path = source_dir.join("yazelix_theme.toml");
    let mut base_theme = if source_path.exists() {
        read_required_toml_table(
            &source_path,
            "read_yazi_theme_base",
            "Could not read the bundled Yazi theme base config",
            "Reinstall Yazelix so the runtime includes configs/yazi/yazelix_theme.toml.",
        )?
    } else {
        toml::Table::new()
    };

    if let ThemeFlavorPlan::Uniform { flavor } = &render_plan.theme_flavor {
        let mut flavor_table = toml::Table::new();
        flavor_table.insert("dark".into(), TomlValue::String(flavor.clone()));
        flavor_table.insert("light".into(), TomlValue::String(flavor.clone()));
        base_theme.insert("flavor".into(), TomlValue::Table(flavor_table));
    }

    let current_theme = format!("Current theme: {}", render_plan.resolved_theme);
    let header = generated_header(
        "#",
        "AUTO-GENERATED YAZI THEME CONFIG",
        &[
            "",
            "To customize theme, edit:",
            "  ~/.config/yazelix/settings.jsonc",
            "  [yazi] theme = \"...\"",
            "",
            current_theme.as_str(),
        ],
    );

    let config_content = if base_theme.is_empty() {
        String::new()
    } else {
        toml_to_string_pretty(&TomlValue::Table(base_theme))?
    };
    let target = output_dir.join("theme.toml");
    let changed = write_text_atomic_if_changed(&target, &format!("{header}{config_content}"))?;
    Ok(YaziManagedFileStatus {
        path: target.to_string_lossy().to_string(),
        changed,
    })
}

fn write_generated_keymap_toml(
    source_dir: &Path,
    output_dir: &Path,
    runtime_dir: &Path,
    user_keymap: Option<&toml::Table>,
    semantic_keymap: &toml::Table,
) -> Result<YaziManagedFileStatus, CoreError> {
    let base_path = source_dir.join("yazelix_keymap.toml");
    let mut base_keymap = read_required_toml_table(
        &base_path,
        "read_yazi_keymap_base",
        "Could not read the bundled Yazi keymap config",
        "Reinstall Yazelix so the runtime includes configs/yazi/yazelix_keymap.toml.",
    )?;
    base_keymap = merge_yazi_keymap(base_keymap, semantic_keymap.clone());

    let final_keymap = if let Some(user_keymap) = user_keymap {
        merge_yazi_keymap(base_keymap, user_keymap.clone())
    } else {
        base_keymap
    };

    let header = generated_header(
        "#",
        "AUTO-GENERATED YAZI KEYMAP",
        &[
            "",
            "To add custom keybindings, create:",
            "  ~/.config/yazelix/yazi_keymap.toml",
            "",
        ],
    );
    let keymap_content = render_runtime_root_placeholders(
        &toml_to_string_pretty(&TomlValue::Table(final_keymap))?,
        runtime_dir,
    );
    let target = output_dir.join("keymap.toml");
    let changed = write_text_atomic_if_changed(&target, &format!("{header}{keymap_content}"))?;
    Ok(YaziManagedFileStatus {
        path: target.to_string_lossy().to_string(),
        changed,
    })
}

fn write_generated_init_lua(
    output_dir: &Path,
    runtime_dir: &Path,
    render_plan: &YaziRenderPlanData,
    user_init_lua: Option<&str>,
) -> Result<(YaziManagedFileStatus, Vec<String>, bool), CoreError> {
    let plugins_dir = output_dir.join("plugins");
    let core_plugins = &render_plan.init_lua.core_plugins;
    let all_plugins = &render_plan.init_lua.load_order;
    let missing_plugins = all_plugins
        .iter()
        .filter(|name| !plugins_dir.join(format!("{name}.yazi")).exists())
        .cloned()
        .collect::<Vec<_>>();
    let valid_plugins = all_plugins
        .iter()
        .filter(|name| plugins_dir.join(format!("{name}.yazi")).exists())
        .cloned()
        .collect::<Vec<_>>();

    let requires = valid_plugins
        .iter()
        .map(|name| {
            if core_plugins.contains(name) {
                format!("-- Core plugin (always loaded)\nrequire(\"{name}\"):setup()")
            } else if name == "starship" {
                let starship_config_path = output_dir.join("yazelix_starship.toml");
                format!(
                    "-- User plugin (from settings.jsonc)\nrequire(\"starship\"):setup({{\n    config_file = \"{}\"\n}})",
                    starship_config_path.to_string_lossy()
                )
            } else {
                let local_name = name.replace('-', "_");
                format!(
                    "-- User plugin (from settings.jsonc)\nlocal _{local_name} = require(\"{name}\")\nif type(_{local_name}.setup) == \"function\" then _{local_name}:setup() end"
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let header = generated_header(
        "--",
        "AUTO-GENERATED YAZI INIT.LUA",
        &[
            "",
            "To customize plugins, edit:",
            "  ~/.config/yazelix/settings.jsonc",
            "  [yazi] plugins = [...]",
            "",
            "For custom Lua code, create:",
            "  ~/.config/yazelix/yazi_init.lua",
            "",
        ],
    );
    let mut final_content =
        render_runtime_root_placeholders(&format!("{header}{requires}\n"), runtime_dir);
    let mut user_init_appended = false;
    if let Some(user_init) = user_init_lua {
        let user_section = [
            "",
            "-- ========================================",
            "-- USER CUSTOM CODE",
            "-- ========================================",
            "-- From: ~/.config/yazelix/yazi_init.lua",
            "-- ========================================",
            "",
            user_init,
        ]
        .join("\n");
        final_content = render_runtime_root_placeholders(
            &format!("{final_content}{user_section}"),
            runtime_dir,
        );
        user_init_appended = true;
    }

    let target = output_dir.join("init.lua");
    let changed = write_text_atomic_if_changed(&target, &final_content)?;
    Ok((
        YaziManagedFileStatus {
            path: target.to_string_lossy().to_string(),
            changed,
        },
        missing_plugins,
        user_init_appended,
    ))
}

fn generated_header(comment: &str, title: &str, body: &[&str]) -> String {
    let border = format!("{comment} ========================================");
    let mut lines = vec![
        border.clone(),
        format!("{comment} {title}"),
        border.clone(),
        format!("{comment} This file is automatically generated by Yazelix."),
        format!("{comment} Do not edit directly - changes will be lost!"),
    ];
    lines.extend(body.iter().map(|line| {
        if line.is_empty() {
            comment.to_string()
        } else {
            format!("{comment} {line}")
        }
    }));
    lines.push(border);
    lines.push(String::new());
    lines.join("\n")
}

pub(super) fn bundled_yazi_assets_missing(
    source_dir: &Path,
    output_dir: &Path,
) -> Result<bool, CoreError> {
    let source_plugins = source_dir.join("plugins");
    let output_plugins = output_dir.join("plugins");
    let source_flavors = source_dir.join("flavors");
    let output_flavors = output_dir.join("flavors");
    let source_starship = source_dir.join("yazelix_starship.toml");
    let output_starship = output_dir.join("yazelix_starship.toml");

    Ok(
        asset_tree_missing_targets(&source_plugins, &output_plugins)?
            || asset_tree_missing_targets(&source_flavors, &output_flavors)?
            || (source_starship.exists() && !output_starship.exists()),
    )
}

pub(super) fn asset_tree_missing_targets(
    source_root: &Path,
    target_root: &Path,
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
            let file_type = entry.file_type().map_err(|source| {
                CoreError::io(
                    "inspect_yazi_asset_source_entry",
                    "Could not inspect a bundled Yazi asset entry",
                    "Reinstall Yazelix so the runtime includes readable Yazi assets.",
                    path.to_string_lossy(),
                    source,
                )
            })?;
            if path == source_root && !file_type.is_dir() {
                continue;
            }
            let source_path = entry.path();
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
            if !target_path.exists() {
                return Ok(true);
            }
            if file_type.is_dir() {
                stack.push(source_path);
            }
        }
    }

    Ok(false)
}

fn sync_bundled_yazi_assets(
    source_dir: &Path,
    output_dir: &Path,
    runtime_dir: &Path,
) -> Result<(), CoreError> {
    sync_named_child_directories(
        &source_dir.join("plugins"),
        &output_dir.join("plugins"),
        runtime_dir,
    )?;
    sync_named_child_directories(
        &source_dir.join("flavors"),
        &output_dir.join("flavors"),
        runtime_dir,
    )?;
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
            "Could not inspect the bundled Yazi asset directory",
            "Reinstall Yazelix so the runtime includes the bundled Yazi assets.",
            source_root.to_string_lossy(),
            source,
        )
    })? {
        let entry = entry.map_err(|source| {
            CoreError::io(
                "read_yazi_asset_entry",
                "Could not inspect a bundled Yazi asset entry",
                "Reinstall Yazelix so the runtime includes the bundled Yazi assets.",
                source_root.to_string_lossy(),
                source,
            )
        })?;
        let source_path = entry.path();
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
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
    let file_type = fs::symlink_metadata(source).map_err(|source_err| {
        CoreError::io(
            "inspect_yazi_asset_source",
            "Could not inspect a bundled Yazi asset path",
            "Reinstall Yazelix so the runtime includes a readable Yazi asset tree.",
            source.to_string_lossy(),
            source_err,
        )
    })?;

    if file_type.is_dir() {
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
                "Could not read a bundled Yazi asset directory",
                "Reinstall Yazelix so the runtime includes a readable Yazi asset tree.",
                source.to_string_lossy(),
                source_err,
            )
        })? {
            let entry = entry.map_err(|source_err| {
                CoreError::io(
                    "read_yazi_asset_dir_entry",
                    "Could not read a bundled Yazi asset entry",
                    "Reinstall Yazelix so the runtime includes a readable Yazi asset tree.",
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
            "Could not read a bundled Yazi asset file",
            "Reinstall Yazelix so the runtime includes a readable Yazi asset tree.",
            source.to_string_lossy(),
            source_err,
        )
    })?;
    let rendered = match String::from_utf8(bytes) {
        Ok(text) => render_runtime_root_placeholders(&text, runtime_dir).into_bytes(),
        Err(non_utf8) => non_utf8.into_bytes(),
    };
    write_bytes_atomic(target, &rendered)?;
    Ok(())
}

pub(super) fn preserve_yazelix_edit_opener(base: &toml::Table, merged: &mut toml::Table) {
    let Some(base_opener) = base.get("opener").and_then(TomlValue::as_table) else {
        return;
    };
    let Some(yazelix_edit) = base_opener.get("edit").cloned() else {
        return;
    };

    if !merged.contains_key("opener") {
        merged.insert("opener".into(), TomlValue::Table(toml::Table::new()));
    }
    let opener = merged
        .get_mut("opener")
        .and_then(TomlValue::as_table_mut)
        .expect("opener inserted as a table");
    opener.insert("edit".into(), yazelix_edit);
}

fn upsert_nested_string(root: &mut toml::Table, path: &[&str], leaf: &str, value: &str) {
    let mut current = root;
    for segment in path {
        if !current.contains_key(*segment) {
            current.insert((*segment).into(), TomlValue::Table(toml::Table::new()));
        }
        current = current
            .get_mut(*segment)
            .and_then(TomlValue::as_table_mut)
            .expect("path inserted as nested tables");
    }
    current.insert(leaf.into(), TomlValue::String(value.to_string()));
}

pub(super) fn merge_yazi_toml_config(
    base_config: toml::Table,
    user_config: toml::Table,
) -> toml::Table {
    let mut merged = TomlValue::Table(base_config);
    deep_merge_toml(&mut merged, &TomlValue::Table(user_config));
    merged.as_table().cloned().unwrap_or_default()
}

fn deep_merge_toml(base: &mut TomlValue, user: &TomlValue) {
    match (base, user) {
        (TomlValue::Table(base_table), TomlValue::Table(user_table)) => {
            for (key, user_value) in user_table {
                match base_table.get_mut(key) {
                    Some(base_value) => deep_merge_toml(base_value, user_value),
                    None => {
                        base_table.insert(key.clone(), user_value.clone());
                    }
                }
            }
        }
        (TomlValue::Array(base_array), TomlValue::Array(user_array)) => {
            base_array.extend(user_array.iter().cloned());
        }
        (base_value, user_value) => {
            *base_value = user_value.clone();
        }
    }
}

pub(super) fn merge_yazi_keymap(base_keymap: toml::Table, user_keymap: toml::Table) -> toml::Table {
    let mut merged = base_keymap;
    for (section, user_value) in user_keymap {
        let TomlValue::Table(user_section) = user_value else {
            merged.insert(section, user_value);
            continue;
        };

        let Some(base_section) = merged.get_mut(&section).and_then(TomlValue::as_table_mut) else {
            merged.insert(section, TomlValue::Table(user_section));
            continue;
        };

        let base_subsections = base_section.keys().cloned().collect::<Vec<_>>();
        for subsection in &base_subsections {
            let Some(user_value) = user_section.get(subsection) else {
                continue;
            };
            let Some(base_array) = base_section
                .get_mut(subsection)
                .and_then(TomlValue::as_array_mut)
            else {
                continue;
            };
            match user_value {
                TomlValue::Array(user_array) => base_array.extend(user_array.iter().cloned()),
                other => base_array.push(other.clone()),
            }
        }
        for (subsection, user_value) in user_section {
            if !base_subsections.contains(&subsection) {
                base_section.insert(subsection.clone(), user_value.clone());
            }
        }
    }
    merged
}

fn read_required_toml_table(
    path: &Path,
    code: &str,
    message: &str,
    remediation: &str,
) -> Result<toml::Table, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(code, message, remediation, path.to_string_lossy(), source)
    })?;
    toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::toml(
            "invalid_toml",
            message,
            remediation,
            path.to_string_lossy(),
            source,
        )
    })
}

fn toml_to_string_pretty(value: &TomlValue) -> Result<String, CoreError> {
    toml::to_string_pretty(value).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_yazi_toml",
            format!("Could not serialize generated Yazi content: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })
}

pub(super) fn render_runtime_root_placeholders(content: &str, runtime_dir: &Path) -> String {
    content.replace(
        RUNTIME_DIR_PLACEHOLDER,
        runtime_dir.to_string_lossy().as_ref(),
    )
}

fn write_text_atomic_if_changed(path: &Path, content: &str) -> Result<bool, CoreError> {
    if fs::read_to_string(path).ok().as_deref() == Some(content) {
        return Ok(false);
    }
    write_bytes_atomic(path, content.as_bytes())?;
    Ok(true)
}

fn write_bytes_atomic(path: &Path, content: &[u8]) -> Result<(), CoreError> {
    let parent = path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "invalid_yazi_output_path",
            "Generated Yazi output path has no parent directory",
            "Report this as a Yazelix internal error.",
            json!({ "path": path.to_string_lossy() }),
        )
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        CoreError::io(
            "create_yazi_output_parent",
            "Could not create the parent directory for generated Yazi output",
            "Check permissions for the Yazelix state directory and retry.",
            parent.to_string_lossy(),
            source,
        )
    })?;
    let temporary_path = path.with_file_name(format!(
        ".{}.yazelix-tmp-{}-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("yazi"),
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    fs::write(&temporary_path, content).map_err(|source| {
        CoreError::io(
            "write_yazi_output_temp",
            "Could not write temporary generated Yazi output",
            "Check permissions for the Yazelix state directory and retry.",
            temporary_path.to_string_lossy(),
            source,
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|source| {
        CoreError::io(
            "rename_yazi_output_temp",
            "Could not replace generated Yazi output",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })?;
    set_writable(path, false)?;
    Ok(())
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
