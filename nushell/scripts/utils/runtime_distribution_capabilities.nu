#!/usr/bin/env nu

use common.nu get_yazelix_runtime_dir
use install_ownership.nu [
    get_manual_runtime_reference_path
    has_home_manager_managed_install
    is_manual_runtime_reference_path
]

def is_package_runtime_root [runtime_dir?: string] {
    if $runtime_dir == null {
        return false
    }

    let candidate = ($runtime_dir | path expand)
    (
        (($candidate | path join "yazelix_default.toml") | path exists)
        and (($candidate | path join "bin" "yzx") | path exists)
        and (($candidate | path join "bin" "nu") | path exists)
    )
}

def build_profile [
    mode: string
    tier: string
    title: string
    doctor_message: string
    doctor_details: string
    update_guidance: string
    runtime_dir?: string
] {
    {
        mode: $mode
        tier: $tier
        title: $title
        runtime_dir: ($runtime_dir | default null)
        doctor_message: $doctor_message
        doctor_details: $doctor_details
        update_guidance: $update_guidance
    }
}

export def get_runtime_distribution_capability_profile [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let manual_runtime_reference = (get_manual_runtime_reference_path)
    let home_manager_managed = (has_home_manager_managed_install)
    let installer_managed = (is_manual_runtime_reference_path $manual_runtime_reference)

    if $home_manager_managed {
        return (build_profile
            "home_manager_managed"
            "full"
            "Home Manager-managed full runtime"
            "Runtime/distribution capability: Home Manager-managed full runtime"
            "Home Manager owns the packaged Yazelix runtime path and update transition in this mode."
            "Reapply or upgrade the Home Manager configuration that provides Yazelix \(for example `home-manager switch`\)."
            $runtime_dir
        )
    }

    if $installer_managed {
        return (build_profile
            "installer_managed"
            "full"
            "compatibility installer runtime"
            "Runtime/distribution capability: compatibility installer runtime"
            "This runtime still has legacy installer-owned artifacts, but Yazelix no longer owns an in-app runtime updater."
            "If you still use the compatibility installer path, rerun `nix run github:luccahuguet/yazelix#install`, or prefer a package-manager update flow."
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
            "Update or reinstall the package that provides Yazelix \(for example `nix profile upgrade`, `home-manager switch`, or a system rebuild\)."
            $runtime_dir
        )
    }

    (build_profile
        "runtime_root_only"
        "narrowed"
        "runtime-root-only mode"
        "Runtime/distribution capability: runtime-root-only mode"
        "This Yazelix session has a runtime root but no package-manager-owned distribution surface."
        "Refresh the current runtime root manually, or switch to the packaged `#yazelix` surface or Home Manager."
        $runtime_dir
    )
}
