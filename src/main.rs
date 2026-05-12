use std::io::{self, BufReader};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use codex_zai_proxy::{CompatMode, Proxy, ProxyConfig};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Streamable HTTP MCP endpoint to proxy.
    #[arg(long)]
    upstream_url: String,

    /// Name of the environment variable that contains a bearer token.
    #[arg(long)]
    bearer_token_env: Option<String>,

    /// Literal bearer token. Prefer --bearer-token-env for real use.
    #[arg(long, hide_env_values = true)]
    bearer_token: Option<String>,

    /// Read KEY=VALUE pairs from a dotenv-style file. Can be repeated.
    #[arg(long = "env-file")]
    env_files: Vec<PathBuf>,

    /// Extra upstream header in NAME=VALUE form. Can be repeated.
    #[arg(long = "header")]
    headers: Vec<String>,

    /// Upstream TCP/TLS connection timeout.
    #[arg(long, default_value_t = 10)]
    connect_timeout_secs: u64,

    /// Full upstream request timeout.
    #[arg(long, default_value_t = 120)]
    request_timeout_secs: u64,

    /// Compatibility behavior for imperfect Streamable HTTP MCP servers.
    #[arg(long, default_value = "auto")]
    compat_mode: CompatMode,

    /// Log level written to stderr.
    #[arg(long, default_value = "warn")]
    log_level: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    init_logging(&args.log_level);

    let config = ProxyConfig {
        upstream_url: args.upstream_url,
        bearer_token: args.bearer_token,
        bearer_token_env: args.bearer_token_env,
        env_files: args.env_files,
        headers: args.headers,
        connect_timeout: Duration::from_secs(args.connect_timeout_secs),
        request_timeout: Duration::from_secs(args.request_timeout_secs),
        compat_mode: args.compat_mode,
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut proxy = Proxy::new(config)?;
    proxy.run(BufReader::new(stdin.lock()), stdout.lock())
}

fn init_logging(log_level: &str) {
    let filter = EnvFilter::try_new(log_level).unwrap_or_else(|_| EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .compact()
        .init();
}
