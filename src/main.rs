mod components;
mod entities;
mod errors;
mod routers;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use clap::Parser;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use tokio::net::TcpListener;
use tracing::info;

use crate::{
    components::{
        app_state::AppState,
        settings::{LogFormat, Settings},
    },
    routers::build_main_router,
};

/// Sets up a relevant shutdown signals. This will exit on either SIGINT
/// (aka Ctrl+C) or SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to create Ctrl+C handler");
    };

    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to create SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        () = ctrl_c => {},
        () = sigterm => {},
    }

    info!("shutdown signal received")
}

/// Creates a [PgPool] if possible. The pool has its max_connections value set
/// to mirror the tokio worker thread count.
pub async fn get_db_pool(connect_options: PgConnectOptions) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(
            tokio::runtime::Handle::current()
                .metrics()
                .num_workers()
                .try_into()
                .expect("num_workers to be less than 2^32"),
        )
        .connect_with(connect_options)
        .await
}

fn main() -> anyhow::Result<()> {
    let settings = Settings::parse();

    let mut rt = tokio::runtime::Builder::new_multi_thread();
    if let Some(threads) = settings.threads {
        rt.worker_threads(threads);
    }

    rt.enable_all()
        .build()?
        .block_on(async { run(settings).await })
}

async fn run(settings: Settings) -> anyhow::Result<()> {
    let settings_clone = settings.clone();

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(settings_clone.log_level.tracing_level())
        .with_target(false);
    match settings_clone.log_format {
        LogFormat::Text => subscriber.with_ansi(false).init(),
        LogFormat::TextColor => subscriber.with_ansi(true).init(),
        LogFormat::Json => subscriber.json().with_span_list(false).init(),
    }

    let database = get_db_pool(settings_clone.database_url).await?;
    sqlx::migrate!().run(&database).await?;

    let router = build_main_router(AppState {
        database,
        settings: Arc::new(settings),
    });

    let listener = TcpListener::bind(settings_clone.listen)
        .await
        .context(format!("could not listen to `{}`", settings_clone.listen))?;

    info!("starting server on `{}`", settings_clone.listen);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("failed to start server")?;

    Ok(())
}
