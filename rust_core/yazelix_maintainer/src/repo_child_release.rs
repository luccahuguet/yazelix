//! Maintainer validation for coupled child-repo release transactions.

use crate::repo_validation::ValidationReport;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChildInputLock {
    node: String,
    owner: String,
    repo: String,
    rev: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FirstPartyCargoGitDependency {
    package_key: String,
    owner: String,
    repo: String,
    rev: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ZellijPluginWasmPackageContract {
    package_attr: &'static str,
    system: &'static str,
    plugin_name: &'static str,
    wasm_path: &'static str,
}

const ZELLIJ_PLUGIN_WASM_PACKAGE_CONTRACTS: &[ZellijPluginWasmPackageContract] = &[
    ZellijPluginWasmPackageContract {
        package_attr: "yazelix_zellij_pane_orchestrator",
        system: "aarch64-darwin",
        plugin_name: "yazelix-zellij-pane-orchestrator",
        wasm_path: "share/yazelix_zellij_pane_orchestrator/yazelix_pane_orchestrator.wasm",
    },
    ZellijPluginWasmPackageContract {
        package_attr: "yazelix_zellij_popup",
        system: "aarch64-darwin",
        plugin_name: "yazelix-zellij-popup",
        wasm_path: "share/yazelix_zellij_popup/yzpp.wasm",
    },
];

const ZELLIJ_PLUGIN_WASM_METADATA_FIELDS: &[&str] = &[
    "schemaVersion",
    "pluginName",
    "packageAttr",
    "wasmPath",
    "wasmTarget",
    "cargoAuditableDisabled",
    "cargoBuildHookDisabled",
    "preBuildPreservesNixRustToolchain",
    "wasmTargetRustcEnvPinned",
    "cargoBuildSerialized",
    "installCheckVerifiesWasm",
];

const CURSOR_PACKAGE_METADATA_FIELDS: &[&str] = &[
    "schemaVersion",
    "packageName",
    "shareRoot",
    "shaderRoot",
    "generatedEffectRoot",
    "requiredTargets",
    "requiredShaderFiles",
    "forbiddenShaderFiles",
];

const CURSOR_PACKAGE_REQUIRED_TARGETS: &[&str] = &[
    "ghostty",
    "yzxterm",
    "rio",
    "ratty",
    "protocol_cursor_positions",
];

const CURSOR_PACKAGE_REQUIRED_SHADER_FILES: &[&str] = &[
    "cursor_trail_common.glsl",
    "cursor_trail_reef.glsl",
    "upstream_effects/ripple_rectangle_cursor.glsl",
    "generated_effects/tail.glsl",
];

pub fn validate_child_release_transaction(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let lock_path = repo_root.join("flake.lock");
    let raw = fs::read_to_string(&lock_path)
        .map_err(|error| format!("Failed to read {}: {error}", lock_path.display()))?;
    let inputs = locked_child_inputs(&raw)?;
    if inputs.is_empty() {
        report
            .errors
            .push("flake.lock contains no first-party Yazelix child inputs".to_string());
        return Ok(report);
    }

    for input in inputs {
        if let Some(warning) = local_child_checkout_warning(repo_root, &input)? {
            report.warnings.push(warning);
        }
        if !remote_rev_is_fetchable(&input)? {
            report.errors.push(format!(
                "Child input `{}` pins unpublished or unreachable commit {} for {}/{}. Push the child repo first, update flake.lock to the published revision, then run no-overrides validation.",
                input.node, input.rev, input.owner, input.repo
            ));
        }
    }

    report
        .errors
        .extend(validate_cargo_git_output_hashes(repo_root)?);
    report
        .errors
        .extend(validate_zellij_plugin_wasm_package_contracts(repo_root)?);
    report
        .errors
        .extend(validate_cursor_package_contracts(repo_root)?);
    report
        .errors
        .extend(validate_main_cursor_shader_tree_deleted(repo_root));

    Ok(report)
}

fn locked_child_inputs(raw_lock: &str) -> Result<Vec<ChildInputLock>, String> {
    let parsed: JsonValue = serde_json::from_str(raw_lock)
        .map_err(|error| format!("Invalid flake.lock JSON: {error}"))?;
    let nodes = parsed
        .get("nodes")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| "flake.lock is missing object `nodes`".to_string())?;
    let mut inputs = Vec::new();
    for (node, data) in nodes {
        let Some(locked) = data.get("locked").and_then(JsonValue::as_object) else {
            continue;
        };
        if locked.get("type").and_then(JsonValue::as_str) != Some("github") {
            continue;
        }
        let owner = locked
            .get("owner")
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        let repo = locked.get("repo").and_then(JsonValue::as_str).unwrap_or("");
        let rev = locked.get("rev").and_then(JsonValue::as_str).unwrap_or("");
        if owner != "luccahuguet" || !repo.starts_with("yazelix-") || rev.is_empty() {
            continue;
        }
        inputs.push(ChildInputLock {
            node: node.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            rev: rev.to_string(),
        });
    }
    inputs.sort_by(|left, right| left.node.cmp(&right.node));
    Ok(inputs)
}

fn local_child_checkout_warning(
    repo_root: &Path,
    input: &ChildInputLock,
) -> Result<Option<String>, String> {
    let Some(parent) = repo_root.parent() else {
        return Ok(None);
    };
    let checkout = parent.join(&input.repo);
    if !checkout.join(".git").exists() {
        return Ok(None);
    }

    let status = Command::new("git")
        .args([
            "-C",
            checkout.to_string_lossy().as_ref(),
            "status",
            "--short",
        ])
        .output()
        .map_err(|error| {
            format!(
                "Failed to inspect local child checkout {}: {error}",
                checkout.display()
            )
        })?;
    if !status.status.success() {
        return Err(format!(
            "Failed to inspect local child checkout {}\n{}",
            checkout.display(),
            String::from_utf8_lossy(&status.stderr).trim()
        ));
    }
    let dirty = String::from_utf8_lossy(&status.stdout).trim().to_string();
    if dirty.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "Local child checkout {} has uncommitted changes; finish or stash them before running a coupled release transaction.",
        checkout.display()
    )))
}

