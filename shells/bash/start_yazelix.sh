#!/bin/bash
# ~/.config/yazelix/shells/bash/start_yazelix.sh

# Resolve HOME using shell expansion
HOME=$(eval echo ~)
if [ -z "$HOME" ] || [ ! -d "$HOME" ]; then
  echo "Error: Cannot resolve HOME directory"
  exit 1
fi

echo "Resolved HOME=$HOME"

# Resolve Yazelix runtime root from this script location
YAZELIX_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Navigate to Yazelix directory
# This is important for devenv to find devenv.nix in the current directory
cd "$YAZELIX_DIR" || { echo "Error: Cannot cd to $YAZELIX_DIR"; exit 1; }

# Ensure devenv is available
if ! command -v devenv >/dev/null 2>&1; then
  echo ""
  echo "❌ 'devenv' command not found."
  echo "   Yazelix v11+ moved from flake-based 'nix develop' shells to devenv."
  echo "   Install devenv with:"
  echo "     nix profile install github:cachix/devenv/latest"
  echo "   After installing, relaunch Yazelix (or run 'devenv shell')."
  echo "   Old commands like 'nix develop' are no longer supported."
  echo ""
  exit 1
fi

# Detect configuration changes (requires Nushell)
if command -v nu >/dev/null 2>&1; then
  NEEDS_REFRESH=$(nu -c "use \"$YAZELIX_DIR/nushell/scripts/utils/config_state.nu\" compute_config_state; let state = compute_config_state; if \$state.needs_refresh { 'true' } else { '' }")
  if [ "$NEEDS_REFRESH" = "true" ]; then
    REFRESH_REASON=$(nu -c "use \"$YAZELIX_DIR/nushell/scripts/utils/config_state.nu\" compute_config_state; let state = compute_config_state; if (\$state.refresh_reason? | is-not-empty) { \$state.refresh_reason } else { 'config or devenv inputs changed since last launch' }")
    echo "♻️  ${REFRESH_REASON} – rebuilding environment"
  fi
fi

# Run devenv shell with explicit HOME and let the Nushell launcher read the
# current config directly before starting Zellij.
# Detect number of CPU cores (cross-platform)
if command -v nproc >/dev/null 2>&1; then
  MAX_CORES=$(nproc)  # Linux
else
  MAX_CORES=$(sysctl -n hw.ncpu)  # macOS
fi
HOME="$HOME" devenv --cores "$MAX_CORES" shell -- \
  nu "$YAZELIX_DIR/nushell/scripts/core/start_yazelix_inner.nu"

# Hash is now saved during enterShell hook in devenv.nix
