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
}

const ZELLIJ_PLUGIN_WASM_PACKAGE_CONTRACTS: &[ZellijPluginWasmPackageContract] = &[
    ZellijPluginWasmPackageContract {
        package_attr: "yazelix_zellij_pane_orchestrator",
        system: "aarch64-darwin",
    },
    ZellijPluginWasmPackageContract {
        package_attr: "yazelix_zellij_popup",
        system: "aarch64-darwin",
    },
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
            package_derivation_metadata(repo_root, contract.system, contract.package_attr)?;
        errors.extend(validate_zellij_plugin_wasm_derivation_with(
            contract, &metadata,
        )?);
    }
    Ok(errors)
}

fn package_derivation_metadata(
    repo_root: &Path,
    system: &str,
    package_attr: &str,
) -> Result<String, String> {
    let flake_attr = format!(".#packages.{system}.{package_attr}.drvPath");
    let eval = Command::new("nix")
        .current_dir(repo_root)
        .args(["eval", "--raw", &flake_attr, "--accept-flake-config"])
        .output()
        .map_err(|error| format!("Failed to run `nix eval --raw {flake_attr}`: {error}"))?;
    if !eval.status.success() {
        return Err(format!(
            "Failed to instantiate child package derivation `{flake_attr}`\n{}",
            String::from_utf8_lossy(&eval.stderr).trim()
        ));
    }

    let drv_path = String::from_utf8_lossy(&eval.stdout).trim().to_string();
    if drv_path.is_empty() {
        return Err(format!(
            "`nix eval --raw {flake_attr}` returned an empty drv path"
        ));
    }

    let show = Command::new("nix")
        .args(["derivation", "show", &drv_path])
        .output()
        .map_err(|error| format!("Failed to run `nix derivation show {drv_path}`: {error}"))?;
    if !show.status.success() {
        return Err(format!(
            "Failed to inspect child package derivation `{drv_path}`\n{}",
            String::from_utf8_lossy(&show.stderr).trim()
        ));
    }

    String::from_utf8(show.stdout)
        .map_err(|error| format!("Invalid UTF-8 from `nix derivation show {drv_path}`: {error}"))
}

fn validate_zellij_plugin_wasm_derivation_with(
    contract: &ZellijPluginWasmPackageContract,
    raw_metadata: &str,
) -> Result<Vec<String>, String> {
    let env = single_derivation_env(raw_metadata)?;
    let mut errors = Vec::new();

    require_derivation_system(&mut errors, contract.package_attr, contract.system, &env);
    require_derivation_env_value(
        &mut errors,
        contract.package_attr,
        contract.system,
        &env,
        "dontCargoBuild",
        "1",
        "disable cargoBuildHook for the manual wasm package build",
    );

    let Some(build_phase) = env.get("buildPhase") else {
        errors.push(format!(
            "`{}` {} derivation metadata has no buildPhase.",
            contract.package_attr, contract.system
        ));
        return Ok(errors);
    };

    require_build_phase_marker(
        &mut errors,
        contract,
        build_phase,
        "export CARGO=",
        "export explicit CARGO from the combined wasm-capable Rust toolchain",
    );
    require_build_phase_marker(
        &mut errors,
        contract,
        build_phase,
        "export PATH=",
        "put the combined wasm-capable Rust toolchain on PATH",
    );
    require_build_phase_marker(
        &mut errors,
        contract,
        build_phase,
        "export RUSTC=",
        "export explicit RUSTC from the combined wasm-capable Rust toolchain",
    );
    require_build_phase_marker(
        &mut errors,
        contract,
        build_phase,
        "--print target-libdir --target wasm32-wasip1",
        "check that the selected Rust toolchain has wasm32-wasip1 rust-std",
    );
    require_build_phase_marker(
        &mut errors,
        contract,
        build_phase,
        "\"$CARGO\" build",
        "invoke Cargo through the explicit CARGO variable",
    );
    require_build_phase_marker(
        &mut errors,
        contract,
        build_phase,
        "--target wasm32-wasip1",
        "build the plugin for wasm32-wasip1",
    );
    require_build_phase_order(
        &mut errors,
        contract,
        build_phase,
        "export CARGO=",
        "runHook preBuild",
        "export the wasm-capable Cargo before preBuild hooks can run",
    );
    require_build_phase_order(
        &mut errors,
        contract,
        build_phase,
        "export RUSTC=",
        "runHook preBuild",
        "export the wasm-capable Rust compiler before preBuild hooks can run",
    );
    require_build_phase_order(
        &mut errors,
        contract,
        build_phase,
        "--print target-libdir --target wasm32-wasip1",
        "runHook preBuild",
        "verify wasm32-wasip1 rust-std before preBuild hooks can run",
    );
    require_build_phase_order(
        &mut errors,
        contract,
        build_phase,
        "runHook preBuild",
        "\"$CARGO\" build",
        "run the manual wasm build after preBuild hooks",
    );

    Ok(errors)
}

