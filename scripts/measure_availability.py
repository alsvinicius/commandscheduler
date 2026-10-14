#!/usr/bin/env python3
"""Collect and report Linux daemon health samples for monthly availability."""

from __future__ import annotations

import argparse
import json
import os
import signal
import sys
import threading
import time
import urllib.error
import urllib.request
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

CLOCK_READ_TOLERANCE_SECONDS = 0.05


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def boot_id() -> str:
    return Path("/proc/sys/kernel/random/boot_id").read_text().strip()


def probe(url: str, timeout: float) -> dict[str, Any]:
    started = time.monotonic()
    status: int | None = None
    error: str | None = None
    healthy = False
    try:
        request = urllib.request.Request(url, method="GET")
        with urllib.request.urlopen(request, timeout=timeout) as response:
            status = response.status
            healthy = status == 200 and response.read(16) == b"ok"
            if not healthy:
                error = "unexpected health response"
    except urllib.error.HTTPError as exc:
        status = exc.code
        error = f"HTTP {exc.code}"
    except (urllib.error.URLError, TimeoutError, OSError) as exc:
        error = str(exc.reason if isinstance(exc, urllib.error.URLError) else exc)[:300]
    finished = time.monotonic()
    boottime = time.clock_gettime(time.CLOCK_BOOTTIME)
    return {
        "timestamp_utc": utc_now(),
        "monotonic_seconds": finished,
        "boottime_seconds": boottime,
        "boot_id": boot_id(),
        "healthy": healthy,
        "http_status": status,
        "latency_ms": round((finished - started) * 1000, 3),
        "error": error,
    }


def monitor(log_path: Path, url: str, interval: float, timeout: float) -> None:
    if not sys.platform.startswith("linux"):
        raise RuntimeError("availability monitor currently supports Linux only")
    if interval <= 0 or timeout <= 0 or timeout >= interval:
        raise ValueError("require interval > timeout > 0")
    log_path.parent.mkdir(parents=True, exist_ok=True)
    stop = threading.Event()
    collector_id = uuid.uuid4().hex
    signal.signal(signal.SIGTERM, lambda *_: stop.set())
    signal.signal(signal.SIGINT, lambda *_: stop.set())
    with log_path.open("a", encoding="utf-8", buffering=1) as output:
        while not stop.is_set():
            row = probe(url, timeout)
            row["interval_seconds"] = interval
            row["collector_id"] = collector_id
            output.write(json.dumps(row, separators=(",", ":")) + "\n")
            output.flush()
            os.fsync(output.fileno())
            # Keep the cadence stable without counting time spent in the HTTP request twice.
            stop.wait(max(0.0, interval - row["latency_ms"] / 1000))


def parse_time(value: str) -> float:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        raise ValueError("report timestamps must include a UTC offset")
    return parsed.timestamp()


