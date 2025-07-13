# validate-config.nix
# Simple Nix expression to evaluate user's yazelix.nix config and output as JSON

let
  # Get HOME from environment or use a default
  homeDir = builtins.getEnv "HOME";

  # Import the user's config with proper error handling
  userConfig =
    if homeDir != "" then
      let
        configFile = "${homeDir}/.config/yazelix/yazelix.nix";
        defaultConfigFile = "${homeDir}/.config/yazelix/yazelix_default.nix";
      in
      if builtins.pathExists configFile then
        import configFile { pkgs = import <nixpkgs> { }; }
      else if builtins.pathExists defaultConfigFile then
        import defaultConfigFile { pkgs = import <nixpkgs> { }; }
      else
        throw "No yazelix config found at ${configFile} or ${defaultConfigFile}"
    else
      throw "HOME environment variable is not set";
in
builtins.toJSON userConfig
