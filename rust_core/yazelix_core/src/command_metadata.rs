use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YzxCommandCategory {
    Config,
    Development,
    Help,
    Integration,
    Session,
    System,
    Workspace,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YzxParameterKind {
    Switch,
    Named,
    Positional,
    Rest,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct YzxCommandParameter {
    pub kind: YzxParameterKind,
    pub name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<&'static str>,
    pub shape: &'static str,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct YzxCommandMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub category: YzxCommandCategory,
    pub parameters: &'static [YzxCommandParameter],
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct YzxCommandMetadataData {
    pub commands: Vec<YzxCommandMetadata>,
    pub extern_content: String,
}

const VERSION_FLAGS: &[YzxCommandParameter] = &[
    switch("version", Some("V")),
    switch("version-short", Some("v")),
];
const ENV_FLAGS: &[YzxCommandParameter] = &[switch("no-shell", Some("n"))];
const RUN_REST: &[YzxCommandParameter] = &[rest("argv")];
const LAUNCH_FLAGS: &[YzxCommandParameter] = &[
    named("path", Some("p"), "string", true),
    switch("home", None),
    named("terminal", Some("t"), "string", true),
    switch("verbose", None),
];
const ENTER_FLAGS: &[YzxCommandParameter] = &[
    named("path", Some("p"), "string", true),
    switch("home", None),
    switch("verbose", None),
];
const UPDATE_NIX_FLAGS: &[YzxCommandParameter] = &[switch("yes", None), switch("verbose", None)];
const CWD_ARGS: &[YzxCommandParameter] = &[positional("target", "string", true)];
const REVEAL_ARGS: &[YzxCommandParameter] = &[positional("target", "string", false)];
const STATUS_FLAGS: &[YzxCommandParameter] = &[switch("versions", Some("V")), switch("json", None)];
const DOCTOR_FLAGS: &[YzxCommandParameter] = &[
    switch("verbose", Some("v")),
    switch("fix", Some("f")),
    switch("json", None),
];
const CONFIG_RESET_FLAGS: &[YzxCommandParameter] = &[switch("force", None)];
const IMPORT_FLAGS: &[YzxCommandParameter] = &[switch("force", None)];
const EDIT_ARGS: &[YzxCommandParameter] = &[rest("query"), switch("print", None)];
const EDIT_CONFIG_FLAGS: &[YzxCommandParameter] = &[switch("print", None)];
const POPUP_ARGS: &[YzxCommandParameter] = &[rest("program")];
const SCREEN_ARGS: &[YzxCommandParameter] = &[positional("style", "string", true)];
const DEV_UPDATE_FLAGS: &[YzxCommandParameter] = &[
    switch("yes", None),
    switch("no-canary", None),
    named("activate", None, "string", true),
    named("home-manager-dir", None, "string", true),
    named("home-manager-input", None, "string", true),
    named("home-manager-attr", None, "string", true),
    switch("canary-only", None),
    named("canaries", None, "string", true),
];
const DEV_BUMP_ARGS: &[YzxCommandParameter] = &[positional("version", "string", false)];
const DEV_SYNC_FLAGS: &[YzxCommandParameter] = &[switch("dry-run", None)];
const DEV_BUILD_FLAGS: &[YzxCommandParameter] = &[switch("sync", None)];
const DEV_TEST_FLAGS: &[YzxCommandParameter] = &[
    switch("verbose", Some("v")),
    switch("new-window", Some("n")),
    switch("lint-only", None),
    switch("profile", None),
    switch("sweep", None),
    switch("visual", None),
    switch("all", Some("a")),
    named("delay", None, "int", true),
];
const DEV_PROFILE_FLAGS: &[YzxCommandParameter] = &[
    switch("cold", Some("c")),
    switch("desktop", None),
    switch("launch", None),
    switch("clear-cache", None),
    named("terminal", Some("t"), "string", true),
    switch("verbose", None),
];
const DEV_LINT_ARGS: &[YzxCommandParameter] =
    &[named("format", Some("f"), "string", true), rest("paths")];
const HM_PREPARE_FLAGS: &[YzxCommandParameter] = &[switch("apply", None), switch("yes", None)];

pub fn yzx_command_metadata() -> Vec<YzxCommandMetadata> {
    let mut commands = vec![
        cmd(
            "yzx",
            "Show Yazelix help or version information",
            YzxCommandCategory::Help,
            VERSION_FLAGS,
        ),
        cmd(
            "yzx config",
            "Show the active Yazelix configuration",
            YzxCommandCategory::Config,
            &[],
        ),
        cmd(
            "yzx config reset",
            "Replace the main Yazelix config with a fresh shipped template",
            YzxCommandCategory::Config,
            CONFIG_RESET_FLAGS,
        ),
        cmd(
            "yzx cwd",
            "Retarget the current Yazelix tab workspace directory",
            YzxCommandCategory::Workspace,
            CWD_ARGS,
        ),
        cmd(
            "yzx desktop install",
            "Install the user-local Yazelix desktop entry and icons",
            YzxCommandCategory::Integration,
            &[],
        ),
        cmd(
            "yzx desktop launch",
            "Launch Yazelix from the desktop entry fast path",
            YzxCommandCategory::Integration,
            &[],
        ),
        cmd(
            "yzx desktop uninstall",
            "Remove the user-local Yazelix desktop entry and icons",
            YzxCommandCategory::Integration,
            &[],
        ),
        cmd(
            "yzx dev",
            "Development and maintainer commands",
            YzxCommandCategory::Development,
            &[],
        ),
        cmd(
            "yzx dev build_pane_orchestrator",
            "Build the Zellij pane-orchestrator wasm",
            YzxCommandCategory::Development,
            DEV_BUILD_FLAGS,
        ),
        cmd(
            "yzx dev bump",
            "Bump the tracked Yazelix version and create release metadata",
            YzxCommandCategory::Development,
            DEV_BUMP_ARGS,
        ),
        cmd(
            "yzx dev lint_nu",
            "Lint Nushell scripts with repo-tuned nu-lint config",
            YzxCommandCategory::Development,
            DEV_LINT_ARGS,
        ),
        cmd(
            "yzx dev profile",
            "Profile launch sequence and identify bottlenecks",
            YzxCommandCategory::Development,
            DEV_PROFILE_FLAGS,
        ),
        cmd(
            "yzx dev sync_issues",
            "Sync GitHub issue lifecycle into Beads locally",
            YzxCommandCategory::Development,
            DEV_SYNC_FLAGS,
        ),
        cmd(
            "yzx dev test",
            "Run Yazelix test suite",
            YzxCommandCategory::Development,
            DEV_TEST_FLAGS,
        ),
        cmd(
            "yzx dev update",
            "Refresh maintainer flake inputs and run update canaries",
            YzxCommandCategory::Development,
            DEV_UPDATE_FLAGS,
        ),
        cmd(
            "yzx doctor",
            "Run health checks and diagnostics",
            YzxCommandCategory::System,
            DOCTOR_FLAGS,
        ),
        cmd(
            "yzx edit",
            "Open a Yazelix-managed config surface in the configured editor",
            YzxCommandCategory::Config,
            EDIT_ARGS,
        ),
        cmd(
            "yzx edit config",
            "Open the main Yazelix config in the configured editor",
            YzxCommandCategory::Config,
            EDIT_CONFIG_FLAGS,
        ),
        cmd(
            "yzx enter",
            "Start Yazelix in the current terminal",
            YzxCommandCategory::Session,
            ENTER_FLAGS,
        ),
        cmd(
            "yzx env",
            "Load the Yazelix environment without UI",
            YzxCommandCategory::Session,
            ENV_FLAGS,
        ),
        cmd(
            "yzx home_manager",
            "Show Yazelix Home Manager takeover helpers",
            YzxCommandCategory::Integration,
            &[],
        ),
        cmd(
            "yzx home_manager prepare",
            "Preview or archive manual-install artifacts before Home Manager takeover",
            YzxCommandCategory::Integration,
            HM_PREPARE_FLAGS,
        ),
        cmd(
            "yzx import",
            "Import native config files into Yazelix-managed override paths",
            YzxCommandCategory::Config,
            &[],
        ),
        cmd(
            "yzx import helix",
            "Import the native Helix config into Yazelix-managed overrides",
            YzxCommandCategory::Config,
            IMPORT_FLAGS,
        ),
        cmd(
            "yzx import yazi",
            "Import native Yazi config files into Yazelix-managed overrides",
            YzxCommandCategory::Config,
            IMPORT_FLAGS,
        ),
        cmd(
            "yzx import zellij",
            "Import the native Zellij config into Yazelix-managed overrides",
            YzxCommandCategory::Config,
            IMPORT_FLAGS,
        ),
        cmd(
            "yzx keys",
            "Show Yazelix-owned keybindings and remaps",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys helix",
            "Alias for yzx keys hx",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys hx",
            "Explain how to discover Helix keybindings and commands",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys nu",
            "Show a small curated subset of useful Nushell keybindings",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys nushell",
            "Alias for yzx keys nu",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys yazi",
            "Explain how to view Yazi's built-in keybindings",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys yzx",
            "Alias for the default Yazelix keybinding view",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx launch",
            "Launch Yazelix",
            YzxCommandCategory::Session,
            LAUNCH_FLAGS,
        ),
        cmd(
            "yzx menu",
            "Interactive command palette for Yazelix",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx popup",
            "Open or toggle the configured Yazelix popup program in Zellij",
            YzxCommandCategory::Workspace,
            POPUP_ARGS,
        ),
        cmd(
            "yzx restart",
            "Restart Yazelix",
            YzxCommandCategory::Session,
            &[],
        ),
        cmd(
            "yzx reveal",
            "Reveal a file or directory in the managed Yazi sidebar",
            YzxCommandCategory::Workspace,
            REVEAL_ARGS,
        ),
        cmd(
            "yzx run",
            "Run a command in the Yazelix environment and exit",
            YzxCommandCategory::Session,
            RUN_REST,
        ),
        cmd(
            "yzx screen",
            "Show an animated Yazelix full-terminal screen",
            YzxCommandCategory::Workspace,
            SCREEN_ARGS,
        ),
        cmd(
            "yzx sponsor",
            "Open the Yazelix sponsor page or print its URL",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx status",
            "Canonical inspection command",
            YzxCommandCategory::System,
            STATUS_FLAGS,
        ),
        cmd(
            "yzx tutor",
            "Show the Yazelix guided overview",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx tutor helix",
            "Alias for yzx tutor hx",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx tutor hx",
            "Launch Helix's built-in tutorial",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx tutor nu",
            "Launch Nushell's built-in tutorial in a fresh Nushell process",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx tutor nushell",
            "Alias for yzx tutor nu",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx update",
            "Show supported update owners",
            YzxCommandCategory::System,
            &[],
        ),
        cmd(
            "yzx update home_manager",
            "Refresh the current Home Manager flake input for Yazelix",
            YzxCommandCategory::System,
            &[],
        ),
        cmd(
            "yzx update nix",
            "Upgrade Determinate Nix through determinate-nixd",
            YzxCommandCategory::System,
            UPDATE_NIX_FLAGS,
        ),
        cmd(
            "yzx update upstream",
            "Upgrade the active Yazelix package in the default Nix profile",
            YzxCommandCategory::System,
            &[],
        ),
        cmd(
            "yzx whats_new",
            "Show the current Yazelix upgrade summary",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx why",
            "Elevator pitch: Why Yazelix",
            YzxCommandCategory::Help,
            &[],
        ),
    ];
    commands.sort_by(|left, right| left.name.cmp(right.name));
    commands
}

pub fn yzx_command_metadata_data() -> YzxCommandMetadataData {
    let commands = yzx_command_metadata();
    let extern_content = render_yzx_externs(&commands);
    YzxCommandMetadataData {
        commands,
        extern_content,
    }
}

pub fn render_yzx_help(commands: &[YzxCommandMetadata]) -> String {
    let width = commands
        .iter()
        .map(|command| command.name.len())
        .max()
        .unwrap_or(3);
    let rows = commands
        .iter()
        .map(|command| {
            format!(
                "  {:width$}  {}",
                command.name,
                command.description,
                width = width
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    [
        "Show Yazelix help or version information".to_string(),
        String::new(),
        "Usage:".to_string(),
        "  yzx [--version]".to_string(),
        "  yzx <command> [args...]".to_string(),
        String::new(),
        "Commands:".to_string(),
        rows,
        String::new(),
        "Flags:".to_string(),
        "  -h, --help           Display help for this command".to_string(),
        "  -V, --version        Show Yazelix version".to_string(),
        "  -v, --version-short  Show Yazelix version".to_string(),
    ]
    .join("\n")
}

pub fn render_yzx_externs(commands: &[YzxCommandMetadata]) -> String {
    let header = [
        "# Generated by Yazelix from Rust-owned yzx command metadata.",
        "# Restores Nushell completion/signature knowledge for the external yzx CLI.",
        "",
    ]
    .join("\n");
    let body = commands
        .iter()
        .map(render_extern_block)
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{header}{body}\n")
}

const fn switch(name: &'static str, short: Option<&'static str>) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Switch,
        name,
        short,
        shape: "string",
        optional: true,
    }
}

const fn named(
    name: &'static str,
    short: Option<&'static str>,
    shape: &'static str,
    optional: bool,
) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Named,
        name,
        short,
        shape,
        optional,
    }
}

const fn positional(
    name: &'static str,
    shape: &'static str,
    optional: bool,
) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Positional,
        name,
        short: None,
        shape,
        optional,
    }
}

