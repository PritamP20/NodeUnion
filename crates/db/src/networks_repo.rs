use crate::schema::NetworkRow;
use crate::DbPool;
use anyhow::Result;

pub async fn create_network(pool: &DbPool, network: &NetworkRow) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO networks (network_id, name, description, status, created_at_epoch_secs)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (network_id) DO UPDATE SET
            name = $2,
            description = $3,
            status = $4
        "#,
    )
    .bind(&network.network_id)
    .bind(&network.name)
    .bind(&network.description)
    .bind(&network.status)
    .bind(network.created_at_epoch_secs)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_network(pool: &DbPool, network_id: &str) -> Result<Option<NetworkRow>> {
    let row = sqlx::query_as::<_, NetworkRow>("SELECT * FROM networks WHERE network_id = $1")
        .bind(network_id)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn list_networks(pool: &DbPool) -> Result<Vec<NetworkRow>> {
    let rows =
        sqlx::query_as::<_, NetworkRow>("SELECT * FROM networks ORDER BY created_at_epoch_secs DESC")
            .fetch_all(pool)
            .await?;

    Ok(rows)
}

pub async fn set_network_status(pool: &DbPool, network_id: &str, status: &str) -> Result<()> {
    sqlx::query("UPDATE networks SET status = $1 WHERE network_id = $2")
        .bind(status)
        .bind(network_id)
        .execute(pool)
        .await?;

    Ok(())
}
