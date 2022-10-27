use anyhow::Result;
use sqlx::{MySqlPool};
use sqlx::pool::PoolConnection;
use crate::domain::short_link::ShortLink;

pub struct MysqlRepo {
    pub pool: MySqlPool,
}


impl MysqlRepo {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
    pub async fn serve_link(&self, slink: &str) -> Result<String> {
        let ret = sqlx::query_scalar("SELECT dest FROM slinks where slink = ?")
            .bind(slink)
            .fetch_one(&self.pool)
            .await?;
        Ok(ret)
    }
    pub async fn save_link(&self, slink: ShortLink) -> Result<()> {
        sqlx::query("INSERT INTO slinks (slink, dest) VALUES (?,?)")
            .bind(slink.slink)
            .bind(slink.dest)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

}