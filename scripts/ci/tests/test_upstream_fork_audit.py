"""Regression coverage for the resumable upstream fork census."""
import pathlib
import sys
import tempfile
import unittest
import json

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import upstream_fork_audit as audit


class FakeApi:
    def __init__(self, pages):
        self.pages = pages
        self.calls = []

    def get(self, path):
        self.calls.append(path)
        if path not in self.pages:
            raise AssertionError(f"unexpected request {path}")
        return self.pages[path]


class ForkAuditTests(unittest.TestCase):
    def test_walks_descendants_and_paginates_without_duplicates(self):
        api = FakeApi({
            "/repos/acme/openhuman/forks?per_page=100&page=1&sort=oldest": [
                {"id": 1, "full_name": "alice/openhuman", "default_branch": "dev", "forks_count": 1},
                {"id": 2, "full_name": "bob/openhuman", "default_branch": "main", "forks_count": 0},
            ],
            "/repos/alice/openhuman/forks?per_page=100&page=1&sort=oldest": [
                {"id": 2, "full_name": "bob/openhuman", "default_branch": "main", "forks_count": 0},
                {"id": 3, "full_name": "carol/openhuman", "default_branch": "main", "forks_count": 0},
            ]
        })
        result = audit.enumerate_forks(api, "acme/openhuman")
        self.assertEqual(set(result), {"alice/openhuman", "bob/openhuman", "carol/openhuman"})
        self.assertEqual(result["alice/openhuman"]["default_branch"], "dev")

    def test_ahead_behind_and_resume_are_measured_not_guessed(self):
        api = FakeApi({
            "/repos/acme/openhuman/compare/deadbeef...alice:dev": {
                "ahead_by": 2, "behind_by": 7, "status": "diverged",
                "commits": [{"sha": "123", "commit": {"message": "Fix race\n\nBody"}}]
            },
        })
        existing = {"bob/openhuman": {"full_name": "bob/openhuman", "ahead_by": 0}}
        rows = [
            {"full_name": "alice/openhuman", "default_branch": "dev"},
            {"full_name": "bob/openhuman", "default_branch": "main"},
        ]
        results = audit.compare_forks(api, "acme/openhuman", "deadbeef", rows, existing)
        self.assertEqual(results["alice/openhuman"]["ahead_by"], 2)
        self.assertEqual(results["alice/openhuman"]["recent_unique_commits"][0]["message"], "Fix race")
        self.assertEqual(results["bob/openhuman"]["ahead_by"], 0)
        self.assertEqual(len(api.calls), 1)

    def test_report_never_marks_incomplete_fork_census_complete(self):
        rows = {"alice/openhuman": {"full_name": "alice/openhuman", "ahead_by": 1, "behind_by": 2}}
        report = audit.markdown_report("acme/openhuman", "deadbeef", 3, rows, {"bob/openhuman": "404"})
        self.assertIn("INCOMPLETE", report)
        self.assertIn("alice/openhuman", report)
        self.assertIn("bob/openhuman", report)

    def test_api_accepts_compare_three_dot_syntax_and_rejects_path_traversal(self):
        from io import BytesIO
        from unittest.mock import patch
        api = audit.GitHubApi("test-token", 2)
        with patch.object(audit, "urlopen", return_value=BytesIO(b'{"ahead_by":1}')):
            result = api.get("/repos/acme/openhuman/compare/deadbeef...alice:main")
        self.assertEqual(result["ahead_by"], 1)
        with self.assertRaises(ValueError):
            api.get("/repos/acme/openhuman/../../secrets")

    def test_atomic_checkpoint_roundtrip(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = pathlib.Path(tmp) / "state.json"
            state = {"upstream": "acme/openhuman", "sha": "deadbeef"}
            audit.write_checkpoint(target, state)
            self.assertEqual(json.loads(target.read_text()), state)
            self.assertFalse((target.parent / "state.json.tmp").exists())


if __name__ == "__main__":
    unittest.main()
