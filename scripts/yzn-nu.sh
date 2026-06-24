# shellcheck shell=bash

: "${HOME:?HOME is required}"
: "${YZN_PACKAGED_NU:?YZN_PACKAGED_NU is required}"

nu_quote() {
  local quoted="$1"
  quoted="${quoted//\\/\\\\}"
  quoted="${quoted//\"/\\\"}"
  printf '"%s"' "$quoted"
}

config_home="${YAZELIX_NEXT_CONFIG_HOME:-${XDG_CONFIG_HOME:-$HOME/.config}/yazelix-next}"
user_nu="$config_home/nu"
runtime_nu="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}}/yazelix-next/nu"
mkdir -p "$runtime_nu"

env_config="$runtime_nu/env.nu"
config="$runtime_nu/config.nu"
env_tmp="$(mktemp "$runtime_nu/env.nu.XXXXXX")"
config_tmp="$(mktemp "$runtime_nu/config.nu.XXXXXX")"

printf 'source-env %s\n' "$(nu_quote "$YZN_PACKAGED_NU/env.nu")" > "$env_tmp"
if [ -f "$user_nu/env.nu" ]; then
  printf 'source-env %s\n' "$(nu_quote "$user_nu/env.nu")" >> "$env_tmp"
fi

printf 'source %s\n' "$(nu_quote "$YZN_PACKAGED_NU/config.nu")" > "$config_tmp"
if [ -f "$user_nu/config.nu" ]; then
  printf 'source %s\n' "$(nu_quote "$user_nu/config.nu")" >> "$config_tmp"
fi

mv "$env_tmp" "$env_config"
mv "$config_tmp" "$config"

exec nu --env-config "$env_config" --config "$config" "$@"
