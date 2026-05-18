#!/usr/bin/env python3
"""Verify .sha256 files generated for release assets."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("path", nargs="+", help="Directory or .sha256 file to verify")
    args = parser.parse_args()

    checksum_files: list[Path] = []
    for raw_path in args.path:
        path = Path(raw_path)
        if path.is_dir():
            checksum_files.extend(sorted(path.rglob("*.sha256")))
        else:
            checksum_files.append(path)

    if not checksum_files:
        raise SystemExit("no .sha256 files found")

    for checksum_file in checksum_files:
        verify_checksum_file(checksum_file)


def verify_checksum_file(checksum_file: Path) -> None:
    line = checksum_file.read_text(encoding="utf-8").strip()
    expected, filename = line.split(maxsplit=1)
    asset = checksum_file.parent / filename
    actual = hashlib.sha256(asset.read_bytes()).hexdigest()
    if actual != expected:
        raise SystemExit(f"{asset}: checksum mismatch")
    print(f"{asset}: OK")


if __name__ == "__main__":
    main()
