use std::{fmt::Display, path::Path};

pub(crate) enum AppError {
    Usage(String),
    Startup {
        reason: String,
        check: String,
        status: i32,
    },
}

pub(crate) fn startup(reason: impl Into<String>, check: impl Display, status: i32) -> AppError {
    AppError::Startup {
        reason: reason.into(),
        check: check.to_string(),
        status,
    }
}

pub(crate) fn path_error(action: &str, path: &Path, check: &Path, error: impl Display) -> AppError {
    startup(
        format!("failed to {action} {}: {error}", path.display()),
        check.display(),
        1,
    )
}

impl AppError {
    pub(crate) fn report(self) -> i32 {
        match self {
            Self::Usage(message) => {
                eprint!("{message}");
                64
            }
            Self::Startup {
                reason,
                check,
                status,
            } => {
                eprintln!(
                    "Yazelix could not start.
"
                );
                eprintln!("Reason:");
                for line in reason.lines() {
                    eprintln!("  {line}");
                }
                if !check.is_empty() {
                    eprintln!(
                        "
Check:
  {check}"
                    );
                }
                status
            }
        }
    }
}
