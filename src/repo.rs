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

#[cfg(test)]
mod tests {
    use crate::repo::MysqlRepo;
    use crate::domain::short_link::ShortLink;
    use sqlx::MySqlPool;
    use anyhow::Result;
    use sqlx::mysql::MySqlConnectOptions;
    use sqlx::mysql::MySqlSslMode;
    use sqlx::mysql::MySqlPoolOptions;
    use std::env;

    #[tokio::test]
    async fn test_serve_link() -> Result<()> {
        let pool = get_pool().await?;
        let repo = MysqlRepo::new(pool);
        let slink = repo.serve_link("test").await?;
        assert_eq!(slink, "https://www.google.com");
        Ok(())
    }

    #[tokio::test]
    async fn test_save_link() -> Result<()> {
        let pool = get_pool().await?;
        let repo = MysqlRepo::new(pool);
        let slink = ShortLink::new("test".to_string(), "https://www.google.com".to_string());
        repo.save_link(slink).await?;
        Ok(())
    }

    async fn get_pool() -> Result<MySqlPool> {
        let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await?;
        Ok(pool)
    }
}