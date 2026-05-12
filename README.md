# codex-zai-proxy

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
- stderr logging that avoids printing bearer tokens

## Status

This is a focused v0.1 tool. It is intentionally one process per upstream MCP server. It does not route multiple upstreams, cache tool responses, or store secrets.

## Install

```bash
cargo install --path .
```

For local development:

```bash
cargo build
cargo test
```

## CLI

```bash
codex-zai-proxy \
  --upstream-url https://api.z.ai/api/mcp/zread/mcp \
  --env-file ~/.zai.env \
  --bearer-token-env ZAI_MCP_TOKEN
```

Options:

- `--upstream-url <URL>`: required Streamable HTTP MCP endpoint.
- `--bearer-token-env <ENV_NAME>`: read a bearer token from an environment variable.
- `--bearer-token <TOKEN>`: pass a bearer token directly. Useful for local tests; avoid it in shell history.
- `--env-file <PATH>`: read dotenv-style `KEY=VALUE` entries when the process environment does not contain `--bearer-token-env`. Can be repeated.
- `--header <NAME=VALUE>`: add an upstream HTTP header. Can be repeated.
- `--connect-timeout-secs <N>`: TCP/TLS connection timeout. Default: `10`.
- `--request-timeout-secs <N>`: full upstream request timeout. Default: `120`.
- `--compat-mode <auto|strict|zai>`: response normalization mode. Default: `auto`.
- `--log-level <error|warn|info|debug|trace>`: stderr log level. Default: `warn`.

## Codex configuration

Add one stdio MCP entry per upstream server:

```toml
[mcp_servers.zread]
command = "/Users/blib/.local/bin/codex-zai-proxy"
args = [
  "--upstream-url", "https://api.z.ai/api/mcp/zread/mcp",
  "--env-file", "/path/to/private.env",
  "--bearer-token-env", "ZAI_MCP_TOKEN",
]
```

`--env-file` is useful for GUI-launched Codex sessions because desktop apps often do not inherit shell environment variables.

For the Z.AI MCP endpoints:

```toml
[mcp_servers.web_reader]
command = "/Users/blib/.local/bin/codex-zai-proxy"
args = [
  "--upstream-url", "https://api.z.ai/api/mcp/web_reader/mcp",
  "--env-file", "/path/to/private.env",
  "--bearer-token-env", "ZAI_MCP_TOKEN",
]

[mcp_servers.web_search]
command = "/Users/blib/.local/bin/codex-zai-proxy"
args = [
  "--upstream-url", "https://api.z.ai/api/mcp/web_search_prime/mcp",
  "--env-file", "/path/to/private.env",
  "--bearer-token-env", "ZAI_MCP_TOKEN",
]

[mcp_servers.zread]
command = "/Users/blib/.local/bin/codex-zai-proxy"
args = [
  "--upstream-url", "https://api.z.ai/api/mcp/zread/mcp",
  "--env-file", "/path/to/private.env",
  "--bearer-token-env", "ZAI_MCP_TOKEN",
]
```

## How it works

The proxy reads newline-delimited JSON-RPC messages from stdin. For each message it sends a POST request to the configured upstream with:

- `Accept: application/json, text/event-stream`
- `Content-Type: application/json`
- `Authorization: Bearer ...` when configured
- `Mcp-Session-Id: ...` after the upstream returns a session id

For request messages, the proxy writes one normalized JSON-RPC response to stdout. For notification messages, the proxy forwards the notification upstream and does not write a client response.

## Compatibility modes

`auto` is the normal mode. It accepts standard JSON responses, extracts JSON from SSE responses, and treats empty upstream responses as acceptable for notifications and other no-body responses.

`strict` is useful when developing against a fully conformant upstream. It rejects empty responses and missing `Content-Type` cases.

`zai` is reserved for Z.AI-specific transport fixes. In v0.1 it behaves like `auto`; the mode exists so future fixes can be enabled without changing Codex config shape.

## Security notes

- Prefer `--bearer-token-env` over `--bearer-token`.
- Prefer `--env-file` for GUI clients instead of embedding secrets in MCP configuration.
- Logs go to stderr, never stdout, so MCP framing stays clean.
- The proxy does not persist tokens, session ids, requests, or responses.
- Avoid `--log-level trace` when handling sensitive prompts or tool arguments.

## Development

```bash
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings
```

The core behavior is covered by unit tests for header parsing, bearer token resolution, JSON normalization, SSE parsing, and compatibility handling.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
