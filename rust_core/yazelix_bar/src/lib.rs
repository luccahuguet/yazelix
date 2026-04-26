pub const WIDGET_TRAY_PLACEHOLDER: &str = "__YAZELIX_WIDGET_TRAY__";
pub const CUSTOM_TEXT_PLACEHOLDER: &str = "__YAZELIX_CUSTOM_TEXT_SEGMENT__";
pub const ZJSTATUS_PLUGIN_URL_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_PLUGIN_URL__";
pub const ZJSTATUS_RUNTIME_DIR_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_DIR__";

pub const WIDGET_EDITOR: &str = "editor";
pub const WIDGET_SHELL: &str = "shell";
pub const WIDGET_TERM: &str = "term";
pub const WIDGET_WORKSPACE: &str = "workspace";
pub const WIDGET_CPU: &str = "cpu";
pub const WIDGET_RAM: &str = "ram";

pub const COMMAND_WORKSPACE: &str = "{command_workspace}";
pub const COMMAND_CPU: &str = "{command_cpu}";
pub const COMMAND_RAM: &str = "{command_ram}";
pub const COMMAND_VERSION: &str = "{command_version}";

pub const DEFAULT_WIDGET_TRAY: &[&str] = &[
    WIDGET_EDITOR,
    WIDGET_SHELL,
    WIDGET_TERM,
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
pub enum BarRenderError {
    InvalidWidgetTrayEntry { entry: String },
}

impl BarRenderError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidWidgetTrayEntry { .. } => "invalid_widget_tray_entry",
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
        .map(|parts| parts.join(" "))
}

pub fn render_custom_text_segment(custom_text: &str) -> String {
    let trimmed = custom_text.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("#[fg=#ffff00,bold][{trimmed}] ")
    }
}

fn render_widget(widget: &str, request: &BarRenderRequest) -> Result<String, BarRenderError> {
    match widget {
        WIDGET_EDITOR => Ok(format!(
            "#[fg=#00ff88,bold][editor: {}]",
            request.editor_label
        )),
        WIDGET_SHELL => Ok(format!(
            "#[fg=#00ff88,bold][shell: {}]",
            request.shell_label
        )),
        WIDGET_TERM => Ok(format!(
            "#[fg=#00ff88,bold][term: {}]",
            request.terminal_label
        )),
        WIDGET_WORKSPACE => Ok(COMMAND_WORKSPACE.to_string()),
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
            "#[fg=#00ff88,bold][editor: hx] #[fg=#00ff88,bold][shell: nu] #[fg=#00ff88,bold][term: ghostty] {command_cpu} {command_ram}"
        );
    }

    // Defends: a deliberately empty tray stays empty rather than introducing stray spacing.
    // Strength: defect=1 behavior=2 resilience=1 cost=1 uniqueness=2 total=7/10
    #[test]
    fn renders_empty_widget_tray_without_padding() {
        let rendered = render_widget_tray_segment(&render_request(&[])).unwrap();

        assert_eq!(rendered, "");
    }

    // Defends: the workspace widget is a dynamic zjstatus command placeholder rather than a static startup label.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_workspace_widget_as_dynamic_command_placeholder() {
        let rendered = render_widget_tray_segment(&render_request(&["workspace"])).unwrap();

        assert_eq!(rendered, "{command_workspace}");
    }

    // Defends: custom text remains trim-aware and does not reserve bar space when absent.
    // Strength: defect=1 behavior=2 resilience=1 cost=1 uniqueness=2 total=7/10
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
}
