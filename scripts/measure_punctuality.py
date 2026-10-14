#!/usr/bin/env python3
"""Measure scheduled start latency using an isolated daemon and SQLite file."""

# Covers: BDD-JRN-001-14

from __future__ import annotations

import argparse
import json
import math
import os
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
import uuid
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXE = ".exe" if os.name == "nt" else ""
DAEMON_CANDIDATES = (
    ROOT / "target/release" / f"command-scheduler{EXE}",
    ROOT / "target/debug" / f"command-scheduler{EXE}",
)


def request_json(base: str, path: str, payload: object | None = None) -> dict:
    data = None if payload is None else json.dumps(payload).encode()
    request = urllib.request.Request(
        base + path,
        data=data,
        headers={"Content-Type": "application/json"} if data is not None else {},
        method="POST" if data is not None else "GET",
    )
    with urllib.request.urlopen(request, timeout=3) as response:
        return json.load(response)


def percentile(values: list[float], percent: float) -> float:
    ordered = sorted(values)
    return ordered[max(0, math.ceil(percent * len(ordered)) - 1)]


def choose_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def wait_health(base: str, process: subprocess.Popen[bytes]) -> None:
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"daemon exited during startup with status {process.returncode}")
        try:
            with urllib.request.urlopen(base + "/health", timeout=1) as response:
                if response.read() == b"ok":
                    return
        except (OSError, urllib.error.URLError):
            time.sleep(0.1)
    raise TimeoutError("daemon did not become healthy within 15 seconds")


def run(samples: int, spacing_ms: int) -> dict:
    daemon = next((candidate for candidate in DAEMON_CANDIDATES if candidate.is_file()), None)
    ui = ROOT / "ui/dist/index.html"
    if daemon is None or not ui.is_file():
        raise RuntimeError("build the Rust binaries and UI first; see the punctuality guide")

    with tempfile.TemporaryDirectory(prefix="command-scheduler-punctuality-") as temporary:
        port = choose_port()
        base = f"http://127.0.0.1:{port}"
        env = os.environ.copy()
        env.update(
            {
                "COMMAND_SCHEDULER_ADDR": f"127.0.0.1:{port}",
                "COMMAND_SCHEDULER_DB_PATH": str(Path(temporary) / "measure.sqlite3"),
                "COMMAND_SCHEDULER_WEB_DIR": str(ROOT / "ui/dist"),
            }
        )
        process = subprocess.Popen(
            [str(daemon)],
            cwd=ROOT,
            env=env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        try:
            wait_health(base, process)
            batch = str(uuid.uuid4())
            first_due = datetime.now(timezone.utc) + timedelta(seconds=5)
            schedule_ids: list[str] = []
            for index in range(samples):
                due = first_due + timedelta(milliseconds=spacing_ms * index)
                schedule = request_json(
                    base,
                    "/api/v1/schedules",
                    {
                        "name": f"punctuality-{batch}-{index}",
                        "program": sys.executable,
                        "args": ["-c", "pass"],
                        "timing": {"type": "once", "at": due.isoformat()},
                        "missedPolicy": "skip",
                        "retryPolicy": {"maxRetries": 0, "delaysSeconds": []},
                    },
                )
                schedule_ids.append(schedule["id"])

            latest_due = first_due + timedelta(milliseconds=spacing_ms * (samples - 1))
            deadline = latest_due.timestamp() + 35
            starts: dict[str, float] = {}
            statuses: dict[str, str] = {}
            while time.time() < deadline and len(statuses) < samples:
                for schedule_id in schedule_ids:
                    if schedule_id in statuses:
                        continue
                    page = request_json(
                        base,
                        f"/api/v1/schedules/{schedule_id}/occurrences?limit=1",
                    )
                    if not page["items"]:
                        continue
                    occurrence = page["items"][0]
                    status = occurrence["status"]
                    if status in {"succeeded", "failed", "interrupted", "cancelled"}:
                        statuses[schedule_id] = status
                        if occurrence.get("startedAt") and occurrence.get("scheduledFor"):
                            started = datetime.fromisoformat(occurrence["startedAt"])
                            scheduled = datetime.fromisoformat(occurrence["scheduledFor"])
                            starts[schedule_id] = (started - scheduled).total_seconds()
                if len(statuses) < samples:
                    time.sleep(0.2)

            latencies = list(starts.values())
            failures = sum(status != "succeeded" for status in statuses.values())
            try:
                load = os.getloadavg()
            except (AttributeError, OSError):
                load = None
            result = {
                "platform": sys.platform,
                "samples_requested": samples,
                "samples_started": len(latencies),
                "occurrences_terminal": len(statuses),
                "occurrences_missing_or_unfinished": samples - len(statuses),
                "failed_or_cancelled": failures,
                "spacing_ms": spacing_ms,
                "latency_seconds": {
                    "p50": percentile(latencies, 0.50) if latencies else None,
                    "p95": percentile(latencies, 0.95) if latencies else None,
                    "p99": percentile(latencies, 0.99) if latencies else None,
                    "max": max(latencies) if latencies else None,
                },
                "load_average_at_report": load,
                "warning": "Record host load and conditions; this result only describes this run.",
            }
            if len(latencies) != samples or failures or result["latency_seconds"]["p95"] > 5 or result["latency_seconds"]["p99"] > 30:
                result["passed"] = False
            else:
                result["passed"] = True
            return result
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--samples", type=int, default=100)
    parser.add_argument("--spacing-ms", type=int, default=250)
    options = parser.parse_args()
    if options.samples < 100:
        parser.error("--samples must be at least 100 to report a meaningful p99")
    if options.spacing_ms < 100:
        parser.error("--spacing-ms must be at least 100 to keep this an idle-host check")
    try:
        result = run(options.samples, options.spacing_ms)
    except Exception as error:  # pragma: no cover - command line boundary
        print(f"measurement failed: {error}", file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2))
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
