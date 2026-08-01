use ratconfig::DEFAULT_CONFIG_SOURCE_ID;

pub(crate) const DEFAULT_CONFIG_TOML: &str = include_str!("../../../defaults/config.toml");

pub(crate) const APPEARANCE_MODE_PATH: &str = "appearance.mode";
pub(crate) const OPEN_LOG_LEVEL_PATH: &str = "open.log_level";
pub(crate) const SHELL_PROGRAM_PATH: &str = "shell.program";
pub(crate) const EDITOR_COMMAND_PATH: &str = "editor.command";
pub(crate) const AGENT_COMMAND_PATH: &str = "agent.command";
pub(crate) const AGENT_ARGS_PATH: &str = "agent.args";
pub(crate) const AGENT_POPUP_KDL_PATH: &str = "agent.popup.kdl";
pub(crate) const AGENT_AUTO_COMMAND: &str = "auto";
pub(crate) const WELCOME_ENABLED_PATH: &str = "welcome.enabled";
pub(crate) const WELCOME_STYLE_PATH: &str = "welcome.style";
pub(crate) const WELCOME_DURATION_SECONDS_PATH: &str = "welcome.duration_seconds";
pub(crate) const WELCOME_STYLE_VALUES: &[&str] = &[
    "static",
    "logo",
    "asciiquarium",
    "boids",
    "boids_predator",
    "boids_schools",
    "mandelbrot",
    "game_of_life_gliders",
    "game_of_life_oscillators",
    "game_of_life_bloom",
    "random",
];
pub(crate) const POPUP_SIDE_MARGIN_PATH: &str = "popup.side_margin";
pub(crate) const POPUP_VERTICAL_MARGIN_PATH: &str = "popup.vertical_margin";
pub(crate) const CUSTOM_POPUPS_KDL_PATH: &str = "popups.kdl";
pub(crate) const CUSTOM_POPUP_KEYBINDINGS_KDL_PATH: &str = "popups.keybindings.kdl";
pub(crate) const KEYBINDINGS_CONFIG_PATH: &str = "keybindings.config";
pub(crate) const KEYBINDINGS_AGENT_PATH: &str = "keybindings.agent";
pub(crate) const KEYBINDINGS_GIT_PATH: &str = "keybindings.git";
pub(crate) const KEYBINDINGS_MENU_PATH: &str = "keybindings.menu";
pub(crate) const KEYBINDINGS_SCREEN_PATH: &str = "keybindings.screen";
pub(crate) const KEYBINDINGS_SIDEBAR_PATH: &str = "keybindings.sidebar";
pub(crate) const KEYBINDINGS_SIDEBAR_FOCUS_PATH: &str = "keybindings.sidebar_focus";
pub(crate) const BAR_WIDGETS_PATH: &str = "bar.widgets";
pub(crate) const BAR_WIDGET_VALUES: &[&str] = &[
    "session",
    "editor",
    "shell",
    "term",
    "claude_usage",
    "codex_usage",
    "opencode_go_usage",
    "cpu",
    "ram",
];
pub(crate) const ROOT_CONFIG_RECOMMENDED_PATHS: &[&str] = &[
    APPEARANCE_MODE_PATH,
    SHELL_PROGRAM_PATH,
    EDITOR_COMMAND_PATH,
    AGENT_COMMAND_PATH,
    WELCOME_ENABLED_PATH,
    WELCOME_STYLE_PATH,
    KEYBINDINGS_CONFIG_PATH,
    KEYBINDINGS_AGENT_PATH,
    KEYBINDINGS_GIT_PATH,
    KEYBINDINGS_MENU_PATH,
    KEYBINDINGS_SCREEN_PATH,
    KEYBINDINGS_SIDEBAR_PATH,
    KEYBINDINGS_SIDEBAR_FOCUS_PATH,
    BAR_WIDGETS_PATH,
];
pub(crate) const ZELLIJ_RECOMMENDED_PATHS: &[&str] = &[
    "theme_dark",
    "theme_light",
    "pane_frames",
    "mouse_mode",
    "copy_on_select",
    "ui.pane_frames.rounded_corners",
];
pub(crate) const STARSHIP_RECOMMENDED_PATHS: &[&str] =
    &["format", "right_format", "add_newline", "character.format"];
pub(crate) const DEFAULT_MARS_CONFIG_TOML: &str =
    include_str!("../../../defaults/mars/config.toml");
