mod err;
mod routes;
mod repo;
mod config;
mod domain;

use axum::{
    async_trait,
    extract::{Extension, FromRequest, Path, RequestParts},
    http::{StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

use std::{
    future::ready,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::config::ServerConfig;
use crate::routes::health_check::health_check;
use anyhow::Result;
use crate::domain::short_link::ShortLink;
use repo::MysqlRepo;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config: ServerConfig = toml::from_str(include_str!("../fixtures/server.conf"))?;


    // setup connection pool
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(config.db.db_conn.as_ref())
        .await
        .expect("can connect to database");

    let recorder_handle = setup_metrics_recorder();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/s/:slink", get(serve_slink))
        .route("/s", post(save_link))

        .route("/fast", get(|| async {}))
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }),
        )
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .layer(Extension(pool));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3002));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}


// we can extract the connection pool with `Extension`
async fn serve_slink(
    Path(slink): Path<String>,
    repo: MysqlRepo,
) -> core::result::Result<impl IntoResponse, (StatusCode, String)> {
    repo.serve_link(slink.as_str())
        .await
        .and_then(|dest: String| Ok(Redirect::to(dest.as_str())))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
async fn save_link(
    Json(slink): Json<ShortLink>,
    repo: MysqlRepo,
) -> core::result::Result<impl IntoResponse, (StatusCode, String)> {
    let short_link = slink.clone();
    repo.save_link(slink)
        .await
        .and_then(|_| Ok(Json(short_link)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application
//struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::MySql>);

#[async_trait]
impl<B> FromRequest<B> for MysqlRepo
    where
        B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<MySqlPool>::from_request(req)
            .await
            .map_err(internal_error)?;


        Ok(Self { pool })
    }
}


/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
