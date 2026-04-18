use super::schema::SettlementRow;
use super::DbPool;
use anyhow::Result;
use uuid::Uuid;

pub async fn create_settlement(
    pool: &DbPool,
    job_id: &str,
    user_wallet: &str,
    provider_wallet: Option<String>,
    network_id: &str,
    units_metered: i64,
    amount_tokens: i64,
    settlement_type: &str,
) -> Result<()> {
    let settlement_id = Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    sqlx::query(
        r#"
        INSERT INTO settlements (
            settlement_id, job_id, user_wallet, provider_wallet, network_id,
            units_metered, amount_tokens, tx_status, settlement_type, created_at_epoch_secs
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'Pending', $8, $9)
        "#,
    )
    .bind(&settlement_id)
    .bind(job_id)
    .bind(user_wallet)
    .bind(provider_wallet)
    .bind(network_id)
    .bind(units_metered)
    .bind(amount_tokens)
    .bind(settlement_type)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_settlement_tx(
    pool: &DbPool,
    settlement_id: &str,
    tx_hash: &str,
    tx_status: &str,
) -> Result<()> {
    sqlx::query("UPDATE settlements SET tx_hash = $1, tx_status = $2 WHERE settlement_id = $3")
        .bind(tx_hash)
        .bind(tx_status)
        .bind(settlement_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_settlement(pool: &DbPool, settlement_id: &str) -> Result<Option<SettlementRow>> {
    let row = sqlx::query_as::<_, SettlementRow>(
        "SELECT * FROM settlements WHERE settlement_id = $1",
    )
    .bind(settlement_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn list_job_settlements(pool: &DbPool, job_id: &str) -> Result<Vec<SettlementRow>> {
    let rows = sqlx::query_as::<_, SettlementRow>(
        "SELECT * FROM settlements WHERE job_id = $1 ORDER BY created_at_epoch_secs DESC",
    )
    .bind(job_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn list_user_settlements(
    pool: &DbPool,
    user_wallet: &str,
) -> Result<Vec<SettlementRow>> {
    let rows = sqlx::query_as::<_, SettlementRow>(
        "SELECT * FROM settlements WHERE user_wallet = $1 ORDER BY created_at_epoch_secs DESC",
    )
    .bind(user_wallet)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn list_pending_settlements(pool: &DbPool) -> Result<Vec<SettlementRow>> {
    let rows = sqlx::query_as::<_, SettlementRow>(
        "SELECT * FROM settlements WHERE tx_status = 'Pending' ORDER BY created_at_epoch_secs ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}