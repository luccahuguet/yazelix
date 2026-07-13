{ pkgs, runtime, foundation ? false }:

pkgs.runCommand "yazelix-runtime-release-contracts" { } ''
  set -eu

  runtime=${runtime}

  test -x "$runtime/bin/yzx"
  test -s "$runtime/settings_default.jsonc"
  test -s "$runtime/runtime_identity.json"
  test -s "$runtime/runtime_tools.json"
  test -s "$runtime/runtime_components.json"
  test -x "$runtime/toolbin/nu"
  test -x "$runtime/toolbin/tu"
  ${pkgs.lib.optionalString foundation ''
    for command_name in \
      cargo-audit cargo-msrv-1.89 cc ccboard file home-manager pkg-config rtk \
      rustc-msrv-1.89 rustdoc-msrv-1.89 sqlite3; do
      test -x "$runtime/toolbin/$command_name"
      test -x "$runtime/bin/$command_name"
      test "$(readlink -f "$runtime/toolbin/$command_name")" = \
        "$(readlink -f "$runtime/bin/$command_name")"
    done
    if [ "$(uname -s)" = Linux ]; then
      test -x "$runtime/bin/yzx-desktop-launch"
      test -x "$runtime/bin/yzx-agent-workspace-launch"
      test -s "$runtime/share/applications/com.yazelix.Yazelix.Kitty.desktop"
      test -s "$runtime/share/applications/com.flexnetos.Yazelix.Agent.desktop"
      grep -F 'Exec=/usr/bin/env sh -lc "exec ~/.nix-profile/bin/yzx-desktop-launch"' \
        "$runtime/share/applications/com.yazelix.Yazelix.Kitty.desktop" >/dev/null
      grep -F 'Exec=/usr/bin/env sh -lc "exec ~/.nix-profile/bin/yzx-agent-workspace-launch"' \
        "$runtime/share/applications/com.flexnetos.Yazelix.Agent.desktop" >/dev/null
    fi
    test -s "$runtime/nushell/config/config.nu"
    test -s "$runtime/nushell/config/rtk_wrappers.nu"
    grep -F 'use rtk_wrappers.nu *' "$runtime/nushell/config/config.nu" >/dev/null
    grep -F 'export def --wrapped cargo' "$runtime/nushell/config/rtk_wrappers.nu" >/dev/null
    grep -F '{ ^rtk cargo' "$runtime/nushell/config/rtk_wrappers.nu" >/dev/null
    grep -F 'export def --wrapped codex' "$runtime/nushell/config/rtk_wrappers.nu" >/dev/null
    grep -F '{ ^rtk codex' "$runtime/nushell/config/rtk_wrappers.nu" >/dev/null
    grep -F '^rtk proxy -- cargo test' "$runtime/nushell/config/rtk_wrappers.nu" >/dev/null
    if grep -F '`^cargo test' "$runtime/nushell/config/rtk_wrappers.nu" >/dev/null; then
      echo "RTK Nu policy must proxy raw evidence instead of bypassing RTK" >&2
      exit 1
    fi
    "$runtime/toolbin/cargo-audit" --version | grep -F 'cargo-audit 0.22.1' >/dev/null
    "$runtime/toolbin/cargo-msrv-1.89" --version | grep -F 'cargo 1.89.0' >/dev/null
    "$runtime/toolbin/rustc-msrv-1.89" --version | grep -F 'rustc 1.89.0' >/dev/null
    "$runtime/toolbin/file" --version >/dev/null
    "$runtime/toolbin/home-manager" --version >/dev/null
    "$runtime/toolbin/pkg-config" --version >/dev/null
    "$runtime/toolbin/sqlite3" --version >/dev/null

    mkdir -p msrv-probe/src msrv-cargo-home
    cat > msrv-probe/Cargo.toml <<'EOF'
    [package]
    name = "yazelix-msrv-probe"
    version = "0.0.0"
    edition = "2024"
    publish = false

    [workspace]
    EOF
    printf 'fn main() {}\n' > msrv-probe/src/main.rs
    CARGO_HOME="$PWD/msrv-cargo-home" \
      "$runtime/toolbin/cargo-msrv-1.89" check \
        --offline --manifest-path msrv-probe/Cargo.toml

    if [ "$(uname -s)" = Linux ] && [ "$(uname -m)" = x86_64 ]; then
      for command_name in Xvfb sqld; do
        test -x "$runtime/toolbin/$command_name"
        test -x "$runtime/bin/$command_name"
      done
      "$runtime/toolbin/sqld" --version | grep -F 'sqld 0.24.33' >/dev/null

      musl_gcc="$runtime/toolbin/x86_64-linux-musl-gcc"
      musl_gxx="$runtime/toolbin/x86_64-linux-musl-g++"
      musl_ar="$runtime/toolbin/x86_64-linux-musl-ar"
      musl_ranlib="$runtime/toolbin/x86_64-linux-musl-ranlib"
      for command_name in \
        x86_64-linux-musl-ar x86_64-linux-musl-g++ \
        x86_64-linux-musl-gcc x86_64-linux-musl-ranlib \
        x86_64-unknown-linux-musl-ar x86_64-unknown-linux-musl-g++ \
        x86_64-unknown-linux-musl-gcc x86_64-unknown-linux-musl-ranlib; do
        test -x "$runtime/toolbin/$command_name"
        test -x "$runtime/bin/$command_name"
      done
      printf 'int main(void) { return 0; }\n' \
        | "$musl_gcc" -static -x c - -o musl-c-probe
      printf 'int main() { return 0; }\n' \
        | "$musl_gxx" -static -x c++ - -o musl-cxx-probe
      printf 'int yazelix_archive_probe(void) { return 0; }\n' \
        | "$musl_gcc" -x c -c - -o musl-archive-probe.o
      "$musl_ar" rcs libyazelix-musl-probe.a musl-archive-probe.o
      "$musl_ranlib" libyazelix-musl-probe.a
      test -s libyazelix-musl-probe.a

      printf 'fn main() {}\n' \
        | "$runtime/toolbin/rustc" - --target x86_64-unknown-linux-musl \
          -C "linker=$musl_gcc" -o musl-static-probe
      "$runtime/toolbin/file" musl-static-probe \
        | grep -E 'statically linked|static-pie linked' >/dev/null
    fi
  ''}
  test -x "$runtime/runtime_tools/ccboard/bin/ccboard"
  grep -F '"ccboard":' "$runtime/runtime_tools.json" >/dev/null
  grep -F '"commands":["ccboard"]' "$runtime/runtime_tools.json" >/dev/null
  grep -F 'Mission Control launches this tool through libexec/ccboard.' "$runtime/runtime_tools.json" >/dev/null
  test -s "$runtime/runtime_tools/ccboard/runtime-tool-metadata.json"
  grep -F '"source_repo":"https://github.com/FlexNetOS/ccboard"' "$runtime/runtime_tools/ccboard/runtime-tool-metadata.json" >/dev/null
  grep -F '"commands":["ccboard"]' "$runtime/runtime_tools/ccboard/runtime-tool-metadata.json" >/dev/null
  test -s "$runtime/config_metadata/ccboard_runtime_tool.toml"
  grep -F 'YAZELIX_CCBOARD_BIN = "runtime_tools/ccboard/bin/ccboard"' "$runtime/config_metadata/ccboard_runtime_tool.toml" >/dev/null
  grep -F '"codedb":' "$runtime/runtime_tools.json" >/dev/null
  grep -F '"commands":["codedb","nu_plugin_codedb"]' "$runtime/runtime_tools.json" >/dev/null
  test -x "$runtime/runtime_tools/codedb/bin/codedb"
  test -x "$runtime/runtime_tools/codedb/bin/nu_plugin_codedb"
  codedb_plugin_command_count="$(
    "$runtime/toolbin/nu" --no-config-file \
      --plugins "$runtime/runtime_tools/codedb/bin/nu_plugin_codedb" \
      -c 'scope commands | where type == plugin | where name == "codedb doctor" | length'
  )"
  test "$codedb_plugin_command_count" = 1
  test -s "$runtime/runtime_tools/codedb/runtime-tool-metadata.json"
  test -s "$runtime/config_metadata/codedb_runtime_tool.toml"
  agent_layout="$runtime/configs/zellij/layouts/flexnetos_agent_workspace.kdl"
  test -s "$agent_layout"
  grep -F 'tab name="Mission Control"' "$agent_layout" >/dev/null
  grep -F 'pane name="ccboard"' "$agent_layout" >/dev/null
  grep -F 'command "__YAZELIX_RUNTIME_DIR__/libexec/ccboard"' "$agent_layout" >/dev/null
  test -x "$runtime/libexec/yazelix_zellij_bar_widget"
  resolved_bar_widget="$(readlink -f "$runtime/libexec/yazelix_zellij_bar_widget")"
  test -x "$resolved_bar_widget"
  grep -F '/toolbin:/nix/store/' "$resolved_bar_widget" >/dev/null
  grep -F '/bin:$PATH' "$resolved_bar_widget" >/dev/null

  # The foundation profile enables Weave's governed-web feature and must ship
  # the separate Obscura process it drives. Keep both commands on the profile
  # bin/toolbin trust roots, then prove a real no-network MCP handshake through
  # `weave web tab_list`. Other Yazelix package shapes contain neither command
  # and intentionally skip this foundation-only contract.
  if [ -e "$runtime/libexec/weave" ] || [ -e "$runtime/libexec/obscura" ]; then
    for command_name in weave obscura; do
      test -x "$runtime/libexec/$command_name"
      test -x "$runtime/toolbin/$command_name"
      test -x "$runtime/bin/$command_name"
      test -L "$runtime/toolbin/$command_name"
      test -L "$runtime/bin/$command_name"
    done

    probe_home="$TMPDIR/weave-obscura-profile-home"
    mkdir -p "$probe_home/.nix-profile/bin"
    ln -s "$runtime/bin/obscura" "$probe_home/.nix-profile/bin/obscura"

    HOME="$probe_home" \
      WEAVE_DB="$probe_home/weave.db" \
      "$runtime/bin/weave" web --list >weave-web-list.txt
    grep -F 'web ops available:' weave-web-list.txt >/dev/null
    grep -F 'tab_list' weave-web-list.txt >/dev/null

    unset WEAVE_MUX_DIR WEAVE_OBSCURA_BIN
    HOME="$probe_home" \
      WEAVE_DB="$probe_home/weave.db" \
      WEAVE_OBSCURA_ALLOW_OPS=tab_list \
      WEAVE_OBSCURA_STEALTH=1 \
      timeout 30 "$runtime/bin/weave" web tab_list >weave-obscura-probe.txt
    grep -F 'No tabs open.' weave-obscura-probe.txt >/dev/null
  fi

  for size in 48x48 64x64 128x128 256x256; do
    test -s "$runtime/assets/icons/$size/yazelix.png"
  done

  test -s "$runtime/configs/zellij/plugins/zjstatus.wasm"
  test -s "$runtime/configs/yazi/plugins/smart-tabs.yazi/main.lua"
  if grep -R -I -F 'https://github.com/dj95/zjstatus/releases/latest/download/zjstatus.wasm' \
    "$runtime/configs" "$runtime/shells" >/dev/null; then
    echo "Yazelix runtime must use packaged file-backed zjstatus.wasm, not upstream URL auto-download" >&2
    exit 1
  fi

  runtime_variant="$(cat "$runtime/runtime_variant")"
  case "$runtime_variant" in
    kitty)
      test -d "$runtime/configs/terminal_emulators/kitty"
      test ! -L "$runtime/configs/terminal_emulators/mars"
      ;;
    mars)
      mars_config="$runtime/share/mars/config.toml"
      test -s "$mars_config"
      grep -F 'family = "JetBrains Mono"' "$mars_config" >/dev/null
      grep -F 'font-family = "Symbols Nerd Font Mono"' "$mars_config" >/dev/null
      grep -F '${pkgs.jetbrains-mono}/share/fonts/truetype' "$mars_config" >/dev/null
      grep -F '${pkgs.nerd-fonts.symbols-only}/share/fonts/truetype/NerdFonts/Symbols' "$mars_config" >/dev/null
      test -d '${pkgs.jetbrains-mono}/share/fonts/truetype'
      test -d '${pkgs.nerd-fonts.symbols-only}/share/fonts/truetype/NerdFonts/Symbols'
      ;;
    *)
      echo "unsupported Yazelix runtime variant: $runtime_variant" >&2
      exit 1
      ;;
  esac

  touch "$out"
''
