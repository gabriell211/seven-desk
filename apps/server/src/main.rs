#![forbid(unsafe_code)]

use seven_server::{AppState, router};
use std::{net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "seven_server=info".into()),
        )
        .init();

    // Deliberately loopback-only: persistence, TLS and deployment policy are
    // required before this control plane can be exposed to untrusted networks.
    let addr: SocketAddr = std::env::var("SEVEN_BIND")
        .unwrap_or_else(|_| "127.0.0.1:43821".to_string())
        .parse()?;
    if !addr.ip().is_loopback()
        && std::env::var("SEVEN_UNSAFE_ALLOW_REMOTE").as_deref() != Ok("I_ACCEPT_THE_RISK")
    {
        return Err(
            "remote binding is disabled until TLS/deployment hardening is completed".into(),
        );
    }
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(bind=%addr, "Seven Desk developer control plane started");
    axum::serve(listener, router(Arc::new(AppState::default()))).await?;
    Ok(())
}