fn single_derivation_env(raw_metadata: &str) -> Result<HashMap<String, String>, String> {
    let parsed: JsonValue = serde_json::from_str(raw_metadata)
        .map_err(|error| format!("Invalid JSON from `nix derivation show`: {error}"))?;
    let derivations = parsed
        .get("derivations")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| {
            "`nix derivation show` output is missing object `derivations`".to_string()
        })?;
    if derivations.len() != 1 {
        return Err(format!(
            "`nix derivation show` returned {} derivations; expected exactly one",
            derivations.len()
        ));
    }
    let derivation = derivations
        .values()
        .next()
        .ok_or_else(|| "`nix derivation show` returned no derivation entries".to_string())?;
    let env = derivation
        .get("env")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| {
            "`nix derivation show` derivation entry is missing object `env`".to_string()
        })?;

    Ok(env
        .iter()
        .filter_map(|(key, value)| value.as_str().map(|value| (key.clone(), value.to_string())))
        .collect())
}

fn require_derivation_system(
    errors: &mut Vec<String>,
    package_attr: &str,
    expected_system: &str,
    env: &HashMap<String, String>,
) {
    let system = env.get("system").map(String::as_str);
    if system != Some(expected_system) {
        errors.push(format!(
            "`{package_attr}` derivation metadata has system {:?}; expected `{expected_system}`.",
            system
        ));
    }
}

fn require_derivation_env_value(
    errors: &mut Vec<String>,
    package_attr: &str,
    system: &str,
    env: &HashMap<String, String>,
    key: &str,
    expected: &str,
    description: &str,
) {
    let actual = env.get(key).map(String::as_str);
    if actual == Some(expected) {
        return;
    }
    errors.push(format!(
        "`{package_attr}` {system} derivation metadata must {description}. Expected env `{key}` = `{expected}`, found {:?}.",
        actual
    ));
}

fn require_build_phase_marker(
    errors: &mut Vec<String>,
    contract: &ZellijPluginWasmPackageContract,
    build_phase: &str,
    marker: &str,
    description: &str,
) {
    if build_phase.contains(marker) {
        return;
    }
    errors.push(format!(
        "`{}` {} buildPhase must {}. Missing marker `{}`.",
        contract.package_attr, contract.system, description, marker
    ));
}

