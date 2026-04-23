#!/bin/sh
# Minimal POSIX launcher for terminal configs (no bashrc)

PATH="$HOME/.local/state/nix/profile/bin:$HOME/.nix-profile/bin:/nix/var/nix/profiles/default/bin:$PATH"

# Load Nix profile if available (mirrors login shell behavior)
for nix_profile in "$HOME/.nix-profile/etc/profile.d/nix.sh" "/nix/var/nix/profiles/default/etc/profile.d/nix.sh"; do
  if [ -f "$nix_profile" ]; then
    . "$nix_profile"
    break
  fi
done

RUNTIME_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh"

if [ ! -f "$runtime_env_script" ]; then
  echo "Error: Missing Yazelix runtime env helper: $runtime_env_script" >&2
  exit 1
fi

export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$RUNTIME_DIR"
. "$runtime_env_script" || exit 1
unset YAZELIX_BOOTSTRAP_RUNTIME_DIR

yzx_control_bin="${YAZELIX_YZX_CONTROL_BIN:-$RUNTIME_DIR/libexec/yzx_control}"
if [ ! -x "$yzx_control_bin" ]; then
  release_candidate="$RUNTIME_DIR/rust_core/target/release/yzx_control"
  debug_candidate="$RUNTIME_DIR/rust_core/target/debug/yzx_control"
  if [ -x "$release_candidate" ] && [ -x "$debug_candidate" ]; then
    if [ "$debug_candidate" -nt "$release_candidate" ]; then
      yzx_control_bin="$debug_candidate"
    else
      yzx_control_bin="$release_candidate"
    fi
  elif [ -x "$release_candidate" ]; then
    yzx_control_bin="$release_candidate"
  elif [ -x "$debug_candidate" ]; then
    yzx_control_bin="$debug_candidate"
  fi
fi

if [ ! -x "$yzx_control_bin" ]; then
  echo "Error: Missing Yazelix Rust control helper: $yzx_control_bin" >&2
  echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
  exit 1
fi

exec "$yzx_control_bin" enter
