use crate::{
    PACKAGE_VARIANT, VERSION, YAZI_SOURCE, error::AppError, paths::zellij_session_label,
    runtime::Runtime,
};

pub(crate) fn print_status() -> Result<(), AppError> {
    let runtime = Runtime::prepare_with_yazi()?;
    println!("Yazelix Nova status");
    println!("package: {PACKAGE_VARIANT}");
    println!("config home: {}", runtime.config_home.display());
    println!("state dir: {}", runtime.state_dir.display());
    println!("runtime identity: {}", runtime.runtime_identity.display());
    println!("shell: {}", runtime.shell_program);
    println!("editor command: {}", runtime.editor_command);
    println!("editor: {}", runtime.editor);
    println!("agent command: {}", runtime.agent_command);
    println!("agent args: {}", runtime.agent_args);
    println!("open log: {}", runtime.yzx_open_log);
    println!("welcome enabled: {}", runtime.welcome_enabled);
    println!("welcome style: {}", runtime.welcome_style);
    println!("welcome duration: {}s", runtime.welcome_duration_seconds);
    println!("mars config: {}", runtime.mars_config());
    println!("zellij config: {}", runtime.zellij_config());
    println!("zellij sidecar: {}", runtime.zellij_sidecar.display());
    println!("bar widgets: {}", runtime.bar_widgets);
    println!("popup side margin: {}", runtime.popup_side_margin);
    println!("popup vertical margin: {}", runtime.popup_vertical_margin);
    for binding in &runtime.managed_keybindings {
        println!("{} keybinding: {}", binding.label, binding.configured);
    }
    println!("layout: {}", runtime.layout());
    println!("yazi source: {YAZI_SOURCE}");
    println!("yazi: {}", runtime.yazi().yazi.display());
    println!("ya: {}", runtime.yazi().ya.display());
    println!("yazi version: {}", runtime.yazi().version);
    println!("inside zellij: {}", zellij_session_label("yes", "no"));
    Ok(())
}

pub(crate) fn print_status_json() -> Result<(), AppError> {
    let runtime = Runtime::prepare_with_yazi()?;
    println!("{}", status_json(&runtime));
    Ok(())
}

fn status_json(runtime: &Runtime) -> String {
    let config_home = runtime.config_home.to_string_lossy();
    let state_dir = runtime.state_dir.to_string_lossy();
    let mut json = String::from("{\"schema_version\":1");
    for (key, value) in [
        ("name", "Yazelix Nova"),
        ("version", VERSION),
        ("package", PACKAGE_VARIANT),
        ("config_home", config_home.as_ref()),
        ("state_dir", state_dir.as_ref()),
        ("shell", runtime.shell_program.as_str()),
        ("editor_command", runtime.editor_command.as_str()),
        ("editor", runtime.editor.as_str()),
        ("agent_command", runtime.agent_command.as_str()),
    ] {
        json.push_str(&format!(",\"{key}\":{}", json_string(value)));
    }
    json.push_str(&format!(
        ",\"inside_zellij\":{}}}",
        zellij_session_label("true", "false")
    ));
    json
}

pub(crate) fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000C}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character <= '\u{001F}' => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use super::json_string;

    #[test]
    fn json_string_escapes_json_control_characters() {
        assert_eq!(
            json_string("\"\\\u{0008}\u{000C}\n\r\t\u{001F}"),
            r#""\"\\\b\f\n\r\t\u001f""#
        );
    }
}
