use std::{
    fs,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::Path,
    process::{Command, Output},
};

use crate::error::{AppError, path_error, startup};

pub(crate) fn exec(mut command: Command, check: &str) -> Result<(), AppError> {
    Err(startup(
        format!("failed to exec {check}: {}", command.exec()),
        check,
        1,
    ))
}

pub(crate) fn run_checked(check: &Path, command: &mut Command) -> Result<String, AppError> {
    match command.output() {
        Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout).into()),
        Ok(output) => Err(startup(
            output_reason(&output).unwrap_or_else(|| {
                format!(
                    "{} failed with status {}",
                    command.get_program().to_string_lossy(),
                    output.status.code().unwrap_or(1)
                )
            }),
            check.display(),
            output.status.code().unwrap_or(1),
        )),
        Err(error) => Err(startup(
            format!(
                "failed to run {}: {error}",
                command.get_program().to_string_lossy()
            ),
            check.display(),
            1,
        )),
    }
}

pub(crate) fn executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

fn output_reason(output: &Output) -> Option<String> {
    let trimmed = trim_output(format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ));
    (!trimmed.is_empty()).then_some(trimmed)
}

pub(crate) fn create_dir_all_checked(path: &Path, check: &Path) -> Result<(), AppError> {
    fs::create_dir_all(path).map_err(|error| path_error("create", path, check, error))
}

pub(crate) fn trim_output(text: String) -> String {
    text.trim_end_matches(['\n', '\r']).to_owned()
}
