use super::schema::AttemptRow;
use super::DbPool;
use anyhow::Result;

pub async fn create_attempt(pool: &DbPool, attempt: &AttemptRow) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO attempts (
            attempt_id, job_id, attempt_number, assigned_node_id,
            last_error, next_retry_at_epoch_secs, created_at_epoch_secs
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(&attempt.attempt_id)
    .bind(&attempt.job_id)
    .bind(attempt.attempt_number)
    .bind(&attempt.assigned_node_id)
    .bind(&attempt.last_error)
    .bind(attempt.next_retry_at_epoch_secs)
    .bind(attempt.created_at_epoch_secs)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_attempts_for_job(pool: &DbPool, job_id: &str) -> Result<i64> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM attempts WHERE job_id = $1")
        .bind(job_id)
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}

pub async fn latest_attempt_for_job(pool: &DbPool, job_id: &str) -> Result<Option<AttemptRow>> {
    let row = sqlx::query_as::<_, AttemptRow>(
        r#"
        SELECT * FROM attempts
        WHERE job_id = $1
        ORDER BY attempt_number DESC
        LIMIT 1
        "#,
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}