# Install codex-zai-proxy

Most users should install `codex-zai-proxy` from a GitHub Release. You do not need Rust or Cargo on the client machine when you use a prebuilt release asset.

## Pick the right asset

Download the asset for your operating system and CPU:

| System | Asset suffix |
| --- | --- |
| Linux x86_64 | `x86_64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `x86_64-pc-windows-msvc.zip` |

Release page:

```text
https://github.com/blib/codex-zai-proxy/releases/latest
```

## macOS and Linux

```bash
version="v0.1.0"
target="aarch64-apple-darwin" # change for your machine
asset="codex-zai-proxy-${version}-${target}.tar.gz"

curl -L -o "/tmp/${asset}" "https://github.com/blib/codex-zai-proxy/releases/download/${version}/${asset}"
curl -L -o "/tmp/${asset}.sha256" "https://github.com/blib/codex-zai-proxy/releases/download/${version}/${asset}.sha256"

cd /tmp
shasum -a 256 -c "${asset}.sha256"
tar -xzf "${asset}"

mkdir -p "$HOME/.local/bin"
cp "codex-zai-proxy-${version}-${target}/codex-zai-proxy" "$HOME/.local/bin/"
chmod +x "$HOME/.local/bin/codex-zai-proxy"
"$HOME/.local/bin/codex-zai-proxy" --version
```

If `~/.local/bin` is not on `PATH`, either add it to your shell profile or use the absolute path in Codex configuration.

## Windows

In PowerShell:

```powershell
$Version = "v0.1.0"
$Target = "x86_64-pc-windows-msvc"
$Asset = "codex-zai-proxy-$Version-$Target.zip"
$InstallDir = "$env:LOCALAPPDATA\codex-zai-proxy\bin"

Invoke-WebRequest -Uri "https://github.com/blib/codex-zai-proxy/releases/download/$Version/$Asset" -OutFile "$env:TEMP\$Asset"
Invoke-WebRequest -Uri "https://github.com/blib/codex-zai-proxy/releases/download/$Version/$Asset.sha256" -OutFile "$env:TEMP\$Asset.sha256"

Set-Location $env:TEMP
Get-FileHash $Asset -Algorithm SHA256
Get-Content "$Asset.sha256"

Expand-Archive -Path "$env:TEMP\$Asset" -DestinationPath "$env:TEMP\codex-zai-proxy" -Force
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item "$env:TEMP\codex-zai-proxy\codex-zai-proxy-$Version-$Target\codex-zai-proxy.exe" "$InstallDir\codex-zai-proxy.exe" -Force
& "$InstallDir\codex-zai-proxy.exe" --version
```

Windows does not include a built-in `sha256sum -c` equivalent. Compare the `Get-FileHash` value with the `.sha256` file before installing.

## Codex configuration

Add one MCP server entry per upstream:

```toml
[mcp_servers.zread]
command = "/Users/you/.local/bin/codex-zai-proxy"
args = [
  "--upstream-url", "https://api.z.ai/api/mcp/zread/mcp",
  "--env-file", "/Users/you/.zai.env",
  "--bearer-token-env", "ZAI_MCP_TOKEN",
]
```

Use an absolute path for `command`. GUI apps often do not inherit shell `PATH` or shell environment variables.

## Installing from source

Developers can build from source with Cargo:

```bash
cargo install --git https://github.com/blib/codex-zai-proxy
```

This path requires a Rust toolchain. It is not needed for normal client installations.
