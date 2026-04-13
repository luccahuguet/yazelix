{
  pkgs,
  lib,
  fenixPkgs,
  bdPackage,
  repoRoot,
  nixgl ? null,
}:

let
  runtimeDeps = import ./packaging/runtime_deps.nix { inherit pkgs nixgl; };
  rustWasiToolchain = fenixPkgs.combine [
    fenixPkgs.stable.cargo
    fenixPkgs.stable.rustc
    fenixPkgs.stable.rustfmt
    fenixPkgs.stable.clippy
    fenixPkgs.targets.wasm32-wasip1.stable.rust-std
  ];
  maintainerDeps =
    [ pkgs.github-cli ]
    ++ [ pkgs.nu-lint ]
    ++ [ bdPackage ]
    ++ [ rustWasiToolchain ];
  allDeps = lib.unique (runtimeDeps ++ maintainerDeps);

  yazelixNixConfig = ''
    warn-dirty = false
    extra-substituters = https://cache.numtide.com
    extra-trusted-public-keys = niks3.numtide.com-1:DTx8wZduET09hRmMtKdQDxNNthLQETkc/yaX7M4qK0g=
  '';
in
pkgs.mkShell {
  packages = allDeps;

  shellHook = ''
    export YAZELIX_CONFIG_DIR="''${XDG_CONFIG_HOME:-$HOME/.config}/yazelix"
    export YAZELIX_STATE_DIR="''${XDG_DATA_HOME:-$HOME/.local/share}/yazelix"
    export YAZELIX_LOGS_DIR="$YAZELIX_STATE_DIR/logs"
    export IN_YAZELIX_SHELL="true"
    export NIX_CONFIG='${yazelixNixConfig}'

    runtime_env_json="$(${pkgs.nushell}/bin/nu -c 'use "${repoRoot}/nushell/scripts/utils/runtime_env.nu" [get_runtime_env]; get_runtime_env | to json -r')"

    export PATH="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.PATH | join(":")')"
    export YAZELIX_RUNTIME_DIR="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.YAZELIX_RUNTIME_DIR')"
    export ZELLIJ_DEFAULT_LAYOUT="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.ZELLIJ_DEFAULT_LAYOUT')"
    export YAZI_CONFIG_HOME="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.YAZI_CONFIG_HOME')"
    export EDITOR="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.EDITOR')"

    managed_helix_binary="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.YAZELIX_MANAGED_HELIX_BINARY // empty')"
    if [ -n "$managed_helix_binary" ]; then
      export YAZELIX_MANAGED_HELIX_BINARY="$managed_helix_binary"
    else
      unset YAZELIX_MANAGED_HELIX_BINARY
    fi

    helix_runtime="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.HELIX_RUNTIME // empty')"
    if [ -n "$helix_runtime" ]; then
      export HELIX_RUNTIME="$helix_runtime"
    else
      unset HELIX_RUNTIME
    fi

    if [ -t 1 ] && [ "''${YAZELIX_ENV_ONLY:-false}" != "true" ]; then
      echo "🧭 Yazelix maintainer shell"
      echo "   Flake-owned runtime + maintainer toolchain."
      echo "   EDITOR: $EDITOR"
    fi

    ${pkgs.nushell}/bin/nu "${repoRoot}/nushell/scripts/setup/environment.nu" --skip-welcome
  '';
}
