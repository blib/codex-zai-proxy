# Security policy

## Supported versions

Only the latest released version is supported for security fixes.

## Reporting a vulnerability

Please report security issues privately instead of opening a public issue.

Use GitHub Security Advisories if available for this repository, or contact the maintainer through the GitHub profile listed on the repository.

Do not include live bearer tokens, API keys, private `.env` files, or proprietary prompts in reports unless the maintainer explicitly asks for them through a private channel.

## Security expectations

- `stdout` is reserved for MCP JSON-RPC framing.
- Diagnostics and logs go to `stderr`.
- Bearer tokens must not be printed in logs, release artifacts, issues, or examples.
- Prefer `--bearer-token-env` and `--env-file` over `--bearer-token`.
