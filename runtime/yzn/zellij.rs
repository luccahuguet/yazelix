use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    DEFAULT_BAR_WIDGETS_JSON, DEFAULT_POPUP_SIDE_MARGIN, DEFAULT_POPUP_VERTICAL_MARGIN,
    DEFAULT_SHELL_PROGRAM, LAYOUT, LAYOUT_BAR_PLACEHOLDER, LAYOUT_SWAP_TEMPLATE, LAYOUT_TEMPLATE,
    LAYOUT_YAZI_PLACEHOLDER, YZN_AGENT, YZN_BAR_RENDER, YZN_BAR_RENDER_REQUEST,
    YZN_SIDEBAR_REFRESH, YZN_YAZI, ZELLIJ_HOME_PLACEHOLDER,
    command::{create_dir_all_checked, run_checked, trim_output},
    error::{AppError, path_error, startup},
    paths::parent,
    runtime::PopupKeybinding,
};

pub(crate) fn active_layout(
    state_dir: &Path,
    bar_widgets: &str,
    shell_label: &str,
) -> Result<(&'static str, PathBuf), AppError> {
    if bar_widgets == DEFAULT_BAR_WIDGETS_JSON && shell_label == DEFAULT_SHELL_PROGRAM {
        return Ok(("packaged", PathBuf::from(LAYOUT)));
    }

    let layout = state_dir.join("zellij/layout.kdl");
    let plugin_block = render_bar_plugin_block(bar_widgets, shell_label)?;
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
    agent_popup_kdl: &str,
    custom_popups_kdl: &str,
    custom_popup_keybindings_kdl: &str,
    zellij_plugins_sidecar: &Path,
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
    patched = patch_agent_popup(patched, &config, agent_popup_kdl)?;
    patched = inject_snippet_before(
        patched,
        &config,
        custom_popups_kdl,
        "        }\n    }\n\n    yazelix_pane_orchestrator",
        "Zellij config is missing the packaged popup block",
    )?;
    patched = patch_zellij_plugin_sidecar(patched, &config, zellij_plugins_sidecar)?;
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

fn patch_agent_popup(
    text: String,
    config: &Path,
    agent_popup_kdl: &str,
) -> Result<String, AppError> {
    let replacement = agent_popup_kdl.trim_end();
    if replacement.is_empty() {
        return Ok(text);
    }
    let marker = format!(
        "            agent {{\n                command {}\n                pane_title \"agent_popup\"\n                width_percent 100\n                height_percent 100\n                toggle_close_behavior \"hide\"\n            }}",
        kdl_string(YZN_AGENT),
    );
    if !text.contains(&marker) {
        return Err(startup(
            "Zellij config is missing the packaged agent popup block",
            config.display(),
            1,
        ));
    }
    Ok(text.replacen(&marker, replacement, 1))
}

const OWNED_ZELLIJ_PLUGIN_IDS: &[&str] = &["yzpp", "yazelix_pane_orchestrator"];

#[derive(Clone, Copy)]
enum ZellijPluginBlock {
    Plugins,
    LoadPlugins,
}

#[derive(Default)]
struct ZellijPluginSidecar {
    plugins: String,
    load_plugins: String,
}

fn patch_zellij_plugin_sidecar(
    text: String,
    config: &Path,
    sidecar: &Path,
) -> Result<String, AppError> {
    if !sidecar.is_file() {
        return Ok(text);
    }

    let sidecar_text =
        fs::read_to_string(sidecar).map_err(|error| path_error("read", sidecar, sidecar, error))?;
    let sidecar_blocks = parse_zellij_plugin_sidecar(sidecar, &sidecar_text)?;
    let patched = inject_snippet_before(
        text,
        config,
        &sidecar_blocks.plugins,
        "    yazelix_pane_orchestrator location=",
        "Zellij config is missing the packaged plugins block",
    )?;
    inject_snippet_before(
        patched,
        config,
        &sidecar_blocks.load_plugins,
        "    yazelix_pane_orchestrator\n}",
        "Zellij config is missing the packaged load_plugins block",
    )
}

fn parse_zellij_plugin_sidecar(path: &Path, text: &str) -> Result<ZellijPluginSidecar, AppError> {
    let mut blocks = ZellijPluginSidecar::default();
    let mut current = None;
    let mut body = String::new();
    let mut body_start_line = 1usize;
    let mut depth = 0usize;
    let mut seen_plugins = false;
    let mut seen_load_plugins = false;

    for (index, line) in text.lines().enumerate() {
        if let Some(block) = current {
            if depth == 1 && zellij_code_before_comment(line).trim() == "}" {
                let finished_body = body.trim_end().to_string();
                body.clear();
                validate_zellij_plugin_block_ids(path, block, body_start_line, &finished_body)?;
                match block {
                    ZellijPluginBlock::Plugins => blocks.plugins = finished_body,
                    ZellijPluginBlock::LoadPlugins => blocks.load_plugins = finished_body,
                }
                current = None;
                depth = 0;
                continue;
            }
            depth = zellij_sidecar_depth(path, index + 1, depth, line)?;
            body.push_str(line);
            body.push('\n');
            continue;
        }

        let code = zellij_code_before_comment(line);
        let Some(name) = first_token(code) else {
            continue;
        };
        let block = match name {
            "plugins" => ZellijPluginBlock::Plugins,
            "load_plugins" => ZellijPluginBlock::LoadPlugins,
            _ => {
                return Err(startup(
                    format!(
                        "Zellij plugin sidecar supports only top-level `plugins` and `load_plugins`, found `{name}`"
                    ),
                    path.display(),
                    1,
                ));
            }
        };
        let rest = code.trim_start()[name.len()..].trim();
        if rest != "{" {
            return Err(startup(
                format!("Zellij plugin sidecar `{name}` block must open with `{name} {{`"),
                path.display(),
                1,
            ));
        }
        match block {
            ZellijPluginBlock::Plugins if seen_plugins => {
                return Err(startup(
                    "Zellij plugin sidecar has duplicate `plugins` blocks",
                    path.display(),
                    1,
                ));
            }
            ZellijPluginBlock::LoadPlugins if seen_load_plugins => {
                return Err(startup(
                    "Zellij plugin sidecar has duplicate `load_plugins` blocks",
                    path.display(),
                    1,
                ));
            }
            ZellijPluginBlock::Plugins => seen_plugins = true,
            ZellijPluginBlock::LoadPlugins => seen_load_plugins = true,
        }
        current = Some(block);
        body_start_line = index + 2;
        depth = 1;
    }

    if current.is_some() {
        return Err(startup(
            "Zellij plugin sidecar has an unclosed block",
            path.display(),
            1,
        ));
    }
    Ok(blocks)
}

fn validate_zellij_plugin_block_ids(
    path: &Path,
    block: ZellijPluginBlock,
    body_start_line: usize,
    body: &str,
) -> Result<(), AppError> {
    let mut depth = 0usize;
    for (index, line) in body.lines().enumerate() {
        if depth == 0 {
            if let Some(id) = first_token(line) {
                if OWNED_ZELLIJ_PLUGIN_IDS.contains(&id) {
                    let block_name = match block {
                        ZellijPluginBlock::Plugins => "plugins",
                        ZellijPluginBlock::LoadPlugins => "load_plugins",
                    };
                    return Err(startup(
                        format!(
                            "Zellij plugin sidecar {block_name} entry `{id}` is owned by Yazelix"
                        ),
                        format!("{}:{}", path.display(), body_start_line + index),
                        1,
                    ));
                }
            }
        }
        depth = zellij_sidecar_depth(path, body_start_line + index, depth, line)?;
    }
    Ok(())
}

fn zellij_sidecar_depth(
    path: &Path,
    line_number: usize,
    depth: usize,
    line: &str,
) -> Result<usize, AppError> {
    let next = depth as isize + zellij_brace_delta(line);
    if next < 0 {
        return Err(startup(
            "Zellij plugin sidecar has an unmatched closing brace",
            format!("{}:{line_number}", path.display()),
            1,
        ));
    }
    Ok(next as usize)
}

fn zellij_brace_delta(line: &str) -> isize {
    zellij_code_before_comment(line)
        .chars()
        .fold((0isize, false, false), |(depth, in_string, escaped), ch| {
            if in_string {
                return match (escaped, ch) {
                    (true, _) => (depth, true, false),
                    (false, '\\') => (depth, true, true),
                    (false, '"') => (depth, false, false),
                    (false, _) => (depth, true, false),
                };
            }
            match ch {
                '"' => (depth, true, false),
                '{' => (depth + 1, false, false),
                '}' => (depth - 1, false, false),
                _ => (depth, false, false),
            }
        })
        .0
}

fn first_token(line: &str) -> Option<&str> {
    let line = zellij_code_before_comment(line).trim_start();
    if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
        return None;
    }
    line.split(|ch: char| ch.is_whitespace() || ch == '{' || ch == ';')
        .next()
        .filter(|token| !token.is_empty())
}

fn zellij_code_before_comment(line: &str) -> &str {
    if line.trim_start().starts_with('#') {
        return "";
    }

    let mut in_string = false;
    let mut escaped = false;
    let mut chars = line.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if in_string {
            match (escaped, ch) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                (false, _) => {}
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '/' if chars.peek().map(|(_, next)| *next == '/').unwrap_or(false) => {
                return &line[..index];
            }
            _ => {}
        }
    }
    line
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

fn render_bar_plugin_block(bar_widgets: &str, shell_label: &str) -> Result<String, AppError> {
    let template_path = Path::new(YZN_BAR_RENDER_REQUEST);
    let template = fs::read_to_string(template_path)
        .map_err(|error| path_error("read", template_path, template_path, error))?;
    let request = template
        .replace(r#""__YZN_BAR_WIDGET_TRAY__""#, bar_widgets)
        .replace("__YZN_SHELL_LABEL__", shell_label);
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
