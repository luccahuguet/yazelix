#!/bin/sh
shell_program="$(@yznConfig@ --get shell.program)"

case "$shell_program" in
  nu) exec @yznNu@ "$@" ;;
  bash) exec @bash@ -i "$@" ;;
  zsh) exec @zsh@ -i "$@" ;;
  fish) exec @fish@ -i "$@" ;;
esac

printf '%s\n' "Unsupported shell.program: $shell_program" >&2
exit 64
