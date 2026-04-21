#!/usr/bin/env nu

use install_ownership.nu evaluate_install_ownership_report

export def get_home_manager_yzx_profile_paths [] {
    (evaluate_install_ownership_report).home_manager_profile_yzx_candidates
}

export def get_existing_home_manager_yzx_profile_path [] {
    (evaluate_install_ownership_report).existing_home_manager_profile_yzx
}

export def resolve_stable_yzx_wrapper_path [] {
    (evaluate_install_ownership_report).stable_yzx_wrapper
}

export def resolve_desktop_launcher_path [runtime_dir: string] {
    (evaluate_install_ownership_report --runtime-dir $runtime_dir).desktop_launcher_path
}
