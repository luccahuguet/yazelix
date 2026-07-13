{
  agentUsagePackages,
  beadsRustPackage,
  kgpPackages,
  mkYazelix,
  pkgs,
  rtkPackage,
  gritPackage,
  homeManagerPackage,
  icmPackage,
  weavePackage,
  weaveLibsqlPackage,
  obscuraPackage,
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
  flexnetos_foundation_weave_libsql = weaveLibsqlPackage system pkgs;
  flexnetos_foundation_obscura = obscuraPackage system pkgs;
  flexnetos_foundation_meta = metaPackage system pkgs;
  flexnetos_foundation_home_manager = homeManagerPackage system;
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
  flexnetos_foundation_musl_toolchain =
    if system == "x86_64-linux" then
      pkgs.symlinkJoin {
        name = "flexnetos-foundation-musl-toolchain";
        paths = [ pkgs.pkgsCross.musl64.stdenv.cc ];
        postBuild = ''
          ln -s "$out/bin/x86_64-unknown-linux-musl-gcc" "$out/bin/x86_64-linux-musl-gcc"
          ln -s "$out/bin/x86_64-unknown-linux-musl-g++" "$out/bin/x86_64-linux-musl-g++"
          ln -s "$out/bin/x86_64-unknown-linux-musl-ar" "$out/bin/x86_64-linux-musl-ar"
          ln -s "$out/bin/x86_64-unknown-linux-musl-ranlib" "$out/bin/x86_64-linux-musl-ranlib"
        '';
      }
    else
      null;
  flexnetos_foundation_rust_toolchain = fenixPkgs.combine (
    [
      fenixPkgs.latest.cargo
      fenixPkgs.latest.rustc
      fenixPkgs.latest.rustfmt
      fenixPkgs.latest.clippy
    ]
    ++ pkgs.lib.optionals (system == "x86_64-linux") [
      # The Rust target and its C linker/sysroot form one static-build lane.
      # Keep the host compiler above as the sole default compiler.
      fenixPkgs.targets.x86_64-unknown-linux-musl.latest.rust-std
    ]
  );
  flexnetos_foundation_rust_1_89 = fenixPkgs.fromToolchainName {
    name = "1.89.0";
    sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
  };
  # Keep nightly as the interactive/default compiler while exposing an exact,
  # immutable MSRV lane for envctl compatibility gates.
  flexnetos_foundation_rust_1_89_lane = pkgs.runCommand
    "flexnetos-foundation-rust-1.89-lane"
    { nativeBuildInputs = [ pkgs.makeWrapper ]; }
    ''
      mkdir -p "$out/bin"
      makeWrapper "${flexnetos_foundation_rust_1_89.cargo}/bin/cargo" \
        "$out/bin/cargo-msrv-1.89" \
        --unset CARGO_BUILD_RUSTC_WRAPPER \
        --unset RUSTC_WRAPPER \
        --unset RUSTUP_TOOLCHAIN \
        --set RUSTC "${flexnetos_foundation_rust_1_89.rustc}/bin/rustc" \
        --set RUSTDOC "${flexnetos_foundation_rust_1_89.rustc}/bin/rustdoc"
      ln -s "${flexnetos_foundation_rust_1_89.rustc}/bin/rustc" \
        "$out/bin/rustc-msrv-1.89"
      ln -s "${flexnetos_foundation_rust_1_89.rustc}/bin/rustdoc" \
        "$out/bin/rustdoc-msrv-1.89"
    '';
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
  lifeos_foundation_yzx_base = mkYazelix {
    inherit pkgs;
    # Kitty is the packaged default terminal; ghostty (host-installed) is the
    # backup. Mars was removed from the foundation (operator directive 2026-07-11).
    runtimeVariant = "kitty";
    name = "lifeos-foundation-yzx";
    runtimeName = "lifeos-foundation-yzx-runtime";
    extraRuntimePackages =
      defaultRuntimePackages
      ++ [
        flexnetos_foundation_claude
        flexnetos_foundation_codex
        flexnetos_foundation_git_kb
        flexnetos_foundation_kache_wrapped
        flexnetos_foundation_grit
        flexnetos_foundation_icm
        flexnetos_foundation_weave
        flexnetos_foundation_obscura
        flexnetos_foundation_meta
        flexnetos_foundation_home_manager
        flexnetos_foundation_notebooklm
        flexnetos_foundation_rtk
        flexnetos_foundation_rust_1_89_lane
        flexnetos_foundation_rust_toolchain
        flexnetos_foundation_bun
        # beads_rust ships `br` (agent-first issue tracker); the .claude
        # SessionStart/PreCompact hooks and AGENTS.md beads workflow depend on it
        # resolving from the runtime profile, not just maintainer/CI shells.
        beads_rust
        # actionlint backs envctl's ci/gates/actionlint.sh (workflow syntax + custom
        # runner labels); the gate SKIPs until this ships on toolbin.
        pkgs.actionlint
        pkgs.cargo-audit
        pkgs.cargo-tauri
        pkgs.clang
        pkgs.corepack
        pkgs.file
        pkgs.kitty
        pkgs.nodejs_24
        pkgs.pkg-config
        pkgs.sqlite
        pkgs.stdenv.cc
        pkgs.wasm-pack
        pkgs.wild
      ]
      ++ pkgs.lib.optionals (system == "x86_64-linux") [
        flexnetos_foundation_musl_toolchain
        pkgs.sqld
        pkgs.xorg-server
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
      "cargo-audit"
      "cargo-msrv-1.89"
      "cargo-tauri"
      "cc"
      "clang"
      "clang++"
      "clippy-driver"
      "cargo-fmt"
      "cargo-clippy"
      "corepack"
      "file"
      "home-manager"
      "kache"
      "kache-rustc-wrapper"
      "ld.wild"
      "node"
      "notebooklm"
      "npm"
      "nu_plugin_codedb"
      "pnpm"
      "pkg-config"
      "rtk"
      "weave"
      "obscura"
      "rustc"
      "rustc-msrv-1.89"
      "rustdoc"
      "rustdoc-msrv-1.89"
      "rustfmt"
      "sqlite3"
      "wasm-pack"
      "wild"
      "yarn"
    ] ++ pkgs.lib.optionals (system == "x86_64-linux") [
      "Xvfb"
      "sqld"
      "x86_64-linux-musl-ar"
      "x86_64-linux-musl-g++"
      "x86_64-linux-musl-gcc"
      "x86_64-linux-musl-ranlib"
      "x86_64-unknown-linux-musl-ar"
      "x86_64-unknown-linux-musl-g++"
      "x86_64-unknown-linux-musl-gcc"
      "x86_64-unknown-linux-musl-ranlib"
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
      "cargo-audit"
      "cargo-msrv-1.89"
      "cargo-tauri"
      "cc"
      "clang"
      "clang++"
      "clippy-driver"
      "cargo-fmt"
      "cargo-clippy"
      "corepack"
      "file"
      "home-manager"
      "kache"
      "kache-rustc-wrapper"
      "ld.wild"
      "node"
      "notebooklm"
      "npm"
      "nu_plugin_codedb"
      "pnpm"
      "pkg-config"
      "rtk"
      "weave"
      "obscura"
      "rust-analyzer"
      "rustc"
      "rustc-msrv-1.89"
      "rustdoc"
      "rustdoc-msrv-1.89"
      "rustfmt"
      "sqlite3"
      "uv"
      "uvx"
      "wasm-pack"
      "wild"
      "yarn"
    ] ++ pkgs.lib.optionals (system == "x86_64-linux") [
      "Xvfb"
      "sqld"
      "x86_64-linux-musl-ar"
      "x86_64-linux-musl-g++"
      "x86_64-linux-musl-gcc"
      "x86_64-linux-musl-ranlib"
      "x86_64-unknown-linux-musl-ar"
      "x86_64-unknown-linux-musl-g++"
      "x86_64-unknown-linux-musl-gcc"
      "x86_64-unknown-linux-musl-ranlib"
    ];
  };
  # The only real-home profile element carries its own desktop integration.
  # Per-user desktop files would be parallel shadows of these package-owned
  # entries, so the launchers re-enter through the sole `.nix-profile` route.
  lifeos_foundation_yzx_desktop_launch = pkgs.writeShellScriptBin "yzx-desktop-launch" ''
    set -eu
    profile_home="''${YAZELIX_PROFILE_HOME:-''${HOME:?HOME is required}}"
    profile="$profile_home/.nix-profile"
    if [ ! -L "$profile" ] || [ ! -x "$profile/bin/yzx" ]; then
      printf 'yzx-desktop-launch: real-home profile frontdoor is missing: %s\n' "$profile" >&2
      exit 78
    fi
    exec "$profile/bin/yzx" desktop launch
  '';
  lifeos_foundation_yzx_agent_launch = pkgs.writeShellScriptBin "yzx-agent-workspace-launch" ''
    set -eu
    profile_home="''${YAZELIX_PROFILE_HOME:-''${HOME:?HOME is required}}"
    profile="$profile_home/.nix-profile"
    layout="$profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl"
    if [ ! -L "$profile" ] || [ ! -x "$profile/bin/yzx" ]; then
      printf 'yzx-agent-workspace-launch: real-home profile frontdoor is missing: %s\n' "$profile" >&2
      exit 78
    fi
    if [ ! -s "$layout" ]; then
      printf 'yzx-agent-workspace-launch: profile-owned layout is missing: %s\n' "$layout" >&2
      exit 78
    fi
    YAZELIX_LAYOUT_OVERRIDE="$layout"
    export YAZELIX_LAYOUT_OVERRIDE
    exec "$profile/bin/yzx" desktop launch
  '';
  lifeos_foundation_yzx_desktop = pkgs.runCommand
    "lifeos-foundation-yzx-desktop-integration"
    { nativeBuildInputs = [ pkgs.desktop-file-utils ]; }
    ''
      mkdir -p "$out/bin" "$out/share/applications"
      ln -s "${lifeos_foundation_yzx_desktop_launch}/bin/yzx-desktop-launch" \
        "$out/bin/yzx-desktop-launch"
      ln -s "${lifeos_foundation_yzx_agent_launch}/bin/yzx-agent-workspace-launch" \
        "$out/bin/yzx-agent-workspace-launch"

      cat > "$out/share/applications/com.yazelix.Yazelix.Kitty.desktop" <<'EOF'
      [Desktop Entry]
      Version=1.4
      Type=Application
      Name=New Yazelix - Kitty
      Comment=Yazi + Zellij + Helix integrated terminal environment
      Icon=yazelix
      StartupWMClass=com.yazelix.Yazelix
      Terminal=false
      X-Yazelix-Managed=true
      Exec=/usr/bin/env sh -lc "exec ~/.nix-profile/bin/yzx-desktop-launch"
      Categories=Development;
      EOF

      cat > "$out/share/applications/com.flexnetos.Yazelix.Agent.desktop" <<'EOF'
      [Desktop Entry]
      Version=1.4
      Type=Application
      Name=FlexNetOS Yazelix Agent
      Comment=Yazelix Kitty with the profile-owned FlexNetOS agent workspace layout
      Icon=yazelix
      StartupWMClass=com.yazelix.Yazelix
      Terminal=false
      X-Yazelix-Managed=true
      X-FlexNetOS-Managed=true
      Exec=/usr/bin/env sh -lc "exec ~/.nix-profile/bin/yzx-agent-workspace-launch"
      Categories=Development;
      EOF

      desktop-file-validate "$out/share/applications/com.yazelix.Yazelix.Kitty.desktop"
      desktop-file-validate "$out/share/applications/com.flexnetos.Yazelix.Agent.desktop"
      for size in 48x48 64x64 128x128 256x256; do
        destination="$out/share/icons/hicolor/$size/apps"
        mkdir -p "$destination"
        ln -s "${lifeos_foundation_yzx_base}/assets/icons/$size/yazelix.png" \
          "$destination/yazelix.png"
      done
    '';
  lifeos_foundation_yzx = pkgs.symlinkJoin {
    name = "lifeos-foundation-yzx";
    paths = [ lifeos_foundation_yzx_base ]
      ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
        lifeos_foundation_yzx_desktop
      ];
    meta = lifeos_foundation_yzx_base.meta;
  };
  packages =
    {
      br = beads_rust;
      claude = flexnetos_foundation_claude;
      codex = flexnetos_foundation_codex;
      git_kb = flexnetos_foundation_git_kb;
      rtk = flexnetos_foundation_rtk;
      weave = flexnetos_foundation_weave;
      weave_libsql = flexnetos_foundation_weave_libsql;
      obscura = flexnetos_foundation_obscura;
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
