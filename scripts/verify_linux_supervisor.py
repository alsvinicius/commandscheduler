#!/usr/bin/env python3
"""Verify systemd restarts the daemon and recovers a due schedule on Linux."""

from __future__ import annotations

import json
import os
import socket
import subprocess
import tempfile
import time
import urllib.request
import uuid
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DAEMON = ROOT / "target/release/command-scheduler"
WEB = ROOT / "ui/dist"


def free_port() -> int:
    with socket.socket() as server:
        server.bind(("127.0.0.1", 0))
        return server.getsockname()[1]


def http(base: str, path: str, payload: object | None = None) -> bytes:
    body = None if payload is None else json.dumps(payload).encode()
    request = urllib.request.Request(
        base + path,
        data=body,
        headers={"Content-Type": "application/json"} if body is not None else {},
        method="POST" if body is not None else "GET",
    )
    with urllib.request.urlopen(request, timeout=3) as response:
        return response.read()


def property_value(unit: str, key: str) -> str:
    return subprocess.check_output(
        ["systemctl", "--user", "show", unit, f"--property={key}", "--value"],
        text=True,
        timeout=5,
    ).strip()


def wait_for(predicate, timeout: float, description: str) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(0.1)
    raise TimeoutError(f"timed out waiting for {description}")


def main() -> int:
    if not sys_platform_linux():
        raise RuntimeError("this verifier requires Linux")
    if not DAEMON.is_file() or not WEB.joinpath("index.html").is_file():
        raise RuntimeError("build release binaries and the UI first; see linux-service.md")

    unit = f"command-scheduler-supervisor-check-{uuid.uuid4().hex[:12]}"
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    with tempfile.TemporaryDirectory(prefix="command-scheduler-supervisor-") as temporary:
        command = [
            "systemd-run", "--user", f"--unit={unit}", "--collect",
            "--property=Restart=on-failure", "--property=RestartSec=2s",
            f"--working-directory={temporary}",
            f"--setenv=COMMAND_SCHEDULER_ADDR=127.0.0.1:{port}",
            f"--setenv=COMMAND_SCHEDULER_DB_PATH={temporary}/state.sqlite3",
            f"--setenv=COMMAND_SCHEDULER_WEB_DIR={WEB}",
            "--", str(DAEMON),
        ]
        try:
            subprocess.run(command, check=True, capture_output=True, text=True, timeout=15)
            wait_for(lambda: http_ok(base), 15, "initial daemon health")

            due = (datetime.now(timezone.utc) + timedelta(seconds=1.5)).isoformat()
            schedule = json.loads(http(base, "/api/v1/schedules", {
                "program": os.sys.executable,
                "args": ["-c", "print('recovered-after-supervisor-restart')"],
                "timing": {"type": "once", "at": due},
                "missedPolicy": "sequence",
            }))
            before = property_value(unit, "MainPID")
            subprocess.run(
                ["systemctl", "--user", "kill", "--kill-whom=main", "--signal=SIGKILL", unit],
                check=True,
                capture_output=True,
                text=True,
                timeout=5,
            )

            def restarted() -> bool:
                try:
                    return (
                        http_ok(base)
                        and property_value(unit, "MainPID") not in {"0", before}
                        and int(property_value(unit, "NRestarts")) >= 1
                    )
                except (OSError, subprocess.SubprocessError, ValueError):
                    return False

            wait_for(restarted, 20, "systemd restart after daemon failure")

            result: dict | None = None

            def recovered() -> bool:
                nonlocal result
                try:
                    page = json.loads(http(
                        base, f"/api/v1/schedules/{schedule['id']}/occurrences?limit=10"
                    ))
                    if not page["items"]:
                        return False
                    result = json.loads(http(base, f"/api/v1/occurrences/{page['items'][0]['id']}"))
                    return result["status"] in {"succeeded", "failed", "cancelled", "interrupted"}
                except (OSError, KeyError, ValueError):
                    return False

            wait_for(recovered, 15, "recovered scheduled occurrence")
            if (
                result is None
                or result["status"] != "succeeded"
                or "recovered-after-supervisor-restart" not in result["stdout"]
            ):
                raise RuntimeError(f"recovered occurrence has unexpected result: {result}")
            print(json.dumps({
                "unit": unit,
                "restarts": int(property_value(unit, "NRestarts")),
                "pid_changed": property_value(unit, "MainPID") != before,
                "schedule_recovered": True,
                "occurrence_source": result["source"],
                "occurrence_status": result["status"],
            }))
        finally:
            subprocess.run(["systemctl", "--user", "stop", unit], capture_output=True, timeout=10)
            subprocess.run(["systemctl", "--user", "reset-failed", unit], capture_output=True, timeout=5)
    return 0


def http_ok(base: str) -> bool:
    try:
        return http(base, "/health") == b"ok"
    except OSError:
        return False


def sys_platform_linux() -> bool:
    return os.name == "posix" and Path("/proc/sys/kernel/ostype").exists()


if __name__ == "__main__":
    raise SystemExit(main())
