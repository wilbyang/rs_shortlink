use axum::{
    async_trait,
    extract::{Extension, FromRequest, Path, RequestParts},
    http::StatusCode,
    response::{Redirect, IntoResponse},
    routing::{get, post},
    Router, Json,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:123456@localhost:3308/mylinks".to_string());

    // setup connection pool
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_connection_str)
        .await
        .expect("can connect to database");

    // build our application with some routes
    let app = Router::new()
        .route("/s/:slink", get(serve_slink))
        .route("/s", post(upsert_slink))
        .layer(Extension(pool));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3002));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn upsert_slink(
    Json(slink): Json<SLink>,
    Extension(pool): Extension<MySqlPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let json = slink.clone();
    
    sqlx::query("INSERT INTO slinks (slink, dest) VALUES (?,?)")
        .bind(slink.slink)
        .bind(slink.dest)
        .execute(&pool)
        .await
        .and_then(|_| Ok(Json(json)))
        .map_err(internal_error)

}

// we can extract the connection pool with `Extension`
async fn serve_slink(
    Path(slink): Path<String>,
    Extension(pool): Extension<MySqlPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query_scalar("SELECT dest FROM slinks where slink = ?")
        .bind(slink)
        .fetch_one(&pool)
        .await
        .and_then(|dest: String| Ok(Redirect::to(dest.as_str())))
        .map_err(internal_error)
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
struct SLink {
    slink: String,
    dest: String,
}

// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application
struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::MySql>);

#[async_trait]
impl<B> FromRequest<B> for DatabaseConnection
where
    B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<MySqlPool>::from_request(req)
            .await
            .map_err(internal_error)?;

        let conn = pool.acquire().await.map_err(internal_error)?;

        Ok(Self(conn))
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
