{ ... }:

let
  variants = {
    mars = {
      desktop_label = "Mars";
      kitty_passthrough = true;
      description = "default Rust terminal with Yazelix-owned Mars integration, generated Mars config, cursor trails, and the Yazelix Zellij Kitty graphics bridge";
    };
  };
  supported = [ "mars" ];
  field = name: terminal: variants.${terminal}.${name};
in
{
  default = "mars";
  inherit supported;
  desktopIdSuffix = field "desktop_label";
  desktopLabel = field "desktop_label";
  description = field "description";
  kittyPassthrough = builtins.filter (terminal: variants.${terminal}.kitty_passthrough or false) supported;
  packageOutput = terminal: "yazelix_${terminal}";
  runtimeOutput = terminal: "runtime_${terminal}";
}
