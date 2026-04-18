use super::schema::UserEntitlementRow;
use super::DbPool;
use anyhow::Result;
use uuid::Uuid;

pub async fn create_or_update_entitlement(
    pool: &DbPool,
    user_wallet: &str,
    network_id: &str,
    bought_units: i64,
    escrow_account: Option<String>,
    escrow_tx_hash: Option<String>,
    expiry_epoch_secs: Option<i64>,
) -> Result<()> {
    let entitlement_id = Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    sqlx::query(
        r#"
        INSERT INTO user_entitlements (
            entitlement_id, user_wallet, network_id, bought_units, used_units,
            escrow_account, escrow_tx_hash, expiry_epoch_secs, created_at_epoch_secs
        ) VALUES ($1, $2, $3, $4, 0, $5, $6, $7, $8)
        ON CONFLICT (user_wallet, network_id) DO UPDATE SET
            bought_units = bought_units + $4,
            escrow_account = COALESCE($5, escrow_account),
            escrow_tx_hash = COALESCE($6, escrow_tx_hash),
            expiry_epoch_secs = COALESCE($7, expiry_epoch_secs)
        "#,
    )
    .bind(&entitlement_id)
    .bind(user_wallet)
    .bind(network_id)
    .bind(bought_units)
    .bind(escrow_account)
    .bind(escrow_tx_hash)
    .bind(expiry_epoch_secs)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_entitlement(
    pool: &DbPool,
    user_wallet: &str,
    network_id: &str,
) -> Result<Option<UserEntitlementRow>> {
    let row = sqlx::query_as::<_, UserEntitlementRow>(
        "SELECT * FROM user_entitlements WHERE user_wallet = $1 AND network_id = $2",
    )
    .bind(user_wallet)
    .bind(network_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn check_quota(
    pool: &DbPool,
    user_wallet: &str,
    network_id: &str,
    units_needed: i64,
) -> Result<bool> {
    if let Some(ent) = get_entitlement(pool, user_wallet, network_id).await? {
        let remaining = ent.bought_units - ent.used_units;
        return Ok(remaining >= units_needed);
    }
    Ok(false)
}

pub async fn increment_usage(
    pool: &DbPool,
    user_wallet: &str,
    network_id: &str,
    units: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE user_entitlements SET used_units = used_units + $1 WHERE user_wallet = $2 AND network_id = $3",
    )
    .bind(units)
    .bind(user_wallet)
    .bind(network_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_user_entitlements(
    pool: &DbPool,
    user_wallet: &str,
) -> Result<Vec<UserEntitlementRow>> {
    let rows = sqlx::query_as::<_, UserEntitlementRow>(
        "SELECT * FROM user_entitlements WHERE user_wallet = $1 ORDER BY created_at_epoch_secs DESC",
    )
    .bind(user_wallet)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}