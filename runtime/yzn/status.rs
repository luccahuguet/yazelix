use crate::{error::AppError, paths::zellij_session_label, runtime::Runtime};

pub(crate) fn print_status() -> Result<(), AppError> {
    let runtime = Runtime::prepare()?;
    println!("Yazelix Nova status");
    println!("config home: {}", runtime.config_home.display());
    println!("state dir: {}", runtime.state_dir.display());
    println!("shell: {}", runtime.shell_program);
    println!("editor command: {}", runtime.editor_command);
    println!("editor: {}", runtime.editor);
    println!("agent command: {}", runtime.agent_command);
    println!("agent args: {}", runtime.agent_args);
    println!("open log: {}", runtime.yzn_open_log);
    println!("welcome enabled: {}", runtime.welcome_enabled);
    println!("welcome style: {}", runtime.welcome_style);
    println!("welcome duration: {}s", runtime.welcome_duration_seconds);
    println!("mars config: {}", runtime.mars_config());
    println!("zellij config: {}", runtime.zellij_config());
    println!("zellij sidecar: {}", runtime.zellij_sidecar.display());
    println!("bar widgets: {}", runtime.bar_widgets);
    println!("popup side margin: {}", runtime.popup_side_margin);
    println!("popup vertical margin: {}", runtime.popup_vertical_margin);
    for binding in &runtime.popup_keybindings {
        println!("{} keybinding: {}", binding.label, binding.configured);
    }
    println!("layout: {}", runtime.layout());
    println!("inside zellij: {}", zellij_session_label("yes", "no"));
    Ok(())
}
