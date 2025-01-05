use std::net::SocketAddr;

use clap::Parser;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use tokio::{net::TcpListener, signal};

use tracing::info;
use vssv::{routes::build_router, settings::Settings, ServerState};

/// Listens to SIGINT (aka ctrl-c) and SIGTERM and completes whenever one of
/// those signals happen.
async fn shutdown_signal() {
    let sigint = async {
        signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("creating SIGINT handler should not fail")
            .recv()
            .await;
    };

    let sigterm = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("creating SIGTERM handler should not fail")
            .recv()
            .await;
    };

    tokio::select! {
        () = sigint => {},
        () = sigterm => {},
    }
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
    tracing_subscriber::fmt::init();

    let db_pool = get_db_pool(settings.database_url).await?;
    sqlx::migrate!().run(&db_pool).await?;

    let router = build_router(ServerState { database: db_pool });
    let listener = TcpListener::bind(settings.listen_addr).await?;

    info!("Will start to listen on `{}`...", settings.listen_addr);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}
