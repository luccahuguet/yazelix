#!/bin/sh
set -eu

usage() {
  cat <<'USAGE'
Usage:
  yazelix_runtime_size_report.sh [--top N] [--direct-top N] OUTPUT_PATH
  yazelix_runtime_size_report.sh --build FLAKE_ATTR [--top N] [--direct-top N]

Measures a realized Nix output without building by default.
Use --build only when the command should first realize a flake attribute.
USAGE
}

top_count=25
direct_top_count=40
build_target=""
target=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    --top)
      shift
      [ "$#" -gt 0 ] || { echo "Missing value after --top" >&2; exit 2; }
      top_count="$1"
      ;;
    --direct-top)
      shift
      [ "$#" -gt 0 ] || { echo "Missing value after --direct-top" >&2; exit 2; }
      direct_top_count="$1"
      ;;
    --build)
      shift
      [ "$#" -gt 0 ] || { echo "Missing value after --build" >&2; exit 2; }
      build_target="$1"
      ;;
    -*)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
    *)
      if [ -n "$target" ]; then
        echo "Expected one output path, got extra argument: $1" >&2
        exit 2
      fi
      target="$1"
      ;;
  esac
  shift
done

case "$top_count" in
  ''|*[!0-9]*) echo "--top must be a positive integer" >&2; exit 2 ;;
esac
[ "$top_count" -gt 0 ] || { echo "--top must be a positive integer" >&2; exit 2; }
case "$direct_top_count" in
  ''|*[!0-9]*) echo "--direct-top must be a positive integer" >&2; exit 2 ;;
esac
[ "$direct_top_count" -gt 0 ] || { echo "--direct-top must be a positive integer" >&2; exit 2; }

if [ -n "$build_target" ] && [ -n "$target" ]; then
  echo "Pass either --build FLAKE_ATTR or OUTPUT_PATH, not both" >&2
  exit 2
fi

if [ -n "$build_target" ]; then
  target="$(nix build --no-link --print-out-paths --no-write-lock-file "$build_target")"
  case "$target" in
    *'
'*) echo "--build produced multiple output paths; pass one output path explicitly" >&2; exit 2 ;;
  esac
fi

if [ -z "$target" ]; then
  usage >&2
  exit 2
fi

for command_name in awk head jq mktemp nix nix-store readlink sed sort tr wc; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 1
  fi
done

human_bytes() {
  bytes="$1"
  if command -v numfmt >/dev/null 2>&1; then
    numfmt --to=iec --suffix=B "$bytes"
  else
    printf '%sB\n' "$bytes"
  fi
}

store_path="$(readlink -f "$target" 2>/dev/null || printf '%s\n' "$target")"
case "$store_path" in
  /nix/store/*/*)
    store_path="$(printf '%s\n' "$store_path" | sed 's#^\(/nix/store/[^/]*\)/.*#\1#')"
    ;;
esac

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT HUP INT TERM

closure_json="$tmp_dir/closure.json"
direct_refs="$tmp_dir/direct_refs"
direct_sizes_json="$tmp_dir/direct_sizes.json"

nix path-info -r --json-format 1 --json "$store_path" > "$closure_json"
nix-store -q --references "$store_path" | awk -v self="$store_path" '$0 != self' | sort > "$direct_refs"

summary_json="$(nix path-info -S --json-format 1 --json "$store_path")"
nar_size="$(printf '%s\n' "$summary_json" | jq -r 'to_entries[0].value.narSize')"
closure_size="$(printf '%s\n' "$summary_json" | jq -r 'to_entries[0].value.closureSize')"
closure_paths="$(jq 'length' "$closure_json")"
direct_count="$(wc -l < "$direct_refs" | tr -d ' ')"

cat <<REPORT
Yazelix Runtime Size Report

Output: $store_path
NAR size: $(human_bytes "$nar_size") ($nar_size bytes)
Closure size: $(human_bytes "$closure_size") ($closure_size bytes)
Closure paths: $closure_paths
Direct references: $direct_count

Largest NAR Paths
REPORT

jq -r 'to_entries[] | [.value.narSize, .key] | @tsv' "$closure_json" \
  | sort -nr \
  | head -n "$top_count" \
  | while IFS="$(printf '\t')" read -r bytes path; do
      printf '  %10s  %s\n' "$(human_bytes "$bytes")" "$path"
    done

if [ "$direct_count" -gt 0 ]; then
  # shellcheck disable=SC2046
  nix path-info -S --json-format 1 --json $(cat "$direct_refs") > "$direct_sizes_json"

  cat <<REPORT

Direct References By Closure Size
REPORT

  jq -r 'to_entries[] | [.value.closureSize, .value.narSize, .key] | @tsv' "$direct_sizes_json" \
    | sort -nr \
    | head -n "$direct_top_count" \
    | while IFS="$(printf '\t')" read -r closure_bytes direct_nar direct_path; do
        printf '  %10s closure  %10s nar  %s\n' \
          "$(human_bytes "$closure_bytes")" \
          "$(human_bytes "$direct_nar")" \
          "$direct_path"
      done
fi

cat <<REPORT

Duplicate Store Basenames By NAR Size
REPORT

jq -r 'to_entries[] | [(.key | sub("^/nix/store/[a-z0-9]+-"; "")), .value.narSize] | @tsv' "$closure_json" \
  | awk '
      {
        family = $1
        bytes = $2
        count[family] += 1
        total[family] += bytes
      }
      END {
        for (family in count) {
          if (count[family] > 1) {
            printf "%d\t%d\t%s\n", total[family], count[family], family
          }
        }
      }
    ' \
  | sort -nr \
  | head -n "$top_count" \
  | while IFS="$(printf '\t')" read -r bytes count family; do
      printf '  %10s  x%-3s %s\n' "$(human_bytes "$bytes")" "$count" "$family"
    done
