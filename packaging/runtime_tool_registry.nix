{
  pkgs,
  nixgl ? null,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
}:

let
  lib = pkgs.lib;
  runtimeToolSourceModes = [
    "bundled"
    "host"
    "off"
  ];
  ghosttyPackage =
    if pkgs.stdenv.hostPlatform.isDarwin then
      pkgs."ghostty-bin"
    else
      pkgs.ghostty;
  terminalPackage =
    if runtimeVariant == "ghostty" then
      ghosttyPackage
    else if runtimeVariant == "wezterm" then
      pkgs.wezterm
    else if runtimeVariant == "ratty" then
      if pkgs.stdenv.hostPlatform.isLinux then
        pkgs.ratty
      else
        throw "Yazelix runtimeVariant ratty is only supported on Linux"
    else
      throw "Unsupported Yazelix runtimeVariant: ${runtimeVariant}";
  terminalCommands =
    if runtimeVariant == "ghostty" then
      [ "ghostty" ]
    else if runtimeVariant == "wezterm" then
      [ "wezterm" ]
    else if runtimeVariant == "ratty" then
      [ "ratty" ]
    else
      [ ];
  linuxGraphicsWrappers =
    if pkgs.stdenv.hostPlatform.isLinux && (nixgl != null) then
      import "${nixgl}/default.nix" {
        pkgs = pkgs;
        enable32bits = pkgs.stdenv.hostPlatform.isx86_64;
        enableIntelX86Extensions = pkgs.stdenv.hostPlatform.isx86_64;
      }
    else
      null;
  linuxGlWrapperPackage =
    if linuxGraphicsWrappers != null then
      linuxGraphicsWrappers.nixGLMesa
    else
      null;
  linuxVulkanWrapperPackage =
    if linuxGraphicsWrappers != null && runtimeVariant == "ratty" then
      linuxGraphicsWrappers.nixVulkanMesa
    else
      null;
  makeTool =
    {
      package,
      commands,
      requiredCommands ? commands,
      hostable ? false,
      disableable ? false,
      notes ? [ ],
    }:
    {
      inherit package commands requiredCommands hostable disableable notes;
    };
  tools =
    with pkgs;
    {
      bash = makeTool {
        package = bashInteractive;
        commands = [ "bash" ];
      };
      nushell = makeTool {
        package = nushell;
        commands = [ "nu" ];
      };
      zellij = makeTool {
        package = zellij;
        commands = [ "zellij" ];
      };
      terminal = makeTool {
        package = terminalPackage;
        commands = terminalCommands;
      };
      helix = makeTool {
        package = helix;
        commands = [
          "hx"
          "helix"
        ];
        requiredCommands = [ "hx" ];
        hostable = true;
      };
      neovim = makeTool {
        package = neovim;
        commands = [
          "nvim"
          "neovim"
        ];
        requiredCommands = [ "nvim" ];
        hostable = true;
      };
      yazi = makeTool {
        package = yazi;
        commands = [
          "yazi"
          "ya"
        ];
        requiredCommands = [ "yazi" ];
        hostable = true;
      };
      fzf = makeTool {
        package = fzf;
        commands = [ "fzf" ];
        hostable = true;
      };
      zoxide = makeTool {
        package = zoxide;
        commands = [ "zoxide" ];
        hostable = true;
      };
      starship = makeTool {
        package = starship;
        commands = [ "starship" ];
        hostable = true;
      };
      lazygit = makeTool {
        package = lazygit;
        commands = [
          "lazygit"
          "lg"
        ];
        requiredCommands = [ "lazygit" ];
        hostable = true;
      };
      carapace = makeTool {
        package = carapace;
        commands = [ "carapace" ];
        hostable = true;
      };
      macchina = makeTool {
        package = macchina;
        commands = [ "macchina" ];
        hostable = true;
        disableable = true;
        notes = [ "Optional welcome summary helper. Off mode requires welcome macchina output to be disabled." ];
      };
      mise = makeTool {
        package = mise;
        commands = [ "mise" ];
        hostable = true;
      };
      tombi = makeTool {
        package = tombi;
        commands = [ "tombi" ];
        hostable = true;
      };
      fish = makeTool {
        package = fish;
        commands = [ "fish" ];
      };
      zsh = makeTool {
        package = zsh;
        commands = [ "zsh" ];
      };
      git = makeTool {
        package = git;
        commands = [ "git" ];
        hostable = true;
      };
      jq = makeTool {
        package = jq;
        commands = [ "jq" ];
        hostable = true;
      };
      fd = makeTool {
        package = fd;
        commands = [ "fd" ];
        hostable = true;
      };
      ripgrep = makeTool {
        package = ripgrep;
        commands = [ "rg" ];
        hostable = true;
      };
      p7zip = makeTool {
        package = p7zip;
        commands = [
          "7z"
          "7za"
          "7zr"
        ];
        disableable = true;
        notes = [ "Optional Yazi/archive helper. Off mode intentionally omits archive helper commands from the runtime." ];
      };
      poppler = makeTool {
        package = poppler;
        commands = [
          "pdfinfo"
          "pdftotext"
          "pdftoppm"
          "pdftocairo"
        ];
        disableable = true;
        notes = [ "Optional Yazi/PDF preview helper. Off mode intentionally omits PDF helper commands from the runtime." ];
      };
      resvg = makeTool {
        package = resvg;
        commands = [ "resvg" ];
        disableable = true;
        notes = [ "Optional SVG preview helper. Off mode intentionally omits SVG helper commands from the runtime." ];
      };
      nix = makeTool {
        package = nixVersions.latest;
        commands = [ ];
      };
      coreutils = makeTool {
        package = coreutils;
        commands = [ ];
      };
      findutils = makeTool {
        package = findutils;
        commands = [ ];
      };
      gnugrep = makeTool {
        package = gnugrep;
        commands = [ ];
      };
      gnused = makeTool {
        package = gnused;
        commands = [ ];
      };
      util_linux = makeTool {
        package = util-linux;
        commands = [ ];
      };
    }
    // lib.optionalAttrs (linuxGlWrapperPackage != null) {
      nixgl_mesa = makeTool {
        package = linuxGlWrapperPackage;
        commands = [ ];
      };
    }
    // lib.optionalAttrs (linuxVulkanWrapperPackage != null) {
      nixvulkan_mesa = makeTool {
        package = linuxVulkanWrapperPackage;
        commands = [ ];
      };
    }
    // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
      procps = makeTool {
        package = procps;
        commands = [ ];
      };
      xclip = makeTool {
        package = xclip;
        commands = [ ];
      };
      wl_clipboard = makeTool {
        package = wl-clipboard;
        commands = [ ];
      };
      xsel = makeTool {
        package = xsel;
        commands = [ ];
      };
    };
  runtimeToolNames = builtins.attrNames runtimeToolSources;
  unknownToolNames = lib.filter (name: !(builtins.hasAttr name tools)) runtimeToolNames;
  invalidSourceNames = lib.filter (
    name:
    let
      source = runtimeToolSources.${name};
    in
    !(builtins.isString source && builtins.elem source runtimeToolSourceModes)
  ) runtimeToolNames;
  disallowedHostNames = lib.filter (
    name: runtimeToolSources.${name} == "host" && !(tools.${name}.hostable or false)
  ) runtimeToolNames;
  disallowedOffNames = lib.filter (
    name: runtimeToolSources.${name} == "off" && !(tools.${name}.disableable or false)
  ) runtimeToolNames;
  sourceFor = name: runtimeToolSources.${name} or "bundled";
  bundledToolNames = lib.filter (name: sourceFor name == "bundled") (builtins.attrNames tools);
  bundledTools = map (name: tools.${name}) bundledToolNames;
  runtimePackages = lib.unique (map (tool: tool.package) bundledTools);
  exportedCommands = lib.unique (lib.concatMap (tool: tool.commands) bundledTools);
  manifest = lib.mapAttrs (name: tool: {
    source = sourceFor name;
    commands = tool.commands;
    required_commands = tool.requiredCommands;
    hostable = tool.hostable;
    disableable = tool.disableable;
    notes = tool.notes;
  }) tools;
in
if unknownToolNames != [ ] then
  throw "Unsupported Yazelix runtimeToolSources tool(s): ${lib.concatStringsSep ", " unknownToolNames}"
else if invalidSourceNames != [ ] then
  throw "Unsupported Yazelix runtimeToolSources value(s) for: ${lib.concatStringsSep ", " invalidSourceNames}. Expected one of: ${lib.concatStringsSep ", " runtimeToolSourceModes}"
else if disallowedHostNames != [ ] then
  throw "Yazelix runtimeToolSources host mode is not supported for: ${lib.concatStringsSep ", " disallowedHostNames}"
else if disallowedOffNames != [ ] then
  throw "Yazelix runtimeToolSources off mode is not supported for: ${lib.concatStringsSep ", " disallowedOffNames}"
else
  {
    inherit runtimeToolSourceModes tools runtimePackages exportedCommands manifest;
    manifestJson = builtins.toJSON manifest;
  }
