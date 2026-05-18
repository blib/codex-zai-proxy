# Release process

Releases are built on GitHub from immutable version tags.

## Create a release

1. Update `Cargo.toml` version if needed.
2. Update `CHANGELOG.md`.
3. Add `docs/release-notes/vX.Y.Z.md`.
4. Merge or push the release commit to `master`.
5. Create and push a signed or annotated tag:

```bash
git tag -a vX.Y.Z -m "codex-zai-proxy vX.Y.Z"
git push origin vX.Y.Z
```

The `Release` workflow builds platform binaries, packages archives, verifies checksums, and publishes the GitHub Release.

## Release assets

Each release publishes:

- Linux x86_64 tarball and checksum
- macOS Intel tarball and checksum
- macOS Apple Silicon tarball and checksum
- Windows x86_64 zip and checksum

The archives include the binary, README, changelog, and both license files.
