use std::net::SocketAddr;

use clap::Parser;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use tokio::{net::TcpListener, signal};

use tracing::info;
use vssv::{routes::build_router, Cli, ServerState};

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
/// to the number of CPUs available.
pub async fn get_db_pool(connect_options: PgConnectOptions) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(
            num_cpus::get()
                .try_into()
                .expect("number of CPU cores should fit into an u32"),
        )
        .connect_with(connect_options)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt::init();

    let db_pool = get_db_pool(cli.database_url).await?;
    sqlx::migrate!().run(&db_pool).await?;

    let router = build_router(ServerState { database: db_pool });
    let listener = TcpListener::bind(cli.listen_addr).await?;

    info!("Will start to listen on `{}`...", cli.listen_addr);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}
