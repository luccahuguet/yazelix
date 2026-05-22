pub(crate) fn popup_command_argv_for_yazelix_runtime(
    command: &[String],
    yzx_cli: &str,
) -> Vec<String> {
    let Some(command_path) = command.first() else {
        return Vec::new();
    };

    if command_path == yzx_cli {
        return command.to_vec();
    }

    if command_path == "yzx" {
        return std::iter::once(yzx_cli.to_string())
            .chain(command.iter().skip(1).cloned())
            .collect();
    }

    std::iter::once(yzx_cli.to_string())
        .chain(std::iter::once("run".to_string()))
        .chain(command.iter().cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;

    // Regression: yzpp popup specs launch external TUI tools through `yzx run` so nested editor flows inherit EDITOR/VISUAL.
    #[test]
    fn wraps_external_commands_through_yzx_run() {
        assert_eq!(
            popup_command_argv_for_yazelix_runtime(
                &["lazygit".to_string(), "status".to_string()],
                "/opt/yazelix/shells/posix/yzx_cli.sh",
            ),
            vec![
                "/opt/yazelix/shells/posix/yzx_cli.sh".to_string(),
                "run".to_string(),
                "lazygit".to_string(),
                "status".to_string(),
            ]
        );
    }

    // Invariant: Yazelix-owned commands route directly through the stable CLI wrapper instead of nesting `yzx run yzx`.
    #[test]
    fn routes_yzx_commands_directly_through_wrapper() {
        assert_eq!(
            popup_command_argv_for_yazelix_runtime(
                &["yzx".to_string(), "config".to_string(), "ui".to_string()],
                "/opt/yazelix/shells/posix/yzx_cli.sh",
            ),
            vec![
                "/opt/yazelix/shells/posix/yzx_cli.sh".to_string(),
                "config".to_string(),
                "ui".to_string(),
            ]
        );
    }
}
