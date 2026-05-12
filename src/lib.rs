use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use thiserror::Error;
use tracing::{debug, error, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatMode {
    Auto,
    Strict,
    Zai,
}

impl std::str::FromStr for CompatMode {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "auto" => Ok(Self::Auto),
            "strict" => Ok(Self::Strict),
            "zai" => Ok(Self::Zai),
            other => Err(format!("unsupported compat mode: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub upstream_url: String,
    pub bearer_token: Option<String>,
    pub bearer_token_env: Option<String>,
    pub env_files: Vec<PathBuf>,
    pub headers: Vec<String>,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub compat_mode: CompatMode,
}

impl ProxyConfig {
    pub fn build_upstream_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json, text/event-stream"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(token) = self.resolved_bearer_token()? {
            let value = HeaderValue::from_str(&format!("Bearer {token}"))
                .context("bearer token contains invalid HTTP header characters")?;
            headers.insert(AUTHORIZATION, value);
        }

        for raw in &self.headers {
            let (name, value) = parse_header_arg(raw)?;
            headers.insert(name, value);
        }

        Ok(headers)
    }

    fn resolved_bearer_token(&self) -> Result<Option<String>> {
        if let Some(token) = &self.bearer_token {
            return Ok(Some(token.clone()));
        }

        if let Some(name) = &self.bearer_token_env {
            if let Ok(token) = env::var(name) {
                return Ok(Some(token));
            }

            for path in &self.env_files {
                if let Some(token) = read_env_file_value(path, name)? {
                    return Ok(Some(token));
                }
            }

            bail!("environment variable {name} is not set");
        }

        Ok(None)
    }
}

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("upstream returned HTTP {status}: {body}")]
    UpstreamStatus { status: u16, body: String },
    #[error("upstream response did not contain a usable JSON-RPC message")]
    EmptyResponse,
    #[error("failed to parse upstream response as JSON-RPC: {0}")]
    InvalidJson(String),
}

pub struct Proxy {
    client: Client,
    config: ProxyConfig,
    base_headers: HeaderMap,
    session_id: Option<String>,
}

impl Proxy {
    pub fn new(config: ProxyConfig) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .build()
            .context("failed to build HTTP client")?;
        let base_headers = config.build_upstream_headers()?;

