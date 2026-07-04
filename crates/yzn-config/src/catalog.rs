use ratconfig::DEFAULT_CONFIG_SOURCE_ID;

pub(crate) const DEFAULT_CONFIG_TOML: &str = include_str!("../../../config.toml");
pub(crate) const CONTRACT_ID: &str = "yazelix-next.config";
pub(crate) const CONTRACT_STATE_PATH: &str = "ratconfig.contract";
pub(crate) const CONTRACT_VERSION: u64 = 1;

pub(crate) const OPEN_LOG_LEVEL_PATH: &str = "open.log_level";
pub(crate) const SHELL_PROGRAM_PATH: &str = "shell.program";
pub(crate) const EDITOR_COMMAND_PATH: &str = "editor.command";
pub(crate) const WELCOME_ENABLED_PATH: &str = "welcome.enabled";
pub(crate) const WELCOME_STYLE_PATH: &str = "welcome.style";
pub(crate) const WELCOME_DURATION_SECONDS_PATH: &str = "welcome.duration_seconds";
pub(crate) const WELCOME_STYLE_VALUES: &[&str] = &[
    "static",
    "logo",
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
pub(crate) const DEFAULT_CONFIG_KEYBINDING: &str = "Alt Shift K";
pub(crate) const DEFAULT_AGENT_KEYBINDING: &str = "Alt Shift L";
pub(crate) const DEFAULT_GIT_KEYBINDING: &str = "Alt Shift J";
pub(crate) const DEFAULT_MENU_KEYBINDING: &str = "Alt Shift M";
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
pub(crate) const DEFAULT_MARS_CONFIG_TOML: &str = include_str!("../../../mars.toml");
pub(crate) const DEFAULT_STARSHIP_CONFIG_TOML: &str = "\
format = \":: \"
right_format = \"\"
add_newline = true
";

pub(crate) const SOURCE_CONFIG: &str = DEFAULT_CONFIG_SOURCE_ID;
pub(crate) const SOURCE_MARS: &str = "mars";
pub(crate) const SOURCE_ZELLIJ: &str = "zellij";
pub(crate) const SOURCE_STARSHIP: &str = "starship";
pub(crate) const SOURCE_HELIX: &str = "helix";
pub(crate) const SOURCE_KEYS: &str = "keys";
pub(crate) const SOURCE_ADVANCED: &str = "advanced";
pub(crate) const TAB_CONFIG: &str = " main";
pub(crate) const TAB_MARS: &str = " mars";
pub(crate) const TAB_ZELLIJ: &str = " zellij";
pub(crate) const TAB_STARSHIP: &str = " starship";
pub(crate) const TAB_HELIX: &str = " helix";
pub(crate) const TAB_KEYS: &str = " keys";
pub(crate) const TAB_ADVANCED: &str = " advanced";

pub(crate) const ACTION_HELIX_CONFIG: &str = "helix.config";
pub(crate) const ACTION_HELIX_LANGUAGES: &str = "helix.languages";
pub(crate) const ACTION_HELIX_MODULE: &str = "helix.module";
pub(crate) const ACTION_HELIX_INIT: &str = "helix.init";
pub(crate) const ACTION_NU_ENV: &str = "nu.env";
pub(crate) const ACTION_NU_CONFIG: &str = "nu.config";
pub(crate) const ACTION_YAZI_INIT: &str = "yazi.init";
pub(crate) const ACTION_YAZI_KEYMAP: &str = "yazi.keymap";
pub(crate) const ACTION_ZELLIJ_PLUGINS: &str = "zellij.plugins";
pub(crate) const HELIX_CONFIG_STARTER: &str = include_str!("../../../helix/config.toml");
pub(crate) const HELIX_LANGUAGES_STARTER: &str = "# Managed Helix language overrides.\n";
pub(crate) const HELIX_MODULE_STARTER: &str = ";; Loaded by managed yzn-hx before init.scm.\n";
pub(crate) const HELIX_INIT_STARTER: &str = ";; Loaded by managed yzn-hx at startup.\n";
pub(crate) const NU_ENV_STARTER: &str = "# Loaded after Yazelix Next packaged env.nu.\n";
pub(crate) const NU_CONFIG_STARTER: &str = "# Loaded after Yazelix Next packaged config.nu.\n";
pub(crate) const YAZI_INIT_STARTER: &str = "-- Loaded after Yazelix Next packaged yazi/init.lua.\n";
pub(crate) const YAZI_KEYMAP_STARTER: &str =
    "# Loaded after Yazelix Next packaged yazi/keymap.toml.\n";
pub(crate) const ZELLIJ_PLUGINS_STARTER: &str = "// Extra managed Zellij plugins. Do not declare yzpp or yazelix_pane_orchestrator here.\nplugins {\n}\n\nload_plugins {\n}\n";
pub(crate) const KEY_READ_ONLY_REASON: &str =
    "Read-only key binding; yzn config does not rewrite native keymaps.";

pub(crate) struct PopupKeybindingSpec {
    pub(crate) path: &'static str,
    pub(crate) default: &'static str,
}

pub(crate) const POPUP_KEYBINDINGS: &[PopupKeybindingSpec] = &[
    PopupKeybindingSpec {
        path: KEYBINDINGS_CONFIG_PATH,
        default: DEFAULT_CONFIG_KEYBINDING,
    },
    PopupKeybindingSpec {
        path: KEYBINDINGS_AGENT_PATH,
        default: DEFAULT_AGENT_KEYBINDING,
    },
    PopupKeybindingSpec {
        path: KEYBINDINGS_GIT_PATH,
        default: DEFAULT_GIT_KEYBINDING,
    },
    PopupKeybindingSpec {
        path: KEYBINDINGS_MENU_PATH,
        default: DEFAULT_MENU_KEYBINDING,
    },
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
    key!("Sidebar"; "Alt r"; "Reveal editor file in Yazi"; "Yazelix"; "config.kdl"),
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
    key!("Sidebar"; "Alt Shift h"; "Toggle Yazi sidebar"; "Yazelix"; "config.kdl"),
    key!("File manager"; "Alt z"; "Zoxide jump into the managed editor"; "Yazi"; "yazi/keymap.toml"),
];

pub(crate) const KEY_COLUMNS: &[(&str, usize)] =
    &[("group", 14), ("key", 20), ("action", 40), ("owner", 10)];

pub(crate) const CONFIG_FIELDS: &[ConfigFieldSpec] = &[
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            OPEN_LOG_LEVEL_PATH,
            "Diagnostics written by yzn-open for managed Yazi open requests.",
            &["off", "error", "info", "debug"],
            "off, error, info, or debug",
        ),
        apply_summary: "new opens",
        apply_detail: "Saved values are exported as YZN_OPEN_LOG for managed Yazi opens.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            SHELL_PROGRAM_PATH,
            "Packaged shell launched in new Zellij panes.",
            &["nu", "bash", "zsh", "fish"],
            "nu, bash, zsh, or fish",
        ),
        apply_summary: "new panes",
        apply_detail: "Saved shell selection applies to newly launched panes and sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            EDITOR_COMMAND_PATH,
            "Editor command used by managed Yazi opens. Use yzn-hx for packaged Yazelix Helix, or a host executable such as nvim.",
            &[],
            "one non-empty executable command without arguments",
        ),
        apply_summary: "new opens",
        apply_detail: "Saved editor command applies to newly launched managed Yazi opens.",
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
        field: FieldSpec::string_choice(
            KEYBINDINGS_CONFIG_PATH,
            "Key chord that toggles the managed config popup.",
            &[],
            "key chord like Alt Shift A that does not conflict with a packaged binding",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            KEYBINDINGS_AGENT_PATH,
            "Key chord that hides or shows the managed agent popup.",
            &[],
            "key chord like Alt Shift A that does not conflict with a packaged binding",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            KEYBINDINGS_GIT_PATH,
            "Key chord that toggles the managed Git popup.",
            &[],
            "key chord like Alt Shift A that does not conflict with a packaged binding",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
    ConfigFieldSpec {
        field: FieldSpec::string_choice(
            KEYBINDINGS_MENU_PATH,
            "Key chord that toggles the managed command palette popup.",
            &[],
            "key chord like Alt Shift A that does not conflict with a packaged binding",
        ),
        apply_summary: "next launch",
        apply_detail: "Saved keybindings apply to newly launched Yazelix sessions.",
    },
];

