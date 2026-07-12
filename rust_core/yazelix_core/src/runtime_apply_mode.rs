use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeApplyMode {
    TabSessionRestart,
    ShellTerminalRestart,
    PackageHomeManagerActivation,
}

impl RuntimeApplyMode {
    pub const ALL: &'static [RuntimeApplyMode] = &[
        RuntimeApplyMode::TabSessionRestart,
        RuntimeApplyMode::ShellTerminalRestart,
        RuntimeApplyMode::PackageHomeManagerActivation,
    ];

    pub fn code(self) -> &'static str {
        match self {
            RuntimeApplyMode::TabSessionRestart => "tab_session_restart",
            RuntimeApplyMode::ShellTerminalRestart => "shell_terminal_restart",
            RuntimeApplyMode::PackageHomeManagerActivation => "package_home_manager_activation",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RuntimeApplyMode::TabSessionRestart => "Fresh Yazelix window",
            RuntimeApplyMode::ShellTerminalRestart => "New shell or terminal",
            RuntimeApplyMode::PackageHomeManagerActivation => "After Home Manager switch",
        }
    }
}

impl fmt::Display for RuntimeApplyMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::str::FromStr for RuntimeApplyMode {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "tab_session_restart" => Ok(RuntimeApplyMode::TabSessionRestart),
            "shell_terminal_restart" => Ok(RuntimeApplyMode::ShellTerminalRestart),
            "package_home_manager_activation" => Ok(RuntimeApplyMode::PackageHomeManagerActivation),
            other => Err(format!("unsupported runtime apply mode `{other}`")),
        }
    }
}

pub fn runtime_apply_mode_codes() -> Vec<&'static str> {
    RuntimeApplyMode::ALL
        .iter()
        .map(|mode| mode.code())
        .collect()
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the config contract and UI share one closed apply-mode vocabulary from docs/contracts/runtime_applied_settings.md.
    #[test]
    fn runtime_apply_modes_parse_from_their_stable_codes() {
        for mode in RuntimeApplyMode::ALL {
            assert_eq!(mode.code().parse::<RuntimeApplyMode>(), Ok(*mode));
            assert!(!mode.label().is_empty());
        }

        assert!("restart".parse::<RuntimeApplyMode>().is_err());
    }
}
