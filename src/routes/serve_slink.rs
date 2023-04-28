use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use crate::repo::MysqlRepo;

// we can extract the connection pool with `Extension`
pub async fn serve_slink(
    Path(slink): Path<String>,
    repo: MysqlRepo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    repo.serve_link(slink.as_str())
        .await
        .and_then(|dest: String| Ok(Redirect::to(dest.as_str())))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}