        Ok(Self {
            client,
            config,
            base_headers,
            session_id: None,
        })
    }

    pub fn run<R, W>(&mut self, input: R, mut output: W) -> Result<()>
    where
        R: BufRead,
        W: Write,
    {
        for line in input.lines() {
            let line = line.context("failed to read stdio line")?;
            if line.trim().is_empty() {
                continue;
            }

            match self.handle_client_line(&line) {
                Ok(Some(response)) => {
                    writeln!(output, "{response}").context("failed to write stdio response")?;
                    output.flush().context("failed to flush stdio response")?;
                }
                Ok(None) => {}
                Err(err) => {
                    error!(error = %err, "failed to proxy MCP message");
                    if let Some(error_response) =
                        json_rpc_error_for_line(&line, -32603, &err.to_string())
                    {
                        writeln!(output, "{error_response}")
                            .context("failed to write stdio error response")?;
                        output
                            .flush()
                            .context("failed to flush stdio error response")?;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_client_line(&mut self, line: &str) -> Result<Option<String>> {
        let message: Value =
            serde_json::from_str(line).context("client message is not valid JSON")?;
        let is_notification = json_rpc_id(&message).is_none();
        let method = json_rpc_method(&message).unwrap_or("<unknown>");
        debug!(
            method,
            notification = is_notification,
            "proxying MCP client message"
        );
        let upstream = self.send_upstream(line)?;

        if is_notification {
            if upstream.is_some() {
                warn!("upstream returned a body for a JSON-RPC notification; dropping it");
            }
            Ok(None)
        } else {
            Ok(upstream)
        }
    }

    fn send_upstream(&mut self, body: &str) -> Result<Option<String>> {
        let mut headers = self.base_headers.clone();
        if let Some(session_id) = &self.session_id {
            headers.insert(
                HeaderName::from_static("mcp-session-id"),
                HeaderValue::from_str(session_id)
                    .context("stored MCP session id is not a valid header value")?,
            );
        }

        let response = self
            .client
            .post(&self.config.upstream_url)
            .headers(headers)
            .body(body.to_owned())
            .send()
            .context("failed to send upstream request")?;

        if let Some(session_id) = response.headers().get("mcp-session-id") {
            let session_id = session_id
                .to_str()
                .context("upstream MCP session id is not valid ASCII")?
                .to_owned();
            debug!(session_id = %session_id, "captured upstream MCP session id");
            self.session_id = Some(session_id);
        }

        let status = response.status();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let text = response
            .text()
            .context("failed to read upstream response body")?;

        if !status.is_success() {
            return Err(ProxyError::UpstreamStatus {
                status: status.as_u16(),
                body: redact_secrets(&text),
            }
            .into());
        }

        normalize_upstream_body(&text, content_type.as_deref(), self.config.compat_mode)
            .map_err(Into::into)
    }
}

pub fn parse_header_arg(raw: &str) -> Result<(HeaderName, HeaderValue)> {
    let (name, value) = raw
        .split_once('=')
        .with_context(|| format!("header must use NAME=VALUE syntax: {raw}"))?;
    let name = HeaderName::from_bytes(name.trim().as_bytes())
        .with_context(|| format!("invalid HTTP header name: {name}"))?;
    let value = HeaderValue::from_str(value.trim())
        .with_context(|| format!("invalid HTTP header value for {name}"))?;
    Ok((name, value))
}

pub fn read_env_file_value(path: &Path, key: &str) -> Result<Option<String>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read env file {}", path.display()))?;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };

        if raw_key.trim() == key {
            return Ok(Some(unquote_env_value(raw_value.trim()).to_owned()));
        }
    }

    Ok(None)
}

fn unquote_env_value(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let first = bytes[0];
        let last = bytes[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &value[1..value.len() - 1];
        }
    }

    value
}

pub fn normalize_upstream_body(
    body: &str,
    content_type: Option<&str>,
    compat_mode: CompatMode,
) -> std::result::Result<Option<String>, ProxyError> {
    let trimmed = body.trim();

    if trimmed.is_empty() {
        if compat_mode == CompatMode::Strict {
            return Err(ProxyError::EmptyResponse);
        }
        return Ok(None);
    }

    if content_type
        .map(|value| value.contains("text/event-stream"))
        .unwrap_or(false)
        || trimmed.starts_with("event:")
        || trimmed.starts_with("data:")
    {
        return parse_sse_json_message(trimmed);
    }

    if content_type.is_none() && compat_mode == CompatMode::Strict {
        return Err(ProxyError::InvalidJson("missing Content-Type".to_owned()));
    }

    parse_json_message(trimmed)
}

pub fn parse_sse_json_message(body: &str) -> std::result::Result<Option<String>, ProxyError> {
    let mut current_data = Vec::new();

    for line in body.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            if let Some(message) = finish_sse_event(&mut current_data)? {
                return Ok(Some(message));
            }
            continue;
        }

        if let Some(data) = line.strip_prefix("data:") {
            current_data.push(data.trim_start().to_owned());
        }
    }

    finish_sse_event(&mut current_data)
}

fn finish_sse_event(
    current_data: &mut Vec<String>,
) -> std::result::Result<Option<String>, ProxyError> {
    if current_data.is_empty() {
        return Ok(None);
    }

    let data = current_data.join("\n");
    current_data.clear();

    if data == "[DONE]" {
        return Ok(None);
    }

    parse_json_message(&data)
}

fn parse_json_message(raw: &str) -> std::result::Result<Option<String>, ProxyError> {
    let value: Value =
        serde_json::from_str(raw).map_err(|err| ProxyError::InvalidJson(err.to_string()))?;
    Ok(Some(value.to_string()))
}

fn json_rpc_id(message: &Value) -> Option<&Value> {
    message.as_object()?.get("id")
}

