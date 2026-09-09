import io
import json
import tempfile
import unittest
from pathlib import Path

import dependency_report as report


class DependencyReportContract(unittest.TestCase):
    def test_report_preserves_pins_and_distinguishes_updates_from_unknowns(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "packaging").mkdir()
            sources = {
                "flake.nix": '''yazi-unwrapped = prev.yazi-unwrapped.overrideAttrs (finalAttrs: previousAttrs: {
                  version = "26.8.15";
                  owner = "sxyazi";
                  repo = "yazi";
                  tag = "v${finalAttrs.version}";
                });''',
                "packaging/tokenusage.nix": '''pname = "tokenusage";
                version = "1.5.2";
                src = pkgs.fetchCrate { inherit pname version; };''',
            }
            nodes = {"root": {"inputs": {}}}
            for name, repo, original in [
                ("zjRadar", "Yazelix/zj-radar", {"rev": "a" * 40}),
                ("nixpkgs", "NixOS/nixpkgs", {"ref": "nixos-unstable"}),
                ("branchBehind", "example/behind", {"ref": "work"}),
                ("declaredDefault", "example/default", {}),
                ("unknown", "example/unknown", {"rev": "a" * 40}),
                ("unavailable", "example/unavailable", {"ref": "main"}),
            ]:
                owner, repo_name = repo.split("/")
                nodes["root"]["inputs"][name] = name
                nodes[name] = {"original": original, "locked": {
                    "type": "github", "owner": owner, "repo": repo_name, "rev": "a" * 40,
                }}
            # Child internals do not become Nova's update inventory.
            nodes["transitive"] = {"locked": {"type": "path"}}
            sources["flake.lock"] = json.dumps({"root": "root", "nodes": nodes})
            for name, content in sources.items():
                (root / name).write_text(content)

            calls = []

            def api(url):
                calls.append(url)
                if "unavailable" in url:
                    raise OSError("HTTP 429: rate limited")
                if url.endswith("releases/latest"):
                    return {"tag_name": "v26.9.1", "draft": False, "prerelease": False}
                if url.endswith("crates/tokenusage"):
                    return {"crate": {"max_stable_version": "1.5.2"}}
                if url.endswith("repos/example/default"):
                    return {"default_branch": "trunk"}
                if "/commits/" in url:
                    return {"sha": "b" * 40}
                if "/compare/" in url:
                    status = "ahead" if "zj-radar" in url else "diverged"
                    if "example/behind" in url:
                        status = "behind"
                    return {"status": status, "ahead_by": 2, "behind_by": 1}
                self.fail(f"Unexpected request: {url}")

            output = io.StringIO()
            self.assertEqual(report.report(root, "c" * 40, output, api), 1)
            text = output.getvalue()
            for expected in ["c" * 40, "v26.9.1", "Release available", "Matches release",
                             "Branch ahead", "Diverged", "Pin ahead", "no comparison policy",
                             "HTTP 429", "INCOMPLETE", "Yazi / ya"]:
                self.assertIn(expected, text)
            self.assertTrue(any("commits/nova-v0.6" in call for call in calls))
            self.assertTrue(any("commits/trunk" in call for call in calls))
            self.assertFalse(any("unknown" in call or "transitive" in call for call in calls))
            self.assertIn("/compare/" + "a" * 40 + "..." + "b" * 40, text)
            self.assertNotIn("transitive", text)
            for name, content in sources.items():
                self.assertEqual((root / name).read_text(), content)

            # Incomplete rows can recover without treating available updates as failures.
            del nodes["root"]["inputs"]["unknown"]
            del nodes["root"]["inputs"]["unavailable"]
            (root / "flake.lock").write_text(json.dumps({"root": "root", "nodes": nodes}))
            output = io.StringIO()
            self.assertEqual(report.report(root, "c" * 40, output, api), 0)
            self.assertNotIn("INCOMPLETE", output.getvalue())

            # A changed literal version is read from its owner, including newer-than-release pins.
            (root / "flake.nix").write_text(sources["flake.nix"].replace('"26.8.15"', '"26.10.1"'))
            output = io.StringIO()
            self.assertEqual(report.report(root, "c" * 40, output, api), 0)
            self.assertIn("v26.10.1", output.getvalue())
            self.assertIn("Pin newer than release", output.getvalue())

            def prerelease(url):
                data = api(url)
                if url.endswith("releases/latest"):
                    data["prerelease"] = True
                return data

            self.assertEqual(report.report(root, "c" * 40, io.StringIO(), prerelease), 1)

            # Unsupported source shapes and malformed API data must not claim current.
            (root / "packaging/tokenusage.nix").write_text(sources["packaging/tokenusage.nix"].replace(
                'version = "1.5.2";', 'version = computedVersion;'))
            output = io.StringIO()
            self.assertEqual(report.report(root, "c" * 40, output, api), 1)
            self.assertIn("INCOMPLETE", output.getvalue())
            (root / "packaging/tokenusage.nix").write_text(sources["packaging/tokenusage.nix"])
            self.assertEqual(report.report(root, "c" * 40, io.StringIO(), lambda _: {}), 1)


if __name__ == "__main__":
    unittest.main()