pub(crate) const MARS_APPEARANCE_PRESET_PATH: &str = "mars.appearance.preset";
pub(crate) const MARS_RECOMMENDED_PATHS: &[&str] = &[
    "window.width",
    "window.height",
    "window.mode",
    "window.decorations",
    "window.opacity",
    "window.opacity-cells",
    "window.blur",
    "fonts.family",
    "fonts.size",
    "line-height",
    "confirm-before-quit",
    "copy-on-select",
    "hide-mouse-cursor-when-typing",
    "bell.audio",
    "bell.visual",
];
pub(crate) const CURSOR_ENABLED_PATH: &str = "enabled_cursors";
pub(crate) const CURSOR_TRAIL_PATH: &str = "settings.trail";
pub(crate) const CURSOR_RECOMMENDED_PATHS: &[&str] = &[
    CURSOR_ENABLED_PATH,
    CURSOR_TRAIL_PATH,
    "settings.trail_effect",
    "settings.mode_effect",
    "settings.glow",
];
pub(crate) const DEFAULT_STARSHIP_CONFIG_TOML: &str = "\
[character]
format = \":: \"
";
pub(crate) const DEFAULT_HELIX_CONFIG_TOML: &str =
    include_str!("../../../defaults/helix/config.toml");

pub(crate) const SOURCE_CONFIG: &str = DEFAULT_CONFIG_SOURCE_ID;
pub(crate) const SOURCE_MARS: &str = "mars";
pub(crate) const SOURCE_CURSORS: &str = "cursors";
pub(crate) const SOURCE_ZELLIJ: &str = "zellij";
pub(crate) const SOURCE_STARSHIP: &str = "starship";
pub(crate) const SOURCE_HELIX: &str = "helix";
pub(crate) const SOURCE_HELIX_CONFIG: &str = "helix-config";
pub(crate) const SOURCE_HELIX_LANGUAGES: &str = "helix-languages";
pub(crate) const SOURCE_YAZI: &str = "yazi";
pub(crate) const SOURCE_YAZI_CONFIG: &str = "yazi-config";
pub(crate) const SOURCE_YAZI_THEME: &str = "yazi-theme";
pub(crate) const SOURCE_KEYS: &str = "keys";
pub(crate) const SOURCE_ADVANCED: &str = "advanced";
pub(crate) const TAB_CONFIG: &str = " main";
pub(crate) const TAB_POPUPS: &str = " popups";
pub(crate) const TAB_MARS: &str = " mars";
pub(crate) const TAB_CURSORS: &str = "󰇀 cursors";
pub(crate) const TAB_ZELLIJ: &str = " zellij";
pub(crate) const TAB_STARSHIP: &str = " starship";
pub(crate) const TAB_HELIX: &str = " helix";
pub(crate) const TAB_YAZI: &str = "󰇥 yazi";
pub(crate) const TAB_KEYS: &str = " keys";
pub(crate) const TAB_ADVANCED: &str = "advanced";

pub(crate) const ACTION_HELIX_CONFIG: &str = "helix.config";
pub(crate) const ACTION_ROOT_CONFIG: &str = "config.root";
pub(crate) const ACTION_CURSORS_CONFIG: &str = "cursors.config";
pub(crate) const ACTION_HELIX_LANGUAGES: &str = "helix.languages";
pub(crate) const ACTION_HELIX_MODULE: &str = "helix.module";
pub(crate) const ACTION_HELIX_INIT: &str = "helix.init";
pub(crate) const ACTION_NU_ENV: &str = "nu.env";
pub(crate) const ACTION_NU_CONFIG: &str = "nu.config";
pub(crate) const ACTION_STARSHIP_CONFIG: &str = "starship.config";
pub(crate) const ACTION_YAZI_CONFIG: &str = "yazi.config";
pub(crate) const ACTION_YAZI_INIT: &str = "yazi.init";
pub(crate) const ACTION_YAZI_KEYMAP: &str = "yazi.keymap";
pub(crate) const ACTION_YAZI_PACKAGE: &str = "yazi.package";
pub(crate) const ACTION_YAZI_THEME: &str = "yazi.theme";
pub(crate) const ACTION_ZELLIJ_CONFIG: &str = "zellij.config";
pub(crate) const ACTION_ZELLIJ_PLUGINS: &str = "zellij.plugins";
pub(crate) const HELIX_CONFIG_STARTER: &str =
    "# User overrides layered over Yazelix Nova packaged Helix config.\n";
