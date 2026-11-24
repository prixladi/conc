use std::{net::SocketAddr, path::PathBuf, process::ExitCode, sync::Arc};

use app_config::AppConfig;
use clap::{Parser, Subcommand};
use daemon_client::{Requester, SocketClient};
use keys::KeyStore;
use tracing_subscriber::EnvFilter;

mod dto;
mod error;
mod keys;
mod server;

/// HTTP API proxy for the conc daemon
#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    /// Path to the SQLite database with API keys [default: ~/.conc/api.db]
    #[arg(long, global = true, env = "CONC_API_DB")]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the HTTP server
    Serve {
        /// Address to listen on
        #[arg(long, short, env = "CONC_API_BIND", default_value = "127.0.0.1:7777")]
        bind: SocketAddr,
        /// Daemon socket path [default: from ~/.conc/conf.json]
        #[arg(long, env = "CONC_API_SOCKET")]
        socket: Option<String>,
        /// Origin allowed to call the API from a browser (repeatable), e.g. http://localhost:5173
        #[arg(
            long = "cors-origin",
            env = "CONC_API_CORS_ORIGINS",
            value_delimiter = ','
        )]
        cors_origins: Vec<String>,
    },
    /// Manage API keys
    #[command(subcommand)]
    Keys(KeysCommand),
}

#[derive(Debug, Subcommand)]
enum KeysCommand {
    /// Create a new API key, the key is printed only once
    Create {
        /// Unique name of the key, e.g. the client using it
        name: String,
    },
    /// List API keys
    #[clap(visible_alias("ls"))]
    List {
        /// Include revoked keys
        #[arg(long, short)]
        all: bool,
    },
    /// Revoke an API key by name
    Revoke { name: String },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = match cli.db {
        Some(p) => p,
        None => PathBuf::from(std::env::var("HOME")?).join(".conc/api.db"),
    };
    let store = KeyStore::open(&db_path)?;

    match cli.command {
        Command::Keys(cmd) => run_keys(&store, cmd),
        Command::Serve {
            bind,
            socket,
            cors_origins,
        } => {
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| EnvFilter::new("conca=info,tower_http=info")),
                )
                .init();

            let config = AppConfig::new()?;
            let socket = socket.unwrap_or(config.daemon_socket_path);
            // The API has no meaningful caller environment, services get the daemon's.
            let requester = Requester::new(SocketClient::new(&socket), false);

            let state = server::AppState {
                requester,
                keys: Arc::new(store),
                default_restart: config.default_service_restart_behavior,
            };

            tokio::runtime::Runtime::new()?.block_on(async move {
                let listener = tokio::net::TcpListener::bind(bind).await?;
                tracing::info!(%bind, %socket, db = %db_path.display(), "listening");
                if !bind.ip().is_loopback() {
                    tracing::warn!("listening on a non-loopback address without TLS, put a TLS-terminating proxy in front");
                }
                axum::serve(listener, server::router(state, &cors_origins))
                    .with_graceful_shutdown(shutdown_signal())
                    .await
            })?;
            Ok(())
        }
    }
}

fn run_keys(store: &KeyStore, cmd: KeysCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        KeysCommand::Create { name } => {
            let (key, token) = store.create(&name)?;
            eprintln!(
                "Created API key '{}' (id {}). Store it now, it will not be shown again:",
                key.name, key.id
            );
            println!("{token}");
        }
        KeysCommand::List { all } => {
            let keys = store.list(all)?;
            if keys.is_empty() {
                eprintln!("No API keys.");
                return Ok(());
            }
            println!(
                "{:<4} {:<24} {:<15} {:<12} {:<12} REVOKED",
                "ID", "NAME", "PREFIX", "CREATED", "LAST USED"
            );
            for k in keys {
                println!(
                    "{:<4} {:<24} {:<15} {:<12} {:<12} {}",
                    k.id,
                    k.name,
                    format!("{}…", k.prefix),
                    ago(Some(k.created_at)),
                    ago(k.last_used_at),
                    ago(k.revoked_at),
                );
            }
        }
        KeysCommand::Revoke { name } => {
            if store.revoke(&name)? {
                eprintln!("Revoked API key '{name}'.");
            } else {
                return Err(format!("no active API key named '{name}'").into());
            }
        }
    }
    Ok(())
}

fn ago(ts: Option<u64>) -> String {
    let Some(ts) = ts else { return "-".into() };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(ts);
    let s = now.saturating_sub(ts);
    match s {
        0..60 => format!("{s}s ago"),
        60..3600 => format!("{}m ago", s / 60),
        3600..86400 => format!("{}h ago", s / 3600),
        _ => format!("{}d ago", s / 86400),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    let term = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            s.recv().await;
        }
    };
    tokio::select! { _ = ctrl_c => {}, _ = term => {} }
    tracing::info!("shutting down");
}
