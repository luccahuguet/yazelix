#!/bin/sh
if [ -z "${YAZELIX_STATE_DIR:-}" ]; then
  if [ -n "${XDG_DATA_HOME:-}" ]; then
    YAZELIX_STATE_DIR="$XDG_DATA_HOME/yazelix"
  elif [ -n "${HOME:-}" ]; then
    YAZELIX_STATE_DIR="$HOME/.local/share/yazelix"
  else
    printf '%s\n' 'yzx-hx: HOME is required when YAZELIX_STATE_DIR and XDG_DATA_HOME are unset' >&2
    exit 1
  fi
fi
export YAZELIX_STATE_DIR
if [ -z "${YAZELIX_CONFIG_HOME:-}" ]; then
  if [ -n "${XDG_CONFIG_HOME:-}" ]; then
    YAZELIX_CONFIG_HOME="$XDG_CONFIG_HOME/yazelix"
  elif [ -n "${HOME:-}" ]; then
    YAZELIX_CONFIG_HOME="$HOME/.config/yazelix"
  else
    printf '%s\n' 'yzx-hx: HOME is required when YAZELIX_CONFIG_HOME and XDG_CONFIG_HOME are unset' >&2
    exit 1
  fi
fi
user_helix_dir="$YAZELIX_CONFIG_HOME/helix"
user_helix_config="$user_helix_dir/config.toml"
packaged_helix_dir="@yzxHelixConfig@"
packaged_helix_config="$packaged_helix_dir/config.toml"
packaged_steel_dir="@yzxHelixSteelConfig@"
effective_helix_config="$YAZELIX_STATE_DIR/helix/config.toml"
helix_config_dir="$packaged_helix_dir"
helix_config_file="$effective_helix_config"
steel_config_dir="$packaged_steel_dir"
steel_config_dir_needs_mkdir=false

if [ -f "$user_helix_config" ] ||
  [ -f "$user_helix_dir/languages.toml" ] ||
  { [ -f "$user_helix_dir/helix.scm" ] && [ -f "$user_helix_dir/init.scm" ]; }; then
  helix_config_dir="$user_helix_dir"
  steel_config_dir="$YAZELIX_STATE_DIR/helix-steel"
  steel_config_dir_needs_mkdir=true
  if [ -f "$user_helix_dir/helix.scm" ] && [ -f "$user_helix_dir/init.scm" ]; then
    steel_config_dir="$user_helix_dir"
    steel_config_dir_needs_mkdir=false
  fi
fi
HELIX_STEEL_CONFIG="$steel_config_dir"
export HELIX_STEEL_CONFIG

if [ "${YAZELIX_HELIX_BRIDGE:-1}" != 0 ]; then
  if [ -z "${YAZELIX_HELIX_BRIDGE_SESSION_ID:-}" ]; then
    YAZELIX_HELIX_BRIDGE_SESSION_ID="yzx-helper-$(@date@ +%s)-$$"
  fi
  export YAZELIX_HELIX_BRIDGE_SESSION_ID

  export YAZELIX_HELIX_BRIDGE=1
  YAZELIX_HELIX_BRIDGE_INSTANCE_ID="hx-$(@date@ +%s)-$$"
  export YAZELIX_HELIX_BRIDGE_INSTANCE_ID
  YAZELIX_HELIX_BRIDGE_AUTH_TOKEN="$(@od@ -An -N32 -tx1 /dev/urandom | @tr@ -d ' \n')"
  export YAZELIX_HELIX_BRIDGE_AUTH_TOKEN
  YAZELIX_HELIX_MANAGED_CONFIG_PATH="$helix_config_file"
  export YAZELIX_HELIX_MANAGED_CONFIG_PATH
fi

@mkdir@ -p "$YAZELIX_STATE_DIR"
if ! @yzxConfig@ --write-effective-helix-config "$packaged_helix_config" "$user_helix_config" "$helix_config_file"; then
  exit 1
fi
if [ "$steel_config_dir_needs_mkdir" = true ]; then
  @mkdir@ -p "$steel_config_dir"
fi
exec @hx@ --config-dir "$helix_config_dir" -c "$helix_config_file" "$@"
