#!/bin/sh
# Stable Yazelix CLI entrypoint for external tools and editors.

PATH="$HOME/.local/state/nix/profile/bin:$HOME/.nix-profile/bin:/nix/var/nix/profiles/default/bin:$PATH"

resolve_stable_profile_yzx() {
  if [ -n "${HOME:-}" ]; then
    if [ -e "$HOME/.nix-profile/bin/yzx" ] || [ -L "$HOME/.nix-profile/bin/yzx" ]; then
      printf '%s\n' "$HOME/.nix-profile/bin/yzx"
      return 0
    fi
  fi

  if [ -n "${USER:-}" ]; then
    user_profile_yzx="/etc/profiles/per-user/$USER/bin/yzx"
    if [ -e "$user_profile_yzx" ] || [ -L "$user_profile_yzx" ]; then
      printf '%s\n' "$user_profile_yzx"
      return 0
    fi
  fi

  return 1
}

resolve_wrapper_target() {
  target="$1"
  if [ -z "$target" ]; then
    return 1
  fi

  resolved="$(readlink -f "$target" 2>/dev/null || true)"
  if [ -n "$resolved" ]; then
    printf '%s\n' "$resolved"
  else
    printf '%s\n' "$target"
  fi
}

maybe_redirect_stale_store_invocation() {
  if [ "${YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT:-}" = "1" ]; then
    return 0
  fi

  invoked_yzx_path="${YAZELIX_INVOKED_YZX_PATH:-$0}"
  case "$invoked_yzx_path" in
    /nix/store/*/bin/yzx) ;;
    *) return 0 ;;
  esac

  stable_profile_yzx="$(resolve_stable_profile_yzx)" || return 0
  stable_profile_target="$(resolve_wrapper_target "$stable_profile_yzx")"
  invoked_yzx_target="$(resolve_wrapper_target "$invoked_yzx_path")"

  if [ "$stable_profile_target" = "$invoked_yzx_target" ]; then
    return 0
  fi

  export YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH="$invoked_yzx_path"
  export YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1
  exec "$stable_profile_yzx" "$@"
}

maybe_redirect_stale_store_invocation "$@"

for nix_profile in "$HOME/.nix-profile/etc/profile.d/nix.sh" "/nix/var/nix/profiles/default/etc/profile.d/nix.sh"; do
  if [ -f "$nix_profile" ]; then
    . "$nix_profile"
    break
  fi
done

SCRIPT_PATH="$0"
if [ -L "$SCRIPT_PATH" ]; then
  LINK_TARGET="$(readlink "$SCRIPT_PATH")"
  case "$LINK_TARGET" in
    /*) SCRIPT_PATH="$LINK_TARGET" ;;
    *) SCRIPT_PATH="$(dirname "$SCRIPT_PATH")/$LINK_TARGET" ;;
  esac
fi

RUNTIME_DIR="$(cd "$(dirname "$SCRIPT_PATH")/../.." && pwd)"
runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh"

if [ ! -f "$runtime_env_script" ]; then
  echo "Error: Missing Yazelix runtime env helper: $runtime_env_script" >&2
  exit 1
fi

export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$RUNTIME_DIR"
. "$runtime_env_script" || exit 1
unset YAZELIX_BOOTSTRAP_RUNTIME_DIR

yzx_root_bin="${YAZELIX_YZX_BIN:-$RUNTIME_DIR/libexec/yzx}"
if [ ! -x "$yzx_root_bin" ]; then
  release_candidate="$RUNTIME_DIR/rust_core/target/release/yzx"
  debug_candidate="$RUNTIME_DIR/rust_core/target/debug/yzx"
  if [ -x "$release_candidate" ] && [ -x "$debug_candidate" ]; then
    if [ "$debug_candidate" -nt "$release_candidate" ]; then
      yzx_root_bin="$debug_candidate"
    else
      yzx_root_bin="$release_candidate"
    fi
  elif [ -x "$release_candidate" ]; then
    yzx_root_bin="$release_candidate"
  elif [ -x "$debug_candidate" ]; then
    yzx_root_bin="$debug_candidate"
  fi
fi

if [ ! -x "$yzx_root_bin" ]; then
  echo "Error: Missing Yazelix Rust root helper: $yzx_root_bin" >&2
  echo "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again." >&2
  exit 1
fi

exec "$yzx_root_bin" "$@"