pub(crate) const MARS_FIELDS: &[FieldSpec] = &[
    FieldSpec::string_choice(
        "force-theme",
        "Force the Mars window theme.",
        &["dark", "light"],
        "dark or light",
    ),
    FieldSpec::string_choice(
        "colors.background",
        "Mars terminal background color.",
        &[],
        "hex color like #111416",
    ),
    FieldSpec::string_choice(
        "colors.foreground",
        "Mars terminal foreground color.",
        &[],
        "hex color like #eeeeec",
    ),
    FieldSpec::string_choice(
        "colors.dim-foreground",
        "Mars dim foreground color.",
        &[],
        "hex color like #9d9d9c",
    ),
    FieldSpec::string_choice(
        "yazelix.cursor.divider",
        "Mars Yazelix split cursor divider.",
        &["vertical", "horizontal"],
        "vertical or horizontal",
    ),
    FieldSpec::string_list(
        "yazelix.cursor.colors",
        "Mars Yazelix split cursor colors.",
        "exactly two hex colors like [\"#00e6ff\", \"#00ff66\"]",
    ),
    FieldSpec::string_choice(
        "yazelix.cursor.cursor_color",
        "Mars Yazelix cursor color.",
        &[],
        "hex color like #00e6ff",
    ),
    FieldSpec::integer("window.width", "Initial Mars window width.", "pixels"),
    FieldSpec::integer("window.height", "Initial Mars window height.", "pixels"),
    FieldSpec::float("window.opacity", "Mars window opacity.", "0.0 to 1.0"),
    FieldSpec::float("fonts.size", "Mars font size.", "points"),
    FieldSpec::float("line-height", "Mars line height multiplier.", "multiplier"),
    FieldSpec::boolean("enable-scroll-bar", "Show the Mars scrollbar."),
    FieldSpec::boolean("bell.audio", "Play the Mars terminal bell."),
    FieldSpec::boolean("bell.visual", "Flash the Mars visual bell."),
    FieldSpec::boolean("effects.trail-cursor", "Draw the Mars cursor trail."),
];

pub(crate) const STARSHIP_FIELDS: &[FieldSpec] = &[
    FieldSpec::string_choice(
        "format",
        "Left prompt format string.",
        &[],
        "Starship format string",
    ),
    FieldSpec::string_choice(
        "right_format",
        "Right prompt format string.",
        &[],
        "Starship format string",
    ),
    FieldSpec::boolean("add_newline", "Insert a blank line before the prompt."),
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

    const fn float(
        path: &'static str,
        description: &'static str,
        validation: &'static str,
    ) -> Self {
        Self::new(path, "float", description, &[], validation)
    }

    const fn string_choice(
        path: &'static str,
        description: &'static str,
        allowed_values: &'static [&'static str],
        validation: &'static str,
    ) -> Self {
        Self::new(path, "string", description, allowed_values, validation)
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
