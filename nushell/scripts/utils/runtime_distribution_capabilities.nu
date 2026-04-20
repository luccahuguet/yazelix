#!/usr/bin/env nu

use common.nu get_yazelix_runtime_dir
use install_ownership_report.nu evaluate_install_ownership_report

def is_package_runtime_root [runtime_dir?: string] {
    if $runtime_dir == null {
        return false
    }

    let candidate = ($runtime_dir | path expand)
    (
        (($candidate | path join "yazelix_default.toml") | path exists)
        and (($candidate | path join "bin" "yzx") | path exists)
        and (($candidate | path join "libexec" "nu") | path exists)
    )
}

def build_profile [
    mode: string
    tier: string
    title: string
    doctor_message: string
    doctor_details: string
    runtime_dir?: string
] {
    {
        mode: $mode
        tier: $tier
        title: $title
        runtime_dir: ($runtime_dir | default null)
        doctor_message: $doctor_message
        doctor_details: $doctor_details
    }
}

export def get_runtime_distribution_capability_profile [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let io = (evaluate_install_ownership_report)
    let home_manager_managed = $io.has_home_manager_managed_install
    let installer_managed = $io.is_manual_runtime_reference_path

    if $home_manager_managed {
        return (build_profile
            "home_manager_managed"
            "full"
            "Home Manager-managed full runtime"
            "Runtime/distribution capability: Home Manager-managed full runtime"
            "Home Manager owns the packaged Yazelix runtime path and update transition in this mode."
            $runtime_dir
        )
    }

    if $installer_managed {
        return (build_profile
            "installer_managed"
            "full"
            "compatibility installer runtime"
            "Runtime/distribution capability: compatibility installer runtime"
            "This runtime still has legacy installer-owned artifacts from older releases. Current Yazelix no longer ships `#install`; reinstall into a Nix profile or move to Home Manager."
            $runtime_dir
        )
    }

    if (is_package_runtime_root $runtime_dir) {
        return (build_profile
            "package_runtime"
            "narrowed"
            "store/package runtime"
            "Runtime/distribution capability: store/package runtime"
            "This Yazelix runtime runs directly from a packaged runtime root."
            $runtime_dir
        )
    }

    (build_profile
        "runtime_root_only"
        "narrowed"
        "runtime-root-only mode"
        "Runtime/distribution capability: runtime-root-only mode"
        "This Yazelix session has a runtime root but no package-manager-owned distribution surface."
        $runtime_dir
    )
}
