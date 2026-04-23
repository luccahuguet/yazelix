use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const POLICY_ONLY_SPEC_PATHS: &[&str] = &["docs/specs/test_suite_governance.md"];
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

#[derive(Debug, Default)]
pub struct ValidationReport {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ContractItem {
    pub id: String,
    pub spec: String,
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

pub fn validate_specs(repo_root: &Path) -> Result<ValidationReport, String> {
    let spec_files = load_spec_files(repo_root)?;
    if spec_files.is_empty() {
        return Ok(ValidationReport::default());
    }

    let bead_ids = load_bead_ids(repo_root)?;
    let contract_items = load_contract_items(repo_root)?;
    let mut report = ValidationReport::default();
    let mut seen_ids: HashMap<String, String> = HashMap::new();

    for spec_path in &spec_files {
        report
            .errors
            .extend(validate_spec_file(repo_root, spec_path, &bead_ids)?);
    }

    for item in &contract_items {
        if let Some(existing_spec) = seen_ids.get(&item.id) {
            report.errors.push(format!(
                "Duplicate contract item id `{}` appears in both {} and {}",
                item.id, existing_spec, item.spec
            ));
        } else {
            seen_ids.insert(item.id.clone(), item.spec.clone());
        }

        report.errors.extend(validate_contract_item(item));
    }

    Ok(report)
}

pub fn validate_default_test_traceability(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();

    for test_path in load_all_nu_test_file_paths(repo_root)? {
        let relative_path = relative_to_repo(repo_root, &test_path)?;
        report.errors.push(format!(
            "Governed Nushell test files are no longer part of the canonical suite; port strong tests to Rust nextest or demote shell-heavy probes out of the test_*.nu namespace: {}",
            relative_path
        ));
    }

    Ok(report)
}

pub fn validate_rust_test_traceability(repo_root: &Path) -> Result<ValidationReport, String> {
    let contract_items = load_contract_items(repo_root)?;
    let mut report = ValidationReport::default();

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

        let minimum_strength =
            minimum_strength_for_lane(&lane).ok_or_else(|| format!("Unknown lane: {lane}"))?;
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

            let strength = get_rust_definition_test_strength(
                repo_root,
                &relative_path,
                test_record.attribute_index,
                &test_record.test_name,
            )?;
            if strength < minimum_strength {
                report.errors.push(format!(
                    "Governed Rust test is below the minimum strength bar of {}/10 for lane '{}': {} :: {} :: {}/10",
                    minimum_strength, lane, relative_path, test_record.test_name, strength
                ));
            }
        }
    }

    Ok(report)
}

fn validate_spec_file(
    repo_root: &Path,
    spec_path: &Path,
    bead_ids: &HashSet<String>,
) -> Result<Vec<String>, String> {
    let relative_path = relative_to_repo(repo_root, spec_path)?;
    let content = fs::read_to_string(spec_path).map_err(|error| {
        format!(
            "Failed to read spec file {}: {}",
            spec_path.display(),
            error
        )
    })?;
    let traceability = get_traceability_section(&content);

    if traceability.trim().is_empty() {
        return Ok(vec![format!(
            "{}: missing `## Traceability` section",
            relative_path
        )]);
    }

    let bead_match = traceability
        .lines()
        .map(str::trim)
        .find_map(parse_traceability_bead_line);
    let defended_by_count = traceability
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("- Defended by:"))
        .count();

    let mut errors = Vec::new();

    match bead_match {
        None => errors.push(format!(
            "{}: missing valid `- Bead: ` traceability entry",
            relative_path
        )),
        Some(bead_id) if !bead_ids.contains(&bead_id) => errors.push(format!(
            "{}: traceability bead `{}` does not exist in .beads/issues.jsonl",
            relative_path, bead_id
        )),
        Some(_) => {}
    }

    if defended_by_count == 0 {
        errors.push(format!(
            "{}: expected at least one `- Defended by:` traceability entry",
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
            item.spec, item.id
        )),
        Some(item_type) if !ALLOWED_CONTRACT_TYPES.contains(&item_type) => errors.push(format!(
            "{}: contract item `{}` declares unsupported type `{}`",
            item.spec, item.id, item_type
        )),
        Some(_) => {}
    }

    let Some(status) = item.status.as_deref() else {
        errors.push(format!(
            "{}: contract item `{}` is missing `- Status:`",
            item.spec, item.id
        ));
        return errors;
    };

    if !ALLOWED_CONTRACT_STATUSES.contains(&status) {
        errors.push(format!(
            "{}: contract item `{}` declares unsupported status `{}`",
            item.spec, item.id, status
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
                    item.spec, item.id, label
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
                item.spec, item.id
            ));
        }
    }

    if status == "live" && item.verification.is_none() {
        errors.push(format!(
            "{}: live contract item `{}` must name a verification path or explicit manual/unverified reason",
            item.spec, item.id
        ));
    }

    errors
}

