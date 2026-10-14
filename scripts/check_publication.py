#!/usr/bin/env python3
"""Check publication metadata, migration embedding and the Cargo file boundary."""
import json
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    assert manifest["package"]["license"] == "MIT", "Cargo license must match LICENSE"
    for file in ("package.json", "ui/package.json"):
        assert json.loads((ROOT / file).read_text())["license"] == "MIT", file
    for file in ("LICENSE", "SECURITY.md", "THIRD_PARTY_NOTICES.md",
                 "ui/public/third-party-notices.txt"):
        assert (ROOT / file).is_file(), f"Missing publication file: {file}"
    package_files = subprocess.check_output(
        ["cargo", "package", "--locked", "--list"], cwd=ROOT, text=True
    ).splitlines()
    allowed = {"Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "LICENSE", "README.md",
               "SECURITY.md", "THIRD_PARTY_NOTICES.md", "ui/public/third-party-notices.txt",
               "docs/engineering/api/openapi.yaml", ".cargo_vcs_info.json"}
    for file in package_files:
        assert file in allowed or file.startswith(("src/", "migrations/")), (
            f"Unexpected Cargo publication file: {file}")
    assert "src/database.rs" in package_files
    embedded = set(re.findall(r'include_str!\("\.\./migrations/([^\"]+)"\)',
                              (ROOT / "src/storage.rs").read_text()))
    migrations = {p.name for p in (ROOT / "migrations").glob("*.sql")}
    assert embedded == migrations, "Every migration must be embedded exactly once"
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    names = {p["name"] for p in lock["package"]}
    assert not names & {"rsa", "sqlx-mysql", "sqlx-postgres"}, (
        "Unused server database dependencies have returned")
    checked = 0
    files = [ROOT / name for name in ("README.md", "CONTRIBUTING.md", "SECURITY.md",
                                     "THIRD_PARTY_NOTICES.md")]
    files += list((ROOT / "docs").rglob("*.md"))
    files += list((ROOT / ".agents").rglob("*.md"))
    for file in files:
        for target in re.findall(r'\]\(([^)\n]+)\)', file.read_text()):
            target = target.strip().split(" ")[0].strip("<>")
            if target.startswith(("https:", "http:", "mailto:", "#", "plugin:")):
                continue
            target = target.split("#")[0]
            if target:
                assert (file.parent / target).exists(), f"Broken link: {file.relative_to(ROOT)} → {target}"
                checked += 1
    print(f"Publication checks passed: {len(package_files)} Cargo files, {checked} local links")


if __name__ == "__main__":
    main()
