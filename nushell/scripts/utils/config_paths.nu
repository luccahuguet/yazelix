#!/usr/bin/env nu
# Non-owning managed config path helpers.

use runtime_paths.nu get_yazelix_user_config_dir

export const MAIN_CONFIG_FILENAME = "yazelix.toml"

export def get_main_user_config_path [config_root?: string] {
    (get_yazelix_user_config_dir $config_root) | path join $MAIN_CONFIG_FILENAME
}