const fn rest(name: &'static str) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Rest,
        name,
        short: None,
        shape: "string",
        optional: true,
    }
}

const fn cmd(
    name: &'static str,
    description: &'static str,
    category: YzxCommandCategory,
    parameters: &'static [YzxCommandParameter],
) -> YzxCommandMetadata {
    YzxCommandMetadata {
        name,
        description,
        category,
        parameters,
    }
}

fn render_extern_block(command: &YzxCommandMetadata) -> String {
    if command.parameters.is_empty() {
        return format!("export extern \"{}\" []", command.name);
    }

    let parameters = command
        .parameters
        .iter()
        .map(render_parameter)
        .collect::<Vec<_>>()
        .join("\n");
    format!("export extern \"{}\" [\n{}\n]", command.name, parameters)
}

fn render_parameter(parameter: &YzxCommandParameter) -> String {
    match parameter.kind {
        YzxParameterKind::Switch => render_flag(parameter),
        YzxParameterKind::Named => render_named(parameter),
        YzxParameterKind::Positional => render_positional(parameter),
        YzxParameterKind::Rest => format!("    ...{}: {}", parameter.name, parameter.shape),
    }
}

fn render_flag(parameter: &YzxCommandParameter) -> String {
    match parameter.short {
        Some(short) => format!("    --{}(-{})", parameter.name, short),
        None => format!("    --{}", parameter.name),
    }
}

