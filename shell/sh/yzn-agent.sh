#!/bin/sh
if ! command -v codex >/dev/null 2>&1; then
  printf '%s\n' "Yazelix Next agent popup

codex is not available on PATH.
Install Codex or make \`codex\` executable on PATH before using Alt Shift L." >&2
  if [ -t 0 ]; then
    printf '\nPress Enter to close this popup...' >&2
    read -r _ || true
  fi
  exit 127
fi

exec codex resume "$@"
