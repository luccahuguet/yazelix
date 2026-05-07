use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeApplyMode {
    Live,
    LiveWithPaneRefresh,
    GeneratedRuntimeRefresh,
    TabSessionRestart,
    ShellTerminalRestart,
    PackageHomeManagerActivation,
    NeverLive,
}

impl RuntimeApplyMode {
    pub const ALL: &'static [RuntimeApplyMode] = &[
        RuntimeApplyMode::Live,
        RuntimeApplyMode::LiveWithPaneRefresh,
        RuntimeApplyMode::GeneratedRuntimeRefresh,
        RuntimeApplyMode::TabSessionRestart,
        RuntimeApplyMode::ShellTerminalRestart,
        RuntimeApplyMode::PackageHomeManagerActivation,
        RuntimeApplyMode::NeverLive,
    ];

    pub fn code(self) -> &'static str {
        match self {
            RuntimeApplyMode::Live => "live",
            RuntimeApplyMode::LiveWithPaneRefresh => "live_with_pane_refresh",
            RuntimeApplyMode::GeneratedRuntimeRefresh => "generated_runtime_refresh",
            RuntimeApplyMode::TabSessionRestart => "tab_session_restart",
            RuntimeApplyMode::ShellTerminalRestart => "shell_terminal_restart",
            RuntimeApplyMode::PackageHomeManagerActivation => "package_home_manager_activation",
            RuntimeApplyMode::NeverLive => "never_live",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RuntimeApplyMode::Live => "Applies now",
            RuntimeApplyMode::LiveWithPaneRefresh => "Applies after pane refresh",
            RuntimeApplyMode::GeneratedRuntimeRefresh => "Saved, refresh generated config",
            RuntimeApplyMode::TabSessionRestart => "Saved, restart this tab/session",
            RuntimeApplyMode::ShellTerminalRestart => "Saved, restart terminal/shell",
            RuntimeApplyMode::PackageHomeManagerActivation => {
                "Saved, activate package/Home Manager"
            }
            RuntimeApplyMode::NeverLive => "Not live-applicable",
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
            "live" => Ok(RuntimeApplyMode::Live),
            "live_with_pane_refresh" => Ok(RuntimeApplyMode::LiveWithPaneRefresh),
            "generated_runtime_refresh" => Ok(RuntimeApplyMode::GeneratedRuntimeRefresh),
            "tab_session_restart" => Ok(RuntimeApplyMode::TabSessionRestart),
            "shell_terminal_restart" => Ok(RuntimeApplyMode::ShellTerminalRestart),
            "package_home_manager_activation" => Ok(RuntimeApplyMode::PackageHomeManagerActivation),
            "never_live" => Ok(RuntimeApplyMode::NeverLive),
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
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=2 total=8/10
    #[test]
    fn runtime_apply_modes_parse_from_their_stable_codes() {
        for mode in RuntimeApplyMode::ALL {
            assert_eq!(mode.code().parse::<RuntimeApplyMode>(), Ok(*mode));
            assert!(!mode.label().is_empty());
        }

        assert!("restart".parse::<RuntimeApplyMode>().is_err());
    }
}