pub(crate) const HELIX_LANGUAGES_STARTER: &str = "# Managed Helix language overrides.\n";
pub(crate) const HELIX_MODULE_STARTER: &str = ";; Loaded by managed yzx-hx before init.scm.\n";
pub(crate) const HELIX_INIT_STARTER: &str = ";; Loaded by managed yzx-hx at startup.\n";
pub(crate) const NU_ENV_STARTER: &str = "# Loaded after Yazelix Nova packaged env.nu.\n";
pub(crate) const NU_CONFIG_STARTER: &str = "# Loaded after Yazelix Nova packaged config.nu.\n";
pub(crate) const STARSHIP_CONFIG_STARTER: &str =
    "# Sparse overrides layered over packaged Starship defaults.\n";
pub(crate) const YAZI_CONFIG_STARTER: &str = "# Extended over Yazelix Nova packaged yazi.toml.\n";
pub(crate) const YAZI_INIT_STARTER: &str = "-- Loaded after Yazelix Nova packaged yazi/init.lua.\n";
pub(crate) const YAZI_KEYMAP_STARTER: &str =
    "# Loaded after Yazelix Nova packaged yazi/keymap.toml.\n";
pub(crate) const YAZI_PACKAGE_STARTER: &str = "# Managed Yazi package metadata. Yazelix does not run ya pkg.\n[plugin]\ndeps = []\n\n[flavor]\ndeps = []\n";
pub(crate) const YAZI_THEME_STARTER: &str = "# Managed native Yazi theme config.\n";
pub(crate) const ZELLIJ_CONFIG_STARTER: &str =
    "// Sparse native Zellij overrides layered over Yazelix packaged configuration.\n";
pub(crate) const ZELLIJ_PLUGINS_STARTER: &str = "// Extra managed Zellij plugins. Do not declare yzpp or yazelix_pane_orchestrator here.\nplugins {\n}\n\nload_plugins {\n}\n";
pub(crate) const KEY_READ_ONLY_REASON: &str =
    "Read-only key binding; yzx config does not rewrite native keymaps.";

pub(crate) const MANAGED_KEYBINDINGS: &[(&str, &str)] = &[
    (KEYBINDINGS_CONFIG_PATH, "Alt Shift K"),
    (KEYBINDINGS_AGENT_PATH, "Alt Shift L"),
    (KEYBINDINGS_GIT_PATH, "Alt Shift J"),
    (KEYBINDINGS_MENU_PATH, "Alt Shift M"),
    (KEYBINDINGS_SCREEN_PATH, "Alt Shift S"),
    (KEYBINDINGS_SIDEBAR_PATH, "Alt Shift H"),
    (KEYBINDINGS_SIDEBAR_FOCUS_PATH, "Ctrl y"),
];

macro_rules! key {
    ($group:literal; $chord:literal; $action:literal; $owner:literal; $source:literal) => {
        [$group, $chord, $action, $owner, $source]
    };
}

pub(crate) const KEY_BINDINGS: &[[&str; 5]] = &[
    key!("Workspace"; "Ctrl Alt g"; "Toggle locked mode"; "Zellij"; "config.kdl"),
    key!("Workspace"; "Ctrl Alt o"; "Open session mode"; "Zellij"; "config.kdl"),
    key!("Workspace"; "Ctrl q"; "Quit Yazelix session"; "Zellij"; "config.kdl"),
    key!("Panes"; "Ctrl p"; "Toggle pane mode"; "Zellij"; "config.kdl"),
    key!("Panes"; "Ctrl n"; "Toggle resize mode"; "Zellij"; "config.kdl"),
    key!("Panes"; "Alt m"; "Open a new pane"; "Zellij"; "config.kdl"),
    key!("Panes"; "Alt h / Alt Left"; "Move focus left or previous tab"; "Yazelix"; "config.kdl"),
    key!("Panes"; "Alt l / Alt Right"; "Move focus right or next tab"; "Yazelix"; "config.kdl"),
    key!("Panes"; "Alt Shift F"; "Toggle focused pane fullscreen"; "Zellij"; "config.kdl"),
    key!("Sidebar"; "Ctrl y"; "Toggle editor/sidebar focus"; "Yazelix"; "config.kdl"),
    key!("Editor / Yazi"; "Alt r"; "Reveal editor file or close focused Yazi popup"; "Yazelix"; "config.kdl"),
    key!("Tabs"; "Ctrl t"; "Toggle tab mode"; "Zellij"; "config.kdl"),
    key!("Tabs"; "Alt 1-9"; "Go directly to tab 1-9"; "Zellij"; "config.kdl"),
    key!("Tabs"; "n in tab mode"; "Open a new tab"; "Zellij"; "config.kdl"),
    key!("Tabs"; "Ctrl Alt h"; "Move tab left"; "Zellij"; "config.kdl"),
    key!("Panes"; "Ctrl Alt j"; "Move pane down"; "Zellij"; "config.kdl"),
    key!("Panes"; "Ctrl Alt k"; "Move pane up"; "Zellij"; "config.kdl"),
    key!("Tabs"; "Ctrl Alt l"; "Move tab right"; "Zellij"; "config.kdl"),
    key!("Popups"; "Alt Shift J"; "Toggle Git popup"; "Yazelix"; "config.kdl"),
    key!("Popups"; "Alt Shift K"; "Toggle config popup"; "Yazelix"; "config.kdl"),
    key!("Popups"; "Alt Shift L"; "Hide or show agent popup"; "Yazelix"; "config.kdl"),
    key!("Popups"; "Alt Shift M"; "Toggle menu popup"; "Yazelix"; "config.kdl"),
    key!("Popups"; "Alt Shift S"; "Show a random full-screen visual"; "Yazelix"; "config.kdl"),
    key!("Popups"; "Alt Shift Y"; "Hide or show Yazi popup"; "Yazelix"; "config.kdl"),
    key!("Sidebar"; "Alt Shift H"; "Toggle Yazi sidebar"; "Yazelix"; "config.kdl"),
    key!("File manager"; "Alt z"; "Retarget tab workspace with zoxide"; "Yazi"; "yazi/keymap.toml"),
];

