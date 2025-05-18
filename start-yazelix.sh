#!/bin/bash

# Navigate to Yazelix directory to ensure flake.nix is found
cd ~/.config/yazelix || { echo "Error: Cannot cd to ~/.config/yazelix"; exit 1; }

# Enter Nix development shell and start Zellij with Nushell as default shell
nix develop --command zellij --config-dir ~/.config/yazelix/zellij options --default-layout yazelix --default-shell nu
# nix develop --command zellij --config-dir ~/.config/yazelix/zellij options --default-layout yazelix --default-shell "nu --config ~/.config/yazelix/nushell/config.nu"
