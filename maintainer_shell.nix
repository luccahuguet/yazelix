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
  rustCoreHelper = import ./packaging/rust_core_helper.nix {
    inherit pkgs fenixPkgs;
    src = repoRoot;
  };
  openssl = pkgs.openssl;
  maintainerDeps =
    [ pkgs.github-cli ]
    ++ [ pkgs.nu-lint ]
    ++ [ bdPackage ]
    ++ [ rustWasiToolchain ]
    ++ [ openssl ];
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
    export YAZELIX_YZX_CORE_BIN="${rustCoreHelper}/bin/yzx_core"
    export YAZELIX_YZX_CONTROL_BIN="${rustCoreHelper}/bin/yzx_control"

    maintainer_runtime_dir="$(git -C "$PWD" rev-parse --show-toplevel 2>/dev/null || printf '%s\n' "$PWD")"
    if [ -f "$maintainer_runtime_dir/flake.nix" ] && [ -f "$maintainer_runtime_dir/yazelix_default.toml" ]; then
      export YAZELIX_RUNTIME_DIR="$maintainer_runtime_dir"
    else
      unset YAZELIX_RUNTIME_DIR
    fi

    runtime_env_json="$(${pkgs.nushell}/bin/nu --no-config-file -c 'use "${repoRoot}/nushell/scripts/utils/runtime_env.nu" [get_runtime_env]; get_runtime_env | to json -r')"

    export PATH="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.PATH | join(":")')"
    computed_runtime_dir="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.YAZELIX_RUNTIME_DIR')"
    if [ -z "''${YAZELIX_RUNTIME_DIR:-}" ]; then
      export YAZELIX_RUNTIME_DIR="$computed_runtime_dir"
    fi
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

    export OPENSSL_DIR="${pkgs.openssl.out}"
    export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
    export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"

    ${pkgs.nushell}/bin/nu --no-config-file "${repoRoot}/nushell/scripts/setup/environment.nu" --skip-welcome
  '';
}
