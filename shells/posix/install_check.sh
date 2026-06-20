#!/usr/bin/env sh
set -eu

cache_url="https://yazelix.cachix.org"
cache_key="yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mpUNzDJ97xC2y7QWYjmGNM="
install_target="${YAZELIX_INSTALL_TARGET:-github:luccahuguet/yazelix#yazelix}"
status=0

say() {
  printf '%s\n' "$*"
}

ok() {
  say "ok   $*"
}

warn() {
  say "warn $*"
}

fail() {
  say "fail $*"
  status=1
}

info() {
  say "info $*"
}

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

contains() {
  printf '%s' "$1" | grep -F "$2" >/dev/null 2>&1
}

detect_nix_mode() {
  nix_version=$1

  if contains "$nix_version" "Determinate"; then
    say "determinate"
  elif [ -e /etc/NIXOS ]; then
    say "nixos"
  elif [ "$(uname -s 2>/dev/null || say unknown)" = "Darwin" ]; then
    say "darwin"
  elif [ -S /nix/var/nix/daemon-socket/socket ]; then
    say "daemon"
  else
    say "single-user-or-unknown"
  fi
}

cache_guidance() {
  nix_mode=$1

  say ""
  say "Optional cache setup:"
  case "$nix_mode" in
    determinate)
      say "  Determinate Nix: append this to /etc/nix/nix.custom.conf, then restart the Nix daemon or reboot:"
      say ""
      say "    extra-substituters = $cache_url"
      say "    extra-trusted-public-keys = $cache_key"
      ;;
    nixos)
      say "  NixOS or Home Manager-managed Nix: add these settings to your Nix configuration:"
      say ""
      say "    nix.settings.extra-substituters = [ \"$cache_url\" ];"
      say "    nix.settings.extra-trusted-public-keys = [ \"$cache_key\" ];"
      ;;
    daemon | darwin)
      say "  Ordinary daemon Nix: the Cachix helper can write root Nix config when nix-env is available:"
      say ""
      say "    sudo nix run nixpkgs#cachix -- use yazelix --mode root-nixconf"
      say ""
      say "  If that helper fails, add these entries to the daemon Nix config instead:"
      say ""
      say "    extra-substituters = $cache_url"
      say "    extra-trusted-public-keys = $cache_key"
      ;;
    *)
      say "  Add these entries to the Nix config used by this installation:"
      say ""
      say "    extra-substituters = $cache_url"
      say "    extra-trusted-public-keys = $cache_key"
      ;;
  esac
}

say "Yazelix install check"
say ""
say "This command is read-only. It does not run sudo or edit Nix configuration."
say ""

if ! has_cmd nix; then
  fail "nix was not found on PATH"
  say ""
  say "Next steps:"
  say ""
  say "  1. Install Nix:"
  say "     curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install"
  say ""
  say "  2. Open a new terminal or shell after the installer finishes."
  say ""
  say "  3. Rerun the Yazelix install check:"
  say "     curl -fsSL https://raw.githubusercontent.com/luccahuguet/yazelix/main/shells/posix/install_check.sh | sh"
  exit "$status"
fi

nix_version="$(nix --version 2>/dev/null || true)"
if [ -n "$nix_version" ]; then
  ok "Nix is available: $nix_version"
else
  fail "nix exists, but 'nix --version' failed"
fi

nix_mode="$(detect_nix_mode "$nix_version")"
info "Detected Nix mode: $nix_mode"

if nix flake --help >/dev/null 2>&1; then
  ok "Nix flakes command is available"
else
  fail "Nix flakes command is not available"
  info "Yazelix installs through Nix flakes. Install or repair a modern Nix distribution such as Determinate Nix, or enable nix-command and flakes in your Nix config."
fi

current_system="$(nix eval --raw --impure --expr 'builtins.currentSystem' 2>/dev/null || true)"
case "$current_system" in
  x86_64-linux | aarch64-linux | x86_64-darwin | aarch64-darwin)
    ok "Current system is exported by the Yazelix flake: $current_system"
    ;;
  "")
    fail "Could not evaluate builtins.currentSystem"
    ;;
  *)
    fail "Current system is not exported by the Yazelix flake: $current_system"
    ;;
esac

nix_config="$(nix config show 2>/dev/null || true)"
if contains "$nix_config" "$cache_url" && contains "$nix_config" "$cache_key"; then
  ok "Yazelix Cachix cache is present in active Nix config"
else
  warn "Yazelix Cachix cache is not present in active Nix config"
  info "This is a speed issue, not an install blocker. Nix may build Yazelix locally."
  info "Warnings about an untrusted Yazelix substituter usually mean the daemon ignored flake-provided cache settings."
  cache_guidance "$nix_mode"
fi

say ""
if [ "$status" -eq 0 ]; then
  say "Result: install prerequisites look usable."
  say ""
  say "Next steps:"
  say ""
  say "  1. Install Yazelix:"
  say "     nix profile add --refresh --accept-flake-config $install_target"
  say ""
  say "  2. Launch Yazelix:"
  say "     yzx launch"
else
  say "Result: fix the failed checks above before installing Yazelix."
fi

exit "$status"
