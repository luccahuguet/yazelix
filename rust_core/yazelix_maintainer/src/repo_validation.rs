use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const POLICY_ONLY_CONTRACT_PATHS: &[&str] = &["docs/contracts/test_suite_governance.md"];
const PLANNING_LANGUAGE_MARKERS: &[&str] = &[
    "Follow-Up Beads",
    "Follow-up bead",
    "Prototype Outcome",
    "Historical planning note",
    "Historical transition note",
    "Status: Historical",
];
const ALLOWED_CONTRACT_TYPES: &[&str] = &[
    "behavior",
    "invariant",
    "boundary",
    "ownership",
    "failure_mode",
    "non_goal",
];
const ALLOWED_CONTRACT_STATUSES: &[&str] =
    &["live", "planning", "deprecated", "historical", "quarantine"];
const ALLOWED_VERIFICATION_MODES: &[&str] = &["automated", "validator", "manual", "unverified"];
const ALLOWED_TEST_LANES: &[&str] = &["default", "maintainer", "sweep", "manual"];
const PACKAGE_TEST_FORBIDDEN_COMMANDS: &[&str] =
    &["nix", "nix-build", "nix-env", "nix-shell", "home-manager"];
const PACKAGE_TEST_FORBIDDEN_SHELL_SNIPPETS: &[&str] = &[
    "nix build",
    "nix eval",
    "nix flake",
    "nix profile",
    "home-manager switch",
];