pub(crate) const KEY_COLUMNS: &[(&str, usize)] =
    &[("group", 14), ("key", 20), ("action", 40), ("owner", 10)];

pub(crate) const CONFIG_FIELDS: &[ConfigFieldSpec] = &[
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            APPEARANCE_MODE_PATH,
            "Appearance shared by managed Yazelix components.",
            &["dark", "light"],
            "dark or light",
        ),
        apply_summary: "live",
        apply_detail: "Saved values update writable regular-file component config and apply on the next launch when a component config is externally managed or read-only.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            OPEN_LOG_LEVEL_PATH,
            "Diagnostics written by yzx-open for managed Yazi open requests.",
            &["off", "error", "info", "debug"],
            "off, error, info, or debug",
        ),
        apply_summary: "new opens",
        apply_detail: "Saved values are exported as YZX_OPEN_LOG for managed Yazi opens.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            SHELL_PROGRAM_PATH,
            "Packaged Nushell, Bash, Zsh, or Fish launched in new Zellij panes.",
            &["nu", "bash", "zsh", "fish"],
            "nu, bash, zsh, or fish",
        ),
        apply_summary: "new panes",
        apply_detail: "Saved shell selection applies to newly launched panes and sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            EDITOR_COMMAND_PATH,
            "Editor command used by managed file opens. Use hx or yzx-hx for managed Yazelix Helix when included, or another installed executable such as nvim.",
            &[],
            "one non-empty executable command without arguments",
        ),
        apply_summary: "new opens",
        apply_detail: "Saved editor command applies to newly launched managed Yazi opens.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            AGENT_COMMAND_PATH,
            "Command for the managed agent popup. Use auto for the built-in provider fallback.",
            &[],
            "auto or one non-empty executable command without arguments",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved agent command applies to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_list(
            AGENT_ARGS_PATH,
            "Arguments passed to a custom managed agent popup command.",
            "JSON string array; requires agent.command to be custom",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved agent arguments apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::boolean(
            WELCOME_ENABLED_PATH,
            "Show the startup welcome splash before entering the managed runtime.",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved welcome settings apply to newly launched sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            WELCOME_STYLE_PATH,
            "Startup welcome style.",
            WELCOME_STYLE_VALUES,
            "known welcome style id",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved welcome settings apply to newly launched sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::integer(
            WELCOME_DURATION_SECONDS_PATH,
            "Startup welcome duration.",
            "integer from 1 to 60 seconds",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved welcome settings apply to newly launched sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::integer(
            POPUP_SIDE_MARGIN_PATH,
            "Left and right cell margin for managed popups. Set to 1 for a little margin.",
            "non-negative integer",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved popup margins apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::integer(
            POPUP_VERTICAL_MARGIN_PATH,
            "Top and bottom cell margin for managed popups. Set to 1 for a little margin.",
            "non-negative integer",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved popup margins apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::managed_keybinding(
            KEYBINDINGS_CONFIG_PATH,
            "Key chord that toggles the managed config popup. Set false to leave it unmapped.",
            "key chord like Alt Shift A that does not conflict with a packaged binding, or false",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::managed_keybinding(
            KEYBINDINGS_AGENT_PATH,
            "Key chord that hides or shows the managed agent popup. Set false to leave it unmapped.",
            "key chord like Alt Shift A that does not conflict with a packaged binding, or false",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::managed_keybinding(
            KEYBINDINGS_GIT_PATH,
            "Key chord that toggles the managed Git popup. Set false to leave it unmapped.",
            "key chord like Alt Shift A that does not conflict with a packaged binding, or false",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::managed_keybinding(
            KEYBINDINGS_MENU_PATH,
            "Key chord that toggles the managed command palette popup. Set false to leave it unmapped.",
            "key chord like Alt Shift A that does not conflict with a packaged binding, or false",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::managed_keybinding(
            KEYBINDINGS_SCREEN_PATH,
            "Key chord that opens a random full-screen visual. Set false to leave it unmapped.",
            "key chord like Alt Shift A that does not conflict with a packaged binding, or false",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::managed_keybinding(
            KEYBINDINGS_SIDEBAR_PATH,
            "Key chord that hides or shows the managed Yazi sidebar. Set false to leave it unmapped.",
            "key chord like Alt Shift A that does not conflict with a packaged binding, or false",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::managed_keybinding(
            KEYBINDINGS_SIDEBAR_FOCUS_PATH,
            "Key chord that toggles focus between the editor and managed Yazi sidebar. Set false to leave it unmapped.",
            "key chord like Ctrl y that does not conflict with a packaged binding, or false",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
];

pub(crate) const ZELLIJ_FORBIDDEN_TOP_LEVEL: &[&str] = &[
    "keybinds",
    "plugins",
    "load_plugins",
    "default_shell",
    "default_layout",
    "layout",
    "support_kitty_keyboard_protocol",
    "env",
    "session_name",
    "attach_to_session",
];

pub(crate) const ZELLIJ_FIELDS: &[FieldSpec] = &[
    FieldSpec::string_choice(
        "theme_dark",
        "Zellij theme used for dark Yazelix appearance. Custom names remain valid in the native sidecar.",
        &[],
        "packaged theme choice; custom sidecar names remain accepted",
    ),
    FieldSpec::string_choice(
        "theme_light",
        "Zellij theme used for light Yazelix appearance. Custom names remain valid in the native sidecar.",
        &[],
        "packaged theme choice; custom sidecar names remain accepted",
    ),
    FieldSpec::boolean("pane_frames", "Show Zellij pane frames."),
    FieldSpec::boolean("mouse_mode", "Enable mouse support in Zellij."),
    FieldSpec::integer(
        "scroll_buffer_size",
        "Lines kept in Zellij scrollback.",
        "positive integer",
    ),
    FieldSpec::boolean("copy_on_select", "Copy selected text automatically."),
    FieldSpec::string_choice(
        "copy_clipboard",
        "Clipboard target for Zellij copy operations.",
        &["system", "primary"],
        "system or primary",
    ),
    FieldSpec::boolean(
        "styled_underlines",
        "Render styled underlines in Zellij panes.",
    ),
    FieldSpec::boolean("show_startup_tips", "Show Zellij startup tips."),
    FieldSpec::boolean(
        "ui.pane_frames.rounded_corners",
        "Use rounded Zellij pane frame corners.",
    ),
];

#[derive(Debug, Clone, Copy)]
pub(crate) struct ConfigFieldSpec {
    pub(crate) field: FieldSpec,
    pub(crate) apply_summary: &'static str,
    pub(crate) apply_detail: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldSpec {
    pub(crate) path: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) description: &'static str,
    pub(crate) allowed_values: &'static [&'static str],
    pub(crate) validation: &'static str,
}

impl FieldSpec {
    const fn boolean(path: &'static str, description: &'static str) -> Self {
        Self::new(path, "boolean", description, &[], "true or false")
    }

    const fn integer(
        path: &'static str,
        description: &'static str,
        validation: &'static str,
    ) -> Self {
        Self::new(path, "integer", description, &[], validation)
    }

    const fn string_choice(
        path: &'static str,
        description: &'static str,
        allowed_values: &'static [&'static str],
        validation: &'static str,
    ) -> Self {
        Self::new(path, "string", description, allowed_values, validation)
    }

    const fn managed_keybinding(
        path: &'static str,
        description: &'static str,
        validation: &'static str,
    ) -> Self {
        Self::new(path, "key chord or false", description, &[], validation)
    }

    const fn string_list(
        path: &'static str,
        description: &'static str,
        validation: &'static str,
    ) -> Self {
        Self::new(path, "string_list", description, &[], validation)
    }

    const fn new(
        path: &'static str,
        kind: &'static str,
        description: &'static str,
        allowed_values: &'static [&'static str],
        validation: &'static str,
    ) -> Self {
        Self {
            path,
            kind,
            description,
            allowed_values,
            validation,
        }
    }
}
