"""Read-only discovery for Nova-owned dependency pins; Python standard library only."""

import html
import json
import os
import re
import subprocess
import sys
import urllib.parse
import urllib.request
from pathlib import Path


# Comparison targets omitted by commit-pinned inputs. These are not version pins.
BRANCHES = {
    "Yazelix/auto-layout.yazi": "candidate/yazi-v26.8.15",
    "Yazelix/nova-rio": "edge",
    "Yazelix/nova-zellij": "candidate/v0.45.0",
    "Yazelix/nova-helix": "main",
    "Yazelix/zj-radar": "nova-v0.6",
    "luccahuguet/yazelix-forest": "main",
    "chuwy/notify.hx": "main",
    "Ra77a3l3-jar/glyph.hx": "main",
    "Rolv-Apneseth/starship.yazi": "main",
    "yazi-rs/plugins": "main",
    "yazi-rs/schemas": "main",
}
YAZI = {"autoLayoutYazi", "starshipYazi", "gitYazi", "yaziBistro", "yaziSchemas", "Yazi / ya"}


def get_json(url):
    headers = {"User-Agent": "Nova dependency report (https://github.com/Yazelix/nova)"}
    if urllib.parse.urlparse(url).netloc == "api.github.com":
        headers["Accept"] = "application/vnd.github+json"
        if os.environ.get("GH_TOKEN"):
            headers["Authorization"] = "Bearer " + os.environ["GH_TOKEN"]
    with urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=8) as response:
        data = response.read(4 * 1024 * 1024 + 1)
    if len(data) > 4 * 1024 * 1024:
        raise ValueError("API response exceeds 4 MiB")
    return json.loads(data)


def one(pattern, text):
    matches = re.findall(pattern, text, re.MULTILINE | re.DOTALL)
    if len(matches) != 1:
        raise ValueError(f"Expected one literal declaration matching {pattern}")
    return matches[0]


def literal(name, text):
    return one(r'^\s*' + name + r'\s*=\s*"([^"\n]+)"\s*;', text)


def inventory(root):
    lock = json.loads((root / "flake.lock").read_text())
    items = []
    for name, key in lock["nodes"][lock["root"]]["inputs"].items():
        node = lock["nodes"][key]
        locked, original = node["locked"], node["original"]
        if locked["type"] != "github":
            raise ValueError(f"{name}: unsupported input type {locked['type']}")
        repo = locked["owner"] + "/" + locked["repo"]
        target = original.get("ref") or BRANCHES.get(repo)
        if not target and "rev" not in original:
            target = "@default"  # The input URL itself selects the default branch.
        items.append((name, "branch", repo, locked["rev"], target))

    # ponytail: two owned literal package declarations; use Nix evaluation if they become computed.
    yazi = one(r'yazi-unwrapped = prev\.yazi-unwrapped\.overrideAttrs '
               r'\(finalAttrs: previousAttrs: \{(.*?)^\s*\}\);', (root / "flake.nix").read_text())
    if literal("tag", yazi) != "v${finalAttrs.version}":
        raise ValueError("Yazi source no longer follows its literal release version")
    items.append(("Yazi / ya", "release", literal("owner", yazi) + "/" + literal("repo", yazi),
                  "v" + literal("version", yazi), "latest full release"))
    crate = (root / "packaging/tokenusage.nix").read_text()
    one(r'src = pkgs\.fetchCrate \{(.*?)\};', crate)
    items.append((literal("pname", crate), "crate", "crates.io", literal("version", crate),
                  "latest stable crate"))
    return sorted(items, key=lambda item: (item[0] not in YAZI, item[0].lower()))


def version(value):
    if not re.fullmatch(r"v?\d+\.\d+\.\d+", value):
        raise ValueError(f"Review required: unsupported release version {value}")
    return tuple(map(int, value.removeprefix("v").split(".")))


def compare(item, api):
    name, kind, repo, current, target = item
    quote = lambda value: urllib.parse.quote(value, safe="")
    if kind == "crate":
        available = api(f"https://crates.io/api/v1/crates/{quote(name)}")["crate"]["max_stable_version"]
        link = f"https://crates.io/crates/{quote(name)}/{quote(available)}"
    else:
        if not re.fullmatch(r"[\w.-]+/[\w.-]+", repo):
            raise ValueError("Invalid GitHub repository")
        base = f"https://api.github.com/repos/{repo}"
        if kind == "branch":
            if not target:
                raise ValueError("Review required: no comparison policy for pinned input")
            if target == "@default":
                target = api(base)["default_branch"]
            available = api(base + "/commits/" + quote(target))["sha"]
            if not all(re.fullmatch(r"[0-9a-f]{40}", sha) for sha in [current, available]):
                raise ValueError("Invalid commit revision")
            link = f"https://github.com/{repo}/compare/{current}...{available}"
            if current == available:
                result = "Matches branch"
            else:
                diff = api(f"{base}/compare/{current}...{available}?per_page=1")
                result = {
                    "identical": "Matches branch",
                    "ahead": f"Branch ahead (+{diff['ahead_by']})",
                    "behind": "Pin ahead of branch; review",
                    "diverged": "Diverged; review",
                }[diff["status"]]
            return target, available, result, link
        release = api(base + "/releases/latest")
        if release["draft"] is not False or release["prerelease"] is not False:
            raise ValueError("API did not return a published full release")
        available = release["tag_name"]
        link = f"https://github.com/{repo}/releases/tag/{quote(available)}"
    old, new = version(current), version(available)
    result = "Release available" if new > old else "Matches release"
    if new < old:
        result = "Pin newer than release; review"
    return target, available, result, link


def cell(value):
    text = html.escape(str(value)).replace("\n", " ").replace("\r", " ")
    for char in "|`[]*_\\":
        text = text.replace(char, f"&#{ord(char)};")
    return text


def report(root, revision, output, api=get_json):
    print(f"# Dependency updates\n\nEdge inventory: `{revision}`.\n", file=output)
    print("Available changes require maintainer review; this is not a compatibility or security verdict.\n",
          file=output)
    try:
        items = inventory(root)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"**INCOMPLETE inventory:** {cell(error)}", file=output)
        return 1
    print("| Group | Dependency | Current | Comparison target | Available | Finding |", file=output)
    print("| --- | --- | --- | --- | --- | --- |", file=output)
    failures = 0
    for item in items:
        name, kind, repo, current, target = item
        group = "Yazi / ya" if name in YAZI else "Other"
        try:
            target, available, result, link = compare(item, api)
            short = available[:12] if kind == "branch" else available
            available_cell = f"[{cell(short)}]({link})"
        except (OSError, ValueError, KeyError, TypeError) as error:
            failures += 1
            available_cell, result = "Unknown", f"INCOMPLETE: {error}"
        short_current = current[:12] if kind == "branch" else current
        print(f"| {group} | {cell(name)} | {cell(short_current)} | {cell(repo)}: {cell(target or 'unknown')} "
              f"| {available_cell} | {cell(result)} |", file=output, flush=True)
    print(f"\n**{'INCOMPLETE' if failures else 'Complete'}:** {len(items)} dependencies, "
          f"{failures} unavailable comparisons. Nixpkgs tools and child internals stay with their owners.",
          file=output)
    return int(failures > 0)


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[2]
    subprocess.run(["git", "diff", "--exit-code", "HEAD", "--", "flake.nix", "flake.lock",
                    "packaging/tokenusage.nix"], cwd=root, check=True)
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    sys.exit(report(root, revision, sys.stdout))
