"""Tests for .rulesync/skills/bump-release/scripts, run by `mise run skills:test-bump-release`."""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[2] / ".rulesync/skills/bump-release/scripts"
sys.path.insert(0, str(SCRIPTS))

from analyze_commits import BumpType, CommitParser  # noqa: E402

# The exit codes prepare-release.yml maps to a bump type.
EXIT_CODE = {"none": 0, "patch": 1, "minor": 2, "major": 3}


class ParseCommitTest(unittest.TestCase):
    def test_bump_type_by_subject(self):
        cases = {
            "feat!: drop the flag": BumpType.MAJOR,
            "feat(scope)!: drop the flag": BumpType.MAJOR,
            "✨ feat!: drop the flag": BumpType.MAJOR,
            "✨ feat(scope)!: drop the flag": BumpType.MAJOR,
            "fix(sdk)!: change the error type": BumpType.MAJOR,
            "✨ feat(scope): add a flag": BumpType.MINOR,
            "🩹 fix: handle a 429": BumpType.PATCH,
            "📚 docs: reword": BumpType.NONE,
            "not a conventional commit": BumpType.NONE,
        }
        for subject, expected in cases.items():
            with self.subTest(subject=subject):
                self.assertEqual(CommitParser.parse_commit(subject), expected)

    def test_breaking_change_footer_is_major(self):
        msg = "✨ feat: add a flag\n\nBREAKING CHANGE: the old flag is gone"
        self.assertEqual(CommitParser.parse_commit(msg), BumpType.MAJOR)


def git(repo, *args):
    subprocess.run(
        ["git", "-c", "user.name=t", "-c", "user.email=t@example.com", *args],
        cwd=repo, check=True, capture_output=True,
    )


class JsonModeTest(unittest.TestCase):
    """Run analyze_commits.py and bump_release.sh's analyze_commits() in a scratch repo."""

    def repo_with_commit(self, subject):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        repo = tmp.name
        git(repo, "init", "-q")
        git(repo, "commit", "-q", "--allow-empty", "-m", "chore: initial")
        git(repo, "tag", "v1.2.3")
        git(repo, "commit", "-q", "--allow-empty", "-m", subject)
        return repo

    def run_analyze_commits(self, repo):
        return subprocess.run(
            [str(SCRIPTS / "analyze_commits.py"), "--format", "json"],
            cwd=repo, capture_output=True, text=True,
        )

    def run_shell_function(self, repo):
        # Source the script (it only runs main when executed) and call the function.
        return subprocess.run(
            ["bash", "-c", f'source "{SCRIPTS}/bump_release.sh"; analyze_commits'],
            cwd=repo, capture_output=True, text=True,
        )

    def test_exit_code_and_json_for_each_bump(self):
        cases = {
            "patch": ("🩹 fix: handle a 429", "1.2.4"),
            "minor": ("✨ feat: add a flag", "1.3.0"),
            "major": ("✨ feat(scope)!: drop the flag", "2.0.0"),
            "none": ("📚 docs: reword", "1.2.3"),
        }
        for bump, (subject, new_version) in cases.items():
            with self.subTest(bump=bump):
                repo = self.repo_with_commit(subject)

                result = self.run_analyze_commits(repo)
                self.assertEqual(result.returncode, EXIT_CODE[bump])
                self.assertEqual(json.loads(result.stdout)["bump_type"], bump)

                # bump_release.sh must treat every one of those exits as success
                # and print only the JSON on stdout.
                result = self.run_shell_function(repo)
                self.assertEqual(result.returncode, 0, result.stderr)
                analysis = json.loads(result.stdout)
                self.assertEqual(analysis["bump_type"], bump)
                self.assertEqual(analysis["new_version"], new_version)

    def test_shell_function_fails_when_there_are_no_commits_since_the_tag(self):
        repo = self.repo_with_commit("🩹 fix: handle a 429")
        git(repo, "tag", "v1.2.4")

        result = self.run_shell_function(repo)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Failed to analyze commits", result.stderr)


if __name__ == "__main__":
    unittest.main()
