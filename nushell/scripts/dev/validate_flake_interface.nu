#!/usr/bin/env nu

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)

export def main [] {
    let repo_root_literal = (($REPO_ROOT | into string) | to json -r)
    let expr = (
        "let\n"
        + "  flake = builtins.getFlake " + $repo_root_literal + ";\n"
        + "  system = builtins.currentSystem;\n"
        + "in\n"
        + "  builtins.hasAttr \"packages\" flake &&\n"
        + "  builtins.hasAttr system flake.packages &&\n"
        + "  builtins.hasAttr \"default\" flake.packages.${system} &&\n"
        + "  builtins.hasAttr \"runtime\" flake.packages.${system} &&\n"
        + "  builtins.hasAttr \"yazelix\" flake.packages.${system} &&\n"
        + "  !builtins.hasAttr \"install\" flake.packages.${system} &&\n"
        + "  flake.packages.${system}.default.outPath == flake.packages.${system}.yazelix.outPath &&\n"
        + "  builtins.hasAttr \"apps\" flake &&\n"
        + "  builtins.hasAttr system flake.apps &&\n"
        + "  builtins.hasAttr \"default\" flake.apps.${system} &&\n"
        + "  builtins.hasAttr \"yazelix\" flake.apps.${system} &&\n"
        + "  !builtins.hasAttr \"install\" flake.apps.${system} &&\n"
        + "  builtins.hasAttr \"homeManagerModules\" flake &&\n"
        + "  builtins.hasAttr \"default\" flake.homeManagerModules &&\n"
        + "  builtins.hasAttr \"yazelix\" flake.homeManagerModules &&\n"
        + "  builtins.isFunction flake.homeManagerModules.default &&\n"
        + "  builtins.isFunction flake.homeManagerModules.yazelix\n"
    )

    let result = (^nix eval --impure --json --expr $expr | complete)

    if $result.exit_code != 0 {
        let stderr = ($result.stderr | default "" | str trim)
        let stdout = ($result.stdout | default "" | str trim)
        let detail = if ($stderr | is-not-empty) { $stderr } else { $stdout }
        error make { msg: $"Top-level flake interface evaluation failed.\n($detail)" }
    }

    let ok = ($result.stdout | from json)
    if not $ok {
        error make { msg: "Top-level flake interface is missing required package/app/Home Manager outputs, still exposes legacy install outputs, or still points packages.default at the lower-level runtime." }
    }

    validate_package_platforms $repo_root_literal
}

def validate_package_platforms [repo_root_literal: string] {
    let platform_check_expr = (
        "let\n"
        + "  flake = builtins.getFlake " + $repo_root_literal + ";\n"
        + "  lib = flake.inputs.nixpkgs.lib;\n"
        + "  systems = builtins.attrNames flake.packages;\n"
        + "  check = system:\n"
        + "    let\n"
        + "      pkg = flake.packages.${system}.yazelix;\n"
        + "      platformEntry = lib.systems.elaborate { inherit system; };\n"
        + "    in {\n"
        + "      inherit system;\n"
        + "      available = lib.meta.availableOn platformEntry pkg;\n"
        + "      platforms = pkg.meta.platforms or [];\n"
        + "    };\n"
        + "in\n"
        + "  builtins.map check systems\n"
    )

    let platform_result = (^nix eval --impure --json --expr $platform_check_expr | complete)

    if $platform_result.exit_code != 0 {
        let stderr = ($platform_result.stderr | default "" | str trim)
        let stdout = ($platform_result.stdout | default "" | str trim)
        let detail = if ($stderr | is-not-empty) { $stderr } else { $stdout }
        error make { msg: $"First-party flake package platform validation failed.\n($detail)" }
    }

    let platform_rows = ($platform_result.stdout | from json)
    let unavailable = ($platform_rows | where {|row| not $row.available })

    if ($unavailable | is-not-empty) {
        let unavailable_detail = (
            $unavailable
            | each {|row| $"($row.system) \(meta.platforms=($row.platforms | to json -r)\)" }
            | str join ", "
        )
        error make {
            msg: $"First-party flake package reports as unavailable on exported systems: ($unavailable_detail). Each system exported in flake.nix must be included in the package meta.platforms."
        }
    }

    print "✅ First-party flake package is available on all exported systems"
}