fn require_build_phase_order(
    errors: &mut Vec<String>,
    contract: &ZellijPluginWasmPackageContract,
    build_phase: &str,
    before: &str,
    after: &str,
    description: &str,
) {
    let before_index = build_phase.find(before);
    let after_index = build_phase.find(after);
    if matches!((before_index, after_index), (Some(left), Some(right)) if left < right) {
        return;
    }
    errors.push(format!(
        "`{}` {} buildPhase must {}. Expected marker `{}` before `{}`.",
        contract.package_attr, contract.system, description, before, after
    ));
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;

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

    // Regression: first-party Zellij plugin child packages must keep the explicit wasm-capable toolchain hardening that fixed macOS issue 604.
    #[test]
    fn zellij_plugin_wasm_contract_accepts_hardened_build_phase() {
        let contract = ZellijPluginWasmPackageContract {
            package_attr: "yazelix_zellij_popup",
            system: "aarch64-darwin",
        };
        let metadata = derivation_metadata_json(
            "aarch64-darwin",
            r#"
            export CARGO="/nix/store/toolchain/bin/cargo"
            export RUSTC="/nix/store/toolchain/bin/rustc"
            export PATH="/nix/store/toolchain/bin:$PATH"
            wasm_target_libdir="$("$RUSTC" --print target-libdir --target wasm32-wasip1)"
            runHook preBuild
            "$CARGO" build --profile release --target wasm32-wasip1
            runHook postBuild
            "#,
            &[("dontCargoBuild", "1")],
        );

        let errors = validate_zellij_plugin_wasm_derivation_with(&contract, &metadata).unwrap();

        assert!(errors.is_empty());
    }

    // Regression: a child package that falls back to plain cargo must fail before another Darwin wasm target build reaches users.
    #[test]
    fn zellij_plugin_wasm_contract_rejects_plain_cargo_build_phase() {
        let contract = ZellijPluginWasmPackageContract {
            package_attr: "yazelix_zellij_popup",
            system: "aarch64-darwin",
        };
        let metadata = derivation_metadata_json(
            "aarch64-darwin",
            r#"
            cargo build --profile release --target wasm32-wasip1
            "#,
            &[],
        );

        let errors = validate_zellij_plugin_wasm_derivation_with(&contract, &metadata).unwrap();

        assert!(errors.len() >= 7);
        assert!(errors.iter().any(|error| error.contains("dontCargoBuild")));
        assert!(errors.iter().any(|error| error.contains("export CARGO=")));
        assert!(errors.iter().any(|error| error.contains("export RUSTC=")));
        assert!(errors.iter().any(|error| error.contains("export PATH=")));
        assert!(
            errors
                .iter()
                .any(|error| error.contains("--print target-libdir --target wasm32-wasip1"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("\"$CARGO\" build"))
        );
    }

    // Regression: issue 604 reproduced when runHook preBuild ran before the Fenix exports, because cargoBuildHook could use the wrong Darwin Rust.
    #[test]
    fn zellij_plugin_wasm_contract_rejects_prebuild_before_toolchain_setup() {
        let contract = ZellijPluginWasmPackageContract {
            package_attr: "yazelix_zellij_popup",
            system: "aarch64-darwin",
        };
        let metadata = derivation_metadata_json(
            "aarch64-darwin",
            r#"
            runHook preBuild
            export CARGO="/nix/store/toolchain/bin/cargo"
            export RUSTC="/nix/store/toolchain/bin/rustc"
            export PATH="/nix/store/toolchain/bin:$PATH"
            wasm_target_libdir="$("$RUSTC" --print target-libdir --target wasm32-wasip1)"
            "$CARGO" build --profile release --target wasm32-wasip1
            runHook postBuild
            "#,
            &[("dontCargoBuild", "1")],
        );

        let errors = validate_zellij_plugin_wasm_derivation_with(&contract, &metadata).unwrap();

        assert_eq!(errors.len(), 3);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("export CARGO=") && error.contains("runHook preBuild"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("export RUSTC=") && error.contains("runHook preBuild"))
        );
        assert!(errors.iter().any(|error| {
            error.contains("--print target-libdir --target wasm32-wasip1")
                && error.contains("runHook preBuild")
        }));
    }

    // Defends: the derivation metadata gate checks the evaluated system instead of assuming the flake attr path returned the requested platform.
    #[test]
    fn zellij_plugin_wasm_contract_rejects_wrong_derivation_system() {
        let contract = ZellijPluginWasmPackageContract {
            package_attr: "yazelix_zellij_popup",
            system: "aarch64-darwin",
        };
        let metadata = derivation_metadata_json(
            "x86_64-linux",
            r#"
            export CARGO="/nix/store/toolchain/bin/cargo"
            export RUSTC="/nix/store/toolchain/bin/rustc"
            export PATH="/nix/store/toolchain/bin:$PATH"
            wasm_target_libdir="$("$RUSTC" --print target-libdir --target wasm32-wasip1)"
            runHook preBuild
            "$CARGO" build --profile release --target wasm32-wasip1
            runHook postBuild
            "#,
            &[("dontCargoBuild", "1")],
        );

        let errors = validate_zellij_plugin_wasm_derivation_with(&contract, &metadata).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("expected `aarch64-darwin`"));
    }

    fn derivation_metadata_json(
        system: &str,
        build_phase: &str,
        extra_env: &[(&str, &str)],
    ) -> String {
        let mut env = serde_json::json!({
            "system": system,
            "buildPhase": build_phase,
        });
        let env = env.as_object_mut().expect("fixture env is object");
        for (key, value) in extra_env {
            env.insert((*key).to_string(), serde_json::json!(value));
        }

        serde_json::json!({
            "derivations": {
                "sample.drv": {
                    "env": env,
                },
            },
        })
        .to_string()
    }
}
