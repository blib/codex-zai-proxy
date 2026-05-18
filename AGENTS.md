# Agent instructions

This is a small Rust CLI project. Keep changes focused and preserve the transport contract.

## Invariants

- stdout is MCP JSON-RPC only.
- logs and diagnostics go to stderr.
- secrets must not be printed.
- one process proxies one upstream MCP server.
- user-facing installation should prefer GitHub Release binaries over requiring Rust.

## Checks

Run before claiming completion:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
```

For release-related changes, also inspect:

```bash
python scripts/package-release.py --help
```
