use dotenvy::dotenv;
use nodeunion_orchestrator::db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL is required (set Neon connection string)");

    let pool = db::new_pool(&database_url).await?;
    db::init_schema(&pool).await?;

    let rows: Vec<(String,)> = sqlx::query_as(
        "select table_name from information_schema.tables where table_schema='public' and table_name in ('networks','nodes','jobs','attempts','user_entitlements','settlements','provider_settlements') order by table_name",
    )
    .fetch_all(&pool)
    .await?;

    println!("schema migration complete. tables:");
    for (name,) in rows {
        println!("- {}", name);
    }

    Ok(())
}
