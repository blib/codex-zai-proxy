#!/usr/bin/env python3
"""Package a built codex-zai-proxy binary for GitHub Releases."""

from __future__ import annotations

import argparse
import hashlib
import os
from pathlib import Path
import stat
import tarfile
import zipfile


ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument("--archive-name", required=True)
    parser.add_argument("--binary-name", required=True)
    args = parser.parse_args()

    binary = ROOT / "target" / args.target / "release" / args.binary_name
    if not binary.exists():
        raise SystemExit(f"built binary not found: {binary}")

    dist = ROOT / "dist"
    dist.mkdir(exist_ok=True)

    package_root = args.archive_name
    if args.binary_name.endswith(".exe"):
        archive = dist / f"{args.archive_name}.zip"
        write_zip(archive, package_root, binary)
    else:
        archive = dist / f"{args.archive_name}.tar.gz"
        write_tar_gz(archive, package_root, binary)

    write_sha256(archive)


def release_files(binary: Path) -> list[tuple[Path, str]]:
    files = [
        (binary, binary.name),
        (ROOT / "README.md", "README.md"),
        (ROOT / "CHANGELOG.md", "CHANGELOG.md"),
        (ROOT / "LICENSE-MIT", "LICENSE-MIT"),
        (ROOT / "LICENSE-APACHE", "LICENSE-APACHE"),
    ]
    return files


def write_tar_gz(archive: Path, package_root: str, binary: Path) -> None:
    with tarfile.open(archive, "w:gz") as tar:
        for source, name in release_files(binary):
            arcname = f"{package_root}/{name}"
            info = tar.gettarinfo(str(source), arcname)
            if source == binary:
                info.mode = stat.S_IRUSR | stat.S_IWUSR | stat.S_IXUSR | stat.S_IRGRP | stat.S_IXGRP | stat.S_IROTH | stat.S_IXOTH
            with source.open("rb") as handle:
                tar.addfile(info, handle)


def write_zip(archive: Path, package_root: str, binary: Path) -> None:
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as zip_file:
        for source, name in release_files(binary):
            zip_info = zipfile.ZipInfo(f"{package_root}/{name}")
            zip_info.compress_type = zipfile.ZIP_DEFLATED
            if source == binary:
                zip_info.external_attr = (0o755 & 0xFFFF) << 16
            with source.open("rb") as handle:
                zip_file.writestr(zip_info, handle.read())


def write_sha256(archive: Path) -> None:
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    checksum = archive.with_suffix(f"{archive.suffix}.sha256")
    checksum.write_text(f"{digest}  {os.path.basename(archive)}\n", encoding="utf-8")


if __name__ == "__main__":
    main()