fn remote_rev_is_fetchable(input: &ChildInputLock) -> Result<bool, String> {
    let url = format!("https://github.com/{}/{}.git", input.owner, input.repo);
    let probe_dir = remote_rev_probe_dir(input)?;
    let init = Command::new("git")
        .args(["init", "--bare"])
        .arg(&probe_dir)
        .output()
        .map_err(|error| format!("Failed to initialize {}: {error}", probe_dir.display()))?;
    if !init.status.success() {
        return Err(format!(
            "Failed to initialize {}\n{}",
            probe_dir.display(),
            String::from_utf8_lossy(&init.stderr).trim()
        ));
    }

    let fetch = Command::new("git")
        .arg("-C")
        .arg(&probe_dir)
        .args(["fetch", "--depth=1", &url, &input.rev])
        .output()
        .map_err(|error| {
            format!(
                "Failed to run `git fetch --depth=1 {url} {}`: {error}",
                input.rev
            )
        });
    let cleanup = fs::remove_dir_all(&probe_dir)
        .map_err(|error| format!("Failed to remove {}: {error}", probe_dir.display()));

    let fetch = fetch?;
    cleanup?;
    if fetch.status.success() {
        return Ok(true);
    }

    let stderr = String::from_utf8_lossy(&fetch.stderr);
    if fetch_failure_means_missing_revision(&stderr) {
        return Ok(false);
    }

    Err(format!(
        "Failed to fetch locked revision {} from {url}\n{}",
        input.rev,
        stderr.trim()
    ))
}

fn remote_rev_probe_dir(input: &ChildInputLock) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("System clock is before UNIX_EPOCH: {error}"))?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "yazelix_child_release_probe_{}_{}_{}",
        std::process::id(),
        stamp,
        input.repo
    )))
}

fn fetch_failure_means_missing_revision(stderr: &str) -> bool {
    stderr.contains("not our ref")
        || stderr.contains("couldn't find remote ref")
        || stderr.contains("Server does not allow request for unadvertised object")
}

fn validate_cargo_git_output_hashes(repo_root: &Path) -> Result<Vec<String>, String> {
    let cargo_lock_path = repo_root.join("rust_core/Cargo.lock");
    let helper_path = repo_root.join("packaging/rust_core_helper.nix");
    let raw_cargo_lock = fs::read_to_string(&cargo_lock_path)
        .map_err(|error| format!("Failed to read {}: {error}", cargo_lock_path.display()))?;
    let raw_helper = fs::read_to_string(&helper_path)
        .map_err(|error| format!("Failed to read {}: {error}", helper_path.display()))?;

    validate_cargo_git_output_hashes_with(&raw_cargo_lock, &raw_helper, prefetch_source_hash)
}

