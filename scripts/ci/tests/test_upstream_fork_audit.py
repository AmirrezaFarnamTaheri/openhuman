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

    def test_api_recovers_from_a_remote_disconnect_without_abandoning_progress(self):
        from http.client import RemoteDisconnected
        from io import BytesIO
        from unittest.mock import patch
        api = audit.GitHubApi("test-token", 5)
        with patch.object(audit, "urlopen", side_effect=[
            RemoteDisconnected("remote closed"), BytesIO(b'{"ahead_by":1}')
        ]) as fetch, patch.object(audit.time, "sleep") as sleep:
            response = api.get("/repos/acme/openhuman/compare/deadbeef...alice:main")
        self.assertEqual(response["ahead_by"], 1)
        self.assertEqual(fetch.call_count, 2)
        self.assertEqual(api.requests, 2)
        sleep.assert_called_once()

    def test_api_defers_after_bounded_transport_retries_instead_of_crashing(self):
        from http.client import RemoteDisconnected
        from unittest.mock import patch
        api = audit.GitHubApi("test-token", 8)
        with patch.object(audit, "urlopen", side_effect=RemoteDisconnected("remote closed")), \
                patch.object(audit.time, "sleep"):
            with self.assertRaises(audit.GitHubTemporarilyUnavailable):
                api.get("/repos/acme/openhuman/compare/deadbeef...alice:main")
        self.assertEqual(api.requests, 3)

    def test_failed_fork_is_checkpointed_and_later_forks_are_still_compared(self):
        from contextlib import redirect_stdout
        from io import BytesIO, StringIO
        from http.client import RemoteDisconnected
        from unittest.mock import patch

        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            checkpoint = root / "state.json"
            audit.write_checkpoint(checkpoint, {
                "upstream": "acme/openhuman", "sha": "deadbeef",
                "declared_fork_count": 2,
                "forks": {
                    name: {"full_name": name, "default_branch": "main"}
                    for name in ["alice/openhuman", "bob/openhuman"]
                },
                "comparisons": {}, "errors": {},
            })
            argv = [
                "audit", "--upstream", "acme/openhuman", "--budget", "8",
                "--checkpoint", str(checkpoint),
                "--output-json", str(root / "out.json"),
                "--output-md", str(root / "out.md"),
            ]
            with patch.object(sys, "argv", argv), \
                    patch.dict(audit.os.environ, {"GITHUB_TOKEN": "test-token"}), \
                    patch.object(audit, "urlopen", side_effect=[
                        RemoteDisconnected("remote closed"),
                        RemoteDisconnected("remote closed"),
                        RemoteDisconnected("remote closed"),
                        BytesIO(b'{"ahead_by": 1}'),
                    ]) as fetch, patch.object(audit.time, "sleep"), redirect_stdout(StringIO()):
                self.assertEqual(audit.main(), 2)
            self.assertEqual(fetch.call_count, 4)
            state = json.loads(checkpoint.read_text())
            self.assertIn("alice/openhuman", state["errors"])
            self.assertEqual(state["comparisons"]["bob/openhuman"]["ahead_by"], 1)
            self.assertIn("INCOMPLETE", (root / "out.md").read_text())

            # The next run retries only the failed fork, retains the completed
            # one and clears the old error after a successful comparison.
            with patch.object(sys, "argv", argv), \
                    patch.dict(audit.os.environ, {"GITHUB_TOKEN": "test-token"}), \
                    patch.object(audit, "urlopen", return_value=BytesIO(b'{"ahead_by": 0}')) as fetch, \
                    redirect_stdout(StringIO()):
                self.assertEqual(audit.main(), 0)
            self.assertEqual(fetch.call_count, 1)
            state = json.loads(checkpoint.read_text())
            self.assertEqual(state["errors"], {})
            self.assertEqual(len(state["comparisons"]), 2)

    def test_api_respects_the_request_budget_during_retries(self):
        from http.client import RemoteDisconnected
        from unittest.mock import patch
        api = audit.GitHubApi("test-token", 2)
        with patch.object(audit, "urlopen", side_effect=RemoteDisconnected("remote closed")), \
                patch.object(audit.time, "sleep"):
            with self.assertRaises(audit.RequestBudgetExhausted):
                api.get("/repos/acme/openhuman/compare/deadbeef...alice:main")
        self.assertEqual(api.requests, 2)

    def test_enumeration_transport_failure_preserves_an_incomplete_checkpoint(self):
        from contextlib import redirect_stdout
        from http.client import RemoteDisconnected
        from io import BytesIO, StringIO
        from unittest.mock import patch

        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            checkpoint = root / "state.json"
            state = {
                "upstream": "acme/openhuman", "sha": "deadbeef",
                "declared_fork_count": 2,
                "forks": {}, "comparisons": {}, "errors": {},
            }
            audit.write_checkpoint(checkpoint, state)
            argv = [
                "audit", "--upstream", "acme/openhuman", "--budget", "8",
                "--checkpoint", str(checkpoint),
                "--output-json", str(root / "out.json"),
                "--output-md", str(root / "out.md"),
            ]
            with patch.object(sys, "argv", argv), \
                    patch.dict(audit.os.environ, {"GITHUB_TOKEN": "test-token"}), \
                    patch.object(audit, "urlopen", side_effect=RemoteDisconnected("remote closed")), \
                    patch.object(audit.time, "sleep"), redirect_stdout(StringIO()) as output:
                self.assertEqual(audit.main(), 2)
            self.assertIn("Audit paused", output.getvalue())
            saved = json.loads(checkpoint.read_text())
            self.assertEqual(saved["sha"], state["sha"])
            self.assertEqual(saved["forks"], {})
            self.assertIn("_enumeration", saved["errors"])
            self.assertEqual(json.loads((root / "out.json").read_text()), saved)
            self.assertIn("INCOMPLETE", (root / "out.md").read_text())

            with patch.object(sys, "argv", argv), \
                    patch.dict(audit.os.environ, {"GITHUB_TOKEN": "test-token"}), \
                    patch.object(audit, "urlopen", side_effect=[
                        BytesIO(json.dumps([
                            {"id": index, "full_name": name, "default_branch": "main"}
                            for index, name in enumerate(["alice/openhuman", "bob/openhuman"], 1)
                        ]).encode()),
                        BytesIO(b'{"ahead_by": 0}'),
                        BytesIO(b'{"ahead_by": 1}'),
                    ]), redirect_stdout(StringIO()):
                self.assertEqual(audit.main(), 0)
            self.assertEqual(json.loads(checkpoint.read_text())["errors"], {})

    def test_atomic_checkpoint_roundtrip(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = pathlib.Path(tmp) / "state.json"
            state = {"upstream": "acme/openhuman", "sha": "deadbeef"}
            audit.write_checkpoint(target, state)
            self.assertEqual(json.loads(target.read_text()), state)
            self.assertFalse((target.parent / "state.json.tmp").exists())

    def test_initial_metadata_disconnect_is_checkpointed_and_resumes_setup(self):
        from contextlib import redirect_stdout
        from http.client import RemoteDisconnected
        from io import BytesIO, StringIO
        from unittest.mock import patch

        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            checkpoint = root / "state.json"
            argv = [
                "audit", "--upstream", "acme/openhuman", "--budget", "8",
                "--checkpoint", str(checkpoint),
                "--output-json", str(root / "out.json"),
                "--output-md", str(root / "out.md"),
            ]
            with patch.object(sys, "argv", argv), \
                    patch.dict(audit.os.environ, {"GITHUB_TOKEN": "test-token"}), \
                    patch.object(audit, "urlopen", side_effect=RemoteDisconnected("remote closed")), \
                    patch.object(audit.time, "sleep"), redirect_stdout(StringIO()):
                self.assertEqual(audit.main(), 2)
            state = json.loads(checkpoint.read_text())
            self.assertEqual(state["sha"], "")
            self.assertIsNone(state["declared_fork_count"])
            self.assertIn("_setup", state["errors"])
            self.assertEqual(json.loads((root / "out.json").read_text()), state)
            report = (root / "out.md").read_text()
            self.assertIn("INCOMPLETE", report)
            self.assertIn("(unresolved)", report)

            with patch.object(sys, "argv", argv), \
                    patch.dict(audit.os.environ, {"GITHUB_TOKEN": "test-token"}), \
                    patch.object(audit, "urlopen", side_effect=[
                        BytesIO(b'{"default_branch": "main", "forks_count": 1}'),
                        BytesIO(b'{"sha": "deadbeef"}'),
                        BytesIO(b'[{"id": 1, "full_name": "alice/openhuman", "default_branch": "main"}]'),
                        BytesIO(b'{"ahead_by": 0}'),
                    ]), redirect_stdout(StringIO()):
                self.assertEqual(audit.main(), 0)
            state = json.loads(checkpoint.read_text())
            self.assertEqual(state["sha"], "deadbeef")
            self.assertEqual(state["errors"], {})
            self.assertIn("alice/openhuman", state["comparisons"])


if __name__ == "__main__":
    unittest.main()
