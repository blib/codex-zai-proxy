# Contributing

This project is intentionally small. Contributions should preserve the core shape:

- one process proxies one upstream MCP server
- stdout is reserved for MCP JSON-RPC only
- logs and diagnostics go to stderr
- secrets must not be printed
- behavior should be covered by tests before implementation

## Development loop

```bash
cargo fmt
cargo test --locked
cargo clippy --all-targets -- -D warnings
```

Release changes should also preserve the GitHub Actions release workflow and the installation instructions in `docs/install.md` and `docs/llm-install.md`.

## Good first issues

- Add a mock HTTP integration test for a specific upstream behavior.
- Improve compatibility handling for a documented MCP transport edge case.
- Add packaging instructions for another OS or package manager.
