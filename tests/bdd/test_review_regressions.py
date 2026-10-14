"""Reproductions of the security, concurrency and execution review findings."""

import concurrent.futures
import http.client
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import sys
import time
import tomllib
import unittest
import urllib.parse
import uuid

import test_journey_api as journeys

DAEMON, ROOT = journeys.DAEMON, journeys.ROOT
free_port, timestamp_after = journeys.free_port, journeys.timestamp_after


class ReviewRegressions(unittest.TestCase):
    setUp = journeys.LocalDaemon.setUp
    tearDown = journeys.LocalDaemon.tearDown
    start_daemon = journeys.LocalDaemon.start_daemon
    stop_daemon = journeys.LocalDaemon.stop_daemon
    http = journeys.LocalDaemon.http
    json = journeys.LocalDaemon.json
    make_schedule = journeys.LocalDaemon.make_schedule
    wait_occurrence = journeys.LocalDaemon.wait_occurrence
    run_now = journeys.LocalDaemon.run_now

    def request(self, method, path, body=None, headers=None, timeout=5):
        connection = http.client.HTTPConnection("127.0.0.1", self.port, timeout=timeout)
        try:
            connection.request(method, "/api/v1" + path,
                               None if body is None else json.dumps(body),
                               {**({"Content-Type": "application/json"} if body is not None else {}), **(headers or {})})
            response = connection.getresponse()
            raw = response.read()
            return response.status, json.loads(raw) if "json" in response.getheader("Content-Type", "") else {"detail": raw.decode()}
        finally:
            connection.close()

    def wait_status(self, occurrence_id, status, timeout=3):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            item = self.json("GET", f"/occurrences/{occurrence_id}")
            if item["status"] == status:
                return item
            time.sleep(.02)
        self.fail(f"{occurrence_id} did not reach {status}: {item}")

    def test_host_and_origin_reject_rebinding_and_allow_local_proxy(self):
        marker = Path(self.temp.name) / "must-not-execute"
        body = {"program": sys.executable,
                "args": ["-c", f"open({str(marker)!r},'w').close()"]}
        for headers in ({"Host": "attacker.example", "Origin": "http://attacker.example"},
                        {"Host": "attacker.example"}):
            status, _ = self.request("POST", "/commands/test", body, headers)
            self.assertEqual(status, 403)
            self.assertFalse(marker.exists())
        for host in (f"127.0.0.1:{self.port}", "localhost:5173", "127.0.0.1:5173", "[::1]:5173"):
            status, result = self.request("POST", "/commands/test",
                                          {"program": sys.executable, "args": ["-c", "print('local')"]},
                                          {"Host": host, "Origin": f"http://{host}"})
            self.assertEqual(status, 200, result)
            self.assertEqual(result["stdout"].strip(), "local")

    def test_concurrent_rejected_parameter_updates_release_connections(self):
        schedule = self.make_schedule()
        self.json("POST", f"/schedules/{schedule['id']}/cancel")
        started = time.monotonic()
        for schedule_id, expected in ((schedule["id"], 409), (str(uuid.uuid4()), 404)):
            with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
                statuses = list(executor.map(
                    lambda n: self.request("PUT", f"/schedules/{schedule_id}/parameters",
                                           {"n": n})[0], range(20)))
            self.assertEqual(statuses, [expected] * 20)
        self.assertLess(time.monotonic() - started, 5)

    def test_second_daemon_cannot_recover_an_active_database(self):
        schedule = self.make_schedule(args=["-c", "import time; time.sleep(4)"])
        occurrence = self.run_now(schedule["id"])
        self.wait_status(occurrence["id"], "running")
        for port in (self.port, free_port()):
            env = {**os.environ, "COMMAND_SCHEDULER_DB_PATH": str(self.db),
                   "COMMAND_SCHEDULER_ADDR": f"127.0.0.1:{port}"}
            second = subprocess.run([str(DAEMON)], cwd=ROOT, env=env,
                                    capture_output=True, timeout=2)
            self.assertNotEqual(second.returncode, 0)
            self.assertIn(b"already owns", second.stderr)
            self.assertEqual(self.json("GET", f"/occurrences/{occurrence['id']}")["status"], "running")

    @unittest.skipUnless(os.name == "posix", "symlink regression on Unix")
    def test_database_symlink_uses_the_same_daemon_lock(self):
        alias = Path(self.temp.name) / "alias.sqlite3"
        alias.symlink_to(self.db)
        env = {**os.environ, "COMMAND_SCHEDULER_DB_PATH": str(alias),
               "COMMAND_SCHEDULER_ADDR": f"127.0.0.1:{free_port()}"}
        second = subprocess.Popen([str(DAEMON)], cwd=ROOT, env=env,
                                  stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        try:
            try:
                _, stderr = second.communicate(timeout=2)
            except subprocess.TimeoutExpired:
                self.fail("a database alias bypassed the daemon lock")
            self.assertNotEqual(second.returncode, 0)
            self.assertIn(b"already owns", stderr)
        finally:
            if second.poll() is None:
                second.terminate()
                second.communicate(timeout=5)

    def test_license_and_windows_continuous_task_are_declared(self):
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
        self.assertEqual(manifest["package"]["license"], "MIT")
        license_text = (ROOT / "LICENSE").read_text()
        self.assertIn("Permission is hereby granted", license_text)
        self.assertIn("THE SOFTWARE IS PROVIDED", license_text)
        installer = (ROOT / "packaging/windows/install-service.ps1").read_text()
        self.assertIn("-ExecutionTimeLimit ([TimeSpan]::Zero)", installer)
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        self.assertIn("Validate Windows continuous task settings", workflow)
        self.assertIn("./tests/windows/task-settings.ps1", workflow)

    def test_cancelled_schedule_does_not_enqueue_or_execute_automatic_retries(self):
        schedule = self.make_schedule(args=["-c", "import time; time.sleep(.5); raise SystemExit(7)"],
                                      retryPolicy={"maxRetries": 3, "delaysSeconds": [1]})
        first = self.run_now(schedule["id"])
        self.wait_status(first["id"], "running")
        self.json("POST", f"/schedules/{schedule['id']}/cancel")
        self.wait_occurrence(first["id"])
        status, _ = self.request("POST", f"/occurrences/{first['id']}/retry")
        self.assertEqual(status, 409)
        with sqlite3.connect(self.db) as db:
            self.assertEqual(db.execute("SELECT count(*) FROM occurrences WHERE schedule_id=? AND source='retry'",
                                        (schedule["id"],)).fetchone()[0], 0)
        other = self.make_schedule(args=["-c", "raise SystemExit(7)"],
                                   retryPolicy={"maxRetries": 3, "delaysSeconds": [60]})
        self.wait_occurrence(self.run_now(other["id"])["id"])
        self.json("POST", f"/schedules/{other['id']}/cancel")
        with sqlite3.connect(self.db) as db:
            db.execute("UPDATE occurrences SET scheduled_for=0 WHERE schedule_id=? AND source='retry'", (other["id"],))
        time.sleep(.5)
        with sqlite3.connect(self.db) as db:
            retries = db.execute("SELECT status,started_at FROM occurrences WHERE schedule_id=? AND source='retry'",
                                 (other["id"],)).fetchall()
        self.assertEqual(len(retries), 1)
        self.assertIsNone(retries[0][1])
        self.assertEqual(retries[0][0], "cancelled")

    def test_sequence_runs_in_order_without_blocking_another_schedule(self):
        schedule = self.make_schedule(args=["-c", "import time; time.sleep(.5)"], missedPolicy="sequence")
        other = self.make_schedule()
        now = time.time_ns() // 1000
        ids = [str(uuid.uuid4()) for _ in range(3)]
        with sqlite3.connect(self.db) as db:
            for index, occurrence_id in enumerate(ids):
                db.execute("INSERT INTO occurrences(id,schedule_id,status,source,created_at,scheduled_for) VALUES(?,?,'queued','recovery',?,?)",
                           (occurrence_id, schedule["id"], now + index, now - (3-index)*60_000_000))
        self.wait_status(ids[0], "running")
        independent = self.run_now(other["id"])
        independent_result = self.wait_status(independent["id"], "succeeded")
        for occurrence_id in ids:
            self.wait_occurrence(occurrence_id)
        with sqlite3.connect(self.db) as db:
            rows = db.execute("SELECT id,started_at,finished_at FROM occurrences WHERE schedule_id=? ORDER BY scheduled_for",
                              (schedule["id"],)).fetchall()
        self.assertEqual([r[0] for r in rows], ids)
        self.assertTrue(all(a[2] <= b[1] for a, b in zip(rows, rows[1:])), rows)
        from datetime import datetime
        independent_started = int(datetime.fromisoformat(independent_result["startedAt"]).timestamp()*1_000_000)
        self.assertLess(independent_started, rows[-1][2], "recovery must not impose a global concurrency limit")

    def descendant_script(self, parent_sleep=0):
        pidfile = Path(self.temp.name) / "descendant.pid"
        script = ("import subprocess,sys,time; "
                  "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(90)']); "
                  f"open({str(pidfile)!r},'w').write(str(p.pid)); "
                  "print('retained-prefix',flush=True); "
                  f"time.sleep({parent_sleep})")
        return pidfile, script

    def kill_descendant(self, pidfile):
        if os.name == "posix" and pidfile.exists():
            try:
                os.kill(int(pidfile.read_text()), signal.SIGKILL)
            except ProcessLookupError:
                pass

    def assert_descendant_stopped(self, pidfile):
        if os.name != "posix":
            return
        pid = pidfile.read_text()
        deadline = time.monotonic() + 1
        while time.monotonic() < deadline:
            state = subprocess.run(["ps", "-o", "stat=", "-p", pid], capture_output=True, text=True).stdout.strip()
            if not state or state.startswith("Z"):
                return
            time.sleep(.02)
        self.fail(f"descendant {pid} is still alive after completion: {state}")

    def test_command_deadline_includes_descendant_output_and_retains_prefix(self):
        pidfile, script = self.descendant_script(parent_sleep=2)
        try:
            before = time.monotonic()
            status, result = self.request("POST", "/commands/test",
                                          {"program": sys.executable, "args": ["-c", script]}, timeout=35)
            elapsed = time.monotonic() - before
            self.assertEqual(status, 200, result)
            self.assertLess(elapsed, 31, f"deadline took {elapsed:.2f}s")
            self.assertTrue(result["timedOut"])
            self.assertIn("retained-prefix", result["stdout"])
            self.assert_descendant_stopped(pidfile)
        finally:
            self.kill_descendant(pidfile)

    def test_cancellation_remains_active_while_descendants_hold_pipes(self):
        pidfile, script = self.descendant_script()
        try:
            schedule = self.make_schedule(args=["-c", script], retryPolicy={"maxRetries": 0})
            occurrence = self.run_now(schedule["id"])
            deadline = time.monotonic() + 3
            while not pidfile.exists() and time.monotonic() < deadline:
                time.sleep(.02)
            self.assertTrue(pidfile.exists())
            time.sleep(.2)  # parent has exited; inherited pipes remain open
            status, _ = self.request("POST", f"/occurrences/{occurrence['id']}/cancel")
            self.assertEqual(status, 202)
            result = self.wait_status(occurrence["id"], "cancelled", timeout=2)
            self.assertIn("retained-prefix", result["stdout"])
            self.assert_descendant_stopped(pidfile)
        finally:
            self.kill_descendant(pidfile)

    def test_timing_change_resets_cursor_but_unrelated_edits_do_not(self):
        schedule = self.make_schedule(missedPolicy="sequence")
        old_cursor = time.time_ns() // 1000 - 180_000_000
        with sqlite3.connect(self.db) as db:
            db.execute("UPDATE schedules SET last_evaluated_at=? WHERE id=?", (old_cursor, schedule["id"]))
        self.json("PATCH", f"/schedules/{schedule['id']}", {"name": "renamed", "timing": schedule["timing"]})
        with sqlite3.connect(self.db) as db:
            self.assertEqual(db.execute("SELECT last_evaluated_at FROM schedules WHERE id=?", (schedule["id"],)).fetchone()[0], old_cursor)
        before = time.time_ns() // 1000
        result = self.json("PATCH", f"/schedules/{schedule['id']}", {"timing": {"type": "cron", "expression": "* * * * *"}})
        self.assertIn("id", result)
        with sqlite3.connect(self.db) as db:
            self.assertGreaterEqual(db.execute("SELECT last_evaluated_at FROM schedules WHERE id=?", (schedule["id"],)).fetchone()[0], before)
            self.assertEqual(db.execute("SELECT count(*) FROM occurrences WHERE schedule_id=? AND scheduled_for<?", (schedule["id"], before)).fetchone()[0], 0)

    def test_past_once_is_rejected_but_idempotent_replay_survives_due_time(self):
        status, _ = self.request("POST", "/schedules", {"program": sys.executable, "args": [],
                                  "timing": {"type": "once", "at": timestamp_after(-180)}, "missedPolicy": "sequence"})
        self.assertEqual(status, 400)
        body = {"program": sys.executable, "args": ["-c", "print('once')"],
                "timing": {"type": "once", "at": timestamp_after(.6)}, "missedPolicy": "sequence"}
        status, first = self.request("POST", "/schedules", body, {"Idempotency-Key": "once-replay"})
        self.assertEqual(status, 201, first)
        time.sleep(.8)
        status, replay = self.request("POST", "/schedules", body, {"Idempotency-Key": "once-replay"})
        self.assertEqual(status, 200, replay)
        self.assertEqual(replay["id"], first["id"])

    def test_agenda_selects_future_items_globally_and_keeps_history_capacity(self):
        minute = self.make_schedule(timing={"type": "cron", "expression": "* * * * *"})
        hour = self.make_schedule(timing={"type": "cron", "expression": "0 * * * *"})
        query = urllib.parse.urlencode({"from": timestamp_after(60), "to": timestamp_after(86400)})
        items = self.json("GET", f"/agenda?{query}")["items"]
        self.assertEqual(len(items), 500)
        self.assertIn(hour["id"], {item["scheduleId"] for item in items})
        self.assertEqual([item["at"] for item in items], sorted(item["at"] for item in items))
        now = time.time_ns() // 1000
        with sqlite3.connect(self.db) as db:
            for index in range(500):
                db.execute("INSERT INTO occurrences(id,schedule_id,status,source,created_at,finished_at) VALUES(?,?,'succeeded','manual',?,?)",
                           (str(uuid.uuid4()), minute["id"], now-index*1000, now-index*1000))
        query = urllib.parse.urlencode({"from": timestamp_after(-60), "to": timestamp_after(86400)})
        items = self.json("GET", f"/agenda?{query}")["items"]
        self.assertEqual(len(items), 1000)
        self.assertEqual(sum(item.get("occurrenceId") is not None for item in items), 500)

    def test_concurrent_parameter_updates_are_atomic_with_their_triggers(self):
        schedule = self.make_schedule(parameterUpdateTrigger={"enabled": True, "scheduleBehavior": "coexist"})
        with concurrent.futures.ThreadPoolExecutor(max_workers=50) as executor:
            results = list(executor.map(lambda index: self.request("PUT", f"/schedules/{schedule['id']}/parameters", {"version": index}), range(50)))
        self.assertTrue(all(status == 200 for status, _ in results), results)
        ids = {body["occurrenceId"] for _, body in results}
        self.assertEqual(len(ids), 50)
        with sqlite3.connect(self.db) as db:
            self.assertEqual(db.execute("SELECT count(*) FROM occurrences WHERE schedule_id=? AND source='parameter_update'", (schedule["id"],)).fetchone()[0], 50)

    def test_keyset_pages_do_not_repeat_or_skip_after_insert_and_removal(self):
        schedules = [self.make_schedule() for _ in range(4)]
        with sqlite3.connect(self.db) as db:
            db.execute("UPDATE schedules SET created_at=?", (time.time_ns() // 1000,))
        first = self.json("GET", "/schedules?limit=2")
        self.make_schedule()
        self.json("POST", f"/schedules/{first['items'][0]['id']}/cancel")
        second = self.json("GET", "/schedules?limit=2&cursor=" + first["nextCursor"])
        self.assertEqual({i["id"] for i in first["items"] + second["items"]}, {s["id"] for s in schedules})
        sid = schedules[0]["id"]
        now = time.time_ns() // 1000
        originals = [str(uuid.uuid4()) for _ in range(5)]
        with sqlite3.connect(self.db) as db:
            for oid in originals:
                db.execute("INSERT INTO occurrences(id,schedule_id,status,source,created_at,finished_at) VALUES(?,?,'succeeded','manual',?,?)", (oid, sid, now, now))
        for path in ("/occurrences", f"/schedules/{sid}/occurrences"):
            for mutation in ("insert", "remove_from_filter"):
                first = self.json("GET", path + "?limit=2&status=succeeded")
                with sqlite3.connect(self.db) as db:
                    if mutation == "insert":
                        db.execute("INSERT INTO occurrences(id,schedule_id,status,source,created_at,finished_at) VALUES(?,?,'succeeded','manual',?,?)", (str(uuid.uuid4()), sid, now+100, now+100))
                    else:
                        db.execute("UPDATE occurrences SET status='failed' WHERE id=?", (first["items"][0]["id"],))
                second = self.json("GET", path + "?limit=2&status=succeeded&cursor=" + first["nextCursor"])
                self.assertFalse({i["id"] for i in first["items"]} & {i["id"] for i in second["items"]})
                self.assertEqual([i["id"] for i in second["items"]], sorted(originals, reverse=True)[2:4])
                with sqlite3.connect(self.db) as db:
                    db.execute("DELETE FROM occurrences WHERE id NOT IN (" + ",".join("?" for _ in originals) + ")", originals)
                    db.execute("UPDATE occurrences SET status='succeeded' WHERE schedule_id=?", (sid,))

    def test_accepted_queue_cancellation_never_leaves_the_command_running(self):
        schedule = self.make_schedule(args=["-c", "import time; time.sleep(.5)"], retryPolicy={"maxRetries": 0})
        for _ in range(20):
            item = self.run_now(schedule["id"])
            status, result = self.request("POST", f"/occurrences/{item['id']}/cancel")
            self.assertIn(status, (202, 409), result)
            if status == 202:
                self.assertEqual(self.wait_occurrence(item["id"])["status"], "cancelled")
