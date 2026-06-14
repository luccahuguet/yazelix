{ isLinux }:

let
  order = [ "ghostty" "kitty" "rio" "yzxterm" "wezterm" "foot" "ratty" ];
  variants = {
    ghostty = {
      desktop_label = "Ghostty";
      kitty_passthrough = true;
      description = "default packaged terminal with Yazelix cursor trails, Ghostty config effects, and Yazi image previews through Zellij";
    };
    kitty = {
      desktop_label = "Kitty";
      kitty_passthrough = true;
      description = "packaged Kitty terminal with generated Kitty config and the Yazelix Zellij Kitty graphics bridge";
    };
    rio = {
      desktop_label = "Rio";
      kitty_passthrough = true;
      description = "Rio-compatible Yazelix Terminal child package path with generated Rio config and the Yazelix Zellij Kitty graphics bridge";
    };
    yzxterm = {
      desktop_label = "Yzxterm";
      kitty_passthrough = true;
      description = "experimental Yazelix-owned Rio fork with Rio trail cursor defaults and opt-in shader support";
    };
    wezterm = {
      desktop_label = "WezTerm";
      description = "explicit alternate packaged terminal";
    };
    foot = {
      desktop_label = "Foot";
      linux_only = true;
      description = "Linux packaged Foot terminal with generated Foot config";
    };
    ratty = {
      desktop_label = "Ratty";
      kitty_passthrough = true;
      linux_only = true;
      description = "experimental Linux packaged terminal with Ratty and the Yazelix Zellij Kitty graphics bridge";
    };
  };
  supported =
    builtins.filter (terminal: !(variants.${terminal}.linux_only or false) || isLinux) order;
  field = name: terminal: variants.${terminal}.${name};
in
{
  default = "ghostty";
  inherit supported;
  desktopIdSuffix = field "desktop_label";
  desktopLabel = field "desktop_label";
  description = field "description";
  kittyPassthrough = builtins.filter (terminal: variants.${terminal}.kitty_passthrough or false) supported;
  packageOutput = terminal: if terminal == "yzxterm" then "yzxterm" else "yazelix_${terminal}";
  runtimeOutput = terminal: "runtime_${terminal}";
}