fn validate_cargo_git_output_hashes_with(
    raw_cargo_lock: &str,
    raw_helper: &str,
    fetch_hash: impl Fn(&FirstPartyCargoGitDependency) -> Result<String, String>,
) -> Result<Vec<String>, String> {
    let dependencies = first_party_cargo_git_dependencies(raw_cargo_lock)?;
    let output_hashes = rust_core_output_hashes(raw_helper);
    let mut errors = Vec::new();

    for dependency in dependencies {
        let expected_hash = fetch_hash(&dependency)?;
        match output_hashes.get(&dependency.package_key) {
            Some(actual_hash) if actual_hash == &expected_hash => {}
            Some(actual_hash) => errors.push(format!(
                "Stale cargoLock.outputHashes entry for `{}` in packaging/rust_core_helper.nix: Cargo.lock pins {}/{} at {}, expected {}, found {}. Refresh it with `nix flake prefetch --json github:{}/{}/{}`.",
                dependency.package_key,
                dependency.owner,
                dependency.repo,
                dependency.rev,
                expected_hash,
                actual_hash,
                dependency.owner,
                dependency.repo,
                dependency.rev
            )),
            None => errors.push(format!(
                "Missing cargoLock.outputHashes entry for `{}` in packaging/rust_core_helper.nix: Cargo.lock pins {}/{} at {}, expected {}. Add the hash from `nix flake prefetch --json github:{}/{}/{}`.",
                dependency.package_key,
                dependency.owner,
                dependency.repo,
                dependency.rev,
                expected_hash,
                dependency.owner,
                dependency.repo,
                dependency.rev
            )),
        }
    }

    Ok(errors)
}

fn first_party_cargo_git_dependencies(
    raw_cargo_lock: &str,
) -> Result<Vec<FirstPartyCargoGitDependency>, String> {
    let parsed: toml::Value = toml::from_str(raw_cargo_lock)
        .map_err(|error| format!("Invalid rust_core/Cargo.lock TOML: {error}"))?;
    let packages = parsed
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "rust_core/Cargo.lock is missing array `package`".to_string())?;
    let mut dependencies = Vec::new();

    for package in packages {
        let Some(source) = package.get("source").and_then(toml::Value::as_str) else {
            continue;
        };
        let Some((owner, repo, rev)) = parse_first_party_github_source(source) else {
            continue;
        };
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| "Cargo.lock package is missing `name`".to_string())?;
        let version = package
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| format!("Cargo.lock package `{name}` is missing `version`"))?;
        dependencies.push(FirstPartyCargoGitDependency {
            package_key: format!("{name}-{version}"),
            owner,
            repo,
            rev,
        });
    }

    dependencies.sort_by(|left, right| left.package_key.cmp(&right.package_key));
    Ok(dependencies)
}

fn parse_first_party_github_source(source: &str) -> Option<(String, String, String)> {
    let rest = source.strip_prefix("git+https://github.com/")?;
    let (owner_repo, query) = rest.split_once("?rev=")?;
    let (owner, repo) = owner_repo.split_once('/')?;
    let rev = query.split_once('#').map_or(query, |(rev, _)| rev);
    if owner != "luccahuguet" || !repo.starts_with("yazelix-") || rev.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string(), rev.to_string()))
}

fn rust_core_output_hashes(raw_helper: &str) -> HashMap<String, String> {
    let mut hashes = HashMap::new();
    for line in raw_helper.lines().map(str::trim) {
        let Some(after_open_quote) = line.strip_prefix('"') else {
            continue;
        };
        let Some((package_key, rest)) = after_open_quote.split_once('"') else {
            continue;
        };
        let Some(after_equals) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let value = after_equals.trim().trim_end_matches(';').trim();
        let Some(after_value_open_quote) = value.strip_prefix('"') else {
            continue;
        };
        let Some((hash, _)) = after_value_open_quote.split_once('"') else {
            continue;
        };
        hashes.insert(package_key.to_string(), hash.to_string());
    }
    hashes
}

