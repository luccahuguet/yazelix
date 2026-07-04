use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Output},
};

use crate::error::{path_error, startup, AppError};

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

pub(crate) fn touch_checked(path: &Path) -> Result<(), AppError> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map(|_| ())
        .map_err(|error| path_error("create", path, path, error))
}

pub(crate) fn seed_permission_checked(
    path: &Path,
    plugin: &str,
    permissions: &[&str],
) -> Result<(), AppError> {
    let current =
        fs::read_to_string(path).map_err(|error| path_error("read", path, path, error))?;
    if current.contains(&format!("\"{plugin}\" {{")) {
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|error| path_error("open", path, path, error))?;
    writeln!(
        file,
        "\"{plugin}\" {{\n    {}\n}}",
        permissions.join("\n    ")
    )
    .map_err(|error| path_error("write", path, path, error))
}

pub(crate) fn trim_output(text: String) -> String {
    text.trim_end_matches(['\n', '\r']).to_owned()
}
