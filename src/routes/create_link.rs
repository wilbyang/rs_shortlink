use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use crate::domain::short_link::ShortLink;
use crate::repo::MysqlRepo;

pub async fn save_link(
    Json(slink): Json<ShortLink>,
    repo: MysqlRepo,
) -> core::result::Result<impl IntoResponse, (StatusCode, String)> {
    let short_link = slink.clone();
    repo.save_link(slink)
        .await
        .and_then(|_| Ok(Json(short_link)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}