#!/usr/bin/env nu

use common.nu [get_installed_yazelix_runtime_dir get_yazelix_runtime_dir]
use install_ownership.nu [
    get_manual_runtime_reference_path
    get_manual_yzx_cli_path
    has_home_manager_managed_install
    is_manual_runtime_reference_path
    is_manual_yzx_cli_path
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
    runtime_update_guidance: string
    runtime_update_unavailable_reason: string
    runtime_dir?: string
    installed_runtime?: string
    --supports-runtime-update
    --supports-install-artifact-checks
] {
    {
        mode: $mode
        tier: $tier
        title: $title
        runtime_dir: ($runtime_dir | default null)
        installed_runtime: ($installed_runtime | default null)
        supports_runtime_update: $supports_runtime_update
        supports_install_artifact_checks: $supports_install_artifact_checks
        doctor_message: $doctor_message
        doctor_details: $doctor_details
        runtime_update_guidance: $runtime_update_guidance
        runtime_update_unavailable_reason: $runtime_update_unavailable_reason
    }
}

export def get_runtime_distribution_capability_profile [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let installed_runtime = (get_installed_yazelix_runtime_dir)
    let manual_runtime_reference = (get_manual_runtime_reference_path)
    let manual_yzx_cli = (get_manual_yzx_cli_path)
    let home_manager_managed = (has_home_manager_managed_install)
    let installer_managed = (
        ($installed_runtime != null)
        or (is_manual_runtime_reference_path $manual_runtime_reference)
        or (is_manual_yzx_cli_path $manual_yzx_cli)
    )

    if $home_manager_managed {
        return (build_profile
            "home_manager_managed"
            "full"
            "Home Manager-managed full runtime"
            "Runtime/distribution capability: Home Manager-managed full runtime"
            "Home Manager owns the packaged Yazelix runtime path, profile launcher, and runtime repair/update path in this mode. `yzx update runtime` is intentionally unavailable here because Home Manager owns the update transition."
            "Reapply or upgrade the Home Manager configuration that provides Yazelix \(for example `home-manager switch`\)."
            "Home Manager owns Yazelix updates in this mode."
            $runtime_dir
            $installed_runtime
            --supports-install-artifact-checks)
    }

    if $installer_managed {
        return (build_profile
            "installer_managed"
            "full"
            "installer-managed full runtime"
            "Runtime/distribution capability: installer-managed full runtime"
            "The flake installer owns the stable Yazelix runtime identity, `runtime/current`, and the stable `yzx` launcher in this mode. Installer-owned runtime repair and `yzx update runtime` are valid here."
            "Run `yzx update runtime` to refresh the installed runtime."
            ""
            $runtime_dir
            $installed_runtime
            --supports-runtime-update
            --supports-install-artifact-checks)
    }

    if (is_package_runtime_root $runtime_dir) {
        return (build_profile
            "package_runtime"
            "narrowed"
            "store/package runtime"
            "Runtime/distribution capability: store/package runtime"
            "This Yazelix runtime runs directly from a packaged runtime root. Installer-owned `runtime/current` and `~/.local/bin/yzx` repair checks are intentionally skipped here because this mode does not own a mutable installed runtime."
            "Update or reinstall the package that provides Yazelix \(for example `nix profile upgrade`, `home-manager switch`, or a system rebuild\)."
            "This mode runs directly from a packaged runtime root and does not own a mutable installed runtime."
            $runtime_dir
            $installed_runtime)
    }

    (build_profile
        "runtime_root_only"
        "narrowed"
        "runtime-root-only mode"
        "Runtime/distribution capability: runtime-root-only mode"
        "This Yazelix session has a runtime root but no installer-owned distribution surface. Installer-owned `runtime/current` and stable-launcher repair checks are intentionally skipped here because this mode does not own a mutable installed runtime."
        "Use `nix run github:luccahuguet/yazelix#install` to materialize or refresh a full installed runtime, or update the current runtime root manually."
        "This mode does not own a mutable installed runtime."
        $runtime_dir
        $installed_runtime)
}
