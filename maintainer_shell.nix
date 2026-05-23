{
  pkgs,
  lib,
  fenixPkgs,
  brPackage,
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
    ++ [ pkgs.cargo-nextest ]
    ++ [ brPackage ]
    ++ [ rustWasiToolchain ]
    ++ [ openssl ];
  maintainerYzx = pkgs.writeShellScriptBin "yzx" ''
    set -eu

    core_yzx="${rustCoreHelper}/bin/yzx"
    repo_root="$(git -C "$PWD" rev-parse --show-toplevel 2>/dev/null || printf '%s\n' "''${YAZELIX_RUNTIME_DIR:-${repoRoot}}")"

    run_maintainer() {
      exec cargo run --quiet \
        --manifest-path "$repo_root/rust_core/Cargo.toml" \
        -p yazelix_maintainer \
        --bin yzx_repo_maintainer \
        -- \
        --repo-root "$repo_root" \
        "$@"
    }

    if [ "$#" -gt 0 ] && [ "$1" = "dev" ]; then
      shift
      if [ "$#" -eq 0 ] || [ "$1" = "-h" ] || [ "$1" = "--help" ] || [ "$1" = "help" ]; then
        cat <<'USAGE'
    Development and maintainer commands

    Usage:
      yzx dev <command>

    Maintainer commands:
      yzx dev build_pane_orchestrator [--sync]
      yzx dev bump <version>
      yzx dev lint_nu [--format pretty|compact] [paths...]
      yzx dev rust <fmt|check|test>
      yzx dev sync_issues [--dry-run]
      yzx dev sync_yzpp_wasm
      yzx dev test [options]
      yzx dev update [options]

    Runtime diagnostics:
      yzx dev inspect_session [--json]
      yzx dev profile [--cold] [--desktop] [--launch] [--clear-cache]
USAGE
        exit 0
      fi

      subcommand="$1"
      shift
      case "$subcommand" in
        build_pane_orchestrator)
          run_maintainer build-pane-orchestrator "$@"
          ;;
        bump)
          run_maintainer version-bump "$@"
          ;;
        lint_nu)
          run_maintainer lint-nu "$@"
          ;;
        rust)
          run_maintainer rust "$@"
          ;;
        sync_issues)
          run_maintainer sync-issues "$@"
          ;;
        sync_yzpp_wasm)
          run_maintainer sync-yzpp-wasm "$@"
          ;;
        test)
          run_maintainer run-tests "$@"
          ;;
        update)
          run_maintainer dev-update "$@"
          ;;
        *)
          exec "$core_yzx" dev "$subcommand" "$@"
          ;;
      esac
    fi

    exec "$core_yzx" "$@"
  '';
  allDeps = lib.unique (runtimeDeps ++ maintainerDeps ++ [ maintainerYzx ]);

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
    if [ -f "$maintainer_runtime_dir/flake.nix" ] && [ -f "$maintainer_runtime_dir/settings_default.jsonc" ]; then
      export YAZELIX_RUNTIME_DIR="$maintainer_runtime_dir"
    else
      unset YAZELIX_RUNTIME_DIR
    fi

    runtime_env_runtime_dir="''${YAZELIX_RUNTIME_DIR:-${repoRoot}}"
    runtime_env_request="$(${pkgs.jq}/bin/jq -nc \
      --arg runtime_dir "$runtime_env_runtime_dir" \
      --arg home_dir "$HOME" \
      --arg current_path "$PATH" \
      --arg editor_command "''${EDITOR:-hx}" \
      --arg helix_runtime_path "''${HELIX_RUNTIME:-}" \
      '{
        runtime_dir: $runtime_dir,
        home_dir: $home_dir,
        current_path: $current_path,
        editor_command: $editor_command
      } + (
        if $helix_runtime_path == "" then {} else {helix_runtime_path: $helix_runtime_path} end
      )'
    )"
    runtime_env_json="$("${rustCoreHelper}/bin/yzx_core" runtime-env.compute --request-json "$runtime_env_request" | ${pkgs.jq}/bin/jq -rc '.data.runtime_env')"

    export PATH="$(printf '%s' "$runtime_env_json" | ${pkgs.jq}/bin/jq -r '.PATH | join(":")')"
    export PATH="${maintainerYzx}/bin:$PATH"
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

    if [ -f "$YAZELIX_RUNTIME_DIR/runtime_components.json" ]; then
      "${rustCoreHelper}/bin/yzx_control" enter --setup-only >/dev/null
    fi
  '';
}
