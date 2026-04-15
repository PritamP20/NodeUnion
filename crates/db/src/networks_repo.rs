use crate::schema::NetworkRow;
use crate::DbPool;
use anyhow::Result;

pub async fn create_network(pool: &DbPool, network: &NetworkRow) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO networks (network_id, name, description, created_at_epoch_secs)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (network_id) DO UPDATE SET
            name = $2,
            description = $3
        "#,
    )
    .bind(&network.network_id)
    .bind(&network.name)
    .bind(&network.description)
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
