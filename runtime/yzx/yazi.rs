use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    YA_COMMAND, YAZI_COMMAND, YAZI_SOURCE, YAZI_TESTED_VERSION,
    error::{AppError, startup},
    paths::runtime_path,
};

pub(crate) struct YaziRuntime {
    pub(crate) lookup_path: OsString,
    pub(crate) yazi: PathBuf,
    pub(crate) ya: PathBuf,
    pub(crate) version: String,
    pub(crate) warning: Option<String>,
}

impl YaziRuntime {
    pub(crate) fn resolve() -> Result<Self, AppError> {
        let lookup_path = runtime_path();
        if YAZI_SOURCE == "bundled" {
            return Ok(Self {
                lookup_path,
                yazi: YAZI_COMMAND.into(),
                ya: YA_COMMAND.into(),
                version: YAZI_TESTED_VERSION.into(),
                warning: None,
            });
        }
        if YAZI_SOURCE != "host" {
            return Err(startup(
                format!("unknown packaged Yazi source: {YAZI_SOURCE}"),
                YAZI_SOURCE,
                1,
            ));
        }

        let inherited_yazi = env::var_os("YZX_YAZI_BIN").filter(|value| !value.is_empty());
        let inherited_ya = env::var_os("YZX_YA").filter(|value| !value.is_empty());
        let (yazi_command, ya_command) = match (inherited_yazi, inherited_ya) {
            (Some(yazi), Some(ya)) => (yazi, ya),
            _ => (YAZI_COMMAND.into(), YA_COMMAND.into()),
        };
        let ((yazi, yazi_version), (ya, ya_version)) = match (
            probe_command("yazi", &yazi_command, &lookup_path),
            probe_command("ya", &ya_command, &lookup_path),
        ) {
            (Ok(yazi), Ok(ya)) => (yazi, ya),
            (yazi, ya) => {
                return Err(host_pair_error(
                    [yazi.err(), ya.err()].into_iter().flatten().collect(),
                    &lookup_path,
                ));
            }
        };
        let warning = validate_versions(&yazi_version, &ya_version, YAZI_TESTED_VERSION)
            .map_err(|error| host_pair_error(vec![error], &lookup_path))?;

        Ok(Self {
            lookup_path,
            yazi,
            ya,
            version: yazi_version,
            warning,
        })
    }

    pub(crate) fn warn(&self) {
        if let Some(warning) = &self.warning {
            eprintln!("warn yazi compatibility: {warning}");
        }
    }
}

fn probe_command(
    label: &str,
    command: &OsStr,
    lookup_path: &OsStr,
) -> Result<(PathBuf, String), String> {
    let path =
        resolve_command(command, lookup_path).map_err(|error| format!("{label}: {error}"))?;
    let output = Command::new(&path)
        .arg("--version")
        .env("PATH", lookup_path)
        .output()
        .map_err(|error| {
            format!(
                "{label}: failed to run {} --version: {error}",
                path.display()
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "{label}: {} --version failed with status {}",
            path.display(),
            output.status.code().unwrap_or(1)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = if stdout.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };
    let version = parse_version(label, text).ok_or_else(|| {
        format!(
            "{label}: could not parse {} --version output: {text}",
            path.display()
        )
    })?;
    Ok((path, version))
}

fn resolve_command(command: &OsStr, lookup_path: &OsStr) -> Result<PathBuf, String> {
    let candidate = if command.as_encoded_bytes().contains(&b'/') {
        PathBuf::from(command)
    } else {
        env::split_paths(lookup_path)
            .map(|directory| directory.join(command))
            .find(|candidate| executable_file(candidate))
            .ok_or_else(|| format!("command not found in PATH: {}", command.to_string_lossy()))?
    };
    if !executable_file(&candidate) {
        return Err(format!(
            "command is not executable: {}",
            candidate.display()
        ));
    }
    fs::canonicalize(&candidate)
        .map_err(|error| format!("could not resolve {}: {error}", candidate.display()))
}

fn executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

fn parse_version(label: &str, output: &str) -> Option<String> {
    let mut fields = output.split_whitespace();
    let program = fields.next()?;
    if !program.eq_ignore_ascii_case(label) {
        return None;
    }
    let version = fields.next()?.trim_start_matches('v');
    (!version.is_empty() && version.contains('.')).then(|| version.to_string())
}

fn validate_versions(yazi: &str, ya: &str, tested: &str) -> Result<Option<String>, String> {
    if yazi != ya {
        return Err(format!(
            "yazi {yazi} and ya {ya} differ; Yazi requires an exactly matching pair"
        ));
    }
    Ok((yazi != tested).then(|| {
        format!(
            "host yazi/ya {yazi} differs from Nova's tested {tested}; continuing with the host pair"
        )
    }))
}

fn host_pair_error(failures: Vec<String>, lookup_path: &OsStr) -> AppError {
    startup(
        format!("host Yazi pair is invalid:\n{}", failures.join("\n")),
        format!("PATH={}", lookup_path.to_string_lossy()),
        1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn parses_upstream_and_distribution_version_output() {
        assert_eq!(
            parse_version("yazi", "Yazi 26.5.6 (Nixpkgs 2026-05-05)"),
            Some("26.5.6".into())
        );
        assert_eq!(parse_version("ya", "Ya v26.5.6"), Some("26.5.6".into()));
        assert_eq!(parse_version("ya", "Yazi 26.5.6"), None);
        assert_eq!(parse_version("yazi", "Yazi unknown"), None);
    }

    #[test]
    fn rejects_mixed_pairs_and_warns_for_matching_untested_pairs() {
        assert_eq!(validate_versions("26.5.6", "26.5.6", "26.5.6"), Ok(None));
        let error = validate_versions("26.6.1", "26.5.6", "26.5.6").unwrap_err();
        assert!(error.contains("exactly matching pair"), "{error}");
        assert_eq!(
            validate_versions("26.6.1", "26.6.1", "26.5.6"),
            Ok(Some(
                "host yazi/ya 26.6.1 differs from Nova's tested 26.5.6; continuing with the host pair"
                    .into()
            ))
        );
    }

    #[test]
    fn resolves_and_validates_commands() {
        let root = env::temp_dir().join(format!("yzx-yazi-resolve-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let first = root.join("first");
        let second = root.join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let executable = env::current_exe().unwrap();
        let invalid_executable = second.join("yazi");
        fs::write(&invalid_executable, "not an executable format").unwrap();
        fs::set_permissions(&invalid_executable, fs::Permissions::from_mode(0o755)).unwrap();
        symlink(&executable, first.join("yazi")).unwrap();
        let path = env::join_paths([&first, &second]).unwrap();

        assert_eq!(
            resolve_command(OsStr::new("yazi"), &path).unwrap(),
            fs::canonicalize(executable).unwrap()
        );
        fs::set_permissions(&invalid_executable, fs::Permissions::from_mode(0o644)).unwrap();
        let error = resolve_command(invalid_executable.as_os_str(), &path).unwrap_err();
        assert!(error.contains("command is not executable"), "{error}");
        fs::set_permissions(&invalid_executable, fs::Permissions::from_mode(0o755)).unwrap();
        let error = probe_command("yazi", invalid_executable.as_os_str(), &path).unwrap_err();
        assert!(error.contains("failed to run"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }
}
