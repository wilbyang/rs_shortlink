use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    routing::get,
    Router,
};
use sqlx::mysql::{MySqlPoolOptions, MySqlPool};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{net::SocketAddr, time::Duration};

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
        .route(
            "/",
            get(slink_direct).post(using_connection_extractor),
        )
        .layer(Extension(pool));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn using_connection_extractor(
    DatabaseConnection(conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
    let mut conn = conn;
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&mut conn)
        .await
        .map_err(internal_error)
}

// we can extract the connection pool with `Extension`
async fn slink_direct(
    Extension(pool): Extension<MySqlPool>,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("SELECT dest FROM slinks where slink = ?")
        .bind("IvShpROgPt")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)

    
}

#[derive(Debug, sqlx::FromRow)]
struct SLink{
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