def read_log(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open(encoding="utf-8") as source:
        for number, line in enumerate(source, 1):
            if not line.strip():
                continue
            try:
                row = json.loads(line)
                row["_epoch"] = parse_time(row["timestamp_utc"])
                rows.append(row)
            except (json.JSONDecodeError, KeyError, TypeError, ValueError) as exc:
                raise ValueError(f"invalid sample on line {number}: {exc}") from exc
    rows.sort(key=lambda row: row["_epoch"])
    return rows


def normalize_exclusions(
    ranges: list[tuple[float, float, str]], start: float, end: float
) -> list[tuple[float, float, str]]:
    clipped: list[tuple[float, float, str]] = []
    for left, right, evidence in sorted(ranges):
        if right <= left or left < start or right > end or not evidence.strip():
            raise ValueError("known offline ranges must be ordered and inside the report window")
        if clipped and left < clipped[-1][1]:
            raise ValueError("known offline ranges must not overlap")
        clipped.append((left, right, evidence.strip()))
    return clipped


def unexcluded_parts(
    left: float, right: float, exclusions: list[tuple[float, float, str]]
) -> list[tuple[float, float]]:
    parts: list[tuple[float, float]] = []
    cursor = left
    for excluded_left, excluded_right, _ in exclusions:
        if excluded_right <= cursor:
            continue
        if excluded_left >= right:
            break
        if excluded_left > cursor:
            parts.append((cursor, min(right, excluded_left)))
        cursor = max(cursor, excluded_right)
        if cursor >= right:
            break
    if cursor < right:
        parts.append((cursor, right))
    return [(part_left, part_right) for part_left, part_right in parts if part_right > part_left]


def is_full_utc_month(start: float, end: float) -> bool:
    start_dt = datetime.fromtimestamp(start, timezone.utc)
    end_dt = datetime.fromtimestamp(end, timezone.utc)
    next_month = 1 if start_dt.month == 12 else start_dt.month + 1
    next_year = start_dt.year + (start_dt.month == 12)
    return (
        start_dt.day == 1
        and start_dt.hour == start_dt.minute == start_dt.second == start_dt.microsecond == 0
        and end_dt == datetime(next_year, next_month, 1, tzinfo=timezone.utc)
    )


def report(
    path: Path,
    start: float,
    end: float,
    known_offline: list[tuple[float, float, str]] | None = None,
) -> dict[str, Any]:
    if end <= start:
        raise ValueError("report end must be after start")
    exclusions = normalize_exclusions(known_offline or [], start, end)
    rows = read_log(path)
    eligible = healthy = unavailable = excluded = unknown = 0.0
    max_gap = 0.0
    pairs: list[tuple[dict[str, Any], dict[str, Any]]] = list(zip(rows, rows[1:]))

    # The observations must bracket the requested window. Unobserved edges stay unknown.
    if rows:
        leading_end = min(end, rows[0]["_epoch"])
        unknown += sum(right - left for left, right in unexcluded_parts(start, leading_end, exclusions))
        trailing_start = max(start, rows[-1]["_epoch"])
        unknown += sum(right - left for left, right in unexcluded_parts(trailing_start, end, exclusions))
    else:
        unknown += sum(right - left for left, right in unexcluded_parts(start, end, exclusions))

    for previous, current in pairs:
        left = max(start, previous["_epoch"])
        right = min(end, current["_epoch"])
        if right <= left:
            continue
        wall_delta = current["_epoch"] - previous["_epoch"]
        span = right - left
        max_gap = max(max_gap, wall_delta)
        parts = unexcluded_parts(left, right, exclusions)
        excluded += span - sum(part_right - part_left for part_left, part_right in parts)
        for part_left, part_right in parts:
            part_span = part_right - part_left
            if previous.get("collector_id") != current.get("collector_id"):
                unknown += part_span
                continue
            if previous.get("boot_id") != current.get("boot_id"):
                unknown += part_span
                continue
            mono_delta = float(current["monotonic_seconds"]) - float(previous["monotonic_seconds"])
            boottime_delta = float(current["boottime_seconds"]) - float(previous["boottime_seconds"])
            interval = float(current.get("interval_seconds", 10))
            if (
                mono_delta < 0
                or boottime_delta < mono_delta - CLOCK_READ_TOLERANCE_SECONDS
                or wall_delta <= 0
                or abs(wall_delta - boottime_delta) > 2
            ):
                unknown += part_span
                continue
            awake_fraction = min(1.0, mono_delta / boottime_delta) if boottime_delta > 0 else 0.0
            awake_part = part_span * awake_fraction
            excluded += part_span - awake_part
            if mono_delta > interval * 1.5:
                unknown += awake_part
                continue
            eligible += awake_part
            if current.get("healthy") is True:
                healthy += awake_part
            else:
                unavailable += awake_part

    pct = healthy / eligible * 100 if eligible else None
    full_month = is_full_utc_month(start, end)
    complete = unknown == 0 and eligible > 0
    return {
        "window_start_utc": datetime.fromtimestamp(start, timezone.utc).isoformat(),
        "window_end_utc": datetime.fromtimestamp(end, timezone.utc).isoformat(),
        "target_percent": 99.5,
        "availability_percent": round(pct, 5) if pct is not None else None,
        "eligible_seconds": round(eligible, 3),
        "healthy_seconds": round(healthy, 3),
        "unavailable_seconds": round(unavailable, 3),
        "excluded_sleep_or_known_offline_seconds": round(excluded, 3),
        "unknown_seconds": round(unknown, 3),
        "max_observation_gap_seconds": round(max_gap, 3),
        "samples": len(rows),
        "complete": complete,
        "full_utc_calendar_month": full_month,
        "known_offline_ranges_utc": [
            {
                "from": datetime.fromtimestamp(left, timezone.utc).isoformat(),
                "to": datetime.fromtimestamp(right, timezone.utc).isoformat(),
                "evidence": evidence,
            }
            for left, right, evidence in exclusions
        ],
        "target_met": bool(complete and full_month and pct is not None and pct >= 99.5),
        "source_log": str(path),
        "note": "A target result requires a complete, audited month; inspect raw samples and explain all excluded or unknown intervals.",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    collect = commands.add_parser("monitor", help="append health probes to a JSONL log")
    collect.add_argument("--log", type=Path, required=True)
    collect.add_argument("--url", default="http://127.0.0.1:4371/health")
    collect.add_argument("--interval-seconds", type=float, default=10)
    collect.add_argument("--timeout-seconds", type=float, default=2)
    summarize = commands.add_parser("report", help="summarize a UTC observation window")
    summarize.add_argument("--log", type=Path, required=True)
    summarize.add_argument("--from", dest="start", required=True, help="ISO timestamp with UTC offset")
    summarize.add_argument("--to", dest="end", required=True, help="ISO timestamp with UTC offset")
    summarize.add_argument(
        "--known-offline",
        action="append",
        nargs=3,
        metavar=("FROM", "TO", "EVIDENCE"),
        default=[],
        help="exclude a verified powered-off interval and record its evidence; may be repeated",
    )
    args = parser.parse_args()
    try:
        if args.command == "monitor":
            monitor(args.log, args.url, args.interval_seconds, args.timeout_seconds)
            return 0
        ranges = [(parse_time(left), parse_time(right), evidence) for left, right, evidence in args.known_offline]
        result = report(args.log, parse_time(args.start), parse_time(args.end), ranges)
        print(json.dumps(result, indent=2))
        return 0 if result["complete"] else 2
    except (OSError, ValueError, RuntimeError) as exc:
        parser.error(str(exc))
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
