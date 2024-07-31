#!/bin/sh

# Move focus to the next pane
zellij action focus-next-pane

# Get the running command in the current pane
RUNNING_COMMAND=$(zellij action list-clients | awk 'NR==2 {print $3}')

# Check if the command running in the current pane is helix (hx)
if echo "$RUNNING_COMMAND" | grep -q "/hx$"; then
    # The current pane is running helix, use zellij actions to open the file
    zellij action write 27
    zellij action write-chars ":open $1"
    zellij action write 13
else
    # The current pane is not running helix, so open helix in a new pane
    zellij action new-pane
    sleep 0.3
    # Get the working directory
    if [ -d "$1" ]; then
        WORKING_DIR="$1"
    else
        WORKING_DIR=$(dirname "$1")
    fi
    zellij action write-chars "hx $1 -w $WORKING_DIR"
    zellij action write 13
fi
