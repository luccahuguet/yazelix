{
  home.username = "demo"; # Change to your username
  home.homeDirectory = "/home/demo"; # Change to your home directory
  home.stateVersion = "24.11";

  programs.home-manager.enable = true;

  programs.yazelix = {
    enable = true;
  };
}
