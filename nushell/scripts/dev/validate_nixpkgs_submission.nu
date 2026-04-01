#!/usr/bin/env nu

use ./nixpkgs_package_smoke.nu *

def build_submission_draft [] {
    with-env { HOME: (make_temp_home) } {
        ^nix build --file packaging/nixpkgs/default.nix --extra-experimental-features "nix-command flakes" --print-out-paths | complete
    }
}

export def main [] {
    let build_result = (build_submission_draft)
    require_success $build_result "Failed to build packaging/nixpkgs/default.nix during nixpkgs submission validation"

    let package_root = ($build_result.stdout | lines | last | str trim)
    if ($package_root | is-empty) {
        error make { msg: "nix build for the nixpkgs submission draft did not return an output path" }
    }

    with-env { SHELL: "/usr/bin/true" } {
        verify_yazelix_package $package_root
    }
    print "✅ Nixpkgs submission draft smoke check passed"
}
