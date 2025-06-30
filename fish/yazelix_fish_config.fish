# Yazelix Fish Configuration
# This file is sourced by ~/.config/fish/config.fish

# Source generated initializers if they exist
set -l FISH_INITIALIZERS_DIR "$HOME/.config/yazelix/fish/initializers"

# Source each initializer if it exists
for file in $FISH_INITIALIZERS_DIR/*.fish
    if test -f $file
        source $file
    end
end

# Yazelix aliases
alias yazelix="$HOME/.config/yazelix/bash/launch-yazelix.sh"
alias yzx="$HOME/.config/yazelix/bash/launch-yazelix.sh" 