fn render_named(parameter: &YzxCommandParameter) -> String {
    match parameter.short {
        Some(short) => format!("    --{}(-{}): {}", parameter.name, short, parameter.shape),
        None => format!("    --{}: {}", parameter.name, parameter.shape),
    }
}

fn render_positional(parameter: &YzxCommandParameter) -> String {
    if parameter.optional {
        format!("    {}?: {}", parameter.name, parameter.shape)
    } else {
        format!("    {}: {}", parameter.name, parameter.shape)
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: Rust metadata is the public source for migrated control-plane leaves that no longer live in the Nushell command tree.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn metadata_includes_rust_owned_control_plane_commands() {
        let names = yzx_command_metadata()
            .into_iter()
            .map(|command| command.name)
            .collect::<Vec<_>>();
        assert!(names.contains(&"yzx env"));
        assert!(names.contains(&"yzx run"));
        assert!(names.contains(&"yzx update"));
        assert!(names.contains(&"yzx update nix"));
    }

    // Defends: generated Nushell externs come from Rust metadata, including Rust-only leaves exactly once.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_externs_for_rust_only_leaves_once() {
        let data = yzx_command_metadata_data();
        assert_eq!(
            data.extern_content
                .matches("export extern \"yzx env\"")
                .count(),
            1
        );
        assert_eq!(
            data.extern_content
                .matches("export extern \"yzx run\"")
                .count(),
            1
        );
        assert!(data.extern_content.contains("--no-shell(-n)"));
        assert!(data.extern_content.contains("...argv: string"));
    }
}
