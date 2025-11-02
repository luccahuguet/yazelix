# devenv.nix - minimal starting point for PoC
{ pkgs, lib, config, inputs, ... }:

{
  # Basic packages to test
  packages = with pkgs; [
    nushell
    zellij
    helix
  ];

  # Test environment variables
  env.YAZELIX_DIR = "$HOME/.config/yazelix";
  env.IN_YAZELIX_SHELL = "true";

  # Test shell hook execution
  enterShell = ''
    echo "✅ devenv shell activated"
  '';

  # Required for flakes usage - set to yazelix directory
  devenv.root = builtins.getEnv "HOME" + "/.config/yazelix";
}
