use serde::Serialize;
use serde_json::{Value, json};
use std::io;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    Usage,
    Config,
    Io,
    Runtime,
    Internal,
}

impl ErrorClass {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorClass::Usage => "usage",
            ErrorClass::Config => "config",
            ErrorClass::Io => "io",
            ErrorClass::Runtime => "runtime",
            ErrorClass::Internal => "internal",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            ErrorClass::Usage => 64,
            ErrorClass::Config => 65,
            ErrorClass::Io => 66,
            ErrorClass::Runtime | ErrorClass::Internal => 70,
        }
    }
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("{message}")]
    Classified {
        class: ErrorClass,
        code: String,
        message: String,
        remediation: String,
        details: Value,
    },
    #[error("{message}: {source}")]
    Io {
        code: String,
        message: String,
        remediation: String,
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("{message}: {source}")]
    Toml {
        code: String,
        message: String,
        remediation: String,
        path: String,
        #[source]
        source: Box<toml::de::Error>,
    },
}

impl CoreError {
    pub fn classified(
        class: ErrorClass,
        code: impl Into<String>,
        message: impl Into<String>,
        remediation: impl Into<String>,
        details: Value,
    ) -> Self {
        Self::Classified {
            class,
            code: code.into(),
            message: message.into(),
            remediation: remediation.into(),
            details,
        }
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Self::classified(
            ErrorClass::Usage,
            "invalid_arguments",
            message,
            "Run the helper with a supported command and required flags.",
            json!({}),
        )
    }

    pub fn io(
        code: impl Into<String>,
        message: impl Into<String>,
        remediation: impl Into<String>,
        path: impl Into<String>,
        source: io::Error,
    ) -> Self {
        Self::Io {
            code: code.into(),
            message: message.into(),
            remediation: remediation.into(),
            path: path.into(),
            source,
        }
    }

    pub fn toml(
        code: impl Into<String>,
        message: impl Into<String>,
        remediation: impl Into<String>,
        path: impl Into<String>,
        source: toml::de::Error,
    ) -> Self {
        Self::Toml {
            code: code.into(),
            message: message.into(),
            remediation: remediation.into(),
            path: path.into(),
            source: Box::new(source),
        }
    }

    pub fn class(&self) -> ErrorClass {
        match self {
            Self::Classified { class, .. } => *class,
            Self::Io { .. } => ErrorClass::Io,
            Self::Toml { .. } => ErrorClass::Config,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            Self::Classified { code, .. } => code,
            Self::Io { code, .. } => code,
            Self::Toml { code, .. } => code,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Classified { message, .. } => message.clone(),
            Self::Io {
                message, source, ..
            } => format!("{message}: {source}"),
            Self::Toml {
                message, source, ..
            } => format!("{message}: {source}"),
        }
    }

    pub fn remediation(&self) -> String {
        match self {
            Self::Classified { remediation, .. } => remediation.clone(),
            Self::Io { remediation, .. } => remediation.clone(),
            Self::Toml { remediation, .. } => remediation.clone(),
        }
    }

    pub fn details(&self) -> Value {
        match self {
            Self::Classified { details, .. } => details.clone(),
            Self::Io { path, .. } | Self::Toml { path, .. } => json!({ "path": path }),
        }
    }
}

#[derive(Serialize)]
pub struct SuccessEnvelope<T: Serialize> {
    schema_version: u8,
    command: String,
    status: &'static str,
    data: T,
    warnings: Vec<Value>,
}

#[derive(Serialize)]
pub struct ErrorEnvelope {
    schema_version: u8,
    command: String,
    status: &'static str,
    error: ErrorBody,
}

#[derive(Serialize)]
pub struct ErrorBody {
    class: String,
    code: String,
    message: String,
    remediation: String,
    details: Value,
}

pub fn success_envelope<T: Serialize>(command: impl Into<String>, data: T) -> SuccessEnvelope<T> {
    SuccessEnvelope {
        schema_version: 1,
        command: command.into(),
        status: "ok",
        data,
        warnings: Vec::new(),
    }
}

pub fn error_envelope(command: impl Into<String>, error: &CoreError) -> ErrorEnvelope {
    ErrorEnvelope {
        schema_version: 1,
        command: command.into(),
        status: "error",
        error: ErrorBody {
            class: error.class().as_str().to_string(),
            code: error.code().to_string(),
            message: error.message(),
            remediation: error.remediation(),
            details: error.details(),
        },
    }
}
