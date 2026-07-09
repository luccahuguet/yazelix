use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryOptions {
    pub yazelix_root: PathBuf,
    pub envctl_root: Option<PathBuf>,
    pub meta_root: Option<PathBuf>,
    pub real_home: Option<PathBuf>,
    pub nix_store: Option<PathBuf>,
    pub system_service_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryRow {
    pub target_id: String,
    pub absolute_path: PathBuf,
    pub normalized_logical_path: String,
    pub owner: String,
    pub source_of_truth_class: String,
    pub exists: bool,
    pub file_kind: String,
    pub parser_hint: String,
    pub mutability: String,
    pub reproduction_policy: String,
    pub safety_policy: String,
    pub import_mode: String,
}

pub fn collect_yazelix_file_inventory(
    options: &InventoryOptions,
) -> Result<Vec<InventoryRow>, String> {
    let mut collector = InventoryCollector::default();

    collector.walk_existing_root(&options.yazelix_root, RootKind::YazelixRepo)?;

    if let Some(envctl_root) = &options.envctl_root {
        collector.walk_existing_root(envctl_root, RootKind::EnvctlRepo)?;
    }

    if let Some(meta_root) = &options.meta_root {
        collector.scan_local_surfaces(&meta_root.join(".local"), RootKind::MetaLocal)?;
    }

    if let Some(real_home) = &options.real_home {
        collector.scan_local_surfaces(&real_home.join(".local"), RootKind::RealHomeLocal)?;
        collector
            .walk_existing_root(&real_home.join(".config/yazelix"), RootKind::RealHomeConfig)?;
        collector.walk_existing_root(
            &real_home.join(".config/yazelix_cursors"),
            RootKind::RealHomeConfig,
        )?;
        collector.walk_existing_root(&real_home.join(".cache/yazelix"), RootKind::RealHomeCache)?;
        collector.walk_existing_root(
            &real_home.join(".local/state/yazelix"),
            RootKind::RealHomeState,
        )?;
        collector.walk_matching_local_names(
            &real_home.join(".config"),
            &real_home.join(".config/systemd/user"),
            RootKind::UserService,
        )?;
    }

    if let Some(nix_store) = &options.nix_store {
        collector.scan_nix_store(nix_store)?;
    }

    for system_root in &options.system_service_roots {
        collector.walk_matching_local_names(system_root, system_root, RootKind::SystemService)?;
    }

    let mut rows = collector.rows;
    rows.sort_by(|left, right| {
        left.normalized_logical_path
            .cmp(&right.normalized_logical_path)
            .then_with(|| left.absolute_path.cmp(&right.absolute_path))
    });
    Ok(rows)
}

pub fn default_inventory_options(repo_root: &Path) -> InventoryOptions {
    let meta_root = repo_root
        .parent()
        .and_then(|parent| {
            if parent.file_name().and_then(|name| name.to_str()) == Some("src") {
                parent.parent()
            } else {
                None
            }
        })
        .map(Path::to_path_buf);
    let envctl_root = repo_root
        .parent()
        .map(|parent| parent.join("envctl"))
        .filter(|path| path.exists());
    let real_home = std::env::var_os("HOME").map(PathBuf::from);
    let nix_store = Some(PathBuf::from("/nix/store"));

    InventoryOptions {
        yazelix_root: repo_root.to_path_buf(),
        envctl_root,
        meta_root,
        real_home,
        nix_store,
        system_service_roots: vec![
            PathBuf::from("/etc/systemd/system"),
            PathBuf::from("/etc/systemd/user"),
            PathBuf::from("/usr/lib/systemd/system"),
            PathBuf::from("/usr/share/applications"),
        ],
    }
}

pub fn write_inventory_json(options: &InventoryOptions, out_path: &Path) -> Result<usize, String> {
    let rows = collect_yazelix_file_inventory(options)?;
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(&rows)
        .map_err(|error| format!("failed to serialize inventory rows: {error}"))?;
    fs::write(out_path, bytes)
        .map_err(|error| format!("failed to write {}: {error}", out_path.display()))?;
    Ok(rows.len())
}

#[derive(Debug, Clone, Copy)]
enum RootKind {
    YazelixRepo,
    EnvctlRepo,
    MetaLocal,
    RealHomeLocal,
    RealHomeConfig,
    RealHomeCache,
    RealHomeState,
    UserService,
    SystemService,
    NixStore,
}

#[derive(Default)]
struct InventoryCollector {
    seen: BTreeSet<PathBuf>,
    rows: Vec<InventoryRow>,
}

impl InventoryCollector {
    fn walk_existing_root(&mut self, root: &Path, kind: RootKind) -> Result<(), String> {
        if !root.exists() {
            return Ok(());
        }
        self.walk(root, root, kind)
    }

    fn walk(&mut self, root: &Path, path: &Path, kind: RootKind) -> Result<(), String> {
        if ignored_path(path) {
            return Ok(());
        }

        let metadata = fs::symlink_metadata(path)
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;

        if metadata.is_file() || metadata.file_type().is_symlink() {
            self.push_row(root, path, kind, &metadata)?;
            return Ok(());
        }

        if !metadata.is_dir() {
            self.push_row(root, path, kind, &metadata)?;
            return Ok(());
        }

        for entry in fs::read_dir(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?
        {
            let entry = entry.map_err(|error| {
                format!("failed to read entry under {}: {error}", path.display())
            })?;
            self.walk(root, &entry.path(), kind)?;
        }

        Ok(())
    }

    fn scan_nix_store(&mut self, nix_store: &Path) -> Result<(), String> {
        if !nix_store.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(nix_store)
            .map_err(|error| format!("failed to read {}: {error}", nix_store.display()))?
        {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to read entry under {}: {error}",
                    nix_store.display()
                )
            })?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !is_yazelix_nix_store_name(name) {
                continue;
            }
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
            self.push_row(nix_store, &path, RootKind::NixStore, &metadata)?;
        }

        Ok(())
    }

    fn scan_local_surfaces(&mut self, local_root: &Path, kind: RootKind) -> Result<(), String> {
        if !local_root.exists() {
            return Ok(());
        }

        self.walk_existing_root(&local_root.join("share/yazelix"), kind)?;
        self.walk_matching_local_names(local_root, &local_root.join("share/icons/hicolor"), kind)?;

        let applications = local_root.join("share/applications");
        if applications.exists() {
            for entry in fs::read_dir(&applications)
                .map_err(|error| format!("failed to read {}: {error}", applications.display()))?
            {
                let entry = entry.map_err(|error| {
                    format!(
                        "failed to read entry under {}: {error}",
                        applications.display()
                    )
                })?;
                let path = entry.path();
                let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if is_yazelix_local_name(name) {
                    let metadata = fs::symlink_metadata(&path).map_err(|error| {
                        format!("failed to inspect {}: {error}", path.display())
                    })?;
                    self.push_row(local_root, &path, kind, &metadata)?;
                }
            }
        }

        let bin = local_root.join("bin");
        if bin.exists() {
            for entry in fs::read_dir(&bin)
                .map_err(|error| format!("failed to read {}: {error}", bin.display()))?
            {
                let entry = entry.map_err(|error| {
                    format!("failed to read entry under {}: {error}", bin.display())
                })?;
                let path = entry.path();
                let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if is_yazelix_local_name(name) {
                    let metadata = fs::symlink_metadata(&path).map_err(|error| {
                        format!("failed to inspect {}: {error}", path.display())
                    })?;
                    self.push_row(local_root, &path, kind, &metadata)?;
                }
            }
        }

        Ok(())
    }

    fn walk_matching_local_names(
        &mut self,
        local_root: &Path,
        path: &Path,
        kind: RootKind,
    ) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if metadata.is_file() || metadata.file_type().is_symlink() {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                return Ok(());
            };
            if is_yazelix_local_name(name) {
                self.push_row(local_root, path, kind, &metadata)?;
            }
            return Ok(());
        }
        if !metadata.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?
        {
            let entry = entry.map_err(|error| {
                format!("failed to read entry under {}: {error}", path.display())
            })?;
            self.walk_matching_local_names(local_root, &entry.path(), kind)?;
        }
        Ok(())
    }

    fn push_row(
        &mut self,
        root: &Path,
        path: &Path,
        kind: RootKind,
        metadata: &fs::Metadata,
    ) -> Result<(), String> {
        let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if !self.seen.insert(absolute_path.clone()) {
            return Ok(());
        }

        let relative_path = path.strip_prefix(root).unwrap_or(path);
        let class = classify(kind, relative_path, path);
        let parser_hint = parser_hint(path, metadata);
        let file_kind = file_kind(metadata);

        self.rows.push(InventoryRow {
            target_id: stable_target_id(kind, relative_path, path),
            absolute_path,
            normalized_logical_path: normalized_logical_path(kind, relative_path, path),
            owner: class.owner.to_string(),
            source_of_truth_class: class.source_of_truth_class.to_string(),
            exists: true,
            file_kind: file_kind.to_string(),
            parser_hint: parser_hint.to_string(),
            mutability: class.mutability.to_string(),
            reproduction_policy: class.reproduction_policy.to_string(),
            safety_policy: class.safety_policy.to_string(),
            import_mode: class.import_mode.to_string(),
        });

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Classification {
    owner: &'static str,
    source_of_truth_class: &'static str,
    mutability: &'static str,
    reproduction_policy: &'static str,
    safety_policy: &'static str,
    import_mode: &'static str,
}

fn classify(kind: RootKind, relative_path: &Path, absolute_path: &Path) -> Classification {
    let rel = relative_path.to_string_lossy();
    match kind {
        RootKind::YazelixRepo => Classification {
            owner: "yazelix",
            source_of_truth_class: "repo_source",
            mutability: "source_controlled",
            reproduction_policy: "git_checkout",
            safety_policy: "source_content_import_allowed",
            import_mode: "content_blob",
        },
        RootKind::EnvctlRepo => Classification {
            owner: "envctl",
            source_of_truth_class: "envctl_control_surface",
            mutability: "source_controlled",
            reproduction_policy: "git_checkout",
            safety_policy: "source_content_import_allowed",
            import_mode: "content_blob",
        },
        RootKind::MetaLocal => local_classification("meta_local_generated", "meta_root", &rel),
        RootKind::RealHomeLocal => {
            let abs = absolute_path.to_string_lossy();
            if abs.contains("/.local/share/applications/") {
                Classification {
                    owner: "desktop",
                    source_of_truth_class: "real_home_desktop_entry",
                    mutability: "user_home_mutable",
                    reproduction_policy: "desktop_entry_bridge_or_manual_install",
                    safety_policy: "real_home_metadata_first",
                    import_mode: "metadata_only",
                }
            } else {
                local_classification("real_home_runtime_state", "real_home", &rel)
            }
        }
        RootKind::RealHomeConfig => Classification {
            owner: "user",
            source_of_truth_class: "real_home_user_config",
            mutability: "user_home_mutable",
            reproduction_policy: "user_config_source_or_import",
            safety_policy: "real_home_metadata_first",
            import_mode: "metadata_only",
        },
        RootKind::RealHomeCache => Classification {
            owner: "yazelix",
            source_of_truth_class: "real_home_cache",
            mutability: "runtime_mutable",
            reproduction_policy: "cache_regeneration_or_observed_only",
            safety_policy: "cache_no_content_import",
            import_mode: "metadata_only",
        },
        RootKind::RealHomeState => Classification {
            owner: "yazelix",
            source_of_truth_class: "real_home_runtime_state",
            mutability: "runtime_mutable",
            reproduction_policy: "observed_runtime_state_only",
            safety_policy: "runtime_state_no_content_import",
            import_mode: "metadata_only",
        },
        RootKind::UserService => Classification {
            owner: "user_service_manager",
            source_of_truth_class: "user_service_target",
            mutability: "user_home_mutable",
            reproduction_policy: "service_bridge_or_manual_install",
            safety_policy: "service_metadata_first",
            import_mode: "metadata_only",
        },
        RootKind::SystemService => Classification {
            owner: "system_service_manager",
            source_of_truth_class: "system_service_target",
            mutability: "system_mutable",
            reproduction_policy: "package_or_system_configuration",
            safety_policy: "system_metadata_only",
            import_mode: "metadata_only",
        },
        RootKind::NixStore => Classification {
            owner: "nix",
            source_of_truth_class: "nix_store_package_output",
            mutability: "immutable",
            reproduction_policy: "nix_derivation_or_flake_output",
            safety_policy: "nix_store_metadata_first",
            import_mode: "metadata_only",
        },
    }
}

fn local_classification(
    source_of_truth_class: &'static str,
    owner: &'static str,
    relative_path: &str,
) -> Classification {
    let generated_content = relative_path.contains("share/yazelix/configs/")
        || relative_path.contains("share/yazelix/generated/")
        || relative_path.contains("share/yazelix/initializers/")
        || relative_path.starts_with("configs/")
        || relative_path.starts_with("generated/")
        || relative_path.starts_with("initializers/");

    if generated_content {
        Classification {
            owner: "yazelix",
            source_of_truth_class,
            mutability: "generated",
            reproduction_policy: "regenerate_from_yazelix_runtime",
            safety_policy: "generated_content_import_allowed",
            import_mode: "content_blob",
        }
    } else {
        Classification {
            owner,
            source_of_truth_class,
            mutability: "runtime_mutable",
            reproduction_policy: "observed_runtime_state_only",
            safety_policy: "runtime_state_no_content_import",
            import_mode: "metadata_only",
        }
    }
}

fn ignored_path(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        matches!(
            name.as_ref(),
            ".git" | "target" | ".direnv" | ".devenv" | "node_modules"
        )
    })
}

