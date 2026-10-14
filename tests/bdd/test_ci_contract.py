"""CI contract scenarios that can be checked from any platform."""

from pathlib import Path
import plistlib
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]


class CiJourneyContract(unittest.TestCase):
    def test_publication_workflows_use_read_only_permissions_and_pinned_actions(self):
        for path in (ROOT / ".github/workflows").glob("*.yml"):
            workflow = path.read_text()
            self.assertRegex(workflow, r"permissions:\n  contents: read\n")
            self.assertNotIn("pull_request_target", workflow)
            actions = re.findall(r"uses:\s*([^\s]+)", workflow)
            self.assertTrue(actions)
            for action in actions:
                self.assertRegex(action, r"^[^@]+@[0-9a-f]{40}$")

    def test_security_workflow_audits_dependencies_and_complete_history(self):
        workflow = (ROOT / ".github/workflows/security.yml").read_text()
        for required in ("cargo audit", "pnpm audit", "fetch-depth: 0",
                         "gitleaks git . --redact", "sha256sum --check",
                         "export_third_party_notices.py --check",
                         "check_publication.py"):
            self.assertIn(required, workflow)
        self.assertNotIn("--ignore", workflow)

    # Covers: BDD-JRN-002-15
    def test_bdd_jrn_002_15_bdd_suites_run_in_three_os_matrix(self) -> None:
        """Given journey BDDs; when CI runs; then its matrix covers three operating systems."""
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        matrix = re.search(r"(?m)^\s+os:\s*\[([^]]+)\]", workflow)
        self.assertIsNotNone(matrix)
        systems = {item.strip() for item in matrix.group(1).split(",")}
        self.assertEqual(systems, {"ubuntu-latest", "macos-latest", "windows-latest"})
        self.assertIn("runs-on: ${{ matrix.os }}", workflow)
        self.assertIn("Run web journey BDD scenarios", workflow)
        self.assertIn("Run journey BDD scenarios", workflow)
        self.assertIn("Measure scheduled punctuality", workflow)
        self.assertIn("Validate Windows installer syntax", workflow)
        self.assertIn("Parser]::ParseFile", workflow)

    # Covers: BDD-JRN-001-13
    def test_bdd_jrn_001_13_all_supervisors_restart_failed_daemon(self) -> None:
        """Given a daemon failure; when supervised; then each OS package declares restart."""
        systemd = (ROOT / "packaging/systemd/command-scheduler.service").read_text()
        launchd = (ROOT / "packaging/launchd/com.command-scheduler.daemon.plist.in").read_text()
        windows = (ROOT / "packaging/windows/install-service.ps1").read_text()
        self.assertRegex(systemd, r"(?m)^Restart=on-failure$")
        self.assertIn("<key>KeepAlive</key><true/>", launchd)
        self.assertIn("-RestartCount 3", windows)
        self.assertIn("-RestartInterval (New-TimeSpan -Minutes 1)", windows)
        self.assertIn("command-scheduler-cli.exe", windows)
        self.assertIn("SetEnvironmentVariable('Path'", windows)
        self.assertIn("COMMAND_SCHEDULER_WEB_DIR=%h/.local/share/command-scheduler/ui", systemd)
        self.assertIn("@INSTALL_DIR@/ui", launchd)
        self.assertIn("Copy-Item $webDirectory", windows)
        self.assertIn("Stop-ScheduledTask -TaskName $taskName", windows)
        self.assertLess(
            windows.index("Stop-ScheduledTask -TaskName $taskName"),
            windows.index("Copy-Item $daemon"),
        )
        rendered = launchd.replace("@INSTALL_DIR@", "/tmp/command-scheduler").replace(
            "@HOME_DIR@", "/tmp/user"
        )
        plist = plistlib.loads(rendered.encode())
        self.assertTrue(plist["KeepAlive"])
        self.assertEqual(
            plist["EnvironmentVariables"]["COMMAND_SCHEDULER_WEB_DIR"],
            "/tmp/command-scheduler/ui",
        )


if __name__ == "__main__":
    unittest.main()
