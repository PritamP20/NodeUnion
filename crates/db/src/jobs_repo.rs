use crate::schema::JobRow;
use crate::DbPool;
use anyhow::Result;

pub async fn create_job(pool: &DbPool, job: &JobRow) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO jobs (
            job_id, network_id, image, command, cpu_limit, ram_limit_mb, status, assigned_node_id, created_at_epoch_secs
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(&job.job_id)
    .bind(&job.network_id)
    .bind(&job.image)
    .bind(&job.command)
    .bind(job.cpu_limit)
    .bind(job.ram_limit_mb)
    .bind(&job.status)
    .bind(&job.assigned_node_id)
    .bind(job.created_at_epoch_secs)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_job(pool: &DbPool, job_id: &str) -> Result<Option<JobRow>> {
    let row = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs WHERE job_id = $1")
        .bind(job_id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_all_jobs(pool: &DbPool) -> Result<Vec<JobRow>> {
    let rows = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn list_pending_jobs(pool: &DbPool) -> Result<Vec<JobRow>> {
    list_pending_jobs_in_network(pool, "default").await
}

pub async fn list_pending_jobs_in_network(pool: &DbPool, network_id: &str) -> Result<Vec<JobRow>> {
    let rows = sqlx::query_as::<_, JobRow>(
        "SELECT * FROM jobs WHERE status = 'Pending' AND network_id = $1 ORDER BY created_at_epoch_secs ASC",
    )
    .bind(network_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn mark_job_scheduled(pool: &DbPool, job_id: &str, node_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE jobs
        SET status = 'Scheduled', assigned_node_id = $1
        WHERE job_id = $2
        "#
    )
    .bind(node_id)
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_job_running(pool: &DbPool, job_id: &str) -> Result<()> {
    sqlx::query("UPDATE jobs SET status = 'Running' WHERE job_id = $1")
        .bind(job_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_job_status(pool: &DbPool, job_id: &str, status: &str) -> Result<()> {
    sqlx::query("UPDATE jobs SET status = $1 WHERE job_id = $2")
        .bind(status)
        .bind(job_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn reset_job_to_pending(pool: &DbPool, job_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE jobs
        SET status = 'Pending', assigned_node_id = NULL
        WHERE job_id = $1
        "#,
    )
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}