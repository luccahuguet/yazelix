# Yazelix v7 Quick Demo - Essential Features
# A concise demonstration of yazelix v7's core capabilities

Output yazelix-v7-quick-demo.gif

Set Theme "Catppuccin Mocha"
Set FontSize 12
Set Width 1100
Set Height 800
Set LetterSpacing 2
Set Shell "bash"
Set TypingSpeed 10ms
Set Margin 20
Set MarginFill "#674EFF"
Set BorderRadius 10

Show

# Quick intro
Type "# Yazelix v7: zellij, yazi, nushell and lua scripting in a nix flake add a File Tree to Helix, in an IDE-like experience"
Enter
Sleep 2s

Set TypingSpeed 40ms

# Start yazelix
# Type " cd ~/.config/yazelix && nix develop --impure --command bash -c 'zellij --config-dir ~/.config/yazelix/configs/zellij options --default-layout yazelix --default-shell nu'"
Type " ~/.config/yazelix/shells/bash/start_yazelix.sh"
Enter
Sleep 8s
Enter
Sleep 500ms

# exit the zellij welcome screen
Ctrl+c
Sleep 500ms

# Navigate in Yazi (we're now in the file manager)
# use zoxide
Sleep 500ms
Type "Z"
Sleep 500ms
Type "yaze"
Sleep 500ms
Enter
Sleep 500ms
Left
Sleep 500ms
# enter yazelix repo
Enter
Sleep 500ms
Type "scripts"

Enter            # Open file in Helix
Sleep 500ms

# Now we're in Helix editor
Type "o"         # Insert mode
Sleep 500ms
Type "# Yazelix v7 rocks! 🚀 IDE-like experience with Nix!"
Sleep 500ms
Escape           # Back to normal mode
Sleep 500ms

# Enter tab mode

# Ctrl+t
# Sleep 2s
# Type "n"
# Sleep 2s

# Create a new zellij pane
Alt+M
Sleep 500ms

Type "z yaze"
Sleep 500ms
Enter
Sleep 500ms

# Now we're in a Nushell terminal
Type " # Welcome to Nushell..."
Enter
Sleep 1s
Type " ls | select name type size | first 3"
Enter
Sleep 1s

# Start lazygit for git integration demo
Type " lg"
Enter
Sleep 2s
Type "q"         # Quit lazygit
Sleep 1s

# Final showcase
Type " echo 'Hope you enjoyed this'"
Sleep 1s