fn json_rpc_method(message: &Value) -> Option<&str> {
    message.as_object()?.get("method")?.as_str()
}

fn json_rpc_error_for_line(line: &str, code: i64, message: &str) -> Option<String> {
    let parsed: Value = serde_json::from_str(line).ok()?;
    let id = json_rpc_id(&parsed)?;
    Some(
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message,
            }
        })
        .to_string(),
    )
}

fn redact_secrets(value: &str) -> String {
    let mut redacted = value.to_owned();
    let markers = HashMap::from([("Bearer ", "Bearer <redacted>")]);
    for (needle, replacement) in markers {
        if let Some(index) = redacted.find(needle) {
            let tail = redacted[index + needle.len()..]
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_owned();
            redacted = redacted.replace(&format!("{needle}{tail}"), replacement);
        }
    }
    redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_custom_header_arguments() {
        let (name, value) = parse_header_arg("X-Test=value").unwrap();

        assert_eq!(name.as_str(), "x-test");
        assert_eq!(value.to_str().unwrap(), "value");
    }

    #[test]
    fn rejects_malformed_custom_header_arguments() {
        let err = parse_header_arg("X-Test").unwrap_err().to_string();

        assert!(err.contains("NAME=VALUE"));
    }

    #[test]
    fn normalizes_json_response_without_reformatting_semantics() {
        let response = normalize_upstream_body(
            r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#,
            Some("application/json"),
            CompatMode::Auto,
        )
        .unwrap();

        assert_eq!(
            response.unwrap(),
            r#"{"id":1,"jsonrpc":"2.0","result":{"ok":true}}"#
        );
    }

    #[test]
    fn accepts_empty_response_in_compat_mode() {
        let response = normalize_upstream_body("", None, CompatMode::Auto).unwrap();

        assert!(response.is_none());
    }

    #[test]
    fn rejects_empty_response_in_strict_mode() {
        let err = normalize_upstream_body("", None, CompatMode::Strict).unwrap_err();

        assert!(matches!(err, ProxyError::EmptyResponse));
    }

    #[test]
    fn extracts_first_json_message_from_sse() {
        let response = parse_sse_json_message(
            "event: message\n\
             data: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"tools\":[]}}\n\n",
        )
        .unwrap();

        assert_eq!(
            response.unwrap(),
            r#"{"id":1,"jsonrpc":"2.0","result":{"tools":[]}}"#
        );
    }

    #[test]
    fn builds_authorization_header_from_env() {
        env::set_var("CODEX_ZAI_PROXY_TEST_TOKEN", "secret-token");
        let config = ProxyConfig {
            upstream_url: "https://example.com/mcp".to_owned(),
            bearer_token: None,
            bearer_token_env: Some("CODEX_ZAI_PROXY_TEST_TOKEN".to_owned()),
            env_files: vec![],
            headers: vec![],
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            compat_mode: CompatMode::Auto,
        };

        let headers = config.build_upstream_headers().unwrap();

        assert_eq!(
            headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
            "Bearer secret-token"
        );
    }

    #[test]
    fn builds_authorization_header_from_env_file() {
        env::remove_var("CODEX_ZAI_PROXY_TEST_FILE_TOKEN");
        let dir = env::temp_dir().join(format!("codex-zai-proxy-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let env_file = dir.join("test.env");
        fs::write(
            &env_file,
            "\n# ignored\nOTHER=value\nCODEX_ZAI_PROXY_TEST_FILE_TOKEN=\"file-token\"\n",
        )
        .unwrap();

        let config = ProxyConfig {
            upstream_url: "https://example.com/mcp".to_owned(),
            bearer_token: None,
            bearer_token_env: Some("CODEX_ZAI_PROXY_TEST_FILE_TOKEN".to_owned()),
            env_files: vec![env_file],
            headers: vec![],
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            compat_mode: CompatMode::Auto,
        };

        let headers = config.build_upstream_headers().unwrap();

        assert_eq!(
            headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
            "Bearer file-token"
        );
    }
}
