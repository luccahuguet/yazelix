//! Re-exports of the child-owned cursor registry contract used by Yazelix.

pub use yazelix_cursors::{
    CursorColor, CursorDefinition, CursorFamily, CursorRegistry, CursorSettings,
    DEFAULT_CURSOR_CONFIG_FILENAME, DEFAULT_GHOSTTY_TRAIL_DURATION, GHOSTTY_TRAIL_DURATION_MAX,
    GHOSTTY_TRAIL_DURATION_MIN, ResolvedCursorRegistryState, SplitDivider, SplitTransition,
    format_ghostty_trail_duration, load_cursor_config, write_ghostty_cursor_effect_shaders,
    write_ghostty_cursor_palette_shaders,
};
