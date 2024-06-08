use std::{env, net::SocketAddr};

use anyhow::Context;
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::{net::TcpListener, signal};

use tracing::info;
use vssv::{routes::build_router, ServerState};

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
pub async fn get_db_pool(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(
            num_cpus::get()
                .try_into()
                .expect("number of CPU cores should fit into an u32"),
        )
        .connect(db_url)
        .await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let listen_url = env::var("LISTEN_URL").context("LISTEN_URL must be set")?;

    let db_pool = get_db_pool(&database_url).await?;
    sqlx::migrate!().run(&db_pool).await?;

    let router = build_router(ServerState { database: db_pool });
    let listener = TcpListener::bind(listen_url).await?;

    info!("Will start to listen on `{}`...", listener.local_addr()?);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}
