{
  agentUsagePackages,
  beadsRustPackage,
  kgpPackages,
  mkYazelix,
  pkgs,
  rtkPackage,
  gritPackage,
  icmPackage,
  weavePackage,
  metaPackage,
  runtimePackage,
  system,
  yazelixCursors,
  yazelixPackage,
  yazelixScreen,
  yazelixYaziAssets,
  yazelixZellijBar,
  yazelixZellijPaneOrchestrator,
  yazelixZellijPopup,
  fenixPkgs,
}:

let
  defaultRuntimePackages = agentUsagePackages system;
  # Kitty is the packaged terminal; mars was removed (operator directive 2026-07-11).
  runtime_kitty = runtimePackage system pkgs "kitty" defaultRuntimePackages;
  yazelix_kitty = yazelixPackage system pkgs "kitty" defaultRuntimePackages;
  yazelix_zellij_bar = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
  yazelix_screen = yazelixScreen.packages.${system}.yzs;
  yazelix_cursors = yazelixCursors.packages.${system}.yazelix_cursors;
  yazelix_helix = kgpPackages.helixPackage system;
  yazelix_zellij_config_pack = import ./yazelix_zellij_config_pack.nix {
    inherit pkgs fenixPkgs;
    src = ../.;
  };
  yazelix_zellij_pane_orchestrator =
    yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
  yazelix_zellij_popup = yazelixZellijPopup.packages.${system}.yzpp;
  yazelix_yazi_assets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
  beads_rust = beadsRustPackage system pkgs;
  install_check = import ./install_check.nix { inherit pkgs; };
  flexnetos_foundation_claude = import ./claude_code_release.nix {
    inherit pkgs;
    version = "2.1.207";
  };
  flexnetos_foundation_codex = import ./codex_cli_release.nix {
    inherit pkgs system;
    version = "0.144.0";
  };
  flexnetos_foundation_git_kb = import ./git_kb_release.nix {
    inherit pkgs;
    version = "0.2.12";
  };
  flexnetos_foundation_rtk = rtkPackage system pkgs;
  flexnetos_foundation_grit = gritPackage system pkgs;
  flexnetos_foundation_icm = icmPackage system pkgs;
  flexnetos_foundation_weave = weavePackage system pkgs;
  flexnetos_foundation_meta = metaPackage system pkgs;
  flexnetos_foundation_kache = import ./kache_release.nix { inherit pkgs; };
  flexnetos_foundation_notebooklm = import ./notebooklm_release.nix {
    inherit pkgs;
    version = "0.8.0a3";
  };
  flexnetos_foundation_kache_wrapped = pkgs.symlinkJoin {
    name = "kache-with-rustc-wrapper-${flexnetos_foundation_kache.version}";
    paths = [ flexnetos_foundation_kache ];
    postBuild = ''
      mkdir -p "$out/bin" "$out/libexec/kache"
      cat > "$out/libexec/kache/rustc" <<'EOF'
      #!${pkgs.runtimeShell}
      set -eu
      cargo_auditable="''${FLEXNETOS_KACHE_CARGO_AUDITABLE:-cargo-auditable}"
      exec "$cargo_auditable" rustc "$@"
      EOF
      chmod +x "$out/libexec/kache/rustc"

      cat > "$out/bin/kache-rustc-wrapper" <<EOF
      #!${pkgs.runtimeShell}
      set -eu
      KACHE_BIN="''${KACHE_BIN:-$out/bin/kache}"
      FLEXNETOS_KACHE_RUSTC_SHIM="''${FLEXNETOS_KACHE_RUSTC_SHIM:-$out/libexec/kache/rustc}"
      if [ ! -x "\$KACHE_BIN" ]; then
        printf 'kache-rustc-wrapper: Kache binary is not executable: %s\n' "\$KACHE_BIN" >&2
        exit 127
      fi
      if [ "\$#" -ge 2 ]; then
        first_name="\$(basename -- "\$1")"
        second_name="\$(basename -- "\$2")"
        if [ "\$first_name" = cargo-auditable ] && { [ "\$second_name" = rustc ] || [ "\$second_name" = clippy-driver ] || case "\$second_name" in rustc-*) true ;; *) false ;; esac; }; then
          if [ ! -x "\$FLEXNETOS_KACHE_RUSTC_SHIM" ]; then
            printf 'kache-rustc-wrapper: rustc shim is not executable: %s\n' "\$FLEXNETOS_KACHE_RUSTC_SHIM" >&2
            exit 127
          fi
          export FLEXNETOS_KACHE_CARGO_AUDITABLE="\$1"
          shift 2
          exec "\$KACHE_BIN" "\$FLEXNETOS_KACHE_RUSTC_SHIM" "\$@"
        fi
      fi
      exec "\$KACHE_BIN" "\$@"
      EOF
      chmod +x "$out/bin/kache-rustc-wrapper"
    '';
  };
  flexnetos_foundation_rust_toolchain = fenixPkgs.combine [
    fenixPkgs.latest.cargo
    fenixPkgs.latest.rustc
    fenixPkgs.latest.rustfmt
    fenixPkgs.latest.clippy
    # musl static lane (envctl blueprint R9/TASK-0093): rust-std for the
    # x86_64-unknown-linux-musl target so `cargo build --target
    # x86_64-unknown-linux-musl` links fully-static binaries. std only —
    # the host cargo/rustc above stay the single compiler.
    fenixPkgs.targets.x86_64-unknown-linux-musl.latest.rust-std
  ];
  # bun pinned ahead of nixpkgs-unstable (ships 1.3.13; upstream stable is
  # 1.3.14, https://github.com/oven-sh/bun/releases/tag/bun-v1.3.14).
  # Same official-binary source the nixpkgs derivation uses. Drop this
  # override once nixpkgs-unstable ships bun >= 1.3.14.
  flexnetos_foundation_bun_sources = {
    x86_64-linux = pkgs.fetchurl {
      url = "https://github.com/oven-sh/bun/releases/download/bun-v1.3.14/bun-linux-x64.zip";
      hash = "sha256-lR7iruhV8IWVruxiJSJqKY0/6oOj3NZGXAnLzN9+hI8=";
    };
  };
  flexnetos_foundation_bun =
    if flexnetos_foundation_bun_sources ? ${system} then
      pkgs.bun.overrideAttrs (old: {
        version = "1.3.14";
        src = flexnetos_foundation_bun_sources.${system};
      })
    else
      pkgs.bun;
  lifeos_foundation_yzx = mkYazelix {
    inherit pkgs;
    # Kitty is the packaged default terminal; ghostty (host-installed) is the
    # backup. Mars was removed from the foundation (operator directive 2026-07-11).
    runtimeVariant = "kitty";
    name = "lifeos-foundation-yzx";
    runtimeName = "lifeos-foundation-yzx-runtime";
    extraRuntimePackages = defaultRuntimePackages ++ [
      flexnetos_foundation_claude
      flexnetos_foundation_codex
      flexnetos_foundation_git_kb
      flexnetos_foundation_kache_wrapped
      flexnetos_foundation_grit
      flexnetos_foundation_icm
      flexnetos_foundation_weave
      flexnetos_foundation_meta
      flexnetos_foundation_notebooklm
      flexnetos_foundation_rtk
      flexnetos_foundation_rust_toolchain
      flexnetos_foundation_bun
      # beads_rust ships `br` (agent-first issue tracker); the .claude
      # SessionStart/PreCompact hooks and AGENTS.md beads workflow depend on it
      # resolving from the runtime profile, not just maintainer/CI shells.
      beads_rust
      # actionlint backs envctl's ci/gates/actionlint.sh (workflow syntax + custom
      # runner labels); the gate SKIPs until this ships on toolbin.
      pkgs.actionlint
      pkgs.cargo-tauri
      pkgs.clang
      pkgs.corepack
      pkgs.kitty
      pkgs.nodejs_24
      pkgs.wasm-pack
      pkgs.wild
    ];
    extraRuntimeCommands = [
      "tu"
      "actionlint"
      "br"
      "claude"
      "kitty"
      "ccboard"
      "codex"
      "codedb"
      "git-kb"
      "grit"
      "icm"
      "meta"
      "meta-git"
      "meta-mcp"
      "meta-project"
      "loop"
      "bun"
      "bunx"
      "cargo"
      "cargo-tauri"
      "clang"
      "clang++"
      "clippy-driver"
      "cargo-fmt"
      "cargo-clippy"
      "corepack"
      "kache"
      "kache-rustc-wrapper"
      "ld.wild"
      "node"
      "notebooklm"
      "npm"
      "nu_plugin_codedb"
      "pnpm"
      "rtk"
      "weave"
      "rustc"
      "rustdoc"
      "rustfmt"
      "wasm-pack"
      "wild"
      "yarn"
    ];
    exportedBinCommands = [
      "claude"
      "ccboard"
      "codex"
      "codedb"
      "git-kb"
      "grit"
      "icm"
      "meta"
      "meta-git"
      "meta-mcp"
      "meta-project"
      "loop"
      "bun"
      "bunx"
      "cargo"
      "cargo-tauri"
      "clang"
      "clang++"
      "clippy-driver"
      "cargo-fmt"
      "cargo-clippy"
      "corepack"
      "kache"
      "kache-rustc-wrapper"
      "ld.wild"
      "node"
      "notebooklm"
      "npm"
      "nu_plugin_codedb"
      "pnpm"
      "rtk"
      "weave"
      "rust-analyzer"
      "rustc"
      "rustdoc"
      "rustfmt"
      "uv"
      "uvx"
      "wasm-pack"
      "wild"
      "yarn"
    ];
  };
  packages =
    {
      br = beads_rust;
      claude = flexnetos_foundation_claude;
      codex = flexnetos_foundation_codex;
      git_kb = flexnetos_foundation_git_kb;
      rtk = flexnetos_foundation_rtk;
      weave = flexnetos_foundation_weave;
      inherit beads_rust install_check;
      inherit runtime_kitty yazelix_kitty;
      inherit lifeos_foundation_yzx;
      inherit yazelix_cursors yazelix_helix yazelix_screen;
      inherit yazelix_yazi_assets yazelix_zellij_bar yazelix_zellij_config_pack;
      inherit yazelix_zellij_pane_orchestrator yazelix_zellij_popup;
      default = yazelix_kitty;
      runtime = runtime_kitty;
      runtime_agent_tools = runtime_kitty;
      yazelix = yazelix_kitty;
      yazelix_agent_tools = yazelix_kitty;
      yazelix_kgp_zellij = (kgpPackages.graphicsPkgs pkgs).zellij;
      yzs = yazelix_screen;
    };

  appFor = packageName: binName: {
    type = "app";
    program = "${packages.${packageName}}/bin/${binName}";
  };
  yzxApp = packageName: appFor packageName "yzx";
in
{
  inherit packages;

  apps = {
    default = yzxApp "yazelix";
    yazelix = yzxApp "yazelix";
    yazelix_agent_tools = yzxApp "yazelix_agent_tools";
    lifeos_foundation_yzx = yzxApp "lifeos_foundation_yzx";
    yazelix_kitty = yzxApp "yazelix_kitty";
    yazelix_screen = appFor "yazelix_screen" "yzs";
    yzs = appFor "yazelix_screen" "yzs";
    yazelix_cursors = appFor "yazelix_cursors" "yzc";
    yzc = appFor "yazelix_cursors" "yzc";
    install_check = appFor "install_check" "install_check";
  };
}
