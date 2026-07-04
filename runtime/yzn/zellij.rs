use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    command::{create_dir_all_checked, run_checked, trim_output},
    error::{path_error, startup, AppError},
    paths::parent,
    runtime::PopupKeybinding,
    DEFAULT_BAR_WIDGETS_JSON, DEFAULT_POPUP_SIDE_MARGIN, DEFAULT_POPUP_VERTICAL_MARGIN, LAYOUT,
    LAYOUT_BAR_PLACEHOLDER, LAYOUT_SWAP_TEMPLATE, LAYOUT_TEMPLATE, LAYOUT_YAZI_PLACEHOLDER,
    YZN_BAR_RENDER, YZN_BAR_RENDER_REQUEST, YZN_SIDEBAR_REFRESH, YZN_YAZI, ZELLIJ_HOME_PLACEHOLDER,
};

pub(crate) fn active_layout(
    state_dir: &Path,
    bar_widgets: &str,
) -> Result<(&'static str, PathBuf), AppError> {
    if bar_widgets == DEFAULT_BAR_WIDGETS_JSON {
        return Ok(("packaged", PathBuf::from(LAYOUT)));
    }

    let layout = state_dir.join("zellij/layout.kdl");
    let plugin_block = render_bar_plugin_block(bar_widgets)?;
    materialize_layout(&layout, &plugin_block)?;
    Ok(("runtime", layout))
}

pub(crate) fn active_zellij_config(
    state_dir: &Path,
    source: &'static str,
    config: PathBuf,
    layout: &Path,
    popup_side_margin: &str,
    popup_vertical_margin: &str,
    popup_keybindings: &[PopupKeybinding],
    custom_popups_kdl: &str,
    custom_popup_keybindings_kdl: &str,
    home_dir: &Path,
) -> Result<(&'static str, PathBuf), AppError> {
    let runtime_config = state_dir.join("zellij/config.kdl");
    let text =
        fs::read_to_string(&config).map_err(|error| path_error("read", &config, &config, error))?;
    let mut patched = text;
    let replaced = patched.replace(ZELLIJ_HOME_PLACEHOLDER, &kdl_string(home_dir.display()));
    if replaced == patched {
        return Err(startup(
            "Zellij config is missing the managed home cwd placeholder",
            config.display(),
            1,
        ));
    }
    patched = replaced;
    if layout != Path::new(LAYOUT) {
        let replaced = patched.replace(LAYOUT, &layout.display().to_string());
        if replaced == patched {
            return Err(startup(
                "Zellij config is missing the packaged layout path",
                config.display(),
                1,
            ));
        }
        patched = replaced;
    }
    patched =
        patch_popup_default_margins(patched, &config, popup_side_margin, popup_vertical_margin)?;
    patched = patch_popup_keybindings(patched, &config, popup_keybindings)?;
    patched = inject_snippet_before(
        patched,
        &config,
        custom_popups_kdl,
        "        }\n    }\n\n    yazelix_pane_orchestrator",
        "Zellij config is missing the packaged popup block",
    )?;
    patched = inject_snippet_before(
        patched,
        &config,
        custom_popup_keybindings_kdl,
        r#"        bind "Alt h" "Alt Left" { MessagePlugin "yazelix_pane_orchestrator" { name "move_focus_left_or_tab"; }; }"#,
        "Zellij config is missing the packaged shared keybind block",
    )?;
    create_dir_all_checked(parent(&runtime_config), &runtime_config)?;
    fs::write(&runtime_config, patched)
        .map_err(|error| path_error("write", &runtime_config, &runtime_config, error))?;
    Ok((
        if source == "sidecar" {
            "sidecar+runtime"
        } else {
            "runtime"
        },
        runtime_config,
    ))
}

