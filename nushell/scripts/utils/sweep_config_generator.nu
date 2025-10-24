#!/usr/bin/env nu
# Sweep Testing - Configuration Generation Utilities
# Generates temporary Yazelix configurations for testing different combinations

# Generate temporary yazelix.nix config for testing
export def generate_sweep_config [
    shell: string,
    terminal: string,
    features: record,
    test_id: string
]: nothing -> string {
    let temp_dir = $"($env.HOME)/.local/share/yazelix/sweep_tests"
    mkdir $temp_dir

    let config_path = $"($temp_dir)/yazelix_test_($test_id).nix"

    let config_content = $"{ pkgs }:
{
  # Sweep test configuration - ($test_id)
  # Shell: ($shell), Terminal: ($terminal)

  # Core settings
  default_shell = \"($shell)\";
  preferred_terminal = \"($terminal)\";
  helix_mode = \"($features.helix_mode)\";

  # Feature flags
  enable_sidebar = ($features.enable_sidebar);
  persistent_sessions = ($features.persistent_sessions);
  recommended_deps = ($features.recommended_deps);
  yazi_extensions = ($features.yazi_extensions);
  yazi_media = false;  # Keep minimal for testing

  # Disable features that might cause issues in testing
  debug_mode = false;
  skip_welcome_screen = true;  # Suppress output for clean testing
  enable_atuin = false;
  disable_zellij_tips = true;  # Prevent tips popup during visual testing

  # Minimal extras for testing
  extra_shells = [];
  extra_terminals = [];
  packs = [];
  user_packages = with pkgs; [];

  # Terminal config mode
  terminal_config_mode = \"yazelix\";

  # Session settings
  session_name = \"sweep_test_($test_id)\";

  # Force sweep test layout - uses yzx_sweep_test for testing
  zellij_layout_override = \"yzx_sweep_test\";

  # Appearance \(minimal\)
  cursor_trail = \"none\";
  transparency = \"none\";
  ascii_art_mode = \"static\";
  show_macchina_on_welcome = false;
}
"

    $config_content | save --force $config_path
    $config_path
}

# Clean up temporary test configs
export def cleanup_sweep_configs []: nothing -> nothing {
    let temp_dir = $"($env.HOME)/.local/share/yazelix/sweep_tests"
    if ($temp_dir | path exists) {
        rm -rf $temp_dir
    }
}