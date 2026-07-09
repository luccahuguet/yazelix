#!/bin/sh
set -eu

event_name=
base_sha=
head_sha=
output_file=

while [ "$#" -gt 0 ]; do
  case "$1" in
    --event)
      event_name="${2:-}"
      shift 2
      ;;
    --base)
      base_sha="${2:-}"
      shift 2
      ;;
    --head)
      head_sha="${2:-}"
      shift 2
      ;;
    --output)
      output_file="${2:-}"
      shift 2
      ;;
    *)
      printf 'unknown argument: %s\n' "$1" >&2
      exit 2
      ;;
  esac
done

if [ -z "$event_name" ] || [ -z "$head_sha" ] || [ -z "$output_file" ]; then
  printf 'usage: ci_changed_surfaces.sh --event <name> --base <sha> --head <sha> --output <path>\n' >&2
  exit 2
fi

changed_files="$(mktemp)"
trap 'rm -f "$changed_files"' EXIT

is_zero_sha() {
  case "$1" in
    "" | 0000000000000000000000000000000000000000) return 0 ;;
    *) return 1 ;;
  esac
}

commit_exists() {
  git cat-file -e "$1^{commit}" 2>/dev/null
}

if [ "$event_name" = "schedule" ] || [ "$event_name" = "workflow_dispatch" ]; then
  : > "$changed_files"
elif is_zero_sha "$base_sha"; then
  {
    printf '%s\n' '.github/workflows/ci.yml'
    printf '%s\n' '.github/workflows/publish_nix_cache.yml'
    printf '%s\n' 'flake.nix'
    printf '%s\n' 'flake.lock'
  } > "$changed_files"
elif ! is_zero_sha "$base_sha" && commit_exists "$base_sha" && commit_exists "$head_sha"; then
  git diff --name-only "$base_sha" "$head_sha" > "$changed_files"
elif commit_exists "$head_sha" && git rev-parse "$head_sha^" >/dev/null 2>&1; then
  git diff --name-only "$head_sha^" "$head_sha" > "$changed_files"
else
  {
    printf '%s\n' '.github/workflows/ci.yml'
    printf '%s\n' '.github/workflows/publish_nix_cache.yml'
    printf '%s\n' 'flake.nix'
    printf '%s\n' 'flake.lock'
  } > "$changed_files"
fi

nix_customization=false
child_release=false
darwin_wasm=false
cold_install=false

while IFS= read -r file; do
  case "$file" in
    .github/workflows/ci.yml | .github/actions/* | shells/posix/ci_changed_surfaces.sh | docs/contracts/test_suite_governance.md)
      nix_customization=true
      child_release=true
      darwin_wasm=true
      ;;
    .github/workflows/publish_nix_cache.yml)
      nix_customization=true
      ;;
    flake.nix | flake.lock | yazelix_runtime_package.nix)
      nix_customization=true
      child_release=true
      darwin_wasm=true
      cold_install=true
      ;;
    home_manager/* | home_manager/**/*)
      nix_customization=true
      ;;
    rust_core/yazelix_maintainer/src/repo_contract_validation.rs | rust_core/yazelix_maintainer/src/repo_contract_validation/nix_interface.rs | rust_core/yazelix_maintainer/src/repo_contract_validation/nix_package.rs)
      nix_customization=true
      ;;
    rust_core/yazelix_maintainer/src/repo_contract_validation/installed_runtime.rs)
      cold_install=true
      ;;
    rust_core/yazelix_maintainer/src/repo_child_release.rs)
      child_release=true
      darwin_wasm=true
      ;;
    rust_core/yazelix_maintainer/src/bin/yzx_repo_validator.rs)
      nix_customization=true
      child_release=true
      ;;
    rust_core/yazelix_core/src/runtime_* | rust_core/yazelix_core/src/runtime_*/* | rust_core/yazelix_core/src/zellij_materialization* | rust_core/yazelix_core/src/workspace_asset_contract.rs | rust_core/yazelix_core/src/bin/yzx_core.rs)
      cold_install=true
      ;;
    rust_core/yazelix_zellij_config_pack/* | rust_core/yazelix_zellij_config_pack/**/*)
      child_release=true
      darwin_wasm=true
      ;;
  esac
done < "$changed_files"

{
  printf 'nix_customization=%s\n' "$nix_customization"
  printf 'child_release=%s\n' "$child_release"
  printf 'darwin_wasm=%s\n' "$darwin_wasm"
  printf 'cold_install=%s\n' "$cold_install"
} >> "$output_file"

printf 'Changed CI surfaces:\n'
printf '  nix_customization=%s\n' "$nix_customization"
printf '  child_release=%s\n' "$child_release"
printf '  darwin_wasm=%s\n' "$darwin_wasm"
printf '  cold_install=%s\n' "$cold_install"