fn prefetch_source_hash(dependency: &FirstPartyCargoGitDependency) -> Result<String, String> {
    let flake_ref = format!(
        "github:{}/{}/{}",
        dependency.owner, dependency.repo, dependency.rev
    );
    let output = Command::new("nix")
        .args(["flake", "prefetch", "--json", &flake_ref])
        .output()
        .map_err(|error| {
            format!("Failed to run `nix flake prefetch --json {flake_ref}`: {error}")
        })?;
    if !output.status.success() {
        return Err(format!(
            "Failed to prefetch first-party Cargo dependency source `{}`\n{}",
            flake_ref,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let parsed: JsonValue = serde_json::from_slice(&output.stdout).map_err(|error| {
        format!("Invalid JSON from `nix flake prefetch --json {flake_ref}`: {error}")
    })?;
    parsed
        .get("hash")
        .and_then(JsonValue::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("`nix flake prefetch --json {flake_ref}` did not report `hash`"))
}

fn validate_zellij_plugin_wasm_package_contracts(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    for contract in ZELLIJ_PLUGIN_WASM_PACKAGE_CONTRACTS {
        let metadata =
            package_passthru_metadata(repo_root, contract.system, contract.package_attr)?;
        errors.extend(validate_zellij_plugin_wasm_package_contract_with(
            contract, &metadata,
        )?);
    }
    Ok(errors)
}

fn validate_cursor_package_contracts(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    for system in ["x86_64-linux", "aarch64-darwin"] {
        let metadata = package_passthru_metadata(repo_root, system, "yazelix_cursors")?;
        errors.extend(validate_cursor_package_contract_with(system, &metadata)?);
    }
    Ok(errors)
}

fn validate_cursor_package_contract_with(
    system: &str,
    raw_passthru: &str,
) -> Result<Vec<String>, String> {
    let parsed: JsonValue = serde_json::from_str(raw_passthru)
        .map_err(|error| format!("Invalid package passthru JSON: {error}"))?;
    let passthru = parsed
        .as_object()
        .ok_or_else(|| "Package passthru metadata must be a JSON object".to_string())?;
    let Some(metadata) = passthru
        .get("yazelixCursorPackageContract")
        .and_then(JsonValue::as_object)
    else {
        return Ok(vec![format!(
            "`yazelix_cursors` {system} package passthru must expose `yazelixCursorPackageContract`."
        )]);
    };

    let mut errors = Vec::new();
    for required in CURSOR_PACKAGE_METADATA_FIELDS {
        if !metadata.contains_key(*required) {
            errors.push(format!(
                "`yazelix_cursors` {system} yazelixCursorPackageContract is missing required field `{required}`."
            ));
        }
    }
    for actual in metadata.keys() {
        if !CURSOR_PACKAGE_METADATA_FIELDS.contains(&actual.as_str()) {
            errors.push(format!(
                "`yazelix_cursors` {system} yazelixCursorPackageContract has unsupported field `{actual}`."
            ));
        }
    }
    require_cursor_contract_number(&mut errors, system, metadata, "schemaVersion", 1);
    require_cursor_contract_string(
        &mut errors,
        system,
        metadata,
        "packageName",
        "yazelix-cursors",
    );
    require_cursor_contract_string(
        &mut errors,
        system,
        metadata,
        "shareRoot",
        "share/yazelix/yazelix_cursors",
    );
    require_cursor_contract_string(
        &mut errors,
        system,
        metadata,
        "shaderRoot",
        "share/yazelix/yazelix_cursors/shaders",
    );
    require_cursor_contract_string(
        &mut errors,
        system,
        metadata,
        "generatedEffectRoot",
        "share/yazelix/yazelix_cursors/shaders/generated_effects",
    );
    require_cursor_contract_string_array(
        &mut errors,
        system,
        metadata,
        "requiredTargets",
        CURSOR_PACKAGE_REQUIRED_TARGETS,
    );
    require_cursor_contract_string_array(
        &mut errors,
        system,
        metadata,
        "requiredShaderFiles",
        CURSOR_PACKAGE_REQUIRED_SHADER_FILES,
    );
    require_cursor_contract_string_array(
        &mut errors,
        system,
        metadata,
        "forbiddenShaderFiles",
        &["build_shaders.nu"],
    );
    Ok(errors)
}

fn validate_main_cursor_shader_tree_deleted(repo_root: &Path) -> Vec<String> {
    let path = repo_root
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");
    if path.exists() {
        vec![format!(
            "Main repo must not own Ghostty cursor shader assets at {}; consume `yazelix_cursors` package assets instead.",
            path.display()
        )]
    } else {
        Vec::new()
    }
}

fn package_passthru_metadata(
    repo_root: &Path,
    system: &str,
    package_attr: &str,
) -> Result<String, String> {
    let flake_attr = format!(".#packages.{system}.{package_attr}.passthru");
    let eval = Command::new("nix")
        .current_dir(repo_root)
        .args(["eval", "--json", &flake_attr, "--accept-flake-config"])
        .output()
        .map_err(|error| format!("Failed to run `nix eval --json {flake_attr}`: {error}"))?;
    if !eval.status.success() {
        return Err(format!(
            "Failed to evaluate child package passthru metadata `{flake_attr}`\n{}",
            String::from_utf8_lossy(&eval.stderr).trim()
        ));
    }

    String::from_utf8(eval.stdout)
        .map_err(|error| format!("Invalid UTF-8 from `nix eval --json {flake_attr}`: {error}"))
}

fn require_cursor_contract_number(
    errors: &mut Vec<String>,
    system: &str,
    metadata: &serde_json::Map<String, JsonValue>,
    key: &str,
    expected: u64,
) {
    let actual = metadata.get(key).and_then(JsonValue::as_u64);
    if actual != Some(expected) {
        errors.push(format!(
            "`yazelix_cursors` {system} yazelixCursorPackageContract field `{key}` must be {expected}; found {actual:?}."
        ));
    }
}

fn require_cursor_contract_string(
    errors: &mut Vec<String>,
    system: &str,
    metadata: &serde_json::Map<String, JsonValue>,
    key: &str,
    expected: &str,
) {
    let actual = metadata.get(key).and_then(JsonValue::as_str);
    if actual != Some(expected) {
        errors.push(format!(
            "`yazelix_cursors` {system} yazelixCursorPackageContract field `{key}` must be `{expected}`; found {actual:?}."
        ));
    }
}

fn require_cursor_contract_string_array(
    errors: &mut Vec<String>,
    system: &str,
    metadata: &serde_json::Map<String, JsonValue>,
    key: &str,
    expected: &[&str],
) {
    let actual = metadata
        .get(key)
        .and_then(JsonValue::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(JsonValue::as_str)
                .collect::<Vec<_>>()
        });
    if actual.as_deref() != Some(expected) {
        errors.push(format!(
            "`yazelix_cursors` {system} yazelixCursorPackageContract field `{key}` must be {expected:?}; found {actual:?}."
        ));
    }
}

fn validate_zellij_plugin_wasm_package_contract_with(
    contract: &ZellijPluginWasmPackageContract,
    raw_passthru: &str,
) -> Result<Vec<String>, String> {
    let parsed: JsonValue = serde_json::from_str(raw_passthru)
        .map_err(|error| format!("Invalid package passthru JSON: {error}"))?;
    let passthru = parsed
        .as_object()
        .ok_or_else(|| "Package passthru metadata must be a JSON object".to_string())?;
    let mut errors = Vec::new();
    require_passthru_wasm_path(&mut errors, contract, passthru);
    let Some(metadata) = passthru
        .get("zellijPluginWasmPackageContract")
        .and_then(JsonValue::as_object)
    else {
        errors.push(format!(
            "`{}` {} package passthru must expose `zellijPluginWasmPackageContract`.",
            contract.package_attr, contract.system
        ));
        return Ok(errors);
    };

    require_exact_zellij_plugin_wasm_metadata_fields(&mut errors, contract, metadata);
    require_contract_number(&mut errors, contract, metadata, "schemaVersion", 1);
    require_contract_string(
        &mut errors,
        contract,
        metadata,
        "pluginName",
        contract.plugin_name,
    );
    require_contract_string(
        &mut errors,
        contract,
        metadata,
        "packageAttr",
        contract.package_attr,
    );
    require_contract_string(
        &mut errors,
        contract,
        metadata,
        "wasmPath",
        contract.wasm_path,
    );
    require_contract_string(
        &mut errors,
        contract,
        metadata,
        "wasmTarget",
        "wasm32-wasip1",
    );
    for key in [
        "cargoAuditableDisabled",
        "cargoBuildHookDisabled",
        "preBuildPreservesNixRustToolchain",
        "wasmTargetRustcEnvPinned",
        "cargoBuildSerialized",
        "installCheckVerifiesWasm",
    ] {
        require_contract_bool(&mut errors, contract, metadata, key, true);
    }

    Ok(errors)
}

fn require_passthru_wasm_path(
    errors: &mut Vec<String>,
    contract: &ZellijPluginWasmPackageContract,
    passthru: &serde_json::Map<String, JsonValue>,
) {
    let actual = passthru.get("wasmPath").and_then(JsonValue::as_str);
    if actual != Some(contract.wasm_path) {
        errors.push(format!(
            "`{}` {} package passthru must expose wasmPath `{}`; found {:?}.",
            contract.package_attr, contract.system, contract.wasm_path, actual
        ));
    }
}

fn require_exact_zellij_plugin_wasm_metadata_fields(
    errors: &mut Vec<String>,
    contract: &ZellijPluginWasmPackageContract,
    metadata: &serde_json::Map<String, JsonValue>,
) {
    for required in ZELLIJ_PLUGIN_WASM_METADATA_FIELDS {
        if !metadata.contains_key(*required) {
            errors.push(format!(
                "`{}` {} zellijPluginWasmPackageContract is missing required field `{}`.",
                contract.package_attr, contract.system, required
            ));
        }
    }
    for actual in metadata.keys() {
        if !ZELLIJ_PLUGIN_WASM_METADATA_FIELDS.contains(&actual.as_str()) {
            errors.push(format!(
                "`{}` {} zellijPluginWasmPackageContract has unsupported field `{}`.",
                contract.package_attr, contract.system, actual
            ));
        }
    }
}

fn require_contract_number(
    errors: &mut Vec<String>,
    contract: &ZellijPluginWasmPackageContract,
    metadata: &serde_json::Map<String, JsonValue>,
    key: &str,
    expected: u64,
) {
    let actual = metadata.get(key).and_then(JsonValue::as_u64);
    if actual != Some(expected) {
        errors.push(format!(
            "`{}` {} zellijPluginWasmPackageContract field `{}` must be {}; found {:?}.",
            contract.package_attr, contract.system, key, expected, actual
        ));
    }
}

fn require_contract_string(
    errors: &mut Vec<String>,
    contract: &ZellijPluginWasmPackageContract,
    metadata: &serde_json::Map<String, JsonValue>,
    key: &str,
    expected: &str,
) {
    let actual = metadata.get(key).and_then(JsonValue::as_str);
    if actual != Some(expected) {
        errors.push(format!(
            "`{}` {} zellijPluginWasmPackageContract field `{}` must be `{}`; found {:?}.",
            contract.package_attr, contract.system, key, expected, actual
        ));
    }
}

fn require_contract_bool(
    errors: &mut Vec<String>,
    contract: &ZellijPluginWasmPackageContract,
    metadata: &serde_json::Map<String, JsonValue>,
    key: &str,
    expected: bool,
) {
    let actual = metadata.get(key).and_then(JsonValue::as_bool);
    if actual != Some(expected) {
        errors.push(format!(
            "`{}` {} zellijPluginWasmPackageContract field `{}` must be {}; found {:?}.",
            contract.package_attr, contract.system, key, expected, actual
        ));
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Defends: the child-release validator scopes itself to first-party Yazelix GitHub inputs instead of every flake dependency.
    #[test]
    fn locked_child_inputs_collects_first_party_yazelix_nodes_only() {
        let inputs = locked_child_inputs(
            r#"{
              "nodes": {
                "nixpkgs": {
                  "locked": {
                    "type": "github",
                    "owner": "NixOS",
                    "repo": "nixpkgs",
                    "rev": "1111111111111111111111111111111111111111"
                  }
                },
                "yazelixScreen": {
                  "locked": {
                    "type": "github",
                    "owner": "luccahuguet",
                    "repo": "yazelix-screen",
                    "rev": "2222222222222222222222222222222222222222"
                  }
                }
              }
            }"#,
        )
        .unwrap();

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].node, "yazelixScreen");
        assert_eq!(inputs[0].repo, "yazelix-screen");
    }

    // Defends: first-party Cargo git dependencies are matched to their buildRustPackage output hash entries.
    #[test]
    fn first_party_cargo_dependencies_collect_yazelix_git_sources_only() {
        let dependencies = first_party_cargo_git_dependencies(
            r#"
            [[package]]
            name = "serde"
            version = "1.0.0"
            source = "registry+https://github.com/rust-lang/crates.io-index"

            [[package]]
            name = "yazelix_screen"
            version = "0.1.0"
            source = "git+https://github.com/luccahuguet/yazelix-screen?rev=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa#aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            "#,
        )
        .unwrap();

        assert_eq!(
            dependencies,
            vec![FirstPartyCargoGitDependency {
                package_key: "yazelix_screen-0.1.0".to_string(),
                owner: "luccahuguet".to_string(),
                repo: "yazelix-screen".to_string(),
                rev: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            }]
        );
    }

    // Regression: stale Cargo vendor hashes must fail before a Nix fixed-output hash mismatch.
    #[test]
    fn cargo_output_hash_validation_reports_stale_first_party_hashes() {
        let errors = validate_cargo_git_output_hashes_with(
            r#"
            [[package]]
            name = "yazelix_screen"
            version = "0.1.0"
            source = "git+https://github.com/luccahuguet/yazelix-screen?rev=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb#bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            "#,
            r#"
            cargoLock = {
              outputHashes = {
                "yazelix_screen-0.1.0" = "sha256-old";
              };
            };
            "#,
            |_| Ok("sha256-new".to_string()),
        )
        .unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Stale cargoLock.outputHashes entry"));
        assert!(errors[0].contains("expected sha256-new"));
        assert!(errors[0].contains("found sha256-old"));
    }

    // Regression: unpublished-child detection must treat a missing fetched object as a validation error without conflating transport failures with unpublished commits.
    #[test]
    fn fetch_failure_classifier_identifies_unreachable_revision() {
        assert!(fetch_failure_means_missing_revision(
            "fatal: remote error: upload-pack: not our ref cccccccccccccccccccccccccccccccccccccccc"
        ));
        assert!(fetch_failure_means_missing_revision(
            "fatal: couldn't find remote ref cccccccccccccccccccccccccccccccccccccccc"
        ));
        assert!(!fetch_failure_means_missing_revision(
            "fatal: unable to access 'https://github.com/example/repo.git/': Could not resolve host: github.com"
        ));
    }

    // Regression: first-party Zellij plugin child packages must publish the child-owned wasm package contract that replaced main-side buildPhase inspection.
    #[test]
    fn zellij_plugin_wasm_contract_accepts_declared_package_metadata() {
        let contract = popup_contract();
        let metadata = zellij_plugin_wasm_passthru_json(serde_json::json!({
            "schemaVersion": 1,
            "pluginName": "yazelix-zellij-popup",
            "packageAttr": "yazelix_zellij_popup",
            "wasmPath": "share/yazelix_zellij_popup/yzpp.wasm",
            "wasmTarget": "wasm32-wasip1",
            "cargoAuditableDisabled": true,
            "cargoBuildHookDisabled": true,
            "preBuildPreservesNixRustToolchain": true,
            "wasmTargetRustcEnvPinned": true,
            "cargoBuildSerialized": true,
            "installCheckVerifiesWasm": true,
        }));

        let errors =
            validate_zellij_plugin_wasm_package_contract_with(&contract, &metadata).unwrap();

        assert!(errors.is_empty());
    }

    // Regression: stale child-declared artifact paths must fail before main packages a missing wasm.
    #[test]
    fn zellij_plugin_wasm_contract_rejects_stale_wasm_paths() {
        let contract = popup_contract();
        let metadata = zellij_plugin_wasm_passthru_json(serde_json::json!({
            "schemaVersion": 1,
            "pluginName": "yazelix-zellij-popup",
            "packageAttr": "yazelix_zellij_popup",
            "wasmPath": "share/yazelix_zellij_popup/old.wasm",
            "wasmTarget": "wasm32-wasip1",
            "cargoAuditableDisabled": true,
            "cargoBuildHookDisabled": true,
            "preBuildPreservesNixRustToolchain": true,
            "wasmTargetRustcEnvPinned": true,
            "cargoBuildSerialized": true,
            "installCheckVerifiesWasm": true,
        }));

        let errors =
            validate_zellij_plugin_wasm_package_contract_with(&contract, &metadata).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("field `wasmPath`"));
    }

    // Regression: issue 604 remains guarded by child-declared Darwin wasm hardening metadata.
    #[test]
    fn zellij_plugin_wasm_contract_rejects_disabled_hardening_flags() {
        let contract = popup_contract();
        let metadata = zellij_plugin_wasm_passthru_json(serde_json::json!({
            "schemaVersion": 1,
            "pluginName": "yazelix-zellij-popup",
            "packageAttr": "yazelix_zellij_popup",
            "wasmPath": "share/yazelix_zellij_popup/yzpp.wasm",
            "wasmTarget": "wasm32-wasip1",
            "cargoAuditableDisabled": false,
            "cargoBuildHookDisabled": false,
            "preBuildPreservesNixRustToolchain": false,
            "wasmTargetRustcEnvPinned": false,
            "cargoBuildSerialized": false,
            "installCheckVerifiesWasm": false,
        }));

        let errors =
            validate_zellij_plugin_wasm_package_contract_with(&contract, &metadata).unwrap();

        assert_eq!(errors.len(), 6);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cargoAuditableDisabled"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cargoBuildHookDisabled"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("preBuildPreservesNixRustToolchain"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("wasmTargetRustcEnvPinned"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cargoBuildSerialized"))
        );
    }

    // Defends: child metadata must use the exact supported contract shape instead of growing unvalidated future fields.
    #[test]
    fn zellij_plugin_wasm_contract_rejects_unknown_metadata_fields() {
        let contract = popup_contract();
        let metadata = zellij_plugin_wasm_passthru_json(serde_json::json!({
            "schemaVersion": 1,
            "pluginName": "yazelix-zellij-popup",
            "packageAttr": "yazelix_zellij_popup",
            "wasmPath": "share/yazelix_zellij_popup/yzpp.wasm",
            "wasmTarget": "wasm32-wasip1",
            "cargoAuditableDisabled": true,
            "cargoBuildHookDisabled": true,
            "preBuildPreservesNixRustToolchain": true,
            "wasmTargetRustcEnvPinned": true,
            "cargoBuildSerialized": true,
            "installCheckVerifiesWasm": true,
            "futureBuildPhaseHint": "do not accept this",
        }));

        let errors =
            validate_zellij_plugin_wasm_package_contract_with(&contract, &metadata).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("unsupported field `futureBuildPhaseHint`"));
    }

    // Defends: cursor package metadata must stay exact so main cannot silently consume stale or future child artifact shapes.
    #[test]
    fn cursor_package_contract_accepts_declared_metadata() {
        let metadata = cursor_package_passthru_json(valid_cursor_package_contract_json());

        let errors = validate_cursor_package_contract_with("x86_64-linux", &metadata).unwrap();

        assert!(errors.is_empty());
    }

    // Defends: child cursor package metadata cannot grow unused planning fields that main never validates.
    #[test]
    fn cursor_package_contract_rejects_unknown_metadata_fields() {
        let mut contract = valid_cursor_package_contract_json();
        contract
            .as_object_mut()
            .expect("test fixture is an object")
            .insert("requiredRuntimeScripts".to_string(), serde_json::json!([]));
        let metadata = cursor_package_passthru_json(contract);

        let errors = validate_cursor_package_contract_with("x86_64-linux", &metadata).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("unsupported field `requiredRuntimeScripts`"));
    }

    fn valid_cursor_package_contract_json() -> JsonValue {
        serde_json::json!({
            "schemaVersion": 1,
            "packageName": "yazelix-cursors",
            "shareRoot": "share/yazelix/yazelix_cursors",
            "shaderRoot": "share/yazelix/yazelix_cursors/shaders",
            "generatedEffectRoot": "share/yazelix/yazelix_cursors/shaders/generated_effects",
            "requiredTargets": CURSOR_PACKAGE_REQUIRED_TARGETS,
            "requiredShaderFiles": CURSOR_PACKAGE_REQUIRED_SHADER_FILES,
            "forbiddenShaderFiles": ["build_shaders.nu"],
        })
    }

    // Regression: the main repo must not reintroduce a mirrored Ghostty cursor shader source tree after child ownership.
    #[test]
    fn child_release_validation_rejects_main_cursor_shader_tree() {
        let temp = tempdir().unwrap();
        let shader_root = temp
            .path()
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty")
            .join("shaders");
        fs::create_dir_all(&shader_root).unwrap();

        let errors = validate_main_cursor_shader_tree_deleted(temp.path());

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Main repo must not own Ghostty cursor shader assets"));
    }

    fn popup_contract() -> ZellijPluginWasmPackageContract {
        ZellijPluginWasmPackageContract {
            package_attr: "yazelix_zellij_popup",
            system: "aarch64-darwin",
            plugin_name: "yazelix-zellij-popup",
            wasm_path: "share/yazelix_zellij_popup/yzpp.wasm",
        }
    }

    fn zellij_plugin_wasm_passthru_json(contract: JsonValue) -> String {
        serde_json::json!({
            "wasmPath": "share/yazelix_zellij_popup/yzpp.wasm",
            "zellijPluginWasmPackageContract": contract,
        })
        .to_string()
    }

    fn cursor_package_passthru_json(contract: JsonValue) -> String {
        serde_json::json!({
            "yazelixCursorPackageContract": contract,
        })
        .to_string()
    }
}