fn is_yazelix_nix_store_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("yazelix") || lower.contains("yzx") || lower.starts_with("mars")
}

fn is_yazelix_local_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("yazelix") || lower.contains("yzx") || lower.contains("mars")
}

fn file_kind(metadata: &fs::Metadata) -> &'static str {
    if metadata.file_type().is_symlink() {
        "symlink"
    } else if metadata.is_file() {
        "regular_file"
    } else if metadata.is_dir() {
        "directory"
    } else {
        "special"
    }
}

fn parser_hint(path: &Path, metadata: &fs::Metadata) -> &'static str {
    if metadata.is_dir() {
        return "directory";
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if name.ends_with(".desktop") {
        return "desktop_entry";
    }
    if name.ends_with(".service") {
        return "systemd_unit";
    }
    if name.ends_with(".jsonc") {
        return "jsonc";
    }

    match path.extension().and_then(|extension| extension.to_str()) {
        Some("json") => "json",
        Some("toml") => "toml",
        Some("kdl") => "kdl",
        Some("nu") => "nu",
        Some("lua") => "lua",
        Some("yaml" | "yml") => "yaml",
        Some("md") => "markdown",
        Some("nix") => "nix",
        Some("sh" | "bash") => "shell",
        Some("log") => "log",
        Some("jsonl") => "jsonl",
        Some("scm") => "scheme",
        Some("txt") => "plain_text",
        Some(_) => "opaque",
        None => "plain_or_binary",
    }
}

fn stable_target_id(kind: RootKind, relative_path: &Path, absolute_path: &Path) -> String {
    normalized_logical_path(kind, relative_path, absolute_path)
        .replace(['/', '.', ':'], "_")
        .trim_matches('_')
        .to_string()
}

fn normalized_logical_path(kind: RootKind, relative_path: &Path, absolute_path: &Path) -> String {
    let prefix = match kind {
        RootKind::YazelixRepo => "yazelix_repo",
        RootKind::EnvctlRepo => "envctl_repo",
        RootKind::MetaLocal => "meta_local",
        RootKind::RealHomeLocal => "real_home_local",
        RootKind::RealHomeConfig => "real_home_config",
        RootKind::RealHomeCache => "real_home_cache",
        RootKind::RealHomeState => "real_home_state",
        RootKind::UserService => "user_service",
        RootKind::SystemService => "system_service",
        RootKind::NixStore => "nix_store",
    };
    let raw = match kind {
        RootKind::NixStore => absolute_path
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| relative_path.to_path_buf()),
        _ => relative_path.to_path_buf(),
    };
    format!("{}:{}", prefix, raw.to_string_lossy())
}
