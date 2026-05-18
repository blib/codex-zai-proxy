# LLM installation guide

Use this file when an LLM agent needs to install or configure `codex-zai-proxy` for a user.

## Rules

- Do not ask the user to install Rust or Cargo for a normal installation.
- Prefer GitHub Release binaries.
- Never print bearer tokens, API keys, `.env` contents, or `Authorization` headers.
- Prefer `--env-file` plus `--bearer-token-env` for GUI-launched clients.
- Use absolute paths in MCP client configuration.
- Verify the installed binary with `codex-zai-proxy --version`.

## Decision flow

1. Identify OS and CPU architecture.
2. Download the matching asset from `https://github.com/blib/codex-zai-proxy/releases/latest`.
3. Download the matching `.sha256` file.
4. Verify the checksum.
5. Install the binary into a user-writable bin directory.
6. Configure the MCP client to invoke that binary over stdio.
7. Test with a harmless MCP request such as `tools/list`.

## Asset mapping

| OS / arch | Release asset |
| --- | --- |
| Linux x86_64 | `codex-zai-proxy-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `codex-zai-proxy-v0.1.0-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `codex-zai-proxy-v0.1.0-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `codex-zai-proxy-v0.1.0-x86_64-pc-windows-msvc.zip` |

For newer versions, keep the same target suffix and replace `v0.1.0` with the selected release tag.

## macOS/Linux install pattern

```bash
version="v0.1.0"
target="aarch64-apple-darwin"
asset="codex-zai-proxy-${version}-${target}.tar.gz"
base_url="https://github.com/blib/codex-zai-proxy/releases/download/${version}"

curl -L -o "/tmp/${asset}" "${base_url}/${asset}"
curl -L -o "/tmp/${asset}.sha256" "${base_url}/${asset}.sha256"

cd /tmp
shasum -a 256 -c "${asset}.sha256"
tar -xzf "${asset}"
mkdir -p "$HOME/.local/bin"
cp "codex-zai-proxy-${version}-${target}/codex-zai-proxy" "$HOME/.local/bin/"
chmod +x "$HOME/.local/bin/codex-zai-proxy"
```

## Windows install pattern

```powershell
$Version = "v0.1.0"
$Target = "x86_64-pc-windows-msvc"
$Asset = "codex-zai-proxy-$Version-$Target.zip"
$BaseUrl = "https://github.com/blib/codex-zai-proxy/releases/download/$Version"
$InstallDir = "$env:LOCALAPPDATA\codex-zai-proxy\bin"

Invoke-WebRequest -Uri "$BaseUrl/$Asset" -OutFile "$env:TEMP\$Asset"
Invoke-WebRequest -Uri "$BaseUrl/$Asset.sha256" -OutFile "$env:TEMP\$Asset.sha256"
Get-FileHash "$env:TEMP\$Asset" -Algorithm SHA256
Get-Content "$env:TEMP\$Asset.sha256"
Expand-Archive -Path "$env:TEMP\$Asset" -DestinationPath "$env:TEMP\codex-zai-proxy" -Force
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item "$env:TEMP\codex-zai-proxy\codex-zai-proxy-$Version-$Target\codex-zai-proxy.exe" "$InstallDir\codex-zai-proxy.exe" -Force
```

Ask the user to compare the displayed hash with the `.sha256` file if you cannot automate the comparison safely.

## Codex MCP configuration pattern

```toml
[mcp_servers.zread]
command = "/absolute/path/to/codex-zai-proxy"
args = [
  "--upstream-url", "https://api.z.ai/api/mcp/zread/mcp",
  "--env-file", "/absolute/path/to/private.env",
  "--bearer-token-env", "ZAI_MCP_TOKEN",
]
```

Repeat the block for each upstream MCP server. The proxy intentionally handles one upstream per process.

## Troubleshooting

- `environment variable ... is not set`: add `--env-file` or fix the variable name.
- `connection closed: initialize response`: run the proxy directly with a single JSON-RPC `initialize` message and inspect stderr, but do not print secrets.
- `permission denied`: install into a user-writable directory and use an absolute `command` path.
- Z.AI `GET stream error: 405 Method Not Allowed`: this can be non-fatal for Streamable HTTP servers that accept POST-based MCP calls.
