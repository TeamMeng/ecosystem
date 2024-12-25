use anyhow::Result;
use deadpool_postgres::{
    tokio_postgres::NoTls, Config, GenericClient, ManagerConfig, RecyclingMethod, Runtime,
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.dbname = Some("backend".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("postgres".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    let client = pool.get().await?;

    let rows = client.query("SELECT * FROM urls", &[]).await?;

    for row in rows {
        let id: i64 = row.get(0);
        let short: String = row.get(1);
        let user_id: i64 = row.get(2);
        let url: String = row.get(3);
        println!("{:?}, {:?}, {:?}, {:?}", id, short, user_id, url);
    }

    Ok(())
}