#[derive(Debug, Default)]
pub struct ValidationReport {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ContractItem {
    pub id: String,
    pub contract_path: String,
    pub item_type: Option<String>,
    pub status: Option<String>,
    pub owner: Option<String>,
    pub statement: Option<String>,
    pub verification: Option<String>,
}

#[derive(Debug, Clone)]
struct RustTestRecord {
    attribute_index: usize,
    test_name: String,
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

pub fn validate_contracts(repo_root: &Path) -> Result<ValidationReport, String> {
    let contract_files = load_contract_files(repo_root)?;
    if contract_files.is_empty() {
        return Ok(ValidationReport::default());
    }

    let contract_items = load_contract_items(repo_root)?;
    let mut report = ValidationReport::default();
    let mut seen_ids: HashMap<String, String> = HashMap::new();

    for contract_path in &contract_files {
        report
            .errors
            .extend(validate_contract_file(repo_root, contract_path)?);
    }

    for item in &contract_items {
        if let Some(existing_contract_path) = seen_ids.get(&item.id) {
            report.errors.push(format!(
                "Duplicate contract item id `{}` appears in both {} and {}",
                item.id, existing_contract_path, item.contract_path
            ));
        } else {
            seen_ids.insert(item.id.clone(), item.contract_path.clone());
        }

        report.errors.extend(validate_contract_item(item));
    }

    Ok(report)
}

pub fn validate_rust_test_traceability(repo_root: &Path) -> Result<ValidationReport, String> {
    let contract_items = load_contract_items(repo_root)?;
    let mut report = ValidationReport::default();

    for test_path in load_all_nu_test_file_paths(repo_root)? {
        let relative_path = relative_to_repo(repo_root, &test_path)?;
        report.errors.push(format!(
            "Governed Nushell test files are no longer part of the canonical suite; port strong tests to Rust nextest or demote shell-heavy probes out of the test_*.nu namespace: {}",
            relative_path
        ));
    }

    for rust_path in load_rust_test_file_paths(repo_root)? {
        let relative_path = relative_to_repo(repo_root, &rust_path)?;
        if !file_contains_rust_tests(repo_root, &relative_path)? {
            continue;
        }

        let Some(lane) = parse_rust_test_lane(repo_root, &relative_path)? else {
            report.errors.push(format!(
                "Missing '// Test lane:' declaration in Rust test file: {}",
                relative_path
            ));
            continue;
        };

        if !ALLOWED_TEST_LANES.contains(&lane.as_str()) {
            report.errors.push(format!(
                "Rust test file declares unsupported lane '{}': {}",
                lane, relative_path
            ));
            continue;
        }

        for test_record in load_defined_rust_tests(repo_root, &relative_path)? {
            if !has_valid_rust_definition_test_justification(
                repo_root,
                &relative_path,
                test_record.attribute_index,
            )? {
                report.errors.push(format!(
                    "Governed Rust test is missing a nearby '// Defends:', '// Regression:', or '// Invariant:' marker: {} :: {}",
                    relative_path, test_record.test_name
                ));
            }

            report
                .errors
                .extend(collect_rust_definition_contract_traceability_errors(
                    repo_root,
                    &relative_path,
                    &test_record,
                    &lane,
                    &contract_items,
                )?);
        }
    }

    Ok(report)
}

pub fn validate_package_rust_test_purity(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();

    for rust_path in load_rust_test_file_paths(repo_root)? {
        let relative_path = relative_to_repo(repo_root, &rust_path)?;
        let lines = read_lines(&rust_path)?;
        let Some(start_index) = package_test_scan_start_index(&relative_path, &lines) else {
            continue;
        };

        for (line_index, line) in lines.iter().enumerate().skip(start_index) {
            if let Some(reason) = package_test_forbidden_host_tool_reason(line) {
                report.errors.push(format!(
                    "Package-time Rust test uses host-only tooling ({reason}): {}:{}\n   Move this check into a maintainer validator or explicit package gate instead of the default Cargo test set.",
                    relative_path,
                    line_index + 1,
                ));
            }
        }
    }

    Ok(report)
}

fn validate_contract_file(repo_root: &Path, contract_path: &Path) -> Result<Vec<String>, String> {
    let relative_path = relative_to_repo(repo_root, contract_path)?;
    let content = fs::read_to_string(contract_path).map_err(|error| {
        format!(
            "Failed to read contract file {}: {}",
            contract_path.display(),
            error
        )
    })?;

    let mut errors = Vec::new();

    if content.contains("docs/specs") {
        errors.push(format!(
            "{}: canonical contracts must not reference stale `docs/specs` paths",
            relative_path
        ));
    }

    for (line_index, line) in content.lines().enumerate() {
        let line_number = line_index + 1;
        let lower_line = line.to_lowercase();
        if lower_line.contains("bead") || line_contains_bead_id(line) {
            errors.push(format!(
                "{}:{}: canonical contracts must not mention Beads; put planning traceability in the issue tracker instead",
                relative_path, line_number
            ));
        }
        for marker in PLANNING_LANGUAGE_MARKERS {
            if line.contains(marker) {
                errors.push(format!(
                    "{}:{}: canonical contracts must not contain planning marker `{}`",
                    relative_path, line_number, marker
                ));
            }
        }
    }

    if !content.contains("## Verification")
        && !content.contains("- Verification:")
        && !content.contains("- Defended by:")
    {
        errors.push(format!(
            "{}: canonical contract must name a concrete verification path",
            relative_path
        ));
    }

    Ok(errors)
}

fn validate_contract_item(item: &ContractItem) -> Vec<String> {
    let mut errors = Vec::new();

    match item.item_type.as_deref() {
        None => errors.push(format!(
            "{}: contract item `{}` is missing `- Type:`",
            item.contract_path, item.id
        )),
        Some(item_type) if !ALLOWED_CONTRACT_TYPES.contains(&item_type) => errors.push(format!(
            "{}: contract item `{}` declares unsupported type `{}`",
            item.contract_path, item.id, item_type
        )),
        Some(_) => {}
    }

    let Some(status) = item.status.as_deref() else {
        errors.push(format!(
            "{}: contract item `{}` is missing `- Status:`",
            item.contract_path, item.id
        ));
        return errors;
    };

    if !ALLOWED_CONTRACT_STATUSES.contains(&status) {
        errors.push(format!(
            "{}: contract item `{}` declares unsupported status `{}`",
            item.contract_path, item.id, status
        ));
        return errors;
    }

    if status != "historical" {
        for (label, field) in [
            ("Owner", item.owner.as_ref()),
            ("Statement", item.statement.as_ref()),
            ("Verification", item.verification.as_ref()),
        ] {
            if field.is_none() {
                errors.push(format!(
                    "{}: contract item `{}` is missing `- {}:`",
                    item.contract_path, item.id, label
                ));
            }
        }
    }

    if let Some(verification) = item.verification.as_deref() {
        if !ALLOWED_VERIFICATION_MODES
            .iter()
            .any(|mode| verification.contains(mode))
        {
            errors.push(format!(
                "{}: contract item `{}` has `- Verification:` but no allowed verification mode keyword",
                item.contract_path, item.id
            ));
        }
    }

    if status == "live" && item.verification.is_none() {
        errors.push(format!(
            "{}: live contract item `{}` must name a verification path or explicit manual/unverified reason",
            item.contract_path, item.id
        ));
    }

    errors
}

fn load_contract_items(repo_root: &Path) -> Result<Vec<ContractItem>, String> {
    let mut items = Vec::new();
    for contract_path in load_contract_files(repo_root)? {
        let relative_path = relative_to_repo(repo_root, &contract_path)?;
        let lines = read_lines(&contract_path)?;
        let mut current: Option<ContractItem> = None;

        for line in lines {
            let trimmed = line.trim();
            if let Some(id) = parse_contract_heading(trimmed) {
                if let Some(item) = current.take() {
                    items.push(item);
                }
                current = Some(ContractItem {
                    id,
                    contract_path: relative_path.clone(),
                    item_type: None,
                    status: None,
                    owner: None,
                    statement: None,
                    verification: None,
                });
                continue;
            }

            let Some(item) = current.as_mut() else {
                continue;
            };

            if (trimmed.starts_with("# ")
                || trimmed.starts_with("## ")
                || trimmed.starts_with("### "))
                && !trimmed.starts_with("#### ")
            {
                items.push(current.take().unwrap());
                continue;
            }

            if let Some((field_name, value)) = parse_contract_field(trimmed) {
                match normalize_contract_field_name(&field_name).as_str() {
                    "type" => item.item_type = Some(value),
                    "status" => item.status = Some(value),
                    "owner" => item.owner = Some(value),
                    "statement" => item.statement = Some(value),
                    "verification" => item.verification = Some(value),
                    _ => {}
                }
            }
        }

        if let Some(item) = current.take() {
            items.push(item);
        }
    }

    Ok(items)
}

fn load_contract_files(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let contracts_dir = repo_root.join("docs").join("contracts");
    for entry in fs::read_dir(&contracts_dir).map_err(|error| {
        format!(
            "Failed to read contracts directory {}: {}",
            contracts_dir.display(),
            error
        )
    })? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn load_all_nu_test_file_paths(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let dev_dir = repo_root.join("nushell").join("scripts").join("dev");
    let mut files = Vec::new();
    for entry in fs::read_dir(&dev_dir)
        .map_err(|error| format!("Failed to read {}: {}", dev_dir.display(), error))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("nu")
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("test_"))
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn load_rust_test_file_paths(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_rust_rs_files(&repo_root.join("rust_core"), &mut files)?;
    collect_rust_rs_files(&repo_root.join("rust_plugins"), &mut files)?;
    files.sort();
    files.dedup();
    Ok(files)
}

fn package_test_scan_start_index(relative_path: &str, lines: &[String]) -> Option<usize> {
    if relative_path.contains("/tests/") {
        return Some(0);
    }

    lines.iter().position(|line| line.trim() == "#[cfg(test)]")
}

fn package_test_forbidden_host_tool_reason(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("//") {
        return None;
    }

    let whitespace_stripped = trimmed.split_whitespace().collect::<String>();
    for command in PACKAGE_TEST_FORBIDDEN_COMMANDS {
        if whitespace_stripped.contains(&format!("Command::new(\"{command}\")"))
            || whitespace_stripped.contains(&format!("std::process::Command::new(\"{command}\")"))
        {
            return Some(format!("Command::new(\"{command}\")"));
        }
    }

    let executable_string_context = trimmed.contains(".arg(")
        || trimmed.contains(".args(")
        || whitespace_stripped.contains("Command::new(\"sh\")")
        || whitespace_stripped.contains("Command::new(\"/bin/sh\")");
    for snippet in PACKAGE_TEST_FORBIDDEN_SHELL_SNIPPETS {
        if executable_string_context && trimmed.contains(snippet) {
            return Some(format!("shell snippet `{snippet}`"));
        }
    }

    None
}

fn collect_rust_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(dir).map_err(|error| format!("Failed to read {}: {}", dir.display(), error))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path
            .components()
            .any(|component| component.as_os_str().to_string_lossy() == "target")
        {
            continue;
        }
        if path.is_dir() {
            collect_rust_rs_files(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn parse_rust_test_lane(repo_root: &Path, relative_path: &str) -> Result<Option<String>, String> {
    let full_path = repo_root.join(relative_path);
    let lines = read_lines(&full_path)?;
    Ok(lines
        .into_iter()
        .map(|line| line.trim().to_string())
        .find_map(|line| {
            line.strip_prefix("// Test lane:")
                .map(|lane| lane.trim().to_string())
        }))
}

fn load_defined_rust_tests(
    repo_root: &Path,
    relative_path: &str,
) -> Result<Vec<RustTestRecord>, String> {
    let lines = read_lines(&repo_root.join(relative_path))?;
    let mut tests = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        if !is_rust_test_attribute_line(line) {
            continue;
        }
        tests.push(RustTestRecord {
            attribute_index: index,
            test_name: parse_rust_test_name_after_index(&lines, relative_path, index)?,
        });
    }

    Ok(tests)
}

fn load_rust_definition_traceability_lines(
    repo_root: &Path,
    relative_path: &str,
    attribute_index: usize,
) -> Result<Vec<String>, String> {
    Ok(
        get_prior_nonempty_lines_before_index(repo_root, relative_path, attribute_index)?
            .into_iter()
            .filter(|line| {
                [
                    "// Defends:",
                    "// Regression:",
                    "// Invariant:",
                    "// Contract:",
                ]
                .iter()
                .any(|prefix| line.starts_with(prefix))
            })
            .collect(),
    )
}

fn collect_rust_definition_contract_traceability_errors(
    repo_root: &Path,
    relative_path: &str,
    test_record: &RustTestRecord,
    lane: &str,
    contract_items: &[ContractItem],
) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let contract_ids =
        load_rust_definition_contract_ids(repo_root, relative_path, test_record.attribute_index)?;
    let defended_contract_paths = load_rust_definition_defended_contract_paths(
        repo_root,
        relative_path,
        test_record.attribute_index,
    )?;
    let has_regression_or_invariant = has_rust_definition_regression_or_invariant(
        repo_root,
        relative_path,
        test_record.attribute_index,
    )?;

    if contract_ids.is_empty()
        && rust_definition_has_policy_only_traceability(
            repo_root,
            relative_path,
            test_record.attribute_index,
        )?
    {
        errors.push(format!(
            "Governed Rust test cannot rely only on `docs/contracts/test_suite_governance.md` as nearby traceability: {} :: {}",
            relative_path, test_record.test_name
        ));
    }

    if lane == "default"
        && contract_ids.is_empty()
        && !has_regression_or_invariant
        && defended_contract_paths
            .iter()
            .any(|contract_path| contract_has_contract_items(contract_items, contract_path))
    {
        errors.push(format!(
            "Default-lane Rust test defends a contract with indexed items but is missing a nearby '// Contract:' marker: {} :: {}",
            relative_path, test_record.test_name
        ));
    }

    for contract_id in contract_ids {
        match find_contract_item(contract_items, &contract_id) {
            None => errors.push(format!(
                "Governed Rust test references unknown contract id `{}`: {} :: {}",
                contract_id, relative_path, test_record.test_name
            )),
            Some(item)
                if !matches!(
                    item.status.as_deref(),
                    Some("live" | "deprecated" | "quarantine")
                ) =>
            {
                errors.push(format!(
                    "Governed Rust test references contract id `{}` with unsupported status `{}`: {} :: {}",
                    contract_id,
                    item.status.as_deref().unwrap_or(""),
                    relative_path,
                    test_record.test_name
                ))
            }
            Some(_) => {}
        }
    }

    Ok(errors)
}

fn has_valid_rust_definition_test_justification(
    repo_root: &Path,
    relative_path: &str,
    attribute_index: usize,
) -> Result<bool, String> {
    Ok(
        load_rust_definition_traceability_lines(repo_root, relative_path, attribute_index)?
            .iter()
            .any(|line| {
                ["// Defends:", "// Regression:", "// Invariant:"]
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
            }),
    )
}

fn rust_definition_has_policy_only_traceability(
    repo_root: &Path,
    relative_path: &str,
    attribute_index: usize,
) -> Result<bool, String> {
    let contract_paths =
        load_rust_definition_defended_contract_paths(repo_root, relative_path, attribute_index)?;
    if contract_paths.is_empty() {
        return Ok(false);
    }
    if has_rust_definition_regression_or_invariant(repo_root, relative_path, attribute_index)? {
        return Ok(false);
    }
    Ok(contract_paths
        .iter()
        .all(|contract_path| is_policy_only_contract_path(contract_path)))
}

fn has_rust_definition_regression_or_invariant(
    repo_root: &Path,
    relative_path: &str,
    attribute_index: usize,
) -> Result<bool, String> {
    Ok(
        load_rust_definition_traceability_lines(repo_root, relative_path, attribute_index)?
            .iter()
            .any(|line| line.starts_with("// Regression:") || line.starts_with("// Invariant:")),
    )
}

fn load_rust_definition_contract_ids(
    repo_root: &Path,
    relative_path: &str,
    attribute_index: usize,
) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();
    for line in load_rust_definition_traceability_lines(repo_root, relative_path, attribute_index)?
    {
        if line.starts_with("// Contract:") {
            for id in parse_contract_marker_ids(&line, "// Contract:") {
                push_unique(&mut ids, id);
            }
        }
    }
    Ok(ids)
}

fn load_rust_definition_defended_contract_paths(
    repo_root: &Path,
    relative_path: &str,
    attribute_index: usize,
) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for line in load_rust_definition_traceability_lines(repo_root, relative_path, attribute_index)?
    {
        if let Some(contract_path) = parse_rust_defends_contract_path(repo_root, &line) {
            push_unique(&mut paths, contract_path);
        }
    }
    Ok(paths)
}

fn file_contains_rust_tests(repo_root: &Path, relative_path: &str) -> Result<bool, String> {
    Ok(read_lines(&repo_root.join(relative_path))?
        .iter()
        .any(|line| is_rust_test_attribute_line(line)))
}

fn is_rust_test_attribute_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !(trimmed.starts_with("#[") && trimmed.ends_with(']')) {
        return false;
    }
    let inner = &trimmed[2..trimmed.len() - 1];
    let base = inner.split('(').next().unwrap_or("").trim();
    matches!(base.split("::").last(), Some("test"))
}

