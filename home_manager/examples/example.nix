# Yazelix Home Manager sparse config and sidecars example
{ ... }:

{
  programs.yazelix = {
    enable = true;

    config.settings = {
      editor.command = "hx";
      welcome.enabled = false;
      popups.zenith = {
        command = "zenith";
        keybinding = "Alt Shift I";
        keep_alive = true;
      };
    };

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
