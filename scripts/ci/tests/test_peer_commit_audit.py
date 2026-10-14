"""Small offline Git DAG regression tests for the full peer commit audit."""
import importlib.util
import json
import pathlib
import subprocess
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parents[1] / "audit-peer-commits.py"


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


def write_commit(repo, name, body, message):
    (repo / name).parent.mkdir(parents=True, exist_ok=True)
    (repo / name).write_text(body)
    git(repo, "add", name)
    git(repo, "commit", "-qm", message)
    return git(repo, "rev-parse", "HEAD")


class PeerAuditTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)
        self.source = self.root / "source"
        self.target = self.root / "target"
        for repo in (self.source, self.target):
            repo.mkdir()
            git(repo, "init", "-q")
            git(repo, "config", "user.email", "test@example.com")
            git(repo, "config", "user.name", "Test")
        write_commit(self.source, "README.md", "common\n", "base")
        # Same content as source base, but a different unrelated commit history.
        write_commit(self.target, "README.md", "common\n", "other independent base")

    def tearDown(self):
        self.tmp.cleanup()

    def run_audit(self):
        report = self.root / "out.json"
        cmd = ["python3", str(SCRIPT), "--source", str(self.source),
               "--target", str(self.target), "--output", str(report),
               "--summary", str(self.root / "out.md")]
        run = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(run.returncode, 0, run.stderr)
        return json.loads(report.read_text()), (self.root / "out.md").read_text()

    def test_scans_all_branches_even_with_unrelated_git_history(self):
        write_commit(self.source, "crates/core/src/security.rs", "fn gate() {}\n", "fix(security): gate")
        git(self.source, "checkout", "-qb", "feature/other")
        write_commit(self.source, "app/src/settings.ts", "export const a = 1;\n", "feat(ui): add widget")
        result, _ = self.run_audit()
        self.assertEqual(result["source_commits"], 3)
        self.assertIn("feature/other", result["source_branches"])
        self.assertEqual(len(result["commits"]), 3)
        self.assertIn("security", {d for c in result["commits"] for d in c["domains"]})

    def test_identifies_patch_equivalence_across_different_commit_ids(self):
        write_commit(self.source, "new.txt", "shared patch\n", "source change")
        write_commit(self.target, "new.txt", "shared patch\n", "same patch, separate SHA")
        result, _ = self.run_audit()
        source_change = next(c for c in result["commits"] if c["subject"] == "source change")
        self.assertEqual(source_change["status"], "equivalent-patch")

    def test_rejects_shallow_clone_instead_of_claiming_exhaustive_history(self):
        write_commit(self.source, "extra.txt", "late\n", "second commit")
        shallow = self.root / "shallow"
        subprocess.check_call(["git", "clone", "-q", "--depth", "1",
                               self.source.as_uri(), str(shallow)])
        report = self.root / "shallow-report.json"
        proc = subprocess.run(["python3", str(SCRIPT), "--source", str(shallow),
                               "--target", str(self.target), "--output", str(report),
                               "--summary", str(self.root / "shallow-report.md")],
                              capture_output=True, text=True)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("shallow", proc.stderr.lower())
        self.assertFalse(report.exists())

    def test_keeps_unique_small_changes_and_reverted_code_in_exhaustive_report(self):
        write_commit(self.source, "tools/feature.py", "secret = 1\n", "feat: first")
        git(self.source, "revert", "--no-edit", "HEAD")
        result, _ = self.run_audit()
        subjects = [x["subject"] for x in result["commits"]]
        self.assertTrue(any("Revert" in item for item in subjects))
        self.assertTrue(any(x["status"] == "candidate" for x in result["commits"]))
        self.assertTrue(any(x["status"] == "not-current-in-source" for x in result["commits"]))


if __name__ == "__main__":
    unittest.main()
