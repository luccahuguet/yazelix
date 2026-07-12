# Yazelix Home Manager package-plus-sidecars example
{ inputs, pkgs, ... }:

{
  programs.yazelix = {
    enable = true;
    package = inputs.yazelix.packages.${pkgs.stdenv.hostPlatform.system}.yazelix;

    config.settings = {
      editor.command = "hx";
      welcome.enabled = false;
      popups.zenith = {
        command = "zenith";
        keybinding = "Alt Shift I";
        keep_alive = true;
      };
    };

    config.cursors.source = ./cursors.toml;
    config.mars.text = ''
      [window]
      opacity = 0.9
    '';
    config.zellij.text = ''
      scroll_buffer_size 5000
      mouse_mode true
    '';
  };
}
