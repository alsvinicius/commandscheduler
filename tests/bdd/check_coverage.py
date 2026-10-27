#!/usr/bin/env python3
"""Check that BDD test links reference real journey scenarios and report coverage."""

import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[2]
journeys = "\n".join(path.read_text(encoding="utf-8") for path in (root / "docs/product/journeys").glob("*.md"))
documented = set(re.findall(r"BDD-JRN-\d{3}-\d{2}", journeys))
test_files = [
    *(root / "tests/bdd").glob("test_*.py"),
    *(root / "tests/web").glob("*.spec.ts"),
    *(root / "src").rglob("*.rs"),
    *(root / "scripts").glob("measure_*.py"),
    *(root / "scripts").glob("verify_*.py"),
]
linked_lines = []
for path in test_files:
    linked_lines.extend(
        line for line in path.read_text(encoding="utf-8").splitlines()
        if re.match(r"\s*(?:#|//)\s*Covers:.*", line)
    )
covered = set(re.findall(r"BDD-JRN-\d{3}-\d{2}", "\n".join(linked_lines)))
unknown = covered - documented
if unknown:
    print("BDD IDs not found in journey docs:", ", ".join(sorted(unknown)), file=sys.stderr)
    raise SystemExit(1)
print(f"BDD coverage: {len(covered)}/{len(documented)} journey scenarios linked to executable tests.")
print(f"Pending automation: {len(documented - covered)} scenarios.")
