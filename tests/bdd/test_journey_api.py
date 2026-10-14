"""Executable API/CLI journey scenarios. Test names retain source BDD-JRN identifiers."""

from __future__ import annotations

import json
import os
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest
import urllib.error
import urllib.request
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXE = ".exe" if os.name == "nt" else ""
DAEMON = ROOT / "target" / "debug" / f"command-scheduler{EXE}"
CLI = ROOT / "target" / "debug" / f"command-scheduler-cli{EXE}"


def free_port() -> int:
    with socket.socket() as server:
        server.bind(("127.0.0.1", 0))
        return server.getsockname()[1]


def timestamp_after(seconds: float) -> str:
    return (datetime.now(timezone.utc) + timedelta(seconds=seconds)).isoformat()


class LocalDaemon(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="command-scheduler-bdd-")
        self.port = free_port()
        self.base = f"http://127.0.0.1:{self.port}"
        self.db = Path(self.temp.name) / "bdd.sqlite3"
        env = os.environ.copy()
        env.update(
            {
                "COMMAND_SCHEDULER_ADDR": f"127.0.0.1:{self.port}",
                "COMMAND_SCHEDULER_DB_PATH": str(self.db),
                "COMMAND_SCHEDULER_WEB_DIR": str(ROOT / "ui/dist"),
            }
        )
        self.process: subprocess.Popen[bytes] | None = None
        self.start_daemon(env)

    def tearDown(self) -> None:
        self.stop_daemon()
        self.temp.cleanup()

    def start_daemon(self, env: dict[str, str] | None = None) -> None:
        if env is None:
            env = os.environ.copy()
            env.update(
                {
                    "COMMAND_SCHEDULER_ADDR": f"127.0.0.1:{self.port}",
                    "COMMAND_SCHEDULER_DB_PATH": str(self.db),
                    "COMMAND_SCHEDULER_WEB_DIR": str(ROOT / "ui/dist"),
                }
            )
        self.process = subprocess.Popen(
            [str(DAEMON)], cwd=ROOT, env=env,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                self.fail(f"daemon exited with {self.process.returncode}")
            try:
                if self.http("GET", "/health", api=False) == b"ok":
                    return
            except (OSError, urllib.error.URLError):
                time.sleep(0.1)
        self.fail("daemon did not become healthy")

    def stop_daemon(self) -> None:
        if self.process is not None and self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
        self.process = None

    def http(self, method: str, path: str, body: object | None = None,
             headers: dict[str, str] | None = None, api: bool = True) -> bytes:
        url = self.base + ("/api/v1" if api else "") + path
        data = None if body is None else json.dumps(body).encode()
        request_headers = dict(headers or {})
        if data is not None:
            request_headers["Content-Type"] = "application/json"
        request = urllib.request.Request(url, data=data, method=method, headers=request_headers)
        try:
            with urllib.request.urlopen(request, timeout=5) as response:
                return response.read()
        except urllib.error.HTTPError as error:
            self.last_error_status = error.code
            return error.read()

    def json(self, method: str, path: str, body: object | None = None,
             headers: dict[str, str] | None = None) -> dict:
        content = self.http(method, path, body, headers)
        return json.loads(content) if content else {}

    def make_schedule(self, **extra: object) -> dict:
        body = {
            "program": sys.executable,
            "args": ["-c", "print('bdd-ok')"],
            "timing": {"type": "once", "at": timestamp_after(3600)},
            "missedPolicy": "skip",
            **extra,
        }
        return self.json("POST", "/schedules", body)

    def wait_occurrence(self, occurrence_id: str, terminal: set[str] | None = None) -> dict:
        terminal = terminal or {"succeeded", "failed", "cancelled", "interrupted"}
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            occurrence = self.json("GET", f"/occurrences/{occurrence_id}")
            if occurrence.get("status") in terminal:
                return occurrence
            time.sleep(0.05)
        self.fail(f"occurrence {occurrence_id} did not reach a terminal state")

    def run_now(self, schedule_id: str) -> dict:
        return self.json("POST", f"/schedules/{schedule_id}/runs")

    # Covers: BDD-JRN-002-01 BDD-JRN-002-02 BDD-JRN-002-06
    def test_bdd_jrn_002_01_02_06_create_replay_and_conflict(self) -> None:
        """Given a valid request; when replayed with its key; then one schedule remains."""
        body = {
            "program": sys.executable,
            "args": ["-c", "print('ok')"],
            "timing": {"type": "once", "at": timestamp_after(3600)},
            "missedPolicy": "sequence",
        }
        headers = {"Idempotency-Key": "bdd-create"}
        first = self.json("POST", "/schedules", body, headers)
        replay = self.json("POST", "/schedules", body, headers)
        self.assertEqual(first["id"], replay["id"])
        changed = dict(body, name="different request")
        self.http("POST", "/schedules", changed, headers)
        self.assertEqual(self.last_error_status, 409)
        page = self.json("GET", "/schedules?limit=100")
        self.assertEqual(sum(item["id"] == first["id"] for item in page["items"]), 1)

    # Covers: BDD-JRN-001-03
    def test_bdd_jrn_001_03_schedule_persists_after_daemon_restart(self) -> None:
        """Given a saved schedule; when daemon restarts; then config and missed-time policy are recovered."""
        script = (
            "import os,json; print(json.dumps({key:os.environ.get(key) for key in "
            "('COMMAND_SCHEDULER_MISSED_COUNT','COMMAND_SCHEDULER_MISSED_FROM','COMMAND_SCHEDULER_MISSED_THROUGH')}))"
        )
        created = self.make_schedule(
            name="restart-persistence",
            program=sys.executable,
            args=["-c", script],
            timing={"type": "cron", "expression": "* * * * *"},
            missedPolicy="consolidate",
        )
        self.stop_daemon()
        cursor = int(time.time() // 60 * 60 * 1_000_000) - 5 * 60 * 1_000_000
        with sqlite3.connect(self.db) as database:
            database.execute(
                "UPDATE schedules SET last_evaluated_at=? WHERE id=?",
                (cursor, created["id"]),
            )
        self.start_daemon()
        restored = self.json("GET", f"/schedules/{created['id']}")
        self.assertEqual(restored["id"], created["id"])
        self.assertEqual(restored["name"], "restart-persistence")
        self.assertEqual(restored["missedPolicy"], "consolidate")
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            page = self.json("GET", f"/schedules/{created['id']}/occurrences?limit=10")
            if page["items"]:
                occurrence = self.wait_occurrence(page["items"][0]["id"])
                if occurrence["status"] == "succeeded":
                    self.assertIsNotNone(occurrence.get("missedOccurrences"))
                    self.assertIn("COMMAND_SCHEDULER_MISSED_COUNT", occurrence["stdout"])
                    return
            time.sleep(0.05)
        self.fail("restart did not recover missed occurrences under the saved policy")

    # Covers: BDD-JRN-001-04 BDD-JRN-001-05 BDD-JRN-001-06
    def test_bdd_jrn_001_04_05_06_restart_applies_each_missed_policy(self) -> None:
        """Given overdue cron schedules; when daemon restarts; then each saved policy applies."""
        schedules = {
            policy: self.make_schedule(
                name=f"restart-{policy}",
                args=["-c", "print('recovered')"],
                timing={"type": "cron", "expression": "* * * * *"},
                missedPolicy=policy,
            )
            for policy in ("skip", "consolidate", "sequence")
        }
        self.stop_daemon()
        cursor = int(time.time() // 60 * 60 * 1_000_000) - 5 * 60 * 1_000_000
        with sqlite3.connect(self.db) as database:
            for schedule in schedules.values():
                database.execute(
                    "UPDATE schedules SET last_evaluated_at=? WHERE id=?",
                    (cursor, schedule["id"]),
                )
        self.start_daemon()

        self.assertEqual(
            self.json("GET", f"/schedules/{schedules['skip']['id']}/occurrences?limit=100")["items"],
            [],
        )
        recovered: dict[str, list[dict]] = {}
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            for policy in ("consolidate", "sequence"):
                page = self.json(
                    "GET", f"/schedules/{schedules[policy]['id']}/occurrences?limit=100"
                )
                recovered[policy] = page["items"]
            if recovered["consolidate"] and recovered["sequence"]:
                break
            time.sleep(0.05)
        self.assertTrue(recovered["consolidate"], "consolidate policy produced no run")
        self.assertTrue(recovered["sequence"], "sequence policy produced no runs")
        for policy, occurrences in recovered.items():
            for occurrence in occurrences:
                self.wait_occurrence(occurrence["id"])
        self.assertTrue(
            any(item.get("missedOccurrences") for item in recovered["consolidate"]),
            "consolidated recovery did not expose missed occurrence metadata",
        )
        self.assertGreaterEqual(len(recovered["sequence"]), 2)

    # Covers: BDD-JRN-002-04 BDD-JRN-002-07 BDD-JRN-002-13
    def test_bdd_jrn_002_04_07_parameters_reach_process_without_shell(self) -> None:
        """Given JSON parameters; when a child starts; then env and literal args arrive."""
        script = "import os,sys; print(os.environ['COMMAND_SCHEDULER_PARAMS_JSON']); print(sys.argv[1])"
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", script, "literal value with spaces"],
            parameters={"nested": [True, 5]},
        )
        occurrence = self.run_now(schedule["id"])
        result = self.wait_occurrence(occurrence["id"])
        self.assertEqual(result["status"], "succeeded")
        self.assertIn('{"nested":[true,5]}', result["stdout"])
        self.assertIn("literal value with spaces", result["stdout"])

    # Covers: BDD-JRN-002-04 BDD-JRN-002-05
    def test_bdd_jrn_002_04_05_accepts_json_values_and_rejects_oversize(self) -> None:
        """Given JSON values of any kind; when schedules are created; then size limits hold."""
        values = [{"a": 1}, [True, 2], "text", 42, False, None]
        for index, value in enumerate(values):
            schedule = self.make_schedule(name=f"json-{index}", parameters=value)
            self.assertEqual(schedule["parameters"], value)
        before = len(self.json("GET", "/schedules?limit=100")["items"])
        oversized = self.make_schedule(name="too-large", parameters={"value": "x" * 65536})
        self.assertEqual(self.last_error_status, 413)
        self.assertTrue(oversized)
        after = len(self.json("GET", "/schedules?limit=100")["items"])
        self.assertEqual(after, before)

    # Covers: BDD-JRN-003-01 BDD-JRN-007-01
    def test_bdd_jrn_003_01_007_01_parameter_update_triggers_run(self) -> None:
        """Given an enabled trigger; when parameters change; then the new values execute now."""
        script = "import os; print(os.environ['COMMAND_SCHEDULER_PARAMS_JSON'])"
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", script],
            parameters={"old": True},
            parameterUpdateTrigger={"enabled": True, "scheduleBehavior": "coexist"},
        )
        updated = self.json("PUT", f"/schedules/{schedule['id']}/parameters", {"new": 2})
        self.assertEqual(updated["schedule"]["parameters"], {"new": 2})
        occurrence = self.wait_occurrence(updated["occurrenceId"])
        self.assertEqual(occurrence["source"], "parameter_update")
        self.assertIn('{"new":2}', occurrence["stdout"])

    # Covers: BDD-JRN-003-02
    def test_bdd_jrn_003_02_late_parameter_update_keeps_running_snapshot(self) -> None:
        """Given a running occurrence; when parameters change; then its snapshot is unchanged."""
        script = (
            "import os,time; print('started', flush=True); time.sleep(1); "
            "print(os.environ['COMMAND_SCHEDULER_PARAMS_JSON'])"
        )
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", script],
            parameters={"version": "old"},
        )
        occurrence = self.run_now(schedule["id"])
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            current = self.json("GET", f"/occurrences/{occurrence['id']}")
            if current["status"] == "running":
                break
            time.sleep(0.02)
        else:
            self.fail("occurrence did not start before parameter update")

        updated = self.json(
            "PUT", f"/schedules/{schedule['id']}/parameters", {"version": "new"}
        )
        self.assertEqual(updated["schedule"]["parameters"], {"version": "new"})
        completed = self.wait_occurrence(occurrence["id"])
        self.assertEqual(completed["status"], "succeeded")
        self.assertIn('{"version":"old"}', completed["stdout"])
        self.assertNotIn('{"version":"new"}', completed["stdout"])

    # Covers: BDD-JRN-007-02
    def test_bdd_jrn_007_02_parameter_update_without_trigger_does_not_run(self) -> None:
        """Given a disabled update trigger; when parameters change; then no run starts."""
        schedule = self.make_schedule(
            parameters={"version": "old"},
            parameterUpdateTrigger={"enabled": False, "scheduleBehavior": "coexist"},
        )
        updated = self.json(
            "PUT", f"/schedules/{schedule['id']}/parameters", {"version": "new"}
        )
        self.assertEqual(updated["schedule"]["parameters"], {"version": "new"})
        self.assertNotIn("occurrenceId", updated)
        page = self.json("GET", f"/schedules/{schedule['id']}/occurrences?limit=10")
        self.assertEqual(page["items"], [])

    # Covers: BDD-JRN-007-03
    def test_bdd_jrn_007_03_rejected_parameter_update_does_not_run(self) -> None:
        """Given an enabled update trigger; when parameters are rejected; then no run starts."""
        schedule = self.make_schedule(
            parameters={"version": "old"},
            parameterUpdateTrigger={"enabled": True, "scheduleBehavior": "coexist"},
        )
        response = self.http(
            "PUT", f"/schedules/{schedule['id']}/parameters", {"value": "x" * 65536}
        )
        self.assertEqual(self.last_error_status, 413)
        self.assertTrue(response)
        saved = self.json("GET", f"/schedules/{schedule['id']}")
        self.assertEqual(saved["parameters"], {"version": "old"})
        page = self.json("GET", f"/schedules/{schedule['id']}/occurrences?limit=10")
        self.assertEqual(page["items"], [])

    # Covers: BDD-JRN-003-03 BDD-JRN-003-04
    def test_bdd_jrn_003_03_04_invalid_parameter_update_preserves_previous_value(self) -> None:
        """Given existing parameters; when an oversized update is rejected; then the old value remains."""
        schedule = self.make_schedule(parameters={"kept": True})
        self.http(
            "PUT", f"/schedules/{schedule['id']}/parameters", {"value": "x" * 65536}
        )
        self.assertEqual(self.last_error_status, 413)
        saved = self.json("GET", f"/schedules/{schedule['id']}")
        self.assertEqual(saved["parameters"], {"kept": True})
        updated = self.json("PUT", f"/schedules/{schedule['id']}/parameters", [1, "valid"])
        self.assertEqual(updated["schedule"]["parameters"], [1, "valid"])

    # Covers: BDD-JRN-006-06
    def test_bdd_jrn_006_06_pause_resume_uses_saved_missed_policy(self) -> None:
        """Given a due time passes while paused; when resumed; then saved policy runs it."""
        schedule = self.make_schedule(
            timing={"type": "once", "at": timestamp_after(1.5)},
            missedPolicy="sequence",
        )
        self.json("POST", f"/schedules/{schedule['id']}/pause")
        time.sleep(2.0)
        resumed = self.json("POST", f"/schedules/{schedule['id']}/resume")
        self.assertEqual(resumed["status"], "active")
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            page = self.json("GET", f"/schedules/{schedule['id']}/occurrences?limit=10")
            if page["items"] and page["items"][0]["status"] == "succeeded":
                return
            time.sleep(0.05)
        self.fail("resume did not apply the configured missed policy")

    # Covers: BDD-JRN-006-03
    def test_bdd_jrn_006_03_query_resolves_missing_pause_confirmation(self) -> None:
        """Given pause response is lost; when state is queried; then current state is known."""
        schedule = self.make_schedule()
        request = urllib.request.Request(
            self.base + f"/api/v1/schedules/{schedule['id']}/pause", method="POST"
        )
        with urllib.request.urlopen(request, timeout=5) as response:
            # Simulate the caller losing the response after the server applies the action.
            response.close()
        current = self.json("GET", f"/schedules/{schedule['id']}")
        self.assertEqual(current["status"], "paused")

    # Covers: BDD-JRN-002-03 BDD-JRN-002-09 BDD-JRN-002-14
    def test_bdd_jrn_002_03_09_14_openapi_validation_invalid_request_and_origin(self) -> None:
        """Given local API; when docs and hostile origin are queried; then contract is served and origin denied."""
        spec = self.http("GET", "/openapi.yaml", api=False)
        self.assertIn(b"openapi: 3.1.0", spec)
        page = self.http("GET", "/docs/api", api=False)
        self.assertIn(b"<html", page.lower())
        invalid = self.http(
            "POST", "/schedules",
            {"program": sys.executable, "args": [], "timing": {"type": "cron", "expression": "invalid"}},
        )
        self.assertEqual(self.last_error_status, 400)
        self.assertTrue(invalid)
        bad = self.http(
            "POST", "/schedules",
            {"program": sys.executable, "args": [], "timing": {"type": "cron", "expression": "* * * * *"}},
            {"Origin": "https://example.invalid"},
        )
        self.assertEqual(self.last_error_status, 403)
        self.assertTrue(bad)

    # Covers: BDD-JRN-002-10
    def test_bdd_jrn_002_10_api_unavailable_does_not_confirm_creation(self) -> None:
        """Given the API is unavailable; when creation is attempted; then nothing is confirmed."""
        self.stop_daemon()
        request = urllib.request.Request(
            self.base + "/api/v1/schedules",
            data=json.dumps({
                "program": sys.executable,
                "args": ["-c", "print('must-not-run')"],
                "timing": {"type": "once", "at": timestamp_after(3600)},
                "missedPolicy": "skip",
            }).encode(),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with self.assertRaises(urllib.error.URLError):
            urllib.request.urlopen(request, timeout=1)
        self.start_daemon()
        self.assertEqual(self.json("GET", "/schedules?limit=100")["items"], [])

    # Covers: BDD-JRN-011-03
    def test_bdd_jrn_011_03_cancel_active_child(self) -> None:
        """Given a running child; when cancelled; then it ends cancelled and is queryable."""
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", "import time; time.sleep(30)"],
        )
        occurrence = self.run_now(schedule["id"])
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            current = self.json("GET", f"/occurrences/{occurrence['id']}")
            if current["status"] == "running":
                break
            time.sleep(0.05)
        else:
            self.fail("child did not start")
        self.http("POST", f"/occurrences/{occurrence['id']}/cancel")
        result = self.wait_occurrence(occurrence["id"])
        self.assertEqual(result["status"], "cancelled")

    # Covers: BDD-JRN-011-02
    def test_bdd_jrn_011_02_accepts_concurrent_runs_without_global_cap(self) -> None:
        """Given several accepted runs; when another is requested; then all can run together."""
        occurrence_ids = []
        for index in range(4):
            schedule = self.make_schedule(
                name=f"concurrent-{index}",
                program=sys.executable,
                args=["-c", "import time; time.sleep(2); print('done')"],
            )
            occurrence_ids.append(self.run_now(schedule["id"])["id"])

        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            states = [
                self.json("GET", f"/occurrences/{occurrence_id}")["status"]
                for occurrence_id in occurrence_ids
            ]
            if all(state == "running" for state in states):
                break
            if any(state in {"failed", "cancelled", "interrupted"} for state in states):
                self.fail(f"a concurrent run was rejected or failed: {states}")
            time.sleep(0.05)
        else:
            self.fail(f"all accepted runs did not overlap: {states}")

        for occurrence_id in occurrence_ids:
            self.assertEqual(self.wait_occurrence(occurrence_id)["status"], "succeeded")

    # Covers: BDD-JRN-008-03
    def test_bdd_jrn_008_03_cancelling_terminal_occurrence_is_rejected(self) -> None:
        """Given no active occurrence; when cancellation is requested; then state is reported."""
        schedule = self.make_schedule()
        occurrence = self.wait_occurrence(self.run_now(schedule["id"])["id"])
        self.assertEqual(occurrence["status"], "succeeded")
        response = self.http("POST", f"/occurrences/{occurrence['id']}/cancel")
        self.assertEqual(self.last_error_status, 409)
        self.assertTrue(response)
        current = self.json("GET", f"/occurrences/{occurrence['id']}")
        self.assertEqual(current["status"], "succeeded")

    # Covers: BDD-JRN-009-04 BDD-JRN-012-04
    def test_bdd_jrn_012_04_interrupted_retry_requires_confirmation(self) -> None:
        """Given persisted running state after restart; retry waits for explicit confirmation."""
        schedule = self.make_schedule()
        occurrence = self.run_now(schedule["id"])
        self.wait_occurrence(occurrence["id"])
        started = int(time.time() * 1_000_000)
        with sqlite3.connect(self.db) as database:
            database.execute(
                "UPDATE occurrences SET status='running', started_at=?, finished_at=NULL WHERE id=?",
                (started, occurrence["id"]),
            )
        self.stop_daemon()
        self.start_daemon()
        interrupted = self.json("GET", f"/occurrences/{occurrence['id']}")
        self.assertEqual(interrupted["status"], "interrupted")
        self.http("POST", f"/occurrences/{occurrence['id']}/retry", {"confirmInterrupted": False})
        self.assertEqual(self.last_error_status, 409)
        retry = self.json(
            "POST", f"/occurrences/{occurrence['id']}/retry", {"confirmInterrupted": True}
        )
        self.assertEqual(retry["retryOf"], occurrence["id"])
        self.assertEqual(self.wait_occurrence(retry["id"])["status"], "succeeded")

    # Covers: BDD-JRN-012-01
    def test_bdd_jrn_012_01_api_retries_failed_occurrence(self) -> None:
        """Given a failed run; when API retries; then the retry links to the original."""
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", "raise SystemExit(7)"],
            retryPolicy={"maxRetries": 0, "delaysSeconds": []},
        )
        original = self.wait_occurrence(self.run_now(schedule["id"])["id"])
        self.assertEqual(original["status"], "failed")
        retry = self.json("POST", f"/occurrences/{original['id']}/retry")
        self.assertEqual(retry["retryOf"], original["id"])
        self.assertEqual(self.wait_occurrence(retry["id"])["status"], "failed")

    # Covers: BDD-JRN-009-03 BDD-JRN-010-01 BDD-JRN-010-02 BDD-JRN-010-03 BDD-JRN-011-01 BDD-JRN-011-04 BDD-JRN-012-03
    def test_bdd_jrn_010_01_02_03_011_01_04_012_03_query_run_and_reject_invalid_controls(self) -> None:
        """Given an empty schedule; when queried/run/controlled; then observable results and state errors are returned."""
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", "import sys; print('stdout-value'); print('stderr-value', file=sys.stderr)"],
        )
        empty = self.json("GET", f"/schedules/{schedule['id']}/occurrences?limit=10")
        self.assertEqual(empty["items"], [])

        occurrence = self.wait_occurrence(self.run_now(schedule["id"])["id"])
        self.assertEqual(occurrence["status"], "succeeded")
        self.assertIn("stdout-value", occurrence["stdout"])
        self.assertIn("stderr-value", occurrence["stderr"])
        listed = self.json("GET", f"/schedules/{schedule['id']}/occurrences?limit=10")
        self.assertEqual([item["id"] for item in listed["items"]], [occurrence["id"]])
        fetched = self.json("GET", f"/occurrences/{occurrence['id']}")
        self.assertEqual(fetched["exitCode"], 0)

        self.http("POST", f"/occurrences/{occurrence['id']}/cancel")
        self.assertEqual(self.last_error_status, 409)
        self.http("POST", f"/occurrences/{occurrence['id']}/retry")
        self.assertEqual(self.last_error_status, 409)
        unchanged = self.json("GET", f"/occurrences/{occurrence['id']}")
        self.assertEqual(unchanged["status"], "succeeded")

    # Covers: BDD-JRN-004-04 BDD-JRN-010-03 BDD-JRN-012-06
    def test_bdd_jrn_010_03_012_06_output_is_capped_and_marked(self) -> None:
        """Given a child emits over 1 MiB per stream; when queried; then both streams are capped and marked."""
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", "import sys; sys.stdout.write('x'*1100000); sys.stderr.write('y'*1100000)"],
        )
        occurrence = self.wait_occurrence(self.run_now(schedule["id"])["id"])
        self.assertEqual(occurrence["status"], "succeeded")
        self.assertEqual(len(occurrence["stdout"].encode()), 1024 * 1024)
        self.assertEqual(len(occurrence["stderr"].encode()), 1024 * 1024)
        self.assertTrue(occurrence["stdoutTruncated"])
        self.assertTrue(occurrence["stderrTruncated"])

    # Covers: BDD-JRN-002-08 BDD-JRN-006-01 BDD-JRN-006-02 BDD-JRN-006-04 BDD-JRN-006-05
    def test_bdd_jrn_002_08_006_01_02_04_05_api_schedule_policy_and_lifecycle(self) -> None:
        """Given a schedule; when policy/configuration and lifecycle change; then API state reflects accepted values."""
        schedule = self.make_schedule(missedPolicy="consolidate")
        self.assertEqual(schedule["missedPolicy"], "consolidate")
        updated = self.json(
            "PATCH", f"/schedules/{schedule['id']}", {"missedPolicy": "sequence"}
        )
        self.assertEqual(updated["missedPolicy"], "sequence")
        current = self.json("GET", f"/schedules/{schedule['id']}")
        self.assertEqual(current["missedPolicy"], "sequence")

        paused = self.json("POST", f"/schedules/{schedule['id']}/pause")
        self.assertEqual(paused["status"], "paused")
        resumed = self.json("POST", f"/schedules/{schedule['id']}/resume")
        self.assertEqual(resumed["status"], "active")
        cancelled = self.json("POST", f"/schedules/{schedule['id']}/cancel")
        self.assertEqual(cancelled["status"], "cancelled")
        fetched = self.json("GET", f"/schedules/{schedule['id']}")
        self.assertEqual(fetched["status"], "cancelled")

    # Covers: BDD-JRN-002-12 BDD-JRN-002-17
    def test_bdd_jrn_002_12_17_api_recovers_delayed_occurrences_with_range_metadata(self) -> None:
        """Given delayed cron occurrences without a daemon restart; when processed; then consolidation metadata reaches API and child."""
        script = (
            "import os,json; print(json.dumps({key:os.environ.get(key) for key in "
            "('COMMAND_SCHEDULER_MISSED_COUNT','COMMAND_SCHEDULER_MISSED_FROM','COMMAND_SCHEDULER_MISSED_THROUGH')}))"
        )
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", script],
            timing={"type": "cron", "expression": "* * * * *"},
            missedPolicy="consolidate",
        )
        current_minute = int(time.time() // 60 * 60 * 1_000_000)
        cursor = current_minute - 5 * 60 * 1_000_000
        with sqlite3.connect(self.db) as database:
            database.execute(
                "UPDATE schedules SET last_evaluated_at=? WHERE id=?",
                (cursor, schedule["id"]),
            )
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            page = self.json("GET", f"/schedules/{schedule['id']}/occurrences?limit=10")
            if page["items"]:
                occurrence = self.wait_occurrence(page["items"][0]["id"])
                if occurrence["status"] == "succeeded":
                    break
            time.sleep(0.05)
        else:
            self.fail("delayed cron occurrences were not consolidated")
        self.assertIn("missedOccurrences", occurrence, repr(occurrence))
        metadata = occurrence["missedOccurrences"]
        child_metadata = json.loads(occurrence["stdout"])
        self.assertEqual(occurrence["source"], "recovery")
        self.assertIn(metadata["count"], {5, 6})
        self.assertEqual(child_metadata["COMMAND_SCHEDULER_MISSED_COUNT"], str(metadata["count"]))
        self.assertEqual(child_metadata["COMMAND_SCHEDULER_MISSED_FROM"], metadata["from"])
        self.assertEqual(child_metadata["COMMAND_SCHEDULER_MISSED_THROUGH"], metadata["through"])

    # Covers: BDD-JRN-012-02 BDD-JRN-012-05
    def test_bdd_jrn_012_02_05_automatic_retry_policy_and_default_schedule(self) -> None:
        """Given configured and default retry policies; when a run fails; then retries are queued with policy timing."""
        configured = self.make_schedule(
            program=sys.executable,
            args=["-c", "raise SystemExit(9)"],
            retryPolicy={"maxRetries": 1, "delaysSeconds": [1]},
        )
        first = self.wait_occurrence(self.run_now(configured["id"])["id"])
        self.assertEqual(first["status"], "failed")
        deadline = time.monotonic() + 5
        retry = None
        while time.monotonic() < deadline:
            items = self.json("GET", f"/schedules/{configured['id']}/occurrences?limit=10")["items"]
            retry = next((item for item in items if item["retryOf"] == first["id"]), None)
            if retry:
                break
            time.sleep(0.05)
        self.assertIsNotNone(retry)
        self.assertEqual(retry["attempt"], 2)
        self.assertEqual(self.wait_occurrence(retry["id"])["status"], "failed")

        default = self.make_schedule(
            program=sys.executable,
            args=["-c", "raise SystemExit(9)"],
        )
        self.assertEqual(default["retryPolicy"], {"maxRetries": 3, "delaysSeconds": [60, 300, 900]})
        original = self.wait_occurrence(self.run_now(default["id"])["id"])
        items = self.json("GET", f"/schedules/{default['id']}/occurrences?limit=10")["items"]
        queued = next(item for item in items if item["retryOf"] == original["id"])
        delay = (datetime.fromisoformat(queued["scheduledFor"].replace("Z", "+00:00")) -
                 datetime.fromisoformat(original["finishedAt"].replace("Z", "+00:00"))).total_seconds()
        self.assertGreaterEqual(delay, 60)

    # Covers: BDD-JRN-009-02 BDD-JRN-009-05
    def test_bdd_jrn_009_02_05_default_retries_follow_all_saved_delays(self) -> None:
        """Given a persistently failing run; when default retries apply; then three delays are used."""
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", "raise SystemExit(9)"],
        )
        original = self.wait_occurrence(self.run_now(schedule["id"])["id"])
        self.assertEqual(original["status"], "failed")
        previous = original
        expected_delays = [60, 300, 900]

        for attempt, expected_delay in enumerate(expected_delays, start=2):
            deadline = time.monotonic() + 5
            retry = None
            while time.monotonic() < deadline:
                items = self.json(
                    "GET", f"/schedules/{schedule['id']}/occurrences?limit=100"
                )["items"]
                retry = next(
                    (item for item in items if item.get("retryOf") == previous["id"]), None
                )
                if retry:
                    break
                time.sleep(0.05)
            self.assertIsNotNone(retry, f"attempt {attempt} was not queued")
            self.assertEqual(retry["attempt"], attempt)
            scheduled = datetime.fromisoformat(retry["scheduledFor"].replace("Z", "+00:00"))
            finished = datetime.fromisoformat(previous["finishedAt"].replace("Z", "+00:00"))
            self.assertGreaterEqual((scheduled - finished).total_seconds(), expected_delay)

            # Advance only this isolated test database so the next configured delay
            # can be exercised without a 21 minute wall-clock wait.
            with sqlite3.connect(self.db) as database:
                database.execute(
                    "UPDATE occurrences SET scheduled_for=? WHERE id=?",
                    (int(time.time() * 1_000_000) - 1, retry["id"]),
                )
            previous = self.wait_occurrence(retry["id"])
            self.assertEqual(previous["status"], "failed")

        items = self.json("GET", f"/schedules/{schedule['id']}/occurrences?limit=100")["items"]
        self.assertFalse(any(item.get("retryOf") == previous["id"] for item in items))

    # Covers: BDD-JRN-014-04 BDD-JRN-014-06
    def test_bdd_jrn_014_04_06_cli_cancel_and_invalid_update(self) -> None:
        """Given a CLI-managed schedule; when invalid update and cancellation are requested; then state is correct."""
        schedule = self.make_schedule()
        invalid = subprocess.run(
            [str(CLI), "--server", self.base, "update", schedule["id"], '{"missedPolicy":"invalid"}'],
            capture_output=True, text=True, timeout=10,
        )
        self.assertNotEqual(invalid.returncode, 0)
        self.assertEqual(self.json("GET", f"/schedules/{schedule['id']}")["missedPolicy"], "skip")
        cancelled = subprocess.run(
            [str(CLI), "--server", self.base, "cancel", schedule["id"]],
            check=True, capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(json.loads(cancelled.stdout)["status"], "cancelled")

    # Covers: BDD-JRN-015-05
    def test_bdd_jrn_015_05_cli_requires_confirmation_for_interrupted_retry(self) -> None:
        """Given an interrupted occurrence; when CLI retries; then explicit confirmation is required."""
        schedule = self.make_schedule()
        occurrence = self.wait_occurrence(self.run_now(schedule["id"])["id"])
        started = int(time.time() * 1_000_000)
        with sqlite3.connect(self.db) as database:
            database.execute(
                "UPDATE occurrences SET status='running', started_at=?, finished_at=NULL WHERE id=?",
                (started, occurrence["id"]),
            )
        self.stop_daemon()
        self.start_daemon()
        interrupted = self.json("GET", f"/occurrences/{occurrence['id']}")
        self.assertEqual(interrupted["status"], "interrupted")
        command = [str(CLI), "--server", self.base, "retry", occurrence["id"]]
        denied = subprocess.run(command, capture_output=True, text=True, timeout=10)
        self.assertNotEqual(denied.returncode, 0)
        self.assertEqual(denied.stdout, "")
        retried = subprocess.run(
            [*command, "--confirm"], check=True, capture_output=True, text=True, timeout=10
        )
        retry = json.loads(retried.stdout)
        self.assertEqual(retry["retryOf"], occurrence["id"])
        self.assertEqual(self.wait_occurrence(retry["id"])["status"], "succeeded")

    # Covers: BDD-JRN-013-01 BDD-JRN-014-01 BDD-JRN-014-02 BDD-JRN-014-03 BDD-JRN-014-05 BDD-JRN-015-01 BDD-JRN-015-02
    def test_bdd_jrn_013_01_014_01_02_03_05_015_01_02_cli_create_manage_run_and_capture(self) -> None:
        """Given local daemon; when CLI creates/edits/runs; then results match the API."""
        code = "import os; print(os.environ['COMMAND_SCHEDULER_PARAMS_JSON'])"
        result = subprocess.run(
            [str(CLI), "--server", self.base, "create", sys.executable,
             "--arg=-c", "--arg", code, "--cron", "0 9 * * 1-5",
             "--trigger-on-parameters", "--trigger-behavior", "coexist"],
            check=True, capture_output=True, text=True, timeout=10,
        )
        created = json.loads(result.stdout)
        listed = subprocess.run(
            [str(CLI), "--server", self.base, "schedules"],
            check=True, capture_output=True, text=True, timeout=10,
        )
        self.assertIn(created["id"], [item["id"] for item in json.loads(listed.stdout)["items"]])
        updated = subprocess.run(
            [str(CLI), "--server", self.base, "update", created["id"], '{"missedPolicy":"sequence"}'],
            check=True, capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(json.loads(updated.stdout)["missedPolicy"], "sequence")
        paused = subprocess.run(
            [str(CLI), "--server", self.base, "pause", created["id"]],
            check=True, capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(json.loads(paused.stdout)["status"], "paused")
        resumed = subprocess.run(
            [str(CLI), "--server", self.base, "resume", created["id"]],
            check=True, capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(json.loads(resumed.stdout)["status"], "active")
        parameters = subprocess.run(
            [str(CLI), "--server", self.base, "parameters", created["id"], '{"from":"cli"}'],
            check=True, capture_output=True, text=True, timeout=10,
        )
        update_result = json.loads(parameters.stdout)
        self.assertTrue(update_result.get("occurrenceId"))
        triggered = self.wait_occurrence(update_result["occurrenceId"])
        self.assertIn('{"from":"cli"}', triggered["stdout"])
        started = subprocess.run(
            [str(CLI), "--server", self.base, "run", created["id"]],
            check=True, capture_output=True, text=True, timeout=10,
        )
        occurrence = json.loads(started.stdout)
        final = self.wait_occurrence(occurrence["id"])
        self.assertEqual(final["status"], "succeeded")
        self.assertEqual(final["stdout"].replace("\r\n", "\n"), '{"from":"cli"}\n')

    # Covers: BDD-JRN-013-03
    def test_bdd_jrn_013_03_cli_does_not_confirm_creation_without_daemon(self) -> None:
        """Given an unavailable daemon; when CLI creation is attempted; then it reports failure without success output."""
        unavailable = f"http://127.0.0.1:{free_port()}"
        result = subprocess.run(
            [str(CLI), "--server", unavailable, "create", "/bin/echo", "--arg", "hello", "--at", timestamp_after(3600)],
            capture_output=True, text=True, timeout=10,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn("error:", result.stderr)

    # Covers: BDD-JRN-013-02
    def test_bdd_jrn_013_02_cli_rejects_invalid_schedule_without_creating_it(self) -> None:
        """Given an invalid schedule; when CLI creation runs; then it reports an error and stores nothing."""
        before = self.json("GET", "/schedules?limit=100")["items"]
        result = subprocess.run(
            [str(CLI), "--server", self.base, "create", sys.executable, "--arg", "-c", "--cron", "not-a-cron"],
            capture_output=True, text=True, timeout=10,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn("error:", result.stderr)
        after = self.json("GET", "/schedules?limit=100")["items"]
        self.assertEqual([item["id"] for item in after], [item["id"] for item in before])

    # Covers: BDD-JRN-015-03
    def test_bdd_jrn_015_03_cancel_child_through_cli(self) -> None:
        """Given a running occurrence; when CLI cancels it; then terminal state is observable."""
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", "import time; time.sleep(30)"],
        )
        started = subprocess.run(
            [str(CLI), "--server", self.base, "run", schedule["id"]],
            check=True, capture_output=True, text=True, timeout=10,
        )
        occurrence = json.loads(started.stdout)
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            current = self.json("GET", f"/occurrences/{occurrence['id']}")
            if current["status"] == "running":
                break
            time.sleep(0.05)
        else:
            self.fail("child did not start")
        subprocess.run(
            [str(CLI), "--server", self.base, "cancel-run", occurrence["id"]],
            check=True, capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(self.wait_occurrence(occurrence["id"])["status"], "cancelled")

    # Covers: BDD-JRN-015-04
    def test_bdd_jrn_015_04_cli_retries_failed_occurrence(self) -> None:
        """Given a failed occurrence; when CLI retries; then a separate retry is created."""
        schedule = self.make_schedule(
            program=sys.executable,
            args=["-c", "raise SystemExit(7)"],
            retryPolicy={"maxRetries": 0, "delaysSeconds": []},
        )
        started = subprocess.run(
            [str(CLI), "--server", self.base, "run", schedule["id"]],
            check=True, capture_output=True, text=True, timeout=10,
        )
        failed = self.wait_occurrence(json.loads(started.stdout)["id"])
        self.assertEqual(failed["status"], "failed")
        retried = subprocess.run(
            [str(CLI), "--server", self.base, "retry", failed["id"]],
            check=True, capture_output=True, text=True, timeout=10,
        )
        retry = json.loads(retried.stdout)
        self.assertEqual(retry["retryOf"], failed["id"])
        self.assertEqual(self.wait_occurrence(retry["id"])["status"], "failed")


if __name__ == "__main__":
    unittest.main()
