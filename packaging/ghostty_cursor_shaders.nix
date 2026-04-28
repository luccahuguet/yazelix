{
  pkgs,
  src ? ../.,
  rust_core_src ? src,
  fenixPkgs ? null,
}:

let
  rustCoreHelper = import ./rust_core_helper.nix {
    inherit pkgs fenixPkgs;
    src = rust_core_src;
  };
in
pkgs.runCommand "yazelix-ghostty-cursor-shaders"
  {
    nativeBuildInputs = [ pkgs.nushell ];
    meta = {
      description = "Standalone Ghostty cursor shader export generated from Yazelix cursor presets";
      homepage = "https://github.com/luccahuguet/yazelix";
      license = pkgs.lib.licenses.mit;
      platforms = pkgs.lib.platforms.all;
    };
  }
  ''
    set -eu

    work="$TMPDIR/yazelix_ghostty_cursor_shader_export"
    config_dir="$work/config"
    state_dir="$work/state"
    share_dir="$out/share/yazelix/ghostty_cursor_shaders"
    shader_out="$share_dir/shaders"
    examples_dir="$share_dir/examples"

    mkdir -p "$config_dir/user_configs" "$state_dir" "$examples_dir"

    PATH="${pkgs.nushell}/bin:$PATH" \
      ${rustCoreHelper}/bin/yzx_core ghostty-materialization.generate \
        --runtime-dir ${src} \
        --config-dir "$config_dir" \
        --state-dir "$state_dir" \
        --transparency none \
        --cursor-config ${src}/yazelix_cursors_default.toml \
        > "$work/materialization.json"

    generated_shaders="$state_dir/configs/terminal_emulators/ghostty/shaders"
    generated_config="$state_dir/configs/terminal_emulators/ghostty/config"

    cp -R "$generated_shaders" "$shader_out"

    cat > "$examples_dir/ghostty_blaze_tail.conf" <<EOF
# Yazelix Ghostty cursor shader export example
#
# Add these lines to a Ghostty config to try the blaze palette with the tail effect
custom-shader = $shader_out/cursor_trail_blaze.glsl
custom-shader = $shader_out/generated_effects/tail.glsl
EOF

    cat > "$share_dir/README.md" <<EOF
# Yazelix Ghostty Cursor Shaders

This package exports complete Ghostty cursor shader files generated from Yazelix cursor presets

Use one cursor palette shader and one optional effect shader in your Ghostty config:

\`\`\`conf
custom-shader = $shader_out/cursor_trail_blaze.glsl
custom-shader = $shader_out/generated_effects/tail.glsl
\`\`\`

Generated shader root:

\`\`\`text
$shader_out
\`\`\`

Example config:

\`\`\`text
$examples_dir/ghostty_blaze_tail.conf
\`\`\`

This package does not mutate your Ghostty config and does not include Yazelix runtime reroll behavior
EOF

    required_files="
      $generated_config
      $shader_out/cursor_trail_blaze.glsl
      $shader_out/cursor_trail_snow.glsl
      $shader_out/cursor_trail_neon.glsl
      $shader_out/cursor_trail_inferno.glsl
      $shader_out/generated_effects/tail.glsl
      $shader_out/generated_effects/ripple.glsl
      $examples_dir/ghostty_blaze_tail.conf
    "
    for required in $required_files; do
      test -s "$required"
    done
    grep -q "custom-shader = $shader_out/cursor_trail_blaze.glsl" "$examples_dir/ghostty_blaze_tail.conf"
  ''
