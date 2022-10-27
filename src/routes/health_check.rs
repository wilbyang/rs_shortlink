
use axum::response::Json;

pub async fn health_check() -> Json<&'static str> {
    Json("Ok")
}