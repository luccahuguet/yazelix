#!/bin/bash
# ~/.config/yazelix/shells/bash/start_yazelix.sh

# Resolve HOME using shell expansion
HOME=$(eval echo ~)
if [ -z "$HOME" ] || [ ! -d "$HOME" ]; then
  echo "Error: Cannot resolve HOME directory"
  exit 1
fi

# Resolve Yazelix runtime root from this script location
runtime_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

runtime_env_script="$runtime_dir/shells/posix/runtime_env.sh"
if [ ! -f "$runtime_env_script" ]; then
  echo "Error: Missing Yazelix runtime env helper: $runtime_env_script"
  exit 1
fi

export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$runtime_dir"
. "$runtime_env_script" || exit 1
unset YAZELIX_BOOTSTRAP_RUNTIME_DIR

echo "Resolved HOME=$HOME"

# Navigate to Yazelix directory
# This is important for devenv to find devenv.nix in the current directory
cd "$runtime_dir" || { echo "Error: Cannot cd to $runtime_dir"; exit 1; }

# Prefer runtime-owned tools over host/profile PATH entries.
DEVENV_BIN="$runtime_dir/bin/devenv"
if [ ! -x "$DEVENV_BIN" ]; then
  DEVENV_BIN="$(command -v devenv 2>/dev/null || true)"
fi

# Ensure devenv is available
if [ -z "$DEVENV_BIN" ] || [ ! -x "$DEVENV_BIN" ]; then
  echo ""
  echo "❌ 'devenv' command not found in the installed Yazelix runtime."
  echo "   Repair the runtime with:"
  echo "     yzx update runtime"
  echo "   Then rerun Yazelix."
  echo ""
  exit 1
fi

if [ -z "$YAZELIX_NU_BIN" ] || [ ! -x "$YAZELIX_NU_BIN" ]; then
  echo ""
  echo "❌ 'nu' command not found in the installed Yazelix runtime."
  echo "   Repair the runtime with:"
  echo "     yzx update runtime"
  echo "   Then rerun Yazelix."
  echo ""
  exit 1
fi

# Detect configuration changes (requires Nushell)
if [ -x "$YAZELIX_NU_BIN" ]; then
  NEEDS_REFRESH=$("$YAZELIX_NU_BIN" -c "use \"$runtime_dir/nushell/scripts/utils/config_state.nu\" compute_config_state; let state = compute_config_state; if \$state.needs_refresh { 'true' } else { '' }")
  if [ "$NEEDS_REFRESH" = "true" ]; then
    REFRESH_REASON=$("$YAZELIX_NU_BIN" -c "use \"$runtime_dir/nushell/scripts/utils/config_state.nu\" compute_config_state; let state = compute_config_state; if (\$state.refresh_reason? | is-not-empty) { \$state.refresh_reason } else { 'config or devenv inputs changed since last launch' }")
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
HOME="$HOME" "$DEVENV_BIN" --cores "$MAX_CORES" shell -- \
  "$YAZELIX_NU_BIN" "$runtime_dir/nushell/scripts/core/start_yazelix_inner.nu"

# Hash is now saved during enterShell hook in devenv.nix