fn patch_popup_keybindings(
    text: String,
    config: &Path,
    popup_keybindings: &[PopupKeybinding],
) -> Result<String, AppError> {
    let mut patched = text;
    for (index, binding) in popup_keybindings.iter().enumerate() {
        if binding.configured == binding.default {
            continue;
        }
        let marker = format!("bind {}", kdl_string(binding.default));
        if !patched.contains(&marker) {
            return Err(startup(
                format!(
                    "Zellij config is missing the packaged {} key binding",
                    binding.label
                ),
                config.display(),
                1,
            ));
        }
        patched = patched.replace(&marker, &format!("bind __YZN_POPUP_KEY_{index}__"));
    }
    for (index, binding) in popup_keybindings.iter().enumerate() {
        if binding.configured == binding.default {
            continue;
        }
        patched = patched.replace(
            &format!("__YZN_POPUP_KEY_{index}__"),
            &kdl_string(&binding.configured),
        );
    }
    Ok(patched)
}

fn patch_popup_default_margins(
    text: String,
    config: &Path,
    side_margin: &str,
    vertical_margin: &str,
) -> Result<String, AppError> {
    let marker = format!(
        "        popup_defaults {{\n            side_margin {DEFAULT_POPUP_SIDE_MARGIN}\n            vertical_margin {DEFAULT_POPUP_VERTICAL_MARGIN}\n            on_close {{\n                command {}\n            }}\n            on_hide {{\n                command {}\n            }}\n        }}",
        kdl_string(YZN_SIDEBAR_REFRESH),
        kdl_string(YZN_SIDEBAR_REFRESH),
    );
    if !text.contains(&marker) {
        return Err(startup(
            "Zellij config is missing packaged popup defaults",
            config.display(),
            1,
        ));
    }
    Ok(text.replacen(
        &marker,
        &format!(
            "        popup_defaults {{\n            side_margin {side_margin}\n            vertical_margin {vertical_margin}\n            on_close {{\n                command {}\n            }}\n            on_hide {{\n                command {}\n            }}\n        }}",
            kdl_string(YZN_SIDEBAR_REFRESH),
            kdl_string(YZN_SIDEBAR_REFRESH),
        ),
        1,
    ))
}

fn inject_snippet_before(
    text: String,
    config: &Path,
    snippet: &str,
    marker: &str,
    missing_message: &str,
) -> Result<String, AppError> {
    let snippet = snippet.trim_end();
    if snippet.is_empty() {
        return Ok(text);
    }
    if !text.contains(marker) {
        return Err(startup(missing_message, config.display(), 1));
    }
    Ok(text.replacen(marker, &format!("{snippet}\n{marker}"), 1))
}

fn render_bar_plugin_block(bar_widgets: &str) -> Result<String, AppError> {
    let template_path = Path::new(YZN_BAR_RENDER_REQUEST);
    let template = fs::read_to_string(template_path)
        .map_err(|error| path_error("read", template_path, template_path, error))?;
    let request = template.replace(r#""__YZN_BAR_WIDGET_TRAY__""#, bar_widgets);
    Ok(trim_output(run_checked(
        Path::new(YZN_BAR_RENDER),
        Command::new(YZN_BAR_RENDER).arg(request),
    )?))
}

fn materialize_layout(path: &Path, plugin_block: &str) -> Result<(), AppError> {
    let template_path = Path::new(LAYOUT_TEMPLATE);
    let swap_template_path = Path::new(LAYOUT_SWAP_TEMPLATE);
    let template = fs::read_to_string(template_path)
        .map_err(|error| path_error("read", template_path, template_path, error))?;
    let swap_template = fs::read_to_string(swap_template_path)
        .map_err(|error| path_error("read", swap_template_path, swap_template_path, error))?;
    let layout = template
        .replace(LAYOUT_YAZI_PLACEHOLDER, YZN_YAZI)
        .replace(LAYOUT_BAR_PLACEHOLDER, plugin_block);
    let swap_layout = swap_template.replace(LAYOUT_YAZI_PLACEHOLDER, YZN_YAZI);
    let swap_path = path.with_file_name("layout.swap.kdl");
    create_dir_all_checked(parent(path), path)?;
    fs::write(path, layout).map_err(|error| path_error("write", path, path, error))?;
    fs::write(&swap_path, swap_layout)
        .map_err(|error| path_error("write", &swap_path, &swap_path, error))
}

fn kdl_string(value: impl Display) -> String {
    format!("{:?}", value.to_string())
}
