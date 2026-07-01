{ pkgs, runtime }:

pkgs.runCommand "yazelix-runtime-release-contracts" { } ''
  set -eu

  runtime=${runtime}

  test -x "$runtime/bin/yzx"
  test -s "$runtime/settings_default.jsonc"
  test -s "$runtime/runtime_identity.json"
  test -s "$runtime/runtime_tools.json"
  test -s "$runtime/runtime_components.json"

  for size in 48x48 64x64 128x128 256x256; do
    test -s "$runtime/assets/icons/$size/yazelix.png"
  done

  test -s "$runtime/configs/zellij/plugins/zjstatus.wasm"
  if grep -R -I -F 'https://github.com/dj95/zjstatus/releases/latest/download/zjstatus.wasm' \
    "$runtime/configs" "$runtime/shells" >/dev/null; then
    echo "Yazelix runtime must use packaged file-backed zjstatus.wasm, not upstream URL auto-download" >&2
    exit 1
  fi

  mars_config="$runtime/share/mars/config.toml"
  test -s "$mars_config"
  grep -F 'family = "JetBrains Mono"' "$mars_config" >/dev/null
  grep -F 'font-family = "Symbols Nerd Font Mono"' "$mars_config" >/dev/null
  grep -F '${pkgs.jetbrains-mono}/share/fonts/truetype' "$mars_config" >/dev/null
  grep -F '${pkgs.nerd-fonts.symbols-only}/share/fonts/truetype/NerdFonts/Symbols' "$mars_config" >/dev/null

  test -d '${pkgs.jetbrains-mono}/share/fonts/truetype'
  test -d '${pkgs.nerd-fonts.symbols-only}/share/fonts/truetype/NerdFonts/Symbols'

  touch "$out"
''
