pub const WIDGET_TRAY_PLACEHOLDER: &str = "__YAZELIX_WIDGET_TRAY__";
pub const CUSTOM_TEXT_PLACEHOLDER: &str = "__YAZELIX_CUSTOM_TEXT_SEGMENT__";
pub const TAB_NORMAL_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_TAB_NORMAL__";
pub const TAB_NORMAL_FULLSCREEN_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_TAB_NORMAL_FULLSCREEN__";
pub const TAB_NORMAL_SYNC_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_TAB_NORMAL_SYNC__";
pub const TAB_ACTIVE_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_TAB_ACTIVE__";
pub const TAB_ACTIVE_FULLSCREEN_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_TAB_ACTIVE_FULLSCREEN__";
pub const TAB_ACTIVE_SYNC_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_TAB_ACTIVE_SYNC__";
pub const TAB_RENAME_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_TAB_RENAME__";
pub const ZJSTATUS_PLUGIN_URL_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_PLUGIN_URL__";
pub const ZJSTATUS_RUNTIME_DIR_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_DIR__";
pub const ZJSTATUS_NU_BIN_PLACEHOLDER: &str = "__YAZELIX_NU_BIN__";
pub const ZJSTATUS_YZX_CONTROL_BIN_PLACEHOLDER: &str = "__YAZELIX_YZX_CONTROL_BIN__";

pub const WIDGET_EDITOR: &str = "editor";
pub const WIDGET_SHELL: &str = "shell";
pub const WIDGET_TERM: &str = "term";
pub const WIDGET_WORKSPACE: &str = "workspace";
pub const WIDGET_CURSOR: &str = "cursor";
pub const WIDGET_CLAUDE_USAGE: &str = "claude_usage";
pub const WIDGET_CODEX_USAGE: &str = "codex_usage";
pub const WIDGET_OPENCODE_GO_USAGE: &str = "opencode_go_usage";
pub const WIDGET_CPU: &str = "cpu";
pub const WIDGET_RAM: &str = "ram";

pub const COMMAND_WORKSPACE: &str = "{command_workspace}";
pub const COMMAND_CURSOR: &str = "{command_cursor}";
pub const COMMAND_CLAUDE_USAGE: &str = "{command_claude_usage}";
pub const COMMAND_CODEX_USAGE: &str = "{command_codex_usage}";
pub const COMMAND_OPENCODE_GO_USAGE: &str = "{command_opencode_go_usage}";
pub const COMMAND_CPU: &str = "{command_cpu}";
pub const COMMAND_RAM: &str = "{command_ram}";
pub const COMMAND_VERSION: &str = "{command_version}";
pub const TAB_LABEL_MODE_FULL: &str = "full";
pub const TAB_LABEL_MODE_COMPACT: &str = "compact";
pub const DEFAULT_STANDALONE_WASM_URL: &str = "__YAZELIX_BAR_ZJSTATUS_WASM__";

