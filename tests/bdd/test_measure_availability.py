"""Unit checks for availability interval classification and monthly reports."""

import json
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path

from scripts.measure_availability import report


class AvailabilityReportTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.log = Path(self.temporary.name) / "samples.jsonl"
        self.start = 1_800_000_000.0
        self.end = self.start + 10

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def write_rows(self, rows: list[dict]) -> None:
        with self.log.open("w", encoding="utf-8") as output:
            for row in rows:
                output.write(json.dumps(row) + "\n")

    def row(
        self,
        offset: int,
        mono: float,
        *,
        healthy: bool = True,
        boot: str = "boot-a",
        interval: float = 10,
        boottime: float | None = None,
        collector: str = "collector-a",
    ) -> dict:
        stamp = datetime.fromtimestamp(self.start + offset, timezone.utc).isoformat()
        return {
            "timestamp_utc": stamp,
            "monotonic_seconds": mono,
            "boottime_seconds": mono if boottime is None else boottime,
            "boot_id": boot,
            "collector_id": collector,
            "healthy": healthy,
            "interval_seconds": interval,
        }

    def get_report(self) -> dict:
        return report(self.log, self.start, self.end)

    def test_counts_healthy_interval(self) -> None:
        self.write_rows([self.row(0, 0), self.row(10, 10)])
        result = self.get_report()
        self.assertTrue(result["complete"])
        self.assertFalse(result["full_utc_calendar_month"])
        self.assertFalse(result["target_met"])
        self.assertEqual(result["healthy_seconds"], 10)

    def test_counts_failed_health_probe_as_unavailable(self) -> None:
        self.write_rows([self.row(0, 0), self.row(10, 10, healthy=False)])
        result = self.get_report()
        self.assertEqual(result["availability_percent"], 0)
        self.assertEqual(result["unavailable_seconds"], 10)
        self.assertFalse(result["target_met"])

    def test_calculates_weighted_availability_across_healthy_and_failed_intervals(self) -> None:
        self.end = self.start + 20
        self.write_rows([self.row(0, 0), self.row(10, 10), self.row(20, 20, healthy=False)])
        result = self.get_report()
        self.assertEqual(result["availability_percent"], 50)
        self.assertEqual(result["healthy_seconds"], 10)
        self.assertEqual(result["unavailable_seconds"], 10)

    def test_excludes_suspend_time_from_denominator(self) -> None:
        self.write_rows([self.row(0, 0, boottime=0), self.row(30, 10, boottime=30)])
        result = self.get_report()
        self.assertTrue(result["complete"])
        self.assertEqual(result["eligible_seconds"], 3.333)
        self.assertEqual(result["excluded_sleep_or_known_offline_seconds"], 6.667)

    def test_marks_unobserved_gap_unknown(self) -> None:
        self.write_rows([self.row(0, 0), self.row(30, 30)])
        result = self.get_report()
        self.assertFalse(result["complete"])
        self.assertFalse(result["target_met"])
        self.assertEqual(result["unknown_seconds"], 10)

    def test_marks_wall_clock_adjustment_unknown_instead_of_sleep(self) -> None:
        self.write_rows([self.row(0, 0), self.row(30, 10, boottime=10)])
        result = self.get_report()
        self.assertFalse(result["complete"])
        self.assertEqual(result["unknown_seconds"], 10)
        self.assertEqual(result["excluded_sleep_or_known_offline_seconds"], 0)

    def test_tolerates_clock_reads_taken_a_few_microseconds_apart(self) -> None:
        self.write_rows([self.row(0, 0, boottime=0), self.row(10, 10, boottime=9.99999)])
        result = self.get_report()
        self.assertTrue(result["complete"])
        self.assertEqual(result["availability_percent"], 100)

    def test_marks_collector_restart_as_unknown_even_when_gap_is_short(self) -> None:
        self.write_rows([self.row(0, 0), self.row(10, 10, collector="collector-b")])
        result = self.get_report()
        self.assertFalse(result["complete"])
        self.assertEqual(result["unknown_seconds"], 10)

    def test_marks_boot_transition_unknown(self) -> None:
        self.write_rows([self.row(0, 0), self.row(10, 1, boot="boot-b")])
        result = self.get_report()
        self.assertFalse(result["complete"])
        self.assertEqual(result["unknown_seconds"], 10)

    def test_empty_log_is_unknown_only_once(self) -> None:
        self.write_rows([])
        result = self.get_report()
        self.assertEqual(result["unknown_seconds"], 10)
        self.assertFalse(result["complete"])

    def test_marks_target_only_for_a_complete_calendar_month(self) -> None:
        self.start = datetime(2026, 10, 1, tzinfo=timezone.utc).timestamp()
        self.end = datetime(2026, 11, 1, tzinfo=timezone.utc).timestamp()
        month = self.end - self.start
        self.write_rows([self.row(0, 0, interval=month), self.row(int(month), month, interval=month)])
        result = self.get_report()
        self.assertTrue(result["complete"])
        self.assertTrue(result["full_utc_calendar_month"])
        self.assertTrue(result["target_met"])

    def test_excludes_a_boot_gap_only_when_caller_supplies_verified_range(self) -> None:
        self.end = self.start + 20
        self.write_rows([self.row(0, 0), self.row(10, 10), self.row(20, 1, boot="boot-b")])
        result = report(
            self.log,
            self.start,
            self.end,
            [(self.start + 10, self.start + 20, "fixture: verified power-off interval")],
        )
        self.assertTrue(result["complete"])
        self.assertEqual(result["eligible_seconds"], 10)
        self.assertEqual(result["excluded_sleep_or_known_offline_seconds"], 10)
        self.assertEqual(result["known_offline_ranges_utc"][0]["evidence"], "fixture: verified power-off interval")
        self.assertFalse(result["target_met"])


if __name__ == "__main__":
    unittest.main()
