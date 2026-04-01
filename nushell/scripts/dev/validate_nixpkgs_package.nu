#!/usr/bin/env nu

use ./nixpkgs_package_smoke.nu *

def build_yazelix_package [] {
    with-env { HOME: (make_temp_home) } {
        ^nix build .#yazelix --print-out-paths --extra-experimental-features "nix-command flakes" | complete
    }
}

export def main [] {
    let build_result = (build_yazelix_package)
    require_success $build_result "Failed to build .#yazelix during nixpkgs package validation"

    let package_root = ($build_result.stdout | lines | last | str trim)
    if ($package_root | is-empty) {
        error make { msg: "nix build .#yazelix did not return an output path" }
    }

    with-env { SHELL: "/usr/bin/true" } {
        verify_yazelix_package $package_root
    }
    print "✅ Nixpkgs-style Yazelix package smoke check passed"
}
