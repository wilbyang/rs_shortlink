
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct ShortLink {
    pub slink: String,
    pub dest: String,
}