pub const DEFAULT_WIDGET_TRAY: &[&str] = &[
    WIDGET_EDITOR,
    WIDGET_SHELL,
    WIDGET_TERM,
    WIDGET_CURSOR,
    WIDGET_CODEX_USAGE,
    WIDGET_CPU,
    WIDGET_RAM,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarRenderRequest {
    pub widget_tray: Vec<String>,
    pub editor_label: String,
    pub shell_label: String,
    pub terminal_label: String,
    pub custom_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarRenderData {
    pub widget_tray_segment: String,
    pub custom_text_segment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabLabelFormats {
    pub tab_normal: &'static str,
    pub tab_normal_fullscreen: &'static str,
    pub tab_normal_sync: &'static str,
    pub tab_active: &'static str,
    pub tab_active_fullscreen: &'static str,
    pub tab_active_sync: &'static str,
    pub tab_rename: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarRenderError {
    InvalidWidgetTrayEntry { entry: String },
    InvalidTabLabelMode { mode: String },
}

impl BarRenderError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidWidgetTrayEntry { .. } => "invalid_widget_tray_entry",
            Self::InvalidTabLabelMode { .. } => "invalid_tab_label_mode",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandalonePresetOptions {
    pub wasm_url: String,
    pub brand_label: String,
    pub brand_color: String,
    pub session_color: String,
    pub datetime_color: String,
    pub datetime_format: String,
    pub tab_label_mode: String,
    pub format_left: Vec<StandalonePresetPart>,
    pub format_center: Vec<StandalonePresetPart>,
    pub format_right: Vec<StandalonePresetPart>,
    pub command_widgets: Vec<StandaloneCommandWidget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandalonePresetPart {
    Mode,
    Tabs,
    Session,
    Datetime,
    Brand,
    Command(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneCommandWidget {
    pub name: String,
    pub command: String,
    pub format: String,
    pub interval: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandalonePresetError {
    InvalidTabLabelMode { mode: String },
    InvalidColor { field: String, value: String },
    InvalidCommandName { name: String },
    EmptyCommand { name: String },
    DuplicateCommandName { name: String },
}

impl Default for StandalonePresetOptions {
    fn default() -> Self {
        Self {
            wasm_url: DEFAULT_STANDALONE_WASM_URL.to_string(),
            brand_label: "YAZELIX BAR".to_string(),
            brand_color: "#00ccff".to_string(),
            session_color: "#ff0088".to_string(),
            datetime_color: "#bb88ff".to_string(),
            datetime_format: "%H:%M %d/%m".to_string(),
            tab_label_mode: TAB_LABEL_MODE_FULL.to_string(),
            format_left: vec![StandalonePresetPart::Mode, StandalonePresetPart::Tabs],
            format_center: Vec::new(),
            format_right: vec![
                StandalonePresetPart::Session,
                StandalonePresetPart::Datetime,
                StandalonePresetPart::Brand,
            ],
            command_widgets: Vec::new(),
        }
    }
}

impl StandaloneCommandWidget {
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            format: " #[fg=#00ff88,bold][{stdout}]".to_string(),
            interval: "30".to_string(),
        }
    }
}

pub fn generate_standalone_preset(
    options: &StandalonePresetOptions,
) -> Result<String, StandalonePresetError> {
    validate_hex_color("brand_color", &options.brand_color)?;
    validate_hex_color("session_color", &options.session_color)?;
    validate_hex_color("datetime_color", &options.datetime_color)?;
    let tab_labels = render_zjstatus_tab_label_formats(&options.tab_label_mode).map_err(|_| {
        StandalonePresetError::InvalidTabLabelMode {
            mode: options.tab_label_mode.clone(),
        }
    })?;
    validate_commands(&options.command_widgets)?;

    let mut lines = vec![
        format!(
            "plugin location=\"{}\" {{",
            escape_kdl_string(&options.wasm_url)
        ),
        "    // Generated by yazelix_bar_generate. Edit the options or raw KDL.".to_string(),
        format!(
            "    format_left   \"{}\"",
            escape_kdl_string(&render_standalone_format(&options.format_left, options))
        ),
        format!(
            "    format_center \"{}\"",
            escape_kdl_string(&render_standalone_format(&options.format_center, options))
        ),
        format!(
            "    format_right  \"{}\"",
            escape_kdl_string(&render_standalone_format(&options.format_right, options))
        ),
        "    format_hide_on_overlength \"true\"".to_string(),
        "    format_precedence \"lrc\"".to_string(),
        "    format_space  \"\"".to_string(),
        String::new(),
        "    border_enabled  \"false\"".to_string(),
        String::new(),
        "    mode_normal  \"#[bg=#00ff88,fg=#000000,bold] NORMAL \"".to_string(),
        "    mode_tmux    \"#[bg=#ffff00,fg=#000000,bold] TMUX \"".to_string(),
        "    mode_session \"#[bg=#ff6600,fg=#000000,bold] SESSION \"".to_string(),
        "    mode_scroll  \"#[bg=#ff0088,fg=#ffffff,bold] SCROLL \"".to_string(),
        String::new(),
        format!("    {}", tab_labels.tab_normal),
        format!("    {}", tab_labels.tab_normal_fullscreen),
        format!("    {}", tab_labels.tab_normal_sync),
        format!("    {}", tab_labels.tab_active),
        format!("    {}", tab_labels.tab_active_fullscreen),
        format!("    {}", tab_labels.tab_active_sync),
        "    tab_separator \"\"".to_string(),
        format!("    {}", tab_labels.tab_rename),
        "    tab_sync_indicator       \"<> \"".to_string(),
        "    tab_fullscreen_indicator \"[] \"".to_string(),
        "    tab_floating_indicator   \"o \"".to_string(),
        "    tab_display_count \"6\"".to_string(),
        "    tab_truncate_start_format \"#[fg=#ff6600,bold]< +{count} ... \"".to_string(),
        "    tab_truncate_end_format   \"#[fg=#ff6600,bold]... +{count} > \"".to_string(),
        String::new(),
        format!(
            "    datetime        \"#[fg={},bold] {{format}} \"",
            options.datetime_color
        ),
        format!(
            "    datetime_format \"{}\"",
            escape_kdl_string(&options.datetime_format)
        ),
    ];

    if !options.command_widgets.is_empty() {
        lines.push(String::new());
        for command in &options.command_widgets {
            lines.push(format!(
                "    command_{}_command \"{}\"",
                command.name,
                escape_kdl_string(&command.command)
            ));
            lines.push(format!(
                "    command_{}_format \"{}\"",
                command.name,
                escape_kdl_string(&command.format)
            ));
            lines.push(format!(
                "    command_{}_interval \"{}\"",
                command.name,
                escape_kdl_string(&command.interval)
            ));
            lines.push(String::new());
        }
        if lines.last().is_some_and(String::is_empty) {
            lines.pop();
        }
    }

    lines.push("}".to_string());
    lines.push(String::new());
    Ok(lines.join("\n"))
}

pub fn standalone_part_from_token(token: &str) -> Option<StandalonePresetPart> {
    let token = token.trim();
    match token {
        "mode" => Some(StandalonePresetPart::Mode),
        "tabs" => Some(StandalonePresetPart::Tabs),
        "session" => Some(StandalonePresetPart::Session),
        "datetime" => Some(StandalonePresetPart::Datetime),
        "brand" => Some(StandalonePresetPart::Brand),
        _ => token
            .strip_prefix("command:")
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(|name| StandalonePresetPart::Command(name.to_string())),
    }
}

fn render_standalone_format(
    parts: &[StandalonePresetPart],
    options: &StandalonePresetOptions,
) -> String {
    let rendered = parts
        .iter()
        .map(|part| match part {
            StandalonePresetPart::Mode => "{mode}".to_string(),
            StandalonePresetPart::Tabs => "{tabs}".to_string(),
            StandalonePresetPart::Session => {
                format!("#[fg={},bold]{{session}}", options.session_color)
            }
            StandalonePresetPart::Datetime => "{datetime}".to_string(),
            StandalonePresetPart::Brand => {
                format!("#[fg={},bold]{}", options.brand_color, options.brand_label)
            }
            StandalonePresetPart::Command(name) => format!("{{command_{name}}}"),
        })
        .collect::<Vec<_>>()
        .join(" ");
    if rendered.is_empty() {
        rendered
    } else {
        format!("{rendered} ")
    }
}

fn validate_commands(commands: &[StandaloneCommandWidget]) -> Result<(), StandalonePresetError> {
    let mut names = std::collections::BTreeSet::new();
    for command in commands {
        validate_command_name(&command.name)?;
        if command.command.trim().is_empty() {
            return Err(StandalonePresetError::EmptyCommand {
                name: command.name.clone(),
            });
        }
        if !names.insert(command.name.clone()) {
            return Err(StandalonePresetError::DuplicateCommandName {
                name: command.name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_command_name(name: &str) -> Result<(), StandalonePresetError> {
    let valid = !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && !name.starts_with("command_");
    if valid {
        Ok(())
    } else {
        Err(StandalonePresetError::InvalidCommandName {
            name: name.to_string(),
        })
    }
}

fn validate_hex_color(field: &str, color: &str) -> Result<(), StandalonePresetError> {
    let valid = color.len() == 7
        && color.starts_with('#')
        && color[1..].bytes().all(|byte| byte.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err(StandalonePresetError::InvalidColor {
            field: field.to_string(),
            value: color.to_string(),
        })
    }
}

fn escape_kdl_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

pub fn render_zjstatus_bar_segments(
    request: &BarRenderRequest,
) -> Result<BarRenderData, BarRenderError> {
    Ok(BarRenderData {
        widget_tray_segment: render_widget_tray_segment(request)?,
        custom_text_segment: render_custom_text_segment(&request.custom_text),
    })
}

pub fn render_widget_tray_segment(request: &BarRenderRequest) -> Result<String, BarRenderError> {
    request
        .widget_tray
        .iter()
        .map(|widget| render_widget(widget, request))
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| {
            parts
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("")
        })
}

pub fn render_custom_text_segment(custom_text: &str) -> String {
    let trimmed = custom_text.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("#[fg=#ffff00,bold][{trimmed}] ")
    }
}

pub fn render_zjstatus_tab_label_formats(mode: &str) -> Result<TabLabelFormats, BarRenderError> {
    match mode {
        TAB_LABEL_MODE_FULL => Ok(TabLabelFormats {
            tab_normal: r##"tab_normal   "#[fg=#ffff00] [{index}] {name} ""##,
            tab_normal_fullscreen: r##"tab_normal_fullscreen "#[fg=#ffff00] [{index}] {name} [] ""##,
            tab_normal_sync: r##"tab_normal_sync       "#[fg=#ffff00] [{index}] {name} <> ""##,
            tab_active: r##"tab_active   "#[bg=#ff6600,fg=#000000,bold] [{index}] {name} {floating_indicator}""##,
            tab_active_fullscreen: r##"tab_active_fullscreen "#[bg=#ff6600,fg=#000000,bold] [{index}] {name} {fullscreen_indicator}""##,
            tab_active_sync: r##"tab_active_sync       "#[bg=#ff6600,fg=#000000,bold] [{index}] {name} {sync_indicator}""##,
            tab_rename: r##"tab_rename    "#[bg=#ff6600,fg=#000000,bold] {index} {name} {floating_indicator} ""##,
        }),
        TAB_LABEL_MODE_COMPACT => Ok(TabLabelFormats {
            tab_normal: r##"tab_normal   "#[fg=#ffff00] [{index}] ""##,
            tab_normal_fullscreen: r##"tab_normal_fullscreen "#[fg=#ffff00] [{index}] [] ""##,
            tab_normal_sync: r##"tab_normal_sync       "#[fg=#ffff00] [{index}] <> ""##,
            tab_active: r##"tab_active   "#[bg=#ff6600,fg=#000000,bold] [{index}] {floating_indicator}""##,
            tab_active_fullscreen: r##"tab_active_fullscreen "#[bg=#ff6600,fg=#000000,bold] [{index}] {fullscreen_indicator}""##,
            tab_active_sync: r##"tab_active_sync       "#[bg=#ff6600,fg=#000000,bold] [{index}] {sync_indicator}""##,
            tab_rename: r##"tab_rename    "#[bg=#ff6600,fg=#000000,bold] {index} {name} {floating_indicator} ""##,
        }),
        _ => Err(BarRenderError::InvalidTabLabelMode {
            mode: mode.to_string(),
        }),
    }
}

fn render_widget(widget: &str, request: &BarRenderRequest) -> Result<String, BarRenderError> {
    match widget {
        WIDGET_EDITOR => Ok(format!(
            " #[fg=#00ff88,bold][editor: {}]",
            request.editor_label
        )),
        WIDGET_SHELL => Ok(format!(
            " #[fg=#00ff88,bold][shell: {}]",
            request.shell_label
        )),
        WIDGET_TERM => Ok(format!(
            " #[fg=#00ff88,bold][term: {}]",
            request.terminal_label
        )),
        WIDGET_WORKSPACE => Ok(COMMAND_WORKSPACE.to_string()),
        WIDGET_CURSOR => Ok(COMMAND_CURSOR.to_string()),
        WIDGET_CLAUDE_USAGE => Ok(COMMAND_CLAUDE_USAGE.to_string()),
        WIDGET_CODEX_USAGE => Ok(COMMAND_CODEX_USAGE.to_string()),
        WIDGET_OPENCODE_GO_USAGE => Ok(COMMAND_OPENCODE_GO_USAGE.to_string()),
        WIDGET_CPU => Ok(COMMAND_CPU.to_string()),
        WIDGET_RAM => Ok(COMMAND_RAM.to_string()),
        _ => Err(BarRenderError::InvalidWidgetTrayEntry {
            entry: widget.to_string(),
        }),
    }
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    fn render_request(widget_tray: &[&str]) -> BarRenderRequest {
        BarRenderRequest {
            widget_tray: widget_tray
                .iter()
                .map(|widget| widget.to_string())
                .collect(),
            editor_label: "hx".to_string(),
            shell_label: "nu".to_string(),
            terminal_label: "ghostty".to_string(),
            custom_text: String::new(),
        }
    }

    // Defends: the bar registry preserves the existing zjstatus widgets and exact segment syntax.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_existing_widget_tray_segments() {
        let rendered = render_widget_tray_segment(&render_request(DEFAULT_WIDGET_TRAY)).unwrap();

        assert_eq!(
            rendered,
            " #[fg=#00ff88,bold][editor: hx] #[fg=#00ff88,bold][shell: nu] #[fg=#00ff88,bold][term: ghostty]{command_cursor}{command_codex_usage}{command_cpu}{command_ram}"
        );
    }

    // Defends: a deliberately empty tray stays empty rather than introducing stray spacing.
    // Strength: defect=1 behavior=2 resilience=1 cost=2 uniqueness=2 total=8/10
    #[test]
    fn renders_empty_widget_tray_without_padding() {
        let rendered = render_widget_tray_segment(&render_request(&[])).unwrap();

        assert_eq!(rendered, "");
    }

    // Regression: dynamic status-bus widgets render through cached zjstatus command placeholders instead of being silently hidden.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_status_bus_widgets_as_cached_command_placeholders() {
        let rendered =
            render_widget_tray_segment(&render_request(&["workspace", "cursor"])).unwrap();

        assert_eq!(rendered, "{command_workspace}{command_cursor}");
    }

    // Regression: agent usage widgets render through cache readers so expensive providers are never polled by zjstatus.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn renders_agent_usage_widgets_as_cached_command_placeholders() {
        let rendered = render_widget_tray_segment(&render_request(&[
            "claude_usage",
            "codex_usage",
            "opencode_go_usage",
        ]))
        .unwrap();

        assert_eq!(
            rendered,
            "{command_claude_usage}{command_codex_usage}{command_opencode_go_usage}"
        );
    }

    // Regression: dynamic command placeholders must preserve stable spacing around safe widgets.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn dynamic_widgets_do_not_leave_spacing_artifacts() {
        let rendered =
            render_widget_tray_segment(&render_request(&["editor", "workspace", "shell"])).unwrap();

        assert_eq!(
            rendered,
            " #[fg=#00ff88,bold][editor: hx]{command_workspace} #[fg=#00ff88,bold][shell: nu]"
        );
    }

    // Defends: custom text remains trim-aware and does not reserve bar space when absent.
    // Strength: defect=1 behavior=2 resilience=1 cost=2 uniqueness=2 total=8/10
    #[test]
    fn renders_custom_text_segment_only_when_present() {
        assert_eq!(
            render_custom_text_segment("  verdant-lake  "),
            "#[fg=#ffff00,bold][verdant-lake] "
        );
        assert_eq!(render_custom_text_segment("   "), "");
    }

    // Regression: unsupported widget names must fail fast instead of leaving broken zjstatus placeholders.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn rejects_unknown_widget_tray_entries() {
        let error =
            render_widget_tray_segment(&render_request(&["editor", "weather"])).unwrap_err();

        assert_eq!(
            error,
            BarRenderError::InvalidWidgetTrayEntry {
                entry: "weather".to_string()
            }
        );
        assert_eq!(error.code(), "invalid_widget_tray_entry");
    }

    // Defends: full tab labels keep the existing index plus name format unless compact mode is explicitly enabled.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_full_tab_label_formats_by_default_contract() {
        let formats = render_zjstatus_tab_label_formats(TAB_LABEL_MODE_FULL).unwrap();

        assert!(formats.tab_normal.contains("[{index}] {name}"));
        assert!(formats.tab_active.contains("[{index}] {name}"));
        assert!(formats.tab_rename.contains("{index} {name}"));
    }

    // Defends: compact tab labels remove tab names from normal rendering while preserving index and state indicators.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_compact_tab_label_formats_without_names() {
        let formats = render_zjstatus_tab_label_formats(TAB_LABEL_MODE_COMPACT).unwrap();

        assert_eq!(
            formats.tab_normal,
            r##"tab_normal   "#[fg=#ffff00] [{index}] ""##
        );
        assert!(formats.tab_normal_fullscreen.contains("{index}] []"));
        assert!(formats.tab_active_sync.contains("{sync_indicator}"));
        assert!(!formats.tab_active.contains("{name}"));
        assert!(formats.tab_rename.contains("{name}"));
    }

    // Regression: unsupported tab label modes fail fast instead of emitting broken zjstatus KDL.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn rejects_unknown_tab_label_mode() {
        let error = render_zjstatus_tab_label_formats("tiny").unwrap_err();

        assert_eq!(
            error,
            BarRenderError::InvalidTabLabelMode {
                mode: "tiny".to_string()
            }
        );
        assert_eq!(error.code(), "invalid_tab_label_mode");
    }

    // Defends: the standalone generator emits a complete generic zjstatus preset without Yazelix runtime helper commands.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn standalone_generator_emits_generic_default_preset() {
        let rendered = generate_standalone_preset(&StandalonePresetOptions::default()).unwrap();

        assert!(rendered.contains("plugin location=\"__YAZELIX_BAR_ZJSTATUS_WASM__\""));
        assert!(rendered.contains("format_left   \"{mode} {tabs} \""));
        assert!(rendered.contains("format_right  \"#[fg=#ff0088,bold]{session} {datetime} #[fg=#00ccff,bold]YAZELIX BAR \""));
        assert!(rendered.contains("datetime_format \"%H:%M %d/%m\""));
        assert!(!rendered.contains("yzx_control"));
        assert!(!rendered.contains("__YAZELIX_RUNTIME_DIR__"));
    }

    // Defends: structured command-widget options generate zjstatus command keys without requiring users to copy a whole preset.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn standalone_generator_emits_command_widgets_from_structured_options() {
        let mut options = StandalonePresetOptions {
            brand_label: "DEV BAR".into(),
            format_right: vec![
                StandalonePresetPart::Session,
                StandalonePresetPart::Command("host".into()),
                StandalonePresetPart::Brand,
            ],
            ..StandalonePresetOptions::default()
        };
        options.command_widgets.push(StandaloneCommandWidget {
            name: "host".into(),
            command: "hostname -s".into(),
            format: " #[fg=#00ff88,bold][{stdout}]".into(),
            interval: "30".into(),
        });

        let rendered = generate_standalone_preset(&options).unwrap();

        assert!(rendered.contains(
            "format_right  \"#[fg=#ff0088,bold]{session} {command_host} #[fg=#00ccff,bold]DEV BAR \""
        ));
        assert!(rendered.contains("command_host_command \"hostname -s\""));
        assert!(rendered.contains("command_host_format \" #[fg=#00ff88,bold][{stdout}]\""));
        assert!(rendered.contains("command_host_interval \"30\""));
    }

    // Regression: generator command names must fail fast before producing invalid zjstatus KDL keys.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn standalone_generator_rejects_invalid_command_names() {
        let mut options = StandalonePresetOptions::default();
        options
            .command_widgets
            .push(StandaloneCommandWidget::new("bad-name", "hostname -s"));

        let error = generate_standalone_preset(&options).unwrap_err();

        assert_eq!(
            error,
            StandalonePresetError::InvalidCommandName {
                name: "bad-name".into()
            }
        );
    }
}
