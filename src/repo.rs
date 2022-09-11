use rand::{distributions::Alphanumeric, Rng};
use anyhow::{Result, Ok};
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", generate_random_string(10));
    
    Ok(())

    
    
}

async fn one_batch() -> Result<()> {
    let mut slinks: Vec<String> = Vec::new();
    for _ in 0..10000 {
        let slink = generate_random_string(10);
        slinks.push(slink);


    }
    slinks.dedup();

    
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:123456@localhost:3308/mylinks")
        .await?;
    let mut tx = pool.begin().await?;
    for slink in slinks {
        sqlx::query("INSERT INTO slinks (slink) VALUES (?)")
            .bind(slink)
            .execute(&mut tx)
            .await?;
    }
    tx.commit().await?;
    
    Ok(())
}



fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut s = String::with_capacity(length);
    for _ in 0..length {
        
        s.push(rng.sample(Alphanumeric) as char);
    }

    let sql_create_table = format!("CREATE TABLE IF NOT EXISTS {} (id INT NOT NULL AUTO_INCREMENT, slink VARCHAR(255) NOT NULL default, PRIMARY KEY (id))", s);

    s
}




mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}