fn parse_rust_test_name_after_index(
    lines: &[String],
    relative_path: &str,
    attribute_index: usize,
) -> Result<String, String> {
    let candidate_line = lines
        .iter()
        .skip(attribute_index + 1)
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "Could not find Rust test function after attribute in: {} :: line {}",
                relative_path,
                attribute_index + 1
            )
        })?
        .trim()
        .to_string();

    let mut remainder = candidate_line.as_str();
    if let Some(stripped) = remainder.strip_prefix("pub ") {
        remainder = stripped.trim_start();
    }
    if let Some(stripped) = remainder.strip_prefix("async ") {
        remainder = stripped.trim_start();
    }
    let Some(after_fn) = remainder.strip_prefix("fn ") else {
        return Err(format!(
            "Could not parse Rust test function after attribute in: {} :: {}",
            relative_path, candidate_line
        ));
    };
    let name: String = after_fn
        .chars()
        .take_while(|char| char.is_ascii_alphanumeric() || *char == '_')
        .collect();
    if name.is_empty() {
        return Err(format!(
            "Could not parse Rust test function after attribute in: {} :: {}",
            relative_path, candidate_line
        ));
    }
    Ok(name)
}

fn get_prior_nonempty_lines_before_index(
    repo_root: &Path,
    relative_path: &str,
    line_index: usize,
) -> Result<Vec<String>, String> {
    let lines = read_lines(&repo_root.join(relative_path))?;
    Ok(lines
        .into_iter()
        .take(line_index)
        .rev()
        .filter_map(|line| {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .take(4)
        .collect())
}

fn parse_contract_heading(line: &str) -> Option<String> {
    let candidate = line.strip_prefix("#### ")?.trim();
    if is_valid_contract_id(candidate) {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn is_valid_contract_id(candidate: &str) -> bool {
    let Some((prefix, suffix)) = candidate.split_once('-') else {
        return false;
    };
    if prefix.len() < 2 || prefix.len() > 8 || suffix.len() < 3 {
        return false;
    }
    prefix
        .chars()
        .all(|char| char.is_ascii_uppercase() || char.is_ascii_digit())
        && suffix.chars().all(|char| char.is_ascii_digit())
}

fn parse_contract_field(line: &str) -> Option<(String, String)> {
    let payload = line.strip_prefix("- ")?;
    let (field_name, value) = payload.split_once(':')?;
    if !field_name
        .chars()
        .next()
        .is_some_and(|char| char.is_ascii_alphabetic())
    {
        return None;
    }
    Some((field_name.trim().to_string(), value.trim().to_string()))
}

fn normalize_contract_field_name(field_name: &str) -> String {
    field_name.to_lowercase().replace(' ', "_")
}

fn parse_rust_defends_contract_path(repo_root: &Path, line: &str) -> Option<String> {
    parse_defends_contract_path(repo_root, line, "// Defends:")
}

fn parse_defends_contract_path(repo_root: &Path, line: &str, marker: &str) -> Option<String> {
    let candidate = line.trim().strip_prefix(marker)?.trim();
    if !candidate.starts_with("docs/") || !repo_root.join(candidate).exists() {
        return None;
    }
    Some(candidate.to_string())
}

fn parse_contract_marker_ids(line: &str, marker: &str) -> Vec<String> {
    line.trim()
        .strip_prefix(marker)
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn find_contract_item<'a>(
    contract_items: &'a [ContractItem],
    contract_id: &str,
) -> Option<&'a ContractItem> {
    contract_items.iter().find(|item| item.id == contract_id)
}

fn contract_has_contract_items(contract_items: &[ContractItem], contract_path: &str) -> bool {
    contract_items
        .iter()
        .any(|item| item.contract_path == contract_path)
}

fn is_policy_only_contract_path(contract_path: &str) -> bool {
    POLICY_ONLY_CONTRACT_PATHS.contains(&contract_path)
}

fn line_contains_bead_id(line: &str) -> bool {
    line.split(|character: char| {
        !(character.is_ascii_alphanumeric() || character == '-' || character == '.')
    })
    .any(token_is_bead_id)
}

fn token_is_bead_id(token: &str) -> bool {
    let Some(rest) = token.strip_prefix("yazelix-") else {
        return false;
    };

    let first_segment = rest.split('.').next().unwrap_or_default();
    if first_segment.is_empty()
        || !first_segment
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return false;
    }

    if rest.contains('.') {
        return (3..=6).contains(&first_segment.len());
    }

    first_segment.len() == 4
}

fn read_lines(path: &Path) -> Result<Vec<String>, String> {
    Ok(fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?
        .lines()
        .map(ToOwned::to_owned)
        .collect())
}

fn relative_to_repo(repo_root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(repo_root)
        .map_err(|error| {
            format!(
                "Failed to relativize {} against {}: {}",
                path.display(),
                repo_root.display(),
                error
            )
        })
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
}

fn push_unique<T: PartialEq>(items: &mut Vec<T>, item: T) {
    if !items.contains(&item) {
        items.push(item);
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_rust_test_fixture(relative_path: &str, content: &str) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let repo = tmp.path().to_path_buf();
        let full_path = repo.join(relative_path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(full_path, content).unwrap();
        (tmp, repo)
    }

    fn write_contract_fixture(relative_path: &str, content: &str) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let repo = tmp.path().to_path_buf();
        let full_path = repo.join(relative_path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(full_path, content).unwrap();
        (tmp, repo)
    }

    // Regression: Yazelix component repository links stay valid contract text while Bead ids remain planning-only.
    #[test]
    fn contract_validation_allows_yazelix_component_repository_names() {
        assert!(!line_contains_bead_id(
            "[`luccahuguet/yazelix-cursors`](https://github.com/luccahuguet/yazelix-cursors)"
        ));
        assert!(!line_contains_bead_id(
            "The source repository is `luccahuguet/yazelix-screen`."
        ));

        assert!(line_contains_bead_id("Bead: `yazelix-ak2d`"));
        assert!(line_contains_bead_id("Child bead `yazelix-ak2d.2`"));
        assert!(line_contains_bead_id(
            "Legacy hierarchy `yazelix-subsys.2.1`"
        ));
    }

    // Defends: canonical contracts reject issue-tracker traceability so planning state stays out.
    #[test]
    fn contract_validation_rejects_bead_traceability() {
        let source = [
            "# Example Contract",
            "",
            "## Summary",
            "This contract is intentionally tiny.",
            "",
            "## Verification",
            "",
            "- `yzx_repo_validator validate-contracts`",
            "",
            "## Traceability",
            "",
            "- Bead: `yazelix-7iye`",
        ]
        .join("\n");
        let (_tmp, repo) = write_contract_fixture("docs/contracts/example.md", &source);

        let report = validate_contracts(&repo).unwrap();
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("must not mention Beads")),
            "{:?}",
            report.errors
        );
    }

    // Defends: canonical contracts reject stale spec paths and planning markers.
    #[test]
    fn contract_validation_rejects_stale_spec_paths_and_planning_markers() {
        let source = [
            "# Example Contract",
            "",
            "## Summary",
            "Prototype Outcome: this belongs in Beads, not contracts.",
            "",
            "## Verification",
            "",
            "- stale reference: docs/specs/example.md",
        ]
        .join("\n");
        let (_tmp, repo) = write_contract_fixture("docs/contracts/example.md", &source);

        let report = validate_contracts(&repo).unwrap();
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("docs/specs")),
            "{:?}",
            report.errors
        );
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("Prototype Outcome")),
            "{:?}",
            report.errors
        );
    }

    // Regression: package-time Rust tests must not execute Nix because Nix package test sandboxes do not provide host Nix.
    #[test]
    fn package_rust_test_purity_rejects_nix_command_in_integration_test() {
        let bad_test_source = [
            "use std::process::Command;",
            "",
            "#[test]",
            "fn checks_home_manager_metadata() {",
            "    let _ = Command::new(\"nix\").arg(\"eval\").output();",
            "}",
        ]
        .join("\n");
        let (_tmp, repo) = write_rust_test_fixture(
            "rust_core/yazelix_core/tests/home_manager_option_metadata.rs",
            &bad_test_source,
        );

        let report = validate_package_rust_test_purity(&repo).unwrap();
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].contains("Command::new(\"nix\")"));
        assert!(report.errors[0].contains("maintainer validator"));
    }

    // Defends: production command execution code can still mention Nix outside the package-time test scan region.
    #[test]
    fn package_rust_test_purity_ignores_production_code_before_cfg_test_module() {
        let production_source = [
            "use std::process::Command;",
            "",
            "pub fn run_update() {",
            "    let _ = Command::new(\"nix\").arg(\"profile\").arg(\"upgrade\").output();",
            "}",
            "",
            "#[cfg(test)]",
            "mod tests {",
            "    #[test]",
            "    fn pure_unit_test() {",
            "        assert_eq!(2 + 2, 4);",
            "    }",
            "}",
        ]
        .join("\n");
        let (_tmp, repo) = write_rust_test_fixture(
            "rust_core/yazelix_core/src/update_commands.rs",
            &production_source,
        );

        let report = validate_package_rust_test_purity(&repo).unwrap();
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }
}
