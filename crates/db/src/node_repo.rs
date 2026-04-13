use crate::schema::NodeRow;
use crate::DbPool;
use anyhow::Result;

pub async fn register_node(pool: &DbPool, node: &NodeRow) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO nodes (
            node_id, agent_url, region, labels, status, is_idle,
            cpu_available_pct, ram_available_mb, disk_available_gb,
            running_chunks, last_seen_epoch_secs
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (node_id) DO UPDATE SET
            agent_url = $2,
            region = $3,
            labels = $4,
            status = $5,
            is_idle = $6,
            cpu_available_pct = $7,
            ram_available_mb = $8,
            disk_available_gb = $9,
            running_chunks = $10,
            last_seen_epoch_secs = $11
        "#,
    )
    .bind(&node.node_id)
    .bind(&node.agent_url)
    .bind(&node.region)
    .bind(&node.labels)
    .bind(&node.status)
    .bind(node.is_idle)
    .bind(node.cpu_available_pct)
    .bind(node.ram_available_mb)
    .bind(node.disk_available_gb)
    .bind(node.running_chunks as i32)
    .bind(node.last_seen_epoch_secs as i64)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_node(pool: &DbPool, node_id: &str) -> Result<Option<NodeRow>> {
    let row = sqlx::query_as::<_, NodeRow>("SELECT * FROM nodes WHERE node_id = $1")
        .bind(node_id)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn list_all_nodes(pool: &DbPool) -> Result<Vec<NodeRow>> {
    let rows = sqlx::query_as::<_, NodeRow>("SELECT * FROM nodes ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub async fn find_idle_node(pool: &DbPool) -> Result<Option<NodeRow>> {
    let row = sqlx::query_as::<_, NodeRow>(
        r#"
        SELECT * FROM nodes
        WHERE is_idle = true
          AND (agent_url LIKE 'http://%' OR agent_url LIKE 'https://%')
        ORDER BY last_seen_epoch_secs DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn update_heartbeat(
    pool: &DbPool,
    node_id: &str,
    is_idle: bool,
    status: &str,
    cpu_pct: f32,
    ram_mb: i64,
    disk_gb: i64,
    running_chunks: i32,
    now_epoch_secs: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE nodes SET
            is_idle = $1,
            status = $2,
            cpu_available_pct = $3,
            ram_available_mb = $4,
            disk_available_gb = $5,
            running_chunks = $6,
            last_seen_epoch_secs = $7
        WHERE node_id = $8
        "#,
    )
    .bind(is_idle)
    .bind(status)
    .bind(cpu_pct)
    .bind(ram_mb)
    .bind(disk_gb)
    .bind(running_chunks)
    .bind(now_epoch_secs)
    .bind(node_id)
    .execute(pool)
    .await?;
    Ok(())
}