fn load_bead_ids(repo_root: &Path) -> Result<HashSet<String>, String> {
    let issues_path = repo_root.join(".beads").join("issues.jsonl");
    if !issues_path.exists() {
        return Ok(HashSet::new());
    }

    let mut ids = HashSet::new();
    let content = fs::read_to_string(&issues_path).map_err(|error| {
        format!(
            "Failed to read bead export {}: {}",
            issues_path.display(),
            error
        )
    })?;

    for line in content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let value: JsonValue = serde_json::from_str(line)
            .map_err(|error| format!("Invalid issues.jsonl line: {error}"))?;
        if let Some(id) = value.get("id").and_then(JsonValue::as_str) {
            ids.insert(id.to_string());
        }
    }

    Ok(ids)
}

fn load_contract_items(repo_root: &Path) -> Result<Vec<ContractItem>, String> {
    let mut items = Vec::new();
    for spec_path in load_spec_files(repo_root)? {
        let relative_path = relative_to_repo(repo_root, &spec_path)?;
        let lines = read_lines(&spec_path)?;
        let mut current: Option<ContractItem> = None;

        for line in lines {
            let trimmed = line.trim();
            if let Some(id) = parse_contract_heading(trimmed) {
                if let Some(item) = current.take() {
                    items.push(item);
                }
                current = Some(ContractItem {
                    id,
                    spec: relative_path.clone(),
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

fn load_spec_files(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let specs_dir = repo_root.join("docs").join("specs");
    for entry in fs::read_dir(&specs_dir).map_err(|error| {
        format!(
            "Failed to read specs directory {}: {}",
            specs_dir.display(),
            error
        )
    })? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md")
            && path.file_name().and_then(|name| name.to_str()) != Some("template.md")
        {
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
    let defends_spec_paths = load_rust_definition_defends_spec_paths(
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
            "Governed Rust test cannot rely only on `docs/specs/test_suite_governance.md` as nearby traceability: {} :: {}",
            relative_path, test_record.test_name
        ));
    }

    if lane == "default"
        && contract_ids.is_empty()
        && !has_regression_or_invariant
        && defends_spec_paths
            .iter()
            .any(|spec_path| spec_has_contract_items(contract_items, spec_path))
    {
        errors.push(format!(
            "Default-lane Rust test defends a spec with indexed contract items but is missing a nearby '// Contract:' marker: {} :: {}",
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

fn get_rust_definition_test_strength(
    repo_root: &Path,
    relative_path: &str,
    attribute_index: usize,
    test_name: &str,
) -> Result<u32, String> {
    let strength_line = get_prior_nonempty_lines_before_index(
        repo_root,
        relative_path,
        attribute_index,
    )?
    .into_iter()
    .find(|line| line.starts_with("// Strength:"))
    .ok_or_else(|| {
        format!(
            "Governed Rust test is missing a nearby structured '// Strength:' marker: {} :: {}",
            relative_path, test_name
        )
    })?;

    parse_structured_strength_line(relative_path, test_name, &strength_line, "// Strength:")
}

fn rust_definition_has_policy_only_traceability(
    repo_root: &Path,
    relative_path: &str,
    attribute_index: usize,
) -> Result<bool, String> {
    let spec_paths =
        load_rust_definition_defends_spec_paths(repo_root, relative_path, attribute_index)?;
    if spec_paths.is_empty() {
        return Ok(false);
    }
    if has_rust_definition_regression_or_invariant(repo_root, relative_path, attribute_index)? {
        return Ok(false);
    }
    Ok(spec_paths
        .iter()
        .all(|spec_path| is_policy_only_spec_path(spec_path)))
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

fn load_rust_definition_defends_spec_paths(
    repo_root: &Path,
    relative_path: &str,
    attribute_index: usize,
) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for line in load_rust_definition_traceability_lines(repo_root, relative_path, attribute_index)?
    {
        if let Some(spec_path) = parse_rust_defends_spec_path(repo_root, &line) {
            push_unique(&mut paths, spec_path);
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

fn get_traceability_section(content: &str) -> String {
    let mut in_section = false;
    let mut body = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_end();
        if trimmed == "## Traceability" {
            in_section = true;
            continue;
        }
        if in_section && trimmed.starts_with("## ") {
            break;
        }
        if in_section {
            body.push(trimmed);
        }
    }
    body.join("\n")
}

fn parse_traceability_bead_line(line: &str) -> Option<String> {
    let payload = line.strip_prefix("- Bead:")?.trim();
    let bead_id = payload.strip_prefix('`')?.strip_suffix('`')?;
    if bead_id.is_empty() {
        None
    } else {
        Some(bead_id.to_string())
    }
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

fn parse_rust_defends_spec_path(repo_root: &Path, line: &str) -> Option<String> {
    parse_defends_spec_path(repo_root, line, "// Defends:")
}

fn parse_defends_spec_path(repo_root: &Path, line: &str, marker: &str) -> Option<String> {
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

fn parse_structured_strength_line(
    relative_path: &str,
    test_name: &str,
    strength_line: &str,
    marker: &str,
) -> Result<u32, String> {
    let payload = strength_line
        .trim()
        .strip_prefix(marker)
        .ok_or_else(|| {
            format!(
                "Could not parse structured strength marker near: {} :: {} :: {}",
                relative_path, test_name, strength_line
            )
        })?
        .trim();
    let mut values = HashMap::new();
    for token in payload.split_whitespace() {
        if let Some((key, value)) = token.split_once('=') {
            values.insert(key, value);
        }
    }

    let defect =
        parse_strength_component(relative_path, test_name, strength_line, &values, "defect")?;
    let behavior =
        parse_strength_component(relative_path, test_name, strength_line, &values, "behavior")?;
    let resilience = parse_strength_component(
        relative_path,
        test_name,
        strength_line,
        &values,
        "resilience",
    )?;
    let cost = parse_strength_component(relative_path, test_name, strength_line, &values, "cost")?;
    let uniqueness = parse_strength_component(
        relative_path,
        test_name,
        strength_line,
        &values,
        "uniqueness",
    )?;
    let total_token = values.get("total").copied().ok_or_else(|| {
        format!(
            "Could not parse structured strength marker near: {} :: {} :: {}",
            relative_path, test_name, strength_line
        )
    })?;
    let total = total_token
        .strip_suffix("/10")
        .ok_or_else(|| {
            format!(
                "Could not parse structured strength marker near: {} :: {} :: {}",
                relative_path, test_name, strength_line
            )
        })?
        .parse::<u32>()
        .map_err(|_| {
            format!(
                "Could not parse structured strength marker near: {} :: {} :: {}",
                relative_path, test_name, strength_line
            )
        })?;
    let computed = defect + behavior + resilience + cost + uniqueness;
    if computed != total {
        return Err(format!(
            "Structured strength marker total does not match component sum near: {} :: {} :: expected={}/10 declared={}/10",
            relative_path, test_name, computed, total
        ));
    }
    Ok(total)
}

fn parse_strength_component(
    relative_path: &str,
    test_name: &str,
    strength_line: &str,
    values: &HashMap<&str, &str>,
    key: &str,
) -> Result<u32, String> {
    let raw = values.get(key).copied().ok_or_else(|| {
        format!(
            "Could not parse structured strength marker near: {} :: {} :: {}",
            relative_path, test_name, strength_line
        )
    })?;
    let parsed = raw.parse::<u32>().map_err(|_| {
        format!(
            "Could not parse structured strength marker near: {} :: {} :: {}",
            relative_path, test_name, strength_line
        )
    })?;
    if parsed > 2 {
        return Err(format!(
            "Could not parse structured strength marker near: {} :: {} :: {}",
            relative_path, test_name, strength_line
        ));
    }
    Ok(parsed)
}

fn minimum_strength_for_lane(lane: &str) -> Option<u32> {
    match lane {
        "default" => Some(7),
        "maintainer" => Some(6),
        "sweep" => Some(6),
        "manual" => Some(6),
        _ => None,
    }
}

fn find_contract_item<'a>(
    contract_items: &'a [ContractItem],
    contract_id: &str,
) -> Option<&'a ContractItem> {
    contract_items.iter().find(|item| item.id == contract_id)
}

fn spec_has_contract_items(contract_items: &[ContractItem], spec_path: &str) -> bool {
    contract_items.iter().any(|item| item.spec == spec_path)
}

fn is_policy_only_spec_path(spec_path: &str) -> bool {
    POLICY_ONLY_SPEC_PATHS.contains(&spec_path)
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
