# codex-zai-proxy

[![CI](https://github.com/blib/codex-zai-proxy/actions/workflows/ci.yml/badge.svg)](https://github.com/blib/codex-zai-proxy/actions/workflows/ci.yml)
[![Release](https://github.com/blib/codex-zai-proxy/actions/workflows/release.yml/badge.svg)](https://github.com/blib/codex-zai-proxy/actions/workflows/release.yml)

`codex-zai-proxy` is a small Rust compatibility proxy that lets strict stdio MCP clients, including OpenAI Codex, talk to Streamable HTTP MCP servers.

It was built for the practical case where an upstream MCP endpoint is useful, but its HTTP transport behavior is slightly different from what a strict client expects. The proxy runs as a normal stdio MCP server on the client side and forwards JSON-RPC messages to one HTTP MCP upstream.

## Why this exists

Codex can run MCP servers over stdio very reliably. Some remote MCP servers, however, expose only Streamable HTTP and may return edge-case responses during the MCP handshake, such as an empty notification response without a `Content-Type` header. A strict client can treat that as fatal even when the upstream tool server is otherwise usable.

`codex-zai-proxy` puts a narrow compatibility layer between the two transports:

- stdin/stdout JSON-RPC for Codex and other stdio MCP clients
- Streamable HTTP POST requests for the upstream MCP server
- MCP session header capture and forwarding
- SSE response extraction
- tolerant handling of empty notification responses in compatibility mode

## Install

Download the binary for your platform from the latest release:

https://github.com/blib/codex-zai-proxy/releases/latest

Assets:

- `codex-zai-proxy-macos-arm64`
- `codex-zai-proxy-linux-x86_64`
- `codex-zai-proxy-windows-x86_64.exe`

macOS/Linux:

```bash
mkdir -p ~/.local/bin
cp ./codex-zai-proxy-macos-arm64 ~/.local/bin/codex-zai-proxy
chmod +x ~/.local/bin/codex-zai-proxy
~/.local/bin/codex-zai-proxy --version
```

Windows: put `codex-zai-proxy-windows-x86_64.exe` somewhere stable and use its absolute path in the MCP config.

No Rust or Cargo is needed when using release binaries.

## Codex config

Add one MCP server per upstream:

```toml
[mcp_servers.zread]
command = "/Users/you/.local/bin/codex-zai-proxy"
args = [
  "--upstream-url", "https://api.z.ai/api/mcp/zread/mcp",
  "--env-file", "/Users/you/.zai.env",
  "--bearer-token-env", "ZAI_MCP_TOKEN",
]
```

For GUI-launched Codex, prefer `--env-file`: desktop apps often do not inherit shell environment variables.

## CLI

```bash
codex-zai-proxy \
  --upstream-url https://api.z.ai/api/mcp/zread/mcp \
  --env-file ~/.zai.env \
  --bearer-token-env ZAI_MCP_TOKEN
```

Useful options:

- `--upstream-url <URL>`: Streamable HTTP MCP endpoint.
- `--bearer-token-env <ENV_NAME>`: environment variable containing the bearer token.
- `--env-file <PATH>`: dotenv-style `KEY=VALUE` file; useful for GUI apps.
- `--header <NAME=VALUE>`: extra upstream HTTP header.
- `--compat-mode <auto|strict|zai>`: response normalization mode. Default: `auto`.
- `--log-level <error|warn|info|debug|trace>`: logs go to stderr.

## Notes for LLM agents

- Do not ask users to install Rust for normal use.
- Use release binaries and absolute paths in MCP config.
- Never print tokens or `.env` contents.
- Do not use `--bearer-token` unless the user explicitly accepts shell history risk.
- One `codex-zai-proxy` process proxies one upstream MCP endpoint.

## Development

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
```

## License

MIT
