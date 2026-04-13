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
}
