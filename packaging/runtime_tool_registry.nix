{
  pkgs,
  nixgl ? null,
  runtimeVariant ? "ghostty",
  runtimeToolSources ? { },
  yazelixTerminalPackage ? null,
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
  commandBasename = command: lib.last (lib.splitString "/" command);
  requireYzxtermPackageMetadata =
    package:
    let
      metadata = package.passthru.yzxtermPackageMetadata or null;
    in
    if !(builtins.isAttrs metadata) then
      throw "Yazelix runtimeVariant yzxterm requires the terminal package to expose passthru.yzxtermPackageMetadata"
    else if (metadata.schema_version or null) != 1 then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.schema_version = 1"
    else if (metadata.terminal or null) != "yazelix-terminal" then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.terminal = \"yazelix-terminal\""
    else if !(builtins.isString (metadata.package_name or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.package_name"
    else if !(builtins.isString (metadata.package_profile or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.package_profile"
    else if !(builtins.isBool (metadata.checked_package or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.checked_package"
    else if !(builtins.isString (metadata.metadata_path or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.metadata_path"
    else if !(builtins.isString (metadata.wrapper_commands.desktop or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.wrapper_commands.desktop"
    else if !(builtins.isAttrs (metadata.config_roots or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.config_roots"
    else if !(builtins.isList (metadata.supported_emoji_fonts or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.supported_emoji_fonts"
    else if !(builtins.elem "noto" metadata.supported_emoji_fonts) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.supported_emoji_fonts to include noto"
    else if !(builtins.elem "twitter" metadata.supported_emoji_fonts) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.supported_emoji_fonts to include twitter"
    else if !(builtins.elem "serenityos" metadata.supported_emoji_fonts) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.supported_emoji_fonts to include serenityos"
    else if !(builtins.isList (metadata.supported_appearance_modes or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.supported_appearance_modes"
    else if !(builtins.elem "dark" metadata.supported_appearance_modes) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.supported_appearance_modes to include dark"
    else if !(builtins.elem "light" metadata.supported_appearance_modes) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.supported_appearance_modes to include light"
    else if !(builtins.elem "auto" metadata.supported_appearance_modes) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.supported_appearance_modes to include auto"
    else if (metadata.default_appearance_mode or null) != "dark" then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.default_appearance_mode = \"dark\""
    else if !(builtins.isString (metadata.wrapper_env.appearance or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.wrapper_env.appearance"
    else if !(builtins.isString (metadata.wrapper_env.emoji_font or null)) then
      throw "Yazelix runtimeVariant yzxterm requires yzxtermPackageMetadata.wrapper_env.emoji_font"
    else
      metadata;
  yzxtermPackageMetadata =
    if runtimeVariant == "yzxterm" then
      if yazelixTerminalPackage != null then
        requireYzxtermPackageMetadata yazelixTerminalPackage
      else
        throw "Yazelix runtimeVariant yzxterm requires the yazelix-terminal child package"
    else
      null;
  yzxtermPackageRuntimeIdentity =
    if yzxtermPackageMetadata == null then
      { }
    else
      {
        package_profile =
          if yzxtermPackageMetadata.package_profile == "fast" then
            "yzxterm-fast"
          else
            "yzxterm-${yzxtermPackageMetadata.package_profile}";
        yzxterm_terminal_package = yzxtermPackageMetadata.package_name;
        yzxterm_terminal_package_profile = yzxtermPackageMetadata.package_profile;
        yzxterm_terminal_checked = yzxtermPackageMetadata.checked_package;
        yzxterm_terminal_metadata_schema = yzxtermPackageMetadata.schema_version;
        yzxterm_terminal_supported_appearance_modes =
          yzxtermPackageMetadata.supported_appearance_modes;
        yzxterm_terminal_default_appearance_mode =
          yzxtermPackageMetadata.default_appearance_mode;
      };
  terminalPackage =
    if runtimeVariant == "ghostty" then
      ghosttyPackage
    else if runtimeVariant == "kitty" then
      pkgs.kitty
    else if runtimeVariant == "rio" then
      pkgs.rio
    else if runtimeVariant == "wezterm" then
      pkgs.wezterm
    else if runtimeVariant == "ratty" then
      if pkgs.stdenv.hostPlatform.isLinux then
        pkgs.ratty
      else
        throw "Yazelix runtimeVariant ratty is only supported on Linux"
    else if runtimeVariant == "foot" then
      if pkgs.stdenv.hostPlatform.isLinux then
        pkgs.foot
      else
        throw "Yazelix runtimeVariant foot is only supported on Linux"
    else if runtimeVariant == "yzxterm" then
      if yzxtermPackageMetadata != null then
        yazelixTerminalPackage
      else
        throw "Yazelix runtimeVariant yzxterm requires the yazelix-terminal child package"
    else
      throw "Unsupported Yazelix runtimeVariant: ${runtimeVariant}";
  terminalCommands =
    if runtimeVariant == "ghostty" then
      [ "ghostty" ]
    else if runtimeVariant == "kitty" then
      [ "kitty" ]
    else if runtimeVariant == "rio" then
      [ "rio" ]
    else if runtimeVariant == "wezterm" then
      [ "wezterm" ]
    else if runtimeVariant == "ratty" then
      [ "ratty" ]
    else if runtimeVariant == "foot" then
      [ "foot" ]
    else if runtimeVariant == "yzxterm" then
      [ (commandBasename yzxtermPackageMetadata.wrapper_commands.desktop) ]
    else
      [ ];
  linuxGraphicsWrappers =
    if pkgs.stdenv.hostPlatform.isLinux && (nixgl != null) then
      import "${nixgl}/default.nix" {
        pkgs = pkgs;
        enable32bits = false;
        enableIntelX86Extensions = false;
      }
    else
      null;
  linuxGlWrapperPackage =
    if linuxGraphicsWrappers != null then
      linuxGraphicsWrappers.nixGLMesa
    else
      null;
  linuxVulkanWrapperPackage =
    if linuxGraphicsWrappers != null && builtins.elem runtimeVariant [ "ratty" "yzxterm" ] then
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
        notes = [ "Bundled mode uses the Yazelix-owned Helix Steel fork with --config-dir support." ];
      };
      steel = makeTool {
        package = steel;
        commands = [
          "steel"
          "steel-language-server"
          "forge"
          "cargo-steel-lib"
          "repl-connect"
        ];
        requiredCommands = [
          "steel"
          "steel-language-server"
        ];
        hostable = true;
        disableable = true;
        notes = [ "Optional Helix Steel plugin authoring tools. Managed plugin execution does not depend on these commands." ];
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
      zenith = makeTool {
        package = pkgs.zenith;
        commands = [ "zenith" ];
        requiredCommands = [ "zenith" ];
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
        notes = [ "optional_host_integration" ];
      };
      tombi = makeTool {
        package = tombi;
        commands = [ "tombi" ];
        hostable = true;
        notes = [ "optional_host_integration" ];
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
  defaultSourceFor =
    name:
    if builtins.elem name [
      "mise"
      "tombi"
    ] then
      "host"
    else
      "bundled";
  sourceFor = name: runtimeToolSources.${name} or (defaultSourceFor name);
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
    terminalPackageMetadata = yzxtermPackageMetadata;
    terminalPackageRuntimeIdentity = yzxtermPackageRuntimeIdentity;
    manifestJson = builtins.toJSON manifest;
  }
