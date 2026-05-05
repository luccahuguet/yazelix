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
}
