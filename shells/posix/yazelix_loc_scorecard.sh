#!/usr/bin/env sh
set -eu

usage() {
  cat <<'USAGE'
Usage: yazelix_loc_scorecard.sh [BASE_REF] [HEAD_REF]

Print a delete-first/extraction LOC scorecard for the current Yazelix repo.
Defaults: BASE_REF=v16.3, HEAD_REF=HEAD
USAGE
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

base_ref="${1:-v16.3}"
head_ref="${2:-HEAD}"

require() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Missing required command: %s\n' "$1" >&2
    exit 1
  fi
}

require git
require awk
require mktemp
require tar
require tokei
require jq

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

git rev-parse --verify "$base_ref^{commit}" >/dev/null
git rev-parse --verify "$head_ref^{commit}" >/dev/null

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

mkdir -p "$tmp_dir/base" "$tmp_dir/head"
git archive "$base_ref" | tar -x -C "$tmp_dir/base"
git archive "$head_ref" | tar -x -C "$tmp_dir/head"

tokei --exclude .beads --exclude target --exclude '*.wasm' --output json "$tmp_dir/base" > "$tmp_dir/base.json"
tokei --exclude .beads --exclude target --exclude '*.wasm' --output json "$tmp_dir/head" > "$tmp_dir/head.json"

printf 'LOC extraction scorecard\n'
printf 'Baseline: %s\n' "$base_ref"
printf 'Candidate: %s\n\n' "$head_ref"

printf 'Raw diff excluding .beads:\n'
git diff --shortstat "$base_ref..$head_ref" -- . ':(exclude).beads/*' |
  sed 's/^/  /'

git diff --numstat "$base_ref..$head_ref" -- . ':(exclude).beads/*' |
  awk '
    function category(path) {
      if (path ~ /^README.md$/ || path ~ /^CHANGELOG.md$/ || path ~ /^docs\//) return "docs";
      if (path ~ /(^|\/)tests\// || path ~ /(^|\/)test_/ || path ~ /_test\./ || path ~ /fixtures/) return "tests";
      if (path ~ /^rust_core\/yazelix_maintainer\// || path ~ /^maintainer_shell\.nix$/ || path ~ /^\.github\//) return "maintainer";
      if (path ~ /^config_metadata\// || path ~ /example_generated_config/ || path ~ /generated/) return "generated";
      if (path ~ /^flake\.nix$/ || path ~ /^flake\.lock$/ || path ~ /^packaging\// || path ~ /^home_manager\// || path ~ /\.nix$/) return "packaging";
      if (path ~ /^assets\// || path ~ /\.wasm$/ || path ~ /shaders/) return "assets";
      if (path ~ /^rust_core\/yazelix_core\// || path ~ /^rust_plugins\// || path ~ /^nushell\// || path ~ /^configs\// || path ~ /^user_configs\// || path ~ /^shells\// || path ~ /^yazelix_.*\.toml$/) return "runtime";
      return "other";
    }
    BEGIN {
      order[1] = "runtime";
      order[2] = "maintainer";
      order[3] = "tests";
      order[4] = "docs";
      order[5] = "generated";
      order[6] = "packaging";
      order[7] = "assets";
      order[8] = "other";
    }
    $1 == "-" || $2 == "-" {
      binary[category($3)] += 1;
      next;
    }
    {
      cat = category($3);
      add[cat] += $1;
      del[cat] += $2;
    }
    END {
      printf "\nCategory diff excluding .beads:\n";
      for (i = 1; i <= 8; i++) {
        cat = order[i];
        net = add[cat] - del[cat];
        printf "  %-11s insertions=%6d deletions=%6d net=%7d", cat ":", add[cat], del[cat], net;
        if (binary[cat] > 0) {
          printf " binary_files=%d", binary[cat];
        }
        printf "\n";
      }
    }
  '

printf '\nTokei code LOC excluding .beads, target, and wasm:\n'
jq -nr --slurpfile base "$tmp_dir/base.json" --slurpfile head "$tmp_dir/head.json" '
  "  baseline_code=\($base[0].Total.code) baseline_total=\($base[0].Total.code + $base[0].Total.comments + $base[0].Total.blanks)\n" +
  "  candidate_code=\($head[0].Total.code) candidate_total=\($head[0].Total.code + $head[0].Total.comments + $head[0].Total.blanks)\n" +
  "  delta_code=\($head[0].Total.code - $base[0].Total.code) delta_total=\(($head[0].Total.code + $head[0].Total.comments + $head[0].Total.blanks) - ($base[0].Total.code + $base[0].Total.comments + $base[0].Total.blanks))"
'
